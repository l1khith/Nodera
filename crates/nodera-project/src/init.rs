use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use nodera_parser_core::{ParseOptions, ParserRegistry};

use crate::discovery::ProjectDiscovery;
use crate::error::{ProjectError, Result};
use crate::graph::ProjectGraph;
use crate::index::ProjectIndex;
use crate::model::{generate_stable_project_id, ProjectConfig, RustProject};
use crate::registry::{ProjectRegistry, ProjectRegistryEntry};
use crate::state::{compute_file_hash, FileStateRecord, SourceState};

/// Initialization status indicating whether a fresh initialization or idempotent reconciliation occurred.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InitStatus {
    /// Newly initialized project.
    Initialized,
    /// Project was already initialized; existing metadata and configuration preserved.
    AlreadyInitialized,
}

/// Options controlling project initialization.
#[derive(Debug, Clone, Default)]
pub struct InitOptions {
    /// If true, force re-creation of project metadata and derived state.
    pub force: bool,
}

/// Summary result of a project initialization operation.
#[derive(Debug, Clone)]
pub struct InitResult {
    pub status: InitStatus,
    pub project: RustProject,
    pub project_id: String,
    pub nodera_dir: PathBuf,
    pub config_path: PathBuf,
    pub graph_path: PathBuf,
    pub index_path: PathBuf,
    pub symbols_parsed: usize,
    pub nodes_count: usize,
    pub edges_count: usize,
    pub gitignore_advisory: Option<String>,
}

pub struct ProjectInitializer;

