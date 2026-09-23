use std::path::Path;
use syn::spanned::Spanned;
use syn::visit::Visit;
use syn::{
    Attribute, ExprCall, ExprMethodCall, Field, Fields, ImplItem, Item, ItemConst, ItemEnum,
    ItemFn, ItemImpl, ItemMod, ItemStatic, ItemStruct, ItemTrait, ItemType, ItemUse, Macro, Meta,
    TraitItem, TypePath, UseGroup, UseName, UsePath, UseRename, UseTree,
};

use crate::diagnostics::Diagnostic;
use crate::error::{ParserError, Result};
use crate::ir::{
    ImportDeclaration, ReferenceKind, SourceFile, SourceMetrics, Symbol, SymbolKind,
    SymbolReference, Visibility,
};
use crate::span::{LineIndex, SourceSpan};
use crate::traits::{LanguageId, ParseOptions, SourceParser};

/// Source code parser for the Rust language using the `syn` AST.
#[derive(Debug, Default, Clone)]
pub struct RustParser;

impl RustParser {
    pub fn new() -> Self {
        Self
    }
}

impl SourceParser for RustParser {
    fn language_id(&self) -> LanguageId {
        LanguageId::Rust
    }

    fn supported_extensions(&self) -> &[&str] {
        &["rs"]
    }

    fn parse_source(
        &self,
        path: &Path,
        source: &str,
        options: &ParseOptions,
    ) -> Result<SourceFile> {
        let line_index = LineIndex::new(source);
        let mut source_file = SourceFile::new(path, LanguageId::Rust);

        let syn_file = match syn::parse_file(source) {
            Ok(file) => file,
            Err(err) => {
                let span = proc_macro_span_to_source_span(err.span(), &line_index);
                let diag = Diagnostic::error(err.to_string(), Some(span)).with_code("SYNTAX_ERROR");
                source_file.diagnostics.push(diag);

                if !options.resilient {
                    return Err(ParserError::SyntaxFatal {
                        path: path.to_path_buf(),
                        message: err.to_string(),
                        line: Some(span.start_line),
                        column: Some(span.start_col),
                    });
                }

                if options.extract_metrics {
                    source_file.metrics = calculate_source_metrics(source, 0);
                }
                return Ok(source_file);
            }
        };

        let file_stem = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("module");

        // 1. Extract symbols and imports from items
        for item in &syn_file.items {
            extract_item(
                item,
                file_stem,
                None,
                &line_index,
                options,
                &mut source_file.symbols,
                &mut source_file.imports,
            );
        }

        // 2. Extract symbol references via AST visitor
        if options.extract_references {
            let mut visitor = ReferenceVisitor {
                line_index: &line_index,
                references: Vec::new(),
                current_caller: None,
            };
            visitor.visit_file(&syn_file);
            source_file.references = visitor.references;
        }

        // 3. Compute code metrics
        if options.extract_metrics {
            let total_syms = source_file.all_symbols_flat().len();
            source_file.metrics = calculate_source_metrics(source, total_syms);
        }

        Ok(source_file)
    }
}

// ---------------------------------------------------------------------------
// Item & Symbol Extraction
// ---------------------------------------------------------------------------

