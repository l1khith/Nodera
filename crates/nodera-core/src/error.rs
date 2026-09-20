use std::path::PathBuf;
use thiserror::Error;

/// Root error type for all Nodera domain operations.
#[derive(Debug, Error)]
pub enum NoderaError {
    #[error("Vault error: {0}")]
    Vault(#[from] VaultError),

    #[error("File error: {0}")]
    File(#[from] FileError),

    #[error("Parse error: {0}")]
    Parse(#[from] ParseError),

    #[error("Index error: {0}")]
    Index(#[from] IndexError),

    #[error("Search error: {0}")]
    Search(#[from] SearchError),

    #[error("PDF error: {0}")]
    Pdf(#[from] PdfError),

    #[error("Validation error: {0}")]
    Validation(#[from] ValidationError),

    #[error("URI error: {0}")]
    Uri(#[from] UriError),

    #[error("Plugin error: {0}")]
    Plugin(#[from] PluginError),

    #[error("Operation cancelled: {message}")]
    OperationCancelled { message: String },
}

/// Errors related to vault discovery, opening, initialization, and structure.
#[derive(Debug, Error)]
pub enum VaultError {
    #[error("Vault path does not exist: {path}")]
    NotFound { path: PathBuf },

    #[error("Target path is not a directory: {path}")]
    NotADirectory { path: PathBuf },

    #[error("Directory is already a vault: {path}")]
    AlreadyExists { path: PathBuf },

    #[error("Invalid vault structure at {path}: {reason}")]
    InvalidStructure { path: PathBuf, reason: String },

    #[error("Path traversal detected: {attempted_path} escapes vault root {vault_root}")]
    PathTraversal {
        attempted_path: PathBuf,
        vault_root: PathBuf,
    },

    #[error("Vault configuration error: {reason}")]
    ConfigError { reason: String },
}

/// Errors occurring during file reads, writes, atomic operations, and metadata inspection.
#[derive(Debug, Error)]
pub enum FileError {
    #[error("File not found at: {path}")]
    NotFound { path: PathBuf },

    #[error("File already exists at: {path}")]
    AlreadyExists { path: PathBuf },

    #[error("Invalid filename '{filename}': {reason}")]
    InvalidFilename { filename: String, reason: String },

    #[error("Failed to read file {path}: {source}")]
    ReadFailed {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("Failed to write file {path}: {source}")]
    WriteFailed {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("Atomic write failed for {path} via temporary file {temp_path}: {source}")]
    AtomicWriteFailed {
        path: PathBuf,
        temp_path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("File content is not valid UTF-8 at {path}: {source}")]
    InvalidUtf8 {
        path: PathBuf,
        #[source]
        source: std::string::FromUtf8Error,
    },

    #[error("Filesystem operation failed for {path}: {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
}

/// Errors occurring during Markdown, Wikilink, frontmatter, or task parsing.
#[derive(Debug, Error)]
pub enum ParseError {
    #[error("Malformed frontmatter at line {line}: {reason}")]
    MalformedFrontmatter { line: usize, reason: String },

    #[error("Invalid Wikilink '{raw}': {reason}")]
    InvalidWikilink { raw: String, reason: String },

    #[error("Invalid task syntax at line {line}: {reason}")]
    InvalidTask { line: usize, reason: String },

    #[error("Markdown document parsing error: {reason}")]
    Markdown { reason: String },
}

/// Errors occurring during SQLite metadata or index operations.
#[derive(Debug, Error)]
pub enum IndexError {
    #[error("Index database error: {reason}")]
    Database { reason: String },

    #[error("Index schema corruption or mismatch at {path}: {reason}")]
    Corrupt { path: PathBuf, reason: String },

    #[error("Rebuild failed: {reason}")]
    RebuildFailed { reason: String },
}

/// Errors occurring during search queries or Tantivy operations.
#[derive(Debug, Error)]
pub enum SearchError {
    #[error("Invalid search query '{query}': {reason}")]
    InvalidQuery { query: String, reason: String },

    #[error("Search index error: {reason}")]
    Engine { reason: String },
}

/// Errors occurring during PDF conversion or structure extraction.
#[derive(Debug, Error)]
pub enum PdfError {
    #[error("Invalid or unreadable PDF file at {path}: {reason}")]
    InvalidPdf { path: PathBuf, reason: String },

    #[error("Invalid input at {path}: {reason}")]
    InvalidInput { path: PathBuf, reason: String },

    #[error("PDF document at {path} is encrypted: {reason}")]
    Encrypted { path: PathBuf, reason: String },

    #[error("Text extraction failed for {path} at page {page}: {reason}")]
    ExtractionFailed {
        path: PathBuf,
        page: usize,
        reason: String,
    },

    #[error("PDF conversion failed for {path} at page {page}: {reason}")]
    ConversionFailed {
        path: PathBuf,
        page: usize,
        reason: String,
    },

    #[error("Unsupported PDF feature in {path}: {feature}")]
    UnsupportedFeature { path: PathBuf, feature: String },

    #[error("PDF conversion timed out after {timeout_secs}s")]
    Timeout { timeout_secs: u64 },

    #[error("PDF conversion was cancelled by user")]
    Cancelled,
}

/// Errors validating user inputs, paths, settings, or identifiers.
#[derive(Debug, Error)]
pub enum ValidationError {
    #[error("Invalid note title: '{title}' ({reason})")]
    InvalidTitle { title: String, reason: String },

    #[error("Invalid path: '{path}' ({reason})")]
    InvalidPath { path: PathBuf, reason: String },

    #[error("Empty input: {field}")]
    EmptyInput { field: String },
}

/// Errors occurring during custom URI parsing, dispatch, and protocol handling.
#[derive(Debug, Error)]
pub enum UriError {
    #[error("Invalid scheme: {0}")]
    InvalidScheme(String),

    #[error("Missing parameter: {0}")]
    MissingParameter(String),

    #[error("Unsupported action: {0}")]
    UnsupportedAction(String),

    #[error("Protocol registration error: {0}")]
    RegistrationError(String),
}

/// Errors related to plugin discovery, validation, and execution.
#[derive(Debug, Error)]
pub enum PluginError {
    #[error("Plugin manifest error at {path}: {reason}")]
    ManifestError { path: PathBuf, reason: String },

    #[error("Invalid plugin identifier '{id}': {reason}")]
    InvalidId { id: String, reason: String },

    #[error("Plugin not found: '{id}'")]
    NotFound { id: String },

    #[error("Plugin execution error: {reason}")]
    ExecutionError { reason: String },
}

pub type Result<T> = std::result::Result<T, NoderaError>;

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_vault_error_display() {
        let err = VaultError::NotFound {
            path: PathBuf::from("/nonexistent/vault"),
        };
        assert!(err.to_string().contains("Vault path does not exist"));

        let root_err: NoderaError = err.into();
        assert!(root_err.to_string().contains("Vault error:"));
    }

    #[test]
    fn test_file_error_display() {
        let err = FileError::NotFound {
            path: PathBuf::from("Notes/Missing.md"),
        };
        assert!(err.to_string().contains("File not found"));

        let root_err: NoderaError = err.into();
        assert!(root_err.to_string().contains("File error:"));
    }

    #[test]
    fn test_path_traversal_error() {
        let err = VaultError::PathTraversal {
            attempted_path: PathBuf::from("../../../etc/passwd"),
            vault_root: PathBuf::from("/vault"),
        };
        let msg = err.to_string();
        assert!(msg.contains("Path traversal detected"));
        assert!(msg.contains("../../../etc/passwd"));
    }
}
