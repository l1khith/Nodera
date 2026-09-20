pub mod citations;
pub mod error;
pub mod fs;
pub mod logging;
pub mod note;
pub mod plugin;
pub mod progress;
pub mod service;
pub mod uri;
pub mod vault;

pub use citations::{clean_latex, parse_authors, parse_bibtex_str, BibEntry, BibLibrary};
pub use error::{
    FileError, IndexError, NoderaError, ParseError, PdfError, PluginError, Result, SearchError,
    UriError, ValidationError, VaultError,
};
pub use fs::{
    atomic_write, atomic_write_str, delete_file, ensure_parent_dir, read_to_string,
    sanitize_filename, validate_filename,
};
pub use note::{Note, NoteId, NoteSummary, VaultEntry};
pub use plugin::{
    HookExecutionResult, PluginCommand, PluginHook, PluginManager, PluginManifest, PluginPermission,
};
pub use progress::{IndexingPhase, IndexingProgress};
pub use service::{TrashedNoteSummary, VaultService};
pub use uri::{
    is_windows_protocol_registered, parse_query_string, percent_decode, percent_encode,
    register_windows_protocol, unregister_windows_protocol, NoderaUri,
};
pub use vault::{Vault, VaultConfig, CURRENT_VAULT_VERSION, DEFAULT_FOLDERS};
