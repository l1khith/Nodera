use dioxus::prelude::*;

use crate::icons::{IconClose, IconEdit, IconPlus, IconTemplate, IconTrash};
use crate::state::AppState;
use crate::strings::{actions, dialogs};

#[component]
pub fn Dialogs(state: Signal<AppState>) -> Element {
    let app_state = state.read();
    let show_new_note = app_state.show_new_note_dialog;
    let show_delete = app_state.show_delete_confirm_dialog;
    let delete_path = app_state.note_to_delete.clone();
    let show_rename = app_state.show_rename_dialog;
    let rename_path = app_state.note_to_rename.clone();

    let next_title = app_state.next_available_note_title();
    let mut new_note_title = use_signal(String::new);
    let mut rename_note_title = use_signal(String::new);

    rsx! {
        // Create Note Dialog
        if show_new_note {
            div { class: "modal-overlay",
                div { class: "modal-dialog",
                    h3 {
                        class: "modal-title",
                        style: "display: flex; align-items: center; gap: 8px;",
                        IconPlus { size: 16 }
                        span { "{dialogs::CREATE_NOTE_TITLE}" }
                    }
                    div { class: "modal-body",
                        p { "{dialogs::CREATE_NOTE_PROMPT}" }
                        input {
                            class: "modal-input",
                            r#type: "text",
                            value: "{new_note_title}",
                            placeholder: "{next_title}",
                            autofocus: true,
                            oninput: move |e| new_note_title.set(e.value()),
                            onkeydown: move |e: KeyboardEvent| {
                                if e.key() == Key::Enter {
                                    let input_val = new_note_title.read().trim().to_string();
                                    let mut s = state.write();
                                    let _ = s.create_note(&input_val, None);
                                    s.show_new_note_dialog = false;
                                    new_note_title.set(String::new());
                                } else if e.key() == Key::Escape {
                                    state.write().show_new_note_dialog = false;
                                    new_note_title.set(String::new());
                                }
                            }
                        }
                    }
                    div { class: "modal-actions",
                        button {
                            class: "btn-action",
                            onclick: move |_| {
                                let mut s = state.write();
                                s.show_new_note_dialog = false;
                                new_note_title.set(String::new());
                            },
                            "{actions::CANCEL}"
                        }
                        button {
                            class: "btn-action btn-primary",
                            onclick: move |_| {
                                let input_val = new_note_title.read().trim().to_string();
                                let mut s = state.write();
                                let _ = s.create_note(&input_val, None);
                                s.show_new_note_dialog = false;
                                new_note_title.set(String::new());
                            },
                            "{actions::CREATE_NOTE}"
                        }
                    }
                }
            }
        }

        // Rename Note Dialog
        if show_rename {
            {
                let current_stem = rename_path
                    .as_ref()
                    .and_then(|p| p.file_stem())
                    .and_then(|s| s.to_str())
                    .unwrap_or("Note")
                    .to_string();

                rsx! {
                    div { class: "modal-overlay",
                        div { class: "modal-dialog",
                            h3 {
                                class: "modal-title",
                                style: "display: flex; align-items: center; gap: 8px;",
                                IconEdit { size: 16 }
                                span { "{dialogs::RENAME_NOTE_TITLE}" }
                            }
                            div { class: "modal-body",
                                p { "Enter new name for note '{current_stem}':" }
                                input {
                                    class: "modal-input",
                                    r#type: "text",
                                    value: "{rename_note_title}",
                                    placeholder: "{current_stem}",
                                    autofocus: true,
                                    oninput: move |e| rename_note_title.set(e.value()),
                                }
                            }
                            div { class: "modal-actions",
                                button {
                                    class: "btn-action",
                                    onclick: move |_| {
                                        let mut s = state.write();
                                        s.show_rename_dialog = false;
                                        s.note_to_rename = None;
                                    },
                                    "{actions::CANCEL}"
                                }
                                button {
                                    class: "btn-action btn-primary",
                                    onclick: move |_| {
                                        let new_name = rename_note_title.read().clone();
                                        if !new_name.trim().is_empty() {
                                            if let Some(target) = rename_path.as_ref() {
                                                let mut s = state.write();
                                                let _ = s.rename_note(target, &new_name);
                                            }
                                        }
                                        let mut s = state.write();
                                        s.show_rename_dialog = false;
                                        s.note_to_rename = None;
                                    },
                                    "{actions::RENAME}"
                                }
                            }
                        }
                    }
                }
            }
        }

        // Confirm Delete Dialog
        if show_delete {
            {
                let display_target = delete_path
                    .as_ref()
                    .map(|p| p.display().to_string())
                    .unwrap_or_default();

                rsx! {
                    div { class: "modal-overlay",
                        div { class: "modal-dialog",
                            h3 {
                                class: "modal-title",
                                style: "color: var(--danger); display: flex; align-items: center; gap: 8px;",
                                IconTrash { size: 16 }
                                span { "{dialogs::DELETE_NOTE_TITLE}" }
                            }
                            div { class: "modal-body",
                                p {
                                    "{dialogs::DELETE_NOTE_CONFIRM}"
                                    strong { "'{display_target}'" }
                                    "? You can safely move it to Trash and restore it later, or delete permanently."
                                }
                            }
                            div { class: "modal-actions",
                                button {
                                    class: "btn-action",
                                    onclick: move |_| {
                                        let mut s = state.write();
                                        s.show_delete_confirm_dialog = false;
                                        s.note_to_delete = None;
                                    },
                                    "{actions::CANCEL}"
                                }
                                {
                                    let delete_path_perm = delete_path.clone();
                                    let delete_path_trash = delete_path.clone();
                                    rsx! {
                                        button {
                                            class: "btn-action",
                                            style: "color: var(--danger);",
                                            title: "Delete permanently without moving to trash",
                                            onclick: move |_| {
                                                if let Some(target) = delete_path_perm.as_ref() {
                                                    let mut s = state.write();
                                                    let _ = s.delete_note_permanently(target);
                                                }
                                                let mut s = state.write();
                                                s.show_delete_confirm_dialog = false;
                                                s.note_to_delete = None;
                                            },
                                            "{actions::DELETE_PERMANENTLY}"
                                        }
                                        button {
                                            class: "btn-action btn-primary",
                                            onclick: move |_| {
                                                if let Some(target) = delete_path_trash.as_ref() {
                                                    let mut s = state.write();
                                                    let _ = s.trash_note(target);
                                                }
                                                let mut s = state.write();
                                                s.show_delete_confirm_dialog = false;
                                                s.note_to_delete = None;
                                            },
                                            "Move to Trash"
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        // Template Picker Modal
        if app_state.show_template_modal {
            {
                let templates = app_state.get_available_templates();
                let query = app_state.template_search_query.to_lowercase();
                let filtered: Vec<_> = templates
                    .into_iter()
                    .filter(|t| {
                        query.is_empty()
                            || t.name.to_lowercase().contains(&query)
                            || t.description.to_lowercase().contains(&query)
                    })
                    .collect();

                rsx! {
                    div {
                        class: "modal-overlay",
                        onclick: move |_| {
                            let mut s = state.write();
                            s.show_template_modal = false;
                        },
                        div {
                            class: "modal-dialog",
                            style: "width: 580px; max-width: 90vw; padding: 0; overflow: hidden; border-radius: 8px;",
                            onclick: move |evt| evt.stop_propagation(),

                            // Header with search input
                            div {
                                style: "display: flex; align-items: center; padding: 14px 18px; border-bottom: 1px solid var(--border); gap: 10px; background-color: var(--bg-surface-elevated);",
                                IconTemplate { size: 16 }
                                input {
                                    style: "flex: 1; border: none; background: transparent; font-size: 14px; outline: none; color: var(--text-primary);",
                                    placeholder: "Search templates or type custom...",
                                    autofocus: true,
                                    value: "{app_state.template_search_query}",
                                    oninput: move |evt| {
                                        state.write().template_search_query = evt.value();
                                    },
                                    onkeydown: move |evt: KeyboardEvent| {
                                        if evt.key() == Key::Escape {
                                            state.write().show_template_modal = false;
                                        }
                                    },
                                }
                                button {
                                    class: "btn-icon",
                                    title: "Close (Esc)",
                                    style: "width: 24px; height: 24px;",
                                    onclick: move |_| {
                                        state.write().show_template_modal = false;
                                    },
                                    IconClose { size: 12 }
                                }
                            }

                            // Template items list
                            div {
                                style: "max-height: 400px; overflow-y: auto; padding: 8px; display: flex; flex-direction: column; gap: 4px;",
                                if filtered.is_empty() {
                                    div {
                                        style: "padding: 32px; text-align: center; color: var(--text-muted); font-size: 13px;",
                                        "No templates matching search. You can add .md templates in 'Templates/' directory."
                                    }
                                } else {
                                    for tmpl in filtered {
                                        {
                                            let name = tmpl.name.clone();
                                            let content = tmpl.content.clone();
                                            rsx! {
                                                div {
                                                    key: "{tmpl.name}",
                                                    class: "command-palette-row",
                                                    style: "display: flex; flex-direction: column; gap: 4px; padding: 10px 14px; border-radius: 6px; cursor: pointer; border: 1px solid transparent; transition: all 0.15s ease;",
                                                    onclick: move |_| {
                                                        let mut s = state.write();
                                                        let _ = s.insert_template(Some(&name), &content);
                                                    },
                                                    div { style: "display: flex; align-items: center; justify-content: space-between;",
                                                        span { style: "font-size: 13px; font-weight: 600; color: var(--text-primary);", "{tmpl.name}" }
                                                        span { style: "font-size: 11px; color: var(--accent); font-weight: 500;", "Click to insert" }
                                                    }
                                                    span { style: "font-size: 12px; color: var(--text-muted); line-height: 1.4;", "{tmpl.description}" }
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

        // Trash Bin Modal
        if app_state.show_trash_modal {
            {
                let trash_items = app_state.list_trash();
                let count = trash_items.len();

                rsx! {
                    div { class: "modal-overlay",
                        div { class: "modal-dialog", style: "max-width: 580px;",
                            div { style: "display: flex; align-items: center; justify-content: space-between; margin-bottom: 16px;",
                                h3 {
                                    class: "modal-title",
                                    style: "display: flex; align-items: center; gap: 8px; margin: 0;",
                                    IconTrash { size: 16 }
                                    span { "Trash Bin ({count})" }
                                }
                                div { style: "display: flex; align-items: center; gap: 8px;",
                                    if count > 0 {
                                        button {
                                            class: "btn-action",
                                            style: "color: var(--danger); font-size: 11px;",
                                            onclick: move |_| {
                                                let mut s = state.write();
                                                let _ = s.empty_trash();
                                            },
                                            "Empty Trash"
                                        }
                                    }
                                    button {
                                        class: "btn-icon",
                                        onclick: move |_| {
                                            state.write().show_trash_modal = false;
                                        },
                                        IconClose { size: 14 }
                                    }
                                }
                            }

                            div { style: "max-height: 380px; overflow-y: auto; display: flex; flex-direction: column; gap: 6px; padding-right: 4px;",
                                if trash_items.is_empty() {
                                    div { style: "padding: 32px; text-align: center; color: var(--text-muted); font-size: 13px;",
                                        "Trash is empty. Deleted notes are safely stored here and can be restored."
                                    }
                                } else {
                                    for item in trash_items {
                                        {
                                            let trash_file = item.trash_filename.clone();
                                            let trash_file_del = item.trash_filename.clone();
                                            let orig = item.original_path.display().to_string();
                                            rsx! {
                                                div {
                                                    key: "{item.trash_filename}",
                                                    style: "display: flex; align-items: center; justify-content: space-between; padding: 10px 14px; background: var(--bg-surface); border: 1px solid var(--border); border-radius: 6px;",
                                                    div { style: "display: flex; flex-direction: column; gap: 2px;",
                                                        span { style: "font-weight: 600; font-size: 13px; color: var(--text-primary);", "{item.title}" }
                                                        span { style: "font-size: 11px; color: var(--text-muted);", "Original: {orig}" }
                                                    }
                                                    div { style: "display: flex; align-items: center; gap: 8px;",
                                                        button {
                                                            class: "btn-action btn-primary",
                                                            style: "font-size: 11px; padding: 4px 10px;",
                                                            onclick: move |_| {
                                                                let mut s = state.write();
                                                                let _ = s.restore_trashed_note(&trash_file);
                                                            },
                                                            "Restore"
                                                        }
                                                        button {
                                                            class: "btn-action",
                                                            style: "color: var(--danger); font-size: 11px; padding: 4px 8px;",
                                                            onclick: move |_| {
                                                                let mut s = state.write();
                                                                let _ = s.delete_trashed_permanently(&trash_file_del);
                                                            },
                                                            "Delete Forever"
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
    }
}
