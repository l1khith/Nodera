use dioxus::prelude::*;

use crate::icons::{IconClose, IconFile, IconSearch, IconSliders};
use crate::state::{AppState, PaletteCategory};
use crate::strings::{empty_states, placeholders, tooltips};

#[component]
pub fn CommandPalette(mut state: Signal<AppState>) -> Element {
    let app_state = state.read();
    if !app_state.show_command_palette {
        return rsx! {};
    }

    let items = app_state.get_command_palette_items();
    let query = app_state.command_palette_query.clone();

    // Group items by category preserving global indices for keyboard selection
    let mut doc_items = Vec::new();
    let mut concept_items = Vec::new();
    let mut cmd_items = Vec::new();

    for (idx, item) in items.iter().enumerate() {
        match item.category {
            PaletteCategory::Document => doc_items.push((idx, item)),
            PaletteCategory::Concept => concept_items.push((idx, item)),
            PaletteCategory::Command => cmd_items.push((idx, item)),
        }
    }

    let mut selected_index = use_signal(|| 0usize);
    let total_items = items.len();

    rsx! {
        div {
            class: "modal-overlay",
            onclick: move |_| {
                state.write().show_command_palette = false;
            },
            div {
                class: "modal-dialog",
                style: "width: 580px; padding: 0; overflow: hidden; border-radius: var(--radius-lg); background-color: var(--bg-surface); border: 1px solid var(--border-strong); box-shadow: var(--shadow-lg);",
                onclick: move |evt| {
                    evt.stop_propagation();
                },

                // Search Input Header
                div {
                    style: "display: flex; align-items: center; padding: 12px 16px; border-bottom: 1px solid var(--border); gap: 10px; background-color: var(--bg-surface-elevated);",
                    IconSearch { size: 16, class: "icon text-muted" }
                    input {
                        style: "flex: 1; border: none; background: transparent; font-size: 14px; font-family: var(--font-ui); outline: none; color: var(--text-primary);",
                        placeholder: placeholders::SEARCH_PALETTE,
                        value: "{query}",
                        autofocus: true,
                        oninput: move |evt| {
                            selected_index.set(0);
                            let mut s = state.write();
                            s.command_palette_query = evt.value();
                        },
                        onkeydown: move |evt: KeyboardEvent| {
                            match evt.key() {
                                Key::Escape => {
                                    state.write().show_command_palette = false;
                                }
                                Key::ArrowDown => {
                                    if total_items > 0 {
                                        let cur = *selected_index.read();
                                        selected_index.set((cur + 1).min(total_items - 1));
                                    }
                                }
                                Key::ArrowUp => {
                                    let cur = *selected_index.read();
                                    selected_index.set(cur.saturating_sub(1));
                                }
                                Key::Tab => {
                                    if total_items > 0 {
                                        let cur = *selected_index.read();
                                        selected_index.set((cur + 1) % total_items);
                                    }
                                }
                                Key::Enter => {
                                    let cur_idx = *selected_index.read();
                                    if let Some(target) = items.get(cur_idx) {
                                        let action = target.action.clone();
                                        let _ = state.write().execute_palette_action(action);
                                    }
                                }
                                _ => {}
                            }
                        }
                    }
                    if total_items > 0 {
                        span {
                            style: "font-size: 10px; font-family: var(--font-mono); color: var(--text-muted); background: var(--bg-surface); border: 1px solid var(--border); padding: 2px 6px; border-radius: var(--radius-sm);",
                            "{total_items} matches"
                        }
                    }
                    button {
                        class: "btn-icon",
                        title: tooltips::CLOSE_ESC,
                        style: "width: 24px; height: 24px;",
                        onclick: move |_| {
                            state.write().show_command_palette = false;
                        },
                        IconClose { size: 12 }
                    }
                }

                // Categorized Results List
                div {
                    style: "max-height: 400px; overflow-y: auto; padding: 6px;",
                    if items.is_empty() {
                        div {
                            style: "padding: 32px 16px; text-align: center; color: var(--text-muted); font-size: 12px; display: flex; flex-direction: column; gap: 4px;",
                            span { style: "font-weight: 500; color: var(--text-secondary);", "No results found" }
                            span { style: "font-size: 11px; color: var(--text-disabled);", "{empty_states::NO_PALETTE_MATCHES}" }
                        }
                    } else {
                        // Section: Documents
                        if !doc_items.is_empty() {
                            div {
                                div {
                                    style: "font-size: 10px; font-weight: 700; text-transform: uppercase; letter-spacing: 0.6px; color: var(--text-muted); padding: 6px 12px 4px 12px; display: flex; align-items: center; justify-content: space-between;",
                                    span { "Documents" }
                                    span { style: "font-family: var(--font-mono); font-size: 9px;", "{doc_items.len()}" }
                                }
                                for (global_idx, item) in doc_items {
                                    {
                                        let is_sel = *selected_index.read() == global_idx;
                                        let act = item.action.clone();
                                        let title = item.title.clone();
                                        let desc = item.description.clone();
                                        let row_style = if is_sel {
                                            "display: flex; align-items: center; justify-content: space-between; padding: 7px 12px; border-radius: 4px; cursor: pointer; transition: all 0.08s ease; border-left: 2px solid var(--accent); background-color: var(--accent-focus);"
                                        } else {
                                            "display: flex; align-items: center; justify-content: space-between; padding: 7px 12px; border-radius: 4px; cursor: pointer; transition: all 0.08s ease; border-left: 2px solid transparent; background-color: transparent;"
                                        };
                                        rsx! {
                                            div {
                                                key: "doc_{global_idx}_{title}",
                                                style: "{row_style}",
                                                onmouseenter: move |_| selected_index.set(global_idx),
                                                onclick: move |_| {
                                                    let _ = state.write().execute_palette_action(act.clone());
                                                },
                                                div { style: "display: flex; align-items: center; gap: 8px; overflow: hidden;",
                                                    span { style: if is_sel { "color: var(--text-primary);" } else { "color: var(--text-muted);" },
                                                        IconFile { size: 13 }
                                                    }
                                                    div { style: "display: flex; flex-direction: column; gap: 1px; overflow: hidden;",
                                                        span { style: "font-size: 13px; font-weight: 500; color: var(--text-primary); overflow: hidden; text-overflow: ellipsis; white-space: nowrap;", "{title}" }
                                                        span { style: "font-size: 11px; color: var(--text-muted); font-family: var(--font-mono); overflow: hidden; text-overflow: ellipsis; white-space: nowrap;", "{desc}" }
                                                    }
                                                }
                                                if is_sel {
                                                    span { style: "color: var(--accent); font-family: var(--font-mono); font-size: 10px; font-weight: 600;", "↵ Open" }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }

                        // Section: Concepts
                        if !concept_items.is_empty() {
                            div {
                                div {
                                    style: "font-size: 10px; font-weight: 700; text-transform: uppercase; letter-spacing: 0.6px; color: var(--graph-cyan, #38bdf8); padding: 8px 12px 4px 12px; display: flex; align-items: center; justify-content: space-between;",
                                    span { "Concepts" }
                                    span { style: "font-family: var(--font-mono); font-size: 9px;", "{concept_items.len()}" }
                                }
                                for (global_idx, item) in concept_items {
                                    {
                                        let is_sel = *selected_index.read() == global_idx;
                                        let act = item.action.clone();
                                        let title = item.title.clone();
                                        let desc = item.description.clone();
                                        let row_style = if is_sel {
                                            "display: flex; align-items: center; justify-content: space-between; padding: 7px 12px; border-radius: 4px; cursor: pointer; transition: all 0.08s ease; border-left: 2px solid var(--graph-cyan, #38bdf8); background-color: var(--accent-focus);"
                                        } else {
                                            "display: flex; align-items: center; justify-content: space-between; padding: 7px 12px; border-radius: 4px; cursor: pointer; transition: all 0.08s ease; border-left: 2px solid transparent; background-color: transparent;"
                                        };
                                        rsx! {
                                            div {
                                                key: "concept_{global_idx}_{title}",
                                                style: "{row_style}",
                                                onmouseenter: move |_| selected_index.set(global_idx),
                                                onclick: move |_| {
                                                    let _ = state.write().execute_palette_action(act.clone());
                                                },
                                                div { style: "display: flex; align-items: center; gap: 8px; overflow: hidden;",
                                                    span { style: "color: var(--graph-cyan, #38bdf8); font-size: 11px; font-weight: 700;", "◆" }
                                                    div { style: "display: flex; flex-direction: column; gap: 1px; overflow: hidden;",
                                                        span { style: "font-size: 13px; font-weight: 500; color: var(--text-primary); overflow: hidden; text-overflow: ellipsis; white-space: nowrap;", "{title}" }
                                                        span { style: "font-size: 11px; color: var(--text-muted); font-family: var(--font-mono); overflow: hidden; text-overflow: ellipsis; white-space: nowrap;", "{desc}" }
                                                    }
                                                }
                                                if is_sel {
                                                    span { style: "color: var(--graph-cyan, #38bdf8); font-family: var(--font-mono); font-size: 10px; font-weight: 600;", "↵ Graph" }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }

                        // Section: Commands
                        if !cmd_items.is_empty() {
                            div {
                                div {
                                    style: "font-size: 10px; font-weight: 700; text-transform: uppercase; letter-spacing: 0.6px; color: var(--text-muted); padding: 8px 12px 4px 12px; display: flex; align-items: center; justify-content: space-between;",
                                    span { "Commands" }
                                    span { style: "font-family: var(--font-mono); font-size: 9px;", "{cmd_items.len()}" }
                                }
                                for (global_idx, item) in cmd_items {
                                    {
                                        let is_sel = *selected_index.read() == global_idx;
                                        let act = item.action.clone();
                                        let title = item.title.clone();
                                        let desc = item.description.clone();
                                        let row_style = if is_sel {
                                            "display: flex; align-items: center; justify-content: space-between; padding: 7px 12px; border-radius: 4px; cursor: pointer; transition: all 0.08s ease; border-left: 2px solid var(--accent); background-color: var(--accent-focus);"
                                        } else {
                                            "display: flex; align-items: center; justify-content: space-between; padding: 7px 12px; border-radius: 4px; cursor: pointer; transition: all 0.08s ease; border-left: 2px solid transparent; background-color: transparent;"
                                        };
                                        rsx! {
                                            div {
                                                key: "cmd_{global_idx}_{title}",
                                                style: "{row_style}",
                                                onmouseenter: move |_| selected_index.set(global_idx),
                                                onclick: move |_| {
                                                    let _ = state.write().execute_palette_action(act.clone());
                                                },
                                                div { style: "display: flex; align-items: center; gap: 8px; overflow: hidden;",
                                                    span { style: if is_sel { "color: var(--text-primary);" } else { "color: var(--text-muted);" },
                                                        IconSliders { size: 13 }
                                                    }
                                                    div { style: "display: flex; flex-direction: column; gap: 1px; overflow: hidden;",
                                                        span { style: "font-size: 13px; font-weight: 500; color: var(--text-primary); overflow: hidden; text-overflow: ellipsis; white-space: nowrap;", "{title}" }
                                                        span { style: "font-size: 11px; color: var(--text-muted); overflow: hidden; text-overflow: ellipsis; white-space: nowrap;", "{desc}" }
                                                    }
                                                }
                                                if is_sel {
                                                    span { style: "color: var(--accent); font-family: var(--font-mono); font-size: 10px; font-weight: 600;", "↵ Run" }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                // Modal Footer
                div {
                    style: "display: flex; align-items: center; justify-content: space-between; padding: 8px 16px; border-top: 1px solid var(--border); background-color: var(--bg-surface-elevated); font-size: 11px; font-family: var(--font-mono); color: var(--text-muted);",
                    div { style: "display: flex; align-items: center; gap: 12px;",
                        span { "↑↓ Navigate" }
                        span { "↵ Select" }
                        span { "Tab Cycle" }
                    }
                    span { "Esc Close" }
                }
            }
        }
    }
}
