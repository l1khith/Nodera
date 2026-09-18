use serde::{Deserialize, Serialize};

/// High-level lifecycle phase of a vault indexing or rebuilding operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum IndexingPhase {
    Discovering,
    Reading,
    Parsing,
    Persisting,
    Indexing,
    Finalizing,
    Complete,
    Failed,
    Cancelled,
}

impl std::fmt::Display for IndexingPhase {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IndexingPhase::Discovering => write!(f, "Discovering files"),
            IndexingPhase::Reading => write!(f, "Reading notes"),
            IndexingPhase::Parsing => write!(f, "Parsing Markdown"),
            IndexingPhase::Persisting => write!(f, "Persisting metadata"),
            IndexingPhase::Indexing => write!(f, "Indexing full-text"),
            IndexingPhase::Finalizing => write!(f, "Finalizing index"),
            IndexingPhase::Complete => write!(f, "Complete"),
            IndexingPhase::Failed => write!(f, "Failed"),
            IndexingPhase::Cancelled => write!(f, "Cancelled"),
        }
    }
}

/// Typed progress report for background indexing tasks.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IndexingProgress {
    pub phase: IndexingPhase,
    pub completed: usize,
    pub total: usize,
    pub message: String,
}

impl IndexingProgress {
    pub fn new(
        phase: IndexingPhase,
        completed: usize,
        total: usize,
        message: impl Into<String>,
    ) -> Self {
        Self {
            phase,
            completed,
            total,
            message: message.into(),
        }
    }

    /// Progress percentage between 0 and 100.
    pub fn percentage(&self) -> u32 {
        if self.total == 0 {
            0
        } else {
            ((self.completed as f64 / self.total as f64) * 100.0).clamp(0.0, 100.0) as u32
        }
    }

    pub fn is_finished(&self) -> bool {
        matches!(
            self.phase,
            IndexingPhase::Complete | IndexingPhase::Failed | IndexingPhase::Cancelled
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_indexing_progress_percentage() {
        let p = IndexingProgress::new(IndexingPhase::Parsing, 50, 200, "Parsing notes");
        assert_eq!(p.percentage(), 25);
        assert!(!p.is_finished());

        let p_done = IndexingProgress::new(IndexingPhase::Complete, 200, 200, "Done");
        assert_eq!(p_done.percentage(), 100);
        assert!(p_done.is_finished());
    }
}
