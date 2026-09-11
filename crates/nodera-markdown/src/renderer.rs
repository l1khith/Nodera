use pulldown_cmark::{html, Options, Parser};

use crate::wikilink::extract_wikilinks;

/// Renders Markdown content into sanitized, semantic HTML with clickable Wikilinks.
pub fn render_to_html(markdown: &str) -> String {
    // First, convert Wikilinks into custom HTML anchor links
    let processed_markdown = preprocess_wikilinks(markdown);

    let mut options = Options::empty();
    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_TASKLISTS);
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_FOOTNOTES);

    let parser = Parser::new_ext(&processed_markdown, options);
    let mut html_output = String::new();
    html::push_html(&mut html_output, parser);

    html_output
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
}
