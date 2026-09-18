//! PDF extraction and conversion crate for Nodera.
//!
//! Provides a pure-Rust PDF-to-Markdown conversion pipeline backed by `lopdf`,
//! featuring conservative text normalization, running header/footer removal,
//! page number stripping, heading/chapter detection, and paragraph reconstruction.

pub mod annotations;
pub mod cleanup;
pub mod engine;
pub mod models;
pub mod page;
pub mod service;
pub mod traits;

pub use annotations::*;
pub use cleanup::*;
pub use engine::*;
pub use models::*;
pub use page::*;
pub use service::*;
pub use traits::*;
