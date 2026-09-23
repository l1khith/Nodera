use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use nodera_parser_core::{ParseOptions, ParserRegistry};

use crate::discovery::ProjectDiscovery;
use crate::error::{ProjectError, Result};
use crate::graph::ProjectGraph;
use crate::index::ProjectIndex;
use crate::model::{ProjectConfig, RustProject};
use crate::registry::{ProjectRegistry, ProjectRegistryEntry};
use crate::state::{compute_file_hash, ChangeSet, FileStateRecord, SourceState};

/// Summary result of an incremental synchronization or rebuild operation.
#[derive(Debug, Clone)]
pub struct SyncResult {
    pub project: RustProject,
    pub changes: ChangeSet,
    pub total_symbols: usize,
    pub total_nodes: usize,
    pub total_edges: usize,
    pub was_rebuilt: bool,
    pub last_updated: String,
}

pub struct ProjectSynchronizer;

impl ProjectSynchronizer {
    /// Synchronizes an initialized project with filesystem changes on disk.
    /// Returns an error if `.nodera/project.toml` does not exist.
    pub fn update(target_path: impl AsRef<Path>) -> Result<SyncResult> {
        let root = target_path.as_ref();
        if !root.is_dir() {
            return Err(ProjectError::NotADirectory {
                path: root.to_path_buf(),
            });
        }

        let nodera_dir = root.join(".nodera");
        let config_path = nodera_dir.join("project.toml");

        if !config_path.exists() {
            return Err(ProjectError::NotInitialized {
                path: root.to_path_buf(),
            });
        }

        // Re-discover project files from Cargo.toml
        let project = ProjectDiscovery::discover(root)?;
        let graph_path = nodera_dir.join("graph").join("project.json");
        let index_dir = nodera_dir.join("index");
        let state_path = nodera_dir.join("state").join("source_state.json");

        // If graph or index is missing, execute a safe rebuild
        let needs_rebuild = !graph_path.exists() || !index_dir.join("project.db").exists();

        let all_files: Vec<PathBuf> = project
            .all_rust_files()
            .into_iter()
            .map(|p| p.to_path_buf())
            .collect();

        let parser_registry = ParserRegistry::with_defaults();
        let parse_options = ParseOptions::default();

        if needs_rebuild {
            return Self::full_rebuild(
                &project,
                &all_files,
                &nodera_dir,
                &config_path,
                &graph_path,
                &index_dir,
                &state_path,
                &parser_registry,
                &parse_options,
            );
        }

        // Incremental sync
        let prev_state = SourceState::load_from_file(&state_path)?;
        let changes = SourceState::detect_changes(
            &project.root,
            &all_files,
            &project.manifest_path,
            &prev_state,
        )?;

        let mut graph =
            ProjectGraph::load_from_file(&graph_path).unwrap_or_else(|_| ProjectGraph {
                schema_version: 1,
                project: crate::graph::ProjectHeader {
                    id: crate::model::generate_stable_project_id(&project.root, &project.name),
                    name: project.name.clone(),
                    root: project.root.clone(),
                },
                files: Vec::new(),
                nodes: Vec::new(),
                edges: Vec::new(),
            });

        let index = ProjectIndex::open_or_create(&index_dir)?;
        let mut new_state_files = prev_state.files.clone();

        // 1. Process deleted files
        for deleted_rel in &changes.deleted {
            graph.files.retain(|f| &f.path != deleted_rel);
            graph.nodes.retain(|n| &n.file != deleted_rel);

            let node_ids_in_file: std::collections::HashSet<String> = graph
                .nodes
                .iter()
                .filter(|n| &n.file == deleted_rel)
                .map(|n| n.id.clone())
                .collect();

            graph.edges.retain(|e| {
                !node_ids_in_file.contains(&e.source) && !node_ids_in_file.contains(&e.target)
            });

            let _ = index.remove_file(deleted_rel);
            new_state_files.remove(deleted_rel);
        }

        // 2. Process modified and added files
        let files_to_parse: Vec<&PathBuf> = changes
            .modified
            .iter()
            .chain(changes.added.iter())
            .collect();

        for file_path in files_to_parse {
            let rel_path = file_path
                .strip_prefix(&project.root)
                .unwrap_or(file_path)
                .to_string_lossy()
                .replace('\\', "/");

            // Remove existing graph nodes and index records if modified
            graph.files.retain(|f| f.path != rel_path);
            let old_node_ids: std::collections::HashSet<String> = graph
                .nodes
                .iter()
                .filter(|n| n.file == rel_path)
                .map(|n| n.id.clone())
                .collect();

            graph.nodes.retain(|n| n.file != rel_path);
            graph
                .edges
                .retain(|e| !old_node_ids.contains(&e.source) && !old_node_ids.contains(&e.target));
            let _ = index.remove_file(&rel_path);

            // Parse file
            if let Ok(source_file) = parser_registry.parse_file(file_path, &parse_options) {
                let package_name = project
                    .packages
                    .iter()
                    .find(|p| file_path.starts_with(&p.package_root))
                    .map(|p| p.name.clone())
                    .unwrap_or_else(|| project.name.clone());

                // Build sub-graph for this file
                let sub_graph = ProjectGraph::build(
                    &project,
                    &graph.project.id,
                    &[(file_path.clone(), source_file.clone())],
                );

                for f in sub_graph.files {
                    graph.files.push(f);
                }
                for n in sub_graph.nodes {
                    let _ = index.insert_symbol(&n);
                    graph.nodes.push(n);
                }
                for e in sub_graph.edges {
                    graph.edges.push(e);
                }

                // Update index and state
                let meta = fs::metadata(file_path).ok();
                let mtime = meta
                    .as_ref()
                    .and_then(|m| m.modified().ok())
                    .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                    .map(|d| d.as_secs())
                    .unwrap_or(0);
                let size = meta.as_ref().map(|m| m.len()).unwrap_or(0);
                let hash = compute_file_hash(file_path).unwrap_or_default();

                let _ = index.index_file(
                    &rel_path,
                    &package_name,
                    mtime,
                    size,
                    &hash,
                    source_file.metrics.code_lines,
                    source_file.metrics.symbol_count,
                );

                new_state_files.insert(
                    rel_path,
                    FileStateRecord {
                        mtime_secs: mtime,
                        size_bytes: size,
                        hash,
                    },
                );
            }
        }

        // Deduplicate edges
        let mut unique_edges = Vec::with_capacity(graph.edges.len());
        let mut seen = std::collections::HashSet::new();
        for edge in graph.edges {
            if seen.insert((edge.source.clone(), edge.target.clone(), edge.kind)) {
                unique_edges.push(edge);
            }
        }
        graph.edges = unique_edges;

        // Save updated artifacts
        let now = chrono::Utc::now().to_rfc3339();
        graph.save_to_file(&graph_path)?;

        let new_manifest_hash = compute_file_hash(&project.manifest_path).ok();
        let updated_state = SourceState {
            version: 1,
            last_scan: now.clone(),
            manifest_hash: new_manifest_hash,
            files: new_state_files,
        };
        updated_state.save_to_file(&state_path)?;

        // Update project.toml last_updated_at
        if let Ok(content) = fs::read_to_string(&config_path) {
            if let Ok(mut config) = toml::from_str::<ProjectConfig>(&content) {
                config.state.last_updated_at = now.clone();
                if let Ok(serialized) = toml::to_string_pretty(&config) {
                    let _ = fs::write(&config_path, serialized);
                }
            }
        }

        // Update registry
        let _ = ProjectRegistry::register(ProjectRegistryEntry {
            id: graph.project.id.clone(),
            name: project.name.clone(),
            root: project.root.clone(),
            project_type: "rust".to_string(),
            last_updated: now.clone(),
        });

        let total_symbols = index.symbol_count().unwrap_or(0);

        Ok(SyncResult {
            project,
            changes,
            total_symbols,
            total_nodes: graph.nodes.len(),
            total_edges: graph.edges.len(),
            was_rebuilt: false,
            last_updated: now,
        })
    }

