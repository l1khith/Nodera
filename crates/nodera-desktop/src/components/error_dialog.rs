use dioxus::prelude::*;

use crate::icons::{IconChevronDown, IconChevronRight, IconClose, IconWarning};
use crate::state::AppState;
use crate::strings::{actions, tooltips};

/// Actionable error dialog component displaying clear errors with technical details drawer.
#[component]
pub fn ErrorDialog(state: Signal<AppState>) -> Element {
    let mut show_details = use_signal(|| false);
    let app_state = state.read();

    if !app_state.show_error_dialog {
        return rsx! {};
    }

    let title = app_state.error_dialog_title.clone();
    let message = app_state.error_dialog_message.clone();
    let details = app_state.error_dialog_details.clone();

    rsx! {
        div {
            class: "modal-backdrop",
            style: "position: fixed; top: 0; left: 0; width: 100vw; height: 100vh; background-color: rgba(0, 0, 0, 0.6); display: flex; align-items: center; justify-content: center; z-index: 2000; backdrop-filter: blur(2px);",
            onclick: move |_| {
                state.write().clear_error();
            },

            div {
                class: "modal-dialog error-dialog",
                style: "width: 480px; max-width: 90vw; background-color: var(--bg-surface); border: 1px solid var(--danger); border-radius: 10px; box-shadow: 0 16px 36px rgba(0, 0, 0, 0.4); display: flex; flex-direction: column; overflow: hidden;",
                onclick: move |evt| {
                    evt.stop_propagation();
                },

                // Header
                div {
                    style: "padding: 16px 20px; border-bottom: 1px solid var(--border); display: flex; align-items: center; gap: 10px; background-color: rgba(191, 97, 106, 0.1);",
                    div { style: "color: var(--danger); display: flex; align-items: center;",
                        IconWarning { size: 22 }
                    }
                    h3 {
                        style: "margin: 0; font-size: 16px; font-weight: 600; color: var(--danger); flex: 1;",
                        "{title}"
                    }
                    button {
                        class: "btn-icon",
                        title: tooltips::CLOSE_ESC,
                        onclick: move |_| {
                            state.write().clear_error();
                        },
                        IconClose { size: 14 }
                    }
                }

                // Body
                div {
                    style: "padding: 20px 24px; display: flex; flex-direction: column; gap: 14px; font-size: 13px; color: var(--text-primary); line-height: 1.5;",

                    p { style: "margin: 0;", "{message}" }

                    if let Some(detail_text) = &details {
                        div {
                            style: "display: flex; flex-direction: column; gap: 6px; margin-top: 4px;",

                            button {
                                style: "align-self: flex-start; display: flex; align-items: center; gap: 6px; font-size: 12px; color: var(--text-muted); cursor: pointer; padding: 2px 0; background: none; border: none;",
                                onclick: move |_| {
                                    let current = *show_details.read();
                                    show_details.set(!current);
                                },
                                if *show_details.read() {
                                    IconChevronDown { size: 12 }
                                    span { "{actions::HIDE_DETAILS}" }
                                } else {
                                    IconChevronRight { size: 12 }
                                    span { "{actions::VIEW_DETAILS}" }
                                }
                            }

                            if *show_details.read() {
                                pre {
                                    style: "margin: 0; padding: 10px 12px; border-radius: 6px; background-color: var(--bg-sidebar); border: 1px solid var(--border); font-family: var(--font-editor); font-size: 11px; color: var(--text-secondary); max-height: 160px; overflow-y: auto; white-space: pre-wrap; word-break: break-all;",
                                    "{detail_text}"
                                }
                            }
                        }
                    }
                }

                // Footer
                div {
                    style: "padding: 12px 20px; border-top: 1px solid var(--border); display: flex; justify-content: flex-end; gap: 8px; background-color: var(--bg-sidebar);",

                    button {
                        class: "btn-action btn-primary",
                        onclick: move |_| {
                            state.write().clear_error();
                        },
                        "{actions::DISMISS}"
                    }
                }
            }
        }
    }
}
