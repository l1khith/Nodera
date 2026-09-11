use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

/// Options controlling the PDF to Markdown conversion pipeline.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConversionOptions {
    /// Whether to detect and format chapter/section headings as Markdown headings.
    pub detect_headings: bool,
    /// Whether to detect and remove repeated running headers across pages.
    pub remove_repeated_headers: bool,
    /// Whether to detect and remove standalone page numbers.
    pub remove_page_numbers: bool,
    /// Whether to insert HTML page comment markers (`<!-- nodera:page=X -->`).
    pub add_page_markers: bool,
    /// Default heading depth prefix for top-level detected headings (e.g. 2 for `## `).
    pub heading_prefix_depth: usize,
}

impl Default for ConversionOptions {
    fn default() -> Self {
        Self {
            detect_headings: true,
            remove_repeated_headers: true,
            remove_page_numbers: true,
            add_page_markers: false,
            heading_prefix_depth: 2,
        }
    }
}

/// Stages of conversion executed by the PDF engine.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConversionStage {
    Created,
    Validating,
    Extracting,
    Cleaning,
    Structuring,
    Completed,
    Failed,
    Cancelled,
}

impl std::fmt::Display for ConversionStage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Created => write!(f, "Created"),
            Self::Validating => write!(f, "Validating PDF"),
            Self::Extracting => write!(f, "Extracting text"),
            Self::Cleaning => write!(f, "Cleaning headers & artifacts"),
            Self::Structuring => write!(f, "Structuring Markdown"),
            Self::Completed => write!(f, "Completed"),
            Self::Failed => write!(f, "Failed"),
            Self::Cancelled => write!(f, "Cancelled"),
        }
    }
}

/// Progress event emitted monotonically during PDF conversion.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConversionProgress {
    pub page_current: usize,
    pub page_total: usize,
    pub stage: ConversionStage,
    pub bytes_processed: Option<u64>,
    pub message: String,
}

/// Result produced by a successful PDF conversion.
///
/// Note: The low-level PDF converter does not determine the vault destination.
/// Destination resolution and disk persistence are handled by the application import service.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConversionResult {
    pub title: String,
    pub total_pages: usize,
    pub chapters_detected: usize,
    pub characters_extracted: usize,
    pub markdown_content: String,
}

/// Extracted metadata from a validated PDF file.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PdfMetadata {
    pub title: Option<String>,
    pub author: Option<String>,
    pub page_count: usize,
    pub file_size_bytes: u64,
    pub is_encrypted: bool,
    pub is_accessible_without_password: bool,
}

/// Thread-safe cooperative cancellation token.
#[derive(Debug, Clone, Default)]
pub struct CancellationToken {
    cancelled: Arc<AtomicBool>,
}

impl CancellationToken {
    pub fn new() -> Self {
        Self {
            cancelled: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn cancel(&self) {
        self.cancelled.store(true, Ordering::SeqCst);
    }

    pub fn is_cancelled(&self) -> bool {
        self.cancelled.load(Ordering::SeqCst)
    }

    pub fn reset(&self) {
        self.cancelled.store(false, Ordering::SeqCst);
    }
}

/// Callback type for listening to progress updates safely across thread boundaries.
pub type ProgressSink = Arc<dyn Fn(ConversionProgress) + Send + Sync>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_conversion_options_default() {
        let opts = ConversionOptions::default();
        assert!(opts.detect_headings);
        assert!(opts.remove_repeated_headers);
        assert!(opts.remove_page_numbers);
        assert!(!opts.add_page_markers);
        assert_eq!(opts.heading_prefix_depth, 2);
    }

    #[test]
    fn test_cancellation_token() {
        let token = CancellationToken::new();
        assert!(!token.is_cancelled());

        let cloned = token.clone();
        cloned.cancel();
        assert!(token.is_cancelled());

        token.reset();
        assert!(!token.is_cancelled());
    }
}
