use serde::{Deserialize, Serialize};
use std::fmt;
use std::path::{Path, PathBuf};
use uuid::Uuid;

/// Unique identifier for a note.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct NoteId(pub String);

impl NoteId {
    /// Generates a new random NoteId.
    pub fn new() -> Self {
        Self(Uuid::new_v4().to_string())
    }

    /// Creates a NoteId from a string slice.
    pub fn from_string(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Default for NoteId {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for NoteId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// In-memory representation of a Markdown note.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Note {
    /// Internal note identifier.
    pub id: NoteId,
    /// Vault-relative path to the Markdown file (e.g., "Notes/Meeting.md").
    pub relative_path: PathBuf,
    /// Human-readable title derived from filename or note content.
    pub title: String,
    /// UTF-8 Markdown text content.
    pub content: String,
}

impl Note {
    pub fn new(relative_path: impl Into<PathBuf>, content: impl Into<String>) -> Self {
        let relative_path = relative_path.into();
        let title = relative_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("Untitled")
            .to_string();

        Self {
            id: NoteId::new(),
            relative_path,
            title,
            content: content.into(),
        }
    }

    pub fn with_id(
        id: NoteId,
        relative_path: impl Into<PathBuf>,
        content: impl Into<String>,
    ) -> Self {
        let relative_path = relative_path.into();
        let title = relative_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("Untitled")
            .to_string();

        Self {
            id,
            relative_path,
            title,
            content: content.into(),
        }
    }
}

/// Lightweight summary of a note file for file listings and trees.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NoteSummary {
    pub id: NoteId,
    pub relative_path: PathBuf,
    pub title: String,
    pub size_bytes: u64,
    pub modified_at_millis: u64,
}

/// An entry in the vault's file tree (either a folder or a Markdown note).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum VaultEntry {
    Folder {
        name: String,
        relative_path: PathBuf,
    },
    Note(NoteSummary),
}

impl VaultEntry {
    pub fn relative_path(&self) -> &Path {
        match self {
            VaultEntry::Folder { relative_path, .. } => relative_path,
            VaultEntry::Note(summary) => &summary.relative_path,
        }
    }

    pub fn name(&self) -> &str {
        match self {
            VaultEntry::Folder { name, .. } => name,
            VaultEntry::Note(summary) => &summary.title,
        }
    }

    pub fn is_folder(&self) -> bool {
        matches!(self, VaultEntry::Folder { .. })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_note_id_creation() {
        let id1 = NoteId::new();
        let id2 = NoteId::new();
        assert_ne!(id1, id2);
        assert!(!id1.as_str().is_empty());
    }

    #[test]
    fn test_note_title_derivation() {
        let note = Note::new(PathBuf::from("Notes/My First Note.md"), "# Hello");
        assert_eq!(note.title, "My First Note");
        assert_eq!(note.content, "# Hello");
        assert_eq!(note.relative_path, PathBuf::from("Notes/My First Note.md"));
    }
}
