use crate::models::ConversionOptions;
use crate::page::{CleanedPage, LineType, NormalizedPage, PageText, StructuredLine};
use regex::Regex;
use std::collections::{HashMap, HashSet};
use std::sync::LazyLock;

static REGEX_CHAPTER: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)^CHAPTER\s+([0-9IVXLCDM]+)(?:\s*[:\-—.]\s*(.*))?$")
        .expect("Valid chapter regex")
});

static REGEX_SECTION: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^(\d+\.\d+(?:\.\d+)*)\s+([A-Z].*)$").expect("Valid section regex")
});

static REGEX_PAGE_NUMBER_DIGITS: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^\s*(\d{1,6})\s*$").expect("Valid page number digits regex"));

static REGEX_PAGE_NUMBER_DASH: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^\s*[-—~]\s*(\d{1,6})\s*[-—~]\s*$").expect("Valid page number dash regex")
});

static REGEX_PAGE_NUMBER_ROMAN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^\s*([ivxlcdmIVXLCDM]{1,10})\s*$").expect("Valid roman numerals regex")
});

static REGEX_PAGE_NUMBER_LABEL: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)^\s*Page\s+\d{1,6}(?:\s+of\s+\d{1,6})?\s*$").expect("Valid page label regex")
});

static REGEX_LIST_ITEM: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^(\s*[-*+]\s+|\s*\d+\.\s+)").expect("Valid list item regex"));

/// Standard major section titles that indicate headings when on an isolated short line.
const STANDARD_HEADINGS: &[&str] = &[
    "ABSTRACT",
    "ACKNOWLEDGEMENTS",
    "ACKNOWLEDGMENTS",
    "APPENDIX",
    "BIBLIOGRAPHY",
    "CONCLUSION",
    "CONTENTS",
    "EPILOGUE",
    "FOREWORD",
    "GLOSSARY",
    "INDEX",
    "INTRODUCTION",
    "PREFACE",
    "PROLOGUE",
    "REFERENCES",
    "TABLE OF CONTENTS",
];

/// Normalizes raw page text:
/// - Converts CRLF and CR to LF
/// - Converts non-breaking spaces to standard spaces
/// - Strips unprintable control characters
/// - Trims trailing whitespace while preserving leading indentation
pub fn normalize_page(page: &PageText) -> NormalizedPage {
    let mut normalized_lines = Vec::with_capacity(page.lines.len());

    for line in &page.lines {
        let converted = line.replace('\u{00A0}', " ");
        let cleaned: String = converted
            .chars()
            .filter(|c| !c.is_control() || *c == '\t')
            .collect();
        let trimmed_end = cleaned.trim_end().to_string();
        normalized_lines.push(trimmed_end);
    }

    NormalizedPage {
        page_number: page.page_number,
        lines: normalized_lines,
    }
}

/// Identifies recurring running headers and footers across pages based on positional repetition.
pub fn detect_running_headers_and_footers(
    pages: &[NormalizedPage],
) -> (HashSet<String>, HashSet<String>) {
    let mut headers = HashSet::new();
    let mut footers = HashSet::new();

    if pages.len() < 3 {
        return (headers, footers);
    }

    let mut top_line_counts: HashMap<String, usize> = HashMap::new();
    let mut bottom_line_counts: HashMap<String, usize> = HashMap::new();

    for page in pages {
        let non_empty: Vec<&str> = page
            .lines
            .iter()
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .collect();

        if let Some(&first) = non_empty.first() {
            if first.len() <= 120 && !is_chapter_title(first) {
                *top_line_counts.entry(first.to_string()).or_insert(0) += 1;
            }
        }

        if let Some(&last) = non_empty.last() {
            if last.len() <= 120 && !is_chapter_title(last) {
                *bottom_line_counts.entry(last.to_string()).or_insert(0) += 1;
            }
        }
    }

    let threshold = ((pages.len() as f64) * 0.35).max(3.0) as usize;

    for (line, count) in top_line_counts {
        if count >= threshold {
            headers.insert(line);
        }
    }

    for (line, count) in bottom_line_counts {
        if count >= threshold {
            footers.insert(line);
        }
    }

    (headers, footers)
}

