use crate::models::{
    CancellationToken, ConversionOptions, ConversionResult, PdfMetadata, ProgressSink,
};
use nodera_core::error::PdfError;
use std::path::Path;

/// Trait defining the interface for PDF-to-Markdown converters.
///
/// This abstraction isolates the extraction engine (e.g. native Rust `lopdf` or future
/// alternative backends) from the application layer and desktop UI.
pub trait PdfConverter: Send + Sync {
    /// Validates the PDF file, verifying magic bytes, structure, and readability,
    /// and extracts document metadata (pages, title, encryption).
    fn validate(&self, input: &Path) -> Result<PdfMetadata, PdfError>;

    /// Converts a PDF document into a `ConversionResult` containing structured Markdown.
    ///
    /// The conversion is cooperative with the provided `CancellationToken` and emits
    /// monotonic progress events to the optional `ProgressSink`.
    fn convert(
        &self,
        input: &Path,
        options: &ConversionOptions,
        cancellation: &CancellationToken,
        progress: Option<&ProgressSink>,
    ) -> Result<ConversionResult, PdfError>;
}