impl ProjectInitializer {
    /// Initializes an existing Rust project or workspace as a Nodera-managed source workspace.
    /// Strictly idempotent: preserves existing configuration and indices, and never modifies source files.
    pub fn init(target_path: impl AsRef<Path>, options: &InitOptions) -> Result<InitResult> {
        let project = ProjectDiscovery::discover(target_path)?;

        let nodera_dir = project.root.join(".nodera");
        let config_path = nodera_dir.join("project.toml");
        let graph_dir = nodera_dir.join("graph");
        let graph_path = graph_dir.join("project.json");
        let index_dir = nodera_dir.join("index");
        let state_dir = nodera_dir.join("state");
        let state_path = state_dir.join("source_state.json");

        let project_id = generate_stable_project_id(&project.root, &project.name);

        if config_path.exists() && !options.force {
            // Load existing graph statistics if available
            let (nodes_count, edges_count, symbols_parsed) =
                if let Ok(graph) = ProjectGraph::load_from_file(&graph_path) {
                    let sym_count = graph
                        .nodes
                        .iter()
                        .filter(|n| n.kind != crate::graph::ProjectNodeKind::File)
                        .count();
                    (graph.nodes.len(), graph.edges.len(), sym_count)
                } else {
                    (0, 0, 0)
                };

            // Ensure registration exists
            let now = chrono::Utc::now().to_rfc3339();
            let _ = ProjectRegistry::register(ProjectRegistryEntry {
                id: project_id.clone(),
                name: project.name.clone(),
                root: project.root.clone(),
                project_type: "rust".to_string(),
                last_updated: now,
            });

            let gitignore_advisory = check_gitignore(&project.root);

            return Ok(InitResult {
                status: InitStatus::AlreadyInitialized,
                project,
                project_id,
                nodera_dir,
                config_path,
                graph_path,
                index_path: index_dir,
                symbols_parsed,
                nodes_count,
                edges_count,
                gitignore_advisory,
            });
        }

        // Create directory layout
        for dir in [
            &nodera_dir,
            &graph_dir,
            &index_dir,
            &state_dir,
            &nodera_dir.join("cache"),
        ] {
            fs::create_dir_all(dir).map_err(|e| ProjectError::Io {
                path: dir.to_path_buf(),
                source: e,
            })?;
        }

        // Parse all discovered source files using existing nodera-parser-core
        let parser_registry = ParserRegistry::with_defaults();
        let parse_options = ParseOptions::default();

        let all_files = project.all_rust_files();
        let mut parsed_files = Vec::new();
        let mut state_files = HashMap::new();

        let index = ProjectIndex::open_or_create(&index_dir)?;

        for file_path in all_files {
            if let Ok(source_file) = parser_registry.parse_file(file_path, &parse_options) {
                let rel = file_path
                    .strip_prefix(&project.root)
                    .unwrap_or(file_path)
                    .to_string_lossy()
                    .replace('\\', "/");

                let meta = fs::metadata(file_path).ok();
                let mtime = meta
                    .as_ref()
                    .and_then(|m| m.modified().ok())
                    .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                    .map(|d| d.as_secs())
                    .unwrap_or(0);
                let size = meta.as_ref().map(|m| m.len()).unwrap_or(0);
                let hash = compute_file_hash(file_path).unwrap_or_default();

                let package_name = project
                    .packages
                    .iter()
                    .find(|p| file_path.starts_with(&p.package_root))
                    .map(|p| p.name.clone())
                    .unwrap_or_else(|| project.name.clone());

                let _ = index.index_file(
                    &rel,
                    &package_name,
                    mtime,
                    size,
                    &hash,
                    source_file.metrics.code_lines,
                    source_file.metrics.symbol_count,
                );

                state_files.insert(
                    rel,
                    FileStateRecord {
                        mtime_secs: mtime,
                        size_bytes: size,
                        hash,
                    },
                );

                parsed_files.push((file_path.to_path_buf(), source_file));
            }
        }

        // Build Project Graph
        let graph = ProjectGraph::build(&project, &project_id, &parsed_files);

        // Populate index symbols
        for node in &graph.nodes {
            let _ = index.insert_symbol(node);
        }

        // Save Project Graph to .nodera/graph/project.json
        graph.save_to_file(&graph_path)?;

        // Save Source State to .nodera/state/source_state.json
        let now = chrono::Utc::now().to_rfc3339();
        let manifest_hash = compute_file_hash(&project.manifest_path).ok();
        let state = SourceState {
            version: 1,
            last_scan: now.clone(),
            manifest_hash,
            files: state_files,
        };
        state.save_to_file(&state_path)?;

        // Write .nodera/project.toml
        let config = ProjectConfig::from_project(&project);
        let toml_str =
            toml::to_string_pretty(&config).map_err(|e| ProjectError::Serialization {
                reason: format!("Failed to serialize project config: {e}"),
            })?;

        fs::write(&config_path, toml_str).map_err(|e| ProjectError::Io {
            path: config_path.clone(),
            source: e,
        })?;

        // Register in global ProjectRegistry
        let _ = ProjectRegistry::register(ProjectRegistryEntry {
            id: project_id.clone(),
            name: project.name.clone(),
            root: project.root.clone(),
            project_type: "rust".to_string(),
            last_updated: now,
        });

        let symbols_parsed = graph
            .nodes
            .iter()
            .filter(|n| n.kind != crate::graph::ProjectNodeKind::File)
            .count();

        let gitignore_advisory = check_gitignore(&project.root);

        Ok(InitResult {
            status: InitStatus::Initialized,
            project,
            project_id,
            nodera_dir,
            config_path,
            graph_path,
            index_path: index_dir,
            symbols_parsed,
            nodes_count: graph.nodes.len(),
            edges_count: graph.edges.len(),
            gitignore_advisory,
        })
    }
}

/// Checks if `.gitignore` exists and whether `.nodera` is ignored.
fn check_gitignore(root: &Path) -> Option<String> {
    let gitignore_path = root.join(".gitignore");
    if gitignore_path.exists() {
        if let Ok(content) = fs::read_to_string(&gitignore_path) {
            let has_nodera = content.lines().any(|l| {
                let trimmed = l.trim();
                trimmed == ".nodera"
                    || trimmed == ".nodera/"
                    || trimmed == "/.nodera"
                    || trimmed == "/.nodera/"
                    || trimmed == ".nodera/index"
                    || trimmed == ".nodera/cache"
            });
            if !has_nodera {
                return Some(
                    "Note: '.nodera/' is not ignored in .gitignore. You may want to add '.nodera/index' and '.nodera/cache' to .gitignore."
                        .to_string(),
                );
            }
        }
    }
    None
}