fn is_chapter_title(line: &str) -> bool {
    REGEX_CHAPTER.is_match(line)
}

/// Checks if a trimmed line is a standalone page number.
pub fn is_page_number_candidate(line: &str) -> bool {
    let trimmed = line.trim();
    if trimmed.is_empty() {
        return false;
    }

    REGEX_PAGE_NUMBER_DIGITS.is_match(trimmed)
        || REGEX_PAGE_NUMBER_DASH.is_match(trimmed)
        || REGEX_PAGE_NUMBER_LABEL.is_match(trimmed)
        || (trimmed.len() <= 8 && is_valid_roman_numeral(trimmed))
}

fn is_valid_roman_numeral(s: &str) -> bool {
    let trimmed = s.trim();
    if trimmed.is_empty() || trimmed.len() > 8 {
        return false;
    }
    REGEX_PAGE_NUMBER_ROMAN.is_match(trimmed)
}

/// Cleans normalized pages by removing repeated headers, footers, and standalone page numbers.
pub fn clean_pages(pages: Vec<NormalizedPage>, options: &ConversionOptions) -> Vec<CleanedPage> {
    let (repeated_headers, repeated_footers) = if options.remove_repeated_headers {
        detect_running_headers_and_footers(&pages)
    } else {
        (HashSet::new(), HashSet::new())
    };

    pages
        .into_iter()
        .map(|page| {
            let mut cleaned_lines = Vec::with_capacity(page.lines.len());
            let line_count = page.lines.len();

            for (idx, line) in page.lines.iter().enumerate() {
                let trimmed = line.trim();
                if trimmed.is_empty() {
                    cleaned_lines.push(String::new());
                    continue;
                }

                let is_near_top = idx <= 2;
                let is_near_bottom = line_count > 0 && idx >= line_count.saturating_sub(3);

                // Check repeated header
                if options.remove_repeated_headers
                    && is_near_top
                    && repeated_headers.contains(trimmed)
                {
                    continue;
                }

                // Check repeated footer
                if options.remove_repeated_headers
                    && is_near_bottom
                    && repeated_footers.contains(trimmed)
                {
                    continue;
                }

                // Check standalone page number at page edges
                if options.remove_page_numbers
                    && (is_near_top || is_near_bottom)
                    && is_page_number_candidate(trimmed)
                {
                    continue;
                }

                cleaned_lines.push(line.clone());
            }

            CleanedPage {
                page_number: page.page_number,
                lines: cleaned_lines,
            }
        })
        .collect()
}

/// Classifies a line of text into its semantic `LineType`.
pub fn classify_line(line: &str, options: &ConversionOptions) -> LineType {
    let trimmed = line.trim();
    if trimmed.is_empty() {
        return LineType::Blank;
    }

    if trimmed.starts_with("```") || trimmed.starts_with("~~~") {
        return LineType::Code(line.to_string());
    }

    if trimmed.starts_with("> ") {
        return LineType::Blockquote(line.to_string());
    }

    if REGEX_LIST_ITEM.is_match(trimmed) {
        return LineType::ListItem(line.to_string());
    }

    if options.detect_headings {
        // 1. High-confidence Chapter pattern
        if let Some(caps) = REGEX_CHAPTER.captures(trimmed) {
            let number = caps
                .get(1)
                .map(|m| m.as_str().to_string())
                .unwrap_or_default();
            let title = caps
                .get(2)
                .map(|m| m.as_str().trim().to_string())
                .filter(|s| !s.is_empty());
            return LineType::Chapter { number, title };
        }

        // 2. High-confidence numbered section (e.g. "1.1 Introduction")
        if let Some(caps) = REGEX_SECTION.captures(trimmed) {
            let number = caps
                .get(1)
                .map(|m| m.as_str().to_string())
                .unwrap_or_default();
            let title = caps
                .get(2)
                .map(|m| m.as_str().trim().to_string())
                .unwrap_or_default();
            let dot_count = number.chars().filter(|&c| c == '.').count();
            let level = options.heading_prefix_depth + dot_count;
            return LineType::Section {
                level: level.clamp(1, 6),
                number,
                title,
            };
        }

        // 3. Known standard headings (e.g. "INTRODUCTION")
        let upper = trimmed.to_uppercase();
        if STANDARD_HEADINGS.contains(&upper.as_str()) {
            return LineType::Heading {
                level: options.heading_prefix_depth,
                text: trimmed.to_string(),
            };
        }
    }

    LineType::Text(line.to_string())
}

