use std::path::PathBuf;

use thiserror::Error;

/// Crate-level error type for source parsing operations.
#[derive(Debug, Error)]
pub enum ParserError {
    #[error("I/O error while reading source file {path}: {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("Source file {path} exceeds maximum permitted size: {size_bytes} > {max_bytes} bytes")]
    FileTooLarge {
        path: PathBuf,
        size_bytes: usize,
        max_bytes: usize,
    },

    #[error("No parser registered for language: {language}")]
    UnsupportedLanguage { language: String },

    #[error("No parser registered for file extension: .{extension}")]
    UnsupportedExtension { extension: String },

    #[error("Fatal syntax error in {path} at {line:?}:{column:?}: {message}")]
    SyntaxFatal {
        path: PathBuf,
        message: String,
        line: Option<usize>,
        column: Option<usize>,
    },

    #[error("Parser error: {0}")]
    Other(String),
}

/// Specialized Result alias for parser operations.
pub type Result<T> = std::result::Result<T, ParserError>;
