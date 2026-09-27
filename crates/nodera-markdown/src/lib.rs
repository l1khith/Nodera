//! Markdown processing and knowledge extraction crate for Nodera.

pub mod frontmatter;
pub mod links;
pub mod parser;
pub mod renderer;
pub mod tags;
pub mod task_parser;
pub mod wikilink;

pub use frontmatter::{
    index_note_template, inject_or_update_frontmatter, meeting_note_template, parse_frontmatter,
    permanent_note_template, project_note_template, rough_note_template, serialize_frontmatter,
    source_note_template, video_source_template, Frontmatter, NOTE_TYPE_DAILY, NOTE_TYPE_INDEX,
    NOTE_TYPE_MEETING, NOTE_TYPE_PERMANENT, NOTE_TYPE_PROJECT, NOTE_TYPE_ROUGH, NOTE_TYPE_SOURCE,
    SOURCE_TYPE_ARTICLE, SOURCE_TYPE_BOOK, SOURCE_TYPE_PAPER, SOURCE_TYPE_PDF, SOURCE_TYPE_VIDEO,
    SOURCE_TYPE_WEBPAGE,
};
pub use links::{
    calculate_centrality, detect_communities, BrokenLinkGroup, BrokenLinkItem, GraphData,
    GraphEdge, GraphFilterOptions, GraphNode, LinkAuditReport, LinkGraph, TargetResolver,
};
pub use parser::{parse_document, Heading, ParsedDocument};
pub use renderer::{render_to_html, render_to_html_with_resolver};
pub use tags::extract_tags;
pub use task_parser::{
    extract_due_date, extract_tasks, parse_task_line, toggle_task_at_line, ParsedTask,
};
pub use wikilink::{extract_wikilinks, rewrite_wikilinks, Wikilink};
