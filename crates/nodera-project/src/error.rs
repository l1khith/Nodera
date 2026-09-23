use std::path::PathBuf;
use thiserror::Error;

/// Error type for Rust project discovery, validation, and initialization.
#[derive(Debug, Error)]
pub enum ProjectError {
    #[error("Target path is not a directory: {path}")]
    NotADirectory { path: PathBuf },

    #[error("No Cargo.toml found at {path}. This directory does not appear to be a Rust project.")]
    NotARustProject { path: PathBuf },

    #[error("Manifest file not found at: {path}")]
    ManifestNotFound { path: PathBuf },

    #[error("Failed to parse Cargo manifest at {path}: {reason}")]
    ManifestParseError { path: PathBuf, reason: String },

    #[error("Nodera project already initialized at: {path}")]
    AlreadyInitialized { path: PathBuf },

    #[error("I/O error for {path}: {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("Invalid project layout at {path}: {reason}")]
    InvalidStructure { path: PathBuf, reason: String },

    #[error("Path traversal detected: {path} escapes project root {root}")]
    TraversalError { path: PathBuf, root: PathBuf },

    #[error("This directory is not an initialized Nodera project. No .nodera/project.toml was found at {path}.\n\nRun:\n  nodera init")]
    NotInitialized { path: PathBuf },

    #[error("Project index database error: {reason}")]
    Database { reason: String },

    #[error("Serialization error: {reason}")]
    Serialization { reason: String },

    #[error("Failed to parse source file at {path}: {reason}")]
    Parser { path: PathBuf, reason: String },

    #[error("Corrupted project state at {path}: {reason}")]
    CorruptedState { path: PathBuf, reason: String },
}

pub type Result<T> = std::result::Result<T, ProjectError>;
