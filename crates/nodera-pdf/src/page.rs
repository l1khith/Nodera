/// Raw extracted text of a single PDF page before normalization.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PageText {
    pub page_number: usize,
    pub lines: Vec<String>,
}

impl PageText {
    pub fn new(page_number: usize, lines: Vec<String>) -> Self {
        Self { page_number, lines }
    }

    pub fn from_raw_text(page_number: usize, raw: &str) -> Self {
        let lines = raw
            .replace("\r\n", "\n")
            .replace('\r', "\n")
            .lines()
            .map(|s| s.to_string())
            .collect();
        Self { page_number, lines }
    }
}

/// Normalized page text with standardized whitespace, encoding, and line endings.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NormalizedPage {
    pub page_number: usize,
    pub lines: Vec<String>,
}

/// Cleaned page text with repeated headers, footers, and page numbers removed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CleanedPage {
    pub page_number: usize,
    pub lines: Vec<String>,
}

/// Semantic classification of a line of text within a structured document.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LineType {
    /// High-confidence chapter title (e.g. "Chapter 1: The Beginning").
    Chapter {
        number: String,
        title: Option<String>,
    },
    /// High-confidence numbered section (e.g. "1.1 Background", level = 2).
    Section {
        level: usize,
        number: String,
        title: String,
    },
    /// Formatted or detected heading.
    Heading { level: usize, text: String },
    /// Markdown list item (e.g. "- item" or "1. item").
    ListItem(String),
    /// Blockquote line (e.g. "> quote").
    Blockquote(String),
    /// Code fence or code block line.
    Code(String),
    /// Blank line separating paragraphs.
    Blank,
    /// Normal running paragraph text.
    Text(String),
}

/// A structured line of text with its semantic classification.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StructuredLine {
    pub page_number: usize,
    pub line_type: LineType,
}
