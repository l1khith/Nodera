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

/// Serializes a `Frontmatter` struct back into a clean YAML string enclosed in `---` delimiters.
pub fn serialize_frontmatter(fm: &Frontmatter) -> String {
    let mut map = serde_yaml::Mapping::new();
    if let Some(ref title) = fm.title {
        map.insert(
            serde_yaml::Value::String("title".to_string()),
            serde_yaml::Value::String(title.clone()),
        );
    }
    if !fm.tags.is_empty() {
        let tag_seq: Vec<serde_yaml::Value> = fm
            .tags
            .iter()
            .map(|t| serde_yaml::Value::String(t.clone()))
            .collect();
        map.insert(
            serde_yaml::Value::String("tags".to_string()),
            serde_yaml::Value::Sequence(tag_seq),
        );
    }
    for (k, v) in &fm.extra {
        map.insert(serde_yaml::Value::String(k.clone()), v.clone());
    }

    if map.is_empty() {
        return String::new();
    }

    let yaml_content = serde_yaml::to_string(&map).unwrap_or_default();
    format!("---\n{}---\n", yaml_content)
}

/// Updates or inserts a YAML frontmatter block into a Markdown document, preserving the Markdown body.
pub fn inject_or_update_frontmatter(content: &str, fm: &Frontmatter) -> String {
    let (_existing_fm, body) = parse_frontmatter(content).unwrap_or((None, content));
    let serialized = serialize_frontmatter(fm);
    if serialized.is_empty() {
        body.to_string()
    } else {
        format!("{}{}", serialized, body.trim_start())
    }
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

    #[test]
    fn test_serialize_and_inject_frontmatter() {
        let mut fm = Frontmatter {
            title: Some("Serialized Note".to_string()),
            tags: vec!["alpha".to_string(), "beta".to_string()],
            extra: HashMap::new(),
        };
        fm.extra.insert(
            "status".to_string(),
            serde_yaml::Value::String("active".to_string()),
        );

        let initial_doc = "# Note Content\nHere is some regular markdown.";
        let injected = inject_or_update_frontmatter(initial_doc, &fm);
        assert!(injected.starts_with("---\n"));
        assert!(injected.contains("title: Serialized Note"));
        assert!(injected.contains("status: active"));
        assert!(injected.contains("# Note Content\nHere is some regular markdown."));

        let (parsed_fm, body) = parse_frontmatter(&injected).unwrap();
        assert_eq!(parsed_fm.unwrap().title.as_deref(), Some("Serialized Note"));
        assert_eq!(
            body.trim(),
            "# Note Content\nHere is some regular markdown."
        );
    }
}
