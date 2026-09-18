use pulldown_cmark::{html, Options, Parser};
use std::collections::HashMap;
use std::sync::LazyLock;
use syntect::highlighting::ThemeSet;
use syntect::parsing::SyntaxSet;

use crate::wikilink::extract_wikilinks;

static SYNTAX_SET: LazyLock<SyntaxSet> = LazyLock::new(SyntaxSet::load_defaults_newlines);
static THEME_SET: LazyLock<ThemeSet> = LazyLock::new(ThemeSet::load_defaults);

/// Renders Markdown content into sanitized, semantic HTML with clickable Wikilinks,
/// Callout blocks, syntax highlighted code blocks, and Mermaid diagram previews.
pub fn render_to_html(markdown: &str) -> String {
    render_to_html_with_resolver(markdown, &|_| None)
}

/// Renders Markdown content into HTML using a custom transclusion embed resolver.
pub fn render_to_html_with_resolver(
    markdown: &str,
    resolver: &dyn Fn(&str) -> Option<String>,
) -> String {
    render_to_html_with_resolver_depth(markdown, resolver, 0)
}

fn render_to_html_with_resolver_depth(
    markdown: &str,
    resolver: &dyn Fn(&str) -> Option<String>,
    depth: usize,
) -> String {
    // 1. Preprocess Transclusions (![[Note]])
    let transclusions_processed = preprocess_transclusions(markdown, resolver, depth);

    // 2. Convert Wikilinks into custom HTML anchor links
    let wikilinks_processed = preprocess_wikilinks(&transclusions_processed);

    // 3. Preprocess Citations ([@key] and @key)
    let citations_processed = preprocess_citations(&wikilinks_processed);

    // 4. Preprocess Callout blocks (> [!NOTE], etc.)
    let processed_markdown = preprocess_callouts(&citations_processed);

    let mut options = Options::empty();
    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_TASKLISTS);
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_FOOTNOTES);
    options.insert(Options::ENABLE_MATH);

    let parser = Parser::new_ext(&processed_markdown, options);
    let mut custom_events = Vec::new();
    let mut in_code_block = false;
    let mut code_lang = String::new();
    let mut code_buffer = String::new();

    for event in parser {
        match event {
            pulldown_cmark::Event::Start(pulldown_cmark::Tag::CodeBlock(kind)) => {
                in_code_block = true;
                code_buffer.clear();
                code_lang = match kind {
                    pulldown_cmark::CodeBlockKind::Fenced(lang) => lang.to_string(),
                    pulldown_cmark::CodeBlockKind::Indented => String::new(),
                };
            }
            pulldown_cmark::Event::End(pulldown_cmark::TagEnd::CodeBlock) => {
                in_code_block = false;
                let html_chunk = render_code_block(&code_buffer, &code_lang);
                custom_events.push(pulldown_cmark::Event::Html(html_chunk.into()));
            }
            pulldown_cmark::Event::Text(text) if in_code_block => {
                code_buffer.push_str(&text);
            }
            other => {
                if !in_code_block {
                    custom_events.push(other);
                }
            }
        }
    }

    let mut html_output = String::new();
    html::push_html(&mut html_output, custom_events.into_iter());

    html_output
}

fn render_code_block(code: &str, lang: &str) -> String {
    let trimmed_lang = lang.trim();
    if trimmed_lang.eq_ignore_ascii_case("mermaid") {
        return render_mermaid_diagram(code);
    }

    let syntax = if trimmed_lang.is_empty() {
        SYNTAX_SET.find_syntax_plain_text()
    } else {
        SYNTAX_SET
            .find_syntax_by_token(trimmed_lang)
            .or_else(|| SYNTAX_SET.find_syntax_by_extension(trimmed_lang))
            .unwrap_or_else(|| SYNTAX_SET.find_syntax_plain_text())
    };

    let theme = &THEME_SET.themes["base16-ocean.dark"];
    let highlighted_pre =
        syntect::html::highlighted_html_for_string(code, &SYNTAX_SET, syntax, theme)
            .unwrap_or_else(|_| format!("<pre><code>{}</code></pre>", html_escape(code)));

    let display_lang = if trimmed_lang.is_empty() {
        "code"
    } else {
        trimmed_lang
    };

    format!(
        r#"<div class="code-block-wrapper"><div class="code-block-header"><span class="code-block-lang">{}</span><button class="code-block-copy-btn" onclick="navigator.clipboard.writeText(this.closest('.code-block-wrapper').querySelector('pre').innerText)">Copy</button></div>{}</div>"#,
        display_lang, highlighted_pre
    )
}

