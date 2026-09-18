use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;
use tracing::{debug, info, instrument};

use crate::error::{FileError, Result, ValidationError};
use crate::fs::{atomic_write_str, delete_file, read_to_string, validate_filename};
use crate::note::{Note, NoteId, NoteSummary, VaultEntry};
use crate::vault::Vault;

/// Application service coordinating vault file operations.
/// UI and higher-level use cases interact with VaultService rather than raw filesystem APIs.
#[derive(Debug, Clone)]
pub struct VaultService {
    vault: Vault,
}

impl VaultService {
    pub fn new(vault: Vault) -> Self {
        Self { vault }
    }

    pub fn vault(&self) -> &Vault {
        &self.vault
    }

    /// Recursively lists all vault folders and Markdown notes, skipping hidden directories like `.nodera`.
    #[instrument(skip(self))]
    pub fn list_entries(&self) -> Result<Vec<VaultEntry>> {
        let mut entries = Vec::new();
        self.collect_entries(self.vault.root(), Path::new(""), &mut entries)?;
        entries.sort_by(compare_vault_entries);
        Ok(entries)
    }

    fn collect_entries(
        &self,
        current_dir: &Path,
        rel_prefix: &Path,
        entries: &mut Vec<VaultEntry>,
    ) -> Result<()> {
        let read_dir = fs::read_dir(current_dir).map_err(|source| FileError::Io {
            path: current_dir.to_path_buf(),
            source,
        })?;

        for entry in read_dir {
            let entry = entry.map_err(|source| FileError::Io {
                path: current_dir.to_path_buf(),
                source,
            })?;

            let path = entry.path();
            let file_name = entry.file_name();
            let name_str = file_name.to_string_lossy();

            // Ignore hidden files and directories (e.g. .nodera, .git)
            if name_str.starts_with('.') {
                continue;
            }

            let rel_path = rel_prefix.join(&file_name);

            if path.is_dir() {
                entries.push(VaultEntry::Folder {
                    name: name_str.to_string(),
                    relative_path: rel_path.clone(),
                });
                self.collect_entries(&path, &rel_path, entries)?;
            } else if path.is_file() {
                if let Some(ext) = path.extension() {
                    if ext.eq_ignore_ascii_case("md") {
                        let metadata = entry.metadata().map_err(|source| FileError::Io {
                            path: path.clone(),
                            source,
                        })?;

                        let modified_at = metadata
                            .modified()
                            .ok()
                            .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
                            .map(|d| d.as_millis() as u64)
                            .unwrap_or(0);

                        let title = path
                            .file_stem()
                            .and_then(|s| s.to_str())
                            .unwrap_or("Untitled")
                            .to_string();

                        entries.push(VaultEntry::Note(NoteSummary {
                            id: NoteId::new(),
                            relative_path: rel_path,
                            title,
                            size_bytes: metadata.len(),
                            modified_at_millis: modified_at,
                        }));
                    }
                }
            }
        }

        Ok(())
    }
}

fn compare_vault_entries(a: &VaultEntry, b: &VaultEntry) -> std::cmp::Ordering {
    use std::cmp::Ordering;
    let comps_a: Vec<_> = a.relative_path().components().collect();
    let comps_b: Vec<_> = b.relative_path().components().collect();
    let len_a = comps_a.len();
    let len_b = comps_b.len();
    let min_len = len_a.min(len_b);

    for i in 0..min_len {
        let ca = comps_a[i].as_os_str().to_string_lossy().to_lowercase();
        let cb = comps_b[i].as_os_str().to_string_lossy().to_lowercase();
        if ca != cb {
            let is_dir_a = (i < len_a - 1) || a.is_folder();
            let is_dir_b = (i < len_b - 1) || b.is_folder();
            if is_dir_a != is_dir_b {
                return if is_dir_a {
                    Ordering::Less
                } else {
                    Ordering::Greater
                };
            }
            return ca.cmp(&cb);
        }
    }

    match len_a.cmp(&len_b) {
        Ordering::Less => Ordering::Less,
        Ordering::Greater => Ordering::Greater,
        Ordering::Equal => match (a, b) {
            (VaultEntry::Folder { .. }, VaultEntry::Note(_)) => Ordering::Less,
            (VaultEntry::Note(_), VaultEntry::Folder { .. }) => Ordering::Greater,
            _ => Ordering::Equal,
        },
    }
}