fn extract_item(
    item: &Item,
    module_prefix: &str,
    parent_id: Option<&str>,
    line_index: &LineIndex,
    options: &ParseOptions,
    symbols: &mut Vec<Symbol>,
    imports: &mut Vec<ImportDeclaration>,
) {
    match item {
        Item::Fn(item_fn) => {
            if let Some(sym) = extract_fn(item_fn, module_prefix, parent_id, line_index, options) {
                symbols.push(sym);
            }
        }
        Item::Struct(item_struct) => {
            if let Some(sym) =
                extract_struct(item_struct, module_prefix, parent_id, line_index, options)
            {
                symbols.push(sym);
            }
        }
        Item::Enum(item_enum) => {
            if let Some(sym) =
                extract_enum(item_enum, module_prefix, parent_id, line_index, options)
            {
                symbols.push(sym);
            }
        }
        Item::Trait(item_trait) => {
            if let Some(sym) =
                extract_trait(item_trait, module_prefix, parent_id, line_index, options)
            {
                symbols.push(sym);
            }
        }
        Item::Impl(item_impl) => {
            if let Some(sym) =
                extract_impl(item_impl, module_prefix, parent_id, line_index, options)
            {
                symbols.push(sym);
            }
        }
        Item::Mod(item_mod) => {
            extract_mod(
                item_mod,
                module_prefix,
                parent_id,
                line_index,
                options,
                symbols,
                imports,
            );
        }
        Item::Type(item_type) => {
            if let Some(sym) =
                extract_type(item_type, module_prefix, parent_id, line_index, options)
            {
                symbols.push(sym);
            }
        }
        Item::Const(item_const) => {
            if let Some(sym) =
                extract_const(item_const, module_prefix, parent_id, line_index, options)
            {
                symbols.push(sym);
            }
        }
        Item::Static(item_static) => {
            if let Some(sym) =
                extract_static(item_static, module_prefix, parent_id, line_index, options)
            {
                symbols.push(sym);
            }
        }
        Item::Macro(item_macro) => {
            if let Some(ident) = &item_macro.ident {
                let name = ident.to_string();
                let id = format!("{module_prefix}::{name}");
                let span = proc_macro_span_to_source_span(item_macro.span(), line_index);
                symbols.push(Symbol {
                    id,
                    name,
                    kind: SymbolKind::Macro,
                    visibility: Visibility::Public,
                    span,
                    signature: None,
                    doc_comment: None,
                    parent_id: parent_id.map(|s| s.to_string()),
                    children: Vec::new(),
                });
            }
        }
        Item::Use(item_use) => {
            extract_use_declarations(item_use, line_index, imports);
        }
        _ => {}
    }
}

fn extract_fn(
    item_fn: &ItemFn,
    module_prefix: &str,
    parent_id: Option<&str>,
    line_index: &LineIndex,
    options: &ParseOptions,
) -> Option<Symbol> {
    let vis = convert_visibility(&item_fn.vis);
    if !options.include_private_symbols && !vis.is_public() {
        return None;
    }

    let name = item_fn.sig.ident.to_string();
    let id = format!("{module_prefix}::{name}");
    let span = proc_macro_span_to_source_span(item_fn.span(), line_index);
    let signature = format_fn_signature(&item_fn.sig, &item_fn.vis);
    let doc_comment = if options.extract_doc_comments {
        extract_doc_comments(&item_fn.attrs)
    } else {
        None
    };

    Some(Symbol {
        id,
        name,
        kind: SymbolKind::Function,
        visibility: vis,
        span,
        signature: Some(signature),
        doc_comment,
        parent_id: parent_id.map(|s| s.to_string()),
        children: Vec::new(),
    })
}

fn extract_struct(
    item_struct: &ItemStruct,
    module_prefix: &str,
    parent_id: Option<&str>,
    line_index: &LineIndex,
    options: &ParseOptions,
) -> Option<Symbol> {
    let vis = convert_visibility(&item_struct.vis);
    if !options.include_private_symbols && !vis.is_public() {
        return None;
    }

    let name = item_struct.ident.to_string();
    let id = format!("{module_prefix}::{name}");
    let span = proc_macro_span_to_source_span(item_struct.span(), line_index);
    let doc_comment = if options.extract_doc_comments {
        extract_doc_comments(&item_struct.attrs)
    } else {
        None
    };

    let mut children = Vec::new();
    if let Fields::Named(named_fields) = &item_struct.fields {
        for field in &named_fields.named {
            if let Some(sym) = extract_field(field, &id, line_index, options) {
                children.push(sym);
            }
        }
    }

    let signature = format!("{}struct {}", format_vis(&item_struct.vis), name);

    Some(Symbol {
        id,
        name,
        kind: SymbolKind::Struct,
        visibility: vis,
        span,
        signature: Some(signature),
        doc_comment,
        parent_id: parent_id.map(|s| s.to_string()),
        children,
    })
}

