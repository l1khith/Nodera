use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::diagnostics::{Diagnostic, DiagnosticSeverity};
use crate::span::SourceSpan;
use crate::traits::LanguageId;

/// Classification of symbols discovered in source code.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SymbolKind {
    Function,
    Method,
    Struct,
    Enum,
    EnumVariant,
    Trait,
    Interface,
    Implementation,
    Module,
    TypeAlias,
    Constant,
    Static,
    Variable,
    Field,
    Macro,
    Other(String),
}

/// Declared visibility of a symbol.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Visibility {
    /// Public to all consumers (`pub`).
    Public,
    /// Visible within the defining crate or package (`pub(crate)`).
    Crate,
    /// Restricted visibility path (e.g. `pub(super)`, `pub(in path)`).
    Restricted(String),
    /// Private to the enclosing scope or module.
    Private,
}

impl Visibility {
    /// Returns true if the symbol is accessible outside its defining module.
    pub fn is_public(&self) -> bool {
        matches!(self, Self::Public)
    }
}

/// A parsed symbol declaration (function, struct, enum, trait, etc.).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Symbol {
    /// Stable canonical identifier (e.g. "crate::state::AppState::select_note").
    pub id: String,
    /// Identifier name of the symbol (e.g. "select_note").
    pub name: String,
    /// Categorical symbol kind.
    pub kind: SymbolKind,
    /// Declared visibility.
    pub visibility: Visibility,
    /// Exact source location span.
    pub span: SourceSpan,
    /// Full declaration signature string.
    pub signature: Option<String>,
    /// Cleaned documentation comments.
    pub doc_comment: Option<String>,
    /// Identifier of the enclosing parent symbol, if nested.
    pub parent_id: Option<String>,
    /// Child symbols (e.g. methods on a struct/trait, enum variants, struct fields).
    pub children: Vec<Symbol>,
}

impl Symbol {
    /// Recursively flattens this symbol and all its descendants.
    pub fn flatten(&self) -> Vec<&Symbol> {
        let mut list = vec![self];
        for child in &self.children {
            list.extend(child.flatten());
        }
        list
    }
}

/// Nature of a cross-symbol reference or usage.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReferenceKind {
    /// Function or method invocation.
    Call,
    /// Type annotation, generic argument, or return type.
    TypeUsage,
    /// Imported in a use or import statement.
    Import,
    /// Macro invocation (e.g. `println!`, `rsx!`).
    MacroInvocation,
    /// Trait implementation.
    TraitImplementation,
    /// Other reference pattern.
    Other(String),
}

/// A reference from this source file to another symbol.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SymbolReference {
    /// Target symbol identifier or path (e.g. "std::path::PathBuf", "select_note").
    pub target: String,
    /// Nature of the reference.
    pub kind: ReferenceKind,
    /// Source location span of the reference.
    pub span: SourceSpan,
    /// Enclosing symbol identifier from which the reference originated.
    pub caller_symbol_id: Option<String>,
}

/// An import or use declaration in the source file.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ImportDeclaration {
    /// Base module or package path (e.g. "std::collections::HashMap").
    pub path: String,
    /// Items imported from the module.
    pub imported_items: Vec<String>,
    /// Optional rename or alias (e.g. `as MyAlias`).
    pub alias: Option<String>,
    /// Source location span of the import statement.
    pub span: SourceSpan,
    /// True if this is a glob / wildcard import (`*`).
    pub is_glob: bool,
}

/// Basic statistical metrics for a parsed source file.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct SourceMetrics {
    /// Total lines in the file.
    pub total_lines: usize,
    /// Source lines of code (excluding blank lines and comment-only lines).
    pub code_lines: usize,
    /// Lines containing comments or documentation.
    pub comment_lines: usize,
    /// Empty or whitespace-only lines.
    pub blank_lines: usize,
    /// Total number of symbols discovered (including nested symbols).
    pub symbol_count: usize,
}

/// Normalized Intermediate Representation of a parsed source file.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceFile {
    /// Path of the source file.
    pub path: PathBuf,
    /// Language identifier.
    pub language_id: LanguageId,
    /// Top-level symbols discovered in the file.
    pub symbols: Vec<Symbol>,
    /// Symbol references and calls.
    pub references: Vec<SymbolReference>,
    /// Import declarations.
    pub imports: Vec<ImportDeclaration>,
    /// Statistical metrics.
    pub metrics: SourceMetrics,
    /// Diagnostics (errors, warnings, hints) emitted during parsing.
    pub diagnostics: Vec<Diagnostic>,
}

impl SourceFile {
    /// Creates a new empty `SourceFile` for the given path and language.
    pub fn new(path: impl Into<PathBuf>, language_id: LanguageId) -> Self {
        Self {
            path: path.into(),
            language_id,
            symbols: Vec::new(),
            references: Vec::new(),
            imports: Vec::new(),
            metrics: SourceMetrics::default(),
            diagnostics: Vec::new(),
        }
    }

    /// Checks if any diagnostic with severity `Error` was recorded.
    pub fn has_errors(&self) -> bool {
        self.diagnostics
            .iter()
            .any(|d| d.severity == DiagnosticSeverity::Error)
    }

    /// Flattens all top-level symbols and their recursive children into a single list.
    pub fn all_symbols_flat(&self) -> Vec<&Symbol> {
        let mut list = Vec::new();
        for sym in &self.symbols {
            list.extend(sym.flatten());
        }
        list
    }

    /// Finds a symbol by its canonical ID across top-level and nested symbols.
    pub fn find_symbol_by_id(&self, id: &str) -> Option<&Symbol> {
        self.all_symbols_flat().into_iter().find(|s| s.id == id)
    }

    /// Returns all symbols of the specified kind.
    pub fn symbols_by_kind(&self, kind: SymbolKind) -> Vec<&Symbol> {
        self.all_symbols_flat()
            .into_iter()
            .filter(|s| s.kind == kind)
            .collect()
    }

    /// Returns the file path reference.
    pub fn path(&self) -> &Path {
        &self.path
    }
}