impl VaultService {
    /// Title is validated for illegal characters. If no folder is specified, uses vault default.
    #[instrument(skip(self, content))]
    pub fn create_note(
        &self,
        folder: Option<&str>,
        title: &str,
        content: Option<&str>,
    ) -> Result<Note> {
        let trimmed_title = title.trim();
        validate_filename(trimmed_title)?;

        // Ensure title doesn't end with .md already when generating filename
        let file_stem = if let Some(stripped) = trimmed_title.strip_suffix(".md") {
            stripped
        } else {
            trimmed_title
        };

        let file_name = format!("{file_stem}.md");
        let target_folder = folder.unwrap_or(&self.vault.config().default_folder);

        let relative_path = if target_folder.is_empty() {
            PathBuf::from(&file_name)
        } else {
            PathBuf::from(target_folder).join(&file_name)
        };

        let full_path = self.vault.resolve_path(&relative_path)?;

        if full_path.exists() {
            return Err(FileError::AlreadyExists { path: full_path }.into());
        }

        let initial_content = content.unwrap_or("");
        atomic_write_str(&full_path, initial_content)?;

        info!(path = %relative_path.display(), title = %file_stem, "Created new note");

        Ok(Note {
            id: NoteId::new(),
            relative_path,
            title: file_stem.to_string(),
            content: initial_content.to_string(),
        })
    }

    /// Reads a note from the vault by its relative path.
    #[instrument(skip(self, relative_path))]
    pub fn read_note(&self, relative_path: impl AsRef<Path>) -> Result<Note> {
        let rel = relative_path.as_ref();
        let full_path = self.vault.resolve_path(rel)?;

        let content = read_to_string(&full_path)?;
        let title = rel
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("Untitled")
            .to_string();

        debug!(path = %rel.display(), "Read note content");
        Ok(Note {
            id: NoteId::new(),
            relative_path: rel.to_path_buf(),
            title,
            content,
        })
    }

    /// Writes/saves updated content into a note atomically.
    #[instrument(skip(self, relative_path, content))]
    pub fn write_note(&self, relative_path: impl AsRef<Path>, content: &str) -> Result<Note> {
        let rel = relative_path.as_ref();
        let full_path = self.vault.resolve_path(rel)?;

        atomic_write_str(&full_path, content)?;

        let title = rel
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("Untitled")
            .to_string();

        debug!(path = %rel.display(), bytes = content.len(), "Saved note");
        Ok(Note {
            id: NoteId::new(),
            relative_path: rel.to_path_buf(),
            title,
            content: content.to_string(),
        })
    }

    /// Renames a note file in the vault, preserving its directory location.
    #[instrument(skip(self, relative_path))]
    pub fn rename_note(&self, relative_path: impl AsRef<Path>, new_title: &str) -> Result<Note> {
        let old_rel = relative_path.as_ref();
        let old_full = self.vault.resolve_path(old_rel)?;

        if !old_full.exists() {
            return Err(FileError::NotFound { path: old_full }.into());
        }

        let trimmed_new = new_title.trim();
        validate_filename(trimmed_new)?;

        let new_stem = if let Some(stripped) = trimmed_new.strip_suffix(".md") {
            stripped
        } else {
            trimmed_new
        };
        let new_filename = format!("{new_stem}.md");

        let parent = old_rel.parent().unwrap_or_else(|| Path::new(""));
        let new_rel = parent.join(&new_filename);
        let new_full = self.vault.resolve_path(&new_rel)?;

        if new_full.exists() && new_full != old_full {
            return Err(FileError::AlreadyExists { path: new_full }.into());
        }

        fs::rename(&old_full, &new_full).map_err(|source| FileError::Io {
            path: new_full.clone(),
            source,
        })?;

        let content = read_to_string(&new_full)?;
        info!(old = %old_rel.display(), new = %new_rel.display(), "Renamed note");

        Ok(Note {
            id: NoteId::new(),
            relative_path: new_rel,
            title: new_stem.to_string(),
            content,
        })
    }

