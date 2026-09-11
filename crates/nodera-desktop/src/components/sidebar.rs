use dioxus::prelude::*;
use nodera_core::VaultEntry;

use crate::state::AppState;

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
                        "Workspace"
                    }
                    div { style: "display: flex; gap: 4px;",
                        button {
                            class: "btn-icon",
                            title: "Open Existing Vault",
                            onclick: move |_| {
                                if let Some(folder) = rfd::FileDialog::new().pick_folder() {
                                    let mut s = state.write();
                                    let _ = s.open_vault(folder);
                                }
                            },
                            "📂"
                        }
                        button {
                            class: "btn-icon",
                            title: "Create New Vault",
                            onclick: move |_| {
                                if let Some(folder) = rfd::FileDialog::new().pick_folder() {
                                    let mut s = state.write();
                                    let _ = s.create_vault(folder, None);
                                }
                            },
                            "✨"
                        }
                        if has_vault {
                            button {
                                class: "btn-icon",
                                title: "Import PDF as Markdown (Ctrl+Shift+I)",
                                onclick: move |_| {
                                    let mut s = state.write();
                                    s.open_pdf_import_modal();
                                },
                                "📥"
                            }
                            button {
                                class: "btn-icon",
                                title: "New Note (Ctrl+N)",
                                onclick: move |_| {
                                    let mut s = state.write();
                                    s.show_new_note_dialog = true;
                                },
                                "➕"
                            }
                        }
                    }
                }
                div {
                    class: "vault-badge",
                    title: "{vault_name}",
                    "🗄️ {vault_name}"
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
                    "📝 Notes"
                }
                button {
                    style: if app_state.active_view == crate::state::ActiveView::Tasks { "padding: 5px 8px; border-radius: 4px; background-color: var(--bg-hover); font-weight: 600; display: flex; align-items: center; gap: 8px; width: 100%; text-align: left; color: var(--text-primary);" } else { "padding: 5px 8px; border-radius: 4px; color: var(--text-secondary); display: flex; align-items: center; gap: 8px; width: 100%; text-align: left;" },
                    onclick: move |_| {
                        let mut s = state.write();
                        s.active_view = crate::state::ActiveView::Tasks;
                    },
                    "✅ Tasks"
                }
                div {
                    style: "padding: 5px 8px; border-radius: 4px; color: var(--text-muted); display: flex; align-items: center; gap: 8px; opacity: 0.6; cursor: not-allowed;",
                    title: "Coming in Phase 5",
                    "📚 Library"
                }
            }

            // Quick Search Input
            if has_vault {
                div {
                    style: "padding: 8px 12px; border-bottom: 1px solid var(--border-subtle);",
                    div {
                        style: "display: flex; align-items: center; background-color: var(--bg-surface-elevated); border: 1px solid var(--border); border-radius: 4px; padding: 4px 8px; gap: 6px;",
                        span { style: "font-size: 11px; opacity: 0.7;", "🔍" }
                        input {
                            style: "flex: 1; border: none; background: transparent; font-size: 12px; outline: none; color: var(--text-primary);",
                            placeholder: "Search vault...",
                            value: "{app_state.search_query}",
                            oninput: move |evt| {
                                let mut s = state.write();
                                s.execute_search(&evt.value());
                            }
                        }
                        if !app_state.search_query.is_empty() {
                            button {
                                style: "font-size: 10px; color: var(--text-muted); cursor: pointer;",
                                onclick: move |_| {
                                    let mut s = state.write();
                                    s.execute_search("");
                                },
                                "✕"
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
                        p { "No vault open" }
                        button {
                            class: "btn-action btn-primary",
                            style: "align-self: center;",
                            onclick: move |_| {
                                if let Some(folder) = rfd::FileDialog::new().pick_folder() {
                                    let mut s = state.write();
                                    let _ = s.open_vault(folder);
                                }
                            },
                            "Open Vault Folder"
                        }
                    }
                } else if !app_state.search_query.is_empty() {
                    // Display search results
                    div { style: "display: flex; flex-direction: column; gap: 4px;",
                        div { style: "font-size: 11px; font-weight: 600; color: var(--text-muted); margin-bottom: 4px; padding: 0 4px;",
                            "SEARCH RESULTS ({app_state.search_results.len()})"
                        }
                        if app_state.search_results.is_empty() {
                            p { style: "font-size: 12px; color: var(--text-muted); padding: 8px;", "No matching notes found." }
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
                                            span { style: "font-weight: 600; font-size: 12px; color: var(--text-primary);", "📄 {title}" }
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
                        style: "padding: 20px 8px; text-align: center; color: var(--text-muted); display: flex; flex-direction: column; gap: 8px;",
                        p { "Vault is empty" }
                        button {
                            class: "btn-action",
                            style: "align-self: center;",
                            onclick: move |_| {
                                let mut s = state.write();
                                s.show_new_note_dialog = true;
                            },
                            "Create first note"
                        }
                    }
                } else {
                    for entry in entries {
                        {
                            let entry_path = entry.relative_path().to_path_buf();
                            let is_active = active_path.as_ref() == Some(&entry_path);
                            match entry {
                                VaultEntry::Folder { name, relative_path } => rsx! {
                                    div {
                                        key: "{relative_path.display()}",
                                        style: "padding: 4px 8px; font-weight: 600; font-size: 12px; color: var(--text-secondary); display: flex; align-items: center; gap: 6px; margin-top: 4px;",
                                        "📁 {name}"
                                    }
                                },
                                VaultEntry::Note(summary) => {
                                    let summary_path = summary.relative_path.clone();
                                    let click_path = summary_path.clone();
                                    let delete_target = summary_path.clone();
                                    let rename_target = summary_path.clone();
                                    let active_bg = if is_active { "var(--bg-active)" } else { "transparent" };
                                    let active_color = if is_active { "var(--text-primary)" } else { "var(--text-secondary)" };

                                    rsx! {
                                        div {
                                            key: "{summary_path.display()}",
                                            style: "padding: 4px 8px; border-radius: 4px; display: flex; align-items: center; justify-content: space-between; cursor: pointer; background-color: {active_bg}; color: {active_color}; font-size: 13px;",
                                            onclick: move |_| {
                                                let mut s = state.write();
                                                let _ = s.select_note(&click_path);
                                            },
                                            span {
                                                style: "overflow: hidden; text-overflow: ellipsis; white-space: nowrap; flex: 1;",
                                                "📄 {summary.title}"
                                            }
                                            div {
                                                style: "display: flex; gap: 2px;",
                                                button {
                                                    class: "btn-icon",
                                                    style: "width: 20px; height: 20px; font-size: 10px;",
                                                    title: "Rename",
                                                    onclick: move |e| {
                                                        e.stop_propagation();
                                                        let mut s = state.write();
                                                        s.note_to_rename = Some(rename_target.clone());
                                                        s.show_rename_dialog = true;
                                                    },
                                                    "✏️"
                                                }
                                                button {
                                                    class: "btn-icon btn-danger",
                                                    style: "width: 20px; height: 20px; font-size: 10px;",
                                                    title: "Delete",
                                                    onclick: move |e| {
                                                        e.stop_propagation();
                                                        let mut s = state.write();
                                                        s.note_to_delete = Some(delete_target.clone());
                                                        s.show_delete_confirm_dialog = true;
                                                    },
                                                    "🗑️"
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
