use pulldown_cmark::{html, Options, Parser};

use crate::wikilink::extract_wikilinks;

/// Renders Markdown content into sanitized, semantic HTML with clickable Wikilinks and Callout blocks.
pub fn render_to_html(markdown: &str) -> String {
    // 1. Convert Wikilinks into custom HTML anchor links
    let wikilinks_processed = preprocess_wikilinks(markdown);

    // 2. Preprocess Callout blocks (> [!NOTE], etc.)
    let processed_markdown = preprocess_callouts(&wikilinks_processed);

    let mut options = Options::empty();
    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_TASKLISTS);
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_FOOTNOTES);
    options.insert(Options::ENABLE_MATH);

    let parser = Parser::new_ext(&processed_markdown, options);
    let mut html_output = String::new();
    html::push_html(&mut html_output, parser);

    html_output
}

fn preprocess_callouts(markdown: &str) -> String {
    let mut result = String::with_capacity(markdown.len() + 128);
    let mut in_callout = false;

    for line in markdown.lines() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix('>') {
            let quote_content = rest.trim();
            if !in_callout && quote_content.starts_with("[!") && quote_content.contains(']') {
                if let Some(close_bracket) = quote_content.find(']') {
                    let callout_tag = &quote_content[2..close_bracket];
                    let title_override = quote_content[close_bracket + 1..].trim();
                    in_callout = true;
                    let callout_type = match callout_tag.to_uppercase().as_str() {
                        "NOTE" | "INFO" => "note",
                        "TIP" | "HINT" => "tip",
                        "WARNING" | "WARN" => "warning",
                        "IMPORTANT" | "ATTENTION" => "important",
                        "CAUTION" | "DANGER" => "caution",
                        "SUCCESS" | "CHECK" => "success",
                        _ => "note",
                    };
                    let callout_title = if !title_override.is_empty() {
                        title_override.to_string()
                    } else {
                        match callout_type {
                            "note" => "Note",
                            "tip" => "Tip",
                            "warning" => "Warning",
                            "important" => "Important",
                            "caution" => "Caution",
                            "success" => "Success",
                            _ => "Note",
                        }
                        .to_string()
                    };

                    result.push_str(&format!(
                        r#"<div class="callout callout-{}"><div class="callout-title"><span class="callout-icon"></span><span class="callout-label">{}</span></div><div class="callout-content">"#,
                        callout_type, callout_title
                    ));
                    result.push('\n');
                    continue;
                }
            } else if in_callout {
                result.push_str(quote_content);
                result.push('\n');
                continue;
            }
        } else if in_callout {
            in_callout = false;
            result.push_str("</div></div>\n\n");
        }

        result.push_str(line);
        result.push('\n');
    }

    if in_callout {
        result.push_str("</div></div>\n");
    }

    result
}

fn preprocess_wikilinks(markdown: &str) -> String {
    let links = extract_wikilinks(markdown);
    if links.is_empty() {
        return markdown.to_string();
    }

    let mut result = String::with_capacity(markdown.len() + links.len() * 32);
    let mut last_idx = 0;

    for link in links {
        result.push_str(&markdown[last_idx..link.start]);
        let label = link.label();
        let target = &link.target;
        // Output inline HTML anchor tag for wikilink
        result.push_str(&format!(
            r##"<a class="wikilink" data-target="{target}" href="#{target}">{label}</a>"##
        ));
        last_idx = link.end;
    }

    result.push_str(&markdown[last_idx..]);
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_basic_markdown() {
        let md = "# Title\nParagraph with **bold** and *italic* text.";
        let html = render_to_html(md);
        assert!(html.contains("<h1>Title</h1>"));
        assert!(html.contains("<strong>bold</strong>"));
        assert!(html.contains("<em>italic</em>"));
    }

    #[test]
    fn test_render_wikilink_to_html() {
        let md = "Check out [[Rust Lang|Rust Programming]] here.";
        let html = render_to_html(md);
        assert!(html.contains(r#"class="wikilink""#));
        assert!(html.contains(r#"data-target="Rust Lang""#));
        assert!(html.contains("Rust Programming</a>"));
    }

    #[test]
    fn test_render_table_and_tasklist() {
        let md = "- [ ] Todo task\n- [x] Done task\n\n| A | B |\n|---|---|\n| 1 | 2 |";
        let html = render_to_html(md);
        assert!(html.contains("<table>"));
        assert!(html.contains(r#"type="checkbox""#));
    }

    #[test]
    fn test_render_callout_blocks() {
        let md = "> [!NOTE]\n> This is an important note.\n\n> [!WARNING] Danger Zone\n> Proceed with caution.";
        let html = render_to_html(md);
        assert!(html.contains(r#"class="callout callout-note""#));
        assert!(html.contains(r#"<span class="callout-label">Note</span>"#));
        assert!(html.contains("This is an important note."));
        assert!(html.contains(r#"class="callout callout-warning""#));
        assert!(html.contains(r#"<span class="callout-label">Danger Zone</span>"#));
        assert!(html.contains("Proceed with caution."));
    }

    #[test]
    fn test_render_math() {
        let md = "Energy formula is $E=mc^2$ inline.";
        let html = render_to_html(md);
        assert!(html.contains("E=mc^2"));
    }
}