    /// Moves a note to a new target folder within the vault.
    #[instrument(skip(self, relative_path, target_folder))]
    pub fn move_note(
        &self,
        relative_path: impl AsRef<Path>,
        target_folder: impl AsRef<Path>,
    ) -> Result<Note> {
        let old_rel = relative_path.as_ref();
        let old_full = self.vault.resolve_path(old_rel)?;

        if !old_full.exists() {
            return Err(FileError::NotFound { path: old_full }.into());
        }

        let file_name = old_rel
            .file_name()
            .ok_or_else(|| ValidationError::InvalidPath {
                path: old_rel.to_path_buf(),
                reason: "Cannot move file without filename".to_string(),
            })?;

        let target_rel_dir = target_folder.as_ref();
        let target_rel_file = target_rel_dir.join(file_name);
        let target_full_file = self.vault.resolve_path(&target_rel_file)?;

        if target_full_file.exists() && target_full_file != old_full {
            return Err(FileError::AlreadyExists {
                path: target_full_file,
            }
            .into());
        }

        if let Some(target_dir) = target_full_file.parent() {
            if !target_dir.exists() {
                fs::create_dir_all(target_dir).map_err(|source| FileError::Io {
                    path: target_dir.to_path_buf(),
                    source,
                })?;
            }
        }

        fs::rename(&old_full, &target_full_file).map_err(|source| FileError::Io {
            path: target_full_file.clone(),
            source,
        })?;

        let content = read_to_string(&target_full_file)?;
        let title = target_rel_file
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("Untitled")
            .to_string();

        info!(old = %old_rel.display(), new = %target_rel_file.display(), "Moved note");

        Ok(Note {
            id: NoteId::new(),
            relative_path: target_rel_file,
            title,
            content,
        })
    }

    /// Deletes a note from the vault permanently.
    #[instrument(skip(self, relative_path))]
    pub fn delete_note(&self, relative_path: impl AsRef<Path>) -> Result<()> {
        let rel = relative_path.as_ref();
        let full_path = self.vault.resolve_path(rel)?;

        delete_file(&full_path)?;
        info!(path = %rel.display(), "Deleted note permanently");
        Ok(())
    }

    /// Returns the absolute path to the vault's `.trash` directory.
    pub fn trash_dir(&self) -> PathBuf {
        self.vault.root().join(".trash")
    }

    /// Moves a note into the vault's `.trash/` directory and records it in `.trash/.manifest.json`.
    #[instrument(skip(self, relative_path))]
    pub fn trash_note(&self, relative_path: impl AsRef<Path>) -> Result<TrashedNoteSummary> {
        let rel = relative_path.as_ref();
        let full_path = self.vault.resolve_path(rel)?;
        if !full_path.exists() {
            return Err(FileError::NotFound { path: full_path }.into());
        }

        let trash_dir = self.trash_dir();
        fs::create_dir_all(&trash_dir).map_err(|source| FileError::Io {
            path: trash_dir.clone(),
            source,
        })?;

        let now = std::time::SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0);

        let stem = rel.file_stem().and_then(|s| s.to_str()).unwrap_or("note");
        let trash_filename = format!("{stem}_{now}.md");
        let trash_full_path = trash_dir.join(&trash_filename);

        let metadata = fs::metadata(&full_path).map_err(|source| FileError::Io {
            path: full_path.clone(),
            source,
        })?;
        let size_bytes = metadata.len();

        if fs::rename(&full_path, &trash_full_path).is_err() {
            fs::copy(&full_path, &trash_full_path).map_err(|source| FileError::Io {
                path: full_path.clone(),
                source,
            })?;
            let _ = fs::remove_file(&full_path);
        }

        let summary = TrashedNoteSummary {
            trash_filename: trash_filename.clone(),
            original_path: rel.to_path_buf(),
            title: stem.to_string(),
            trashed_at_millis: now,
            size_bytes,
        };

        let mut manifest = self.read_trash_manifest()?;
        manifest.retain(|item| item.trash_filename != trash_filename);
        manifest.insert(0, summary.clone());
        self.write_trash_manifest(&manifest)?;

