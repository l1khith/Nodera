use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

use crate::wikilink::Wikilink;

/// A node in the visual knowledge graph representing a Markdown note or virtual target.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GraphNode {
    /// Unique identifier for the node (relative file path string or virtual id).
    pub id: String,
    /// Note path relative to the vault root.
    pub path: PathBuf,
    /// Display label (note title or file stem).
    pub label: String,
    /// Total connection degree (in-degree + out-degree) for node sizing.
    pub degree: usize,
    /// Whether this node represents an unresolved/phantom note that does not yet exist on disk.
    #[serde(default)]
    pub is_unresolved: bool,
    /// Whether this node represents a tag entity.
    #[serde(default)]
    pub is_tag: bool,
    /// Number of incoming connections.
    #[serde(default)]
    pub in_degree: usize,
    /// Number of outgoing connections.
    #[serde(default)]
    pub out_degree: usize,
    /// Normalized degree centrality (basis points: 0..10_000).
    #[serde(default)]
    pub centrality: u32,
    /// Assigned community cluster index (from Label Propagation Algorithm).
    #[serde(default)]
    pub community_id: usize,
}

impl Default for GraphNode {
    fn default() -> Self {
        Self {
            id: String::new(),
            path: PathBuf::new(),
            label: String::new(),
            degree: 0,
            is_unresolved: false,
            is_tag: false,
            in_degree: 0,
            out_degree: 0,
            centrality: 0,
            community_id: 0,
        }
    }
}

/// Filter settings applied when constructing graph nodes and edges.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GraphFilterOptions {
    /// When true, suppresses unresolved/non-existent note targets.
    /// When false (default, like Obsidian), links to non-existent notes are rendered as unresolved nodes.
    pub existing_files_only: bool,
    /// When true (default), includes isolated notes that have degree 0.
    /// When false, suppresses nodes with degree 0.
    pub orphans: bool,
    /// When true, includes tag nodes and edges from notes to their tags.
    pub tags: bool,
    /// When true, includes attachment nodes.
    pub attachments: bool,
}

impl Default for GraphFilterOptions {
    fn default() -> Self {
        Self {
            existing_files_only: false,
            orphans: true,
            tags: false,
            attachments: false,
        }
    }
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

