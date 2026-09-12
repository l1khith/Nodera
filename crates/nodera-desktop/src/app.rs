use dioxus::prelude::*;
use tracing::info;

use crate::components::{
    CommandPalette, Dialogs, Editor, ErrorDialog, LibraryView, PdfImportModal, SettingsModal,
    Sidebar, StatusBar, TaskView,
};
use crate::icons::*;
use crate::state::{ActiveView, AppState};
use crate::strings::{actions, app as app_strings, nav, tooltips};
use crate::theme::BASE_CSS;

#[component]
pub fn App() -> Element {
    let mut state = use_signal(|| {
        let mut app_state = AppState::default();
        if let Some(path) = app_state.preferences.last_vault.clone() {
            if path.exists() {
                info!(path = %path.display(), "Auto-opening last vault on startup");
                let _ = app_state.open_vault(&path);
            }
        }
        app_state
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
        .unwrap_or(app_strings::BRAND_TITLE);

    let backlinks = app_state.get_current_backlinks();
    let outgoing_links = app_state.get_current_outgoing_links();
    let current_tags = app_state.get_current_note_tags();

    rsx! {
        style { "{BASE_CSS}" }

        div {
            class: "app-container {theme_class}",
            tabindex: "0",
            onkeydown: move |evt: KeyboardEvent| {
                let mut s = state;
                crate::shortcuts::handle_global_shortcut(&evt, &mut s);
            },

            // Top App Bar
            header { class: "top-bar",
                div { class: "top-bar-left",
                    button {
                        class: "btn-icon",
                        title: tooltips::TOGGLE_SIDEBAR,
                        onclick: move |_| {
                            let mut s = state.write();
                            s.sidebar_open = !s.sidebar_open;
                        },
                        IconMenu { size: 16 }
                    }
                    span { class: "brand-title", "{app_strings::BRAND_TITLE}" }
                    if let Some(vault_path) = &app_state.vault_path {
                        span {
                            class: "vault-badge",
                            style: "display: inline-flex; align-items: center; gap: 5px;",
                            title: "{vault_path.display()}",
                            IconVault { size: 13 }
                            span { "{app_state.vault_name}" }
                        }
                    }
                }

                div { class: "top-bar-center",
                    span {
                        style: "font-weight: 500; font-size: 13px; color: var(--text-secondary);",
                        if app_state.active_view == ActiveView::Tasks {
                            "{nav::GLOBAL_TASKS}"
                        } else {
                            "{active_note_title}"
                        }
                    }
                }

                div { class: "top-bar-right",
                    if has_vault {
                        button {
                            class: "btn-action",
                            title: tooltips::PALETTE,
                            onclick: move |_| {
                                let mut s = state.write();
                                s.show_command_palette = true;
                            },
                            IconSearch { size: 14 }
                            span { "{actions::PALETTE}" }
                        }
                        button {
                            class: "btn-action",
                            title: tooltips::NEW_NOTE,
                            onclick: move |_| {
                                let mut s = state.write();
                                s.show_new_note_dialog = true;
                            },
                            IconPlus { size: 14 }
                            span { "{actions::NOTE_BTN}" }
                        }
                        button {
                            class: "btn-icon",
                            title: tooltips::REBUILD_INDEX,
                            onclick: move |_| {
                                let mut s = state.write();
                                let _ = s.rebuild_vault_index();
                            },
                            IconRefresh { size: 16 }
                        }
                    }
                    button {
                        class: "btn-icon",
                        title: tooltips::SETTINGS,
                        onclick: move |_| {
                            state.write().open_settings();
                        },
                        IconSettings { size: 16 }
                    }
                    button {
                        class: "btn-icon",
                        title: tooltips::TOGGLE_THEME,
                        onclick: move |_| {
                            state.write().toggle_theme();
                        },
                        if app_state.theme == crate::theme::Theme::Dark {
                            IconSun { size: 16 }
                        } else {
                            IconMoon { size: 16 }
                        }
                    }
                    button {
                        class: "btn-icon",
                        title: tooltips::TOGGLE_CONTEXT,
                        onclick: move |_| {
                            let cur = state.read().context_panel_open;
                            state.write().context_panel_open = !cur;
                        },
                        IconInfo { size: 16 }
                    }
                }
            }

            // Main Workspace (Sidebar + Center View + Context)
            div { class: "main-workspace",
                // Left pane: Sidebar / File Explorer
                if sidebar_open {
                    div {
                        style: format!("width: {}px; flex-shrink: 0; display: flex; overflow: hidden;", app_state.sidebar_width),
                        Sidebar { state }
                    }
                    div {
                        class: "pane-resizer",
                        title: tooltips::RESET_SIDEBAR,
                        ondoubleclick: move |_| {
                            state.write().reset_layout();
                        }
                    }
                }

                // Center pane: Markdown Editor, Global Tasks View, or Library View
                match app_state.active_view {
                    ActiveView::Editor => rsx! { Editor { state } },
                    ActiveView::Tasks => rsx! { TaskView { state } },
                    ActiveView::Library => rsx! { LibraryView { state } },
                }

                // Right pane: Contextual Panel (Backlinks / Properties)
                if context_open {
                    div {
                        class: "pane-resizer",
                        title: tooltips::RESET_CONTEXT,
                        ondoubleclick: move |_| {
                            state.write().reset_layout();
                        }
                    }
                    aside {
                        class: "pane-context",
                        style: format!("width: {}px; flex-shrink: 0;", app_state.context_panel_width),
                        div {
                            style: "padding: 12px; border-bottom: 1px solid var(--border); font-weight: 600; font-size: 12px; text-transform: uppercase; color: var(--text-muted);",
                            "{app_strings::CONTEXT_PANEL_TITLE}"
                        }
                        div {
                            style: "flex: 1; overflow-y: auto; padding: 16px; color: var(--text-muted); font-size: 12px; line-height: 1.6; display: flex; flex-direction: column; gap: 16px;",

                            // Backlinks section
                            div {
                                p { style: "font-weight: 600; color: var(--text-secondary); margin-bottom: 6px;", "{app_strings::INCOMING_LINKS} ({backlinks.len()})" }
                                if backlinks.is_empty() {
                                    p { style: "font-style: italic; color: var(--text-muted);", "{crate::strings::empty_states::NO_BACKLINKS}" }
                                } else {
                                    div { class: "link-list",
                                        for backlink in backlinks.iter() {
                                            {
                                                let p = backlink.clone();
                                                let p_click = p.clone();
                                                let display_title = p.file_stem().and_then(|s| s.to_str()).unwrap_or("Note").to_string();
                                                rsx! {
                                                    button {
                                                        key: "{p.display()}",
                                                        class: "link-item",
                                                        title: "{p.display()}",
                                                        onclick: move |_| {
                                                            let mut s = state.write();
                                                            let _ = s.select_note(&p_click);
                                                        },
                                                        span {
                                                            style: "display: inline-flex; align-items: center; gap: 6px;",
                                                            IconLink { size: 12 }
                                                            span { "{display_title}" }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }

                            // Outgoing links section
                            div {
                                p { style: "font-weight: 600; color: var(--text-secondary); margin-bottom: 6px;", "{app_strings::OUTGOING_LINKS} ({outgoing_links.len()})" }
                                if outgoing_links.is_empty() {
                                    p { style: "font-style: italic; color: var(--text-muted);", "{crate::strings::empty_states::NO_OUTGOING_LINKS}" }
                                } else {
                                    div { class: "link-list",
                                        for (link, resolved) in outgoing_links.iter() {
                                            {
                                                let target = link.target.clone();
                                                let target_click = target.clone();
                                                let label = link.label().to_string();
                                                let is_resolved = resolved.is_some();
                                                rsx! {
                                                    button {
                                                        key: "{target}",
                                                        class: if is_resolved { "link-item" } else { "link-item link-item-unresolved" },
                                                        title: if is_resolved { format!("Open '{target}'") } else { format!("Create '{target}'") },
                                                        onclick: move |_| {
                                                            let mut s = state.write();
                                                            let _ = s.open_or_create_target(&target_click);
                                                        },
                                                        span {
                                                            style: "display: inline-flex; align-items: center; gap: 6px;",
                                                            if is_resolved {
                                                                IconExternalLink { size: 12 }
                                                            } else {
                                                                IconPlus { size: 12 }
                                                            }
                                                            span { "{label}" }
                                                        }
                                                        if !is_resolved {
                                                            span { style: "font-size: 10px; opacity: 0.7;", "new" }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }

                            // Tags section
                            if !current_tags.is_empty() {
                                div {
                                    p { style: "font-weight: 600; color: var(--text-secondary); margin-bottom: 6px;", "{app_strings::TAGS} ({current_tags.len()})" }
                                    div { style: "display: flex; flex-wrap: wrap; gap: 4px;",
                                        for tag in current_tags.iter() {
                                            span { key: "{tag}", class: "tag-badge", "#{tag}" }
                                        }
                                    }
                                }
                            }

                            // Properties section
                            div { style: "border-top: 1px solid var(--border-subtle); padding-top: 12px;",
                                p { style: "font-weight: 600; color: var(--text-secondary); margin-bottom: 6px;", "{app_strings::NOTE_PROPERTIES}" }
                                if let Some(note) = &app_state.active_note {
                                    div { "Path: {note.relative_path.display()}" }
                                    div { "Words: {app_state.editor_content.split_whitespace().count()}" }
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
            CommandPalette { state }
            PdfImportModal { state }
            SettingsModal { state }
            ErrorDialog { state }
        }
    }
}
