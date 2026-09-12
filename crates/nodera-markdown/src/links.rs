use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

use crate::wikilink::Wikilink;

/// A node in the visual knowledge graph representing a Markdown note.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GraphNode {
    /// Unique identifier for the node (relative file path string).
    pub id: String,
    /// Note path relative to the vault root.
    pub path: PathBuf,
    /// Display label (note title or file stem).
    pub label: String,
    /// Total connection degree (in-degree + out-degree) for node sizing.
    pub degree: usize,
}

/// A directed edge in the knowledge graph representing a Wikilink from source to target note.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct GraphEdge {
    /// Source note path string.
    pub source: String,
    /// Target note path string.
    pub target: String,
}

/// Complete graph data containing all nodes and resolved edges for visualization.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct GraphData {
    pub nodes: Vec<GraphNode>,
    pub edges: Vec<GraphEdge>,
}

/// Maintains in-memory link relationships and graph connections between notes in a vault.
#[derive(Debug, Clone, Default)]
pub struct LinkGraph {
    /// Maps source note relative path -> list of outgoing Wikilinks
    outgoing: HashMap<PathBuf, Vec<Wikilink>>,
}

impl LinkGraph {
    pub fn new() -> Self {
        Self::default()
    }

    /// Records or updates all outgoing links for a specific note.
    pub fn update_note_links(&mut self, source_path: PathBuf, links: Vec<Wikilink>) {
        self.outgoing.insert(source_path, links);
    }

    /// Removes a note from the link graph upon deletion.
    pub fn remove_note(&mut self, source_path: &Path) {
        self.outgoing.remove(source_path);
    }

    /// Returns all outgoing links from a specific note.
    pub fn get_outgoing_links(&self, source_path: &Path) -> &[Wikilink] {
        self.outgoing
            .get(source_path)
            .map(|v| v.as_slice())
            .unwrap_or(&[])
    }

    /// Resolves a Wikilink target string to an existing note path in the vault.
    ///
    /// Resolution strategy:
    /// 1. Direct path match: if target is `Folder/Note` or `Folder/Note.md`.
    /// 2. Note title/stem match: matches case-insensitively against note filenames.
    pub fn resolve_target(target: &str, all_paths: &[PathBuf]) -> Option<PathBuf> {
        let clean_target = target.trim();
        let target_with_md = if clean_target.ends_with(".md") {
            clean_target.to_string()
        } else {
            format!("{clean_target}.md")
        };

        // 1. Direct relative path match
        for path in all_paths {
            let path_str = path.to_string_lossy().replace('\\', "/");
            if path_str.eq_ignore_ascii_case(&target_with_md)
                || path_str.eq_ignore_ascii_case(clean_target)
            {
                return Some(path.clone());
            }
        }

        // 2. Note title (file stem) match
        let target_stem = clean_target.strip_suffix(".md").unwrap_or(clean_target);

        for path in all_paths {
            if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                if stem.eq_ignore_ascii_case(target_stem) {
                    return Some(path.clone());
                }
            }
        }

