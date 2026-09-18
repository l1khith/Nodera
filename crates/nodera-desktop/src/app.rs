use dioxus::prelude::*;
use tracing::info;

use crate::components::{
    CitationPickerModal, CommandPalette, Dialogs, Editor, ErrorDialog, GraphView, LibraryView,
    LocalGraphView, PdfAnnotationModal, PdfImportModal, SettingsModal, Sidebar, StatusBar,
    TaskView, VaultHealthModal,
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
    let unlinked_mentions = app_state.get_unlinked_mentions();
    let related_notes = app_state.get_related_notes_for_active();

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
                    div {
                        style: "display: inline-flex; align-items: center; gap: 8px;",
                        IconNoderaLogo { size: 22 }
                        span { class: "brand-title", "{app_strings::BRAND_TITLE}" }
                    }
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
                        } else if app_state.active_view == ActiveView::Library {
                            "{nav::LIBRARY}"
                        } else if app_state.active_view == ActiveView::Graph {
                            "{nav::GRAPH}"
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
                            title: "Today's Daily Note (Ctrl+Shift+D)",
                            onclick: move |_| {
                                let mut s = state.write();
                                let _ = s.open_or_create_daily_note();
                            },
                            IconCalendar { size: 16 }
                        }
                        button {
                            class: "btn-icon",
                            title: tooltips::IMPORT_PDF,
                            onclick: move |_| {
                                let mut s = state.write();
                                s.show_pdf_import_modal = true;
                            },
                            IconImport { size: 16 }
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
                        button {
                            class: "btn-icon",
                            title: "Vault Health Doctor (Broken Links & Orphans)",
                            onclick: move |_| {
                                let mut s = state.write();
                                s.show_vault_health_modal = true;
                            },
                            IconActivity { size: 16 }
                        }
                        button {
                            class: "btn-icon",
                            title: "Citation & Bibliography Picker (Ctrl+Shift+C)",
                            onclick: move |_| {
                                let mut s = state.write();
                                s.show_citation_picker_modal = true;
                            },
                            IconQuote { size: 16 }
                        }
                        button {
                            class: "btn-icon",
                            title: "Extract PDF Annotations (Ctrl+Shift+E)",
                            onclick: move |_| {
                                let mut s = state.write();
                                s.show_pdf_annotation_modal = true;
                            },
                            IconFile { size: 16 }
                        }
                    }
                    button {
                        class: "btn-icon",
                        title: tooltips::SETTINGS,
                        onclick: move |_| {
                            let mut s = state.write();
                            s.open_settings();
                        },
                        IconSettings { size: 16 }
                    }
                    button {
                        class: "btn-icon",
                        title: tooltips::TOGGLE_THEME,
                        onclick: move |_| {
                            let mut s = state.write();
                            s.toggle_theme();
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
                            let mut s = state.write();
                            s.context_panel_open = !s.context_panel_open;
                        },
                        IconList { size: 16 }
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

                // Center pane: Markdown Editor, Global Tasks View, Library View, or Graph View
                match app_state.active_view {
                    ActiveView::Editor => rsx! { Editor { state } },
                    ActiveView::Tasks => rsx! { TaskView { state } },
                    ActiveView::Library => rsx! { LibraryView { state } },
                    ActiveView::Graph => rsx! { GraphView { state } },
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

                            // Local 2D Graph section
                            div {
                                p { style: "font-weight: 600; color: var(--text-secondary); margin-bottom: 6px;", "{nav::GRAPH}" }
                                LocalGraphView { state }
                            }

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

                            // Unlinked Mentions section
                            div {
                                p { style: "font-weight: 600; color: var(--text-secondary); margin-bottom: 6px;", "Unlinked Mentions ({unlinked_mentions.len()})" }
                                if unlinked_mentions.is_empty() {
                                    p { style: "font-style: italic; color: var(--text-muted);", "No unlinked mentions found." }
                                } else {
                                    div { style: "display: flex; flex-direction: column; gap: 8px;",
                                        for mention in unlinked_mentions.iter() {
                                            {
                                                let src_path = mention.source_path.clone();
                                                let src_click = src_path.clone();
                                                let matched = mention.matched_text.clone();
                                                let title = mention.source_title.clone();
                                                let before = mention.snippet_before.clone();
                                                let after = mention.snippet_after.clone();
                                                let active_t = app_state.active_note.as_ref().map(|n| n.title.clone()).unwrap_or_default();

                                                rsx! {
                                                    div {
                                                        key: "{mention.source_path.display()}_{mention.matched_text}_{before}",
                                                        class: "unlinked-mention-card",
                                                        div { style: "display: flex; align-items: center; justify-content: space-between; margin-bottom: 4px;",
                                                            button {
                                                                class: "link-item",
                                                                style: "padding: 2px 6px; font-weight: 600;",
                                                                onclick: move |_| {
                                                                    let mut s = state.write();
                                                                    let _ = s.select_note(&src_click);
                                                                },
                                                                "{title}"
                                                            }
                                                            button {
                                                                class: "btn-action btn-primary",
                                                                style: "font-size: 11px; padding: 2px 8px;",
                                                                title: "Convert mention into [[wikilink]]",
                                                                onclick: move |_| {
                                                                    let mut s = state.write();
                                                                    let _ = s.link_unlinked_mention(&src_path, &active_t);
                                                                },
                                                                "Link"
                                                            }
                                                        }
                                                        div { class: "unlinked-snippet",
                                                            span { style: "color: var(--text-muted);", "...{before}" }
                                                            span { style: "background: var(--accent-focus); color: var(--accent-hover); font-weight: 600; padding: 0 2px; border-radius: 2px;", "{matched}" }
                                                            span { style: "color: var(--text-muted);", "{after}..." }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }

                            // Related Notes section (Lexical & Semantic recommendations)
                            if !related_notes.is_empty() {
                                div {
                                    p { style: "font-weight: 600; color: var(--text-secondary); margin-bottom: 6px; display: flex; align-items: center; justify-content: space-between;",
                                        span { "Related Notes ({related_notes.len()})" }
                                    }
                                    div { style: "display: flex; flex-direction: column; gap: 6px;",
                                        for rel in related_notes.iter() {
                                            {
                                                let rel_path = std::path::PathBuf::from(&rel.path);
                                                let rel_path_click = rel_path.clone();
                                                let rel_title = rel.title.clone();
                                                let rel_title_link = rel.title.clone();
                                                let match_pct = rel.match_percentage;
                                                let shared_tags = rel.shared_tags.clone();
                                                let snippet = rel.snippet.clone();

                                                rsx! {
                                                    div {
                                                        key: "{rel.path}",
                                                        class: "unlinked-mention-card",
                                                        style: "display: flex; flex-direction: column; gap: 4px; padding: 8px 10px; background: var(--bg-surface-elevated, rgba(255, 255, 255, 0.02)); border: 1px solid var(--border); border-radius: 6px;",
                                                        div {
                                                            style: "display: flex; align-items: center; justify-content: space-between;",
                                                            button {
                                                                class: "link-item",
                                                                style: "padding: 0; font-weight: 600; text-align: left; font-size: 12px;",
                                                                onclick: move |_| {
                                                                    let mut s = state.write();
                                                                    let _ = s.select_note(&rel_path_click);
                                                                },
                                                                "{rel_title}"
                                                            }
                                                            div { style: "display: flex; align-items: center; gap: 6px;",
                                                                span {
                                                                    style: "font-size: 10px; font-weight: 600; background: rgba(16, 185, 129, 0.15); color: #10B981; padding: 1px 6px; border-radius: 8px;",
                                                                    "{match_pct}%"
                                                                }
                                                                button {
                                                                    class: "btn-action btn-primary",
                                                                    style: "font-size: 10px; padding: 2px 6px;",
                                                                    title: "Add wikilink to this note",
                                                                    onclick: move |_| {
                                                                        let mut s = state.write();
                                                                        let _ = s.append_link_to_active_note(&rel_title_link);
                                                                    },
                                                                    "+ Link"
                                                                }
                                                            }
                                                        }
                                                        if !shared_tags.is_empty() {
                                                            div { style: "display: flex; flex-wrap: wrap; gap: 3px; margin-top: 2px;",
                                                                for t in shared_tags.iter() {
                                                                    span {
                                                                        key: "{t}",
                                                                        style: "font-size: 10px; color: var(--accent); background: rgba(91, 108, 255, 0.1); padding: 0 4px; border-radius: 3px;",
                                                                        "#{t}"
                                                                    }
                                                                }
                                                            }
                                                        }
                                                        if !snippet.is_empty() {
                                                            div {
                                                                style: "font-size: 11px; color: var(--text-muted); line-height: 1.3; overflow: hidden; text-overflow: ellipsis; display: -webkit-box; -webkit-line-clamp: 2; -webkit-box-orient: vertical;",
                                                                "{snippet}"
                                                            }
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
            VaultHealthModal { state }
            CitationPickerModal { state }
            PdfAnnotationModal { state }
        }
    }
}
