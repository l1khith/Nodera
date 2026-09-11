use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::wikilink::Wikilink;

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
}
