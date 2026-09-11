use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A parsed Wikilink occurrence inside a Markdown document.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Wikilink {
    /// Full original text including brackets, e.g. `[[Target|Alias]]`.
    pub raw: String,
    /// Destination note name or path, e.g. `Target` or `folder/Target`.
    pub target: String,
    /// Optional custom display alias, e.g. `Alias`.
    pub display_text: Option<String>,
    /// Byte start offset in source text.
    pub start: usize,
    /// Byte end offset in source text.
    pub end: usize,
}

impl Wikilink {
    /// Returns the effective label to display in the UI (alias if present, otherwise target).
    pub fn label(&self) -> &str {
        self.display_text.as_deref().unwrap_or(&self.target)
    }
}

/// Parses all Wikilinks from the given Markdown text.
pub fn extract_wikilinks(content: &str) -> Vec<Wikilink> {
    let mut links = Vec::new();
    let mut start_search = 0;

    while let Some(open_idx) = content[start_search..].find("[[") {
        let actual_open = start_search + open_idx;
        let after_open = actual_open + 2;

        if let Some(close_idx) = content[after_open..].find("]]") {
            let actual_close = after_open + close_idx;
            let inner = &content[after_open..actual_close];

            // Avoid multi-line matches as Wikilinks must be on a single line
            if !inner.contains('\n') && !inner.trim().is_empty() {
                let raw = content[actual_open..actual_close + 2].to_string();
                let (target, display_text) = parse_wikilink_inner(inner);

                if !target.is_empty() {
                    links.push(Wikilink {
                        raw,
                        target,
                        display_text,
                        start: actual_open,
                        end: actual_close + 2,
                    });
                }
            }

            start_search = actual_close + 2;
        } else {
            break;
        }
    }

    links
}

/// Parses target and optional alias from inner string: `target|alias`.
fn parse_wikilink_inner(inner: &str) -> (String, Option<String>) {
    if let Some(pipe_pos) = inner.find('|') {
        let target = inner[..pipe_pos].trim().to_string();
        let alias = inner[pipe_pos + 1..].trim().to_string();
        let display = if alias.is_empty() { None } else { Some(alias) };
        (target, display)
    } else {
        (inner.trim().to_string(), None)
    }
}

/// Rewrites Wikilink targets in Markdown content using a target rename mapping.
/// Preserves display aliases and non-matching links.
pub fn rewrite_wikilinks(content: &str, renames: &HashMap<String, String>) -> String {
    let links = extract_wikilinks(content);
    if links.is_empty() || renames.is_empty() {
        return content.to_string();
    }

    let mut result = String::with_capacity(content.len());
    let mut last_idx = 0;

    for link in links {
        if let Some(new_target) = renames.get(&link.target) {
            result.push_str(&content[last_idx..link.start]);
            result.push_str("[[");
            result.push_str(new_target);
            if let Some(alias) = &link.display_text {
                result.push('|');
                result.push_str(alias);
            }
            result.push_str("]]");
            last_idx = link.end;
        }
    }

    result.push_str(&content[last_idx..]);
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_simple_wikilink() {
        let text = "Refer to [[Rust Ownership]] for details.";
        let links = extract_wikilinks(text);
        assert_eq!(links.len(), 1);
        assert_eq!(links[0].target, "Rust Ownership");
        assert_eq!(links[0].display_text, None);
        assert_eq!(links[0].label(), "Rust Ownership");
        assert_eq!(links[0].raw, "[[Rust Ownership]]");
    }

    #[test]
    fn test_extract_nested_path_and_alias() {
        let text = "See [[Guides/Rust Patterns|Rust Idioms]] in the vault.";
        let links = extract_wikilinks(text);
        assert_eq!(links.len(), 1);
        assert_eq!(links[0].target, "Guides/Rust Patterns");
        assert_eq!(links[0].display_text.as_deref(), Some("Rust Idioms"));
        assert_eq!(links[0].label(), "Rust Idioms");
    }

    #[test]
    fn test_extract_multiple_wikilinks() {
        let text = "Links: [[Note A]], [[Folder/Note B|B]], and [[Note C]].";
        let links = extract_wikilinks(text);
        assert_eq!(links.len(), 3);
        assert_eq!(links[0].target, "Note A");
        assert_eq!(links[1].target, "Folder/Note B");
        assert_eq!(links[1].label(), "B");
        assert_eq!(links[2].target, "Note C");
    }

    #[test]
    fn test_rewrite_wikilinks() {
        let text = "Check [[Old Note]] and [[Old Note|Custom Label]] here.";
        let mut renames = HashMap::new();
        renames.insert("Old Note".to_string(), "New Note".to_string());

        let rewritten = rewrite_wikilinks(text, &renames);
        assert_eq!(
            rewritten,
            "Check [[New Note]] and [[New Note|Custom Label]] here."
        );
    }
}
