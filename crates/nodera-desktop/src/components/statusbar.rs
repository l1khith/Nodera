use dioxus::prelude::*;

use crate::state::{ActiveView, AppState};

#[component]
pub fn StatusBar(mut state: Signal<AppState>) -> Element {
    let app_state = state.read();
    let status_message = app_state.status_message.clone();

    let notes_count = app_state
        .entries
        .iter()
        .filter(|e| matches!(e, nodera_core::VaultEntry::Note(_)))
        .count();

    let bib_count = app_state.bib_library.entries.len();

    // Query graph telemetry metrics
    let graph_data = app_state.get_full_graph_data();
    let concepts_count = graph_data
        .nodes
        .iter()
        .filter(|n| {
            n.id.starts_with("concept:")
                || n.id.starts_with("sym:")
                || n.is_tag
                || n.id.starts_with('#')
        })
        .count();
    let connections_count = graph_data.edges.len();

    let view_label = match app_state.active_view {
        ActiveView::Editor => "DOCUMENT",
        ActiveView::SplitKnowledge => "SPLIT KNOWLEDGE",
        ActiveView::Graph => "KNOWLEDGE GRAPH",
        ActiveView::Today => "TODAY",
        ActiveView::Tasks => "TASKS",
        ActiveView::ReviewQueue => "REVIEW QUEUE",
        ActiveView::Library => "LIBRARY",
    };

    rsx! {
        footer { class: "telemetry-statusbar",
            // Left Group: Engines & Indexing
            div { class: "telemetry-group",
                span { class: "telemetry-dot ready" }
                span { "{status_message}" }
                span { "·" }
                span { "Tantivy: Ready" }
                span { "·" }
                if let Some(prog) = &app_state.indexing_progress {
                    if !prog.is_finished() {
                        span {
                            style: "color: var(--accent);",
                            "{prog.phase} ({prog.percentage()}%)"
                        }
                    } else {
                        span { "Indexing: Idle" }
                    }
                } else {
                    span { "Indexing: Idle" }
                }
            }

            // Center Group: Working Context & Word Count
            div { style: "display: flex; align-items: center; gap: 8px; font-family: var(--font-mono); font-size: 11px;",
                if let Some(active) = &app_state.active_note {
                    {
                        let words = active.content.split_whitespace().count();
                        let path_display = active.relative_path.display().to_string();
                        rsx! {
                            span { style: "color: var(--text-secondary);", "{path_display}" }
                            span { style: "color: var(--text-muted);", "·" }
                            span { style: "color: var(--text-muted);", "{words} words" }
                        }
                    }
                } else {
                    span { style: "color: var(--accent); font-weight: 600; letter-spacing: 0.5px;", "{view_label}" }
                }
            }

            // Right Group: Knowledge Telemetry
            div { class: "telemetry-group",
                span {
                    title: "Total indexed knowledge metrics",
                    "{notes_count} docs · {concepts_count} concepts · {connections_count} links"
                }
                if bib_count > 0 {
                    span { "·" }
                    span {
                        style: "color: var(--text-secondary); cursor: pointer;",
                        title: "Open Citation Picker (Ctrl+Shift+C)",
                        onclick: move |_| {
                            state.write().show_citation_picker_modal = true;
                        },
                        "{bib_count} citations"
                    }
                }
                span { "·" }
                span { "UTF-8" }
            }
        }
    }
}

