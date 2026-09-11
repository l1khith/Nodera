use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::path::Path;
use tracing::info;

use nodera_core::{Note, Result, VaultEntry, VaultService};
use nodera_markdown::{parse_document, ParsedDocument};

use crate::models::{IndexedTask, SearchResult, TagCount, TaskFilter};
use crate::sqlite::SqliteIndex;
use crate::tantivy_index::TantivyIndex;

/// Coordinated vault index manager providing derived SQLite metadata and Tantivy full-text search.
pub struct VaultIndex {
    sqlite: SqliteIndex,
    tantivy: TantivyIndex,
}

impl std::fmt::Debug for VaultIndex {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("VaultIndex").finish()
    }
}

impl VaultIndex {
    /// Opens the vault index located in `.nodera/` subdirectory of the vault root.
    pub fn open(vault_root: impl AsRef<Path>) -> Result<Self> {
        let root = vault_root.as_ref();
        let nodera_dir = root.join(".nodera");
        let sqlite_path = nodera_dir.join("index.sqlite");
        let tantivy_path = nodera_dir.join("tantivy");

        let sqlite = SqliteIndex::open(sqlite_path)?;
        let tantivy = TantivyIndex::open(tantivy_path)?;

        Ok(Self { sqlite, tantivy })
    }

    /// Creates an in-memory index pair for testing.
    pub fn in_memory() -> Result<Self> {
        let sqlite = SqliteIndex::in_memory()?;
        let tantivy = TantivyIndex::in_memory()?;
        Ok(Self { sqlite, tantivy })
    }

    /// Incrementally indexes or updates a note in both SQLite and Tantivy indexes.
    pub fn index_note(&mut self, note: &Note, parsed: &ParsedDocument) -> Result<()> {
        let path_str = note.relative_path.to_string_lossy().replace('\\', "/");
        let title = parsed.title.clone().unwrap_or_else(|| note.title.clone());

        // Content hash for change detection
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        note.content.hash(&mut hasher);
        let content_hash = format!("{:016x}", hasher.finish());

        let size_bytes = note.content.len() as u64;
        let modified_ns = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos() as u64)
            .unwrap_or(0);

        // Extract headings text
        let headings_text = parsed
            .headings
            .iter()
            .map(|h| h.text.as_str())
            .collect::<Vec<_>>()
            .join(" ");

        // Extract tags string
        let tags_str = parsed.tags.join(" ");

        // Extract properties from frontmatter
        let mut properties = HashMap::new();
        if let Some(fm) = &parsed.frontmatter {
            for (k, v) in &fm.extra {
                let json_val = serde_json::to_value(v).unwrap_or(serde_json::Value::Null);
                properties.insert(k.clone(), json_val);
            }
        }

        // 1. Update SQLite
        self.sqlite.index_note_metadata(
            note.id.as_str(),
            &path_str,
            &title,
            &content_hash,
            size_bytes,
            modified_ns,
            &parsed.wikilinks,
            &parsed.tasks,
            &parsed.tags,
            &properties,
        )?;

        // 2. Update Tantivy
        self.tantivy.index_document(
            note.id.as_str(),
            &path_str,
            &title,
            &headings_text,
            &parsed.body,
            &tags_str,
        )?;
        self.tantivy.commit()?;

        Ok(())
    }

    /// Removes a note from both derived indexes upon deletion.
    pub fn remove_note(&mut self, rel_path: impl AsRef<Path>) -> Result<()> {
        let path_str = rel_path.as_ref().to_string_lossy().replace('\\', "/");
        self.sqlite.remove_file(&path_str)?;
        self.tantivy.delete_document(&path_str)?;
        self.tantivy.commit()?;
        Ok(())
    }

    /// Executes full-text search with ranking and snippets.
    pub fn search(&self, query: &str, limit: usize) -> Result<Vec<SearchResult>> {
        self.tantivy.search(query, limit)
    }

    /// Queries tasks across the vault with filtering criteria.
    pub fn query_tasks(&self, filter: &TaskFilter) -> Result<Vec<IndexedTask>> {
        self.sqlite.query_tasks(filter)
    }

    /// Queries all unique tags with count of notes using them.
    pub fn query_tags(&self) -> Result<Vec<TagCount>> {
        self.sqlite.query_tags()
    }

    /// Queries backlinks pointing to a note target.
    pub fn query_backlinks(&self, target: &str) -> Result<Vec<String>> {
        self.sqlite.query_backlinks(target)
    }

    /// Fully rebuilds the derived indexes from the canonical Markdown files on disk.
    pub fn rebuild(&mut self, vault_service: &VaultService) -> Result<usize> {
        info!("Starting full vault index rebuild");
        self.sqlite.clear_all()?;
        self.tantivy.clear_all()?;

        let entries = vault_service.list_entries()?;
        let mut count = 0;

        for entry in entries {
            if let VaultEntry::Note(summary) = entry {
                if let Ok(note) = vault_service.read_note(&summary.relative_path) {
                    if let Ok(parsed) = parse_document(&note.content) {
                        self.index_note(&note, &parsed)?;
                        count += 1;
                    }
                }
            }
        }

        info!(indexed_notes = count, "Completed full vault index rebuild");
        Ok(count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nodera_core::Vault;
    use tempfile::tempdir;

    #[test]
    fn test_vault_indexer_rebuild_and_search() {
        let tmp = tempdir().unwrap();
        let vault_root = tmp.path().join("IndexVault");
        let vault = Vault::create(&vault_root, Some("Index Vault".to_string())).unwrap();
        let service = VaultService::new(vault);

        // Create 2 notes
        let note1 = service.create_note(
            None,
            "Rust Introduction",
            Some("# Rust Introduction\nRust is a memory-safe systems language with [[Cargo]].\nTags: #rust #systems\n\n- [ ] Install Rustup\n- [x] Read the Book"),
        ).unwrap();

        let _note2 = service.create_note(
            None,
            "Cargo Guide",
            Some("# Cargo Guide\nCargo is the official package manager for [[Rust]].\n\n- [ ] Run cargo build"),
        ).unwrap();

        let mut index = VaultIndex::in_memory().unwrap();

        // 1. Index note 1
        let parsed1 = parse_document(&note1.content).unwrap();
        index.index_note(&note1, &parsed1).unwrap();

        // 2. Search note 1
        let search_res = index.search("memory-safe", 10).unwrap();
        assert_eq!(search_res.len(), 1);
        assert_eq!(search_res[0].title, "Rust Introduction");

        // 3. Test full rebuild
        let count = index.rebuild(&service).unwrap();
        assert_eq!(count, 2);

        // 4. Query tasks across both notes
        let tasks = index.query_tasks(&TaskFilter::default()).unwrap();
        assert_eq!(tasks.len(), 3);

        let incomplete_tasks = index
            .query_tasks(&TaskFilter {
                checked: Some(false),
                ..Default::default()
            })
            .unwrap();
        assert_eq!(incomplete_tasks.len(), 2);

        // 5. Query backlinks for Rust
        let rust_backlinks = index.query_backlinks("Rust").unwrap();
        assert!(rust_backlinks.iter().any(|p| p.contains("Cargo Guide")));
    }
}
