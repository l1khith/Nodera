use std::path::Path;

use nodera_parser_core::{
    DiagnosticSeverity, LanguageId, ParseOptions, ParserError, ParserRegistry, ReferenceKind,
    RustParser, SourceParser, SymbolKind, Visibility,
};

#[test]
fn test_rust_functions_and_signatures() {
    let source = r#"
/// Computes the sum of two integers.
pub fn add(a: i32, b: i32) -> i32 {
    a + b
}

pub(crate) async fn fetch_data(url: String) -> Result<String, ()> {
    Ok(url)
}

const unsafe fn low_level_op() {
}
"#;

    let parser = RustParser::new();
    let options = ParseOptions::default();
    let sf = parser
        .parse_source(Path::new("math.rs"), source, &options)
        .expect("parsing failed");

    assert!(!sf.has_errors());
    assert_eq!(sf.language_id, LanguageId::Rust);

    let symbols = sf.symbols_by_kind(SymbolKind::Function);
    assert_eq!(symbols.len(), 3);

    // 1. add function
    let add_sym = symbols.iter().find(|s| s.name == "add").unwrap();
    assert_eq!(add_sym.visibility, Visibility::Public);
    assert_eq!(
        add_sym.doc_comment.as_deref(),
        Some("Computes the sum of two integers.")
    );
    assert!(add_sym
        .signature
        .as_ref()
        .unwrap()
        .contains("pub fn add(a: i32, b: i32) -> i32"));
    assert_eq!(add_sym.span.start_line, 2);

    // 2. fetch_data function
    let fetch_sym = symbols.iter().find(|s| s.name == "fetch_data").unwrap();
    assert_eq!(fetch_sym.visibility, Visibility::Crate);
    assert!(fetch_sym
        .signature
        .as_ref()
        .unwrap()
        .contains("pub(crate) async fn fetch_data"));

    // 3. low_level_op function
    let low_sym = symbols.iter().find(|s| s.name == "low_level_op").unwrap();
    assert_eq!(low_sym.visibility, Visibility::Private);
    assert!(low_sym
        .signature
        .as_ref()
        .unwrap()
        .contains("const unsafe fn low_level_op"));
}

#[test]
fn test_rust_structs_enums_and_variants() {
    let source = r#"
/// Represents a user profile.
pub struct User {
    pub id: u64,
    name: String,
}

pub enum Status {
    Active,
    Pending(u32),
    Failed { reason: String },
}
"#;

    let parser = RustParser::new();
    let options = ParseOptions::default();
    let sf = parser
        .parse_source(Path::new("models.rs"), source, &options)
        .expect("parsing failed");

    assert!(!sf.has_errors());

    // Struct inspection
    let structs = sf.symbols_by_kind(SymbolKind::Struct);
    assert_eq!(structs.len(), 1);
    let user = structs[0];
    assert_eq!(user.name, "User");
    assert_eq!(user.visibility, Visibility::Public);
    assert_eq!(
        user.doc_comment.as_deref(),
        Some("Represents a user profile.")
    );

    // Field inspection
    assert_eq!(user.children.len(), 2);
    let id_field = user.children.iter().find(|c| c.name == "id").unwrap();
    assert_eq!(id_field.kind, SymbolKind::Field);
    assert_eq!(id_field.visibility, Visibility::Public);

    let name_field = user.children.iter().find(|c| c.name == "name").unwrap();
    assert_eq!(name_field.visibility, Visibility::Private);

    // Enum inspection
    let enums = sf.symbols_by_kind(SymbolKind::Enum);
    assert_eq!(enums.len(), 1);
    let status = enums[0];
    assert_eq!(status.name, "Status");
    assert_eq!(status.children.len(), 3);

    let variant_names: Vec<&str> = status.children.iter().map(|c| c.name.as_str()).collect();
    assert_eq!(variant_names, vec!["Active", "Pending", "Failed"]);
}

#[test]
fn test_rust_traits_and_implementations() {
    let source = r#"
pub trait Greet {
    fn say_hello(&self) -> String;
}

pub struct Greeter;

impl Greet for Greeter {
    fn say_hello(&self) -> String {
        "Hello!".to_string()
    }
}
"#;

    let parser = RustParser::new();
    let options = ParseOptions::default();
    let sf = parser
        .parse_source(Path::new("greet.rs"), source, &options)
        .expect("parsing failed");

    assert!(!sf.has_errors());

    let traits = sf.symbols_by_kind(SymbolKind::Trait);
    assert_eq!(traits.len(), 1);
    assert_eq!(traits[0].name, "Greet");
    assert_eq!(traits[0].children.len(), 1);
    assert_eq!(traits[0].children[0].name, "say_hello");

    let impls = sf.symbols_by_kind(SymbolKind::Implementation);
    assert_eq!(impls.len(), 1);
    assert_eq!(impls[0].name, "Greet for Greeter");
    assert_eq!(impls[0].children.len(), 1);
    assert_eq!(impls[0].children[0].name, "say_hello");
}

