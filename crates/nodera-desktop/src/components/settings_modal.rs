use dioxus::prelude::*;

use crate::icons::{
    IconClose, IconEdit, IconFile, IconFolder, IconKeyboard, IconPalette, IconRefresh, IconSearch,
    IconSettings,
};
use crate::state::AppState;
use crate::strings::{actions, settings, tooltips};
use crate::theme::{Theme, ThemeId};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SettingsTab {
    General,
    Appearance,
    Editor,
    PdfImport,
    Index,
    Shortcuts,
}

/// Settings modal dialog exposing user-facing configuration.
#[component]
pub fn SettingsModal(state: Signal<AppState>) -> Element {
    let mut active_tab = use_signal(|| SettingsTab::General);
    let app_state = state.read();

    if !app_state.show_settings_modal {
        return rsx! {};
    }

    let vault_path_str = app_state
        .vault_path
        .as_ref()
        .map(|p| p.display().to_string())
        .unwrap_or_else(|| "No vault currently open".to_string());

    let editor_font = app_state.preferences.editor_font_size;
    let reading_font = app_state.preferences.reading_font_size;
    let sidebar_w = app_state.sidebar_width;
    let context_w = app_state.context_panel_width;
    let current_theme = app_state.theme;
    let notes_count = app_state.entries.len();

    rsx! {
        div {
            class: "modal-backdrop",
            style: "position: fixed; top: 0; left: 0; width: 100vw; height: 100vh; background-color: rgba(0, 0, 0, 0.55); display: flex; align-items: center; justify-content: center; z-index: 1000; backdrop-filter: blur(2px);",
            onclick: move |_| {
                state.write().close_settings();
            },

            div {
                class: "modal-dialog",
                style: "width: 680px; max-width: 90vw; height: 520px; background-color: var(--bg-surface); border: 1px solid var(--border); border-radius: 10px; box-shadow: 0 12px 32px rgba(0, 0, 0, 0.35); display: flex; flex-direction: column; overflow: hidden;",
                onclick: move |evt| {
                    evt.stop_propagation();
                },

                // Header
                div {
                    style: "padding: 16px 20px; border-bottom: 1px solid var(--border); display: flex; align-items: center; justify-content: space-between; background-color: var(--bg-sidebar);",
                    div {
                        style: "display: flex; align-items: center; gap: 8px;",
                        IconSettings { size: 18 }
                        h3 { style: "margin: 0; font-size: 16px; font-weight: 600; color: var(--text-primary);", "{settings::TITLE}" }
                    }
                    button {
                        class: "btn-icon",
                        title: tooltips::CLOSE_ESC,
                        onclick: move |_| {
                            state.write().close_settings();
                        },
                        IconClose { size: 14 }
                    }
                }

                // Body: Side tabs + Content area
                div {
                    style: "flex: 1; display: flex; overflow: hidden;",

                    // Tabs sidebar
                    div {
                        style: "width: 170px; border-right: 1px solid var(--border); background-color: var(--bg-sidebar); padding: 12px 8px; display: flex; flex-direction: column; gap: 4px;",

                        button {
                            style: if *active_tab.read() == SettingsTab::General { "display: flex; align-items: center; gap: 8px; padding: 8px 12px; border-radius: 6px; background-color: var(--bg-hover); font-weight: 600; text-align: left; color: var(--text-primary);" } else { "display: flex; align-items: center; gap: 8px; padding: 8px 12px; border-radius: 6px; text-align: left; color: var(--text-secondary);" },
                            onclick: move |_| active_tab.set(SettingsTab::General),
                            IconFolder { size: 14 }
                            span { "{settings::TAB_GENERAL}" }
                        }
                        button {
                            style: if *active_tab.read() == SettingsTab::Appearance { "display: flex; align-items: center; gap: 8px; padding: 8px 12px; border-radius: 6px; background-color: var(--bg-hover); font-weight: 600; text-align: left; color: var(--text-primary);" } else { "display: flex; align-items: center; gap: 8px; padding: 8px 12px; border-radius: 6px; text-align: left; color: var(--text-secondary);" },
                            onclick: move |_| active_tab.set(SettingsTab::Appearance),
                            IconPalette { size: 14 }
                            span { "{settings::TAB_APPEARANCE}" }
                        }
                        button {
                            style: if *active_tab.read() == SettingsTab::Editor { "display: flex; align-items: center; gap: 8px; padding: 8px 12px; border-radius: 6px; background-color: var(--bg-hover); font-weight: 600; text-align: left; color: var(--text-primary);" } else { "display: flex; align-items: center; gap: 8px; padding: 8px 12px; border-radius: 6px; text-align: left; color: var(--text-secondary);" },
                            onclick: move |_| active_tab.set(SettingsTab::Editor),
                            IconEdit { size: 14 }
                            span { "{settings::TAB_EDITOR}" }
                        }
                        button {
                            style: if *active_tab.read() == SettingsTab::PdfImport { "display: flex; align-items: center; gap: 8px; padding: 8px 12px; border-radius: 6px; background-color: var(--bg-hover); font-weight: 600; text-align: left; color: var(--text-primary);" } else { "display: flex; align-items: center; gap: 8px; padding: 8px 12px; border-radius: 6px; text-align: left; color: var(--text-secondary);" },
                            onclick: move |_| active_tab.set(SettingsTab::PdfImport),
                            IconFile { size: 14 }
                            span { "{settings::TAB_PDF}" }
                        }
                        button {
                            style: if *active_tab.read() == SettingsTab::Index { "display: flex; align-items: center; gap: 8px; padding: 8px 12px; border-radius: 6px; background-color: var(--bg-hover); font-weight: 600; text-align: left; color: var(--text-primary);" } else { "display: flex; align-items: center; gap: 8px; padding: 8px 12px; border-radius: 6px; text-align: left; color: var(--text-secondary);" },
                            onclick: move |_| active_tab.set(SettingsTab::Index),
                            IconSearch { size: 14 }
                            span { "{settings::TAB_INDEX}" }
                        }
                        button {
                            style: if *active_tab.read() == SettingsTab::Shortcuts { "display: flex; align-items: center; gap: 8px; padding: 8px 12px; border-radius: 6px; background-color: var(--bg-hover); font-weight: 600; text-align: left; color: var(--text-primary);" } else { "display: flex; align-items: center; gap: 8px; padding: 8px 12px; border-radius: 6px; text-align: left; color: var(--text-secondary);" },
                            onclick: move |_| active_tab.set(SettingsTab::Shortcuts),
                            IconKeyboard { size: 14 }
                            span { "{settings::TAB_SHORTCUTS}" }
                        }
                    }

                    // Content panel
                    div {
                        style: "flex: 1; overflow-y: auto; padding: 20px 24px; background-color: var(--bg-surface); font-size: 13px;",

                        match *active_tab.read() {
                            SettingsTab::General => rsx! {
                                div { style: "display: flex; flex-direction: column; gap: 18px;",
                                    h4 { style: "margin: 0; font-size: 15px; color: var(--text-primary);", "General Settings" }

                                    div { style: "display: flex; flex-direction: column; gap: 6px;",
                                        label { style: "font-weight: 500; color: var(--text-secondary);", "Current Vault Directory" }
                                        input {
                                            r#type: "text",
                                            readonly: true,
                                            value: "{vault_path_str}",
                                            style: "padding: 8px 12px; border-radius: 6px; border: 1px solid var(--border); background-color: var(--bg-sidebar); color: var(--text-muted); font-size: 12px;"
                                        }
                                    }

                                    div { style: "display: flex; flex-direction: column; gap: 8px;",
                                        label { style: "font-weight: 500; color: var(--text-secondary);", "Auto-Save Interval" }
                                        div { style: "display: flex; align-items: center; gap: 12px;",
                                            input {
                                                r#type: "range",
                                                min: "1",
                                                max: "30",
                                                value: "{app_state.preferences.auto_save_seconds}",
                                                oninput: move |evt| {
                                                    if let Ok(v) = evt.value().parse::<u32>() {
                                                        let mut s = state.write();
                                                        s.preferences.auto_save_seconds = v;
                                                        s.preferences.save();
                                                    }
                                                }
                                            }
                                            span { style: "font-weight: 600;", "{app_state.preferences.auto_save_seconds} seconds" }
                                        }
                                        span { style: "font-size: 11px; color: var(--text-muted);", "Notes are automatically saved to disk when modified." }
                                    }
                                }
                            },

                            SettingsTab::Appearance => rsx! {
                                div { style: "display: flex; flex-direction: column; gap: 18px;",
                                    h4 { style: "margin: 0; font-size: 15px; color: var(--text-primary);", "Appearance & Layout" }

                                    div { style: "display: flex; flex-direction: column; gap: 10px;",
                                        div { style: "display: flex; align-items: baseline; justify-content: space-between;",
                                            label { style: "font-weight: 600; font-size: 13px; color: var(--text-primary);", "Theme Palette" }
                                            span { style: "font-size: 11px; color: var(--text-muted);", "6 calibrated workspace themes" }
                                        }

                                        div { style: "display: grid; grid-template-columns: repeat(2, 1fr); gap: 10px;",
                                            {ThemeId::ALL.iter().map(|&id| {
                                                let theme_item = Theme::from_id(id);
                                                let is_selected = current_theme == id;

                                                rsx! {
                                                    div {
                                                        key: "{id.as_str()}",
                                                        style: format!(
                                                            "display: flex; flex-direction: column; gap: 8px; padding: 12px; border-radius: 8px; cursor: pointer; border: 1px solid {}; background-color: {}; transition: all 0.15s ease;",
                                                            if is_selected { "var(--accent)" } else { "var(--border)" },
                                                            if is_selected { "var(--bg-hover)" } else { "var(--bg-sidebar)" }
                                                        ),
                                                        onclick: move |_| {
                                                            state.write().set_theme(id);
                                                        },

                                                        div { style: "display: flex; align-items: center; justify-content: space-between;",
                                                            div { style: "display: flex; align-items: center; gap: 8px;",
                                                                span {
                                                                    style: format!(
                                                                        "display: inline-block; width: 14px; height: 14px; border-radius: 50%; border: 2px solid {}; background-color: {}; box-sizing: border-box;",
                                                                        if is_selected { "var(--accent)" } else { "var(--border-strong)" },
                                                                        if is_selected { "var(--accent)" } else { "transparent" }
                                                                    )
                                                                }
                                                                span { style: "font-weight: 600; font-size: 13px; color: var(--text-primary);", "{theme_item.name}" }
                                                            }
                                                            if is_selected {
                                                                span {
                                                                    style: "font-size: 10px; font-weight: 600; text-transform: uppercase; padding: 2px 6px; border-radius: 4px; background-color: var(--accent); color: #ffffff;",
                                                                    "Active"
                                                                }
                                                            }
                                                        }

                                                        div { style: "font-size: 11px; color: var(--text-muted); line-height: 15px; min-height: 30px;", "{theme_item.description}" }

                                                        div {
                                                            style: format!(
                                                                "background-color: {}; border: 1px solid {}; border-radius: 6px; padding: 8px 10px; display: flex; flex-direction: column; gap: 6px; pointer-events: none;",
                                                                theme_item.bg_app,
                                                                theme_item.border
                                                            ),
                                                            div {
                                                                style: format!(
                                                                    "display: flex; align-items: center; justify-content: space-between; background-color: {}; padding: 4px 6px; border-radius: 4px; border: 1px solid {};",
                                                                    theme_item.bg_surface,
                                                                    theme_item.border_subtle
                                                                ),
                                                                span { style: format!("font-size: 10px; font-weight: 700; color: {};", theme_item.accent), "Aa" }
                                                                span { style: format!("font-size: 9px; color: {};", theme_item.text_muted), "Workspace" }
                                                                span {
                                                                    style: format!(
                                                                        "display: inline-block; width: 6px; height: 6px; border-radius: 50%; background-color: {};",
                                                                        theme_item.graph_node
                                                                    )
                                                                }
                                                            }
                                                            div {
                                                                style: format!("font-size: 11px; font-weight: 600; color: {}; margin-top: 2px;", theme_item.text_primary),
                                                                "Technical Precision"
                                                            }
                                                            div {
                                                                style: format!("font-size: 10px; color: {}; line-height: 13px;", theme_item.text_secondary),
                                                                "Semantic design architecture"
                                                            }
                                                            div { style: format!("height: 1px; background-color: {}; width: 100%;", theme_item.border_subtle) }
                                                            div { style: "display: flex; align-items: center; justify-content: space-between; gap: 6px; margin-top: 2px;",
                                                                div {
                                                                    style: format!(
                                                                        "display: inline-flex; align-items: center; padding: 2px 6px; border-radius: 3px; font-size: 9px; font-weight: 600; background-color: {}; color: #ffffff;",
                                                                        theme_item.accent
                                                                    ),
                                                                    "Action"
                                                                }
                                                                div {
                                                                    style: format!(
                                                                        "display: inline-flex; align-items: center; font-size: 9px; color: {};",
                                                                        theme_item.accent_secondary
                                                                    ),
                                                                    "[[knowledge]]"
                                                                }
                                                            }
                                                        }
                                                    }
                                                }
                                            })}
                                        }
                                    }

                                    div { style: "display: flex; flex-direction: column; gap: 8px;",
                                        label { style: "font-weight: 500; color: var(--text-secondary);", "Sidebar Width ({sidebar_w}px)" }
                                        input {
                                            r#type: "range",
                                            min: "180",
                                            max: "450",
                                            value: "{sidebar_w}",
                                            oninput: move |evt| {
                                                if let Ok(v) = evt.value().parse::<u32>() {
                                                    state.write().set_sidebar_width(v);
                                                }
                                            }
                                        }
                                    }

                                    div { style: "display: flex; flex-direction: column; gap: 8px;",
                                        label { style: "font-weight: 500; color: var(--text-secondary);", "Context Panel Width ({context_w}px)" }
                                        input {
                                            r#type: "range",
                                            min: "200",
                                            max: "450",
                                            value: "{context_w}",
                                            oninput: move |evt| {
                                                if let Ok(v) = evt.value().parse::<u32>() {
                                                    state.write().set_context_panel_width(v);
                                                }
                                            }
                                        }
                                    }

                                    div { style: "padding-top: 10px;",
                                        button {
                                            class: "btn-action",
                                            style: "display: inline-flex; align-items: center; gap: 6px;",
                                            onclick: move |_| {
                                                state.write().reset_layout();
                                            },
                                            IconRefresh { size: 13 }
                                            span { "{actions::RESET_LAYOUT}" }
                                        }
                                    }
                                }
                            },

                            SettingsTab::Editor => rsx! {
                                div { style: "display: flex; flex-direction: column; gap: 18px;",
                                    h4 { style: "margin: 0; font-size: 15px; color: var(--text-primary);", "Editor & Typography" }

                                    div { style: "display: flex; flex-direction: column; gap: 8px;",
                                        label { style: "font-weight: 500; color: var(--text-secondary);", "Editor Font Size ({editor_font}px)" }
                                        input {
                                            r#type: "range",
                                            min: "12",
                                            max: "24",
                                            value: "{editor_font}",
                                            oninput: move |evt| {
                                                if let Ok(v) = evt.value().parse::<u32>() {
                                                    let mut s = state.write();
                                                    s.preferences.editor_font_size = v;
                                                    s.preferences.save();
                                                }
                                            }
                                        }
                                    }

                                    div { style: "display: flex; flex-direction: column; gap: 8px;",
                                        label { style: "font-weight: 500; color: var(--text-secondary);", "Reading Mode Font Size ({reading_font}px)" }
                                        input {
                                            r#type: "range",
                                            min: "13",
                                            max: "24",
                                            value: "{reading_font}",
                                            oninput: move |evt| {
                                                if let Ok(v) = evt.value().parse::<u32>() {
                                                    let mut s = state.write();
                                                    s.preferences.reading_font_size = v;
                                                    s.preferences.save();
                                                }
                                            }
                                        }
                                    }

                                    div { style: "display: flex; align-items: center; gap: 8px;",
                                        input {
                                            r#type: "checkbox",
                                            id: "show-line-numbers",
                                            checked: app_state.preferences.show_line_numbers,
                                            onchange: move |evt| {
                                                let mut s = state.write();
                                                s.preferences.show_line_numbers = evt.value() == "true";
                                                s.preferences.save();
                                            }
                                        }
                                        label { r#for: "show-line-numbers", style: "color: var(--text-secondary); cursor: pointer;", "Show line numbers in editor" }
                                    }
                                }
                            },

                            SettingsTab::PdfImport => rsx! {
                                div { style: "display: flex; flex-direction: column; gap: 18px;",
                                    h4 { style: "margin: 0; font-size: 15px; color: var(--text-primary);", "PDF Import Heuristics" }

                                    p { style: "color: var(--text-muted); font-size: 12px; margin: 0;",
                                        "Nodera uses a pure-Rust PDF conversion engine (lopdf) adhering to ADR-0004. Default destination is 'Books/'."
                                    }

                                    div { style: "display: flex; flex-direction: column; gap: 10px;",
                                        div { style: "display: flex; align-items: center; gap: 8px;",
                                            input {
                                                r#type: "checkbox",
                                                id: "pdf-detect-headings",
                                                checked: app_state.pdf_import_options.detect_headings,
                                                onchange: move |evt| {
                                                    state.write().pdf_import_options.detect_headings = evt.value() == "true";
                                                }
                                            }
                                            label { r#for: "pdf-detect-headings", style: "color: var(--text-secondary); cursor: pointer;", "Detect headings and chapter boundaries" }
                                        }

                                        div { style: "display: flex; align-items: center; gap: 8px;",
                                            input {
                                                r#type: "checkbox",
                                                id: "pdf-remove-headers",
                                                checked: app_state.pdf_import_options.remove_repeated_headers,
                                                onchange: move |evt| {
                                                    state.write().pdf_import_options.remove_repeated_headers = evt.value() == "true";
                                                }
                                            }
                                            label { r#for: "pdf-remove-headers", style: "color: var(--text-secondary); cursor: pointer;", "Remove repeated running headers and footers" }
                                        }

                                        div { style: "display: flex; align-items: center; gap: 8px;",
                                            input {
                                                r#type: "checkbox",
                                                id: "pdf-remove-page-nums",
                                                checked: app_state.pdf_import_options.remove_page_numbers,
                                                onchange: move |evt| {
                                                    state.write().pdf_import_options.remove_page_numbers = evt.value() == "true";
                                                }
                                            }
                                            label { r#for: "pdf-remove-page-nums", style: "color: var(--text-secondary); cursor: pointer;", "Remove standalone page numbers" }
                                        }

                                        div { style: "display: flex; align-items: center; gap: 8px;",
                                            input {
                                                r#type: "checkbox",
                                                id: "pdf-add-markers",
                                                checked: app_state.pdf_import_options.add_page_markers,
                                                onchange: move |evt| {
                                                    state.write().pdf_import_options.add_page_markers = evt.value() == "true";
                                                }
                                            }
                                            label { r#for: "pdf-add-markers", style: "color: var(--text-secondary); cursor: pointer;", "Insert '<!-- Page N -->' HTML comments" }
                                        }
                                    }
                                }
                            },

                            SettingsTab::Index => rsx! {
                                div { style: "display: flex; flex-direction: column; gap: 18px;",
                                    h4 { style: "margin: 0; font-size: 15px; color: var(--text-primary);", "Search & Index Health" }

                                    div { style: "padding: 14px; border: 1px solid var(--border); border-radius: 8px; background-color: var(--bg-sidebar); display: flex; flex-direction: column; gap: 8px;",
                                        div { style: "display: flex; justify-content: space-between;",
                                            span { style: "color: var(--text-secondary);", "Indexed Notes:" }
                                            span { style: "font-weight: 600;", "{notes_count}" }
                                        }
                                        div { style: "display: flex; justify-content: space-between;",
                                            span { style: "color: var(--text-secondary);", "Full-Text Engine:" }
                                            span { style: "font-weight: 600;", "Tantivy (BM25)" }
                                        }
                                        div { style: "display: flex; justify-content: space-between;",
                                            span { style: "color: var(--text-secondary);", "Metadata Storage:" }
                                            span { style: "font-weight: 600;", "SQLite (WAL mode)" }
                                        }
                                    }

                                    p { style: "color: var(--text-muted); font-size: 12px; margin: 0;",
                                        "If search results become out-of-sync with vault files, rebuilding the index will rescan all Markdown files and recreate Tantivy and SQLite caches."
                                    }

                                    button {
                                        class: "btn-action btn-primary",
                                        style: "align-self: flex-start; padding: 8px 16px; display: inline-flex; align-items: center; gap: 6px;",
                                        onclick: move |_| {
                                            let mut s = state.write();
                                            let _ = s.rebuild_vault_index();
                                        },
                                        IconRefresh { size: 13 }
                                        span { "{actions::REBUILD_INDEX_NOW}" }
                                    }
                                }
                            },

                            SettingsTab::Shortcuts => rsx! {
                                div { style: "display: flex; flex-direction: column; gap: 14px;",
                                    h4 { style: "margin: 0; font-size: 15px; color: var(--text-primary);", "Keyboard Shortcuts Reference" }

                                    div { style: "border: 1px solid var(--border); border-radius: 8px; overflow: hidden;",
                                        table { style: "width: 100%; border-collapse: collapse; font-size: 12px;",
                                            tr { style: "background-color: var(--bg-sidebar); border-bottom: 1px solid var(--border); text-align: left;",
                                                th { style: "padding: 8px 12px;", "Action" }
                                                th { style: "padding: 8px 12px;", "Shortcut" }
                                            }
                                            tr { style: "border-bottom: 1px solid var(--border-subtle);",
                                                td { style: "padding: 8px 12px;", "New Note" }
                                                td { style: "padding: 8px 12px; font-family: monospace; font-weight: 600;", "Ctrl + N" }
                                            }
                                            tr { style: "border-bottom: 1px solid var(--border-subtle);",
                                                td { style: "padding: 8px 12px;", "Save Active Note" }
                                                td { style: "padding: 8px 12px; font-family: monospace; font-weight: 600;", "Ctrl + S" }
                                            }
                                            tr { style: "border-bottom: 1px solid var(--border-subtle);",
                                                td { style: "padding: 8px 12px;", "Command Palette" }
                                                td { style: "padding: 8px 12px; font-family: monospace; font-weight: 600;", "Ctrl + P" }
                                            }
                                            tr { style: "border-bottom: 1px solid var(--border-subtle);",
                                                td { style: "padding: 8px 12px;", "Import PDF as Markdown" }
                                                td { style: "padding: 8px 12px; font-family: monospace; font-weight: 600;", "Ctrl + Shift + I" }
                                            }
                                            tr { style: "border-bottom: 1px solid var(--border-subtle);",
                                                td { style: "padding: 8px 12px;", "Toggle Reading / Edit Mode" }
                                                td { style: "padding: 8px 12px; font-family: monospace; font-weight: 600;", "Ctrl + E" }
                                            }
                                            tr { style: "border-bottom: 1px solid var(--border-subtle);",
                                                td { style: "padding: 8px 12px;", "Cycle Views (Notes / Tasks / Library)" }
                                                td { style: "padding: 8px 12px; font-family: monospace; font-weight: 600;", "Ctrl + G" }
                                            }
                                            tr { style: "border-bottom: 1px solid var(--border-subtle);",
                                                td { style: "padding: 8px 12px;", "Toggle Sidebar" }
                                                td { style: "padding: 8px 12px; font-family: monospace; font-weight: 600;", "Ctrl + \\" }
                                            }
                                            tr { style: "border-bottom: 1px solid var(--border-subtle);",
                                                td { style: "padding: 8px 12px;", "Open Settings" }
                                                td { style: "padding: 8px 12px; font-family: monospace; font-weight: 600;", "Ctrl + ," }
                                            }
                                            tr {
                                                td { style: "padding: 8px 12px;", "Close Dialog / Modal" }
                                                td { style: "padding: 8px 12px; font-family: monospace; font-weight: 600;", "Escape" }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                // Footer
                div {
                    style: "padding: 12px 20px; border-top: 1px solid var(--border); display: flex; justify-content: flex-end; background-color: var(--bg-sidebar);",
                    button {
                        class: "btn-action btn-primary",
                        onclick: move |_| {
                            state.write().close_settings();
                        },
                        "{actions::DONE}"
                    }
                }
            }
        }
    }
}