        None
    }

    /// Finds all source notes that contain a Wikilink pointing to `target_note`.
    pub fn get_backlinks(&self, target_note: &Path, all_paths: &[PathBuf]) -> Vec<PathBuf> {
        let mut backlinks = Vec::new();

        for (source_path, links) in &self.outgoing {
            if source_path == target_note {
                continue;
            }

            for link in links {
                if let Some(resolved) = Self::resolve_target(&link.target, all_paths) {
                    if resolved == target_note {
                        backlinks.push(source_path.clone());
                        break;
                    }
                }
            }
        }

        backlinks.sort();
        backlinks
    }

    /// Returns link target suggestions for autocompletion while typing `[[...`.
    pub fn suggest_links(prefix: &str, all_paths: &[PathBuf]) -> Vec<String> {
        let query = prefix.trim().to_lowercase();
        let mut suggestions = Vec::new();

        for path in all_paths {
            if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                if stem.to_lowercase().contains(&query) {
                    suggestions.push(stem.to_string());
                }
            }
        }

        suggestions.sort();
        suggestions.dedup();
        suggestions
    }

    /// Builds full knowledge graph data for all notes in the vault.
    pub fn to_graph_data(&self, all_paths: &[PathBuf]) -> GraphData {
        let mut degree_map: HashMap<PathBuf, usize> = HashMap::new();
        let mut edges_set: HashSet<GraphEdge> = HashSet::new();

        // 1. Resolve all edges between known vault notes
        for (source_path, links) in &self.outgoing {
            let source_str = source_path.to_string_lossy().replace('\\', "/");
            for link in links {
                if let Some(target_path) = Self::resolve_target(&link.target, all_paths) {
                    if &target_path != source_path {
                        let target_str = target_path.to_string_lossy().replace('\\', "/");
                        let edge = GraphEdge {
                            source: source_str.clone(),
                            target: target_str,
                        };
                        if edges_set.insert(edge) {
                            *degree_map.entry(source_path.clone()).or_insert(0) += 1;
                            *degree_map.entry(target_path).or_insert(0) += 1;
                        }
                    }
                }
            }
        }

        // 2. Build nodes for all notes
        let mut nodes = Vec::new();
        for path in all_paths {
            let path_str = path.to_string_lossy().replace('\\', "/");
            let label = path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or(&path_str)
                .to_string();
            let degree = degree_map.get(path).copied().unwrap_or(0);
            nodes.push(GraphNode {
                id: path_str,
                path: path.clone(),
                label,
                degree,
            });
        }

        nodes.sort_by(|a, b| a.id.cmp(&b.id));
        let mut edges: Vec<GraphEdge> = edges_set.into_iter().collect();
        edges.sort_by(|a, b| (&a.source, &a.target).cmp(&(&b.source, &b.target)));

        GraphData { nodes, edges }
    }

    /// Extracts a local neighborhood subgraph around `active_note` up to `depth` hops (default 1).
    pub fn to_local_graph_data(
        &self,
        active_note: &Path,
        all_paths: &[PathBuf],
        depth: usize,
    ) -> GraphData {
        let full_graph = self.to_graph_data(all_paths);
        let active_str = active_note.to_string_lossy().replace('\\', "/");

        let mut included_ids: HashSet<String> = HashSet::new();
        included_ids.insert(active_str.clone());

        // Adjacency for undirected exploration
        let mut adj: HashMap<String, Vec<String>> = HashMap::new();
        for edge in &full_graph.edges {
            adj.entry(edge.source.clone())
                .or_default()
                .push(edge.target.clone());
            adj.entry(edge.target.clone())
                .or_default()
                .push(edge.source.clone());
        }

        let mut current_frontier = vec![active_str];
        for _ in 0..depth {
            let mut next_frontier = Vec::new();
            for node_id in current_frontier {
                if let Some(neighbors) = adj.get(&node_id) {
                    for neighbor in neighbors {
                        if included_ids.insert(neighbor.clone()) {
                            next_frontier.push(neighbor.clone());
                        }
                    }
                }
            }
            current_frontier = next_frontier;
            if current_frontier.is_empty() {
                break;
            }
        }

        let nodes: Vec<GraphNode> = full_graph
            .nodes
            .into_iter()
            .filter(|n| included_ids.contains(&n.id))
            .collect();

        let edges: Vec<GraphEdge> = full_graph
            .edges
            .into_iter()
            .filter(|e| included_ids.contains(&e.source) && included_ids.contains(&e.target))
            .collect();

        GraphData { nodes, edges }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolve_target_by_name_and_path() {
        let paths = vec![
            PathBuf::from("Notes/Rust Ownership.md"),
            PathBuf::from("Projects/Nodera Plan.md"),
        ];

        // Match by title
        let resolved = LinkGraph::resolve_target("Rust Ownership", &paths);
        assert_eq!(resolved, Some(PathBuf::from("Notes/Rust Ownership.md")));

        // Match case-insensitively
        let resolved_lower = LinkGraph::resolve_target("rust ownership", &paths);
        assert_eq!(
            resolved_lower,
            Some(PathBuf::from("Notes/Rust Ownership.md"))
        );

        // Match with path
        let resolved_path = LinkGraph::resolve_target("Projects/Nodera Plan", &paths);
        assert_eq!(
            resolved_path,
            Some(PathBuf::from("Projects/Nodera Plan.md"))
        );

        // Unresolved returns None
        let missing = LinkGraph::resolve_target("Nonexistent Note", &paths);
        assert!(missing.is_none());
    }

    #[test]
    fn test_backlinks_detection() {
        let mut graph = LinkGraph::new();

        let note_a = PathBuf::from("Notes/Note A.md");
        let note_b = PathBuf::from("Notes/Note B.md");
        let note_c = PathBuf::from("Notes/Note C.md");

        let paths = vec![note_a.clone(), note_b.clone(), note_c.clone()];

        // Note A links to Note B
        graph.update_note_links(
            note_a.clone(),
            vec![Wikilink {
                raw: "[[Note B]]".to_string(),
                target: "Note B".to_string(),
                display_text: None,
                start: 0,
                end: 10,
            }],
        );

        // Note C links to Note B
        graph.update_note_links(
            note_c.clone(),
            vec![Wikilink {
                raw: "[[Note B|Alias]]".to_string(),
                target: "Note B".to_string(),
                display_text: Some("Alias".to_string()),
                start: 0,
                end: 16,
            }],
        );

        // Note B's backlinks should be Note A and Note C
        let backlinks = graph.get_backlinks(&note_b, &paths);
        assert_eq!(backlinks, vec![note_a, note_c]);
    }

    #[test]
    fn test_suggest_links() {
        let paths = vec![
            PathBuf::from("Notes/Rust Ownership.md"),
            PathBuf::from("Notes/Rust Patterns.md"),
            PathBuf::from("Notes/Python Guide.md"),
        ];

        let suggestions = LinkGraph::suggest_links("rust", &paths);
        assert_eq!(suggestions, vec!["Rust Ownership", "Rust Patterns"]);
    }

    #[test]
    fn test_to_graph_data_and_local_graph() {
        let mut graph = LinkGraph::new();

        let note_a = PathBuf::from("Notes/Note A.md");
        let note_b = PathBuf::from("Notes/Note B.md");
        let note_c = PathBuf::from("Notes/Note C.md");
        let note_d = PathBuf::from("Notes/Note D.md");

        let paths = vec![
            note_a.clone(),
            note_b.clone(),
            note_c.clone(),
            note_d.clone(),
        ];

        // Note A links to Note B
        graph.update_note_links(
            note_a.clone(),
            vec![Wikilink {
                raw: "[[Note B]]".to_string(),
                target: "Note B".to_string(),
                display_text: None,
                start: 0,
                end: 10,
            }],
        );

        // Note B links to Note C
        graph.update_note_links(
            note_b.clone(),
            vec![Wikilink {
                raw: "[[Note C]]".to_string(),
                target: "Note C".to_string(),
                display_text: None,
                start: 0,
                end: 10,
            }],
        );

        // Global graph
        let full = graph.to_graph_data(&paths);
        assert_eq!(full.nodes.len(), 4);
        assert_eq!(full.edges.len(), 2);

        // Note B has in-degree 1 (from A) + out-degree 1 (to C) = degree 2
        let node_b = full.nodes.iter().find(|n| n.label == "Note B").unwrap();
        assert_eq!(node_b.degree, 2);

        // Note D has degree 0 (isolated note)
        let node_d = full.nodes.iter().find(|n| n.label == "Note D").unwrap();
        assert_eq!(node_d.degree, 0);

        // Local graph around Note B with depth 1: includes A, B, C (not D)
        let local_b = graph.to_local_graph_data(&note_b, &paths, 1);
        assert_eq!(local_b.nodes.len(), 3);
        assert_eq!(local_b.edges.len(), 2);
        assert!(local_b.nodes.iter().any(|n| n.label == "Note A"));
        assert!(local_b.nodes.iter().any(|n| n.label == "Note B"));
        assert!(local_b.nodes.iter().any(|n| n.label == "Note C"));
        assert!(!local_b.nodes.iter().any(|n| n.label == "Note D"));

        // Local graph around Note D: only includes D
        let local_d = graph.to_local_graph_data(&note_d, &paths, 1);
        assert_eq!(local_d.nodes.len(), 1);
        assert_eq!(local_d.edges.len(), 0);
    }
}
