//! Indexing and search derivation crate for Nodera.

pub mod indexer;
pub mod models;
pub mod sqlite;
pub mod tantivy_index;

pub use indexer::VaultIndex;
pub use models::{IndexedTask, SearchResult, TagCount, TaskFilter};
pub use sqlite::SqliteIndex;
pub use tantivy_index::TantivyIndex;
