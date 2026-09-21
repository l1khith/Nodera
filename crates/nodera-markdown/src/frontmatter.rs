use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::debug;

use nodera_core::{ParseError, Result};

pub const NOTE_TYPE_ROUGH: &str = "rough";
pub const NOTE_TYPE_PERMANENT: &str = "permanent";
pub const NOTE_TYPE_SOURCE: &str = "source";
pub const NOTE_TYPE_INDEX: &str = "index";
pub const NOTE_TYPE_PROJECT: &str = "project";
pub const NOTE_TYPE_MEETING: &str = "meeting";
pub const NOTE_TYPE_DAILY: &str = "daily";

pub const SOURCE_TYPE_BOOK: &str = "book";
pub const SOURCE_TYPE_PAPER: &str = "paper";
pub const SOURCE_TYPE_ARTICLE: &str = "article";
pub const SOURCE_TYPE_VIDEO: &str = "video";
pub const SOURCE_TYPE_WEBPAGE: &str = "webpage";
pub const SOURCE_TYPE_PDF: &str = "pdf";

/// Parsed metadata from a YAML frontmatter block.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct Frontmatter {
    pub title: Option<String>,
    pub tags: Vec<String>,
    pub extra: HashMap<String, serde_yaml::Value>,
}

impl Frontmatter {
    /// Returns the note type specified in frontmatter (`type` property), if present.
    pub fn note_type(&self) -> Option<&str> {
        self.extra.get("type").and_then(|v| v.as_str())
    }

    /// Sets or updates the note type in frontmatter.
    pub fn set_note_type(&mut self, note_type: impl Into<String>) {
        self.extra.insert("type".to_string(), serde_yaml::Value::String(note_type.into()));
    }

    /// Returns the source type specified in frontmatter (`source_type` property), if present.
    pub fn source_type(&self) -> Option<&str> {
        self.extra.get("source_type").and_then(|v| v.as_str())
    }

    /// Sets or updates the source type in frontmatter.
    pub fn set_source_type(&mut self, source_type: impl Into<String>) {
        self.extra.insert("source_type".to_string(), serde_yaml::Value::String(source_type.into()));
    }

    pub fn is_rough(&self) -> bool {
        self.note_type().map(|t| t.eq_ignore_ascii_case(NOTE_TYPE_ROUGH)).unwrap_or(false)
    }

    pub fn is_permanent(&self) -> bool {
        self.note_type().map(|t| t.eq_ignore_ascii_case(NOTE_TYPE_PERMANENT)).unwrap_or(false)
    }

    pub fn is_source(&self) -> bool {
        self.note_type().map(|t| t.eq_ignore_ascii_case(NOTE_TYPE_SOURCE)).unwrap_or(false)
    }

    pub fn is_index(&self) -> bool {
        self.note_type().map(|t| t.eq_ignore_ascii_case(NOTE_TYPE_INDEX) || t.eq_ignore_ascii_case("moc")).unwrap_or(false)
    }

    pub fn is_reviewed(&self) -> bool {
        self.extra.contains_key("reviewed")
    }

