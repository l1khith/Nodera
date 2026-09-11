use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

use nodera_core::Result;

use crate::frontmatter::{parse_frontmatter, Frontmatter};
use crate::tags::extract_tags;
use crate::task_parser::{extract_tasks, ParsedTask};
use crate::wikilink::{extract_wikilinks, Wikilink};

/// Represents a heading element in the Markdown document.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Heading {
    /// 1 to 6
    pub level: u8,
    pub text: String,
    pub line_number: usize,
}

/// Comprehensive structured model of a parsed Markdown note.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ParsedDocument {
    pub frontmatter: Option<Frontmatter>,
    pub title: Option<String>,
    pub headings: Vec<Heading>,
    pub wikilinks: Vec<Wikilink>,
    pub tags: Vec<String>,
    pub tasks: Vec<ParsedTask>,
    pub body: String,
}

/// Parses a raw Markdown note content into a structured `ParsedDocument`.
pub fn parse_document(raw_content: &str) -> Result<ParsedDocument> {
    let (frontmatter, body) = parse_frontmatter(raw_content)?;

    let wikilinks = extract_wikilinks(body);
    let tasks = extract_tasks(body);
    let inline_tags = extract_tags(body);

    let mut all_tags = BTreeSet::new();
    if let Some(fm) = &frontmatter {
        for t in &fm.tags {
            all_tags.insert(t.clone());
        }
    }
    for t in inline_tags {
        all_tags.insert(t);
    }

    let headings = extract_headings(body);

    // Title resolution: frontmatter title -> first H1 -> None
    let title = frontmatter
        .as_ref()
        .and_then(|fm| fm.title.clone())
        .or_else(|| {
            headings
                .iter()
                .find(|h| h.level == 1)
                .map(|h| h.text.clone())
        });

    Ok(ParsedDocument {
        frontmatter,
        title,
        headings,
        wikilinks,
        tags: all_tags.into_iter().collect(),
        tasks,
        body: body.to_string(),
    })
}

fn extract_headings(body: &str) -> Vec<Heading> {
    let mut headings = Vec::new();

    for (zero_idx, line) in body.lines().enumerate() {
        let trimmed = line.trim_start();
        if trimmed.starts_with('#') {
            let hash_count = trimmed.chars().take_while(|&c| c == '#').count();
            if hash_count <= 6 {
                let rest = &trimmed[hash_count..];
                if rest.starts_with(' ') {
                    headings.push(Heading {
                        level: hash_count as u8,
                        text: rest.trim().to_string(),
                        line_number: zero_idx + 1,
                    });
                }
            }
        }
    }

    headings
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_complete_document() {
        let doc = r#"---
title: System Architecture
tags: [rust, architecture]
---
# Overview
This is a note with #design tag and a link to [[Core Module|Core]].

## Subsystem
- [ ] Implement indexer
- [x] Complete parser
"#;

        let parsed = parse_document(doc).unwrap();
        assert_eq!(parsed.title.as_deref(), Some("System Architecture"));
        assert_eq!(parsed.headings.len(), 2);
        assert_eq!(parsed.headings[0].text, "Overview");
        assert_eq!(parsed.headings[0].level, 1);
        assert_eq!(parsed.headings[1].text, "Subsystem");
        assert_eq!(parsed.headings[1].level, 2);

        assert_eq!(parsed.wikilinks.len(), 1);
        assert_eq!(parsed.wikilinks[0].target, "Core Module");
        assert_eq!(parsed.wikilinks[0].label(), "Core");

        assert_eq!(parsed.tasks.len(), 2);
        assert!(!parsed.tasks[0].checked);
        assert!(parsed.tasks[1].checked);

        assert_eq!(parsed.tags, vec!["architecture", "design", "rust"]);
    }
}