fn render_mermaid_diagram(code: &str) -> String {
    let lines: Vec<&str> = code
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty())
        .collect();

    let mut direction = "TD".to_string();
    let mut edges = Vec::new();
    let mut nodes_order = Vec::new();
    let mut node_labels = HashMap::new();

    for (idx, line) in lines.iter().enumerate() {
        if idx == 0 && (line.starts_with("graph ") || line.starts_with("flowchart ")) {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() > 1 {
                direction = parts[1].to_uppercase();
            }
            continue;
        }

        if line.contains("-->") || line.contains("---") {
            let delim = if line.contains("-->") { "-->" } else { "---" };
            let parts: Vec<&str> = line.split(delim).collect();
            if parts.len() >= 2 {
                let left_raw = parts[0].trim();
                let right_raw = parts[1].trim();

                let (from_id, from_label) = parse_mermaid_node(left_raw);
                let (to_id, to_label) = parse_mermaid_node(right_raw);

                if !nodes_order.contains(&from_id) {
                    nodes_order.push(from_id.clone());
                }
                if !nodes_order.contains(&to_id) {
                    nodes_order.push(to_id.clone());
                }
                node_labels.insert(from_id.clone(), from_label);
                node_labels.insert(to_id.clone(), to_label);
                edges.push((from_id, to_id));
            }
        }
    }

    let is_lr = direction == "LR";
    let svg_preview = if nodes_order.len() >= 2 {
        render_mermaid_svg(&nodes_order, &node_labels, &edges, is_lr)
    } else {
        format!(
            r#"<pre class="mermaid-diagram-text">{}</pre>"#,
            html_escape(code)
        )
    };

    let escaped_raw = html_escape(code);
    format!(
        r#"<div class="mermaid-diagram" data-mermaid="{}"><div class="mermaid-header"><span class="mermaid-badge">MERMAID</span><span class="mermaid-type">{} Graph</span></div><div class="mermaid-content">{}</div><pre class="mermaid" style="display: none;">{}</pre></div>"#,
        escaped_raw, direction, svg_preview, escaped_raw
    )
}

fn parse_mermaid_node(raw: &str) -> (String, String) {
    let raw = raw.trim();
    if let Some(open) = raw.find('[') {
        if let Some(close) = raw.find(']') {
            let id = raw[..open].trim().to_string();
            let label = raw[open + 1..close].trim().to_string();
            return (id, label);
        }
    }
    if let Some(open) = raw.find('(') {
        if let Some(close) = raw.find(')') {
            let id = raw[..open].trim().to_string();
            let label = raw[open + 1..close].trim().to_string();
            return (id, label);
        }
    }
    (raw.to_string(), raw.to_string())
}

