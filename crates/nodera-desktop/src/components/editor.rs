use dioxus::prelude::*;

use crate::icons::{IconBook, IconCheck, IconEdit, IconList, IconNotes, IconPlus, IconSave};
use crate::state::AppState;
use crate::strings::{actions, app as app_strings, empty_states, placeholders, tooltips};

#[component]
pub fn Editor(state: Signal<AppState>) -> Element {
    let app_state = state.read();
    let has_note = app_state.active_note.is_some();
    let note_title = app_state
        .active_note
        .as_ref()
        .map(|n| n.title.clone())
        .unwrap_or_else(|| empty_states::NO_NOTE_SELECTED_TITLE.to_string());
    let note_path = app_state
        .active_note
        .as_ref()
        .map(|n| n.relative_path.display().to_string())
        .unwrap_or_default();
    let is_dirty = app_state.is_dirty;
    let is_reading_mode = app_state.is_reading_mode;
    let content = app_state.editor_content.clone();

    // Markdown HTML rendering for reading mode
    let rendered_html = if is_reading_mode {
        nodera_markdown::render_to_html(&content)
    } else {
        String::new()
    };

    // Stats
    let words_count = content.split_whitespace().count();
    let chars_count = content.chars().count();

    rsx! {
        main { class: "pane-center",
            if !has_note {
                div {
                    style: "flex: 1; display: flex; flex-direction: column; align-items: center; justify-content: center; gap: 16px; color: var(--text-muted); padding: 40px;",
                    IconNotes { size: 48 }
                    h2 { style: "font-size: 18px; font-weight: 600; color: var(--text-secondary);", "{empty_states::NO_NOTE_SELECTED_TITLE}" }
                    p { style: "max-width: 360px; text-align: center; line-height: 1.5;",
                        "{empty_states::NO_NOTE_SELECTED_DESC}"
                    }
                    if app_state.vault_service.is_some() {
                        button {
                            class: "btn-action btn-primary",
                            onclick: move |_| {
                                let mut s = state.write();
                                s.show_new_note_dialog = true;
                            },
                            IconPlus { size: 14 }
                            span { "{actions::CREATE_NOTE}" }
                        }
                    }
                    div {
                        style: "margin-top: 24px; padding: 16px; border: 1px solid var(--border); border-radius: 6px; background-color: var(--bg-surface); font-size: 12px; display: flex; flex-direction: column; gap: 6px;",
                        div { style: "font-weight: 600; margin-bottom: 4px; color: var(--text-primary);", "Keyboard Shortcuts" }
                        div { "Ctrl + N : New Note" }
                        div { "Ctrl + S : Save Note" }
                        div { "Ctrl + E : Toggle Reading / Edit Mode" }
                    }
                }
            } else {
                // Active Note Toolbar
                div {
                    style: "height: 44px; border-bottom: 1px solid var(--border); background-color: var(--bg-surface); display: flex; align-items: center; justify-content: space-between; padding: 0 16px;",
                    div { style: "display: flex; align-items: center; gap: 10px;",
                        span { style: "font-size: 15px; font-weight: 600; color: var(--text-primary);",
                            "{note_title}"
                        }
                        span { style: "font-size: 11px; color: var(--text-muted);",
                            "({note_path})"
                        }
                        if is_dirty {
                            span {
                                style: "background-color: var(--accent-focus); color: var(--accent-hover); font-size: 11px; padding: 2px 6px; border-radius: 3px; display: inline-flex; align-items: center; gap: 5px;",
                                span { style: "width: 6px; height: 6px; border-radius: 50%; background-color: currentColor; display: inline-block;" }
                                span { "{actions::UNSAVED}" }
                            }
                        } else {
                            span {
                                style: "color: var(--success); font-size: 11px; display: inline-flex; align-items: center; gap: 4px;",
                                IconCheck { size: 12 }
                                span { "{actions::SAVED}" }
                            }
                        }
                    }

                    div { style: "display: flex; align-items: center; gap: 8px;",
                        span { style: "font-size: 11px; color: var(--text-muted); margin-right: 8px;",
                            "{words_count} words · {chars_count} chars"
                        }
                        button {
                            class: "btn-action",
                            title: tooltips::TOGGLE_READING,
                            onclick: move |_| {
                                let mut s = state.write();
                                s.is_reading_mode = !s.is_reading_mode;
                            },
                            if is_reading_mode {
                                IconEdit { size: 14 }
                                span { "{actions::EDIT_MODE}" }
                            } else {
                                IconBook { size: 14 }
                                span { "{actions::READING_MODE}" }
                            }
                        }
                        button {
                            class: "btn-action btn-primary",
                            title: tooltips::SAVE_NOTE,
                            onclick: move |_| {
                                let mut s = state.write();
                                let _ = s.save_active_note();
                            },
                            IconSave { size: 14 }
                            span { "{actions::SAVE}" }
                        }
                    }
                }

                // Editor / Reader Content Surface
                div {
                    style: "flex: 1; display: flex; overflow: hidden; background-color: var(--bg-app); position: relative;",

                    if !is_reading_mode {
                        textarea {
                            class: "editor-textarea",
                            style: format!("flex: 1; width: 100%; border: none; padding: 24px 32px; font-family: var(--font-editor); font-size: {}px; line-height: 1.6; resize: none; background: transparent; outline: none; color: var(--text-primary);", app_state.preferences.editor_font_size),
                            value: "{content}",
                            placeholder: placeholders::TYPE_MARKDOWN,
                            oninput: move |evt| {
                                let mut s = state.write();
                                s.update_editor_content(evt.value());
                            }
                        }
                    } else {
                        // Reading Mode Layout: Optional Table of Contents + Reading Document
                        div {
                            style: "flex: 1; display: flex; height: 100%; overflow: hidden;",

                            // Floating or side Table of Contents if headings exist
                            if !app_state.toc_headings.is_empty() {
                                div {
                                    class: "reading-toc",
                                    style: "width: 220px; border-right: 1px solid var(--border-color); background: var(--bg-secondary); overflow-y: auto; padding: 16px 12px; font-size: 12px; display: flex; flex-direction: column; gap: 6px; flex-shrink: 0;",

                                    div {
                                        style: "font-weight: 600; text-transform: uppercase; font-size: 11px; letter-spacing: 0.5px; color: var(--text-muted); margin-bottom: 6px; display: flex; align-items: center; gap: 6px;",
                                        IconList { size: 14 }
                                        span { "{app_strings::CONTENTS}" }
                                    }

                                    for (depth, heading_text) in &app_state.toc_headings {
                                        div {
                                            key: "{heading_text}",
                                            style: format!("padding: 4px 8px; border-radius: 4px; color: var(--text-secondary); white-space: nowrap; overflow: hidden; text-overflow: ellipsis; padding-left: {}px; cursor: pointer;", (depth - 1) * 12 + 8),
                                            title: "{heading_text}",
                                            "{heading_text}"
                                        }
                                    }
                                }
                            }

                            // Reading Document Body
                            div {
                                style: "flex: 1; display: flex; flex-direction: column; height: 100%; overflow: hidden;",

                                div {
                                    class: "reading-view markdown-body",
                                    style: format!("flex: 1; padding: 40px 60px; overflow-y: auto; line-height: 1.8; font-size: {}px; color: var(--text-primary); max-width: 780px; margin: 0 auto; width: 100%; font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Helvetica, Arial, sans-serif;", app_state.preferences.reading_font_size),
                                    dangerous_inner_html: "{rendered_html}"
                                }

                                // Reading Progress Footer Bar
                                if let Some(active) = &app_state.active_note {
                                    {
                                        let path_for_input = active.relative_path.clone();
                                        let path_for_finish = active.relative_path.clone();
                                        let cur_progress = app_state.preferences.reading_progress.get(&path_for_input.to_string_lossy().to_string()).copied().unwrap_or(0);

                                        rsx! {
                                            div {
                                                style: "display: flex; align-items: center; justify-content: space-between; padding: 8px 24px; border-top: 1px solid var(--border-color); background: var(--bg-secondary); font-size: 12px; color: var(--text-muted);",

                                                div {
                                                    style: "display: flex; align-items: center; gap: 12px; flex: 1; max-width: 360px;",
                                                    span { "Reading progress:" }
                                                    input {
                                                        r#type: "range",
                                                        min: "0",
                                                        max: "100",
                                                        value: "{cur_progress}",
                                                        style: "flex: 1; cursor: pointer;",
                                                        oninput: move |evt| {
                                                            if let Ok(val) = evt.value().parse::<u32>() {
                                                                state.write().set_reading_progress(&path_for_input, val);
                                                            }
                                                        }
                                                    }
                                                    span { style: "font-weight: 500; min-width: 32px;", "{cur_progress}%" }
                                                }

                                                div {
                                                    style: "display: flex; gap: 8px;",
                                                    button {
                                                        class: "btn-action",
                                                        style: "padding: 3px 8px; font-size: 11px;",
                                                        onclick: move |_| {
                                                            state.write().set_reading_progress(&path_for_finish, 100);
                                                        },
                                                        "Mark Finished"
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
