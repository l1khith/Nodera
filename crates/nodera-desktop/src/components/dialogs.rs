use dioxus::prelude::*;

use crate::icons::{
    IconCalendar, IconClose, IconEdit, IconFolderPlus, IconPlus, IconTemplate, IconTrash,
};
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
    let show_go_to_date = app_state.show_go_to_date_dialog;
    let go_to_date_val = app_state.go_to_date_input.clone();
    let go_to_date_err = app_state.go_to_date_error.clone();

    let show_new_folder = app_state.show_new_folder_dialog;
    let new_folder_parent = app_state.new_folder_target_parent.clone();
    let show_rename_folder = app_state.show_rename_folder_dialog;
    let folder_to_rename = app_state.folder_to_rename.clone();
    let show_delete_folder = app_state.show_delete_folder_dialog;
    let folder_to_delete = app_state.folder_to_delete.clone();

    let next_title = app_state.next_available_note_title();
    let mut new_note_title = use_signal(String::new);
    let mut rename_note_title = use_signal(String::new);
    let mut new_folder_name = use_signal(String::new);
    let mut rename_folder_name = use_signal(String::new);

    rsx! {
        // Create Note Dialog
        if show_new_note {
            {
                let target_folder = app_state.new_file_target_folder.clone();
                rsx! {
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
                                if let Some(ref target) = target_folder {
                                    if target.as_os_str().is_empty() {
                                        p { style: "font-size: 11px; color: var(--text-muted); margin-top: -4px; margin-bottom: 8px;", "Create in: Vault Root" }
                                    } else {
                                        p { style: "font-size: 11px; color: var(--text-muted); margin-top: -4px; margin-bottom: 8px;", "Create in: {target.display()}" }
                                    }
                                }
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
                                            let target = state.read().new_file_target_folder.clone();
                                            let mut s = state.write();
                                            let _ = s.create_note_in_folder(&input_val, target.as_deref());
                                            s.show_new_note_dialog = false;
                                            s.new_file_target_folder = None;
                                            new_note_title.set(String::new());
                                        } else if e.key() == Key::Escape {
                                            let mut s = state.write();
                                            s.show_new_note_dialog = false;
                                            s.new_file_target_folder = None;
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
                                        s.new_file_target_folder = None;
                                        new_note_title.set(String::new());
                                    },
                                    "{actions::CANCEL}"
                                }
                                button {
                                    class: "btn-action btn-primary",
                                    onclick: move |_| {
                                        let input_val = new_note_title.read().trim().to_string();
                                        let target = state.read().new_file_target_folder.clone();
                                        let mut s = state.write();
                                        let _ = s.create_note_in_folder(&input_val, target.as_deref());
                                        s.show_new_note_dialog = false;
                                        s.new_file_target_folder = None;
                                        new_note_title.set(String::new());
                                    },
                                    "{actions::CREATE_NOTE}"
                                }
                            }
                        }
                    }
                }
            }
        }

        // Create Folder Dialog
        if show_new_folder {
            {
                let target_parent = new_folder_parent.clone();
                rsx! {
                    div { class: "modal-overlay",
                        div { class: "modal-dialog",
                            h3 {
                                class: "modal-title",
                                style: "display: flex; align-items: center; gap: 8px;",
                                IconFolderPlus { size: 16 }
                                span { "New Folder" }
                            }
                            div { class: "modal-body",
                                p { "Enter folder name:" }
                                if let Some(ref parent) = target_parent {
                                    if !parent.as_os_str().is_empty() {
                                        p { style: "font-size: 11px; color: var(--text-muted); margin-top: -4px; margin-bottom: 8px;", "Create in: {parent.display()}" }
                                    } else {
                                        p { style: "font-size: 11px; color: var(--text-muted); margin-top: -4px; margin-bottom: 8px;", "Create in: Vault Root" }
                                    }
                                } else {
                                    p { style: "font-size: 11px; color: var(--text-muted); margin-top: -4px; margin-bottom: 8px;", "Create in: Vault Root" }
                                }
                                {
                                    let target_parent_kd = target_parent.clone();
                                    rsx! {
                                        input {
                                            class: "modal-input",
                                            r#type: "text",
                                            value: "{new_folder_name}",
                                            placeholder: "Folder name",
                                            autofocus: true,
                                            oninput: move |e| new_folder_name.set(e.value()),
                                            onkeydown: move |e: KeyboardEvent| {
                                                if e.key() == Key::Enter {
                                                    let input_val = new_folder_name.read().trim().to_string();
                                                    if !input_val.is_empty() {
                                                        let parent = target_parent_kd.clone();
                                                        let mut s = state.write();
                                                        let _ = s.create_folder(parent.as_deref(), &input_val);
                                                    }
                                                    let mut s = state.write();
                                                    s.show_new_folder_dialog = false;
                                                    s.new_folder_target_parent = None;
                                                    new_folder_name.set(String::new());
                                                } else if e.key() == Key::Escape {
                                                    let mut s = state.write();
                                                    s.show_new_folder_dialog = false;
                                                    s.new_folder_target_parent = None;
                                                    new_folder_name.set(String::new());
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                            div { class: "modal-actions",
                                button {
                                    class: "btn-action",
                                    onclick: move |_| {
                                        let mut s = state.write();
                                        s.show_new_folder_dialog = false;
                                        s.new_folder_target_parent = None;
                                        new_folder_name.set(String::new());
                                    },
                                    "{actions::CANCEL}"
                                }
                                button {
                                    class: "btn-action btn-primary",
                                    onclick: move |_| {
                                        let input_val = new_folder_name.read().trim().to_string();
                                        if !input_val.is_empty() {
                                            let parent = target_parent.clone();
                                            let mut s = state.write();
                                            let _ = s.create_folder(parent.as_deref(), &input_val);
                                        }
                                        let mut s = state.write();
                                        s.show_new_folder_dialog = false;
                                        s.new_folder_target_parent = None;
                                        new_folder_name.set(String::new());
                                    },
                                    "Create Folder"
                                }
                            }
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

        // Rename Folder Dialog
        if show_rename_folder {
            {
                let current_name = folder_to_rename
                    .as_ref()
                    .and_then(|p| p.file_name())
                    .and_then(|s| s.to_str())
                    .unwrap_or("Folder")
                    .to_string();
                let target_folder = folder_to_rename.clone();

                rsx! {
                    div { class: "modal-overlay",
                        div { class: "modal-dialog",
                            h3 {
                                class: "modal-title",
                                style: "display: flex; align-items: center; gap: 8px;",
                                IconEdit { size: 16 }
                                span { "Rename Folder" }
                            }
                            div { class: "modal-body",
                                p { "Enter new name for folder '{current_name}':" }
                                {
                                    let target_folder_kd = target_folder.clone();
                                    rsx! {
                                        input {
                                            class: "modal-input",
                                            r#type: "text",
                                            value: "{rename_folder_name}",
                                            placeholder: "{current_name}",
                                            autofocus: true,
                                            oninput: move |e| rename_folder_name.set(e.value()),
                                            onkeydown: move |e: KeyboardEvent| {
                                                if e.key() == Key::Enter {
                                                    let new_name = rename_folder_name.read().trim().to_string();
                                                    if !new_name.is_empty() {
                                                        if let Some(target) = target_folder_kd.as_ref() {
                                                            let mut s = state.write();
                                                            let _ = s.rename_folder(target, &new_name);
                                                        }
                                                    }
                                                    let mut s = state.write();
                                                    s.show_rename_folder_dialog = false;
                                                    s.folder_to_rename = None;
                                                    rename_folder_name.set(String::new());
                                                } else if e.key() == Key::Escape {
                                                    let mut s = state.write();
                                                    s.show_rename_folder_dialog = false;
                                                    s.folder_to_rename = None;
                                                    rename_folder_name.set(String::new());
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                            div { class: "modal-actions",
                                button {
                                    class: "btn-action",
                                    onclick: move |_| {
                                        let mut s = state.write();
                                        s.show_rename_folder_dialog = false;
                                        s.folder_to_rename = None;
                                        rename_folder_name.set(String::new());
                                    },
                                    "{actions::CANCEL}"
                                }
                                button {
                                    class: "btn-action btn-primary",
                                    onclick: move |_| {
                                        let new_name = rename_folder_name.read().trim().to_string();
                                        if !new_name.is_empty() {
                                            if let Some(target) = target_folder.as_ref() {
                                                let mut s = state.write();
                                                let _ = s.rename_folder(target, &new_name);
                                            }
                                        }
                                        let mut s = state.write();
                                        s.show_rename_folder_dialog = false;
                                        s.folder_to_rename = None;
                                        rename_folder_name.set(String::new());
                                    },
                                    "{actions::RENAME}"
                                }
                            }
                        }
                    }
                }
            }
        }

        // Delete Folder Confirmation Dialog
        if show_delete_folder {
            {
                let current_name = folder_to_delete
                    .as_ref()
                    .and_then(|p| p.file_name())
                    .and_then(|s| s.to_str())
                    .unwrap_or("Folder")
                    .to_string();
                let target_folder = folder_to_delete.clone();

                rsx! {
                    div { class: "modal-overlay",
                        div { class: "modal-dialog",
                            h3 {
                                class: "modal-title",
                                style: "color: var(--danger); display: flex; align-items: center; gap: 8px;",
                                IconTrash { size: 16 }
                                span { "Delete Folder" }
                            }
                            div { class: "modal-body",
                                p {
                                    "Are you sure you want to delete '"
                                    strong { "{current_name}" }
                                    "' and all of its contents? This cannot be undone."
                                }
                            }
                            div { class: "modal-actions",
                                button {
                                    class: "btn-action",
                                    onclick: move |_| {
                                        let mut s = state.write();
                                        s.show_delete_folder_dialog = false;
                                        s.folder_to_delete = None;
                                    },
                                    "{actions::CANCEL}"
                                }
                                button {
                                    class: "btn-action",
                                    style: "background-color: var(--danger); color: white; border: none;",
                                    onclick: move |_| {
                                        if let Some(target) = target_folder.as_ref() {
                                            let mut s = state.write();
                                            let _ = s.delete_folder(target);
                                        }
                                        let mut s = state.write();
                                        s.show_delete_folder_dialog = false;
                                        s.folder_to_delete = None;
                                    },
                                    "Delete Folder"
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

        // Go to Daily Note for Date Dialog
        if show_go_to_date {
            div { class: "modal-overlay",
                div { class: "modal-dialog",
                    h3 {
                        class: "modal-title",
                        style: "display: flex; align-items: center; gap: 8px;",
                        IconCalendar { size: 16 }
                        span { "Go to Daily Note for Date" }
                    }
                    div { class: "modal-body",
                        p { "Enter a date (YYYY-MM-DD) to open or create its daily note:" }
                        input {
                            class: "modal-input",
                            r#type: "date",
                            value: "{go_to_date_val}",
                            autofocus: true,
                            oninput: move |e| {
                                let mut s = state.write();
                                s.go_to_date_input = e.value();
                                s.go_to_date_error = None;
                            },
                            onkeydown: move |e: KeyboardEvent| {
                                if e.key() == Key::Enter {
                                    let val = state.read().go_to_date_input.trim().to_string();
                                    if let Ok(parsed) = chrono::NaiveDate::parse_from_str(&val, "%Y-%m-%d") {
                                        let mut s = state.write();
                                        s.show_go_to_date_dialog = false;
                                        s.go_to_date_error = None;
                                        let _ = s.calendar_select_and_open_date(parsed);
                                    } else {
                                        state.write().go_to_date_error = Some("Invalid date. Please use YYYY-MM-DD format.".to_string());
                                    }
                                } else if e.key() == Key::Escape {
                                    let mut s = state.write();
                                    s.show_go_to_date_dialog = false;
                                    s.go_to_date_error = None;
                                }
                            }
                        }
                        if let Some(err) = &go_to_date_err {
                            p {
                                style: "color: var(--danger); font-size: 12px; margin-top: 6px; margin-bottom: 0;",
                                "{err}"
                            }
                        }
                    }
                    div { class: "modal-actions",
                        button {
                            class: "btn-action",
                            onclick: move |_| {
                                let mut s = state.write();
                                s.show_go_to_date_dialog = false;
                                s.go_to_date_error = None;
                            },
                            "{actions::CANCEL}"
                        }
                        button {
                            class: "btn-action btn-primary",
                            onclick: move |_| {
                                let val = state.read().go_to_date_input.trim().to_string();
                                if let Ok(parsed) = chrono::NaiveDate::parse_from_str(&val, "%Y-%m-%d") {
                                    let mut s = state.write();
                                    s.show_go_to_date_dialog = false;
                                    s.go_to_date_error = None;
                                    let _ = s.calendar_select_and_open_date(parsed);
                                } else {
                                    state.write().go_to_date_error = Some("Invalid date. Please use YYYY-MM-DD format.".to_string());
                                }
                            },
                            "Open / Create Note"
                        }
                    }
                }
            }
        }
    }
}
