use dioxus::prelude::*;

use crate::state::AppState;

/// Handles application-wide keyboard shortcuts.
///
/// Returns `true` if the shortcut was recognized and handled.
pub fn handle_global_shortcut(evt: &KeyboardEvent, state: &mut Signal<AppState>) -> bool {
    let key = evt.key();
    let modifiers = evt.modifiers();
    let has_ctrl_or_cmd = modifiers.ctrl() || modifiers.meta();

    // Escape handles closing any open modals/overlays
    if key == Key::Escape {
        let mut s = state.write();
        if s.show_error_dialog {
            s.clear_error();
            return true;
        }
        if s.show_settings_modal {
            s.close_settings();
            return true;
        }
        if s.show_command_palette {
            s.show_command_palette = false;
            return true;
        }
        if s.show_template_modal {
            s.show_template_modal = false;
            return true;
        }
        if s.show_pdf_import_modal {
            s.close_pdf_import_modal();
            return true;
        }
        if s.show_new_note_dialog {
            s.show_new_note_dialog = false;
            return true;
        }
        if s.show_delete_confirm_dialog {
            s.show_delete_confirm_dialog = false;
            return true;
        }
        if s.show_rename_dialog {
            s.show_rename_dialog = false;
            return true;
        }
        return false;
    }

    if !has_ctrl_or_cmd {
        return false;
    }

    match key {
        // Ctrl+Shift+I: Import PDF as Markdown
        Key::Character(ref c) if (c == "i" || c == "I") && modifiers.shift() => {
            let mut s = state.write();
            s.open_pdf_import_modal();
            true
        }

        // Ctrl+Shift+F: Global Search / Command Palette
        Key::Character(ref c) if (c == "f" || c == "F") && modifiers.shift() => {
            let mut s = state.write();
            s.show_command_palette = true;
            true
        }

        // Ctrl+P: Command Palette
        Key::Character(ref c) if c == "p" || c == "P" => {
            let mut s = state.write();
            s.show_command_palette = !s.show_command_palette;
            true
        }

        // Ctrl+N: New Note
        Key::Character(ref c) if c == "n" || c == "N" => {
            let mut s = state.write();
            if s.vault_service.is_some() {
                s.show_new_note_dialog = true;
            }
            true
        }

        // Ctrl+S: Save Note
        Key::Character(ref c) if c == "s" || c == "S" => {
            let mut s = state.write();
            let _ = s.save_active_note();
            true
        }

        // Ctrl+E: Toggle Reading / Edit Mode
        Key::Character(ref c) if c == "e" || c == "E" => {
            let mut s = state.write();
            s.is_reading_mode = !s.is_reading_mode;
            if s.is_reading_mode {
                s.update_toc_headings();
            }
            true
        }

        // Ctrl+G: Cycle Views (Notes -> Tasks -> Library -> Notes)
        // Ctrl+Shift+D: Open / Create Daily Note
        Key::Character(ref c) if (c == "d" || c == "D") && modifiers.shift() => {
            let mut s = state.write();
            let _ = s.open_or_create_daily_note();
            true
        }

        // Ctrl+T: Insert Template
        Key::Character(ref c) if (c == "t" || c == "T") && !modifiers.shift() => {
            let mut s = state.write();
            if s.vault_service.is_some() {
                s.show_template_modal = !s.show_template_modal;
                s.template_search_query.clear();
            }
            true
        }

        // Ctrl+W: Close active tab
        Key::Character(ref c) if c == "w" || c == "W" => {
            let mut s = state.write();
            if let Some(active_idx) = s.active_tab_index {
                let _ = s.close_tab(active_idx);
            }
            true
        }

        // Ctrl+[: Navigate back
        Key::Character(ref c) if c == "[" => {
            let mut s = state.write();
            let _ = s.navigate_back();
            true
        }

        // Ctrl+]: Navigate forward
        Key::Character(ref c) if c == "]" => {
            let mut s = state.write();
            let _ = s.navigate_forward();
            true
        }

        // Ctrl+G: Cycle Views (Notes -> Tasks -> Library -> Notes)
        Key::Character(ref c) if c == "g" || c == "G" => {
            let mut s = state.write();
            s.cycle_view();
            true
        }

        // Ctrl+\: Toggle Sidebar
        Key::Character(ref c) if c == "\\" => {
            let mut s = state.write();
            s.sidebar_open = !s.sidebar_open;
            true
        }

        // Ctrl+,: Open Settings
        Key::Character(ref c) if c == "," => {
            let mut s = state.write();
            s.open_settings();
            true
        }

        _ => false,
    }
}
