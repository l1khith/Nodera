use dioxus::prelude::*;
use tracing::info;

use crate::components::{
    CitationPickerModal, CommandPalette, Dialogs, Editor, ErrorDialog, GraphView, Inspector,
    LibraryView, PdfAnnotationModal, PdfImportModal, QuickCaptureModal, ReviewQueueView,
    SettingsModal, Sidebar, StatusBar, TaskView, TodayView, VaultHealthModal,
};
use crate::icons::*;
use crate::state::{ActiveView, AppState};
use crate::strings::{app as app_strings, tooltips};
use crate::theme::BASE_CSS;

#[component]
pub fn App() -> Element {
    let mut state = use_signal(|| {
        let mut app_state = AppState::default();
        if let Some(path) = app_state.preferences.last_vault.clone() {
            if path.exists() {
                info!(path = %path.display(), "Auto-opening last vault on startup");
                if let Err(e) = app_state.open_vault(&path) {
                    tracing::warn!("Could not auto-open last vault {}: {e}", path.display());
                    app_state.vault_service = None;
                    app_state.vault_path = None;
                }
            }
        }
        app_state
    });

    let mut show_vault_dropdown = use_signal(|| false);
    let mut show_more_dropdown = use_signal(|| false);
    let mut show_new_dropdown = use_signal(|| false);

    let app_state = state.read();
    let theme_class = app_state.theme.css_class();
    let sidebar_open = app_state.sidebar_open;
    let context_open = app_state.context_panel_open;
    let has_vault = app_state.vault_service.is_some();

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
                        style: "display: inline-flex; align-items: center;",
                        title: "{app_strings::BRAND_TITLE}",
                        IconNoderaLogo { size: 22 }
                    }

                    // [ Vault ▾ ] Selector Dropdown
                    div { style: "position: relative; display: inline-flex;",
                        button {
                            class: "vault-selector-btn",
                            title: "Vault options",
                            onclick: move |_| {
                                show_vault_dropdown.toggle();
                                show_more_dropdown.set(false);
                                show_new_dropdown.set(false);
                            },
                            IconVault { size: 13 }
                            span { "{app_state.vault_name}" }
                            IconChevronDown { size: 11, class: "opacity-60" }
                        }

                        if *show_vault_dropdown.read() {
                            div {
                                class: "dropdown-menu",
                                style: "left: 0; min-width: 240px;",
                                if let Some(vault_path) = &app_state.vault_path {
                                    div { class: "dropdown-header",
                                        div { style: "font-weight: 600; color: var(--text-primary);", "{app_state.vault_name}" }
                                        div { style: "font-size: 10px; color: var(--text-muted); text-transform: none; overflow: hidden; text-overflow: ellipsis; white-space: nowrap;", "{vault_path.display()}" }
                                    }
                                    div { class: "dropdown-divider" }
                                }
                                if !app_state.preferences.recent_vaults.is_empty() {
                                    div { class: "dropdown-header", "Recent Vaults" }
                                    for rv in app_state.preferences.recent_vaults.iter().take(5) {
                                        {
                                            let rv_path = rv.clone();
                                            let rv_name = rv.file_name().and_then(|s| s.to_str()).unwrap_or("Vault").to_string();
                                            let is_cur = app_state.vault_path.as_ref() == Some(&rv_path);
                                            rsx! {
                                                button {
                                                    key: "{rv.display()}",
                                                    class: "dropdown-item",
                                                    style: if is_cur { "font-weight: 600; color: var(--accent);" } else { "" },
                                                    title: "{rv.display()}",
                                                    onclick: move |_| {
                                                        show_vault_dropdown.set(false);
                                                        let mut s = state.write();
                                                        let _ = s.open_vault(&rv_path);
                                                    },
                                                    IconVault { size: 13 }
                                                    span { style: "flex: 1; overflow: hidden; text-overflow: ellipsis; white-space: nowrap;", "{rv_name}" }
                                                    if is_cur {
                                                        span { style: "font-size: 10px; color: var(--accent);", "Active" }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                    div { class: "dropdown-divider" }
                                }
                                button {
                                    class: "dropdown-item",
                                    onclick: move |_| {
                                        show_vault_dropdown.set(false);
                                        spawn(async move {
                                            if let Some(folder) = rfd::AsyncFileDialog::new().pick_folder().await {
                                                let path = folder.path().to_path_buf();
                                                let mut s = state.write();
                                                let _ = s.open_vault(path);
                                            }
                                        });
                                    },
                                    IconFolderOpen { size: 14 }
                                    span { "Open Vault…" }
                                }
                                button {
                                    class: "dropdown-item",
                                    onclick: move |_| {
                                        show_vault_dropdown.set(false);
                                        spawn(async move {
                                            if let Some(folder) = rfd::AsyncFileDialog::new().pick_folder().await {
                                                let path = folder.path().to_path_buf();
                                                let mut s = state.write();
                                                let _ = s.create_vault(path, None);
                                            }
                                        });
                                    },
                                    IconFolderPlus { size: 14 }
                                    span { "New Vault…" }
                                }
                                if has_vault {
                                    div { class: "dropdown-divider" }
                                    button {
                                        class: "dropdown-item danger",
                                        onclick: move |_| {
                                            show_vault_dropdown.set(false);
                                            let mut s = state.write();
                                            let _ = s.close_vault();
                                        },
                                        IconClose { size: 14 }
                                        span { "Close Vault" }
                                    }
                                }
                            }
                        }
                    }
                }

                // Center: Search Notes button
                div { class: "top-bar-center",
                    button {
                        class: "top-search-btn",
                        title: "Search notes (Ctrl+P)",
                        onclick: move |_| {
                            state.write().show_command_palette = true;
                        },
                        IconSearch { size: 13 }
                        span { "Search notes…" }
                        span { class: "kbd-badge", "Ctrl+P" }
                    }
                }

                // Right: Primary Actions + ⋯ More + Inspector Toggle
                div { class: "top-bar-right",
                    if has_vault {
                        // [ + New ▾ ] Note Creation & Quick Capture Dropdown
                        div { style: "position: relative; display: inline-flex;",
                            button {
                                class: "btn-action",
                                title: "Create note or capture thought",
                                onclick: move |_| {
                                    show_new_dropdown.toggle();
                                    show_vault_dropdown.set(false);
                                    show_more_dropdown.set(false);
                                },
                                IconPlus { size: 13 }
                                span { "New" }
                                IconChevronDown { size: 11, class: "opacity-60" }
                            }

                            if *show_new_dropdown.read() {
                                div {
                                    class: "dropdown-menu",
                                    style: "left: 0; min-width: 240px;",
                                    button {
                                        class: "dropdown-item",
                                        onclick: move |_| {
                                            show_new_dropdown.set(false);
                                            state.write().open_quick_capture();
                                        },
                                        IconRough { size: 14 }
                                        span { "Quick Capture" }
                                        span { class: "kbd-badge", style: "margin-left: auto;", "Ctrl+Shift+Q" }
                                    }
                                    div { class: "dropdown-divider" }
                                    button {
                                        class: "dropdown-item",
                                        onclick: move |_| {
                                            show_new_dropdown.set(false);
                                            let _ = state.write().create_typed_note(nodera_markdown::NOTE_TYPE_PERMANENT, "", None);
                                        },
                                        IconPermanent { size: 14 }
                                        span { "Permanent Note" }
                                    }
                                    button {
                                        class: "dropdown-item",
                                        onclick: move |_| {
                                            show_new_dropdown.set(false);
                                            let _ = state.write().create_typed_note(nodera_markdown::NOTE_TYPE_ROUGH, "", None);
                                        },
                                        IconRough { size: 14 }
                                        span { "Rough Note" }
                                    }
                                    button {
                                        class: "dropdown-item",
                                        onclick: move |_| {
                                            show_new_dropdown.set(false);
                                            let _ = state.write().create_typed_note(nodera_markdown::NOTE_TYPE_SOURCE, "", Some("book"));
                                        },
                                        IconSource { size: 14 }
                                        span { "Source Note (Book)" }
                                    }
                                    button {
                                        class: "dropdown-item",
                                        onclick: move |_| {
                                            show_new_dropdown.set(false);
                                            let _ = state.write().create_typed_note(nodera_markdown::NOTE_TYPE_SOURCE, "", Some("video"));
                                        },
                                        IconSource { size: 14 }
                                        span { "Source Note (Video)" }
                                    }
                                    button {
                                        class: "dropdown-item",
                                        onclick: move |_| {
                                            show_new_dropdown.set(false);
                                            let _ = state.write().create_typed_note(nodera_markdown::NOTE_TYPE_INDEX, "", None);
                                        },
                                        IconIndex { size: 14 }
                                        span { "Index / MOC Note" }
                                    }
                                    button {
                                        class: "dropdown-item",
                                        onclick: move |_| {
                                            show_new_dropdown.set(false);
                                            let _ = state.write().create_typed_note(nodera_markdown::NOTE_TYPE_PROJECT, "", None);
                                        },
                                        IconFolderPlus { size: 14 }
                                        span { "Project Note" }
                                    }
                                    button {
                                        class: "dropdown-item",
                                        onclick: move |_| {
                                            show_new_dropdown.set(false);
                                            let _ = state.write().create_typed_note(nodera_markdown::NOTE_TYPE_MEETING, "", None);
                                        },
                                        IconCalendar { size: 14 }
                                        span { "Meeting Note" }
                                    }
                                    div { class: "dropdown-divider" }
                                    button {
                                        class: "dropdown-item",
                                        onclick: move |_| {
                                            show_new_dropdown.set(false);
                                            let mut s = state.write();
                                            s.show_new_note_dialog = true;
                                        },
                                        IconFile { size: 14 }
                                        span { "Blank Note…" }
                                    }
                                }
                            }
                        }
                        button {
                            class: "btn-action",
                            title: "Today's Daily Note (Ctrl+Shift+D)",
                            onclick: move |_| {
                                let mut s = state.write();
                                let _ = s.open_or_create_daily_note();
                            },
                            IconCalendar { size: 13 }
                            span { "Daily" }
                        }
                        button {
                            class: "btn-action",
                            title: "Import Markdown or PDF",
                            onclick: move |_| {
                                let mut s = state.write();
                                s.show_pdf_import_modal = true;
                            },
                            IconImport { size: 13 }
                            span { "Import" }
                        }

                        // ⋯ More Dropdown
                        div { style: "position: relative; display: inline-flex;",
                            button {
                                class: "btn-icon",
                                title: "More tools",
                                onclick: move |_| {
                                    show_more_dropdown.toggle();
                                    show_vault_dropdown.set(false);
                                    show_new_dropdown.set(false);
                                },
                                IconMore { size: 16 }
                            }

                            if *show_more_dropdown.read() {
                                div {
                                    class: "dropdown-menu",
                                    style: "right: 0; min-width: 210px;",
                                    button {
                                        class: "dropdown-item",
                                        onclick: move |_| {
                                            show_more_dropdown.set(false);
                                            let mut s = state.write();
                                            let _ = s.rebuild_vault_index();
                                        },
                                        IconRefresh { size: 14 }
                                        span { "Rebuild Search Index" }
                                    }
                                    button {
                                        class: "dropdown-item",
                                        onclick: move |_| {
                                            show_more_dropdown.set(false);
                                            state.write().show_vault_health_modal = true;
                                        },
                                        IconActivity { size: 14 }
                                        span { "Vault Health Doctor" }
                                    }
                                    button {
                                        class: "dropdown-item",
                                        onclick: move |_| {
                                            show_more_dropdown.set(false);
                                            state.write().show_citation_picker_modal = true;
                                        },
                                        IconQuote { size: 14 }
                                        span { "Citation Picker" }
                                    }
                                    button {
                                        class: "dropdown-item",
                                        onclick: move |_| {
                                            show_more_dropdown.set(false);
                                            state.write().show_pdf_annotation_modal = true;
                                        },
                                        IconFile { size: 14 }
                                        span { "PDF Annotations" }
                                    }
                                    div { class: "dropdown-divider" }
                                    button {
                                        class: "dropdown-item",
                                        onclick: move |_| {
                                            show_more_dropdown.set(false);
                                            state.write().open_settings();
                                        },
                                        IconSettings { size: 14 }
                                        span { "{tooltips::SETTINGS}" }
                                    }
                                    button {
                                        class: "dropdown-item",
                                        onclick: move |_| {
                                            show_more_dropdown.set(false);
                                            state.write().toggle_theme();
                                        },
                                        if app_state.theme.is_dark() {
                                            IconSun { size: 14 }
                                            span { "Switch to Light Theme" }
                                        } else {
                                            IconMoon { size: 14 }
                                            span { "Switch to Dark Theme" }
                                        }
                                    }
                                }
                            }
                        }
                    }

                    // Inspector Toggle
                    button {
                        class: if context_open { "btn-icon active-toggle" } else { "btn-icon" },
                        style: if context_open { "color: var(--accent);" } else { "" },
                        title: "Toggle Inspector (Ctrl+I)",
                        onclick: move |_| {
                            let mut s = state.write();
                            s.context_panel_open = !s.context_panel_open;
                        },
                        IconPanelRight { size: 16 }
                    }
                }
            }

            // Main Workspace (Sidebar + Center View + Contextual Inspector)
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

                // Center pane: Markdown Editor, Today View, Global Tasks View, Review Queue, Library View, or Graph View
                match app_state.active_view {
                    ActiveView::Editor => rsx! { Editor { state } },
                    ActiveView::Today => rsx! { TodayView { state } },
                    ActiveView::Tasks => rsx! { TaskView { state } },
                    ActiveView::ReviewQueue => rsx! { ReviewQueueView { state } },
                    ActiveView::Library => rsx! { LibraryView { state } },
                    ActiveView::Graph => rsx! { GraphView { state } },
                }

                // Right pane: Contextual Inspector
                if context_open {
                    div {
                        class: "pane-resizer",
                        title: tooltips::RESET_CONTEXT,
                        ondoubleclick: move |_| {
                            state.write().reset_layout();
                        }
                    }
                    Inspector { state }
                }
            }

            // Bottom Status Bar
            StatusBar { state }

            // Modals
            Dialogs { state }
            CommandPalette { state }
            QuickCaptureModal { state }
            PdfImportModal { state }
            SettingsModal { state }
            ErrorDialog { state }
            VaultHealthModal { state }
            CitationPickerModal { state }
            PdfAnnotationModal { state }
        }
    }
}
