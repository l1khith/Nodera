use dioxus::prelude::*;

use crate::components::editor::Editor;
use crate::components::graph_view::LocalGraphView;
use crate::icons::*;
use crate::state::{ActiveView, AppState};

/// Split Knowledge Workspace: Document surface on the left, interactive Local Knowledge Graph on the right.
/// Fulfills the core "Document -> Concept -> Relationship -> Knowledge Graph" workflow.
#[component]
pub fn SplitKnowledgeWorkspace(mut state: Signal<AppState>) -> Element {
    let app_state = state.read();
    let note_title = app_state
        .active_note
        .as_ref()
        .map(|n| n.title.clone())
        .unwrap_or_else(|| "Untitled".to_string());

    rsx! {
        div {
            class: "split-knowledge-container",
            // Left Pane: Document Workspace
            div {
                class: "split-knowledge-doc",
                Editor { state }
            }

            // Right Pane: Knowledge Context Surface
            div {
                class: "split-knowledge-graph",
                // Knowledge Header
                div {
                    style: "height: 38px; border-bottom: 1px solid var(--border); background-color: var(--bg-surface-elevated); display: flex; align-items: center; justify-content: space-between; padding: 0 var(--space-3); flex-shrink: 0;",
                    div {
                        style: "display: flex; align-items: center; gap: var(--space-2); font-size: 11px; font-weight: 600; text-transform: uppercase; letter-spacing: 0.05em; color: var(--accent);",
                        IconGraph { size: 14 }
                        span { "Knowledge Context" }
                        span {
                            style: "color: var(--text-muted); font-weight: 400; text-transform: none; max-width: 140px; overflow: hidden; text-overflow: ellipsis; white-space: nowrap;",
                            "· {note_title}"
                        }
                    }
                    div {
                        style: "display: flex; align-items: center; gap: var(--space-1);",
                        button {
                            class: "btn-icon",
                            title: "Open full graph workspace",
                            onclick: move |_| {
                                state.write().active_view = ActiveView::Graph;
                            },
                            IconMaximize { size: 13 }
                        }
                        button {
                            class: "btn-icon",
                            title: "Close split knowledge view (return to Document)",
                            onclick: move |_| {
                                state.write().active_view = ActiveView::Editor;
                            },
                            IconClose { size: 13 }
                        }
                    }
                }

                // Interactive Local Graph Surface
                div {
                    style: "flex: 1; position: relative; overflow: hidden; display: flex;",
                    LocalGraphView { state }
                }
            }
        }
    }
}