fn extract_field(
    field: &Field,
    parent_id: &str,
    line_index: &LineIndex,
    options: &ParseOptions,
) -> Option<Symbol> {
    let vis = convert_visibility(&field.vis);
    if !options.include_private_symbols && !vis.is_public() {
        return None;
    }

    let name = field.ident.as_ref()?.to_string();
    let id = format!("{parent_id}::{name}");
    let span = proc_macro_span_to_source_span(field.span(), line_index);
    let doc_comment = if options.extract_doc_comments {
        extract_doc_comments(&field.attrs)
    } else {
        None
    };

    Some(Symbol {
        id,
        name,
        kind: SymbolKind::Field,
        visibility: vis,
        span,
        signature: None,
        doc_comment,
        parent_id: Some(parent_id.to_string()),
        children: Vec::new(),
    })
}

fn extract_enum(
    item_enum: &ItemEnum,
    module_prefix: &str,
    parent_id: Option<&str>,
    line_index: &LineIndex,
    options: &ParseOptions,
) -> Option<Symbol> {
    let vis = convert_visibility(&item_enum.vis);
    if !options.include_private_symbols && !vis.is_public() {
        return None;
    }

    let name = item_enum.ident.to_string();
    let id = format!("{module_prefix}::{name}");
    let span = proc_macro_span_to_source_span(item_enum.span(), line_index);
    let doc_comment = if options.extract_doc_comments {
        extract_doc_comments(&item_enum.attrs)
    } else {
        None
    };

    let mut children = Vec::new();
    for variant in &item_enum.variants {
        let vname = variant.ident.to_string();
        let vid = format!("{id}::{vname}");
        let vspan = proc_macro_span_to_source_span(variant.span(), line_index);
        let vdoc = if options.extract_doc_comments {
            extract_doc_comments(&variant.attrs)
        } else {
            None
        };
        children.push(Symbol {
            id: vid,
            name: vname,
            kind: SymbolKind::EnumVariant,
            visibility: Visibility::Public,
            span: vspan,
            signature: None,
            doc_comment: vdoc,
            parent_id: Some(id.clone()),
            children: Vec::new(),
        });
    }

    let signature = format!("{}enum {}", format_vis(&item_enum.vis), name);

    Some(Symbol {
        id,
        name,
        kind: SymbolKind::Enum,
        visibility: vis,
        span,
        signature: Some(signature),
        doc_comment,
        parent_id: parent_id.map(|s| s.to_string()),
        children,
    })
}

fn extract_trait(
    item_trait: &ItemTrait,
    module_prefix: &str,
    parent_id: Option<&str>,
    line_index: &LineIndex,
    options: &ParseOptions,
) -> Option<Symbol> {
    let vis = convert_visibility(&item_trait.vis);
    if !options.include_private_symbols && !vis.is_public() {
        return None;
    }

    let name = item_trait.ident.to_string();
    let id = format!("{module_prefix}::{name}");
    let span = proc_macro_span_to_source_span(item_trait.span(), line_index);
    let doc_comment = if options.extract_doc_comments {
        extract_doc_comments(&item_trait.attrs)
    } else {
        None
    };

    let mut children = Vec::new();
    for trait_item in &item_trait.items {
        if let TraitItem::Fn(m) = trait_item {
            let mname = m.sig.ident.to_string();
            let mid = format!("{id}::{mname}");
            let mspan = proc_macro_span_to_source_span(m.span(), line_index);
            let msig = format_fn_signature(&m.sig, &syn::Visibility::Inherited);
            let mdoc = if options.extract_doc_comments {
                extract_doc_comments(&m.attrs)
            } else {
                None
            };
            children.push(Symbol {
                id: mid,
                name: mname,
                kind: SymbolKind::Method,
                visibility: Visibility::Public,
                span: mspan,
                signature: Some(msig),
                doc_comment: mdoc,
                parent_id: Some(id.clone()),
                children: Vec::new(),
            });
        }
    }

    let signature = format!("{}trait {}", format_vis(&item_trait.vis), name);

    Some(Symbol {
        id,
        name,
        kind: SymbolKind::Trait,
        visibility: vis,
        span,
        signature: Some(signature),
        doc_comment,
        parent_id: parent_id.map(|s| s.to_string()),
        children,
    })
}

