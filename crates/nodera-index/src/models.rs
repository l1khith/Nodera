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

/// Scored related note recommendation based on hybrid lexical and semantic similarity.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RelatedNote {
    pub path: String,
    pub title: String,
    pub similarity_score: f32,
    pub match_percentage: u32,
    pub shared_tags: Vec<String>,
    pub snippet: String,
}

/// Category of items in the Review Queue.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReviewCategory {
    RoughNote,
    Unlinked,
    StaleSource,
    Orphan,
}

/// Item surfaced in the Review Queue needing user attention or synthesis.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReviewQueueItem {
    pub path: String,
    pub title: String,
    pub category: ReviewCategory,
    pub reason: String,
    pub modified_ns: u64,
}

/// High-level knowledge health metrics across the vault.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct KnowledgeStats {
    pub total_notes: usize,
    pub rough_count: usize,
    pub permanent_count: usize,
    pub source_count: usize,
    pub index_count: usize,
    pub unlinked_count: usize,
    pub orphan_count: usize,
    pub type_counts: std::collections::HashMap<String, usize>,
}

