use std::fmt;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::error::{ParserError, Result};
use crate::ir::SourceFile;

/// Canonical identifier for supported programming languages.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LanguageId {
    Rust,
    Python,
    TypeScript,
    JavaScript,
    Go,
    C,
    Cpp,
    Custom(String),
}

impl LanguageId {
    /// Returns the canonical string identifier for the language.
    pub fn as_str(&self) -> &str {
        match self {
            Self::Rust => "rust",
            Self::Python => "python",
            Self::TypeScript => "typescript",
            Self::JavaScript => "javascript",
            Self::Go => "go",
            Self::C => "c",
            Self::Cpp => "cpp",
            Self::Custom(s) => s.as_str(),
        }
    }

    /// Infers language identifier from a file extension (without leading dot).
    pub fn from_extension(ext: &str) -> Option<Self> {
        match ext.to_ascii_lowercase().as_str() {
            "rs" => Some(Self::Rust),
            "py" | "pyw" => Some(Self::Python),
            "ts" | "tsx" => Some(Self::TypeScript),
            "js" | "jsx" | "mjs" | "cjs" => Some(Self::JavaScript),
            "go" => Some(Self::Go),
            "c" | "h" => Some(Self::C),
            "cpp" | "cxx" | "cc" | "hpp" | "hxx" => Some(Self::Cpp),
            _ => None,
        }
    }
}

impl fmt::Display for LanguageId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Options controlling parser execution and feature extraction.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ParseOptions {
    /// Whether to extract documentation comments from declarations.
    pub extract_doc_comments: bool,
    /// Whether to perform AST traversal to extract references and calls.
    pub extract_references: bool,
    /// Whether to calculate line-based source code metrics (LOC, SLOC, etc.).
    pub extract_metrics: bool,
    /// Maximum allowed file size in bytes before aborting (default: 5 MB).
    pub max_file_size_bytes: usize,
    /// Whether to include private or crate-internal symbols in the IR.
    pub include_private_symbols: bool,
    /// In resilient mode, syntax errors produce diagnostics instead of aborting.
    pub resilient: bool,
}

impl Default for ParseOptions {
    fn default() -> Self {
        Self {
            extract_doc_comments: true,
            extract_references: true,
            extract_metrics: true,
            max_file_size_bytes: 5 * 1024 * 1024, // 5 MB
            include_private_symbols: true,
            resilient: true,
        }
    }
}

/// Core trait implemented by language-specific source code parsers.
pub trait SourceParser: Send + Sync {
    /// Returns the unique language ID handled by this parser.
    fn language_id(&self) -> LanguageId;

    /// Returns the list of file extensions handled by this parser (e.g. `["rs"]`).
    fn supported_extensions(&self) -> &[&str];

    /// Parses source code text and returns the Normalized Intermediate Representation.
    fn parse_source(&self, path: &Path, source: &str, options: &ParseOptions)
        -> Result<SourceFile>;

    /// Convenience method to parse a source file directly from disk (strictly read-only).
    fn parse_file(&self, path: &Path, options: &ParseOptions) -> Result<SourceFile> {
        let metadata = std::fs::metadata(path).map_err(|e| ParserError::Io {
            path: path.to_path_buf(),
            source: e,
        })?;

        let size = metadata.len() as usize;
        if size > options.max_file_size_bytes {
            return Err(ParserError::FileTooLarge {
                path: path.to_path_buf(),
                size_bytes: size,
                max_bytes: options.max_file_size_bytes,
            });
        }

        let content = std::fs::read_to_string(path).map_err(|e| ParserError::Io {
            path: path.to_path_buf(),
            source: e,
        })?;

        self.parse_source(path, &content, options)
    }
}
