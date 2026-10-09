use notify::{Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};
use tracing::{debug, error, info};

/// Events emitted by the filesystem watcher when external files change.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WatcherEvent {
    Created(PathBuf),
    Modified(PathBuf),
    Deleted(PathBuf),
    Renamed {
        old_path: PathBuf,
        new_path: PathBuf,
    },
}

/// Thread-safe suppression map to avoid reacting to Nodera's own writes.
#[derive(Debug, Clone, Default)]
pub struct WriteSuppressor {
    suppressed: Arc<Mutex<HashMap<PathBuf, Instant>>>,
}

impl WriteSuppressor {
    pub fn new() -> Self {
        Self {
            suppressed: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Suppresses events for `path` for the next `duration`.
    pub fn suppress(&self, path: &Path, duration: Duration) {
        if let Ok(mut map) = self.suppressed.lock() {
            map.insert(path.to_path_buf(), Instant::now() + duration);
        }
    }

    /// Checks if `path` is currently suppressed.
    pub fn is_suppressed(&self, path: &Path) -> bool {
        if let Ok(mut map) = self.suppressed.lock() {
            let now = Instant::now();
            // Prune expired entries
            map.retain(|_, expiry| *expiry > now);
            map.contains_key(path)
        } else {
            false
        }
    }
}

/// Filesystem watcher for monitoring external vault modifications.
pub struct VaultWatcher {
    _watcher: RecommendedWatcher,
    suppressor: WriteSuppressor,
}

impl VaultWatcher {
    /// Starts watching `vault_dir` recursively, streaming changes to the returned receiver.
    pub fn start(
        vault_dir: PathBuf,
    ) -> notify::Result<(Self, UnboundedReceiver<WatcherEvent>, WriteSuppressor)> {
        let (tx, rx) = unbounded_channel();
        let suppressor = WriteSuppressor::new();
        let suppressor_clone = suppressor.clone();

        let event_handler = move |res: notify::Result<Event>| match res {
            Ok(event) => {
                Self::handle_raw_event(event, &suppressor_clone, &tx);
            }
            Err(e) => {
                error!("Vault filesystem watcher error: {}", e);
            }
        };

        let mut watcher = RecommendedWatcher::new(event_handler, Config::default())?;
        watcher.watch(&vault_dir, RecursiveMode::Recursive)?;

        info!(
            "Vault filesystem watcher started for {}",
            vault_dir.display()
        );

        Ok((
            Self {
                _watcher: watcher,
                suppressor: suppressor.clone(),
            },
            rx,
            suppressor,
        ))
    }

    /// Dispatches a raw notify event to the event sender if not suppressed.
    fn handle_raw_event(
        event: Event,
        suppressor: &WriteSuppressor,
        tx: &UnboundedSender<WatcherEvent>,
    ) {
        use notify::event::{ModifyKind, RenameMode};

        // Check if event is a rename with Both paths
        if let EventKind::Modify(ModifyKind::Name(RenameMode::Both)) = event.kind {
            if event.paths.len() >= 2 {
                let old_p = &event.paths[0];
                let new_p = &event.paths[1];

                // Skip internal .nodera metadata folder
                let is_internal = old_p.components().any(|c| c.as_os_str() == ".nodera")
                    || new_p.components().any(|c| c.as_os_str() == ".nodera");
                if is_internal {
                    return;
                }

                // Check suppression
                if suppressor.is_suppressed(old_p) || suppressor.is_suppressed(new_p) {
                    debug!(
                        "Suppressed self-rename event from {} to {}",
                        old_p.display(),
                        new_p.display()
                    );
                    return;
                }

                let is_md_or_dir = |p: &Path| {
                    p.extension().and_then(|ext| ext.to_str()) == Some("md")
                        || p.is_dir()
                        || (p.extension().is_none() && !p.is_file())
                };

                if is_md_or_dir(old_p) || is_md_or_dir(new_p) {
                    let _ = tx.send(WatcherEvent::Renamed {
                        old_path: old_p.clone(),
                        new_path: new_p.clone(),
                    });
                    return;
                }
            }
        }

        for path in event.paths {
            // Skip internal .nodera metadata folder
            if path.components().any(|c| c.as_os_str() == ".nodera") {
                continue;
            }

            // Accept markdown files or directories (including newly created/deleted folder paths without extension)
            let is_md = path.extension().and_then(|ext| ext.to_str()) == Some("md");
            let is_dir = path.is_dir() || (path.extension().is_none() && !path.is_file());
            if !is_md && !is_dir {
                continue;
            }

            // Skip if suppressed due to internal write
            if suppressor.is_suppressed(&path) {
                debug!("Suppressed self-write event for {}", path.display());
                continue;
            }

            match event.kind {
                EventKind::Create(_) => {
                    let _ = tx.send(WatcherEvent::Created(path));
                }
                EventKind::Modify(_) => {
                    let _ = tx.send(WatcherEvent::Modified(path));
                }
                EventKind::Remove(_) => {
                    let _ = tx.send(WatcherEvent::Deleted(path));
                }
                _ => {}
            }
        }
    }

    /// Access the write suppressor to suppress internal writes.
    pub fn suppressor(&self) -> &WriteSuppressor {
        &self.suppressor
    }
}

/// Coalesces a slice of raw watcher events over a debounce window into a minimal, deduplicated sequence.
pub fn coalesce_events(events: Vec<WatcherEvent>) -> Vec<WatcherEvent> {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    enum PathState {
        Created,
        Modified,
        Deleted,
    }

    let mut state_map: HashMap<PathBuf, PathState> = HashMap::new();
    let mut renames: Vec<(PathBuf, PathBuf)> = Vec::new();

    for event in events {
        match event {
            WatcherEvent::Created(path) => match state_map.get(&path) {
                Some(PathState::Deleted) => {
                    state_map.insert(path, PathState::Modified);
                }
                _ => {
                    state_map.insert(path, PathState::Created);
                }
            },
            WatcherEvent::Modified(path) => match state_map.get(&path) {
                Some(PathState::Created) => {
                    // Keep as Created so the new file gets fully ingested
                }
                _ => {
                    state_map.insert(path, PathState::Modified);
                }
            },
            WatcherEvent::Deleted(path) => match state_map.get(&path) {
                Some(PathState::Created) => {
                    // Created and deleted within same debounce window -> ignore scratch file
                    state_map.remove(&path);
                }
                _ => {
                    state_map.insert(path, PathState::Deleted);
                }
            },
            WatcherEvent::Renamed { old_path, new_path } => {
                renames.push((old_path, new_path));
            }
        }
    }

    let mut result = Vec::new();
    for (old_path, new_path) in renames {
        result.push(WatcherEvent::Renamed { old_path, new_path });
    }
    for (path, state) in state_map {
        match state {
            PathState::Created => result.push(WatcherEvent::Created(path)),
            PathState::Modified => result.push(WatcherEvent::Modified(path)),
            PathState::Deleted => result.push(WatcherEvent::Deleted(path)),
        }
    }
    result
}

/// Receives a burst of watcher events within a debounce time window, returning a batched vector of events.
pub async fn next_debounced_batch(
    rx: &mut UnboundedReceiver<WatcherEvent>,
    debounce_duration: Duration,
) -> Option<Vec<WatcherEvent>> {
    let first = rx.recv().await?;
    let mut batch = vec![first];

    loop {
        tokio::select! {
            evt = rx.recv() => {
                match evt {
                    Some(e) => batch.push(e),
                    None => break,
                }
            }
            _ = tokio::time::sleep(debounce_duration) => {
                break;
            }
        }
    }

    Some(batch)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_suppressor_lifecycle() {
        let suppressor = WriteSuppressor::new();
        let path = PathBuf::from("test/note.md");

        assert!(!suppressor.is_suppressed(&path));

        // Suppress for 100ms
        suppressor.suppress(&path, Duration::from_millis(100));
        assert!(suppressor.is_suppressed(&path));

        // After expiry
        std::thread::sleep(Duration::from_millis(150));
        assert!(!suppressor.is_suppressed(&path));
    }

    #[tokio::test]
    async fn test_watcher_detects_external_file() {
        let dir = tempdir().unwrap();
        let vault_path = dir.path().to_path_buf();

        let (_watcher, mut rx, _suppressor) = VaultWatcher::start(vault_path.clone()).unwrap();

        // Create a new markdown file externally
        let note_path = vault_path.join("External.md");
        fs::write(&note_path, "# External Note\nContent").unwrap();

        // Expect to receive Created or Modified event
        let event = tokio::time::timeout(Duration::from_secs(3), rx.recv()).await;
        assert!(event.is_ok(), "Timed out waiting for watcher event");
        let received = event.unwrap();
        assert!(received.is_some());
    }

    #[tokio::test]
    async fn test_watcher_detects_external_directory() {
        let dir = tempdir().unwrap();
        let vault_path = dir.path().to_path_buf();

        let (_watcher, mut rx, _suppressor) = VaultWatcher::start(vault_path.clone()).unwrap();

        // Create a new directory externally
        let folder_path = vault_path.join("SubFolder");
        fs::create_dir(&folder_path).unwrap();

        let event = tokio::time::timeout(Duration::from_secs(3), rx.recv()).await;
        assert!(
            event.is_ok(),
            "Timed out waiting for watcher directory event"
        );
        let received = event.unwrap();
        assert!(received.is_some());
    }

    #[test]
    fn test_coalesce_duplicate_modify_events() {
        let p1 = PathBuf::from("Note1.md");
        let p2 = PathBuf::from("Note2.md");

        let events = vec![
            WatcherEvent::Modified(p1.clone()),
            WatcherEvent::Modified(p1.clone()),
            WatcherEvent::Modified(p1.clone()),
            WatcherEvent::Created(p2.clone()),
            WatcherEvent::Modified(p2.clone()),
        ];

        let coalesced = coalesce_events(events);
        assert_eq!(coalesced.len(), 2);
        assert!(coalesced.contains(&WatcherEvent::Modified(p1)));
        assert!(coalesced.contains(&WatcherEvent::Created(p2)));
    }

    #[test]
    fn test_coalesce_created_and_deleted_transient_file() {
        let p = PathBuf::from("scratch.tmp.md");
        let events = vec![
            WatcherEvent::Created(p.clone()),
            WatcherEvent::Modified(p.clone()),
            WatcherEvent::Deleted(p.clone()),
        ];

        let coalesced = coalesce_events(events);
        assert!(coalesced.is_empty());
    }

    #[tokio::test]
    async fn test_next_debounced_batch() {
        let (tx, mut rx) = unbounded_channel();
        let p = PathBuf::from("Note.md");

        tx.send(WatcherEvent::Created(p.clone())).unwrap();
        tx.send(WatcherEvent::Modified(p.clone())).unwrap();

        let batch = next_debounced_batch(&mut rx, Duration::from_millis(50))
            .await
            .unwrap();

        assert_eq!(batch.len(), 2);
        let coalesced = coalesce_events(batch);
        assert_eq!(coalesced, vec![WatcherEvent::Created(p)]);
    }
}
