//! Markdown processing and knowledge extraction crate for Nodera.

pub mod frontmatter;
pub mod links;
pub mod parser;
pub mod renderer;
pub mod tags;
pub mod task_parser;
pub mod wikilink;

pub use frontmatter::{
    inject_or_update_frontmatter, parse_frontmatter, serialize_frontmatter, Frontmatter,
};
pub use links::{
    calculate_centrality, detect_communities, BrokenLinkGroup, BrokenLinkItem, GraphData,
    GraphEdge, GraphFilterOptions, GraphNode, LinkAuditReport, LinkGraph, TargetResolver,
};
pub use parser::{parse_document, Heading, ParsedDocument};
pub use renderer::{render_to_html, render_to_html_with_resolver};
pub use tags::extract_tags;
pub use task_parser::{extract_tasks, parse_task_line, toggle_task_at_line, ParsedTask};
pub use wikilink::{extract_wikilinks, rewrite_wikilinks, Wikilink};