        info!(path = %rel.display(), trash_file = %trash_filename, "Moved note to trash");
        Ok(summary)
    }

    /// Restores a previously trashed note back to its original location (or a non-colliding path).
    #[instrument(skip(self))]
    pub fn restore_note(&self, trash_filename: &str) -> Result<Note> {
        let mut manifest = self.read_trash_manifest()?;
        let pos = manifest
            .iter()
            .position(|m| m.trash_filename == trash_filename);
        let summary = match pos {
            Some(idx) => manifest.remove(idx),
            None => {
                let trash_file = self.trash_dir().join(trash_filename);
                if !trash_file.exists() {
                    return Err(FileError::NotFound { path: trash_file }.into());
                }
                TrashedNoteSummary {
                    trash_filename: trash_filename.to_string(),
                    original_path: PathBuf::from(trash_filename),
                    title: Path::new(trash_filename)
                        .file_stem()
                        .and_then(|s| s.to_str())
                        .unwrap_or("Restored Note")
                        .to_string(),
                    trashed_at_millis: 0,
                    size_bytes: 0,
                }
            }
        };

        let trash_full_path = self.trash_dir().join(&summary.trash_filename);
        if !trash_full_path.exists() {
            return Err(FileError::NotFound {
                path: trash_full_path,
            }
            .into());
        }

        let mut target_rel = summary.original_path.clone();
        let mut target_full = self.vault.resolve_path(&target_rel)?;

        if target_full.exists() {
            let stem = target_rel
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("note");
            let new_name = format!("{stem}_restored.md");
            target_rel = match target_rel.parent() {
                Some(p) if !p.as_os_str().is_empty() => p.join(new_name),
                _ => PathBuf::from(new_name),
            };
            target_full = self.vault.resolve_path(&target_rel)?;
        }

        if let Some(parent) = target_full.parent() {
            fs::create_dir_all(parent).map_err(|source| FileError::Io {
                path: parent.to_path_buf(),
                source,
            })?;
        }

        if fs::rename(&trash_full_path, &target_full).is_err() {
            fs::copy(&trash_full_path, &target_full).map_err(|source| FileError::Io {
                path: trash_full_path.clone(),
                source,
            })?;
            let _ = fs::remove_file(&trash_full_path);
        }

        self.write_trash_manifest(&manifest)?;
        info!(from = %summary.trash_filename, to = %target_rel.display(), "Restored note from trash");

        self.read_note(&target_rel)
    }

    /// Lists all currently trashed notes.
    pub fn list_trash(&self) -> Result<Vec<TrashedNoteSummary>> {
        self.read_trash_manifest()
    }

    /// Empties all contents of `.trash/`.
    pub fn empty_trash(&self) -> Result<usize> {
        let trash_dir = self.trash_dir();
        if !trash_dir.exists() {
            return Ok(0);
        }

        let mut count = 0;
        let read_dir = fs::read_dir(&trash_dir).map_err(|source| FileError::Io {
            path: trash_dir.clone(),
            source,
        })?;

        for entry in read_dir.flatten() {
            let p = entry.path();
            if p.is_file() {
                let _ = fs::remove_file(&p);
                count += 1;
            } else if p.is_dir() {
                let _ = fs::remove_dir_all(&p);
                count += 1;
            }
        }

        info!(deleted_items = count, "Emptied trash");
        Ok(count)
    }

    /// Permanently deletes a single file from the trash directory.
    pub fn delete_permanently(&self, trash_filename: &str) -> Result<()> {
        let trash_file = self.trash_dir().join(trash_filename);
        if trash_file.exists() {
            fs::remove_file(&trash_file).map_err(|source| FileError::Io {
                path: trash_file.clone(),
                source,
            })?;
        }

        let mut manifest = self.read_trash_manifest()?;
        manifest.retain(|m| m.trash_filename != trash_filename);
        self.write_trash_manifest(&manifest)?;
        Ok(())
    }

    fn read_trash_manifest(&self) -> Result<Vec<TrashedNoteSummary>> {
        let manifest_path = self.trash_dir().join(".manifest.json");
        if !manifest_path.exists() {
            return Ok(Vec::new());
        }
        let data = fs::read_to_string(&manifest_path).unwrap_or_default();
        let items: Vec<TrashedNoteSummary> = serde_json::from_str(&data).unwrap_or_default();
        Ok(items)
    }

    fn write_trash_manifest(&self, items: &[TrashedNoteSummary]) -> Result<()> {
        let trash_dir = self.trash_dir();
        if !trash_dir.exists() {
            fs::create_dir_all(&trash_dir).map_err(|source| FileError::Io {
                path: trash_dir.clone(),
                source,
            })?;
        }
        let manifest_path = trash_dir.join(".manifest.json");
        let json = serde_json::to_string_pretty(items).unwrap_or_default();
        atomic_write_str(&manifest_path, &json)?;
        Ok(())
    }
}

