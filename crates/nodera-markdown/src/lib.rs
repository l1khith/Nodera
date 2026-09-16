//! Markdown processing and knowledge extraction crate for Nodera.

pub mod frontmatter;
pub mod links;
pub mod parser;
pub mod renderer;
pub mod tags;
pub mod task_parser;
pub mod wikilink;

pub use frontmatter::{parse_frontmatter, Frontmatter};
pub use links::{GraphData, GraphEdge, GraphFilterOptions, GraphNode, LinkGraph};
pub use parser::{parse_document, Heading, ParsedDocument};
pub use renderer::render_to_html;
pub use tags::extract_tags;
pub use task_parser::{extract_tasks, parse_task_line, toggle_task_at_line, ParsedTask};
pub use wikilink::{extract_wikilinks, rewrite_wikilinks, Wikilink};
