use dioxus::prelude::*;

use crate::icons::{IconClose, IconCornerDownLeft, IconSearch};
use crate::state::AppState;
use crate::strings::{empty_states, placeholders, tooltips};

#[component]
pub fn CommandPalette(state: Signal<AppState>) -> Element {
    let app_state = state.read();
    if !app_state.show_command_palette {
        return rsx! {};
    }

    let items = app_state.get_command_palette_items();
    let query = app_state.command_palette_query.clone();

    rsx! {
        div {
            class: "modal-overlay",
            onclick: move |_| {
                let mut s = state.write();
                s.show_command_palette = false;
            },
            div {
                class: "modal-dialog",
                style: "width: 540px; padding: 0; overflow: hidden; border-radius: 8px;",
                onclick: move |evt| {
                    // Prevent closing when clicking inside dialog
                    evt.stop_propagation();
                },

                // Search Header
                div {
                    style: "display: flex; align-items: center; padding: 12px 16px; border-bottom: 1px solid var(--border); gap: 10px; background-color: var(--bg-surface-elevated);",
                    IconSearch { size: 16, class: "icon text-muted" }
                    input {
                        style: "flex: 1; border: none; background: transparent; font-size: 14px; outline: none; color: var(--text-primary);",
                        placeholder: placeholders::SEARCH_PALETTE,
                        value: "{query}",
                        autofocus: true,
                        oninput: move |evt| {
                            let mut s = state.write();
                            s.command_palette_query = evt.value();
                        },
                        onkeydown: move |evt: KeyboardEvent| {
                            if evt.key() == Key::Escape {
                                let mut s = state.write();
                                s.show_command_palette = false;
                            }
                        }
                    }
                    button {
                        class: "btn-icon",
                        title: tooltips::CLOSE_ESC,
                        style: "width: 24px; height: 24px;",
                        onclick: move |_| {
                            let mut s = state.write();
                            s.show_command_palette = false;
                        },
                        IconClose { size: 12 }
                    }
                }

                // Results list
                div {
                    style: "max-height: 360px; overflow-y: auto; padding: 6px;",
                    if items.is_empty() {
                        div {
                            style: "padding: 24px; text-align: center; color: var(--text-muted); font-size: 12px;",
                            "{empty_states::NO_PALETTE_MATCHES}"
                        }
                    } else {
                        for item in items {
                            {
                                let action = item.action.clone();
                                rsx! {
                                    div {
                                        key: "{item.title}",
                                        style: "display: flex; align-items: center; justify-content: space-between; padding: 8px 12px; border-radius: 4px; cursor: pointer; transition: background-color 0.1s;",
                                        class: "command-palette-row",
                                        onclick: move |_| {
                                            let mut s = state.write();
                                            let _ = s.execute_palette_action(action.clone());
                                        },
                                        div { style: "display: flex; flex-direction: column; gap: 2px;",
                                            span { style: "font-size: 13px; font-weight: 500; color: var(--text-primary);", "{item.title}" }
                                            span { style: "font-size: 11px; color: var(--text-muted);", "{item.description}" }
                                        }
                                        span { style: "color: var(--text-muted);",
                                            IconCornerDownLeft { size: 12 }
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
