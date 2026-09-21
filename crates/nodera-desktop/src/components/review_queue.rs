use dioxus::prelude::*;
use nodera_index::ReviewCategory;
use std::path::Path;

use crate::icons::{
    IconCheck, IconExternalLink, IconFile, IconNotes, IconPermanent, IconReviewQueue, IconRough,
    IconSource,
};
use crate::state::AppState;

/// Dedicated surface for knowledge hygiene and review triage.
#[component]
pub fn ReviewQueueView(mut state: Signal<AppState>) -> Element {
    let app_state = state.read();
    let filter = app_state.review_queue_filter;
    let items = app_state.get_review_queue_items();
    let stats = app_state.get_knowledge_stats();

    rsx! {
        div {
            class: "main-content-pane",
            style: "display: flex; flex-direction: column; height: 100%; overflow: hidden; background: var(--bg-app);",

            // Top Header
            div {
                style: "padding: 20px 24px 16px 24px; border-bottom: 1px solid var(--border); background: var(--bg-surface);",
                div { style: "display: flex; align-items: center; justify-content: space-between;",
                    div { style: "display: flex; align-items: center; gap: 10px;",
                        IconReviewQueue { size: 20 }
                        div {
                            h2 { style: "font-size: 16px; font-weight: 600; margin: 0;", "Knowledge Review Queue" }
                            p { style: "font-size: 11px; color: var(--text-muted); margin: 2px 0 0 0;",
                                "Triage unprocessed rough notes, synthesize sources, and connect isolated thoughts."
                            }
                        }
                    }
                    div { style: "display: flex; gap: 8px;",
                        button {
                            class: "btn-action",
                            title: "Quick capture a new thought (Ctrl+Shift+Q)",
                            onclick: move |_| {
                                state.write().open_quick_capture();
                            },
                            IconRough { size: 12 }
                            span { "Quick Capture" }
                        }
                    }
                }

                // Category Tabs
                div {
                    style: "display: flex; gap: 8px; margin-top: 16px;",
                    button {
                        class: if filter == ReviewCategory::RoughNote { "btn-action btn-primary" } else { "btn-action" },
                        onclick: move |_| state.write().review_queue_filter = ReviewCategory::RoughNote,
                        IconRough { size: 12 }
                        span { "Rough Notes" }
                        span { class: "kbd-badge", "{stats.rough_count}" }
                    }
                    button {
                        class: if filter == ReviewCategory::Unlinked { "btn-action btn-primary" } else { "btn-action" },
                        onclick: move |_| state.write().review_queue_filter = ReviewCategory::Unlinked,
                        IconNotes { size: 12 }
                        span { "Unlinked Notes" }
                        span { class: "kbd-badge", "{stats.unlinked_count}" }
                    }
                    button {
                        class: if filter == ReviewCategory::StaleSource { "btn-action btn-primary" } else { "btn-action" },
                        onclick: move |_| state.write().review_queue_filter = ReviewCategory::StaleSource,
                        IconSource { size: 12 }
                        span { "Stale Sources" }
                        span { class: "kbd-badge", "{stats.source_count}" }
                    }
                    button {
                        class: if filter == ReviewCategory::Orphan { "btn-action btn-primary" } else { "btn-action" },
                        onclick: move |_| state.write().review_queue_filter = ReviewCategory::Orphan,
                        IconFile { size: 12 }
                        span { "Orphan Notes" }
                        span { class: "kbd-badge", "{stats.orphan_count}" }
                    }
                }
            }

            // Main Queue List
            div {
                style: "flex: 1; overflow-y: auto; padding: 20px 24px; display: flex; flex-direction: column; gap: 10px;",
                if items.is_empty() {
                    div {
                        style: "text-align: center; padding: 60px 20px; color: var(--text-muted);",
                        IconCheck { size: 36, class: "opacity-40" }
                        h4 { style: "margin: 12px 0 4px 0; font-size: 14px; font-weight: 600; color: var(--text-secondary);", "Queue Clear" }
                        p { style: "font-size: 12px; margin: 0;", "No notes require attention under this category." }
                    }
                } else {
                    for item in items.iter() {
                        {
                            let item_path = item.path.clone();
                            let item_path_open = item.path.clone();
                            let item_path_promote = item.path.clone();
                            let item_path_review = item.path.clone();
                            let item_title = item.title.clone();
                            let item_reason = item.reason.clone();
                            let is_rough = item.category == ReviewCategory::RoughNote;

                            rsx! {
                                div {
                                    key: "review_item_{item_path}",
                                    style: "background: var(--bg-surface); border: 1px solid var(--border); border-radius: 6px; padding: 12px 16px; display: flex; align-items: center; justify-content: space-between; gap: 12px;",
                                    div { style: "flex: 1; min-width: 0;",
                                        div { style: "display: flex; align-items: center; gap: 8px;",
                                            span { style: "font-size: 13px; font-weight: 600; color: var(--text-primary); text-overflow: ellipsis; overflow: hidden; white-space: nowrap;",
                                                "{item_title}"
                                            }
                                            span {
                                                style: "font-size: 10px; padding: 1px 6px; border-radius: 4px; background: var(--bg-surface-elevated); color: var(--text-secondary); border: 1px solid var(--border);",
                                                "{item_reason}"
                                            }
                                        }
                                        div { style: "font-size: 11px; color: var(--text-muted); margin-top: 3px; font-family: var(--font-editor);",
                                            "{item_path}"
                                        }
                                    }

                                    // Action Buttons
                                    div { style: "display: flex; align-items: center; gap: 6px;",
                                        button {
                                            class: "btn-action",
                                            title: "Open note in editor",
                                            onclick: move |_| {
                                                let mut s = state.write();
                                                let _ = s.select_note(Path::new(&item_path_open));
                                                s.active_view = crate::state::ActiveView::Editor;
                                            },
                                            IconExternalLink { size: 12 }
                                            span { "Open" }
                                        }

                                        if is_rough {
                                            button {
                                                class: "btn-action btn-primary",
                                                title: "Promote note to Permanent (preserves links & path)",
                                                onclick: move |_| {
                                                    let mut s = state.write();
                                                    let _ = s.select_note(Path::new(&item_path_promote));
                                                    let _ = s.promote_active_note_to_permanent();
                                                },
                                                IconPermanent { size: 12 }
                                                span { "Promote" }
                                            }
                                        }

                                        button {
                                            class: "btn-action",
                                            title: "Mark as reviewed (removes from queue)",
                                            onclick: move |_| {
                                                let mut s = state.write();
                                                let _ = s.mark_note_reviewed(Path::new(&item_path_review));
                                            },
                                            IconCheck { size: 12 }
                                            span { "Reviewed" }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