fn extract_impl(
    item_impl: &ItemImpl,
    module_prefix: &str,
    parent_id: Option<&str>,
    line_index: &LineIndex,
    options: &ParseOptions,
) -> Option<Symbol> {
    let self_ty_str = quote_type_to_string(&item_impl.self_ty);
    let trait_str = item_impl
        .trait_
        .as_ref()
        .map(|(_, path, _)| quote_path_to_string(path));

    let name = match &trait_str {
        Some(tr) => format!("{tr} for {self_ty_str}"),
        None => self_ty_str.clone(),
    };

    let id = format!("{module_prefix}::impl::{name}");
    let span = proc_macro_span_to_source_span(item_impl.span(), line_index);

    let mut children = Vec::new();
    for impl_item in &item_impl.items {
        if let ImplItem::Fn(m) = impl_item {
            let vis = convert_visibility(&m.vis);
            if !options.include_private_symbols && !vis.is_public() {
                continue;
            }
            let mname = m.sig.ident.to_string();
            let mid = format!("{id}::{mname}");
            let mspan = proc_macro_span_to_source_span(m.span(), line_index);
            let msig = format_fn_signature(&m.sig, &m.vis);
            let mdoc = if options.extract_doc_comments {
                extract_doc_comments(&m.attrs)
            } else {
                None
            };
            children.push(Symbol {
                id: mid,
                name: mname,
                kind: SymbolKind::Method,
                visibility: vis,
                span: mspan,
                signature: Some(msig),
                doc_comment: mdoc,
                parent_id: Some(id.clone()),
                children: Vec::new(),
            });
        }
    }

    Some(Symbol {
        id,
        name,
        kind: SymbolKind::Implementation,
        visibility: Visibility::Public,
        span,
        signature: Some(format!("impl {}", trait_str.unwrap_or(self_ty_str))),
        doc_comment: None,
        parent_id: parent_id.map(|s| s.to_string()),
        children,
    })
}

fn extract_mod(
    item_mod: &ItemMod,
    module_prefix: &str,
    parent_id: Option<&str>,
    line_index: &LineIndex,
    options: &ParseOptions,
    symbols: &mut Vec<Symbol>,
    imports: &mut Vec<ImportDeclaration>,
) {
    let vis = convert_visibility(&item_mod.vis);
    if !options.include_private_symbols && !vis.is_public() {
        return;
    }

    let name = item_mod.ident.to_string();
    let id = format!("{module_prefix}::{name}");
    let span = proc_macro_span_to_source_span(item_mod.span(), line_index);
    let doc_comment = if options.extract_doc_comments {
        extract_doc_comments(&item_mod.attrs)
    } else {
        None
    };

    let mut children = Vec::new();
    if let Some((_, items)) = &item_mod.content {
        for inner_item in items {
            extract_item(
                inner_item,
                &id,
                Some(&id),
                line_index,
                options,
                &mut children,
                imports,
            );
        }
    }

    symbols.push(Symbol {
        id,
        name,
        kind: SymbolKind::Module,
        visibility: vis,
        span,
        signature: Some(format!(
            "{}mod {}",
            format_vis(&item_mod.vis),
            item_mod.ident
        )),
        doc_comment,
        parent_id: parent_id.map(|s| s.to_string()),
        children,
    });
}

fn extract_type(
    item_type: &ItemType,
    module_prefix: &str,
    parent_id: Option<&str>,
    line_index: &LineIndex,
    options: &ParseOptions,
) -> Option<Symbol> {
    let vis = convert_visibility(&item_type.vis);
    if !options.include_private_symbols && !vis.is_public() {
        return None;
    }

    let name = item_type.ident.to_string();
    let id = format!("{module_prefix}::{name}");
    let span = proc_macro_span_to_source_span(item_type.span(), line_index);
    let doc_comment = if options.extract_doc_comments {
        extract_doc_comments(&item_type.attrs)
    } else {
        None
    };

    Some(Symbol {
        id,
        name: name.clone(),
        kind: SymbolKind::TypeAlias,
        visibility: vis,
        span,
        signature: Some(format!("{}type {}", format_vis(&item_type.vis), name)),
        doc_comment,
        parent_id: parent_id.map(|s| s.to_string()),
        children: Vec::new(),
    })
}