#[test]
fn test_rust_imports_and_use_trees() {
    let source = r#"
use std::path::PathBuf;
use std::collections::{HashMap, HashSet};
use std::io::Result as IoResult;
use nodera_core::*;
"#;

    let parser = RustParser::new();
    let options = ParseOptions::default();
    let sf = parser
        .parse_source(Path::new("imports.rs"), source, &options)
        .expect("parsing failed");

    assert_eq!(sf.imports.len(), 5);

    // PathBuf
    let p = sf
        .imports
        .iter()
        .find(|i| i.path == "std::path::PathBuf")
        .unwrap();
    assert_eq!(p.imported_items, vec!["PathBuf"]);
    assert!(!p.is_glob);

    // HashMap & HashSet
    assert!(sf
        .imports
        .iter()
        .any(|i| i.path == "std::collections::HashMap"));
    assert!(sf
        .imports
        .iter()
        .any(|i| i.path == "std::collections::HashSet"));

    // Alias
    let a = sf
        .imports
        .iter()
        .find(|i| i.alias.as_deref() == Some("IoResult"))
        .unwrap();
    assert_eq!(a.path, "std::io::Result");

    // Glob
    let g = sf.imports.iter().find(|i| i.is_glob).unwrap();
    assert_eq!(g.path, "nodera_core");
    assert_eq!(g.imported_items, vec!["*"]);
}

#[test]
fn test_rust_references_and_calls() {
    let source = r#"
fn run() {
    let x = std::path::PathBuf::new();
    println!("Hello");
    x.clone();
}
"#;

    let parser = RustParser::new();
    let options = ParseOptions::default();
    let sf = parser
        .parse_source(Path::new("caller.rs"), source, &options)
        .expect("parsing failed");

    // Check calls and macro invocations
    let calls: Vec<&str> = sf
        .references
        .iter()
        .filter(|r| r.kind == ReferenceKind::Call)
        .map(|r| r.target.as_str())
        .collect();
    assert!(calls.contains(&"std::path::PathBuf::new"));
    assert!(calls.contains(&"clone"));

    let macros: Vec<&str> = sf
        .references
        .iter()
        .filter(|r| r.kind == ReferenceKind::MacroInvocation)
        .map(|r| r.target.as_str())
        .collect();
    assert!(macros.contains(&"println"));
}

#[test]
fn test_rust_metrics_calculation() {
    let source =
        "// File header comment\n\nfn foo() {\n    /* block comment */\n    let a = 1;\n}\n";

    let parser = RustParser::new();
    let options = ParseOptions::default();
    let sf = parser
        .parse_source(Path::new("metrics.rs"), source, &options)
        .expect("parsing failed");

    assert_eq!(sf.metrics.total_lines, 6);
    assert_eq!(sf.metrics.blank_lines, 1);
    assert_eq!(sf.metrics.comment_lines, 2);
    assert_eq!(sf.metrics.code_lines, 3);
    assert_eq!(sf.metrics.symbol_count, 1);
}

#[test]
fn test_malformed_syntax_resilience() {
    let invalid_source = r#"
pub fn broken( {
    let x = ;
"#;

    let parser = RustParser::new();

    // 1. Resilient mode (default)
    let resilient_options = ParseOptions {
        resilient: true,
        ..Default::default()
    };
    let sf = parser
        .parse_source(Path::new("broken.rs"), invalid_source, &resilient_options)
        .expect("resilient parsing should succeed with diagnostics");

    assert!(sf.has_errors());
    assert_eq!(sf.diagnostics.len(), 1);
    assert_eq!(sf.diagnostics[0].severity, DiagnosticSeverity::Error);
    assert_eq!(sf.diagnostics[0].code.as_deref(), Some("SYNTAX_ERROR"));
    assert!(sf.diagnostics[0].span.is_some());

    // 2. Non-resilient mode
    let strict_options = ParseOptions {
        resilient: false,
        ..Default::default()
    };
    let err = parser
        .parse_source(Path::new("broken.rs"), invalid_source, &strict_options)
        .expect_err("strict parsing must return syntax error");

    assert!(matches!(err, ParserError::SyntaxFatal { .. }));
}

#[test]
fn test_parser_registry_dispatch() {
    let registry = ParserRegistry::with_defaults();

    assert!(registry.is_extension_supported("rs"));
    assert!(registry.is_extension_supported(".RS"));
    assert!(!registry.is_extension_supported("unknown_ext"));

    let source = "pub fn registered_fn() {}";
    let sf = registry
        .parse_source(Path::new("lib.rs"), source, &ParseOptions::default())
        .expect("dispatch parsing failed");

    assert_eq!(sf.language_id, LanguageId::Rust);
    assert_eq!(sf.symbols.len(), 1);
    assert_eq!(sf.symbols[0].name, "registered_fn");

    let err = registry.parse_source(Path::new("test.xyz"), source, &ParseOptions::default());
    assert!(matches!(err, Err(ParserError::UnsupportedExtension { .. })));
}

#[test]
fn test_real_repo_file_parsing() {
    let registry = ParserRegistry::with_defaults();
    let uri_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join("nodera-core")
        .join("src")
        .join("uri.rs");

    if uri_path.exists() {
        let options = ParseOptions::default();
        let sf = registry
            .parse_file(&uri_path, &options)
            .expect("parsing real file failed");

        assert!(!sf.has_errors());
        assert_eq!(sf.language_id, LanguageId::Rust);
        assert!(!sf.symbols.is_empty());
        assert!(!sf.imports.is_empty());
        assert!(!sf.references.is_empty());
        assert!(sf.metrics.code_lines > 10);
    }
}
