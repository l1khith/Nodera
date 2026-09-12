use dioxus::prelude::*;

use crate::icons::{IconEdit, IconPlus, IconTrash};
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

    let mut new_note_title = use_signal(|| dialogs::DEFAULT_NOTE_TITLE.to_string());
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
                            autofocus: true,
                            oninput: move |e| new_note_title.set(e.value()),
                        }
                    }
                    div { class: "modal-actions",
                        button {
                            class: "btn-action",
                            onclick: move |_| {
                                let mut s = state.write();
                                s.show_new_note_dialog = false;
                            },
                            "{actions::CANCEL}"
                        }
                        button {
                            class: "btn-action btn-primary",
                            onclick: move |_| {
                                let title = new_note_title.read().clone();
                                let mut s = state.write();
                                let _ = s.create_note(&title, None);
                                s.show_new_note_dialog = false;
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
                                    "{dialogs::DELETE_NOTE_WARNING}"
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
                                button {
                                    class: "btn-action btn-danger",
                                    style: "background-color: var(--danger); color: #fff;",
                                    onclick: move |_| {
                                        if let Some(target) = delete_path.as_ref() {
                                            let mut s = state.write();
                                            let _ = s.delete_note(target);
                                        }
                                        let mut s = state.write();
                                        s.show_delete_confirm_dialog = false;
                                        s.note_to_delete = None;
                                    },
                                    "{actions::DELETE_PERMANENTLY}"
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
