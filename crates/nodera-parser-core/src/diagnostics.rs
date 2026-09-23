use std::fmt;

use serde::{Deserialize, Serialize};

use crate::span::SourceSpan;

/// Severity level of a parser diagnostic.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DiagnosticSeverity {
    /// Non-fatal hint or suggestion.
    Hint,
    /// Informational note.
    Info,
    /// Potential issue that did not prevent parsing.
    Warning,
    /// Syntax or semantic error encountered during parsing.
    Error,
}

impl fmt::Display for DiagnosticSeverity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Hint => write!(f, "hint"),
            Self::Info => write!(f, "info"),
            Self::Warning => write!(f, "warning"),
            Self::Error => write!(f, "error"),
        }
    }
}

/// A diagnostic message emitted during parsing or symbol extraction.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Diagnostic {
    /// Diagnostic severity level.
    pub severity: DiagnosticSeverity,
    /// Human-readable explanation of the diagnostic.
    pub message: String,
    /// Optional location where the issue occurred.
    pub span: Option<SourceSpan>,
    /// Optional machine-readable error code (e.g. "E0001", "SYNTAX_ERROR").
    pub code: Option<String>,
}

impl Diagnostic {
    /// Creates an error-level diagnostic.
    pub fn error(message: impl Into<String>, span: Option<SourceSpan>) -> Self {
        Self {
            severity: DiagnosticSeverity::Error,
            message: message.into(),
            span,
            code: None,
        }
    }

    /// Creates a warning-level diagnostic.
    pub fn warning(message: impl Into<String>, span: Option<SourceSpan>) -> Self {
        Self {
            severity: DiagnosticSeverity::Warning,
            message: message.into(),
            span,
            code: None,
        }
    }

    /// Creates an informational diagnostic.
    pub fn info(message: impl Into<String>, span: Option<SourceSpan>) -> Self {
        Self {
            severity: DiagnosticSeverity::Info,
            message: message.into(),
            span,
            code: None,
        }
    }

    /// Creates a hint-level diagnostic.
    pub fn hint(message: impl Into<String>, span: Option<SourceSpan>) -> Self {
        Self {
            severity: DiagnosticSeverity::Hint,
            message: message.into(),
            span,
            code: None,
        }
    }

    /// Attaches an error code to the diagnostic.
    pub fn with_code(mut self, code: impl Into<String>) -> Self {
        self.code = Some(code.into());
        self
    }
}

impl fmt::Display for Diagnostic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(span) = &self.span {
            write!(
                f,
                "{}:{}: [{}]: {}",
                span.start_line, span.start_col, self.severity, self.message
            )
        } else {
            write!(f, "[{}]: {}", self.severity, self.message)
        }
    }
}
