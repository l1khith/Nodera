use dioxus::prelude::*;

use crate::icons::{
    IconBook, IconBookmark, IconCheck, IconChevronLeft, IconChevronRight, IconClose, IconColumns,
    IconEdit, IconFile, IconLink, IconList, IconNotes, IconPanelRight, IconPin, IconPlus,
    IconQuote, IconTemplate,
};
use crate::state::{AppState, SplitDirection};
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

    // Tabs & History state
    let open_tabs = app_state.open_tabs.clone();
    let active_tab_index = app_state.active_tab_index;
    let can_back = app_state.can_navigate_back();
    let can_forward = app_state.can_navigate_forward();

    // Markdown HTML rendering for reading mode & split live preview with transclusion resolver
    let vault_service = app_state.vault_service.clone();
    let rendered_html = if is_reading_mode || app_state.split_pane.is_some() {
        let resolver = move |target: &str| -> Option<String> {
            if let Some(service) = &vault_service {
                let clean_target = target.split('#').next().unwrap_or(target).trim();
                let rel_path = std::path::PathBuf::from(format!("{clean_target}.md"));
                if let Ok(note) = service.read_note(&rel_path) {
                    return Some(note.content);
                }
                if let Ok(entries) = service.list_entries() {
                    for entry in entries {
                        if let nodera_core::VaultEntry::Note(summary) = entry {
                            if summary.title.eq_ignore_ascii_case(clean_target) {
                                if let Ok(note) = service.read_note(&summary.relative_path) {
                                    return Some(note.content);
                                }
                            }
                        }
                    }
                }
            }
            None
        };
        nodera_markdown::render_to_html_with_resolver(&content, &resolver)
    } else {
        String::new()
    };

    // Stats
    let words_count = content.split_whitespace().count();
    let chars_count = content.chars().count();

    rsx! {
        main { class: "pane-center",
            // Multi-Tab Bar
            if !open_tabs.is_empty() {
                div { class: "tab-bar",
                    // Back & Forward navigation buttons
                    div { style: "display: flex; align-items: center; gap: 2px; margin-right: 6px;",
                        button {
                            class: "btn-icon",
                            style: if can_back { "width: 24px; height: 24px;" } else { "width: 24px; height: 24px; opacity: 0.3; cursor: default;" },
                            title: "Navigate Back (Alt+Left)",
                            disabled: !can_back,
                            onclick: move |_| {
                                let mut s = state.write();
                                let _ = s.navigate_back();
                            },
                            IconChevronLeft { size: 13 }
                        }
                        button {
                            class: "btn-icon",
                            style: if can_forward { "width: 24px; height: 24px;" } else { "width: 24px; height: 24px; opacity: 0.3; cursor: default;" },
                            title: "Navigate Forward (Alt+Right)",
                            disabled: !can_forward,
                            onclick: move |_| {
                                let mut s = state.write();
                                let _ = s.navigate_forward();
                            },
                            IconChevronRight { size: 13 }
                        }
                    }

                    // Open Tab items
                    for (idx, tab) in open_tabs.iter().enumerate() {
                        {
                            let is_active = active_tab_index == Some(idx);
                            let is_pinned = tab.is_pinned;
                            let title = tab.title.clone();
                            let path_str = tab.relative_path.display().to_string();

                            rsx! {
                                div {
                                    key: "{path_str}",
                                    class: if is_active { "tab-item active" } else { "tab-item" },
                                    title: "{path_str}",
                                    onclick: move |_| {
                                        let mut s = state.write();
                                        let _ = s.select_tab(idx);
                                    },
                                    if is_pinned {
                                        span {
                                            style: "color: var(--accent); cursor: pointer; display: inline-flex;",
                                            title: "Unpin tab",
                                            onclick: move |e| {
                                                e.stop_propagation();
                                                state.write().toggle_pin_tab(idx);
                                            },
                                            IconPin { size: 11 }
                                        }
                                    } else {
                                        IconFile { size: 12, class: "opacity-70" }
                                    }
                                    span { class: "tab-title", "{title}" }
                                    if is_active && is_dirty {
                                        span { class: "tab-dirty-dot", title: "Unsaved changes" }
                                    }
                                    if !is_pinned {
                                        span {
                                            class: "tab-close",
                                            title: "Close tab (Ctrl+W)",
                                            onclick: move |e| {
                                                e.stop_propagation();
                                                let mut s = state.write();
                                                let _ = s.close_tab(idx);
                                            },
                                            IconClose { size: 11 }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

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
                        div { "Ctrl + W : Close Tab" }
                        div { "Alt + Left / Right : Navigate Back / Forward" }
                        div { "Ctrl + Shift + D : Daily Note" }
                        div { "Ctrl + T : Insert Template" }
                        div { "Ctrl + E : Toggle Reading / Edit Mode" }
                    }
                }
            } else {
                // Active Note Toolbar
                div {
                    style: "height: 44px; border-bottom: 1px solid var(--border); background-color: var(--bg-surface); display: flex; align-items: center; justify-content: space-between; padding: 0 16px;",
                    div { style: "display: flex; align-items: center; gap: 10px;",
                        // Breadcrumbs path navigation
                        div { class: "breadcrumb-container",
                            {
                                let components: Vec<&str> = note_path.split(['/', '\\']).collect();
                                let total = components.len();
                                rsx! {
                                    for (i, seg) in components.iter().enumerate() {
                                        if i > 0 {
                                            span { style: "opacity: 0.4;", "/" }
                                        }
                                        if i + 1 == total {
                                            span { class: "breadcrumb-active", "{note_title}" }
                                        } else {
                                            span { class: "breadcrumb-segment", "{seg}" }
                                        }
                                    }
                                }
                            }
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
                        if let Some(active_idx) = active_tab_index {
                            {
                                let is_pinned = open_tabs.get(active_idx).map(|t| t.is_pinned).unwrap_or(false);
                                rsx! {
                                    button {
                                        class: "btn-icon",
                                        title: if is_pinned { "Unpin tab" } else { "Pin tab" },
                                        onclick: move |_| {
                                            state.write().toggle_pin_tab(active_idx);
                                        },
                                        IconPin { size: 14 }
                                    }
                                }
                            }
                        }
                        if let Some(active_note) = &app_state.active_note {
                            {
                                let is_bm = app_state.is_bookmarked(&active_note.relative_path);
                                let p = active_note.relative_path.clone();
                                rsx! {
                                    button {
                                        class: "btn-icon",
                                        style: if is_bm { "color: var(--accent);" } else { "" },
                                        title: if is_bm { "Remove bookmark" } else { "Bookmark note" },
                                        onclick: move |_| {
                                            state.write().toggle_bookmark(&p);
                                        },
                                        IconBookmark { size: 14 }
                                    }
                                }
                            }
                        }
                        button {
                            class: "btn-action",
                            title: "Insert Wikilink (Ctrl+K)",
                            onclick: move |_| {
                                state.write().insert_wikilink_snippet("Note");
                            },
                            IconLink { size: 13 }
                            span { "Link" }
                        }
                        button {
                            class: "btn-action",
                            title: "Insert Section / Note Embed (![[Note#Heading]])",
                            onclick: move |_| {
                                state.write().insert_embed_snippet("Note#Heading");
                            },
                            span { style: "font-family: monospace; font-size: 11px; font-weight: 700;", "![]" }
                            span { "Embed" }
                        }
                        button {
                            class: "btn-action",
                            title: "Insert BibTeX / Zotero Citation (Ctrl+Shift+C)",
                            onclick: move |_| {
                                state.write().show_citation_picker_modal = true;
                            },
                            IconQuote { size: 13 }
                            span { "Cite" }
                        }
                        button {
                            class: if app_state.split_pane.is_some() { "btn-action active-toggle" } else { "btn-action" },
                            style: if app_state.split_pane.is_some() { "color: var(--accent); border-color: var(--accent);" } else { "" },
                            title: "Toggle Split View (Ctrl+\\)",
                            onclick: move |_| {
                                state.write().toggle_split();
                            },
                            IconColumns { size: 14 }
                            span { "Split" }
                        }
                        button {
                            class: "btn-action",
                            title: tooltips::INSERT_TEMPLATE,
                            onclick: move |_| {
                                let mut s = state.write();
                                s.show_template_modal = true;
                                s.template_search_query.clear();
                            },
                            IconTemplate { size: 14 }
                            span { "{actions::TEMPLATES}" }
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
                            class: if app_state.context_panel_open { "btn-icon active-toggle" } else { "btn-icon" },
                            style: if app_state.context_panel_open { "color: var(--accent);" } else { "" },
                            title: "Toggle Inspector (Ctrl+I)",
                            onclick: move |_| {
                                let mut s = state.write();
                                s.context_panel_open = !s.context_panel_open;
                            },
                            IconPanelRight { size: 14 }
                        }
                    }
                }

                // Editor / Reader Content Surface (Split view or Single pane)
                if let Some(_split) = &app_state.split_pane {
                    div {
                        style: format!(
                            "flex: 1; display: flex; flex-direction: {}; overflow: hidden; background-color: var(--bg-app);",
                            if app_state.split_direction == SplitDirection::Horizontal { "row" } else { "column" }
                        ),

                        // Pane 1: Editor Pane
                        div {
                            style: "flex: 1; display: flex; flex-direction: column; overflow: hidden; min-width: 250px; min-height: 150px; border-right: 1px solid var(--border);",
                            div {
                                class: "split-pane-header",
                                div { style: "display: flex; align-items: center; gap: 6px; font-size: 11px; font-weight: 600; color: var(--text-muted); text-transform: uppercase;",
                                    IconEdit { size: 12 }
                                    span { "Markdown Editor" }
                                }
                            }
                            textarea {
                                class: "editor-textarea",
                                style: format!("flex: 1; width: 100%; border: none; padding: 20px 24px; font-family: var(--font-editor); font-size: {}px; line-height: 26px; resize: none; background: transparent; outline: none; color: var(--text-primary);", app_state.preferences.editor_font_size),
                                value: "{content}",
                                placeholder: placeholders::TYPE_MARKDOWN,
                                oninput: move |evt| {
                                    let mut s = state.write();
                                    s.update_editor_content(evt.value());
                                }
                            }
                        }

                        // Split Divider with controls
                        div {
                            class: "split-divider",
                            button {
                                class: "btn-icon",
                                style: "width: 24px; height: 24px;",
                                title: "Toggle split direction (Horizontal / Vertical)",
                                onclick: move |_| {
                                    state.write().toggle_split_direction();
                                },
                                IconColumns { size: 13 }
                            }
                            button {
                                class: "btn-icon",
                                style: "width: 24px; height: 24px;",
                                title: "Close split view",
                                onclick: move |_| {
                                    state.write().close_split();
                                },
                                IconClose { size: 13 }
                            }
                        }

                        // Pane 2: Live Reading Preview
                        div {
                            style: "flex: 1; display: flex; flex-direction: column; overflow: hidden; min-width: 250px; min-height: 150px; background-color: var(--bg-surface);",
                            div {
                                class: "split-pane-header",
                                div { style: "display: flex; align-items: center; gap: 6px; font-size: 11px; font-weight: 600; color: var(--text-muted); text-transform: uppercase;",
                                    IconBook { size: 12 }
                                    span { "Live Reading Preview" }
                                }
                            }
                            div {
                                class: "reading-view markdown-body",
                                style: format!("flex: 1; padding: 24px 32px; overflow-y: auto; line-height: 26px; font-size: {}px; color: var(--text-primary); font-family: var(--font-editor);", app_state.preferences.reading_font_size),
                                dangerous_inner_html: "{rendered_html}"
                            }
                        }
                    }
                } else {
                    // Single-pane Content Surface
                    div {
                        style: "flex: 1; display: flex; overflow: hidden; background-color: var(--bg-app); position: relative;",

                        if !is_reading_mode {
                            textarea {
                                class: "editor-textarea",
                                style: format!("flex: 1; width: 100%; border: none; padding: 24px 32px; font-family: var(--font-editor); font-size: {}px; line-height: 26px; resize: none; background: transparent; outline: none; color: var(--text-primary);", app_state.preferences.editor_font_size),
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
                                        style: "width: 220px; border-right: 1px solid var(--border); background: var(--bg-surface); overflow-y: auto; padding: 16px 12px; font-size: 12px; display: flex; flex-direction: column; gap: 6px; flex-shrink: 0;",

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
                                        style: format!("flex: 1; padding: 40px 60px; overflow-y: auto; line-height: 26px; font-size: {}px; color: var(--text-primary); max-width: 800px; margin: 0 auto; width: 100%; font-family: var(--font-editor);", app_state.preferences.reading_font_size),
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
                                                    style: "display: flex; align-items: center; justify-content: space-between; padding: 8px 24px; border-top: 1px solid var(--border); background: var(--bg-surface); font-size: 12px; color: var(--text-muted);",

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
}
