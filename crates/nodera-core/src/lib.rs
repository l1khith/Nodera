pub mod citations;
pub mod error;
pub mod fs;
pub mod logging;
pub mod note;
pub mod progress;
pub mod service;
pub mod vault;

pub use citations::{clean_latex, parse_authors, parse_bibtex_str, BibEntry, BibLibrary};
pub use error::{
    FileError, IndexError, NoderaError, ParseError, PdfError, Result, SearchError, ValidationError,
    VaultError,
};
pub use fs::{
    atomic_write, atomic_write_str, delete_file, ensure_parent_dir, read_to_string,
    sanitize_filename, validate_filename,
};
pub use note::{Note, NoteId, NoteSummary, VaultEntry};
pub use progress::{IndexingPhase, IndexingProgress};
pub use service::{TrashedNoteSummary, VaultService};
pub use vault::{Vault, VaultConfig, CURRENT_VAULT_VERSION, DEFAULT_FOLDERS};