    /// Performs a full rebuild of the project representation from source files.
    #[allow(clippy::too_many_arguments)]
    fn full_rebuild(
        project: &RustProject,
        all_files: &[PathBuf],
        _nodera_dir: &Path,
        config_path: &Path,
        graph_path: &Path,
        index_dir: &Path,
        state_path: &Path,
        parser_registry: &ParserRegistry,
        parse_options: &ParseOptions,
    ) -> Result<SyncResult> {
        let mut parsed_files = Vec::new();
        let mut state_files = HashMap::new();

        let index = ProjectIndex::open_or_create(index_dir)?;

        for file_path in all_files {
            if let Ok(source_file) = parser_registry.parse_file(file_path, parse_options) {
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

                parsed_files.push((file_path.clone(), source_file));
            }
        }

        let project_id = crate::model::generate_stable_project_id(&project.root, &project.name);
        let graph = ProjectGraph::build(project, &project_id, &parsed_files);

        // Populate index symbols
        for node in &graph.nodes {
            let _ = index.insert_symbol(node);
        }

        graph.save_to_file(graph_path)?;

        let now = chrono::Utc::now().to_rfc3339();
        let manifest_hash = compute_file_hash(&project.manifest_path).ok();
        let state = SourceState {
            version: 1,
            last_scan: now.clone(),
            manifest_hash,
            files: state_files,
        };
        state.save_to_file(state_path)?;

        // Update project.toml
        let config = ProjectConfig::from_project(project);
        if let Ok(toml_str) = toml::to_string_pretty(&config) {
            let _ = fs::write(config_path, toml_str);
        }

        // Register in global registry
        let _ = ProjectRegistry::register(ProjectRegistryEntry {
            id: project_id,
            name: project.name.clone(),
            root: project.root.clone(),
            project_type: "rust".to_string(),
            last_updated: now.clone(),
        });

        let total_symbols = index.symbol_count().unwrap_or(0);

        Ok(SyncResult {
            project: project.clone(),
            changes: ChangeSet {
                added: all_files.to_vec(),
                modified: Vec::new(),
                deleted: Vec::new(),
                manifest_changed: false,
            },
            total_symbols,
            total_nodes: graph.nodes.len(),
            total_edges: graph.edges.len(),
            was_rebuilt: true,
            last_updated: now,
        })
    }
}
