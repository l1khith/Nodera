use crate::cleanup::{clean_pages, generate_markdown, normalize_page};
use crate::models::{
    CancellationToken, ConversionOptions, ConversionProgress, ConversionResult, ConversionStage,
    PdfMetadata, ProgressSink,
};
use crate::page::PageText;
use crate::traits::PdfConverter;
use nodera_core::error::PdfError;
use std::fs::File;
use std::io::Read;
use std::path::Path;

/// Maximum allowable decompressed text per page (20 MB) to protect against decompression bombs.
const MAX_DECOMPRESSED_PAGE_BYTES: usize = 20 * 1024 * 1024;

/// Native pure-Rust PDF converter using `lopdf`.
#[derive(Debug, Default, Clone)]
pub struct NativePdfConverter;

impl NativePdfConverter {
    pub fn new() -> Self {
        Self
    }

    fn extract_info_field(doc: &lopdf::Document, key: &[u8]) -> Option<String> {
        let info_obj = doc.trailer.get(b"Info").ok().and_then(|obj| match obj {
            lopdf::Object::Reference(id) => doc.get_object(*id).ok(),
            lopdf::Object::Dictionary(_) => Some(obj),
            _ => None,
        })?;

        let dict = info_obj.as_dict().ok()?;
        let val = dict.get(key).ok()?;

        match val {
            lopdf::Object::String(bytes, _) => decode_pdf_string(bytes),
            _ => None,
        }
    }
}

/// Decodes PDF text string supporting UTF-16BE (with BOM) or UTF-8 / ISO-8859-1.
fn decode_pdf_string(bytes: &[u8]) -> Option<String> {
    if bytes.is_empty() {
        return None;
    }

    // Check UTF-16BE BOM: \xFE\xFF
    if bytes.len() >= 2 && bytes[0] == 0xFE && bytes[1] == 0xFF {
        let u16_chars: Vec<u16> = bytes[2..]
            .chunks_exact(2)
            .map(|chunk| u16::from_be_bytes([chunk[0], chunk[1]]))
            .collect();
        return String::from_utf16(&u16_chars).ok();
    }

    // Try UTF-8
    if let Ok(s) = std::str::from_utf8(bytes) {
        return Some(s.to_string());
    }

    // Fallback to ISO-8859-1 (Latin1)
    Some(bytes.iter().map(|&b| b as char).collect())
}

impl PdfConverter for NativePdfConverter {
    fn validate(&self, input: &Path) -> Result<PdfMetadata, PdfError> {
        if !input.exists() {
            return Err(PdfError::InvalidInput {
                path: input.to_path_buf(),
                reason: "File does not exist".to_string(),
            });
        }

        if !input.is_file() {
            return Err(PdfError::InvalidInput {
                path: input.to_path_buf(),
                reason: "Specified path is not a regular file".to_string(),
            });
        }

        let file_size_bytes = match std::fs::metadata(input) {
            Ok(m) => m.len(),
            Err(e) => {
                return Err(PdfError::InvalidInput {
                    path: input.to_path_buf(),
                    reason: format!("Failed to read file metadata: {}", e),
                })
            }
        };

        if file_size_bytes < 5 {
            return Err(PdfError::InvalidPdf {
                path: input.to_path_buf(),
                reason: "File is too small to be a valid PDF".to_string(),
            });
        }

        // Verify PDF magic bytes (%PDF-)
        let mut f = File::open(input).map_err(|e| PdfError::InvalidInput {
            path: input.to_path_buf(),
            reason: format!("Failed to open file: {}", e),
        })?;

        let mut header = [0u8; 5];
        f.read_exact(&mut header)
            .map_err(|e| PdfError::InvalidPdf {
                path: input.to_path_buf(),
                reason: format!("Failed to read PDF header: {}", e),
            })?;

        if &header != b"%PDF-" {
            return Err(PdfError::InvalidPdf {
                path: input.to_path_buf(),
                reason: format!(
                    "Invalid PDF header magic bytes: expected '%PDF-', found '{:?}'",
                    String::from_utf8_lossy(&header)
                ),
            });
        }

        // Load document structure
        let doc = lopdf::Document::load(input).map_err(|e| PdfError::InvalidPdf {
            path: input.to_path_buf(),
            reason: format!("Failed to load PDF structure: {}", e),
        })?;

        let is_encrypted = doc.is_encrypted();
        let pages = doc.get_pages();
        let page_count = pages.len();

        if page_count == 0 {
            return Err(PdfError::InvalidPdf {
                path: input.to_path_buf(),
                reason: "PDF contains 0 pages".to_string(),
            });
        }

        let title = Self::extract_info_field(&doc, b"Title");
        let author = Self::extract_info_field(&doc, b"Author");

        Ok(PdfMetadata {
            title,
            author,
            page_count,
            file_size_bytes,
            is_encrypted,
            is_accessible_without_password: !is_encrypted,
        })
    }

