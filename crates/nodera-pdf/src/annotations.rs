use std::collections::BTreeMap;
use std::path::Path;

use lopdf::{Document, Object};
use nodera_core::error::PdfError;

use crate::models::{AnnotationKind, PdfAnnotation, PdfAnnotationReport};

/// Decodes PDF text string supporting UTF-16BE (with BOM) or UTF-8 / ISO-8859-1.
pub fn decode_pdf_string(bytes: &[u8]) -> Option<String> {
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

/// Helper to resolve an object reference if necessary.
fn resolve_object<'a>(doc: &'a Document, obj: &'a Object) -> Option<&'a Object> {
    match obj {
        Object::Reference(id) => doc.get_object(*id).ok(),
        _ => Some(obj),
    }
}

/// Extracts float value from a numeric PDF object.
fn object_to_f64(obj: &Object) -> Option<f64> {
    match obj {
        Object::Real(f) => Some(*f as f64),
        Object::Integer(i) => Some(*i as f64),
        _ => None,
    }
}

/// Converts PDF Color array (e.g. RGB `[r, g, b]` with values in 0.0..=1.0) to hex string.
fn parse_color(obj: &Object) -> Option<String> {
    if let Object::Array(colors) = obj {
        if colors.len() >= 3 {
            let r = (object_to_f64(&colors[0]).unwrap_or(0.0) * 255.0)
                .round()
                .clamp(0.0, 255.0) as u8;
            let g = (object_to_f64(&colors[1]).unwrap_or(0.0) * 255.0)
                .round()
                .clamp(0.0, 255.0) as u8;
            let b = (object_to_f64(&colors[2]).unwrap_or(0.0) * 255.0)
                .round()
                .clamp(0.0, 255.0) as u8;
            return Some(format!("#{:02x}{:02x}{:02x}", r, g, b));
        } else if colors.len() == 1 {
            // Grayscale
            let g = (object_to_f64(&colors[0]).unwrap_or(0.0) * 255.0)
                .round()
                .clamp(0.0, 255.0) as u8;
            return Some(format!("#{:02x}{:02x}{:02x}", g, g, g));
        }
    }
    None
}

/// Extracts all annotations from a PDF file using `lopdf`.
pub fn extract_annotations(path: &Path) -> Result<PdfAnnotationReport, PdfError> {
    if !path.exists() {
        return Err(PdfError::InvalidInput {
            path: path.to_path_buf(),
            reason: "PDF file does not exist".to_string(),
        });
    }

    let doc = Document::load(path).map_err(|e| PdfError::InvalidPdf {
        path: path.to_path_buf(),
        reason: format!("Failed to parse PDF document: {}", e),
    })?;

    let filename = path
        .file_name()
        .map(|f| f.to_string_lossy().to_string())
        .unwrap_or_else(|| "document.pdf".to_string());

    let pages = doc.get_pages();
    let total_pages = pages.len();
    let mut annotations = Vec::new();

    for (page_num, page_id) in pages {
        let page_obj = match doc.get_object(page_id) {
            Ok(obj) => obj,
            Err(_) => continue,
        };

        let page_dict = match page_obj.as_dict() {
            Ok(d) => d,
            Err(_) => continue,
        };

        let annots_obj = match page_dict.get(b"Annots") {
            Ok(obj) => resolve_object(&doc, obj),
            Err(_) => None,
        };

        let annot_array = match annots_obj {
            Some(Object::Array(ref arr)) => arr,
            _ => continue,
        };

        for annot_item in annot_array {
            let resolved = match resolve_object(&doc, annot_item) {
                Some(r) => r,
                None => continue,
            };

            let dict = match resolved.as_dict() {
                Ok(d) => d,
                Err(_) => continue,
            };

            // Parse Subtype
            let subtype_name = match dict.get(b"Subtype") {
                Ok(Object::Name(ref n)) => String::from_utf8_lossy(n).to_string(),
                _ => continue,
            };

            let kind = match subtype_name.as_str() {
                "Highlight" => AnnotationKind::Highlight,
                "Underline" => AnnotationKind::Underline,
                "Text" => AnnotationKind::StickyNote,
                "StrikeOut" => AnnotationKind::StrikeOut,
                "FreeText" => AnnotationKind::FreeText,
                "Squiggly" => AnnotationKind::Squiggly,
                other => AnnotationKind::Other(other.to_string()),
            };

            // Parse Contents
            let contents = dict.get(b"Contents").ok().and_then(|c| match c {
                Object::String(bytes, _) => decode_pdf_string(bytes),
                _ => None,
            });

            // Parse Author (T)
            let author = dict.get(b"T").ok().and_then(|t| match t {
                Object::String(bytes, _) => decode_pdf_string(bytes),
                _ => None,
            });

            // Parse Color
            let color_hex = dict.get(b"C").ok().and_then(parse_color);

            // Parse Rect
            let rect = dict.get(b"Rect").ok().and_then(|r| match r {
                Object::Array(ref arr) if arr.len() >= 4 => {
                    let x1 = object_to_f64(&arr[0])?;
                    let y1 = object_to_f64(&arr[1])?;
                    let x2 = object_to_f64(&arr[2])?;
                    let y2 = object_to_f64(&arr[3])?;
                    Some([x1, y1, x2, y2])
                }
                _ => None,
            });

            // Parse Date
            let creation_date = dict
                .get(b"CreationDate")
                .or_else(|_| dict.get(b"M"))
                .ok()
                .and_then(|d| match d {
                    Object::String(bytes, _) => decode_pdf_string(bytes),
                    _ => None,
                });

            annotations.push(PdfAnnotation {
                page: page_num as usize,
                kind,
                contents,
                author,
                color_hex,
                rect,
                creation_date,
            });
        }
    }

    Ok(PdfAnnotationReport {
        pdf_filename: filename,
        total_pages,
        annotations,
    })
}

/// Converts extracted PDF annotations into a well-structured, reading-friendly Markdown document.
pub fn annotations_to_markdown(report: &PdfAnnotationReport) -> String {
    let mut md = String::new();

    // Frontmatter
    md.push_str("---\n");
    md.push_str(&format!(
        "title: \"Annotations: {}\"\n",
        report.pdf_filename
    ));
    md.push_str(&format!("source_pdf: \"{}\"\n", report.pdf_filename));
    md.push_str(&format!(
        "total_annotations: {}\n",
        report.annotations.len()
    ));
    md.push_str("tags:\n  - pdf-annotations\n  - research\n");
    md.push_str("---\n\n");

    // Title and summary header
    md.push_str(&format!("# Annotations for {}\n\n", report.pdf_filename));
    md.push_str(&format!(
        "*Extracted {} annotation{} across {} page{}.*\n\n---\n\n",
        report.annotations.len(),
        if report.annotations.len() == 1 {
            ""
        } else {
            "s"
        },
        report.total_pages,
        if report.total_pages == 1 { "" } else { "s" }
    ));

    if report.annotations.is_empty() {
        md.push_str("> [!note]\n> No annotations or highlights were found in this document.\n");
        return md;
    }

    // Group annotations by page
    let mut by_page: BTreeMap<usize, Vec<&PdfAnnotation>> = BTreeMap::new();
    for annot in &report.annotations {
        by_page.entry(annot.page).or_default().push(annot);
    }

    for (page_num, annots) in by_page {
        md.push_str(&format!("## Page {}\n\n", page_num));

        for annot in annots {
            let type_label = format!("{}", annot.kind);
            let color_info = annot
                .color_hex
                .as_ref()
                .map(|c| format!(" ({})", c))
                .unwrap_or_default();

            match annot.kind {
                AnnotationKind::Highlight => {
                    md.push_str(&format!("> [!quote] Highlight{}\n", color_info));
                    if let Some(ref text) = annot.contents {
                        for line in text.lines() {
                            md.push_str(&format!("> {}\n", line));
                        }
                    } else {
                        md.push_str("> *(Highlighted passage without comment)*\n");
                    }
                }
                AnnotationKind::StickyNote => {
                    md.push_str(&format!("> [!note] Sticky Note{}\n", color_info));
                    if let Some(ref text) = annot.contents {
                        for line in text.lines() {
                            md.push_str(&format!("> {}\n", line));
                        }
                    } else {
                        md.push_str("> *(Empty note)*\n");
                    }
                }
                AnnotationKind::Underline => {
                    md.push_str(&format!("> [!tip] Underline{}\n", color_info));
                    if let Some(ref text) = annot.contents {
                        for line in text.lines() {
                            md.push_str(&format!("> {}\n", line));
                        }
                    }
                }
                _ => {
                    md.push_str(&format!("> [!info] {}{}\n", type_label, color_info));
                    if let Some(ref text) = annot.contents {
                        for line in text.lines() {
                            md.push_str(&format!("> {}\n", line));
                        }
                    }
                }
            }

            let mut meta_parts = Vec::new();
            if let Some(ref author) = annot.author {
                meta_parts.push(format!("**Author:** {}", author));
            }
            if let Some(ref date) = annot.creation_date {
                // Simplify PDF date string format if like D:20260918123000
                let clean_date = if date.starts_with("D:") && date.len() >= 10 {
                    format!("{}-{}-{}", &date[2..6], &date[6..8], &date[8..10])
                } else {
                    date.clone()
                };
                meta_parts.push(format!("**Date:** {}", clean_date));
            }

            if !meta_parts.is_empty() {
                md.push_str(&format!(">\n> {}\n", meta_parts.join(" · ")));
            }

            md.push('\n');
        }
    }

    md
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decode_pdf_string() {
        // UTF-8
        let utf8_bytes = b"Hello, PDF Annotation!";
        assert_eq!(
            decode_pdf_string(utf8_bytes),
            Some("Hello, PDF Annotation!".to_string())
        );

        // UTF-16BE with BOM
        let mut utf16_be = vec![0xFE, 0xFF];
        for ch in "Research".encode_utf16() {
            utf16_be.extend_from_slice(&ch.to_be_bytes());
        }
        assert_eq!(decode_pdf_string(&utf16_be), Some("Research".to_string()));
    }

    #[test]
    fn test_parse_color_rgb() {
        let arr = Object::Array(vec![
            Object::Real(1.0_f32),
            Object::Real(1.0_f32),
            Object::Real(0.0_f32),
        ]);
        assert_eq!(parse_color(&arr), Some("#ffff00".to_string()));

        let arr_cyan = Object::Array(vec![
            Object::Real(0.0_f32),
            Object::Real(0.8_f32),
            Object::Real(1.0_f32),
        ]);
        assert_eq!(parse_color(&arr_cyan), Some("#00ccff".to_string()));
    }

    #[test]
    fn test_annotations_to_markdown_formatting() {
        let report = PdfAnnotationReport {
            pdf_filename: "quantum_computing.pdf".to_string(),
            total_pages: 12,
            annotations: vec![
                PdfAnnotation {
                    page: 1,
                    kind: AnnotationKind::Highlight,
                    contents: Some("Qubits exhibit quantum superposition.".to_string()),
                    author: Some("Alice".to_string()),
                    color_hex: Some("#ffff00".to_string()),
                    rect: Some([10.0, 20.0, 100.0, 40.0]),
                    creation_date: Some("D:20260918120000".to_string()),
                },
                PdfAnnotation {
                    page: 3,
                    kind: AnnotationKind::StickyNote,
                    contents: Some("Verify threshold theorem bounds.".to_string()),
                    author: Some("Bob".to_string()),
                    color_hex: None,
                    rect: None,
                    creation_date: None,
                },
            ],
        };

        let md = annotations_to_markdown(&report);
        assert!(md.contains("title: \"Annotations: quantum_computing.pdf\""));
        assert!(md.contains("tags:\n  - pdf-annotations"));
        assert!(md.contains("## Page 1"));
        assert!(md.contains("> [!quote] Highlight (#ffff00)"));
        assert!(md.contains("Qubits exhibit quantum superposition."));
        assert!(md.contains("**Author:** Alice"));
        assert!(md.contains("**Date:** 2026-09-18"));
        assert!(md.contains("## Page 3"));
        assert!(md.contains("> [!note] Sticky Note"));
        assert!(md.contains("Verify threshold theorem bounds."));
        assert!(md.contains("**Author:** Bob"));
    }
}
