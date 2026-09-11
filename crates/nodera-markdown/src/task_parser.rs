use serde::{Deserialize, Serialize};

use nodera_core::{ParseError, Result};

/// A parsed Markdown task item from document source text.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ParsedTask {
    /// 1-based line number in source text.
    pub line_number: usize,
    /// Checked state (`true` if `[x]` or `[X]`, `false` if `[ ]`).
    pub checked: bool,
    /// Task description text after checkbox.
    pub text: String,
    /// Complete original line.
    pub raw_line: String,
}

/// Extracts all task checkbox items from Markdown text.
pub fn extract_tasks(content: &str) -> Vec<ParsedTask> {
    let mut tasks = Vec::new();

    for (zero_idx, line) in content.lines().enumerate() {
        if let Some((checked, text)) = parse_task_line(line) {
            tasks.push(ParsedTask {
                line_number: zero_idx + 1,
                checked,
                text,
                raw_line: line.to_string(),
            });
        }
    }

    tasks
}

/// Inspects a line and extracts task checked status and text if it is a task line.
pub fn parse_task_line(line: &str) -> Option<(bool, String)> {
    let trimmed = line.trim_start();
    let marker = if let Some(rest) = trimmed.strip_prefix("- ") {
        rest
    } else if let Some(rest) = trimmed.strip_prefix("* ") {
        rest
    } else {
        return None;
    };

    if let Some(rest) = marker.strip_prefix("[ ] ") {
        Some((false, rest.trim().to_string()))
    } else {
        marker
            .strip_prefix("[x] ")
            .or_else(|| marker.strip_prefix("[X] "))
            .map(|rest| (true, rest.trim().to_string()))
    }
}

/// Toggles the checkbox state of a task at a specific 1-based line number in source Markdown.
/// Preserves indentation and all surrounding lines.
pub fn toggle_task_at_line(content: &str, line_number: usize) -> Result<String> {
    if line_number == 0 {
        return Err(ParseError::InvalidTask {
            line: 0,
            reason: "Line number must be >= 1".to_string(),
        }
        .into());
    }

    let mut lines: Vec<&str> = content.lines().collect();
    if line_number > lines.len() {
        return Err(ParseError::InvalidTask {
            line: line_number,
            reason: format!(
                "Line number {} exceeds total line count {}",
                line_number,
                lines.len()
            ),
        }
        .into());
    }

    let target_idx = line_number - 1;
    let target_line = lines[target_idx];

    let toggled_line = toggle_single_line(target_line).ok_or_else(|| ParseError::InvalidTask {
        line: line_number,
        reason: format!("Line '{target_line}' is not a valid Markdown task"),
    })?;

    // Reconstruct content preserving newline endings
    let has_trailing_newline = content.ends_with('\n');
    lines[target_idx] = &toggled_line;

    let mut result = lines.join("\n");
    if has_trailing_newline {
        result.push('\n');
    }

    Ok(result)
}

fn toggle_single_line(line: &str) -> Option<String> {
    // Find the marker pattern: "- [ ]", "- [x]", "- [X]", "* [ ]", "* [x]", "* [X]"
    if let Some(pos) = line.find("[ ]") {
        let mut new_line = String::with_capacity(line.len());
        new_line.push_str(&line[..pos]);
        new_line.push_str("[x]");
        new_line.push_str(&line[pos + 3..]);
        Some(new_line)
    } else if let Some(pos) = line.find("[x]").or_else(|| line.find("[X]")) {
        let mut new_line = String::with_capacity(line.len());
        new_line.push_str(&line[..pos]);
        new_line.push_str("[ ]");
        new_line.push_str(&line[pos + 3..]);
        Some(new_line)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_tasks() {
        let doc = r#"
# Project Tasks
- [ ] Task 1
- [x] Task 2 completed
* [ ] Task 3 with asterisk
Some normal paragraph.
    - [ ] Indented subtask
"#;
        let tasks = extract_tasks(doc);
        assert_eq!(tasks.len(), 4);
        assert_eq!(tasks[0].line_number, 3);
        assert!(!tasks[0].checked);
        assert_eq!(tasks[0].text, "Task 1");

        assert_eq!(tasks[1].line_number, 4);
        assert!(tasks[1].checked);
        assert_eq!(tasks[1].text, "Task 2 completed");

        assert_eq!(tasks[2].line_number, 5);
        assert!(!tasks[2].checked);
        assert_eq!(tasks[2].text, "Task 3 with asterisk");

        assert_eq!(tasks[3].line_number, 7);
        assert!(!tasks[3].checked);
        assert_eq!(tasks[3].text, "Indented subtask");
    }

    #[test]
    fn test_toggle_task_at_line() {
        let doc = "- [ ] Open task\n- [x] Completed task\n";
        // Toggle line 1 from unchecked to checked
        let toggled1 = toggle_task_at_line(doc, 1).unwrap();
        assert_eq!(toggled1, "- [x] Open task\n- [x] Completed task\n");

        // Toggle line 2 from checked to unchecked
        let toggled2 = toggle_task_at_line(&toggled1, 2).unwrap();
        assert_eq!(toggled2, "- [x] Open task\n- [ ] Completed task\n");
    }

    #[test]
    fn test_toggle_invalid_line_fails() {
        let doc = "# Heading\nNormal text\n";
        assert!(toggle_task_at_line(doc, 1).is_err());
        assert!(toggle_task_at_line(doc, 99).is_err());
    }
}
