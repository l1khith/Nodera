use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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
    /// Optional filter by exact due date or prefix
    pub due_date: Option<String>,
}

/// Universal activity event recorded in the local evidence stream.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Activity {
    pub id: String,
    pub timestamp_secs: i64,
    pub kind: ActivityKind,
    pub duration_secs: Option<u32>,
    pub source: String,
    pub project_id: Option<String>,
    pub task_id: Option<String>,
    pub note_path: Option<String>,
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Typed categories of verifiable activity.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ActivityKind {
    Coding,
    Reading,
    Writing,
    TaskCompletion,
    JournalEntry,
    Review,
    Custom(String),
}

impl std::fmt::Display for ActivityKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Coding => write!(f, "coding"),
            Self::Reading => write!(f, "reading"),
            Self::Writing => write!(f, "writing"),
            Self::TaskCompletion => write!(f, "task_completion"),
            Self::JournalEntry => write!(f, "journal_entry"),
            Self::Review => write!(f, "review"),
            Self::Custom(s) => write!(f, "{s}"),
        }
    }
}

impl std::str::FromStr for ActivityKind {
    type Err = std::convert::Infallible;
    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        Ok(match s {
            "coding" => Self::Coding,
            "reading" => Self::Reading,
            "writing" => Self::Writing,
            "task_completion" => Self::TaskCompletion,
            "journal_entry" => Self::JournalEntry,
            "review" => Self::Review,
            other => Self::Custom(other.to_string()),
        })
    }
}

/// Criteria for querying the activity event stream.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ActivityFilter {
    pub kind: Option<ActivityKind>,
    pub project_id: Option<String>,
    pub note_path: Option<String>,
    pub since_secs: Option<i64>,
    pub until_secs: Option<i64>,
    pub limit: Option<usize>,
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
