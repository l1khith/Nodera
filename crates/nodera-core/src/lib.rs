pub mod error;
pub mod fs;
pub mod logging;
pub mod note;
pub mod service;
pub mod vault;

pub use error::{
    FileError, IndexError, NoderaError, ParseError, PdfError, Result, SearchError, ValidationError,
    VaultError,
};
pub use fs::{
    atomic_write, atomic_write_str, delete_file, ensure_parent_dir, read_to_string,
    sanitize_filename, validate_filename,
};
pub use note::{Note, NoteId, NoteSummary, VaultEntry};
pub use service::VaultService;
pub use vault::{Vault, VaultConfig, CURRENT_VAULT_VERSION, DEFAULT_FOLDERS};
