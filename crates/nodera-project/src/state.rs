use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::error::{ProjectError, Result};

/// Stored file metadata record tracking size, mtime, and content hash.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FileStateRecord {
    pub mtime_secs: u64,
    pub size_bytes: u64,
    pub hash: String,
}

/// Project-wide source state persisted at `.nodera/state/source_state.json`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceState {
    pub version: u32,
    pub last_scan: String,
    pub manifest_hash: Option<String>,
    pub files: HashMap<String, FileStateRecord>,
}

impl Default for SourceState {
    fn default() -> Self {
        Self {
            version: 1,
            last_scan: chrono::Utc::now().to_rfc3339(),
            manifest_hash: None,
            files: HashMap::new(),
        }
    }
}

/// Computed differential changes between disk and previous source state.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ChangeSet {
    pub added: Vec<PathBuf>,
    pub modified: Vec<PathBuf>,
    pub deleted: Vec<String>,
    pub manifest_changed: bool,
}

impl ChangeSet {
    pub fn is_empty(&self) -> bool {
        self.added.is_empty()
            && self.modified.is_empty()
            && self.deleted.is_empty()
            && !self.manifest_changed
    }

    pub fn total_changes(&self) -> usize {
        self.added.len()
            + self.modified.len()
            + self.deleted.len()
            + if self.manifest_changed { 1 } else { 0 }
    }
}

impl SourceState {
    /// Loads source state from `.nodera/state/source_state.json`.
    pub fn load_from_file(path: &Path) -> Result<Self> {
        if !path.exists() {
            return Ok(Self::default());
        }

        let content = fs::read_to_string(path).map_err(|e| ProjectError::Io {
            path: path.to_path_buf(),
            source: e,
        })?;

        serde_json::from_str(&content).map_err(|e| ProjectError::CorruptedState {
            path: path.to_path_buf(),
            reason: format!("Failed to parse source_state.json: {e}"),
        })
    }

    /// Saves source state to `.nodera/state/source_state.json`.
    pub fn save_to_file(&self, path: &Path) -> Result<()> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|e| ProjectError::Io {
                path: parent.to_path_buf(),
                source: e,
            })?;
        }

        let json = serde_json::to_string_pretty(self).map_err(|e| ProjectError::Serialization {
            reason: format!("Failed to serialize source state: {e}"),
        })?;

        fs::write(path, json).map_err(|e| ProjectError::Io {
            path: path.to_path_buf(),
            source: e,
        })
    }

    /// Detects changes between filesystem files and recorded source state.
    pub fn detect_changes(
        project_root: &Path,
        current_files: &[PathBuf],
        manifest_path: &Path,
        prev_state: &SourceState,
    ) -> Result<ChangeSet> {
        let mut added = Vec::new();
        let mut modified = Vec::new();
        let mut current_rel_paths = HashSet::new();

        for file_path in current_files {
            let rel = file_path
                .strip_prefix(project_root)
                .unwrap_or(file_path)
                .to_string_lossy()
                .replace('\\', "/");

            current_rel_paths.insert(rel.clone());

            let metadata = fs::metadata(file_path).map_err(|e| ProjectError::Io {
                path: file_path.clone(),
                source: e,
            })?;

            let mtime_secs = metadata
                .modified()
                .ok()
                .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
                .map(|d| d.as_secs())
                .unwrap_or(0);

            let size_bytes = metadata.len();

            if let Some(recorded) = prev_state.files.get(&rel) {
                // If timestamp or size differs, check content hash
                if recorded.size_bytes != size_bytes || recorded.mtime_secs != mtime_secs {
                    let new_hash = compute_file_hash(file_path)?;
                    if new_hash != recorded.hash {
                        modified.push(file_path.clone());
                    }
                }
            } else {
                added.push(file_path.clone());
            }
        }

        // Deleted files: in recorded state but no longer on disk
        let mut deleted = Vec::new();
        for rel in prev_state.files.keys() {
            if !current_rel_paths.contains(rel) {
                deleted.push(rel.clone());
            }
        }

        // Check if Cargo.toml changed
        let manifest_changed = if manifest_path.exists() {
            let current_manifest_hash = compute_file_hash(manifest_path).ok();
            current_manifest_hash != prev_state.manifest_hash
        } else {
            false
        };

        Ok(ChangeSet {
            added,
            modified,
            deleted,
            manifest_changed,
        })
    }
}

/// Computes a hex SHA-256 hash of a file's contents.
pub fn compute_file_hash(path: &Path) -> Result<String> {
    let bytes = fs::read(path).map_err(|e| ProjectError::Io {
        path: path.to_path_buf(),
        source: e,
    })?;

    let mut hasher = Sha256::new();
    hasher.update(&bytes);
    Ok(format!("{:x}", hasher.finalize()))
}
