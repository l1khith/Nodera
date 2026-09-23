use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

use nodera_markdown::{GraphData, GraphEdge, GraphNode};
use nodera_parser_core::{ReferenceKind, SourceFile, SourceSpan, Symbol, SymbolKind, Visibility};
use serde::{Deserialize, Serialize};

use crate::error::{ProjectError, Result};
use crate::model::RustProject;

/// Top-level persistent Project Graph artifact stored at `.nodera/graph/project.json`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProjectGraph {
    pub schema_version: u32,
    pub project: ProjectHeader,
    pub files: Vec<ProjectFileNode>,
    pub nodes: Vec<ProjectNode>,
    pub edges: Vec<ProjectEdge>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectHeader {
    pub id: String,
    pub name: String,
    pub root: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectFileNode {
    pub path: String,
    pub package: String,
    pub language: String,
    pub metrics: FileMetrics,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct FileMetrics {
    pub total_lines: usize,
    pub code_lines: usize,
    pub comment_lines: usize,
    pub blank_lines: usize,
    pub symbol_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectNode {
    pub id: String,
    pub kind: ProjectNodeKind,
    pub name: String,
    pub label: String,
    pub file: String,
    pub visibility: String,
    pub span: Option<SourceSpan>,
    pub signature: Option<String>,
    pub doc: Option<String>,
    pub parent_id: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProjectNodeKind {
    File,
    Module,
    Struct,
    Enum,
    EnumVariant,
    Trait,
    Implementation,
    Function,
    Method,
    TypeAlias,
    Constant,
    Static,
    Macro,
    Field,
    Other,
}

impl std::fmt::Display for ProjectNodeKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::File => write!(f, "file"),
            Self::Module => write!(f, "module"),
            Self::Struct => write!(f, "struct"),
            Self::Enum => write!(f, "enum"),
            Self::EnumVariant => write!(f, "variant"),
            Self::Trait => write!(f, "trait"),
            Self::Implementation => write!(f, "impl"),
            Self::Function => write!(f, "function"),
            Self::Method => write!(f, "method"),
            Self::TypeAlias => write!(f, "type_alias"),
            Self::Constant => write!(f, "constant"),
            Self::Static => write!(f, "static"),
            Self::Macro => write!(f, "macro"),
            Self::Field => write!(f, "field"),
            Self::Other => write!(f, "other"),
        }
    }
}

impl From<&SymbolKind> for ProjectNodeKind {
    fn from(kind: &SymbolKind) -> Self {
        match kind {
            SymbolKind::Function => Self::Function,
            SymbolKind::Method => Self::Method,
            SymbolKind::Struct => Self::Struct,
            SymbolKind::Enum => Self::Enum,
            SymbolKind::EnumVariant => Self::EnumVariant,
            SymbolKind::Trait => Self::Trait,
            SymbolKind::Interface => Self::Trait,
            SymbolKind::Implementation => Self::Implementation,
            SymbolKind::Module => Self::Module,
            SymbolKind::TypeAlias => Self::TypeAlias,
            SymbolKind::Constant => Self::Constant,
            SymbolKind::Static => Self::Static,
            SymbolKind::Macro => Self::Macro,
            SymbolKind::Field => Self::Field,
            SymbolKind::Variable | SymbolKind::Other(_) => Self::Other,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ProjectEdge {
    pub source: String,
    pub target: String,
    pub kind: ProjectEdgeKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProjectEdgeKind {
    Contains,
    Calls,
    References,
    Imports,
    Implements,
}

impl ProjectGraph {
    /// Loads and parses project graph from `.nodera/graph/project.json`.
    pub fn load_from_file(path: &Path) -> Result<Self> {
        let content = fs::read_to_string(path).map_err(|e| ProjectError::Io {
            path: path.to_path_buf(),
            source: e,
        })?;

        serde_json::from_str(&content).map_err(|e| ProjectError::CorruptedState {
            path: path.to_path_buf(),
            reason: format!("Failed to parse project graph: {e}"),
        })
    }

    /// Saves project graph to disk formatted as human-inspectable pretty JSON.
    pub fn save_to_file(&self, path: &Path) -> Result<()> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|e| ProjectError::Io {
                path: parent.to_path_buf(),
                source: e,
            })?;
        }

        let json = serde_json::to_string_pretty(self).map_err(|e| ProjectError::Serialization {
            reason: format!("Failed to serialize project graph: {e}"),
        })?;

        fs::write(path, json).map_err(|e| ProjectError::Io {
            path: path.to_path_buf(),
            source: e,
        })
    }

    /// Builds a ProjectGraph from a discovered project and its parsed source files.
    pub fn build(
        project: &RustProject,
        project_id: &str,
        parsed_files: &[(PathBuf, SourceFile)],
    ) -> Self {
        let mut files = Vec::new();
        let mut nodes = Vec::new();
        let mut edges = Vec::new();

        // Index of symbol name -> node id for resolving cross-symbol references
        let mut symbol_name_to_ids: HashMap<String, Vec<String>> = HashMap::new();

        for (file_path, source_file) in parsed_files {
            let rel_path = file_path
                .strip_prefix(&project.root)
                .unwrap_or(file_path)
                .to_string_lossy()
                .replace('\\', "/");

            let package_name = project
                .packages
                .iter()
                .find(|p| file_path.starts_with(&p.package_root))
                .map(|p| p.name.clone())
                .unwrap_or_else(|| project.name.clone());

            // 1. File Node
            let file_node_id = format!("file:{}::{}", package_name, rel_path);
            files.push(ProjectFileNode {
                path: rel_path.clone(),
                package: package_name.clone(),
                language: source_file.language_id.to_string(),
                metrics: FileMetrics {
                    total_lines: source_file.metrics.total_lines,
                    code_lines: source_file.metrics.code_lines,
                    comment_lines: source_file.metrics.comment_lines,
                    blank_lines: source_file.metrics.blank_lines,
                    symbol_count: source_file.metrics.symbol_count,
                },
            });

            let file_project_node = ProjectNode {
                id: file_node_id.clone(),
                kind: ProjectNodeKind::File,
                name: Path::new(&rel_path)
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or(&rel_path)
                    .to_string(),
                label: format!("📄 {}", rel_path),
                file: rel_path.clone(),
                visibility: "pub".to_string(),
                span: None,
                signature: None,
                doc: None,
                parent_id: None,
            };
            nodes.push(file_project_node);

            // 2. Symbols
            for sym in &source_file.symbols {
                collect_symbol_nodes_and_edges(
                    sym,
                    &package_name,
                    &rel_path,
                    &file_node_id,
                    None,
                    &mut nodes,
                    &mut edges,
                    &mut symbol_name_to_ids,
                );
            }

            // 3. References & Calls
            for reference in &source_file.references {
                let caller_id = reference
                    .caller_symbol_id
                    .as_ref()
                    .map(|id| format!("sym:{}::{}::{}", package_name, rel_path, id))
                    .unwrap_or_else(|| file_node_id.clone());

                // Find matching target node by simple or qualified name
                let target_name = reference
                    .target
                    .split("::")
                    .last()
                    .unwrap_or(&reference.target);

                if let Some(target_ids) = symbol_name_to_ids.get(target_name) {
                    for target_id in target_ids {
                        if target_id != &caller_id {
                            let kind = match reference.kind {
                                ReferenceKind::Call => ProjectEdgeKind::Calls,
                                ReferenceKind::TraitImplementation => ProjectEdgeKind::Implements,
                                ReferenceKind::Import => ProjectEdgeKind::Imports,
                                _ => ProjectEdgeKind::References,
                            };
                            edges.push(ProjectEdge {
                                source: caller_id.clone(),
                                target: target_id.clone(),
                                kind,
                            });
                        }
                    }
                }
            }
        }

        // Deduplicate edges
        let mut unique_edges = Vec::with_capacity(edges.len());
        let mut seen = HashSet::new();
        for edge in edges {
            if seen.insert((edge.source.clone(), edge.target.clone(), edge.kind)) {
                unique_edges.push(edge);
            }
        }

        ProjectGraph {
            schema_version: 1,
            project: ProjectHeader {
                id: project_id.to_string(),
                name: project.name.clone(),
                root: project.root.clone(),
            },
            files,
            nodes,
            edges: unique_edges,
        }
    }

    /// Converts the ProjectGraph into the existing Desktop `GraphData` format.
    /// This allows reusing the interactive 2D graph renderer without duplicating visualization logic.
    pub fn to_graph_data(&self) -> GraphData {
        let mut in_degrees: HashMap<String, usize> = HashMap::new();
        let mut out_degrees: HashMap<String, usize> = HashMap::new();

        for edge in &self.edges {
            *out_degrees.entry(edge.source.clone()).or_insert(0) += 1;
            *in_degrees.entry(edge.target.clone()).or_insert(0) += 1;
        }

        // Assign community cluster by file path
        let mut file_to_community: HashMap<String, usize> = HashMap::new();
        let mut community_counter = 0;

        let graph_nodes: Vec<GraphNode> = self
            .nodes
            .iter()
            .map(|node| {
                let in_deg = in_degrees.get(&node.id).copied().unwrap_or(0);
                let out_deg = out_degrees.get(&node.id).copied().unwrap_or(0);
                let degree = in_deg + out_deg;

                let community_id =
                    *file_to_community
                        .entry(node.file.clone())
                        .or_insert_with(|| {
                            let id = community_counter % 12;
                            community_counter += 1;
                            id
                        });

                GraphNode {
                    id: node.id.clone(),
                    path: PathBuf::from(&node.file),
                    label: node.label.clone(),
                    degree,
                    is_unresolved: false,
                    is_tag: node.kind == ProjectNodeKind::File,
                    in_degree: in_deg,
                    out_degree: out_deg,
                    centrality: 0,
                    community_id,
                }
            })
            .collect();

        let graph_edges: Vec<GraphEdge> = self
            .edges
            .iter()
            .map(|edge| GraphEdge {
                source: edge.source.clone(),
                target: edge.target.clone(),
            })
            .collect();

        GraphData {
            nodes: graph_nodes,
            edges: graph_edges,
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn collect_symbol_nodes_and_edges(
    sym: &Symbol,
    package: &str,
    rel_path: &str,
    parent_id: &str,
    parent_sym_name: Option<&str>,
    nodes: &mut Vec<ProjectNode>,
    edges: &mut Vec<ProjectEdge>,
    symbol_name_to_ids: &mut HashMap<String, Vec<String>>,
) {
    let node_kind = ProjectNodeKind::from(&sym.kind);

    let symbol_identity = match parent_sym_name {
        Some(parent) => format!("{}::{}", parent, sym.name),
        None => sym.name.clone(),
    };

    let node_id = format!(
        "sym:{}::{}::{}::{}",
        package, rel_path, node_kind, symbol_identity
    );

    let vis_str = match &sym.visibility {
        Visibility::Public => "pub",
        Visibility::Crate => "pub(crate)",
        Visibility::Restricted(r) => r.as_str(),
        Visibility::Private => "private",
    };

    let label = match node_kind {
        ProjectNodeKind::Function => format!("fn {}()", sym.name),
        ProjectNodeKind::Method => format!("{}.{}()", parent_sym_name.unwrap_or(""), sym.name),
        ProjectNodeKind::Struct => format!("struct {}", sym.name),
        ProjectNodeKind::Enum => format!("enum {}", sym.name),
        ProjectNodeKind::EnumVariant => format!("::{}", sym.name),
        ProjectNodeKind::Trait => format!("trait {}", sym.name),
        ProjectNodeKind::Implementation => format!("impl {}", sym.name),
        ProjectNodeKind::Module => format!("mod {}", sym.name),
        ProjectNodeKind::TypeAlias => format!("type {}", sym.name),
        ProjectNodeKind::Constant => format!("const {}", sym.name),
        ProjectNodeKind::Static => format!("static {}", sym.name),
        ProjectNodeKind::Macro => format!("{}!", sym.name),
        _ => sym.name.clone(),
    };

    nodes.push(ProjectNode {
        id: node_id.clone(),
        kind: node_kind,
        name: sym.name.clone(),
        label,
        file: rel_path.to_string(),
        visibility: vis_str.to_string(),
        span: Some(sym.span),
        signature: sym.signature.clone(),
        doc: sym.doc_comment.clone(),
        parent_id: Some(parent_id.to_string()),
    });

    // Register simple name for cross-reference linking
    symbol_name_to_ids
        .entry(sym.name.clone())
        .or_default()
        .push(node_id.clone());

    // Containment edge from parent
    edges.push(ProjectEdge {
        source: parent_id.to_string(),
        target: node_id.clone(),
        kind: ProjectEdgeKind::Contains,
    });

    // Recurse for child symbols (methods, fields, variants)
    for child in &sym.children {
        collect_symbol_nodes_and_edges(
            child,
            package,
            rel_path,
            &node_id,
            Some(&sym.name),
            nodes,
            edges,
            symbol_name_to_ids,
        );
    }
}