fn extract_const(
    item_const: &ItemConst,
    module_prefix: &str,
    parent_id: Option<&str>,
    line_index: &LineIndex,
    options: &ParseOptions,
) -> Option<Symbol> {
    let vis = convert_visibility(&item_const.vis);
    if !options.include_private_symbols && !vis.is_public() {
        return None;
    }

    let name = item_const.ident.to_string();
    let id = format!("{module_prefix}::{name}");
    let span = proc_macro_span_to_source_span(item_const.span(), line_index);
    let doc_comment = if options.extract_doc_comments {
        extract_doc_comments(&item_const.attrs)
    } else {
        None
    };

    Some(Symbol {
        id,
        name: name.clone(),
        kind: SymbolKind::Constant,
        visibility: vis,
        span,
        signature: Some(format!("{}const {}", format_vis(&item_const.vis), name)),
        doc_comment,
        parent_id: parent_id.map(|s| s.to_string()),
        children: Vec::new(),
    })
}

fn extract_static(
    item_static: &ItemStatic,
    module_prefix: &str,
    parent_id: Option<&str>,
    line_index: &LineIndex,
    options: &ParseOptions,
) -> Option<Symbol> {
    let vis = convert_visibility(&item_static.vis);
    if !options.include_private_symbols && !vis.is_public() {
        return None;
    }

    let name = item_static.ident.to_string();
    let id = format!("{module_prefix}::{name}");
    let span = proc_macro_span_to_source_span(item_static.span(), line_index);
    let doc_comment = if options.extract_doc_comments {
        extract_doc_comments(&item_static.attrs)
    } else {
        None
    };

    Some(Symbol {
        id,
        name: name.clone(),
        kind: SymbolKind::Static,
        visibility: vis,
        span,
        signature: Some(format!("{}static {}", format_vis(&item_static.vis), name)),
        doc_comment,
        parent_id: parent_id.map(|s| s.to_string()),
        children: Vec::new(),
    })
}

// ---------------------------------------------------------------------------
// Imports & Use Tree Traversal
// ---------------------------------------------------------------------------

fn extract_use_declarations(
    item_use: &ItemUse,
    line_index: &LineIndex,
    imports: &mut Vec<ImportDeclaration>,
) {
    let span = proc_macro_span_to_source_span(item_use.span(), line_index);
    collect_use_tree(&item_use.tree, "", span, imports);
}

fn collect_use_tree(
    tree: &UseTree,
    prefix: &str,
    span: SourceSpan,
    imports: &mut Vec<ImportDeclaration>,
) {
    match tree {
        UseTree::Path(UsePath { ident, tree, .. }) => {
            let next_prefix = if prefix.is_empty() {
                ident.to_string()
            } else {
                format!("{prefix}::{ident}")
            };
            collect_use_tree(tree, &next_prefix, span, imports);
        }
        UseTree::Name(UseName { ident }) => {
            let full_path = if prefix.is_empty() {
                ident.to_string()
            } else {
                format!("{prefix}::{ident}")
            };
            imports.push(ImportDeclaration {
                path: full_path,
                imported_items: vec![ident.to_string()],
                alias: None,
                span,
                is_glob: false,
            });
        }
        UseTree::Rename(UseRename { ident, rename, .. }) => {
            let full_path = if prefix.is_empty() {
                ident.to_string()
            } else {
                format!("{prefix}::{ident}")
            };
            imports.push(ImportDeclaration {
                path: full_path,
                imported_items: vec![ident.to_string()],
                alias: Some(rename.to_string()),
                span,
                is_glob: false,
            });
        }
        UseTree::Glob(_) => {
            imports.push(ImportDeclaration {
                path: prefix.to_string(),
                imported_items: vec!["*".to_string()],
                alias: None,
                span,
                is_glob: true,
            });
        }
        UseTree::Group(UseGroup { items, .. }) => {
            for sub_tree in items {
                collect_use_tree(sub_tree, prefix, span, imports);
            }
        }
    }
}

// ---------------------------------------------------------------------------
// AST Visitor for Symbol References
// ---------------------------------------------------------------------------

struct ReferenceVisitor<'a> {
    line_index: &'a LineIndex,
    references: Vec<SymbolReference>,
    current_caller: Option<String>,
}

