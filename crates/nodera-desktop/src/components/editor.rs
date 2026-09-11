use dioxus::prelude::*;

use crate::state::AppState;

#[component]
pub fn Editor(state: Signal<AppState>) -> Element {
    let app_state = state.read();
    let has_note = app_state.active_note.is_some();
    let note_title = app_state
        .active_note
        .as_ref()
        .map(|n| n.title.clone())
        .unwrap_or_else(|| "No Note Selected".to_string());
    let note_path = app_state
        .active_note
        .as_ref()
        .map(|n| n.relative_path.display().to_string())
        .unwrap_or_default();
    let is_dirty = app_state.is_dirty;
    let is_reading_mode = app_state.is_reading_mode;
    let content = app_state.editor_content.clone();

    // Stats
    let words_count = content.split_whitespace().count();
    let chars_count = content.chars().count();

    rsx! {
        main { class: "pane-center",
            if !has_note {
                div {
                    style: "flex: 1; display: flex; flex-direction: column; align-items: center; justify-content: center; gap: 16px; color: var(--text-muted); padding: 40px;",
                    div { style: "font-size: 48px;", "📝" }
                    h2 { style: "font-size: 18px; font-weight: 600; color: var(--text-secondary);", "No Note Selected" }
                    p { style: "max-width: 360px; text-align: center; line-height: 1.5;",
                        "Select a note from the explorer on the left, or create a new note to begin writing."
                    }
                    if app_state.vault_service.is_some() {
                        button {
                            class: "btn-action btn-primary",
                            onclick: move |_| {
                                let mut s = state.write();
                                s.show_new_note_dialog = true;
                            },
                            "➕ Create Note (Ctrl+N)"
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
                                style: "background-color: var(--accent-focus); color: var(--accent-hover); font-size: 11px; padding: 2px 6px; border-radius: 3px;",
                                "● Unsaved"
                            }
                        } else {
                            span {
                                style: "color: var(--success); font-size: 11px;",
                                "✓ Saved"
                            }
                        }
                    }

                    div { style: "display: flex; align-items: center; gap: 8px;",
                        span { style: "font-size: 11px; color: var(--text-muted); margin-right: 8px;",
                            "{words_count} words · {chars_count} chars"
                        }
                        button {
                            class: "btn-action",
                            title: "Toggle Reading Mode (Ctrl+E)",
                            onclick: move |_| {
                                let mut s = state.write();
                                s.is_reading_mode = !s.is_reading_mode;
                            },
                            if is_reading_mode { "✏️ Edit Mode" } else { "📖 Reading Mode" }
                        }
                        button {
                            class: "btn-action btn-primary",
                            title: "Save Note (Ctrl+S)",
                            onclick: move |_| {
                                let mut s = state.write();
                                let _ = s.save_active_note();
                            },
                            "💾 Save"
                        }
                    }
                }

                // Editor Content Surface
                div {
                    style: "flex: 1; display: flex; flex-direction: column; overflow: hidden; background-color: var(--bg-app);",
                    if !is_reading_mode {
                        textarea {
                            class: "editor-textarea",
                            style: "flex: 1; width: 100%; border: none; padding: 24px 32px; font-family: var(--font-editor); font-size: 14px; line-height: 1.6; resize: none; background: transparent; outline: none; color: var(--text-primary);",
                            value: "{content}",
                            placeholder: "Start typing Markdown here...",
                            oninput: move |evt| {
                                let mut s = state.write();
                                s.update_editor_content(evt.value());
                            }
                        }
                    } else {
                        div {
                            class: "reading-view",
                            style: "flex: 1; padding: 32px 48px; overflow-y: auto; line-height: 1.7; font-size: 15px; color: var(--text-primary); max-width: 800px; margin: 0 auto; width: 100%;",
                            // Simple formatted reading view lines
                            for (idx, line) in content.lines().enumerate() {
                                {
                                    if let Some(text) = line.strip_prefix("# ") {
                                        rsx! { h1 { key: "{idx}", style: "font-size: 26px; margin: 20px 0 10px; border-bottom: 1px solid var(--border); padding-bottom: 8px;", "{text}" } }
                                    } else if let Some(text) = line.strip_prefix("## ") {
                                        rsx! { h2 { key: "{idx}", style: "font-size: 20px; margin: 18px 0 8px;", "{text}" } }
                                    } else if let Some(text) = line.strip_prefix("### ") {
                                        rsx! { h3 { key: "{idx}", style: "font-size: 16px; margin: 14px 0 6px;", "{text}" } }
                                    } else if let Some(text) = line.strip_prefix("- [ ] ") {
                                        rsx! {
                                            div { key: "{idx}", style: "display: flex; align-items: center; gap: 8px; margin: 4px 0;",
                                                input { r#type: "checkbox", disabled: true }
                                                span { "{text}" }
                                            }
                                        }
                                    } else if let Some(text) = line.strip_prefix("- [x] ").or_else(|| line.strip_prefix("- [X] ")) {
                                        rsx! {
                                            div { key: "{idx}", style: "display: flex; align-items: center; gap: 8px; margin: 4px 0; text-decoration: line-through; opacity: 0.7;",
                                                input { r#type: "checkbox", checked: true, disabled: true }
                                                span { "{text}" }
                                            }
                                        }
                                    } else if let Some(text) = line.strip_prefix("- ").or_else(|| line.strip_prefix("* ")) {
                                        rsx! { li { key: "{idx}", style: "margin-left: 20px;", "{text}" } }
                                    } else if line.is_empty() {
                                        rsx! { div { key: "{idx}", style: "height: 12px;" } }
                                    } else {
                                        rsx! { p { key: "{idx}", style: "margin: 6px 0;", "{line}" } }
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
