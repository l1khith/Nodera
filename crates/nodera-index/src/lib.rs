//! Indexing and search derivation crate for Nodera.

pub mod indexer;
pub mod models;
pub mod sqlite;
pub mod tantivy_index;

pub use indexer::VaultIndex;
pub use models::{
    Activity, ActivityFilter, ActivityKind, IndexedTask, KnowledgeStats, RelatedNote,
    ReviewCategory, ReviewQueueItem, SearchResult, TagCount, TaskFilter,
};
pub use sqlite::{NoteMetadataRecord, SqliteIndex};
pub use tantivy_index::TantivyIndex;
