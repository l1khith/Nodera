use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::debug;

use nodera_core::{ParseError, Result};

/// Parsed metadata from a YAML frontmatter block.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct Frontmatter {
    pub title: Option<String>,
    pub tags: Vec<String>,
    pub extra: HashMap<String, serde_yaml::Value>,
}

/// Parses frontmatter from a Markdown document.
///
/// Returns the parsed `Frontmatter` (if present) and the remaining document body.
pub fn parse_frontmatter(content: &str) -> Result<(Option<Frontmatter>, &str)> {
    let trimmed = content.trim_start();
    if !trimmed.starts_with("---") {
        return Ok((None, content));
    }

    // Find the end of the opening delimiter line
    let after_opening = match trimmed[3..].find('\n') {
        Some(pos) => &trimmed[3 + pos + 1..],
        None => return Ok((None, content)),
    };

    // Find the closing delimiter ("---" or "...") at the beginning of a line
    let mut end_offset = None;
    let mut current_pos = 0;

    for line in after_opening.lines() {
        let trimmed_line = line.trim();
        if trimmed_line == "---" || trimmed_line == "..." {
            end_offset = Some(current_pos);
            break;
        }
        current_pos += line.len() + 1; // account for newline
    }

    let end_pos = match end_offset {
        Some(pos) => pos,
        None => return Ok((None, content)),
    };

    let yaml_str = &after_opening[..end_pos];
    // Body begins after the closing delimiter line
    let after_closing = &after_opening[end_pos..];
    let body = match after_closing.find('\n') {
        Some(newline_pos) => &after_closing[newline_pos + 1..],
        None => "",
    };

    if yaml_str.trim().is_empty() {
        return Ok((Some(Frontmatter::default()), body));
    }

    let raw_map: HashMap<String, serde_yaml::Value> =
        serde_yaml::from_str(yaml_str).map_err(|e| ParseError::MalformedFrontmatter {
            line: 1,
            reason: format!("YAML syntax error: {e}"),
        })?;

    let mut frontmatter = Frontmatter::default();
    let mut extra = HashMap::new();

    for (key, val) in raw_map {
        match key.as_str() {
            "title" => {
                if let Some(t) = val.as_str() {
                    frontmatter.title = Some(t.to_string());
                } else {
                    extra.insert(key, val);
                }
            }
            "tags" => {
                if let Some(seq) = val.as_sequence() {
                    for item in seq {
                        if let Some(tag_str) = item.as_str() {
                            frontmatter
                                .tags
                                .push(tag_str.trim_start_matches('#').to_string());
                        }
                    }
                } else if let Some(tag_str) = val.as_str() {
                    for part in tag_str.split(',').map(str::trim) {
                        if !part.is_empty() {
                            frontmatter
                                .tags
                                .push(part.trim_start_matches('#').to_string());
                        }
                    }
                } else {
                    extra.insert(key, val);
                }
            }
            _ => {
                extra.insert(key, val);
            }
        }
    }

    frontmatter.extra = extra;
    debug!(tags = ?frontmatter.tags, title = ?frontmatter.title, "Parsed frontmatter successfully");

    Ok((Some(frontmatter), body))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_no_frontmatter() {
        let text = "# Just Markdown\nHello world.";
        let (fm, body) = parse_frontmatter(text).unwrap();
        assert!(fm.is_none());
        assert_eq!(body, text);
    }

    #[test]
    fn test_valid_yaml_frontmatter() {
        let text = "---\ntitle: My Document\ntags:\n  - rust\n  - desktop\nstatus: active\n---\n# Heading 1\nBody text";
        let (fm, body) = parse_frontmatter(text).unwrap();
        assert!(fm.is_some());
        let fm = fm.unwrap();
        assert_eq!(fm.title.as_deref(), Some("My Document"));
        assert_eq!(fm.tags, vec!["rust".to_string(), "desktop".to_string()]);
        assert_eq!(body, "# Heading 1\nBody text");
    }

    #[test]
    fn test_comma_separated_tags_frontmatter() {
        let text = "---\ntitle: Quick Note\ntags: rust, notes, test\n---\nBody";
        let (fm, body) = parse_frontmatter(text).unwrap();
        assert!(fm.is_some());
        let fm = fm.unwrap();
        assert_eq!(fm.tags, vec!["rust", "notes", "test"]);
        assert_eq!(body, "Body");
    }

    #[test]
    fn test_malformed_yaml_frontmatter() {
        let text = "---\n: [invalid yaml\n---\nBody";
        let result = parse_frontmatter(text);
        assert!(result.is_err());
    }
}