/// Conservative paragraph reconstruction from structured lines.
///
/// Joins lines of a continuous paragraph with spaces while preserving blank lines,
/// headings, lists, code fences, and blockquotes.
pub fn reconstruct_paragraphs(structured_lines: Vec<StructuredLine>) -> Vec<String> {
    let mut output_lines = Vec::new();
    let mut current_paragraph: Option<String> = None;

    let flush_paragraph = |output: &mut Vec<String>, current: &mut Option<String>| {
        if let Some(para) = current.take() {
            let trimmed = para.trim();
            if !trimmed.is_empty() {
                output.push(trimmed.to_string());
            }
        }
    };

    for item in structured_lines {
        match item.line_type {
            LineType::Blank => {
                flush_paragraph(&mut output_lines, &mut current_paragraph);
                if !output_lines.last().map(|s| s.is_empty()).unwrap_or(true) {
                    output_lines.push(String::new());
                }
            }
            LineType::Chapter { number, title } => {
                flush_paragraph(&mut output_lines, &mut current_paragraph);
                if !output_lines.last().map(|s| s.is_empty()).unwrap_or(true) {
                    output_lines.push(String::new());
                }
                let heading = match title {
                    Some(t) => format!("## Chapter {}: {}", number, t),
                    None => format!("## Chapter {}", number),
                };
                output_lines.push(heading);
                output_lines.push(String::new());
            }
            LineType::Section {
                level,
                number,
                title,
            } => {
                flush_paragraph(&mut output_lines, &mut current_paragraph);
                if !output_lines.last().map(|s| s.is_empty()).unwrap_or(true) {
                    output_lines.push(String::new());
                }
                let hashes = "#".repeat(level);
                output_lines.push(format!("{} {} {}", hashes, number, title));
                output_lines.push(String::new());
            }
            LineType::Heading { level, text } => {
                flush_paragraph(&mut output_lines, &mut current_paragraph);
                if !output_lines.last().map(|s| s.is_empty()).unwrap_or(true) {
                    output_lines.push(String::new());
                }
                let hashes = "#".repeat(level);
                output_lines.push(format!("{} {}", hashes, text));
                output_lines.push(String::new());
            }
            LineType::ListItem(item_text) => {
                flush_paragraph(&mut output_lines, &mut current_paragraph);
                output_lines.push(item_text);
            }
            LineType::Blockquote(quote_text) => {
                flush_paragraph(&mut output_lines, &mut current_paragraph);
                output_lines.push(quote_text);
            }
            LineType::Code(code_text) => {
                flush_paragraph(&mut output_lines, &mut current_paragraph);
                output_lines.push(code_text);
            }
            LineType::Text(text) => {
                let trimmed = text.trim();
                if trimmed.is_empty() {
                    continue;
                }

                if let Some(ref mut para) = current_paragraph {
                    // Check for hyphenation at line end (e.g. "sys-", "pro-")
                    if para.ends_with('-') && !para.ends_with(" -") {
                        para.pop(); // remove trailing hyphen
                        para.push_str(trimmed);
                    } else {
                        para.push(' ');
                        para.push_str(trimmed);
                    }
                } else {
                    current_paragraph = Some(trimmed.to_string());
                }
            }
        }
    }

    flush_paragraph(&mut output_lines, &mut current_paragraph);

    // Clean up any excessive trailing blank lines
    while output_lines.last().map(|s| s.is_empty()).unwrap_or(false) {
        output_lines.pop();
    }

    output_lines
}

