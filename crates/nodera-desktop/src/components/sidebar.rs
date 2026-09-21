use dioxus::prelude::*;
use nodera_core::VaultEntry;

use crate::icons::*;
use crate::state::AppState;
use crate::strings::{actions, app as app_strings, empty_states, nav, placeholders};

#[component]
pub fn Sidebar(state: Signal<AppState>) -> Element {
    let mut context_menu = use_signal(|| None::<(std::path::PathBuf, f64, f64)>);

    let app_state = state.read();
    let entries = app_state.entries.clone();
    let active_path = app_state
        .active_note
        .as_ref()
        .map(|n| n.relative_path.clone());
    let has_vault = app_state.vault_service.is_some();

    rsx! {
        aside { class: "pane-sidebar",
            // Workspace navigation header
            div {
                style: "padding: 10px 14px; border-bottom: 1px solid var(--border); display: flex; align-items: center; justify-content: space-between;",
                span {
                    style: "font-weight: 600; font-size: 11px; text-transform: uppercase; letter-spacing: 0.5px; color: var(--text-muted);",
                    "{app_strings::WORKSPACE}"
                }
            }

            // Navigation items: Notes, Tasks, Library
            div {
                style: "padding: 8px 12px; border-bottom: 1px solid var(--border-subtle); display: flex; flex-direction: column; gap: 2px;",
                button {
                    style: if app_state.active_view == crate::state::ActiveView::Editor { "padding: 5px 8px; border-radius: 4px; background-color: var(--bg-hover); font-weight: 600; display: flex; align-items: center; gap: 8px; width: 100%; text-align: left; color: var(--text-primary);" } else { "padding: 5px 8px; border-radius: 4px; color: var(--text-secondary); display: flex; align-items: center; gap: 8px; width: 100%; text-align: left;" },
                    onclick: move |_| {
                        let mut s = state.write();
                        s.active_view = crate::state::ActiveView::Editor;
                    },
                    IconNotes { size: 15 }
                    span { "{nav::NOTES}" }
                }
                button {
                    style: if app_state.active_view == crate::state::ActiveView::Tasks { "padding: 5px 8px; border-radius: 4px; background-color: var(--bg-hover); font-weight: 600; display: flex; align-items: center; gap: 8px; width: 100%; text-align: left; color: var(--text-primary);" } else { "padding: 5px 8px; border-radius: 4px; color: var(--text-secondary); display: flex; align-items: center; gap: 8px; width: 100%; text-align: left;" },
                    onclick: move |_| {
                        let mut s = state.write();
                        s.active_view = crate::state::ActiveView::Tasks;
                    },
                    IconTasks { size: 15 }
                    span { "{nav::TASKS}" }
                }
                button {
                    style: if app_state.active_view == crate::state::ActiveView::ReviewQueue { "padding: 5px 8px; border-radius: 4px; background-color: var(--bg-hover); font-weight: 600; display: flex; align-items: center; gap: 8px; width: 100%; text-align: left; color: var(--text-primary);" } else { "padding: 5px 8px; border-radius: 4px; color: var(--text-secondary); display: flex; align-items: center; gap: 8px; width: 100%; text-align: left;" },
                    onclick: move |_| {
                        let mut s = state.write();
                        s.active_view = crate::state::ActiveView::ReviewQueue;
                    },
                    IconReviewQueue { size: 15 }
                    span { "Review Queue" }
                }
                button {
                    style: if app_state.active_view == crate::state::ActiveView::Library { "padding: 5px 8px; border-radius: 4px; background-color: var(--bg-hover); font-weight: 600; display: flex; align-items: center; gap: 8px; width: 100%; text-align: left; color: var(--text-primary);" } else { "padding: 5px 8px; border-radius: 4px; color: var(--text-secondary); display: flex; align-items: center; gap: 8px; width: 100%; text-align: left;" },
                    onclick: move |_| {
                        let mut s = state.write();
                        s.active_view = crate::state::ActiveView::Library;
                    },
                    IconLibrary { size: 15 }
                    span { "{nav::LIBRARY}" }
                }
                button {
                    style: if app_state.active_view == crate::state::ActiveView::Graph { "padding: 5px 8px; border-radius: 4px; background-color: var(--bg-hover); font-weight: 600; display: flex; align-items: center; gap: 8px; width: 100%; text-align: left; color: var(--text-primary);" } else { "padding: 5px 8px; border-radius: 4px; color: var(--text-secondary); display: flex; align-items: center; gap: 8px; width: 100%; text-align: left;" },
                    onclick: move |_| {
                        let mut s = state.write();
                        s.active_view = crate::state::ActiveView::Graph;
                    },
                    IconGraph { size: 15 }
                    span { "{nav::GRAPH}" }
                }
                button {
                    style: "padding: 5px 8px; border-radius: 4px; color: var(--text-secondary); display: flex; align-items: center; gap: 8px; width: 100%; text-align: left;",
                    onclick: move |_| {
                        let mut s = state.write();
                        s.show_trash_modal = true;
                    },
                    IconTrash { size: 15 }
                    span { "Trash Bin" }
                }
                button {
                    style: "padding: 5px 8px; border-radius: 4px; color: var(--text-secondary); display: flex; align-items: center; gap: 8px; width: 100%; text-align: left;",
                    title: "Check broken links & orphan notes",
                    onclick: move |_| {
                        let mut s = state.write();
                        s.show_vault_health_modal = true;
                    },
                    IconActivity { size: 15 }
                    span { "Vault Health" }
                }
            }

            // Quick Search Input
            if has_vault {
                div {
                    style: "padding: 8px 12px; border-bottom: 1px solid var(--border-subtle);",
                    div {
                        style: "display: flex; align-items: center; background-color: var(--bg-surface-elevated); border: 1px solid var(--border); border-radius: 4px; padding: 4px 8px; gap: 6px;",
                        IconSearch { size: 13, class: "opacity-70" }
                        input {
                            style: "flex: 1; border: none; background: transparent; font-size: 12px; outline: none; color: var(--text-primary);",
                            placeholder: placeholders::SEARCH_VAULT,
                            value: "{app_state.search_query}",
                            oninput: move |evt| {
                                let mut s = state.write();
                                s.execute_search(&evt.value());
                            }
                        }
                        if !app_state.search_query.is_empty() {
                            button {
                                style: "font-size: 10px; color: var(--text-muted); cursor: pointer;",
                                title: actions::CLEAR,
                                onclick: move |_| {
                                    let mut s = state.write();
                                    s.execute_search("");
                                },
                                IconClose { size: 11 }
                            }
                        }
                    }
                }
            }

            // File tree / Search Results section
            div {
                style: "flex: 1; overflow-y: auto; padding: 8px;",
                if !has_vault {
                    div {
                        style: "padding: 20px 8px; text-align: center; color: var(--text-muted); display: flex; flex-direction: column; gap: 12px;",
                        p { "{empty_states::NO_VAULT_OPEN}" }
                        button {
                            class: "btn-action btn-primary",
                            style: "align-self: center;",
                            onclick: move |_| {
                                spawn(async move {
                                    if let Some(folder) = rfd::AsyncFileDialog::new().pick_folder().await {
                                        let path = folder.path().to_path_buf();
                                        let mut s = state.write();
                                        let _ = s.open_vault(path);
                                    }
                                });
                            },
                            "{empty_states::OPEN_VAULT_FOLDER}"
                        }
                    }
                } else if !app_state.search_query.is_empty() {
                    // Display search results
                    div { style: "display: flex; flex-direction: column; gap: 4px;",
                        div { style: "font-size: 11px; font-weight: 600; color: var(--text-muted); margin-bottom: 4px; padding: 0 4px;",
                            "SEARCH RESULTS ({app_state.search_results.len()})"
                        }
                        if app_state.search_results.is_empty() {
                            div {
                                style: "padding: 16px 8px; text-align: center; color: var(--text-muted); font-size: 12px; display: flex; flex-direction: column; gap: 4px;",
                                span { style: "font-weight: 500; color: var(--text-secondary);", "{empty_states::NO_SEARCH_RESULTS_TITLE}" }
                                span { style: "font-size: 11px;", "{empty_states::NO_SEARCH_RESULTS_DESC}" }
                            }
                        } else {
                            for res in app_state.search_results.iter() {
                                {
                                    let path_buf = std::path::PathBuf::from(&res.path);
                                    let path_clone = path_buf.clone();
                                    let title = res.title.clone();
                                    let snippet = res.snippet.clone();
                                    rsx! {
                                        button {
                                            key: "{res.path}",
                                            class: "link-item",
                                            style: "flex-direction: column; align-items: flex-start; gap: 3px; padding: 8px;",
                                            onclick: move |_| {
                                                let mut s = state.write();
                                                s.active_view = crate::state::ActiveView::Editor;
                                                let _ = s.select_note(&path_clone);
                                            },
                                            span {
                                                style: "font-weight: 600; font-size: 12px; color: var(--text-primary); display: inline-flex; align-items: center; gap: 6px;",
                                                IconFile { size: 13 }
                                                span { "{title}" }
                                            }
                                            span { style: "font-size: 10px; color: var(--text-muted);", "{path_buf.display()}" }
                                            if !snippet.is_empty() {
                                                div {
                                                    style: "font-size: 11px; color: var(--text-secondary); line-height: 1.4; margin-top: 2px;",
                                                    dangerous_inner_html: "{snippet}"
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                } else if entries.is_empty() {
                    div {
                        style: "padding: 24px 12px; text-align: center; color: var(--text-muted); display: flex; flex-direction: column; align-items: center; gap: 10px;",
                        IconFolderOpen { size: 32 }
                        p { style: "font-size: 13px; font-weight: 500; margin: 0; color: var(--text-primary);", "{empty_states::EMPTY_VAULT_TITLE}" }
                        p { style: "font-size: 11px; line-height: 1.4; margin: 0;", "{empty_states::EMPTY_VAULT_DESC}" }
                        div { style: "display: flex; flex-direction: column; gap: 6px; margin-top: 6px; width: 100%;",
                            button {
                                class: "btn-action btn-primary",
                                style: "width: 100%; justify-content: center;",
                                onclick: move |_| {
                                    let mut s = state.write();
                                    s.show_new_note_dialog = true;
                                },
                                IconPlus { size: 14 }
                                span { "{actions::CREATE_NOTE}" }
                            }
                            button {
                                class: "btn-action",
                                style: "width: 100%; justify-content: center;",
                                onclick: move |_| {
                                    let mut s = state.write();
                                    s.open_pdf_import_modal();
                                },
                                IconImport { size: 14 }
                                span { "{actions::IMPORT_PDF_BTN}" }
                            }
                        }
                    }
                } else {
                    // Bookmarks Section
                    if !app_state.preferences.bookmarks.is_empty() {
                        {
                            let bookmarks = app_state.preferences.bookmarks.clone();
                            let show_bookmarks = app_state.show_bookmarks_section;
                            rsx! {
                                div {
                                    style: "margin-bottom: 8px; border-bottom: 1px solid var(--border-subtle); padding-bottom: 6px;",
                                    div {
                                        style: "display: flex; align-items: center; justify-content: space-between; padding: 4px 8px; cursor: pointer; color: var(--text-muted); font-size: 11px; font-weight: 600; text-transform: uppercase; letter-spacing: 0.5px; border-radius: 4px;",
                                        onclick: move |_| {
                                            let mut s = state.write();
                                            s.show_bookmarks_section = !s.show_bookmarks_section;
                                        },
                                        div { style: "display: flex; align-items: center; gap: 5px;",
                                            if show_bookmarks {
                                                IconChevronDown { size: 11 }
                                            } else {
                                                IconChevronRight { size: 11 }
                                            }
                                            IconBookmark { size: 12 }
                                            span { "Bookmarks ({bookmarks.len()})" }
                                        }
                                    }
                                    if show_bookmarks {
                                        div { style: "display: flex; flex-direction: column; gap: 1px; margin-top: 2px;",
                                            for path in &bookmarks {
                                                {
                                                    let p = path.clone();
                                                    let is_act = active_path.as_ref() == Some(&p);
                                                    let title = p.file_stem().and_then(|s| s.to_str()).unwrap_or("Untitled").to_string();
                                                    let act_bg = if is_act { "var(--bg-active)" } else { "transparent" };
                                                    let act_fg = if is_act { "var(--text-primary)" } else { "var(--text-secondary)" };
                                                    let p_toggle = p.clone();
                                                    let p_select = p.clone();
                                                    rsx! {
                                                        div {
                                                            key: "{p.display()}",
                                                            style: format!("padding: 3px 8px 3px 18px; border-radius: 4px; display: flex; align-items: center; justify-content: space-between; cursor: pointer; background-color: {act_bg}; color: {act_fg}; font-size: 12px;"),
                                                            onclick: move |_| {
                                                                let mut s = state.write();
                                                                s.active_view = crate::state::ActiveView::Editor;
                                                                let _ = s.select_note(&p_select);
                                                            },
                                                            span {
                                                                style: "overflow: hidden; text-overflow: ellipsis; white-space: nowrap; flex: 1; display: inline-flex; align-items: center; gap: 6px;",
                                                                IconBookmark { size: 11 }
                                                                span { "{title}" }
                                                            }
                                                            button {
                                                                class: "btn-icon",
                                                                style: "width: 18px; height: 18px; opacity: 0.6;",
                                                                title: "Remove bookmark",
                                                                onclick: move |e| {
                                                                    e.stop_propagation();
                                                                    state.write().toggle_bookmark(&p_toggle);
                                                                },
                                                                IconClose { size: 10 }
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

                    // Recent Notes Section
                    if !app_state.preferences.recent_notes.is_empty() {
                        {
                            let recent_notes = app_state.preferences.recent_notes.clone();
                            let show_recent = app_state.show_recent_section;
                            rsx! {
                                div {
                                    style: "margin-bottom: 8px; border-bottom: 1px solid var(--border-subtle); padding-bottom: 6px;",
                                    div {
                                        style: "display: flex; align-items: center; justify-content: space-between; padding: 4px 8px; cursor: pointer; color: var(--text-muted); font-size: 11px; font-weight: 600; text-transform: uppercase; letter-spacing: 0.5px; border-radius: 4px;",
                                        onclick: move |_| {
                                            let mut s = state.write();
                                            s.show_recent_section = !s.show_recent_section;
                                        },
                                        div { style: "display: flex; align-items: center; gap: 5px;",
                                            if show_recent {
                                                IconChevronDown { size: 11 }
                                            } else {
                                                IconChevronRight { size: 11 }
                                            }
                                            IconClock { size: 12 }
                                            span { "Recent ({recent_notes.len()})" }
                                        }
                                        button {
                                            class: "btn-icon",
                                            style: "width: 18px; height: 18px; opacity: 0.5;",
                                            title: "Clear recent notes",
                                            onclick: move |e| {
                                                e.stop_propagation();
                                                state.write().clear_recent_notes();
                                            },
                                            IconClose { size: 10 }
                                        }
                                    }
                                    if show_recent {
                                        div { style: "display: flex; flex-direction: column; gap: 1px; margin-top: 2px;",
                                            for path in &recent_notes {
                                                {
                                                    let p = path.clone();
                                                    let is_act = active_path.as_ref() == Some(&p);
                                                    let title = p.file_stem().and_then(|s| s.to_str()).unwrap_or("Untitled").to_string();
                                                    let act_bg = if is_act { "var(--bg-active)" } else { "transparent" };
                                                    let act_fg = if is_act { "var(--text-primary)" } else { "var(--text-secondary)" };
                                                    let p_remove = p.clone();
                                                    let p_select = p.clone();
                                                    rsx! {
                                                        div {
                                                            key: "{p.display()}",
                                                            style: format!("padding: 3px 8px 3px 18px; border-radius: 4px; display: flex; align-items: center; justify-content: space-between; cursor: pointer; background-color: {act_bg}; color: {act_fg}; font-size: 12px;"),
                                                            onclick: move |_| {
                                                                let mut s = state.write();
                                                                s.active_view = crate::state::ActiveView::Editor;
                                                                let _ = s.select_note(&p_select);
                                                            },
                                                            span {
                                                                style: "overflow: hidden; text-overflow: ellipsis; white-space: nowrap; flex: 1; display: inline-flex; align-items: center; gap: 6px;",
                                                                IconClock { size: 11 }
                                                                span { "{title}" }
                                                            }
                                                            button {
                                                                class: "btn-icon",
                                                                style: "width: 18px; height: 18px; opacity: 0.6;",
                                                                title: "Remove from recent",
                                                                onclick: move |e| {
                                                                    e.stop_propagation();
                                                                    state.write().remove_recent_note(&p_remove);
                                                                },
                                                                IconClose { size: 10 }
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

                    // Vault File Tree Header
                    div {
                        style: "padding: 4px 8px; font-weight: 600; font-size: 11px; text-transform: uppercase; letter-spacing: 0.5px; color: var(--text-muted); display: flex; align-items: center; justify-content: space-between; margin-bottom: 2px;",
                        span { "Files" }
                    }

                    for entry in entries {
                        {
                            let entry_path = entry.relative_path().to_path_buf();
                            let is_active = active_path.as_ref() == Some(&entry_path);
                            match entry {
                                VaultEntry::Folder { name, relative_path } => {
                                    let depth = relative_path.components().count().saturating_sub(1);
                                    let pad_left = 8 + depth * 14;
                                    rsx! {
                                        div {
                                            key: "{relative_path.display()}",
                                            style: format!("padding: 5px 8px; padding-left: {pad_left}px; font-weight: 600; font-size: 11px; text-transform: uppercase; letter-spacing: 0.5px; color: var(--text-muted); display: flex; align-items: center; gap: 5px; margin-top: 8px; margin-bottom: 2px; user-select: none;"),
                                            IconChevronDown { size: 11, class: "opacity-70" }
                                            IconFolder { size: 13, class: "opacity-80" }
                                            span { "{name}" }
                                        }
                                    }
                                },
                                VaultEntry::Note(summary) => {
                                    let summary_path = summary.relative_path.clone();
                                    let click_path = summary_path.clone();
                                    let context_path = summary_path.clone();
                                    let is_bookmarked = app_state.is_bookmarked(&summary_path);
                                    let active_bg = if is_active { "var(--bg-active)" } else { "transparent" };
                                    let active_color = if is_active { "var(--text-primary)" } else { "var(--text-secondary)" };
                                    let depth = summary_path.components().count().saturating_sub(1);
                                    let pad_left = 8 + depth * 14;

                                    rsx! {
                                        div {
                                            key: "{summary_path.display()}",
                                            style: format!("padding: 4px 8px; padding-left: {pad_left}px; border-radius: 4px; display: flex; align-items: center; justify-content: space-between; cursor: pointer; background-color: {active_bg}; color: {active_color}; font-size: 13px; user-select: none;"),
                                            onclick: move |_| {
                                                let mut s = state.write();
                                                let _ = s.select_note(&click_path);
                                            },
                                            oncontextmenu: move |evt: MouseEvent| {
                                                evt.prevent_default();
                                                let coords = evt.client_coordinates();
                                                context_menu.set(Some((context_path.clone(), coords.x, coords.y)));
                                            },
                                            span {
                                                style: "overflow: hidden; text-overflow: ellipsis; white-space: nowrap; flex: 1; display: inline-flex; align-items: center; gap: 6px;",
                                                IconFile { size: 14, class: "opacity-70" }
                                                span { "{summary.title}" }
                                            }
                                            if is_bookmarked {
                                                span {
                                                    style: "color: var(--accent); margin-left: 4px; display: inline-flex;",
                                                    title: "Bookmarked",
                                                    IconBookmark { size: 11 }
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

            // Right-Click Context Menu for Note Rows
            if let Some((cm_path, x, y)) = context_menu.read().clone() {
                {
                    let open_path = cm_path.clone();
                    let bookmark_path = cm_path.clone();
                    let rename_path = cm_path.clone();
                    let delete_path = cm_path.clone();

                    rsx! {
                        div {
                            style: "position: fixed; top: 0; left: 0; right: 0; bottom: 0; z-index: 1099;",
                            onclick: move |_| context_menu.set(None),
                            oncontextmenu: move |e| {
                                e.prevent_default();
                                context_menu.set(None);
                            },
                        }
                        div {
                            class: "context-menu",
                            style: "left: {x}px; top: {y}px;",
                            button {
                                class: "context-menu-item",
                                onclick: move |_| {
                                    let mut s = state.write();
                                    s.active_view = crate::state::ActiveView::Editor;
                                    let _ = s.select_note(&open_path);
                                    context_menu.set(None);
                                },
                                IconNotes { size: 13 }
                                span { "Open" }
                            }
                            button {
                                class: "context-menu-item",
                                onclick: move |_| {
                                    let mut s = state.write();
                                    s.toggle_bookmark(&bookmark_path);
                                    context_menu.set(None);
                                },
                                IconBookmark { size: 13 }
                                span {
                                    if app_state.is_bookmarked(&cm_path) {
                                        "Remove Bookmark"
                                    } else {
                                        "Bookmark"
                                    }
                                }
                            }
                            button {
                                class: "context-menu-item",
                                onclick: move |_| {
                                    let mut s = state.write();
                                    s.note_to_rename = Some(rename_path.clone());
                                    s.show_rename_dialog = true;
                                    context_menu.set(None);
                                },
                                IconEdit { size: 13 }
                                span { "Rename…" }
                            }
                            div { class: "dropdown-divider" }
                            button {
                                class: "context-menu-item danger",
                                onclick: move |_| {
                                    let mut s = state.write();
                                    s.note_to_delete = Some(delete_path.clone());
                                    s.show_delete_confirm_dialog = true;
                                    context_menu.set(None);
                                },
                                IconTrash { size: 13 }
                                span { "Delete…" }
                            }
                        }
                    }
                }
            }
        }
    }
}