impl<'ast> Visit<'ast> for ReferenceVisitor<'_> {
    fn visit_item_fn(&mut self, node: &'ast ItemFn) {
        let prev = self.current_caller.take();
        self.current_caller = Some(node.sig.ident.to_string());
        syn::visit::visit_item_fn(self, node);
        self.current_caller = prev;
    }

    fn visit_impl_item_fn(&mut self, node: &'ast syn::ImplItemFn) {
        let prev = self.current_caller.take();
        self.current_caller = Some(node.sig.ident.to_string());
        syn::visit::visit_impl_item_fn(self, node);
        self.current_caller = prev;
    }

    fn visit_expr_call(&mut self, node: &'ast ExprCall) {
        let target = quote_expr_to_string(&node.func);
        if !target.is_empty() {
            let span = proc_macro_span_to_source_span(node.span(), self.line_index);
            self.references.push(SymbolReference {
                target,
                kind: ReferenceKind::Call,
                span,
                caller_symbol_id: self.current_caller.clone(),
            });
        }
        syn::visit::visit_expr_call(self, node);
    }

    fn visit_expr_method_call(&mut self, node: &'ast ExprMethodCall) {
        let target = node.method.to_string();
        let span = proc_macro_span_to_source_span(node.span(), self.line_index);
        self.references.push(SymbolReference {
            target,
            kind: ReferenceKind::Call,
            span,
            caller_symbol_id: self.current_caller.clone(),
        });
        syn::visit::visit_expr_method_call(self, node);
    }

    fn visit_macro(&mut self, node: &'ast Macro) {
        let target = quote_path_to_string(&node.path);
        let span = proc_macro_span_to_source_span(node.span(), self.line_index);
        self.references.push(SymbolReference {
            target,
            kind: ReferenceKind::MacroInvocation,
            span,
            caller_symbol_id: self.current_caller.clone(),
        });
        syn::visit::visit_macro(self, node);
    }

    fn visit_type_path(&mut self, node: &'ast TypePath) {
        let target = quote_path_to_string(&node.path);
        if !target.is_empty() {
            let span = proc_macro_span_to_source_span(node.span(), self.line_index);
            self.references.push(SymbolReference {
                target,
                kind: ReferenceKind::TypeUsage,
                span,
                caller_symbol_id: self.current_caller.clone(),
            });
        }
        syn::visit::visit_type_path(self, node);
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn convert_visibility(vis: &syn::Visibility) -> Visibility {
    match vis {
        syn::Visibility::Public(_) => Visibility::Public,
        syn::Visibility::Restricted(r) => {
            let path_str = quote_path_to_string(&r.path);
            if path_str == "crate" {
                Visibility::Crate
            } else {
                Visibility::Restricted(path_str)
            }
        }
        syn::Visibility::Inherited => Visibility::Private,
    }
}

fn format_vis(vis: &syn::Visibility) -> String {
    match vis {
        syn::Visibility::Public(_) => "pub ".to_string(),
        syn::Visibility::Restricted(r) => format!("pub({}) ", quote_path_to_string(&r.path)),
        syn::Visibility::Inherited => String::new(),
    }
}

fn format_fn_signature(sig: &syn::Signature, vis: &syn::Visibility) -> String {
    let mut parts = Vec::new();
    let vis_str = format_vis(vis);
    if !vis_str.is_empty() {
        parts.push(vis_str.trim().to_string());
    }
    if sig.constness.is_some() {
        parts.push("const".to_string());
    }
    if sig.asyncness.is_some() {
        parts.push("async".to_string());
    }
    if sig.unsafety.is_some() {
        parts.push("unsafe".to_string());
    }
    parts.push(format!("fn {}", sig.ident));

    let inputs: Vec<String> = sig
        .inputs
        .iter()
        .map(|fn_arg| match fn_arg {
            syn::FnArg::Receiver(r) => {
                if r.reference.is_some() {
                    if r.mutability.is_some() {
                        "&mut self".to_string()
                    } else {
                        "&self".to_string()
                    }
                } else if r.mutability.is_some() {
                    "mut self".to_string()
                } else {
                    "self".to_string()
                }
            }
            syn::FnArg::Typed(pat_type) => {
                let pat_str = quote_pat_to_string(&pat_type.pat);
                let ty_str = quote_type_to_string(&pat_type.ty);
                format!("{pat_str}: {ty_str}")
            }
        })
        .collect();

    let output = match &sig.output {
        syn::ReturnType::Default => String::new(),
        syn::ReturnType::Type(_, ty) => format!(" -> {}", quote_type_to_string(ty)),
    };

    format!("{}({}){}", parts.join(" "), inputs.join(", "), output)
}

fn extract_doc_comments(attrs: &[Attribute]) -> Option<String> {
    let mut doc_lines = Vec::new();
    for attr in attrs {
        if attr.path().is_ident("doc") {
            if let Meta::NameValue(nv) = &attr.meta {
                if let syn::Expr::Lit(syn::ExprLit {
                    lit: syn::Lit::Str(lit_str),
                    ..
                }) = &nv.value
                {
                    let val = lit_str.value();
                    let trimmed = val.strip_prefix(' ').unwrap_or(&val);
                    doc_lines.push(trimmed.to_string());
                }
            }
        }
    }

    if doc_lines.is_empty() {
        None
    } else {
        Some(doc_lines.join("\n"))
    }
}

fn proc_macro_span_to_source_span(span: proc_macro2::Span, line_index: &LineIndex) -> SourceSpan {
    let start_line = span.start().line;
    let start_col = span.start().column + 1;
    let end_line = span.end().line;
    let end_col = span.end().column + 1;

    let start_offset = line_index.line_start_offset(start_line).unwrap_or(0) + span.start().column;
    let end_offset = line_index.line_start_offset(end_line).unwrap_or(0) + span.end().column;
    let byte_len = end_offset.saturating_sub(start_offset);

    SourceSpan::new(
        start_line,
        start_col,
        end_line,
        end_col,
        start_offset,
        byte_len,
    )
}

fn calculate_source_metrics(source: &str, symbol_count: usize) -> SourceMetrics {
    let mut total_lines = 0;
    let mut blank_lines = 0;
    let mut comment_lines = 0;
    let mut code_lines = 0;

    let mut in_block_comment = false;

    for line in source.lines() {
        total_lines += 1;
        let trimmed = line.trim();

        if trimmed.is_empty() {
            blank_lines += 1;
            continue;
        }

        if in_block_comment {
            comment_lines += 1;
            if trimmed.contains("*/") {
                in_block_comment = false;
            }
            continue;
        }

        if trimmed.starts_with("//") {
            comment_lines += 1;
        } else if trimmed.starts_with("/*") {
            comment_lines += 1;
            if !trimmed.contains("*/") {
                in_block_comment = true;
            }
        } else {
            code_lines += 1;
        }
    }

    SourceMetrics {
        total_lines,
        code_lines,
        comment_lines,
        blank_lines,
        symbol_count,
    }
}

fn quote_path_to_string(path: &syn::Path) -> String {
    path.segments
        .iter()
        .map(|seg| seg.ident.to_string())
        .collect::<Vec<_>>()
        .join("::")
}

fn quote_type_to_string(ty: &syn::Type) -> String {
    match ty {
        syn::Type::Path(tp) => quote_path_to_string(&tp.path),
        syn::Type::Reference(tr) => {
            let mut s = String::from("&");
            if tr.mutability.is_some() {
                s.push_str("mut ");
            }
            s.push_str(&quote_type_to_string(&tr.elem));
            s
        }
        syn::Type::Slice(ts) => format!("[{}]", quote_type_to_string(&ts.elem)),
        syn::Type::Array(ta) => format!("[{}]", quote_type_to_string(&ta.elem)),
        syn::Type::Tuple(tt) => {
            let items: Vec<String> = tt.elems.iter().map(quote_type_to_string).collect();
            format!("({})", items.join(", "))
        }
        _ => "impl Type".to_string(),
    }
}

fn quote_pat_to_string(pat: &syn::Pat) -> String {
    match pat {
        syn::Pat::Ident(pi) => pi.ident.to_string(),
        syn::Pat::Wild(_) => "_".to_string(),
        _ => "arg".to_string(),
    }
}

fn quote_expr_to_string(expr: &syn::Expr) -> String {
    match expr {
        syn::Expr::Path(ep) => quote_path_to_string(&ep.path),
        _ => String::new(),
    }
}
