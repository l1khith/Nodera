use rayon::prelude::*;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::path::Path;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use tracing::info;

use nodera_core::{IndexingPhase, IndexingProgress, Note, Result, VaultEntry, VaultService};
use nodera_markdown::{parse_document, ParsedDocument};

use crate::models::{
    Activity, ActivityFilter, IndexedTask, KnowledgeStats, RelatedNote, ReviewQueueItem,
    SearchResult, TagCount, TaskFilter,
};
use crate::sqlite::{NoteMetadataRecord, SqliteIndex};
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
    ///
    /// If an index is corrupt (e.g. malformed SQLite file, invalid Tantivy directory),
    /// it automatically quarantines/clears the corrupt cache and initialises a fresh, empty
    /// index ready for immediate rebuilding from canonical Markdown files.
    pub fn open(vault_root: impl AsRef<Path>) -> Result<Self> {
        let root = vault_root.as_ref();
        let nodera_dir = root.join(".nodera");
        let sqlite_path = nodera_dir.join("index.sqlite");
        let tantivy_path = nodera_dir.join("tantivy");

        let sqlite = match SqliteIndex::open(&sqlite_path) {
            Ok(idx) => idx,
            Err(e) => {
                tracing::warn!(
                    "SQLite index failed to open ({e}), executing self-healing recovery"
                );
                let _ = std::fs::remove_file(&sqlite_path);
                SqliteIndex::open(&sqlite_path)?
            }
        };

        let tantivy = match TantivyIndex::open(&tantivy_path) {
            Ok(idx) => idx,
            Err(e) => {
                tracing::warn!(
                    "Tantivy index failed to open ({e}), executing self-healing recovery"
                );
                let _ = std::fs::remove_dir_all(&tantivy_path);
                TantivyIndex::open(&tantivy_path)?
            }
        };

        Ok(Self { sqlite, tantivy })
    }

    /// Creates an in-memory index pair for testing.
    pub fn in_memory() -> Result<Self> {
        let sqlite = SqliteIndex::in_memory()?;
        let tantivy = TantivyIndex::in_memory()?;
        Ok(Self { sqlite, tantivy })
    }

    /// Stages indexing for a note without immediate Tantivy commit. Used for batch indexing.
    pub fn index_note_uncommitted(&mut self, note: &Note, parsed: &ParsedDocument) -> Result<()> {
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

        Ok(())
    }

    /// Incrementally indexes or updates a note in both SQLite and Tantivy indexes, committing changes immediately.
    pub fn index_note(&mut self, note: &Note, parsed: &ParsedDocument) -> Result<()> {
        self.index_note_uncommitted(note, parsed)?;
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
    pub fn search(&self, query_str: &str, limit: usize) -> Result<Vec<SearchResult>> {
        self.tantivy.search(query_str, limit)
    }

    /// Queries tasks across the vault with filtering criteria.
    pub fn query_tasks(&self, filter: &TaskFilter) -> Result<Vec<IndexedTask>> {
        self.sqlite.query_tasks(filter)
    }

    /// Queries all unique tags with count of notes using them.
    pub fn query_tags(&self) -> Result<Vec<TagCount>> {
        self.sqlite.query_tags()
    }

    /// Queries all note paths and their associated tags.
    pub fn query_all_note_tags(&self) -> Result<HashMap<std::path::PathBuf, Vec<String>>> {
        self.sqlite.query_all_note_tags()
    }

    /// Queries backlinks pointing to a note target.
    pub fn query_backlinks(&self, target: &str) -> Result<Vec<String>> {
        self.sqlite.query_backlinks(target)
    }

    /// Queries rough / fleeting notes waiting for synthesis.
    pub fn query_rough_notes(&self) -> Result<Vec<ReviewQueueItem>> {
        self.sqlite.query_rough_notes()
    }

    /// Queries unlinked notes (knowledge islands).
    pub fn query_unlinked_notes(&self) -> Result<Vec<ReviewQueueItem>> {
        self.sqlite.query_unlinked_notes()
    }

    /// Queries stale source notes with no synthesis links.
    pub fn query_stale_sources(&self, cutoff_ns: u64) -> Result<Vec<ReviewQueueItem>> {
        self.sqlite.query_stale_sources(cutoff_ns)
    }

    /// Queries orphan notes (0 incoming backlinks).
    pub fn query_orphan_notes(&self) -> Result<Vec<ReviewQueueItem>> {
        self.sqlite.query_orphan_notes()
    }

    /// Queries high-level knowledge metrics across the vault.
    pub fn query_knowledge_stats(&self) -> Result<KnowledgeStats> {
        self.sqlite.query_knowledge_stats()
    }

    /// Records an activity event in the local evidence stream.
    pub fn record_activity(&mut self, activity: &Activity) -> Result<()> {
        self.sqlite.record_activity(activity)
    }

    /// Queries activity events matching filter criteria.
    pub fn query_activities(&self, filter: &ActivityFilter) -> Result<Vec<Activity>> {
        self.sqlite.query_activities(filter)
    }

    /// Deletes an activity event by id.
    pub fn delete_activity(&mut self, id: &str) -> Result<bool> {
        self.sqlite.delete_activity(id)
    }

    /// Clears all recorded activities.
    pub fn clear_activities(&mut self) -> Result<()> {
        self.sqlite.clear_activities()
    }

    /// Returns scored related note recommendations combining BM25 lexical similarity (70%) and tag Jaccard overlap (30%).
    pub fn get_related_notes(
        &self,
        exclude_path: &str,
        title: &str,
        body: &str,
        tags: &[String],
        limit: usize,
    ) -> Result<Vec<RelatedNote>> {
        let candidates =
            self.tantivy
                .find_related_notes(exclude_path, title, body, tags, limit * 2)?;

        if candidates.is_empty() {
            return Ok(Vec::new());
        }

        let max_score = candidates.iter().map(|c| c.score).fold(0.0f32, f32::max);

        let source_tags_lower: std::collections::HashSet<String> =
            tags.iter().map(|t| t.to_lowercase()).collect();

        let mut scored_notes: Vec<RelatedNote> = candidates
            .into_iter()
            .map(|candidate| {
                let target_tags = self
                    .sqlite
                    .query_tags_for_path(&candidate.path)
                    .unwrap_or_default();

                let mut shared_tags = Vec::new();
                let mut target_tags_lower = std::collections::HashSet::new();

                for t in &target_tags {
                    let tl = t.to_lowercase();
                    if source_tags_lower.contains(&tl) {
                        shared_tags.push(t.clone());
                    }
                    target_tags_lower.insert(tl);
                }

                let jaccard = if source_tags_lower.is_empty() && target_tags_lower.is_empty() {
                    0.0f32
                } else {
                    let union_size = source_tags_lower.union(&target_tags_lower).count();
                    if union_size == 0 {
                        0.0f32
                    } else {
                        shared_tags.len() as f32 / union_size as f32
                    }
                };

                let norm_bm25 = if max_score > 0.0 {
                    (candidate.score / max_score).clamp(0.0, 1.0)
                } else {
                    0.0
                };

                let similarity_score = (0.7 * norm_bm25) + (0.3 * jaccard);
                let match_percentage = ((similarity_score * 100.0).clamp(1.0, 99.0)).round() as u32;

                RelatedNote {
                    path: candidate.path,
                    title: candidate.title,
                    similarity_score,
                    match_percentage,
                    shared_tags,
                    snippet: candidate.snippet,
                }
            })
            .collect();

        scored_notes.sort_by(|a, b| {
            b.similarity_score
                .partial_cmp(&a.similarity_score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        scored_notes.truncate(limit);
        Ok(scored_notes)
    }

    /// Fully rebuilds the derived indexes from the canonical Markdown files on disk.
    pub fn rebuild(&mut self, vault_service: &VaultService) -> Result<usize> {
        self.rebuild_with_progress(vault_service, |_| {}, None)
    }

    /// Fully rebuilds derived indexes using bounded Rayon parallelism for file reading and parsing,
    /// followed by single-transaction SQLite batch write and single-commit Tantivy indexing.
    /// Emits typed `IndexingProgress` events and allows cooperative cancellation.
    pub fn rebuild_with_progress<F>(
        &mut self,
        vault_service: &VaultService,
        progress_cb: F,
        cancel_token: Option<&AtomicBool>,
    ) -> Result<usize>
    where
        F: Fn(IndexingProgress) + Send + Sync,
    {
        info!("Starting parallel vault index rebuild");

        // Stage 1: File Discovery
        progress_cb(IndexingProgress::new(
            IndexingPhase::Discovering,
            0,
            0,
            "Discovering vault files...",
        ));
        let entries = vault_service.list_entries()?;

        let note_summaries: Vec<_> = entries
            .into_iter()
            .filter_map(|e| match e {
                VaultEntry::Note(summary) => Some(summary),
                _ => None,
            })
            .collect();

        let total_notes = note_summaries.len();
        if total_notes == 0 {
            self.sqlite.rebuild_batch(&[])?;
            self.tantivy.clear_all()?;
            self.tantivy.commit()?;
            progress_cb(IndexingProgress::new(
                IndexingPhase::Complete,
                0,
                0,
                "Rebuild complete (0 notes)",
            ));
            return Ok(0);
        }

        if let Some(token) = cancel_token {
            if token.load(Ordering::Relaxed) {
                progress_cb(IndexingProgress::new(
                    IndexingPhase::Cancelled,
                    0,
                    total_notes,
                    "Rebuild cancelled",
                ));
                return Ok(0);
            }
        }

        // Stage 2, 3, 4: Parallel file reads, Markdown parsing, and document preparation
        progress_cb(IndexingProgress::new(
            IndexingPhase::Parsing,
            0,
            total_notes,
            format!("Parsing {total_notes} notes in parallel..."),
        ));

        let completed_counter = AtomicUsize::new(0);

        struct ParsedNoteData {
            note_id: String,
            path: String,
            title: String,
            content_hash: String,
            size_bytes: u64,
            modified_ns: u64,
            wikilinks: Vec<nodera_markdown::Wikilink>,
            tasks: Vec<nodera_markdown::ParsedTask>,
            tags: Vec<String>,
            properties: HashMap<String, serde_json::Value>,
            headings_text: String,
            body: String,
            tags_str: String,
        }

        let parsed_items: Vec<ParsedNoteData> = note_summaries
            .par_iter()
            .filter_map(|summary| {
                if let Some(token) = cancel_token {
                    if token.load(Ordering::Relaxed) {
                        return None;
                    }
                }

                let note = vault_service.read_note(&summary.relative_path).ok()?;
                let parsed = parse_document(&note.content).ok()?;

                let path_str = note.relative_path.to_string_lossy().replace('\\', "/");
                let title = parsed.title.clone().unwrap_or_else(|| note.title.clone());

                let mut hasher = std::collections::hash_map::DefaultHasher::new();
                note.content.hash(&mut hasher);
                let content_hash = format!("{:016x}", hasher.finish());

                let size_bytes = note.content.len() as u64;
                let modified_ns = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|d| d.as_nanos() as u64)
                    .unwrap_or(0);

                let headings_text = parsed
                    .headings
                    .iter()
                    .map(|h| h.text.as_str())
                    .collect::<Vec<_>>()
                    .join(" ");

                let tags_str = parsed.tags.join(" ");

                let mut properties = HashMap::new();
                if let Some(fm) = &parsed.frontmatter {
                    for (k, v) in &fm.extra {
                        let json_val = serde_json::to_value(v).unwrap_or(serde_json::Value::Null);
                        properties.insert(k.clone(), json_val);
                    }
                }

                let done = completed_counter.fetch_add(1, Ordering::Relaxed) + 1;
                if done.is_multiple_of(100) || done == total_notes {
                    progress_cb(IndexingProgress::new(
                        IndexingPhase::Parsing,
                        done,
                        total_notes,
                        format!("Parsed {done}/{total_notes} notes"),
                    ));
                }

                Some(ParsedNoteData {
                    note_id: note.id.as_str().to_string(),
                    path: path_str,
                    title,
                    content_hash,
                    size_bytes,
                    modified_ns,
                    wikilinks: parsed.wikilinks,
                    tasks: parsed.tasks,
                    tags: parsed.tags,
                    properties,
                    headings_text,
                    body: parsed.body,
                    tags_str,
                })
            })
            .collect();

        if let Some(token) = cancel_token {
            if token.load(Ordering::Relaxed) {
                progress_cb(IndexingProgress::new(
                    IndexingPhase::Cancelled,
                    parsed_items.len(),
                    total_notes,
                    "Rebuild cancelled",
                ));
                return Ok(parsed_items.len());
            }
        }

        // Stage 5a: Batch SQLite persistence in a single transaction
        progress_cb(IndexingProgress::new(
            IndexingPhase::Persisting,
            0,
            parsed_items.len(),
            "Persisting metadata into SQLite...",
        ));

        let metadata_records: Vec<NoteMetadataRecord<'_>> = parsed_items
            .iter()
            .map(|item| NoteMetadataRecord {
                note_id: &item.note_id,
                path: &item.path,
                title: &item.title,
                content_hash: &item.content_hash,
                size_bytes: item.size_bytes,
                modified_ns: item.modified_ns,
                links: &item.wikilinks,
                tasks: &item.tasks,
                tags: &item.tags,
                properties: &item.properties,
            })
            .collect();

        self.sqlite.rebuild_batch(&metadata_records)?;

        // Stage 5b: Batch Tantivy indexing in 1 commit
        progress_cb(IndexingProgress::new(
            IndexingPhase::Indexing,
            0,
            parsed_items.len(),
            "Indexing full-text search documents...",
        ));

        self.tantivy.clear_all()?;
        for item in &parsed_items {
            self.tantivy.index_document(
                &item.note_id,
                &item.path,
                &item.title,
                &item.headings_text,
                &item.body,
                &item.tags_str,
            )?;
        }

        progress_cb(IndexingProgress::new(
            IndexingPhase::Finalizing,
            parsed_items.len(),
            parsed_items.len(),
            "Committing search index...",
        ));

        self.tantivy.commit()?;

        let count = parsed_items.len();
        progress_cb(IndexingProgress::new(
            IndexingPhase::Complete,
            count,
            count,
            format!("Rebuild complete ({count} notes indexed)"),
        ));

        info!(
            indexed_notes = count,
            "Completed parallel vault index rebuild"
        );
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

        // 6. Test related notes
        let note1_rel = note1.relative_path.to_string_lossy().replace('\\', "/");
        let related = index
            .get_related_notes(
                &note1_rel,
                "Rust Introduction",
                "Rust systems programming",
                &["rust".to_string(), "systems".to_string()],
                5,
            )
            .unwrap();
        assert_eq!(related.len(), 1);
        assert_eq!(related[0].title, "Cargo Guide");
        assert!(related[0].match_percentage > 0);
    }
}
