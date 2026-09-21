use dioxus::prelude::*;

use crate::icons::{IconClose, IconPlus, IconRough};
use crate::state::AppState;

/// Lightweight modal for rapid, frictionless thought capture directly to the vault Inbox.
#[component]
pub fn QuickCaptureModal(state: Signal<AppState>) -> Element {
    let app_state = state.read();
    if !app_state.show_quick_capture {
        return rsx! {};
    }

    let title_val = app_state.quick_capture_title.clone();
    let body_val = app_state.quick_capture_body.clone();
    let tags_val = app_state.quick_capture_tags.clone();

    rsx! {
        div {
            class: "modal-overlay",
            onclick: move |_| {
                state.write().close_quick_capture();
            },
            div {
                class: "modal-dialog",
                style: "max-width: 520px; width: 100%; box-shadow: var(--shadow-lg);",
                onclick: move |e| e.stop_propagation(),

                // Header
                div {
                    style: "display: flex; align-items: center; justify-content: space-between; margin-bottom: 12px;",
                    div { style: "display: flex; align-items: center; gap: 8px;",
                        IconRough { size: 16 }
                        h3 {
                            class: "modal-title",
                            style: "margin: 0; font-size: 14px; font-weight: 600;",
                            "Quick Capture"
                        }
                    }
                    button {
                        class: "btn-icon",
                        title: "Close (Esc)",
                        onclick: move |_| {
                            state.write().close_quick_capture();
                        },
                        IconClose { size: 14 }
                    }
                }

                // Subtitle guide
                p {
                    style: "font-size: 11px; color: var(--text-muted); margin: 0 0 12px 0;",
                    "Capture fleeting thoughts, ideas, or observations directly to your vault Inbox without interruption."
                }

                // Title Input
                div { style: "margin-bottom: 10px;",
                    label { style: "display: block; font-size: 11px; font-weight: 500; color: var(--text-secondary); margin-bottom: 4px;", "Title (Optional)" }
                    input {
                        class: "modal-input",
                        r#type: "text",
                        placeholder: "Quick Thought (auto-timestamped if blank)",
                        value: "{title_val}",
                        autofocus: true,
                        oninput: move |e| state.write().quick_capture_title = e.value(),
                        onkeydown: move |e: KeyboardEvent| {
                            if e.key() == Key::Enter && !e.modifiers().shift() {
                                let mut s = state.write();
                                let _ = s.execute_quick_capture(false);
                            }
                        }
                    }
                }

                // Body Textarea
                div { style: "margin-bottom: 10px;",
                    label { style: "display: block; font-size: 11px; font-weight: 500; color: var(--text-secondary); margin-bottom: 4px;", "Raw Thought / Observation" }
                    textarea {
                        class: "modal-input",
                        style: "min-height: 110px; resize: vertical; font-family: var(--font-editor); font-size: 12px; line-height: 1.5;",
                        placeholder: "Type observation or rough note here... (Ctrl+Enter to save)",
                        value: "{body_val}",
                        oninput: move |e| state.write().quick_capture_body = e.value(),
                        onkeydown: move |e: KeyboardEvent| {
                            if e.key() == Key::Enter && (e.modifiers().ctrl() || e.modifiers().meta()) {
                                let mut s = state.write();
                                let _ = s.execute_quick_capture(false);
                            }
                        }
                    }
                }

                // Tags Input
                div { style: "margin-bottom: 16px;",
                    label { style: "display: block; font-size: 11px; font-weight: 500; color: var(--text-secondary); margin-bottom: 4px;", "Tags (Optional, comma-separated)" }
                    input {
                        class: "modal-input",
                        r#type: "text",
                        placeholder: "e.g. spark, architecture, research",
                        value: "{tags_val}",
                        oninput: move |e| state.write().quick_capture_tags = e.value(),
                    }
                }

                // Actions
                div { class: "modal-actions", style: "display: flex; justify-content: space-between; align-items: center;",
                    span { style: "font-size: 11px; color: var(--text-muted);",
                        "Saves as "
                        code { style: "background: var(--bg-surface-elevated); border: 1px solid var(--border); padding: 1px 4px; border-radius: 3px;", "type: rough" }
                    }
                    div { style: "display: flex; gap: 8px;",
                        button {
                            class: "btn-action",
                            onclick: move |_| {
                                state.write().close_quick_capture();
                            },
                            "Cancel"
                        }
                        button {
                            class: "btn-action",
                            title: "Save and open in editor",
                            onclick: move |_| {
                                let mut s = state.write();
                                let _ = s.execute_quick_capture(true);
                            },
                            "Capture & Open"
                        }
                        button {
                            class: "btn-action btn-primary",
                            title: "Save to Inbox (Enter)",
                            onclick: move |_| {
                                let mut s = state.write();
                                let _ = s.execute_quick_capture(false);
                            },
                            IconPlus { size: 12 }
                            span { "Capture to Inbox" }
                        }
                    }
                }
            }
        }
    }
}