    /// Builds knowledge graph data with full support for note titles, tag relationships, and filter options.
    pub fn to_graph_data_with_options(
        &self,
        all_paths: &[PathBuf],
        titles: &HashMap<PathBuf, String>,
        note_tags: &HashMap<PathBuf, Vec<String>>,
        options: &GraphFilterOptions,
    ) -> GraphData {
        let mut degree_map: HashMap<String, usize> = HashMap::new();
        let mut in_degree_map: HashMap<String, usize> = HashMap::new();
        let mut out_degree_map: HashMap<String, usize> = HashMap::new();
        let mut edges_set: HashSet<GraphEdge> = HashSet::new();
        let mut unresolved_targets: HashSet<String> = HashSet::new();

        // 1. Resolve all edges between known vault notes, and collect unresolved targets
        for (source_path, links) in &self.outgoing {
            let source_str = source_path.to_string_lossy().replace('\\', "/");
            for link in links {
                let trimmed_target = link.target.trim();
                if trimmed_target.is_empty() {
                    continue;
                }

                if let Some(target_path) = Self::resolve_target(trimmed_target, all_paths) {
                    if &target_path != source_path {
                        let target_str = target_path.to_string_lossy().replace('\\', "/");
                        let edge = GraphEdge {
                            source: source_str.clone(),
                            target: target_str.clone(),
                        };
                        if edges_set.insert(edge) {
                            *degree_map.entry(source_str.clone()).or_insert(0) += 1;
                            *out_degree_map.entry(source_str.clone()).or_insert(0) += 1;
                            *degree_map.entry(target_str.clone()).or_insert(0) += 1;
                            *in_degree_map.entry(target_str).or_insert(0) += 1;
                        }
                    }
                } else if !options.existing_files_only {
                    // Target note does not exist on disk, but existing_files_only is false -> create unresolved node
                    let unresolved_id = format!("unresolved:{}", trimmed_target);
                    let edge = GraphEdge {
                        source: source_str.clone(),
                        target: unresolved_id.clone(),
                    };
                    if edges_set.insert(edge) {
                        *degree_map.entry(source_str.clone()).or_insert(0) += 1;
                        *out_degree_map.entry(source_str.clone()).or_insert(0) += 1;
                        *degree_map.entry(unresolved_id.clone()).or_insert(0) += 1;
                        *in_degree_map.entry(unresolved_id.clone()).or_insert(0) += 1;
                        unresolved_targets.insert(trimmed_target.to_string());
                    }
                }
            }
        }

        // 2. Include tags if options.tags is true
        let mut tag_nodes = Vec::new();
        if options.tags {
            let mut tag_degree_map: HashMap<String, usize> = HashMap::new();
            for (note_path, tags) in note_tags {
                let source_str = note_path.to_string_lossy().replace('\\', "/");
                for tag in tags {
                    let clean_tag = if tag.starts_with('#') {
                        tag.clone()
                    } else {
                        format!("#{tag}")
                    };
                    let tag_id = format!("tag:{}", clean_tag);
                    let edge = GraphEdge {
                        source: source_str.clone(),
                        target: tag_id.clone(),
                    };
                    if edges_set.insert(edge) {
                        *degree_map.entry(source_str.clone()).or_insert(0) += 1;
                        *out_degree_map.entry(source_str.clone()).or_insert(0) += 1;
                        *tag_degree_map.entry(clean_tag.clone()).or_insert(0) += 1;
                        *in_degree_map.entry(tag_id).or_insert(0) += 1;
                    }
                }
            }
            for (tag_name, deg) in tag_degree_map {
                let tag_id = format!("tag:{}", tag_name);
                let in_deg = in_degree_map.get(&tag_id).copied().unwrap_or(deg);
                tag_nodes.push(GraphNode {
                    id: tag_id,
                    path: PathBuf::from(&tag_name),
                    label: tag_name,
                    degree: deg,
                    is_unresolved: false,
                    is_tag: true,
                    in_degree: in_deg,
                    out_degree: 0,
                    centrality: 0,
                    community_id: 0,
                });
            }
        }

        // 3. Build nodes for physical vault notes
        let mut nodes = Vec::new();
        for path in all_paths {
            let path_str = path.to_string_lossy().replace('\\', "/");
            let label = titles
                .get(path)
                .cloned()
                .filter(|t| !t.trim().is_empty())
                .unwrap_or_else(|| {
                    path.file_stem()
                        .and_then(|s| s.to_str())
                        .unwrap_or(&path_str)
                        .to_string()
                });
            let degree = degree_map.get(&path_str).copied().unwrap_or(0);
            // Check orphans filter: if orphans == false, skip notes with degree == 0
            if !options.orphans && degree == 0 {
                continue;
            }

            let in_deg = in_degree_map.get(&path_str).copied().unwrap_or(0);
            let out_deg = out_degree_map.get(&path_str).copied().unwrap_or(0);

            nodes.push(GraphNode {
                id: path_str,
                path: path.clone(),
                label,
                degree,
                is_unresolved: false,
                is_tag: false,
                in_degree: in_deg,
                out_degree: out_deg,
                centrality: 0,
                community_id: 0,
            });
        }

        // 4. Build nodes for unresolved targets
        if !options.existing_files_only {
            for target_name in unresolved_targets {
                let unresolved_id = format!("unresolved:{}", target_name);
                let degree = degree_map.get(&unresolved_id).copied().unwrap_or(1);
                let in_deg = in_degree_map.get(&unresolved_id).copied().unwrap_or(degree);
                let out_deg = out_degree_map.get(&unresolved_id).copied().unwrap_or(0);
                nodes.push(GraphNode {
                    id: unresolved_id,
                    path: PathBuf::from(&target_name),
                    label: target_name,
                    degree,
                    is_unresolved: true,
                    is_tag: false,
                    in_degree: in_deg,
                    out_degree: out_deg,
                    centrality: 0,
                    community_id: 0,
                });
            }
        }

        // 5. Append tag nodes
        nodes.extend(tag_nodes);

        nodes.sort_by(|a, b| a.id.cmp(&b.id));

        // Filter edges to only include those where both source and target exist in nodes
        let node_id_set: HashSet<&str> = nodes.iter().map(|n| n.id.as_str()).collect();
        let mut edges: Vec<GraphEdge> = edges_set
            .into_iter()
            .filter(|e| {
                node_id_set.contains(e.source.as_str()) && node_id_set.contains(e.target.as_str())
            })
            .collect();
        edges.sort_by(|a, b| (&a.source, &a.target).cmp(&(&b.source, &b.target)));

        // Run community detection (LPA) and degree centrality
        detect_communities(&mut nodes, &edges);
        calculate_centrality(&mut nodes);

        GraphData { nodes, edges }
    }

