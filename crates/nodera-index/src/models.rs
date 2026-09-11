use serde::{Deserialize, Serialize};

/// Represents an indexed Markdown task in the derived database.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IndexedTask {
    pub id: String,
    pub note_id: String,
    pub note_path: String,
    pub note_title: String,
    pub line_number: usize,
    pub checked: bool,
    pub text: String,
    pub due_date: Option<String>,
}

/// Filter criteria for querying tasks across the vault.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct TaskFilter {
    /// None = all, Some(false) = incomplete only, Some(true) = completed only
    pub checked: Option<bool>,
    /// Optional filter by specific note relative path
    pub note_path: Option<String>,
    /// Optional text substring match
    pub search_query: Option<String>,
}

/// Search hit returned by full-text search.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SearchResult {
    pub note_id: String,
    pub path: String,
    pub title: String,
    pub score: f32,
    pub snippet: String,
}

/// Aggregated tag count across the vault.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TagCount {
    pub tag: String,
    pub count: usize,
}