/// Metadata summary of a note stored in the vault's `.trash` directory.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TrashedNoteSummary {
    pub trash_filename: String,
    pub original_path: PathBuf,
    pub title: String,
    pub trashed_at_millis: u64,
    pub size_bytes: u64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_create_and_read_note() {
        let tmp = tempdir().unwrap();
        let vault = Vault::create(tmp.path().join("Vault"), None).unwrap();
        let service = VaultService::new(vault);

        let note = service
            .create_note(
                Some("Notes"),
                "Meeting Notes",
                Some("# Today's Meeting\n- item 1"),
            )
            .unwrap();

        assert_eq!(note.title, "Meeting Notes");
        assert_eq!(note.relative_path, PathBuf::from("Notes/Meeting Notes.md"));
        assert_eq!(note.content, "# Today's Meeting\n- item 1");

        // Read it back
        let read = service.read_note("Notes/Meeting Notes.md").unwrap();
        assert_eq!(read.title, "Meeting Notes");
        assert_eq!(read.content, "# Today's Meeting\n- item 1");
    }

    #[test]
    fn test_duplicate_create_fails() {
        let tmp = tempdir().unwrap();
        let vault = Vault::create(tmp.path().join("Vault"), None).unwrap();
        let service = VaultService::new(vault);

        let _ = service
            .create_note(Some("Notes"), "Duplicate", None)
            .unwrap();
        let second = service.create_note(Some("Notes"), "Duplicate", None);
        assert!(second.is_err());
    }

    #[test]
    fn test_write_and_update_note() {
        let tmp = tempdir().unwrap();
        let vault = Vault::create(tmp.path().join("Vault"), None).unwrap();
        let service = VaultService::new(vault);

        let note = service
            .create_note(None, "Editable", Some("initial"))
            .unwrap();
        assert_eq!(note.content, "initial");

        let updated = service
            .write_note(&note.relative_path, "updated content")
            .unwrap();
        assert_eq!(updated.content, "updated content");

        let read = service.read_note(&note.relative_path).unwrap();
        assert_eq!(read.content, "updated content");
    }

    #[test]
    fn test_rename_note() {
        let tmp = tempdir().unwrap();
        let vault = Vault::create(tmp.path().join("Vault"), None).unwrap();
        let service = VaultService::new(vault);

        let note = service
            .create_note(Some("Notes"), "Old Name", Some("hello"))
            .unwrap();
        let renamed = service
            .rename_note(&note.relative_path, "New Name")
            .unwrap();

        assert_eq!(renamed.title, "New Name");
        assert_eq!(renamed.relative_path, PathBuf::from("Notes/New Name.md"));

        // Old path no longer exists
        assert!(service.read_note("Notes/Old Name.md").is_err());
        // New path exists and has content
        let read = service.read_note("Notes/New Name.md").unwrap();
        assert_eq!(read.content, "hello");
    }

    #[test]
    fn test_move_note() {
        let tmp = tempdir().unwrap();
        let vault = Vault::create(tmp.path().join("Vault"), None).unwrap();
        let service = VaultService::new(vault);

        let note = service
            .create_note(Some("Notes"), "MoveMe", Some("test"))
            .unwrap();
        let moved = service.move_note(&note.relative_path, "Projects").unwrap();

        assert_eq!(moved.relative_path, PathBuf::from("Projects/MoveMe.md"));
        assert!(service.read_note("Notes/MoveMe.md").is_err());
        assert!(service.read_note("Projects/MoveMe.md").is_ok());
    }

    #[test]
    fn test_delete_note() {
        let tmp = tempdir().unwrap();
        let vault = Vault::create(tmp.path().join("Vault"), None).unwrap();
        let service = VaultService::new(vault);

        let note = service.create_note(None, "DeleteMe", None).unwrap();
        assert!(service.read_note(&note.relative_path).is_ok());

        service.delete_note(&note.relative_path).unwrap();
        assert!(service.read_note(&note.relative_path).is_err());
    }

    #[test]
    fn test_list_entries() {
        let tmp = tempdir().unwrap();
        let vault = Vault::create(tmp.path().join("Vault"), None).unwrap();
        let service = VaultService::new(vault);

        service.create_note(Some("Notes"), "Note A", None).unwrap();
        service.create_note(Some("Notes"), "Note B", None).unwrap();
        service
            .create_note(Some("Projects"), "Project Plan", None)
            .unwrap();

        let entries = service.list_entries().unwrap();
        let note_names: Vec<&str> = entries
            .iter()
            .filter_map(|e| match e {
                VaultEntry::Note(s) => Some(s.title.as_str()),
                _ => None,
            })
            .collect();

        assert!(note_names.contains(&"Note A"));
        assert!(note_names.contains(&"Note B"));
        assert!(note_names.contains(&"Project Plan"));
    }

    #[test]
    fn test_hierarchical_sorting() {
        let tmp = tempdir().unwrap();
        let vault = Vault::create(tmp.path().join("Vault"), None).unwrap();
        let service = VaultService::new(vault);

        service.create_note(Some(""), "RootNote", None).unwrap();
        service
            .create_note(Some("Books"), "The Intelligent Investor", None)
            .unwrap();
        service.create_note(Some("Notes"), "Note A", None).unwrap();

        let entries = service.list_entries().unwrap();
        let paths: Vec<String> = entries
            .iter()
            .map(|e| e.relative_path().to_string_lossy().replace('\\', "/"))
            .collect();

        assert_eq!(
            paths,
            vec![
                "Attachments",
                "Books",
                "Books/The Intelligent Investor.md",
                "Notes",
                "Notes/Note A.md",
                "Projects",
                "RootNote.md"
            ]
        );
    }

    #[test]
    fn test_soft_delete_and_restore() {
        let tmp = tempdir().unwrap();
        let vault = Vault::create(tmp.path().join("Vault"), None).unwrap();
        let service = VaultService::new(vault);

        let note = service
            .create_note(Some("Notes"), "Important Idea", Some("Contents of idea"))
            .unwrap();

        // 1. Verify note is in list_entries
        let entries = service.list_entries().unwrap();
        assert!(entries
            .iter()
            .any(|e| e.relative_path() == note.relative_path));

        // 2. Soft delete / move to trash
        let trashed = service.trash_note(&note.relative_path).unwrap();
        assert_eq!(trashed.title, "Important Idea");
        assert_eq!(trashed.original_path, note.relative_path);

        // 3. Verify note is no longer in list_entries
        let entries_after = service.list_entries().unwrap();
        assert!(!entries_after
            .iter()
            .any(|e| e.relative_path() == note.relative_path));

        // 4. Verify trash listing
        let trash_list = service.list_trash().unwrap();
        assert_eq!(trash_list.len(), 1);
        assert_eq!(trash_list[0].trash_filename, trashed.trash_filename);

        // 5. Restore note
        let restored = service.restore_note(&trashed.trash_filename).unwrap();
        assert_eq!(restored.title, "Important Idea");
        assert_eq!(restored.content, "Contents of idea");

        // 6. Verify restored note is back in list_entries and trash is empty
        let entries_restored = service.list_entries().unwrap();
        assert!(entries_restored
            .iter()
            .any(|e| e.relative_path() == note.relative_path));
        let trash_after = service.list_trash().unwrap();
        assert!(trash_after.is_empty());
    }

    #[test]
    fn test_empty_trash() {
        let tmp = tempdir().unwrap();
        let vault = Vault::create(tmp.path().join("Vault"), None).unwrap();
        let service = VaultService::new(vault);

        let note1 = service.create_note(Some("Notes"), "N1", None).unwrap();
        let note2 = service.create_note(Some("Notes"), "N2", None).unwrap();

        service.trash_note(&note1.relative_path).unwrap();
        service.trash_note(&note2.relative_path).unwrap();

        let trash_list = service.list_trash().unwrap();
        assert_eq!(trash_list.len(), 2);

        let deleted = service.empty_trash().unwrap();
        assert!(deleted >= 2);

        let trash_after = service.list_trash().unwrap();
        assert!(trash_after.is_empty());
    }
}