    /// Builds full knowledge graph data for all notes in the vault, using note display titles where available.
    pub fn to_graph_data_with_titles(
        &self,
        all_paths: &[PathBuf],
        titles: &HashMap<PathBuf, String>,
    ) -> GraphData {
        self.to_graph_data_with_options(
            all_paths,
            titles,
            &HashMap::new(),
            &GraphFilterOptions {
                existing_files_only: true, // preserve default existing files behavior for this legacy method
                orphans: true,
                tags: false,
                attachments: false,
            },
        )
    }

    /// Builds full knowledge graph data for all notes in the vault.
    pub fn to_graph_data(&self, all_paths: &[PathBuf]) -> GraphData {
        self.to_graph_data_with_titles(all_paths, &HashMap::new())
    }

    /// Extracts a local neighborhood subgraph around `active_note` with filter options.
    pub fn to_local_graph_data_with_options(
        &self,
        active_note: &Path,
        all_paths: &[PathBuf],
        titles: &HashMap<PathBuf, String>,
        note_tags: &HashMap<PathBuf, Vec<String>>,
        options: &GraphFilterOptions,
        depth: usize,
    ) -> GraphData {
        let full_graph = self.to_graph_data_with_options(all_paths, titles, note_tags, options);
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

    /// Extracts a local neighborhood subgraph around `active_note` up to `depth` hops with custom display titles.
    pub fn to_local_graph_data_with_titles(
        &self,
        active_note: &Path,
        all_paths: &[PathBuf],
        titles: &HashMap<PathBuf, String>,
        depth: usize,
    ) -> GraphData {
        self.to_local_graph_data_with_options(
            active_note,
            all_paths,
            titles,
            &HashMap::new(),
            &GraphFilterOptions {
                existing_files_only: true,
                orphans: true,
                tags: false,
                attachments: false,
            },
            depth,
        )
    }

    /// Extracts a local neighborhood subgraph around `active_note` up to `depth` hops (default 1).
    pub fn to_local_graph_data(
        &self,
        active_note: &Path,
        all_paths: &[PathBuf],
        depth: usize,
    ) -> GraphData {
        self.to_local_graph_data_with_titles(active_note, all_paths, &HashMap::new(), depth)
    }

    /// Audits vault links to discover broken wikilinks and orphan notes.
    pub fn audit_vault_links(
        &self,
        all_paths: &[PathBuf],
        note_contents: &HashMap<PathBuf, String>,
    ) -> LinkAuditReport {
        let mut broken_map: HashMap<String, Vec<BrokenLinkItem>> = HashMap::new();
        let mut total_links = 0;

        let mut in_degree: HashMap<PathBuf, usize> = HashMap::new();
        let mut out_degree: HashMap<PathBuf, usize> = HashMap::new();

        for path in all_paths {
            in_degree.insert(path.clone(), 0);
            out_degree.insert(path.clone(), 0);
        }

        for (source_path, links) in &self.outgoing {
            *out_degree.entry(source_path.clone()).or_insert(0) += links.len();
            total_links += links.len();

            let content = note_contents
                .get(source_path)
                .map(String::as_str)
                .unwrap_or("");

            for link in links {
                if let Some(resolved) = Self::resolve_target(&link.target, all_paths) {
                    *in_degree.entry(resolved).or_insert(0) += 1;
                } else {
                    let clean_target = link.target.trim().to_string();

                    // Find line number and snippet in content
                    let mut line_num = 1;
                    let mut snippet = String::new();
                    let target_pattern = format!("[[{}", link.target);

                    for (i, line) in content.lines().enumerate() {
                        if line.contains(&target_pattern) || line.contains(&link.target) {
                            line_num = i + 1;
                            snippet = line.trim().to_string();
                            break;
                        }
                    }

                    if snippet.is_empty() {
                        snippet = format!("[[{}]]", clean_target);
                    }

                    broken_map
                        .entry(clean_target.clone())
                        .or_default()
                        .push(BrokenLinkItem {
                            target: clean_target,
                            source_path: source_path.clone(),
                            line_number: line_num,
                            snippet,
                        });
                }
            }
        }

        // Collect orphan notes: 0 in-degree and 0 out-degree
        let mut orphan_notes = Vec::new();
        for path in all_paths {
            let in_deg = in_degree.get(path).copied().unwrap_or(0);
            let out_deg = out_degree.get(path).copied().unwrap_or(0);
            if in_deg == 0 && out_deg == 0 {
                orphan_notes.push(path.clone());
            }
        }
        orphan_notes.sort();

        let mut broken_links: Vec<BrokenLinkGroup> = broken_map
            .into_iter()
            .map(|(target, occurrences)| BrokenLinkGroup {
                target,
                occurrences,
            })
            .collect();
        broken_links.sort_by(|a, b| {
            b.occurrences
                .len()
                .cmp(&a.occurrences.len())
                .then_with(|| a.target.cmp(&b.target))
        });

        LinkAuditReport {
            broken_links,
            orphan_notes,
            total_notes: all_paths.len(),
            total_links,
        }
    }
}

/// Groups graph nodes into community clusters using the Label Propagation Algorithm (LPA).
pub fn detect_communities(nodes: &mut [GraphNode], edges: &[GraphEdge]) {
    if nodes.is_empty() {
        return;
    }

    // Build bidirectional adjacency list
    let mut adj: HashMap<&str, Vec<&str>> = HashMap::new();
    for edge in edges {
        adj.entry(edge.source.as_str())
            .or_default()
            .push(edge.target.as_str());
        adj.entry(edge.target.as_str())
            .or_default()
            .push(edge.source.as_str());
    }

    // Initialize each node with its index as label
    let mut labels: HashMap<String, usize> = HashMap::new();
    for (i, node) in nodes.iter().enumerate() {
        labels.insert(node.id.clone(), i);
    }

    // Deterministic sorted node ids to ensure reproducible clustering
    let sorted_ids: Vec<String> = nodes.iter().map(|n| n.id.clone()).collect();

    // Iterate up to 15 times or until convergence
    for _ in 0..15 {
        let mut changed = false;

        for id in &sorted_ids {
            let neighbors = match adj.get(id.as_str()) {
                Some(n) if !n.is_empty() => n,
                _ => continue,
            };

            // Count label frequencies among neighbors
            let mut counts: HashMap<usize, usize> = HashMap::new();
            for neighbor in neighbors {
                if let Some(&lbl) = labels.get(*neighbor) {
                    *counts.entry(lbl).or_insert(0) += 1;
                }
            }

            // Find most frequent label, tie-breaking on lowest label id
            if let Some((&best_lbl, _)) =
                counts.iter().max_by(|(lbl_a, count_a), (lbl_b, count_b)| {
                    count_a.cmp(count_b).then_with(|| lbl_b.cmp(lbl_a))
                })
            {
                if let Some(current) = labels.get_mut(id) {
                    if *current != best_lbl {
                        *current = best_lbl;
                        changed = true;
                    }
                }
            }
        }

        if !changed {
            break;
        }
    }

    // Map labels to consecutive community IDs (0..K) ordered by cluster size descending
    let mut cluster_sizes: HashMap<usize, usize> = HashMap::new();
    for &lbl in labels.values() {
        *cluster_sizes.entry(lbl).or_insert(0) += 1;
    }

    let mut ranked_clusters: Vec<(usize, usize)> = cluster_sizes.into_iter().collect();
    ranked_clusters.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));

    let mut label_to_community_id: HashMap<usize, usize> = HashMap::new();
    for (community_id, (old_label, _)) in ranked_clusters.into_iter().enumerate() {
        label_to_community_id.insert(old_label, community_id);
    }

    for node in nodes.iter_mut() {
        if let Some(&raw_lbl) = labels.get(&node.id) {
            node.community_id = label_to_community_id.get(&raw_lbl).copied().unwrap_or(0);
        }
    }
}

