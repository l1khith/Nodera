use crate::models::{CancellationToken, ConversionOptions, ConversionResult, ProgressSink};
use crate::traits::PdfConverter;
use nodera_core::error::{FileError, NoderaError};
use nodera_core::fs::{atomic_write_str, sanitize_filename};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::sync::Arc;

/// Result of a completed PDF import orchestration.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ImportResult {
    /// Relative path inside the vault (e.g. `Books/My Book.md`).
    pub relative_vault_path: PathBuf,
    /// Absolute path on disk.
    pub absolute_path: PathBuf,
    /// Conversion statistics and generated Markdown.
    pub conversion: ConversionResult,
}

/// Service orchestrating the conversion and storage of a PDF file into a vault.
#[derive(Debug, Clone)]
pub struct PdfImportService<C: PdfConverter> {
    converter: Arc<C>,
}

impl<C: PdfConverter> PdfImportService<C> {
    pub fn new(converter: Arc<C>) -> Self {
        Self { converter }
    }

    pub fn converter(&self) -> &Arc<C> {
        &self.converter
    }

    /// Converts a PDF document and writes the generated Markdown file atomically
    /// to the vault under `Books/<title>.md` (with collision avoidance).
    pub fn import_to_vault(
        &self,
        vault_root: &Path,
        input_pdf: &Path,
        options: &ConversionOptions,
        cancellation: &CancellationToken,
        progress: Option<&ProgressSink>,
    ) -> Result<ImportResult, NoderaError> {
        if cancellation.is_cancelled() {
            return Err(NoderaError::OperationCancelled {
                message: "PDF import was cancelled".to_string(),
            });
        }

        // 1. Run conversion
        let conversion = self
            .converter
            .convert(input_pdf, options, cancellation, progress)?;

        if cancellation.is_cancelled() {
            return Err(NoderaError::OperationCancelled {
                message: "PDF import was cancelled".to_string(),
            });
        }

        // 2. Resolve target destination path with collision avoidance
        let (relative_path, absolute_path) = resolve_destination(vault_root, &conversion.title);

        // 3. Ensure destination directory exists
        if let Some(parent) = absolute_path.parent() {
            if !parent.exists() {
                std::fs::create_dir_all(parent).map_err(|e| FileError::WriteFailed {
                    path: parent.to_path_buf(),
                    source: e,
                })?;
            }
        }

        // 4. Atomically write Markdown content
        atomic_write_str(&absolute_path, &conversion.markdown_content)?;

        Ok(ImportResult {
            relative_vault_path: relative_path,
            absolute_path,
            conversion,
        })
    }
}

/// Resolves a non-colliding destination path inside `Books/<sanitized-title>.md`.
///
/// If a note with the same name already exists:
/// `Books/Book.md` -> `Books/Book (1).md` -> `Books/Book (2).md`, etc.
pub fn resolve_destination(vault_root: &Path, title: &str) -> (PathBuf, PathBuf) {
    let sanitized = sanitize_filename(title);
    let base_name = if sanitized.is_empty() {
        "Imported Document".to_string()
    } else {
        sanitized
    };

    let books_dir = vault_root.join("Books");
    let relative_base = PathBuf::from("Books");

    let mut candidate_name = format!("{}.md", base_name);
    let mut counter = 1;

    while books_dir.join(&candidate_name).exists() {
        candidate_name = format!("{} ({}).md", base_name, counter);
        counter += 1;
    }

    let relative_path = relative_base.join(&candidate_name);
    let absolute_path = books_dir.join(&candidate_name);
    (relative_path, absolute_path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_resolve_destination_no_collision() {
        let temp = TempDir::new().unwrap();
        let (rel, abs) = resolve_destination(temp.path(), "My Great Book");

        assert_eq!(rel, PathBuf::from("Books").join("My Great Book.md"));
        assert_eq!(abs, temp.path().join("Books").join("My Great Book.md"));
    }

    #[test]
    fn test_resolve_destination_with_collision() {
        let temp = TempDir::new().unwrap();
        let books = temp.path().join("Books");
        std::fs::create_dir_all(&books).unwrap();

        // Pre-create Book.md and Book (1).md
        std::fs::write(books.join("Rust Guide.md"), "existing").unwrap();
        std::fs::write(books.join("Rust Guide (1).md"), "existing").unwrap();

        let (rel, abs) = resolve_destination(temp.path(), "Rust Guide");

        assert_eq!(rel, PathBuf::from("Books").join("Rust Guide (2).md"));
        assert_eq!(abs, temp.path().join("Books").join("Rust Guide (2).md"));
    }
}