/// Generates valid YAML frontmatter escaping strings as necessary.
pub fn generate_frontmatter(
    title: &str,
    source_filename: &str,
    pages: usize,
    chapters: usize,
) -> String {
    let escaped_title = title.replace('\\', "\\\\").replace('"', "\\\"");
    let escaped_source = source_filename.replace('\\', "\\\\").replace('"', "\\\"");

    format!(
        "---\ntitle: \"{}\"\nsource: \"{}\"\npages: {}\nchapters: {}\nimported_by: nodera\ntags:\n  - book\n  - pdf-import\n---\n\n",
        escaped_title, escaped_source, pages, chapters
    )
}

/// Converts cleaned pages into Markdown text with YAML frontmatter, paragraph unwrapping,
/// and optional deterministic page markers.
pub fn generate_markdown(
    pages: Vec<CleanedPage>,
    options: &ConversionOptions,
    title: &str,
    source_filename: &str,
) -> (String, usize) {
    let mut structured_lines = Vec::new();
    let mut chapters_detected = 0;
    let total_pages = pages.len();

    for page in pages {
        if options.add_page_markers {
            structured_lines.push(StructuredLine {
                page_number: page.page_number,
                line_type: LineType::Text(format!("<!-- nodera:page={} -->", page.page_number)),
            });
            structured_lines.push(StructuredLine {
                page_number: page.page_number,
                line_type: LineType::Blank,
            });
        }

        for line in page.lines {
            let line_type = classify_line(&line, options);
            if let LineType::Chapter { .. } = &line_type {
                chapters_detected += 1;
            }
            structured_lines.push(StructuredLine {
                page_number: page.page_number,
                line_type,
            });
        }
    }

    let reconstructed = reconstruct_paragraphs(structured_lines);
    let frontmatter = generate_frontmatter(title, source_filename, total_pages, chapters_detected);
    let body = reconstructed.join("\n");

    (format!("{}{}", frontmatter, body), chapters_detected)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_page() {
        let raw = PageText::new(
            1,
            vec![
                "Hello \u{00A0} World\r\n".into(),
                "Trailing whitespace   ".into(),
                "\x07Control char".into(),
            ],
        );
        let norm = normalize_page(&raw);
        assert_eq!(norm.lines[0], "Hello   World");
        assert_eq!(norm.lines[1], "Trailing whitespace");
        assert_eq!(norm.lines[2], "Control char");
    }

    #[test]
    fn test_page_number_detection() {
        assert!(is_page_number_candidate("42"));
        assert!(is_page_number_candidate(" - 42 - "));
        assert!(is_page_number_candidate("— 12 —"));
        assert!(is_page_number_candidate("Page 42"));
        assert!(is_page_number_candidate("Page 42 of 600"));
        assert!(is_page_number_candidate("iv"));
        assert!(is_page_number_candidate("XII"));

        // Valid content numbers should NOT be considered standalone page numbers
        assert!(!is_page_number_candidate("Section 42 of the criminal code"));
        assert!(!is_page_number_candidate("42 is the answer to life."));
        assert!(!is_page_number_candidate("The year was 1984."));
    }

    #[test]
    fn test_repeated_header_detection_and_cleanup() {
        let pages = vec![
            NormalizedPage {
                page_number: 1,
                lines: vec![
                    "MY BOOK TITLE".into(),
                    "Some content page 1".into(),
                    "42".into(),
                ],
            },
            NormalizedPage {
                page_number: 2,
                lines: vec![
                    "MY BOOK TITLE".into(),
                    "Some content page 2".into(),
                    "43".into(),
                ],
            },
            NormalizedPage {
                page_number: 3,
                lines: vec![
                    "MY BOOK TITLE".into(),
                    "Some content page 3".into(),
                    "44".into(),
                ],
            },
        ];

        let opts = ConversionOptions {
            remove_repeated_headers: true,
            remove_page_numbers: true,
            ..Default::default()
        };

        let cleaned = clean_pages(pages, &opts);
        assert_eq!(cleaned.len(), 3);

        for page in cleaned {
            assert!(!page.lines.contains(&"MY BOOK TITLE".to_string()));
            assert!(!page.lines.contains(&"42".to_string()));
            assert!(!page.lines.contains(&"43".to_string()));
            assert!(!page.lines.contains(&"44".to_string()));
        }
    }

    #[test]
    fn test_heading_and_chapter_detection() {
        let opts = ConversionOptions::default();

        let ch = classify_line("Chapter 1: The Beginning", &opts);
        match ch {
            LineType::Chapter { number, title } => {
                assert_eq!(number, "1");
                assert_eq!(title, Some("The Beginning".to_string()));
            }
            _ => panic!("Expected Chapter line type"),
        }

        let sec = classify_line("1.2 Background Information", &opts);
        match sec {
            LineType::Section {
                level,
                number,
                title,
            } => {
                assert_eq!(level, 3);
                assert_eq!(number, "1.2");
                assert_eq!(title, "Background Information");
            }
            _ => panic!("Expected Section line type"),
        }

        let intro = classify_line("INTRODUCTION", &opts);
        match intro {
            LineType::Heading { level, text } => {
                assert_eq!(level, 2);
                assert_eq!(text, "INTRODUCTION");
            }
            _ => panic!("Expected Heading line type"),
        }
    }

    #[test]
    fn test_paragraph_reconstruction() {
        let structured = vec![
            StructuredLine {
                page_number: 1,
                line_type: LineType::Text("Rust is a systems programming".into()),
            },
            StructuredLine {
                page_number: 1,
                line_type: LineType::Text("language designed for safety.".into()),
            },
            StructuredLine {
                page_number: 1,
                line_type: LineType::Blank,
            },
            StructuredLine {
                page_number: 1,
                line_type: LineType::Text("It provides memory safety without".into()),
            },
            StructuredLine {
                page_number: 1,
                line_type: LineType::Text("a garbage collector.".into()),
            },
        ];

        let paras = reconstruct_paragraphs(structured);
        assert_eq!(paras.len(), 3); // para 1, blank, para 2
        assert_eq!(
            paras[0],
            "Rust is a systems programming language designed for safety."
        );
        assert_eq!(paras[1], "");
        assert_eq!(
            paras[2],
            "It provides memory safety without a garbage collector."
        );
    }

    #[test]
    fn test_hyphenated_line_join() {
        let structured = vec![
            StructuredLine {
                page_number: 1,
                line_type: LineType::Text("This is an unexpect-".into()),
            },
            StructuredLine {
                page_number: 1,
                line_type: LineType::Text("ed breakthrough.".into()),
            },
        ];

        let paras = reconstruct_paragraphs(structured);
        assert_eq!(paras.len(), 1);
        assert_eq!(paras[0], "This is an unexpected breakthrough.");
    }

    #[test]
    fn test_page_markers_when_enabled() {
        let pages = vec![CleanedPage {
            page_number: 1,
            lines: vec!["Page one content.".into()],
        }];

        let opts = ConversionOptions {
            add_page_markers: true,
            ..Default::default()
        };

        let (md, _) = generate_markdown(pages, &opts, "Test", "test.pdf");
        assert!(md.contains("<!-- nodera:page=1 -->"));
    }
}