fn render_mermaid_svg(
    nodes_order: &[String],
    node_labels: &HashMap<String, String>,
    edges: &[(String, String)],
    is_lr: bool,
) -> String {
    let count = nodes_order.len();
    let mut positions = HashMap::new();

    let (width, height) = if is_lr {
        let w = 100 + count * 150;
        let h = 140;
        for (i, id) in nodes_order.iter().enumerate() {
            let x = 70 + i * 150;
            let y = if i % 2 == 1 { 90 } else { 50 };
            positions.insert(id.clone(), (x, y));
        }
        (w, h)
    } else {
        let w = 360;
        let h = 80 + count * 75;
        for (i, id) in nodes_order.iter().enumerate() {
            let x = if i % 2 == 1 { 205 } else { 155 };
            let y = 45 + i * 75;
            positions.insert(id.clone(), (x, y));
        }
        (w, h)
    };

    let mut svg = format!(
        r#"<svg class="mermaid-svg" viewBox="0 0 {width} {height}" style="max-width: 100%; height: auto;" xmlns="http://www.w3.org/2000/svg"><defs><marker id="mermaid-arrow" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M 0 0 L 10 5 L 0 10 z" fill="var(--accent, #5B6CFF)"/></marker></defs>"#
    );

    for (from, to) in edges {
        if let (Some(&(x1, y1)), Some(&(x2, y2))) = (positions.get(from), positions.get(to)) {
            svg.push_str(&format!(
                r#"<line x1="{x1}" y1="{y1}" x2="{x2}" y2="{y2}" stroke="var(--border-strong, #444A5B)" stroke-width="2" marker-end="url(#mermaid-arrow)" />"#
            ));
        }
    }

    for id in nodes_order {
        if let Some(&(x, y)) = positions.get(id) {
            let label = node_labels.get(id).unwrap_or(id);
            svg.push_str(&format!(
                r#"<rect x="{}" y="{}" width="110" height="32" rx="6" fill="var(--bg-surface-elevated, #1A1E29)" stroke="var(--accent, #5B6CFF)" stroke-width="1.5" /><text x="{}" y="{}" text-anchor="middle" fill="var(--text-primary, #F1F3F8)" font-size="11" font-family="sans-serif">{}</text>"#,
                x - 55,
                y - 16,
                x,
                y + 4,
                html_escape(label)
            ));
        }
    }

    svg.push_str("</svg>");
    svg
}

fn preprocess_transclusions(
    markdown: &str,
    resolver: &dyn Fn(&str) -> Option<String>,
    depth: usize,
) -> String {
    let mut result = String::with_capacity(markdown.len() + 128);
    let mut remaining = markdown;

    while let Some(start_idx) = remaining.find("![[") {
        result.push_str(&remaining[..start_idx]);
        let after_start = &remaining[start_idx + 3..];

        if let Some(end_idx) = after_start.find("]]") {
            let target = after_start[..end_idx].trim();
            if depth >= 2 {
                result.push_str(r#"<div class="note-embed note-embed-warning"><span class="embed-warning-text">[Transclusion recursion limit reached]</span></div>"#);
            } else if let Some(content) = resolver(target) {
                let sub_rendered =
                    render_to_html_with_resolver_depth(&content, resolver, depth + 1);
                result.push_str(&format!(
                    r#"<div class="note-embed" data-target="{}"><div class="note-embed-header"><span class="note-embed-icon">📄</span><span class="note-embed-title">{}</span></div><div class="note-embed-content">{}</div></div>"#,
                    html_escape(target),
                    html_escape(target),
                    sub_rendered
                ));
            } else {
                result.push_str(&format!(
                    r#"<div class="note-embed note-embed-missing" data-target="{}"><div class="note-embed-header"><span class="note-embed-icon">📄</span><span class="note-embed-title">{}</span></div><div class="note-embed-missing-text">Note '{}' not found</div></div>"#,
                    html_escape(target),
                    html_escape(target),
                    html_escape(target)
                ));
            }

            remaining = &after_start[end_idx + 2..];
        } else {
            result.push_str("![[");
            remaining = after_start;
        }
    }

    result.push_str(remaining);
    result
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
        result.push_str(&format!(
            r##"<a class="wikilink" data-target="{target}" href="#{target}">{label}</a>"##
        ));
        last_idx = link.end;
    }

    result.push_str(&markdown[last_idx..]);
    result
}

static CITATION_BRACKET_RE: LazyLock<regex::Regex> =
    LazyLock::new(|| regex::Regex::new(r"\[@([a-zA-Z0-9_\-:]+)(?:,\s*([^\]]+))?\]").unwrap());

static CITATION_INLINE_RE: LazyLock<regex::Regex> =
    LazyLock::new(|| regex::Regex::new(r"(^|\s)@([a-zA-Z0-9_\-:]+)").unwrap());

