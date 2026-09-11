use dioxus::prelude::*;
use tracing::info;

use crate::components::{Dialogs, Editor, Sidebar, StatusBar};
use crate::state::AppState;
use crate::theme::BASE_CSS;

#[component]
pub fn App() -> Element {
    let mut state = use_signal(AppState::default);

    // Attempt to open the last vault on first mount
    use_effect(move || {
        let last_vault = state.read().preferences.last_vault.clone();
        if let Some(path) = last_vault {
            if path.exists() {
                info!(path = %path.display(), "Auto-opening last vault on startup");
                let mut s = state.write();
                let _ = s.open_vault(&path);
            }
        }
    });

    let app_state = state.read();
    let theme_class = app_state.theme.css_class();
    let sidebar_open = app_state.sidebar_open;
    let context_open = app_state.context_panel_open;
    let has_vault = app_state.vault_service.is_some();
    let active_note_title = app_state
        .active_note
        .as_ref()
        .map(|n| n.title.as_str())
        .unwrap_or("Nodera");

    rsx! {
        style { "{BASE_CSS}" }

        div { class: "app-container {theme_class}",
            // Top App Bar
            header { class: "top-bar",
                div { class: "top-bar-left",
                    button {
                        class: "btn-icon",
                        title: "Toggle Sidebar",
                        onclick: move |_| {
                            let mut s = state.write();
                            s.sidebar_open = !s.sidebar_open;
                        },
                        "☰"
                    }
                    span { class: "brand-title", "Nodera" }
                    if let Some(vault_path) = &app_state.vault_path {
                        span {
                            class: "vault-badge",
                            title: "{vault_path.display()}",
                            "🗄️ {app_state.vault_name}"
                        }
                    }
                }

                div { class: "top-bar-center",
                    span {
                        style: "font-weight: 500; font-size: 13px; color: var(--text-secondary);",
                        "{active_note_title}"
                    }
                }

                div { class: "top-bar-right",
                    if has_vault {
                        button {
                            class: "btn-action",
                            title: "New Note (Ctrl+N)",
                            onclick: move |_| {
                                let mut s = state.write();
                                s.show_new_note_dialog = true;
                            },
                            "➕ Note"
                        }
                    }
                    button {
                        class: "btn-icon",
                        title: "Toggle Theme",
                        onclick: move |_| {
                            let mut s = state.write();
                            s.toggle_theme();
                        },
                        if app_state.theme == crate::theme::Theme::Dark { "☀️" } else { "🌙" }
                    }
                    button {
                        class: "btn-icon",
                        title: "Toggle Context Panel",
                        onclick: move |_| {
                            let mut s = state.write();
                            s.context_panel_open = !s.context_panel_open;
                        },
                        "ℹ️"
                    }
                }
            }

            // Main Workspace (Sidebar + Editor + Context)
            div { class: "main-workspace",
                // Left pane: Sidebar / File Explorer
                if sidebar_open {
                    Sidebar { state }
                }

                // Center pane: Markdown Editor / Reading mode
                Editor { state }

                // Right pane: Contextual Panel (Backlinks / Properties)
                if context_open {
                    aside { class: "pane-context",
                        div {
                            style: "padding: 12px; border-bottom: 1px solid var(--border); font-weight: 600; font-size: 12px; text-transform: uppercase; color: var(--text-muted);",
                            "Context & Links"
                        }
                        div {
                            style: "padding: 16px; color: var(--text-muted); font-size: 12px; line-height: 1.6;",
                            p { style: "font-weight: 500; color: var(--text-secondary); margin-bottom: 6px;", "Backlinks" }
                            p { "No incoming links detected." }
                            div { style: "margin-top: 16px; border-top: 1px solid var(--border-subtle); padding-top: 12px;",
                                p { style: "font-weight: 500; color: var(--text-secondary); margin-bottom: 6px;", "Note Properties" }
                                if let Some(note) = &app_state.active_note {
                                    div { "Path: {note.relative_path.display()}" }
                                    div { "Chars: {app_state.editor_content.len()}" }
                                } else {
                                    div { "Select a note to inspect details." }
                                }
                            }
                        }
                    }
                }
            }

            // Bottom Status Bar
            StatusBar { state }

            // Modals
            Dialogs { state }
        }
    }
}