    fn convert(
        &self,
        input: &Path,
        options: &ConversionOptions,
        cancellation: &CancellationToken,
        progress: Option<&ProgressSink>,
    ) -> Result<ConversionResult, PdfError> {
        let emit_progress = |p: ConversionProgress| {
            if let Some(sink) = progress {
                sink(p);
            }
        };

        if cancellation.is_cancelled() {
            return Err(PdfError::Cancelled);
        }

        // 1. Validation Stage
        emit_progress(ConversionProgress {
            page_current: 0,
            page_total: 0,
            stage: ConversionStage::Validating,
            bytes_processed: None,
            message: "Validating PDF file...".to_string(),
        });

        let metadata = self.validate(input)?;

        if metadata.is_encrypted && !metadata.is_accessible_without_password {
            return Err(PdfError::Encrypted {
                path: input.to_path_buf(),
                reason: "PDF is password protected and cannot be extracted without credentials"
                    .to_string(),
            });
        }

        let total_pages = metadata.page_count;

        // Load document into memory
        let doc = lopdf::Document::load(input).map_err(|e| PdfError::InvalidPdf {
            path: input.to_path_buf(),
            reason: format!("Failed to load document for conversion: {}", e),
        })?;

        // 2. Text Extraction Stage
        // Sort page numbers to ensure deterministic monotonic processing
        let mut page_numbers: Vec<u32> = doc.get_pages().keys().copied().collect();
        page_numbers.sort_unstable();

        let mut normalized_pages = Vec::with_capacity(total_pages);

        for (idx, &page_num) in page_numbers.iter().enumerate() {
            if cancellation.is_cancelled() {
                emit_progress(ConversionProgress {
                    page_current: idx,
                    page_total: total_pages,
                    stage: ConversionStage::Cancelled,
                    bytes_processed: None,
                    message: "Conversion cancelled by user".to_string(),
                });
                return Err(PdfError::Cancelled);
            }

            let current_page_idx = idx + 1;

            emit_progress(ConversionProgress {
                page_current: current_page_idx,
                page_total: total_pages,
                stage: ConversionStage::Extracting,
                bytes_processed: None,
                message: format!(
                    "Extracting text from page {} of {}",
                    current_page_idx, total_pages
                ),
            });

            // Extract with safety decompression limit
            let raw_text =
                match doc.extract_text_with_limit(&[page_num], MAX_DECOMPRESSED_PAGE_BYTES) {
                    Ok(text) => text,
                    Err(e) => {
                        return Err(PdfError::ExtractionFailed {
                            path: input.to_path_buf(),
                            page: current_page_idx,
                            reason: format!("Failed to extract page {}: {}", page_num, e),
                        });
                    }
                };

            let page_text = PageText::from_raw_text(current_page_idx, &raw_text);
            let normalized = normalize_page(&page_text);
            normalized_pages.push(normalized);
        }

        if cancellation.is_cancelled() {
            return Err(PdfError::Cancelled);
        }

        // 3. Cleaning Stage
        emit_progress(ConversionProgress {
            page_current: total_pages,
            page_total: total_pages,
            stage: ConversionStage::Cleaning,
            bytes_processed: None,
            message: "Cleaning repeated headers and artifacts...".to_string(),
        });

        let cleaned_pages = clean_pages(normalized_pages, options);

        if cancellation.is_cancelled() {
            return Err(PdfError::Cancelled);
        }

        // 4. Structuring Stage
        emit_progress(ConversionProgress {
            page_current: total_pages,
            page_total: total_pages,
            stage: ConversionStage::Structuring,
            bytes_processed: None,
            message: "Formatting headings and reconstructing paragraphs...".to_string(),
        });

        // Resolve title
        let title = metadata
            .title
            .filter(|s| !s.trim().is_empty())
            .or_else(|| {
                input
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .map(|s| s.to_string())
            })
            .unwrap_or_else(|| "Imported Document".to_string());

        let source_filename = input
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("document.pdf");

        let (markdown_content, chapters_detected) =
            generate_markdown(cleaned_pages, options, &title, source_filename);

        if cancellation.is_cancelled() {
            return Err(PdfError::Cancelled);
        }

        // 5. Completed Stage
        emit_progress(ConversionProgress {
            page_current: total_pages,
            page_total: total_pages,
            stage: ConversionStage::Completed,
            bytes_processed: None,
            message: "Conversion complete".to_string(),
        });

        Ok(ConversionResult {
            title,
            total_pages,
            chapters_detected,
            characters_extracted: markdown_content.len(),
            markdown_content,
        })
    }
}