    pub fn mark_reviewed(&mut self, date_str: &str) {
        self.extra.insert("reviewed".to_string(), serde_yaml::Value::String(date_str.to_string()));
    }
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

/// Generates standard Rough Note Markdown with YAML frontmatter.
pub fn rough_note_template(title: &str, body: &str, date_str: &str) -> String {
    let clean_body = if body.trim().is_empty() {
        ""
    } else {
        body.trim()
    };
    format!(
        "---\ntitle: \"{title}\"\ntype: rough\ncreated: \"{date_str}\"\ntags:\n  - rough\n  - inbox\n---\n# {title}\n\n{clean_body}\n"
    )
}

/// Generates standard Permanent Note Markdown with atomic concept structure.
pub fn permanent_note_template(title: &str, core_idea: &str, date_str: &str) -> String {
    let idea = if core_idea.trim().is_empty() {
        "State the single core concept or principle in your own words."
    } else {
        core_idea.trim()
    };
    format!(
        "---\ntitle: \"{title}\"\ntype: permanent\ncreated: \"{date_str}\"\ntags:\n  - permanent\n---\n# {title}\n\n## Core Idea\n{idea}\n\n## Context & Explanation\n\n## Connections & References\n- Links to sources: \n- Related permanent notes: \n"
    )
}

/// Generates standard Source Note Markdown for literature, papers, books, and articles.
pub fn source_note_template(
    title: &str,
    source_type: &str,
    author: &str,
    url: &str,
    date_str: &str,
) -> String {
    let s_type = if source_type.is_empty() {
        SOURCE_TYPE_BOOK
    } else {
        source_type
    };
    format!(
        "---\ntitle: \"{title}\"\ntype: source\nsource_type: {s_type}\nauthor: \"{author}\"\nurl: \"{url}\"\ncreated: \"{date_str}\"\ntags:\n  - source\n  - {s_type}\n---\n# {title}\n\n**Author**: {author}\n**Source**: {url}\n\n## Summary & Key Takeaways\n1. \n\n## Quotes & Highlights\n> \n\n## Synthesized Notes\n- [[Permanent Note]]\n"
    )
}

/// Generates standard Video Source Note Markdown with timestamped observation table.
pub fn video_source_template(title: &str, channel: &str, url: &str, date_str: &str) -> String {
    format!(
        "---\ntitle: \"{title}\"\ntype: source\nsource_type: video\nchannel: \"{channel}\"\nurl: \"{url}\"\ncreated: \"{date_str}\"\ntags:\n  - source\n  - video\n---\n# {title}\n\n**Channel / Creator**: {channel}\n**URL**: {url}\n\n## Overview & Context\n\n## Timestamps & Observations\n- [00:00] Introduction & Key Thesis\n- [05:00] Core Arguments\n- [15:00] Key Demonstration or Example\n- [25:00] Conclusion & Takeaways\n\n## Linked Concepts\n- \n"
    )
}

/// Generates standard Index / Map of Content (MOC) note.
pub fn index_note_template(title: &str, topic: &str) -> String {
    format!(
        "---\ntitle: \"{title}\"\ntype: index\ntopic: \"{topic}\"\ntags:\n  - index\n  - moc\n---\n# {title}\n\n> Topic hub and Map of Content for **{topic}**.\n\n## Core Permanent Notes\n- \n\n## Sources & Literature\n- \n\n## Inquiries & Future Questions\n- \n"
    )
}

/// Generates standard Project Note Markdown.
pub fn project_note_template(title: &str, date_str: &str) -> String {
    format!(
        "---\ntitle: \"{title}\"\ntype: project\nstatus: planning\ncreated: \"{date_str}\"\ntags:\n  - project\n---\n# {title}\n\n## Objectives & Success Criteria\n- [ ] \n\n## Milestones & Deliverables\n- [ ] Phase 1: \n\n## Related Notes & Research\n- \n"
    )
}

/// Generates standard Meeting Note Markdown.
pub fn meeting_note_template(title: &str, date_str: &str) -> String {
    format!(
        "---\ntitle: \"{title}\"\ntype: meeting\ndate: \"{date_str}\"\ntags:\n  - meeting\n---\n# {title}\n\n**Date**: {date_str}\n**Attendees**: \n\n## Agenda\n1. \n\n## Discussion & Notes\n\n## Action Items\n- [ ] \n"
    )
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

    #[test]
    fn test_note_type_helpers_and_templates() {
        // 1. Rough note template
        let rough = rough_note_template("Fleeting Thought", "Capture this idea quickly", "2026-09-20");
        let (fm, body) = parse_frontmatter(&rough).unwrap();
        let fm = fm.unwrap();
        assert_eq!(fm.note_type(), Some("rough"));
        assert!(fm.is_rough());
        assert!(!fm.is_permanent());
        assert!(fm.tags.contains(&"rough".to_string()));
        assert!(body.contains("Capture this idea quickly"));

        // 2. Safe Promotion to Permanent
        let mut promoted_fm = fm.clone();
        promoted_fm.set_note_type(NOTE_TYPE_PERMANENT);
        assert_eq!(promoted_fm.note_type(), Some("permanent"));
        assert!(promoted_fm.is_permanent());
        assert!(!promoted_fm.is_rough());

        let promoted_content = inject_or_update_frontmatter(&rough, &promoted_fm);
        let (reparsed_fm, reparsed_body) = parse_frontmatter(&promoted_content).unwrap();
        let reparsed_fm = reparsed_fm.unwrap();
        assert!(reparsed_fm.is_permanent());
        assert_eq!(reparsed_body, body); // body is preserved intact!

        // 3. Permanent note template
        let perm = permanent_note_template("Atomic Principle", "One clear thought per note.", "2026-09-20");
        let (fm_p, _) = parse_frontmatter(&perm).unwrap();
        let fm_p = fm_p.unwrap();
        assert!(fm_p.is_permanent());

        // 4. Source note template
        let src = source_note_template("Designing Data-Intensive Apps", SOURCE_TYPE_BOOK, "Martin Kleppmann", "https://dataintensive.net", "2026-09-20");
        let (fm_s, _) = parse_frontmatter(&src).unwrap();
        let fm_s = fm_s.unwrap();
        assert!(fm_s.is_source());
        assert_eq!(fm_s.source_type(), Some("book"));

        // 5. Video template
        let vid = video_source_template("Deep Dive into Rust Memory", "Jon Gjengset", "https://youtube.com/watch?v=123", "2026-09-20");
        let (fm_v, body_v) = parse_frontmatter(&vid).unwrap();
        let fm_v = fm_v.unwrap();
        assert!(fm_v.is_source());
        assert_eq!(fm_v.source_type(), Some("video"));
        assert!(body_v.contains("Timestamps & Observations"));

        // 6. Index / MOC template
        let idx = index_note_template("Distributed Systems MOC", "Distributed Systems");
        let (fm_i, _) = parse_frontmatter(&idx).unwrap();
        let fm_i = fm_i.unwrap();
        assert!(fm_i.is_index());

        // 7. Reviewed status
        let mut fm_review = fm_s.clone();
        assert!(!fm_review.is_reviewed());
        fm_review.mark_reviewed("2026-09-20");
        assert!(fm_review.is_reviewed());
    }
}

