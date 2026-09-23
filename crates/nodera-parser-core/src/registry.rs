use std::collections::HashMap;
use std::path::Path;

use crate::error::{ParserError, Result};
use crate::ir::SourceFile;
use crate::languages::RustParser;
use crate::traits::{LanguageId, ParseOptions, SourceParser};

/// Central registry managing language-specific parsers.
pub struct ParserRegistry {
    parsers: HashMap<LanguageId, Box<dyn SourceParser>>,
    extension_map: HashMap<String, LanguageId>,
}

impl Default for ParserRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl ParserRegistry {
    /// Creates a new empty parser registry.
    pub fn new() -> Self {
        Self {
            parsers: HashMap::new(),
            extension_map: HashMap::new(),
        }
    }

    /// Creates a registry pre-configured with default built-in parsers (e.g. Rust).
    pub fn with_defaults() -> Self {
        let mut registry = Self::new();
        registry.register_parser(Box::new(RustParser::new()));
        registry
    }

    /// Registers a parser implementation.
    pub fn register_parser(&mut self, parser: Box<dyn SourceParser>) {
        let lang = parser.language_id();
        for &ext in parser.supported_extensions() {
            self.extension_map
                .insert(ext.to_ascii_lowercase(), lang.clone());
        }
        self.parsers.insert(lang, parser);
    }

    /// Retrieves a parser by its language identifier.
    pub fn get_by_language(&self, lang: &LanguageId) -> Option<&dyn SourceParser> {
        self.parsers.get(lang).map(|p| p.as_ref())
    }

    /// Retrieves a parser for a given file extension (e.g. "rs").
    pub fn get_by_extension(&self, extension: &str) -> Option<&dyn SourceParser> {
        let cleaned = extension.trim_start_matches('.').to_ascii_lowercase();
        let lang = self.extension_map.get(&cleaned)?;
        self.get_by_language(lang)
    }

    /// Retrieves a parser suitable for a given file path based on its extension.
    pub fn get_by_path(&self, path: &Path) -> Option<&dyn SourceParser> {
        let ext = path.extension().and_then(|e| e.to_str())?;
        self.get_by_extension(ext)
    }

    /// Checks if a file extension is supported.
    pub fn is_extension_supported(&self, extension: &str) -> bool {
        let cleaned = extension.trim_start_matches('.').to_ascii_lowercase();
        self.extension_map.contains_key(&cleaned)
    }

    /// Returns a sorted list of all supported file extensions.
    pub fn supported_extensions(&self) -> Vec<String> {
        let mut exts: Vec<String> = self.extension_map.keys().cloned().collect();
        exts.sort();
        exts
    }

    /// Parses a source code file from disk using the matching registered parser.
    pub fn parse_file(&self, path: &Path, options: &ParseOptions) -> Result<SourceFile> {
        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or_default();
        let parser =
            self.get_by_extension(ext)
                .ok_or_else(|| ParserError::UnsupportedExtension {
                    extension: ext.to_string(),
                })?;

        parser.parse_file(path, options)
    }

    /// Parses source code text using the matching registered parser based on file path.
    pub fn parse_source(
        &self,
        path: &Path,
        source: &str,
        options: &ParseOptions,
    ) -> Result<SourceFile> {
        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or_default();
        let parser =
            self.get_by_extension(ext)
                .ok_or_else(|| ParserError::UnsupportedExtension {
                    extension: ext.to_string(),
                })?;

        parser.parse_source(path, source, options)
    }
}
