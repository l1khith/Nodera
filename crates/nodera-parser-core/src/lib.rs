pub mod diagnostics;
pub mod error;
pub mod ir;
pub mod languages;
pub mod registry;
pub mod span;
pub mod traits;

pub use diagnostics::{Diagnostic, DiagnosticSeverity};
pub use error::{ParserError, Result};
pub use ir::{
    ImportDeclaration, ReferenceKind, SourceFile, SourceMetrics, Symbol, SymbolKind,
    SymbolReference, Visibility,
};
pub use languages::RustParser;
pub use registry::ParserRegistry;
pub use span::{LineIndex, SourceSpan};
pub use traits::{LanguageId, ParseOptions, SourceParser};
