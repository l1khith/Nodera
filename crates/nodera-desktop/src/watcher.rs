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
}

/// Thread-safe suppression map to avoid reacting to Nodera's own writes.
#[derive(Clone, Default)]
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
        for path in event.paths {
            // Only care about markdown files
            if path.extension().and_then(|ext| ext.to_str()) != Some("md") {
                continue;
            }

            // Skip internal .nodera metadata folder
            if path.components().any(|c| c.as_os_str() == ".nodera") {
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
}