pub fn preprocess_citations(markdown: &str) -> String {
    if !markdown.contains('@') {
        return markdown.to_string();
    }

    let mut result = String::with_capacity(markdown.len() + 64);
    let parts: Vec<&str> = markdown.split("```").collect();

    for (i, part) in parts.iter().enumerate() {
        if i % 2 == 1 {
            // Inside fenced code block: do not modify
            result.push_str("```");
            result.push_str(part);
            result.push_str("```");
        } else {
            // Outside fenced code block - split by inline backticks
            let code_parts: Vec<&str> = part.split('`').collect();
            for (j, cpart) in code_parts.iter().enumerate() {
                if j % 2 == 1 {
                    // Inside inline code span: do not modify
                    result.push('`');
                    result.push_str(cpart);
                    result.push('`');
                } else {
                    // Normal text
                    let bracketed =
                        CITATION_BRACKET_RE.replace_all(cpart, |caps: &regex::Captures| {
                            let key = &caps[1];
                            let note = caps.get(2).map(|m| m.as_str());
                            let label = if let Some(n) = note {
                                format!("[@{key}, {n}]")
                            } else {
                                format!("[@{key}]")
                            };
                            format!(r#"<span class="nodera-citation" data-citekey="{key}">{label}</span>"#)
                        });

                    let inline = CITATION_INLINE_RE.replace_all(&bracketed, |caps: &regex::Captures| {
                        let prefix = &caps[1];
                        let key = &caps[2];
                        format!(r#"{prefix}<span class="nodera-citation" data-citekey="{key}">@{key}</span>"#)
                    });

                    result.push_str(&inline);
                }
            }
        }
    }

    result
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
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

    #[test]
    fn test_render_syntax_highlighted_code_block() {
        let md = "```rust\nfn main() {\n    println!(\"Hello World\");\n}\n```";
        let html = render_to_html(md);
        assert!(html.contains(r#"class="code-block-wrapper""#));
        assert!(html.contains(r#"class="code-block-lang">rust</span>"#));
        assert!(html.contains("code-block-copy-btn"));
        assert!(html.contains("color:#")); // Syntect inline color spans
        assert!(html.contains("println!"));
    }

    #[test]
    fn test_render_mermaid_diagram_block() {
        let md = "```mermaid\ngraph TD\n    A[Client] --> B[Server]\n    B --> C[Database]\n```";
        let html = render_to_html(md);
        assert!(html.contains(r#"class="mermaid-diagram""#));
        assert!(html.contains(r#"class="mermaid-badge">MERMAID</span>"#));
        assert!(html.contains("<svg class=\"mermaid-svg\""));
        assert!(html.contains("Client"));
        assert!(html.contains("Server"));
        assert!(html.contains("Database"));
    }

    #[test]
    fn test_render_transclusion_embed() {
        let resolver = |target: &str| -> Option<String> {
            if target == "Embedded Note" {
                Some("# Inside Note\nContent from embed.".to_string())
            } else {
                None
            }
        };

        let md = "Here is an embedded note:\n![[Embedded Note]]";
        let html = render_to_html_with_resolver(md, &resolver);
        assert!(html.contains(r#"class="note-embed""#));
        assert!(html.contains(r#"class="note-embed-title">Embedded Note</span>"#));
        assert!(html.contains("<h1>Inside Note</h1>"));
        assert!(html.contains("Content from embed."));
    }

    #[test]
    fn test_render_transclusion_missing() {
        let md = "![[Missing Target]]";
        let html = render_to_html(md);
        assert!(html.contains("note-embed note-embed-missing"));
        assert!(html.contains("Note 'Missing Target' not found"));
    }

    #[test]
    fn test_render_citations() {
        let md =
            "As demonstrated in [@vaswani2017attention, p. 5] and @rustbook, transformers work.";
        let html = render_to_html(md);
        assert!(html.contains(r#"<span class="nodera-citation" data-citekey="vaswani2017attention">[@vaswani2017attention, p. 5]</span>"#));
        assert!(html
            .contains(r#"<span class="nodera-citation" data-citekey="rustbook">@rustbook</span>"#));

        // Code block and inline code should NOT format as citation
        let md_code = "`@vaswani2017` is a code span.\n```\n@not_a_citation\n```";
        let html_code = render_to_html(md_code);
        assert!(!html_code.contains(r#"data-citekey="vaswani2017""#));
        assert!(!html_code.contains(r#"data-citekey="not_a_citation""#));
    }
}
