use dioxus::prelude::*;
use nodera_core::VaultEntry;

use crate::icons::*;
use crate::state::AppState;
use crate::strings::{actions, app as app_strings, empty_states, nav, placeholders, tooltips};

#[component]
pub fn Sidebar(state: Signal<AppState>) -> Element {
    let app_state = state.read();
    let vault_name = app_state.vault_name.clone();
    let has_vault = app_state.vault_service.is_some();
    let entries = app_state.entries.clone();
    let active_path = app_state
        .active_note
        .as_ref()
        .map(|n| n.relative_path.clone());

    rsx! {
        aside { class: "pane-sidebar",
            // Header / Vault section
            div {
                style: "padding: 12px; border-bottom: 1px solid var(--border); display: flex; flex-direction: column; gap: 8px;",
                div {
                    style: "display: flex; align-items: center; justify-content: space-between;",
                    span {
                        style: "font-weight: 600; font-size: 12px; text-transform: uppercase; letter-spacing: 0.5px; color: var(--text-muted);",
                        "{app_strings::WORKSPACE}"
                    }
                    div { style: "display: flex; gap: 4px;",
                        button {
                            class: "btn-icon",
                            title: actions::OPEN_EXISTING_VAULT,
                            onclick: move |_| {
                                if let Some(folder) = rfd::FileDialog::new().pick_folder() {
                                    let mut s = state.write();
                                    let _ = s.open_vault(folder);
                                }
                            },
                            IconFolderOpen { size: 15 }
                        }
                        button {
                            class: "btn-icon",
                            title: actions::NEW_VAULT,
                            onclick: move |_| {
                                if let Some(folder) = rfd::FileDialog::new().pick_folder() {
                                    let mut s = state.write();
                                    let _ = s.create_vault(folder, None);
                                }
                            },
                            IconFolderPlus { size: 15 }
                        }
                        if has_vault {
                            button {
                                class: "btn-icon",
                                title: tooltips::IMPORT_PDF,
                                onclick: move |_| {
                                    let mut s = state.write();
                                    s.open_pdf_import_modal();
                                },
                                IconImport { size: 15 }
                            }
                            button {
                                class: "btn-icon",
                                title: tooltips::NEW_NOTE,
                                onclick: move |_| {
                                    let mut s = state.write();
                                    s.show_new_note_dialog = true;
                                },
                                IconPlus { size: 15 }
                            }
                        }
                    }
                }
                div {
                    class: "vault-badge",
                    style: "display: flex; align-items: center; gap: 6px;",
                    title: "{vault_name}",
                    IconVault { size: 13 }
                    span { "{vault_name}" }
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
                    style: if app_state.active_view == crate::state::ActiveView::Library { "padding: 5px 8px; border-radius: 4px; background-color: var(--bg-hover); font-weight: 600; display: flex; align-items: center; gap: 8px; width: 100%; text-align: left; color: var(--text-primary);" } else { "padding: 5px 8px; border-radius: 4px; color: var(--text-secondary); display: flex; align-items: center; gap: 8px; width: 100%; text-align: left;" },
                    onclick: move |_| {
                        let mut s = state.write();
                        s.active_view = crate::state::ActiveView::Library;
                    },
                    IconLibrary { size: 15 }
                    span { "{nav::LIBRARY}" }
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
                                if let Some(folder) = rfd::FileDialog::new().pick_folder() {
                                    let mut s = state.write();
                                    let _ = s.open_vault(folder);
                                }
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
                                            style: format!("padding: 4px 8px; padding-left: {pad_left}px; font-weight: 600; font-size: 12px; color: var(--text-secondary); display: flex; align-items: center; gap: 6px; margin-top: 4px;"),
                                            IconFolder { size: 14 }
                                            span { "{name}" }
                                        }
                                    }
                                },
                                VaultEntry::Note(summary) => {
                                    let summary_path = summary.relative_path.clone();
                                    let click_path = summary_path.clone();
                                    let delete_target = summary_path.clone();
                                    let rename_target = summary_path.clone();
                                    let active_bg = if is_active { "var(--bg-active)" } else { "transparent" };
                                    let active_color = if is_active { "var(--text-primary)" } else { "var(--text-secondary)" };
                                    let depth = summary_path.components().count().saturating_sub(1);
                                    let pad_left = 8 + depth * 14;

                                    rsx! {
                                        div {
                                            key: "{summary_path.display()}",
                                            style: format!("padding: 4px 8px; padding-left: {pad_left}px; border-radius: 4px; display: flex; align-items: center; justify-content: space-between; cursor: pointer; background-color: {active_bg}; color: {active_color}; font-size: 13px;"),
                                            onclick: move |_| {
                                                let mut s = state.write();
                                                let _ = s.select_note(&click_path);
                                            },
                                            span {
                                                style: "overflow: hidden; text-overflow: ellipsis; white-space: nowrap; flex: 1; display: inline-flex; align-items: center; gap: 6px;",
                                                IconFile { size: 14 }
                                                span { "{summary.title}" }
                                            }
                                            div {
                                                style: "display: flex; gap: 2px;",
                                                button {
                                                    class: "btn-icon",
                                                    style: "width: 20px; height: 20px;",
                                                    title: tooltips::RENAME_NOTE,
                                                    onclick: move |e| {
                                                        e.stop_propagation();
                                                        let mut s = state.write();
                                                        s.note_to_rename = Some(rename_target.clone());
                                                        s.show_rename_dialog = true;
                                                    },
                                                    IconEdit { size: 11 }
                                                }
                                                button {
                                                    class: "btn-icon btn-danger",
                                                    style: "width: 20px; height: 20px;",
                                                    title: tooltips::DELETE_NOTE,
                                                    onclick: move |e| {
                                                        e.stop_propagation();
                                                        let mut s = state.write();
                                                        s.note_to_delete = Some(delete_target.clone());
                                                        s.show_delete_confirm_dialog = true;
                                                    },
                                                    IconTrash { size: 11 }
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
    }
}