/// Computes normalized degree centrality (in basis points 0..10_000) for each node.
pub fn calculate_centrality(nodes: &mut [GraphNode]) {
    let n = nodes.len();
    if n <= 1 {
        for node in nodes.iter_mut() {
            node.centrality = 0;
        }
        return;
    }

    let max_possible = (n - 1) as f32;
    for node in nodes.iter_mut() {
        let norm = (node.degree as f32 / max_possible).clamp(0.0, 1.0);
        node.centrality = (norm * 10_000.0).round() as u32;
    }
}

/// Represents an occurrence of a broken link in a note.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BrokenLinkItem {
    pub target: String,
    pub source_path: PathBuf,
    pub line_number: usize,
    pub snippet: String,
}

/// A group of broken link occurrences sharing the same missing target.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BrokenLinkGroup {
    pub target: String,
    pub occurrences: Vec<BrokenLinkItem>,
}

/// Comprehensive audit report for broken links and orphan notes across a vault.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct LinkAuditReport {
    pub broken_links: Vec<BrokenLinkGroup>,
    pub orphan_notes: Vec<PathBuf>,
    pub total_notes: usize,
    pub total_links: usize,
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

    #[test]
    fn test_self_links_and_duplicate_edges_and_unresolved() {
        let mut graph = LinkGraph::new();

        let note_a = PathBuf::from("Notes/Alpha.md");
        let note_b = PathBuf::from("Notes/Beta.md");
        let paths = vec![note_a.clone(), note_b.clone()];

        // Note A contains:
        // - Self link to Alpha
        // - Multiple duplicate links to Beta
        // - Unresolved link to NonExistent
        graph.update_note_links(
            note_a.clone(),
            vec![
                Wikilink {
                    raw: "[[Alpha]]".to_string(),
                    target: "Alpha".to_string(),
                    display_text: None,
                    start: 0,
                    end: 9,
                },
                Wikilink {
                    raw: "[[Beta]]".to_string(),
                    target: "Beta".to_string(),
                    display_text: None,
                    start: 10,
                    end: 18,
                },
                Wikilink {
                    raw: "[[Beta|Alias]]".to_string(),
                    target: "Beta".to_string(),
                    display_text: Some("Alias".to_string()),
                    start: 19,
                    end: 33,
                },
                Wikilink {
                    raw: "[[NonExistent Note]]".to_string(),
                    target: "NonExistent Note".to_string(),
                    display_text: None,
                    start: 34,
                    end: 54,
                },
            ],
        );

        let mut titles = HashMap::new();
        titles.insert(note_a.clone(), "Alpha Custom Title".to_string());

        let graph_data = graph.to_graph_data_with_titles(&paths, &titles);

        // Should have exactly 2 nodes (Alpha, Beta)
        assert_eq!(graph_data.nodes.len(), 2);
        // Alpha has custom title
        let node_a = graph_data
            .nodes
            .iter()
            .find(|n| n.id.contains("Alpha"))
            .unwrap();
        assert_eq!(node_a.label, "Alpha Custom Title");
        assert_eq!(node_a.degree, 1);

        // Beta has fallback title from filename
        let node_b = graph_data
            .nodes
            .iter()
            .find(|n| n.id.contains("Beta"))
            .unwrap();
        assert_eq!(node_b.label, "Beta");
        assert_eq!(node_b.degree, 1);

        // Self-link excluded, duplicate edge deduped, unresolved ignored -> exactly 1 edge
        assert_eq!(graph_data.edges.len(), 1);
        assert_eq!(graph_data.edges[0].source, "Notes/Alpha.md");
        assert_eq!(graph_data.edges[0].target, "Notes/Beta.md");
    }

    #[test]
    fn test_local_graph_depth_expansion() {
        let mut graph = LinkGraph::new();

        // Chain: A <-> B <-> C <-> D
        let note_a = PathBuf::from("Notes/A.md");
        let note_b = PathBuf::from("Notes/B.md");
        let note_c = PathBuf::from("Notes/C.md");
        let note_d = PathBuf::from("Notes/D.md");

        let paths = vec![
            note_a.clone(),
            note_b.clone(),
            note_c.clone(),
            note_d.clone(),
        ];

        graph.update_note_links(
            note_a.clone(),
            vec![Wikilink {
                raw: "[[B]]".to_string(),
                target: "B".to_string(),
                display_text: None,
                start: 0,
                end: 5,
            }],
        );
        graph.update_note_links(
            note_b.clone(),
            vec![Wikilink {
                raw: "[[C]]".to_string(),
                target: "C".to_string(),
                display_text: None,
                start: 0,
                end: 5,
            }],
        );
        graph.update_note_links(
            note_c.clone(),
            vec![Wikilink {
                raw: "[[D]]".to_string(),
                target: "D".to_string(),
                display_text: None,
                start: 0,
                end: 5,
            }],
        );

        // Depth 1 from A: should contain A and B
        let d1 = graph.to_local_graph_data(&note_a, &paths, 1);
        assert_eq!(d1.nodes.len(), 2);
        assert_eq!(d1.edges.len(), 1);

        // Depth 2 from A: should contain A, B, and C
        let d2 = graph.to_local_graph_data(&note_a, &paths, 2);
        assert_eq!(d2.nodes.len(), 3);
        assert_eq!(d2.edges.len(), 2);

        // Depth 3 from A: should contain A, B, C, and D
        let d3 = graph.to_local_graph_data(&note_a, &paths, 3);
        assert_eq!(d3.nodes.len(), 4);
        assert_eq!(d3.edges.len(), 3);
    }

    #[test]
    fn test_unresolved_links_filter_options() {
        let mut graph = LinkGraph::new();

        let note_1 = PathBuf::from("Notes/Nodera_Graph_Project.md");
        let note_2 = PathBuf::from("Notes/Nodera_Rust_Architecture.md");
        let paths = vec![note_1.clone(), note_2.clone()];

        // Both notes link to conceptual notes that do not exist on disk
        graph.update_note_links(
            note_1.clone(),
            vec![
                Wikilink {
                    raw: "[[Graph View]]".to_string(),
                    target: "Graph View".to_string(),
                    display_text: None,
                    start: 0,
                    end: 14,
                },
                Wikilink {
                    raw: "[[Markdown Engine]]".to_string(),
                    target: "Markdown Engine".to_string(),
                    display_text: None,
                    start: 15,
                    end: 34,
                },
            ],
        );
        graph.update_note_links(
            note_2.clone(),
            vec![
                Wikilink {
                    raw: "[[Graph View]]".to_string(),
                    target: "Graph View".to_string(),
                    display_text: None,
                    start: 0,
                    end: 14,
                },
                Wikilink {
                    raw: "[[Vault Service]]".to_string(),
                    target: "Vault Service".to_string(),
                    display_text: None,
                    start: 15,
                    end: 32,
                },
            ],
        );

        // 1. When existing_files_only is FALSE (default, matching Obsidian):
        let with_unresolved = graph.to_graph_data_with_options(
            &paths,
            &HashMap::new(),
            &HashMap::new(),
            &GraphFilterOptions {
                existing_files_only: false,
                orphans: true,
                tags: false,
                attachments: false,
            },
        );

        // Should have 2 physical notes + 3 unresolved notes (Graph View, Markdown Engine, Vault Service) = 5 nodes
        assert_eq!(with_unresolved.nodes.len(), 5);
        // Graph View is linked by both Note 1 and Note 2 -> degree 2
        let graph_view_node = with_unresolved
            .nodes
            .iter()
            .find(|n| n.label == "Graph View")
            .unwrap();
        assert!(graph_view_node.is_unresolved);
        assert_eq!(graph_view_node.degree, 2);
        // Total edges: Note1->GraphView, Note1->MarkdownEngine, Note2->GraphView, Note2->VaultService = 4 edges
        assert_eq!(with_unresolved.edges.len(), 4);

        // 2. When existing_files_only is TRUE:
        let only_existing = graph.to_graph_data_with_options(
            &paths,
            &HashMap::new(),
            &HashMap::new(),
            &GraphFilterOptions {
                existing_files_only: true,
                orphans: true,
                tags: false,
                attachments: false,
            },
        );

        // Should have only the 2 physical notes, and 0 connections because they don't directly link to each other
        assert_eq!(only_existing.nodes.len(), 2);
        assert_eq!(only_existing.edges.len(), 0);
    }

    #[test]
    fn test_orphans_and_tags_filters() {
        let mut graph = LinkGraph::new();

        let note_a = PathBuf::from("Notes/Connected.md");
        let note_b = PathBuf::from("Notes/Orphan.md");
        let paths = vec![note_a.clone(), note_b.clone()];

        graph.update_note_links(
            note_a.clone(),
            vec![Wikilink {
                raw: "[[Unresolved Target]]".to_string(),
                target: "Unresolved Target".to_string(),
                display_text: None,
                start: 0,
                end: 21,
            }],
        );

        let mut note_tags = HashMap::new();
        note_tags.insert(note_a.clone(), vec!["systems".to_string()]);

        // When orphans = false: Orphan note is suppressed
        let no_orphans = graph.to_graph_data_with_options(
            &paths,
            &HashMap::new(),
            &note_tags,
            &GraphFilterOptions {
                existing_files_only: true,
                orphans: false,
                tags: false,
                attachments: false,
            },
        );
        // With existing_files_only=true, note_a also has degree 0, so both are suppressed
        assert_eq!(no_orphans.nodes.len(), 0);

        // When tags = true: Tag node is included
        let with_tags = graph.to_graph_data_with_options(
            &paths,
            &HashMap::new(),
            &note_tags,
            &GraphFilterOptions {
                existing_files_only: true,
                orphans: true,
                tags: true,
                attachments: false,
            },
        );
        // Notes (2) + Tag (1: #systems) = 3 nodes
        assert_eq!(with_tags.nodes.len(), 3);
        assert!(with_tags
            .nodes
            .iter()
            .any(|n| n.is_tag && n.label == "#systems"));
        // Edge between Connected and #systems
        assert_eq!(with_tags.edges.len(), 1);
    }

    #[test]
    fn test_detect_communities_and_centrality() {
        let mut nodes = vec![
            // Cluster 0: A0, A1, A2
            GraphNode {
                id: "A0".to_string(),
                path: PathBuf::from("A0.md"),
                label: "A0".to_string(),
                degree: 2,
                ..Default::default()
            },
            GraphNode {
                id: "A1".to_string(),
                path: PathBuf::from("A1.md"),
                label: "A1".to_string(),
                degree: 2,
                ..Default::default()
            },
            GraphNode {
                id: "A2".to_string(),
                path: PathBuf::from("A2.md"),
                label: "A2".to_string(),
                degree: 2,
                ..Default::default()
            },
            // Cluster 1: B0, B1
            GraphNode {
                id: "B0".to_string(),
                path: PathBuf::from("B0.md"),
                label: "B0".to_string(),
                degree: 1,
                ..Default::default()
            },
            GraphNode {
                id: "B1".to_string(),
                path: PathBuf::from("B1.md"),
                label: "B1".to_string(),
                degree: 1,
                ..Default::default()
            },
        ];

        let edges = vec![
            GraphEdge {
                source: "A0".to_string(),
                target: "A1".to_string(),
            },
            GraphEdge {
                source: "A1".to_string(),
                target: "A2".to_string(),
            },
            GraphEdge {
                source: "A2".to_string(),
                target: "A0".to_string(),
            },
            GraphEdge {
                source: "B0".to_string(),
                target: "B1".to_string(),
            },
        ];

        detect_communities(&mut nodes, &edges);
        calculate_centrality(&mut nodes);

        // A nodes should share the same community ID
        let a0_comm = nodes.iter().find(|n| n.id == "A0").unwrap().community_id;
        let a1_comm = nodes.iter().find(|n| n.id == "A1").unwrap().community_id;
        let a2_comm = nodes.iter().find(|n| n.id == "A2").unwrap().community_id;
        assert_eq!(a0_comm, a1_comm);
        assert_eq!(a1_comm, a2_comm);

        // B nodes should share a distinct community ID
        let b0_comm = nodes.iter().find(|n| n.id == "B0").unwrap().community_id;
        let b1_comm = nodes.iter().find(|n| n.id == "B1").unwrap().community_id;
        assert_eq!(b0_comm, b1_comm);
        assert_ne!(a0_comm, b0_comm);

        // Centrality should be non-zero for connected nodes
        let a0_cent = nodes.iter().find(|n| n.id == "A0").unwrap().centrality;
        assert!(a0_cent > 0);
    }

    #[test]
    fn test_audit_vault_links_and_orphans() {
        let mut graph = LinkGraph::new();
        let note_a = PathBuf::from("Notes/Alpha.md");
        let note_b = PathBuf::from("Notes/Beta.md");
        let note_orphan = PathBuf::from("Notes/Lonely.md");
        let all_paths = vec![note_a.clone(), note_b.clone(), note_orphan.clone()];

        // Alpha links to Beta and to a missing note [[NonExistent]]
        graph.update_note_links(
            note_a.clone(),
            vec![
                Wikilink {
                    raw: "[[Beta]]".to_string(),
                    target: "Beta".to_string(),
                    display_text: None,
                    start: 0,
                    end: 8,
                },
                Wikilink {
                    raw: "[[NonExistent]]".to_string(),
                    target: "NonExistent".to_string(),
                    display_text: None,
                    start: 9,
                    end: 24,
                },
            ],
        );

        let mut contents = HashMap::new();
        contents.insert(
            note_a.clone(),
            "See [[Beta]] or explore [[NonExistent]] for missing info.".to_string(),
        );
        contents.insert(note_b.clone(), "# Beta Note\nNothing here.".to_string());
        contents.insert(note_orphan.clone(), "# Lonely Note\nAll alone.".to_string());

        let report = graph.audit_vault_links(&all_paths, &contents);

        // Verify broken link detected
        assert_eq!(report.broken_links.len(), 1);
        assert_eq!(report.broken_links[0].target, "NonExistent");
        assert_eq!(report.broken_links[0].occurrences.len(), 1);
        assert_eq!(report.broken_links[0].occurrences[0].source_path, note_a);
        assert_eq!(report.broken_links[0].occurrences[0].line_number, 1);

        // Verify orphan detected (Lonely note has degree 0)
        assert_eq!(report.orphan_notes, vec![note_orphan]);
        assert_eq!(report.total_notes, 3);
        assert_eq!(report.total_links, 2);
    }
}
