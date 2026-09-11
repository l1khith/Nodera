use std::collections::HashSet;

/// Extracts valid inline tags from a Markdown text body.
///
/// Tags follow the format `#tag` or `#nested/tag`.
/// Excludes Markdown headings (e.g., `# Heading`) and hex color codes (e.g., `#fff`, `#1a2b3c`).
pub fn extract_tags(text: &str) -> Vec<String> {
    let mut tags = HashSet::new();

    for line in text.lines() {
        let trimmed_line = line.trim_start();
        // Skip headings
        if trimmed_line.starts_with('#') {
            let without_hash = trimmed_line.trim_start_matches('#');
            if without_hash.starts_with(' ') || without_hash.is_empty() {
                // This is a Markdown heading (# Title, ## Subtitle), not an inline tag
                continue;
            }
        }

        let mut chars = line.char_indices().peekable();
        while let Some((idx, ch)) = chars.next() {
            if ch == '#' {
                // Must be preceded by whitespace or start of line
                if idx > 0 {
                    let prev_char = line[..idx].chars().next_back().unwrap();
                    if !prev_char.is_whitespace() && prev_char != '(' && prev_char != '[' {
                        continue;
                    }
                }

                // Check next character
                if let Some(&(_, next_ch)) = chars.peek() {
                    if next_ch.is_whitespace() || next_ch == '#' || next_ch == '/' {
                        continue;
                    }
                } else {
                    continue;
                }

                // Read tag token
                let mut tag_token = String::new();
                while let Some(&(_, next_ch)) = chars.peek() {
                    if next_ch.is_alphanumeric()
                        || next_ch == '_'
                        || next_ch == '-'
                        || next_ch == '/'
                    {
                        tag_token.push(next_ch);
                        chars.next();
                    } else {
                        break;
                    }
                }

                // Tag cannot end with a slash or dash
                let cleaned = tag_token.trim_end_matches(['/', '-']);
                if cleaned.is_empty() {
                    continue;
                }

                // Discard hex colors (e.g., fff, 121316, 000000)
                if is_hex_color(cleaned) {
                    continue;
                }

                // Tag must contain at least one letter
                if !cleaned.chars().any(|c| c.is_alphabetic()) {
                    continue;
                }

                tags.insert(cleaned.to_string());
            }
        }
    }

    let mut result: Vec<String> = tags.into_iter().collect();
    result.sort();
    result
}

fn is_hex_color(token: &str) -> bool {
    let len = token.len();
    if (len == 3 || len == 4 || len == 6 || len == 8)
        && token.chars().all(|c| c.is_ascii_hexdigit())
    {
        return true;
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_inline_tags() {
        let text = "Here is a #rust tag and a #project/pdf nested tag.\nAnother tag #desktop_app.";
        let tags = extract_tags(text);
        assert_eq!(tags, vec!["desktop_app", "project/pdf", "rust"]);
    }

    #[test]
    fn test_headings_not_treated_as_tags() {
        let text = "# Main Heading\n## Subheading\nSome text with #real_tag inside.";
        let tags = extract_tags(text);
        assert_eq!(tags, vec!["real_tag"]);
    }

    #[test]
    fn test_hex_colors_ignored() {
        let text = "Color is #ffffff and background is #1e2025 or #fff. Real tag is #color_scheme.";
        let tags = extract_tags(text);
        assert_eq!(tags, vec!["color_scheme"]);
    }

    #[test]
    fn test_tags_in_parentheses() {
        let text = "Look at this (#important) note!";
        let tags = extract_tags(text);
        assert_eq!(tags, vec!["important"]);
    }
}
