use dioxus::prelude::*;

use crate::icons::*;
use crate::state::AppState;
use crate::strings::{actions, app as app_strings, empty_states, nav, placeholders};

#[derive(Clone, PartialEq, Eq)]
enum ContextMenuTarget {
    File(std::path::PathBuf),
    Folder(std::path::PathBuf),
    Root,
}

enum VisibleTreeItem<'a> {
    Folder {
        name: &'a str,
        relative_path: &'a std::path::Path,
        depth: usize,
        is_expanded: bool,
        is_empty: bool,
    },
    File {
        summary: &'a nodera_core::NoteSummary,
        depth: usize,
    },
}

fn flatten_visible_items<'a>(
    nodes: &'a [crate::state::FileTreeNode],
    depth: usize,
    expanded: &std::collections::HashSet<std::path::PathBuf>,
    out: &mut Vec<VisibleTreeItem<'a>>,
) {
    for node in nodes {
        match node {
            crate::state::FileTreeNode::Folder {
                name,
                relative_path,
                children,
            } => {
                let is_expanded = expanded.contains(relative_path);
                out.push(VisibleTreeItem::Folder {
                    name,
                    relative_path,
                    depth,
                    is_expanded,
                    is_empty: children.is_empty(),
                });
                if is_expanded && !children.is_empty() {
                    flatten_visible_items(children, depth + 1, expanded, out);
                }
            }
            crate::state::FileTreeNode::File(summary) => {
                out.push(VisibleTreeItem::File { summary, depth });
            }
        }
    }
}

#[component]
pub fn Sidebar(state: Signal<AppState>) -> Element {
    let mut context_menu = use_signal(|| None::<(ContextMenuTarget, f64, f64)>);
    let mut show_header_dropdown = use_signal(|| false);

    let app_state = state.read();
    let entries = app_state.entries.clone();
    let active_path = app_state
        .active_note
        .as_ref()
        .map(|n| n.relative_path.clone());
    let has_vault = app_state.vault_service.is_some();

    rsx! {
    aside { class: "pane-sidebar",
        // Workspace navigation header
        div {
            style: "padding: 10px 14px; border-bottom: 1px solid var(--border); display: flex; align-items: center; justify-content: space-between;",
            span {
                style: "font-weight: 600; font-size: 11px; text-transform: uppercase; letter-spacing: 0.5px; color: var(--text-muted);",
                "{app_strings::WORKSPACE}"
            }
        }

        // Navigation items: Notes, Tasks, Library
        div {
            style: "padding: 8px 12px; border-bottom: 1px solid var(--border-subtle); display: flex; flex-direction: column; gap: 2px;",
            button {
                style: if app_state.active_view == crate::state::ActiveView::Editor { "padding: 5px 8px; border-radius: 4px; background-color: var(--bg-hover); font-weight: 600; display: flex; align-items: center; gap: 8px; width: 100%; text-align: left; color: var(--text-primary);" } else { "padding: 5px 8px; border-radius: 4px; color: var(--text-secondary); display: flex; align-items: center; gap: 8px; width: 100%; text-align: left;" },
                onclick: move |_| {
                    let mut s = state.write();
                    s.active_view = crate::state::ActiveView::Editor;
                },
                IconNotes { size: 15 }
                span { "{nav::NOTES}" }
            }
            button {
                style: if app_state.active_view == crate::state::ActiveView::Today { "padding: 5px 8px; border-radius: 4px; background-color: var(--bg-hover); font-weight: 600; display: flex; align-items: center; gap: 8px; width: 100%; text-align: left; color: var(--text-primary);" } else { "padding: 5px 8px; border-radius: 4px; color: var(--text-secondary); display: flex; align-items: center; gap: 8px; width: 100%; text-align: left;" },
                onclick: move |_| {
                    let mut s = state.write();
                    s.active_view = crate::state::ActiveView::Today;
                },
                IconCalendar { size: 15 }
                span { "{nav::TODAY}" }
            }
            button {
                style: if app_state.active_view == crate::state::ActiveView::Tasks { "padding: 5px 8px; border-radius: 4px; background-color: var(--bg-hover); font-weight: 600; display: flex; align-items: center; gap: 8px; width: 100%; text-align: left; color: var(--text-primary);" } else { "padding: 5px 8px; border-radius: 4px; color: var(--text-secondary); display: flex; align-items: center; gap: 8px; width: 100%; text-align: left;" },
                onclick: move |_| {
                    let mut s = state.write();
                    s.active_view = crate::state::ActiveView::Tasks;
                },
                IconTasks { size: 15 }
                span { "{nav::TASKS}" }
            }
            button {
                style: if app_state.active_view == crate::state::ActiveView::ReviewQueue { "padding: 5px 8px; border-radius: 4px; background-color: var(--bg-hover); font-weight: 600; display: flex; align-items: center; gap: 8px; width: 100%; text-align: left; color: var(--text-primary);" } else { "padding: 5px 8px; border-radius: 4px; color: var(--text-secondary); display: flex; align-items: center; gap: 8px; width: 100%; text-align: left;" },
                onclick: move |_| {
                    let mut s = state.write();
                    s.active_view = crate::state::ActiveView::ReviewQueue;
                },
                IconReviewQueue { size: 15 }
                span { "Review Queue" }
            }
            button {
                style: if app_state.active_view == crate::state::ActiveView::Library { "padding: 5px 8px; border-radius: 4px; background-color: var(--bg-hover); font-weight: 600; display: flex; align-items: center; gap: 8px; width: 100%; text-align: left; color: var(--text-primary);" } else { "padding: 5px 8px; border-radius: 4px; color: var(--text-secondary); display: flex; align-items: center; gap: 8px; width: 100%; text-align: left;" },
                onclick: move |_| {
                    let mut s = state.write();
                    s.active_view = crate::state::ActiveView::Library;
                },
                IconLibrary { size: 15 }
                span { "{nav::LIBRARY}" }
            }
            button {
                style: if app_state.active_view == crate::state::ActiveView::Graph { "padding: 5px 8px; border-radius: 4px; background-color: var(--bg-hover); font-weight: 600; display: flex; align-items: center; gap: 8px; width: 100%; text-align: left; color: var(--text-primary);" } else { "padding: 5px 8px; border-radius: 4px; color: var(--text-secondary); display: flex; align-items: center; gap: 8px; width: 100%; text-align: left;" },
                onclick: move |_| {
                    let mut s = state.write();
                    s.active_view = crate::state::ActiveView::Graph;
                },
                IconGraph { size: 15 }
                span { "{nav::GRAPH}" }
            }
            button {
                style: "padding: 5px 8px; border-radius: 4px; color: var(--text-secondary); display: flex; align-items: center; gap: 8px; width: 100%; text-align: left;",
                onclick: move |_| {
                    let mut s = state.write();
                    s.show_trash_modal = true;
                },
                IconTrash { size: 15 }
                span { "Trash Bin" }
            }
            button {
                style: "padding: 5px 8px; border-radius: 4px; color: var(--text-secondary); display: flex; align-items: center; gap: 8px; width: 100%; text-align: left;",
                title: "Check broken links & orphan notes",
                onclick: move |_| {
                    let mut s = state.write();
                    s.show_vault_health_modal = true;
                },
                IconActivity { size: 15 }
                span { "Vault Health" }
            }
        }

        // Quick Search Input
        if has_vault {
            div {
                style: "padding: 8px 12px; border-bottom: 1px solid var(--border-subtle);",
                div {
                    style: "display: flex; align-items: center; background-color: var(--bg-surface-elevated); border: 1px solid var(--border); border-radius: 4px; padding: 4px 8px; gap: 6px;",
                    IconSearch { size: 13, class: "opacity-70" }
                    input {
                        style: "flex: 1; border: none; background: transparent; font-size: 12px; outline: none; color: var(--text-primary);",
                        placeholder: placeholders::SEARCH_VAULT,
                        value: "{app_state.search_query}",
                        oninput: move |evt| {
                            let mut s = state.write();
                            s.execute_search(&evt.value());
                        }
                    }
                    if !app_state.search_query.is_empty() {
                        button {
                            style: "font-size: 10px; color: var(--text-muted); cursor: pointer;",
                            title: actions::CLEAR,
                            onclick: move |_| {
                                let mut s = state.write();
                                s.execute_search("");
                            },
                            IconClose { size: 11 }
                        }
                    }
                }
            }
        }

        // File tree / Search Results section
        div {
            style: "flex: 1; overflow-y: auto; padding: 8px;",
            oncontextmenu: move |evt: MouseEvent| {
                evt.prevent_default();
                let coords = evt.client_coordinates();
                context_menu.set(Some((ContextMenuTarget::Root, coords.x, coords.y)));
            },
            if !has_vault {
                div {
                    style: "padding: 20px 8px; text-align: center; color: var(--text-muted); display: flex; flex-direction: column; gap: 12px;",
                    p { "{empty_states::NO_VAULT_OPEN}" }
                    button {
                        class: "btn-action btn-primary",
                        style: "align-self: center;",
                        onclick: move |_| {
                            spawn(async move {
                                if let Some(folder) = rfd::AsyncFileDialog::new().pick_folder().await {
                                    let path = folder.path().to_path_buf();
                                    let mut s = state.write();
                                    let _ = s.open_vault(path);
                                }
                            });
                        },
                        "{empty_states::OPEN_VAULT_FOLDER}"
                    }
                }
            } else if !app_state.search_query.is_empty() {
                // Display search results
                div { style: "display: flex; flex-direction: column; gap: 4px;",
                    div { style: "font-size: 11px; font-weight: 600; color: var(--text-muted); margin-bottom: 4px; padding: 0 4px;",
                        "SEARCH RESULTS ({app_state.search_results.len()})"
                    }
                    if app_state.search_results.is_empty() {
                        div {
                            style: "padding: 16px 8px; text-align: center; color: var(--text-muted); font-size: 12px; display: flex; flex-direction: column; gap: 4px;",
                            span { style: "font-weight: 500; color: var(--text-secondary);", "{empty_states::NO_SEARCH_RESULTS_TITLE}" }
                            span { style: "font-size: 11px;", "{empty_states::NO_SEARCH_RESULTS_DESC}" }
                        }
                    } else {
                        for res in app_state.search_results.iter() {
                            {
                                let path_buf = std::path::PathBuf::from(&res.path);
                                let path_clone = path_buf.clone();
                                let title = res.title.clone();
                                let snippet = res.snippet.clone();
                                rsx! {
                                    button {
                                        key: "{res.path}",
                                        class: "link-item",
                                        style: "flex-direction: column; align-items: flex-start; gap: 3px; padding: 8px;",
                                        onclick: move |_| {
                                            let mut s = state.write();
                                            s.active_view = crate::state::ActiveView::Editor;
                                            let _ = s.select_note(&path_clone);
                                        },
                                        span {
                                            style: "font-weight: 600; font-size: 12px; color: var(--text-primary); display: inline-flex; align-items: center; gap: 6px;",
                                            IconFile { size: 13 }
                                            span { "{title}" }
                                        }
                                        span { style: "font-size: 10px; color: var(--text-muted);", "{path_buf.display()}" }
                                        if !snippet.is_empty() {
                                            div {
                                                style: "font-size: 11px; color: var(--text-secondary); line-height: 1.4; margin-top: 2px;",
                                                dangerous_inner_html: "{snippet}"
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            } else if entries.is_empty() {
                div {
                    style: "padding: 24px 12px; text-align: center; color: var(--text-muted); display: flex; flex-direction: column; align-items: center; gap: 10px;",
                    IconFolderOpen { size: 32 }
                    p { style: "font-size: 13px; font-weight: 500; margin: 0; color: var(--text-primary);", "{empty_states::EMPTY_VAULT_TITLE}" }
                    p { style: "font-size: 11px; line-height: 1.4; margin: 0;", "{empty_states::EMPTY_VAULT_DESC}" }
                    div { style: "display: flex; flex-direction: column; gap: 6px; margin-top: 6px; width: 100%;",
                        button {
                            class: "btn-action btn-primary",
                            style: "width: 100%; justify-content: center;",
                            onclick: move |_| {
                                let mut s = state.write();
                                s.show_new_note_dialog = true;
                            },
                            IconPlus { size: 14 }
                            span { "{actions::CREATE_NOTE}" }
                        }
                        button {
                            class: "btn-action",
                            style: "width: 100%; justify-content: center;",
                            onclick: move |_| {
                                let mut s = state.write();
                                s.open_pdf_import_modal();
                            },
                            IconImport { size: 14 }
                            span { "{actions::IMPORT_PDF_BTN}" }
                        }
                    }
                }
            } else {
                // Bookmarks Section
                if !app_state.current_vault_bookmarks().is_empty() {
                    {
                        let bookmarks = app_state.current_vault_bookmarks();
                        let show_bookmarks = app_state.show_bookmarks_section;
                        rsx! {
                            div {
                                style: "margin-bottom: 8px; border-bottom: 1px solid var(--border-subtle); padding-bottom: 6px;",
                                div {
                                    style: "display: flex; align-items: center; justify-content: space-between; padding: 4px 8px; cursor: pointer; color: var(--text-muted); font-size: 11px; font-weight: 600; text-transform: uppercase; letter-spacing: 0.5px; border-radius: 4px;",
                                    onclick: move |_| {
                                        let mut s = state.write();
                                        s.show_bookmarks_section = !s.show_bookmarks_section;
                                    },
                                    div { style: "display: flex; align-items: center; gap: 5px;",
                                        if show_bookmarks {
                                            IconChevronDown { size: 11 }
                                        } else {
                                            IconChevronRight { size: 11 }
                                        }
                                        IconBookmark { size: 12 }
                                        span { "Bookmarks ({bookmarks.len()})" }
                                    }
                                }
                                if show_bookmarks {
                                    div { style: "display: flex; flex-direction: column; gap: 1px; margin-top: 2px;",
                                        for path in &bookmarks {
                                            {
                                                let p = path.clone();
                                                let is_act = active_path.as_ref() == Some(&p);
                                                let title = p.file_stem().and_then(|s| s.to_str()).unwrap_or("Untitled").to_string();
                                                let act_bg = if is_act { "var(--bg-active)" } else { "transparent" };
                                                let act_fg = if is_act { "var(--text-primary)" } else { "var(--text-secondary)" };
                                                let p_toggle = p.clone();
                                                let p_select = p.clone();
                                                rsx! {
                                                    div {
                                                        key: "{p.display()}",
                                                        style: format!("padding: 3px 8px 3px 18px; border-radius: 4px; display: flex; align-items: center; justify-content: space-between; cursor: pointer; background-color: {act_bg}; color: {act_fg}; font-size: 12px;"),
                                                        onclick: move |_| {
                                                            let mut s = state.write();
                                                            s.active_view = crate::state::ActiveView::Editor;
                                                            let _ = s.select_note(&p_select);
                                                        },
                                                        span {
                                                            style: "overflow: hidden; text-overflow: ellipsis; white-space: nowrap; flex: 1; display: inline-flex; align-items: center; gap: 6px;",
                                                            IconBookmark { size: 11 }
                                                            span { "{title}" }
                                                        }
                                                        button {
                                                            class: "btn-icon",
                                                            style: "width: 18px; height: 18px; opacity: 0.6;",
                                                            title: "Remove bookmark",
                                                            onclick: move |e| {
                                                                e.stop_propagation();
                                                                state.write().toggle_bookmark(&p_toggle);
                                                            },
                                                            IconClose { size: 10 }
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

                // Recent Notes Section
                if !app_state.current_vault_recent_notes().is_empty() {
                    {
                        let recent_notes = app_state.current_vault_recent_notes();
                        let show_recent = app_state.show_recent_section;
                        rsx! {
                            div {
                                style: "margin-bottom: 8px; border-bottom: 1px solid var(--border-subtle); padding-bottom: 6px;",
                                div {
                                    style: "display: flex; align-items: center; justify-content: space-between; padding: 4px 8px; cursor: pointer; color: var(--text-muted); font-size: 11px; font-weight: 600; text-transform: uppercase; letter-spacing: 0.5px; border-radius: 4px;",
                                    onclick: move |_| {
                                        let mut s = state.write();
                                        s.show_recent_section = !s.show_recent_section;
                                    },
                                    div { style: "display: flex; align-items: center; gap: 5px;",
                                        if show_recent {
                                            IconChevronDown { size: 11 }
                                        } else {
                                            IconChevronRight { size: 11 }
                                        }
                                        IconClock { size: 12 }
                                        span { "Recent ({recent_notes.len()})" }
                                    }
                                    button {
                                        class: "btn-icon",
                                        style: "width: 18px; height: 18px; opacity: 0.5;",
                                        title: "Clear recent notes",
                                        onclick: move |e| {
                                            e.stop_propagation();
                                            state.write().clear_recent_notes();
                                        },
                                        IconClose { size: 10 }
                                    }
                                }
                                if show_recent {
                                    div { style: "display: flex; flex-direction: column; gap: 1px; margin-top: 2px;",
                                        for path in &recent_notes {
                                            {
                                                let p = path.clone();
                                                let is_act = active_path.as_ref() == Some(&p);
                                                let title = p.file_stem().and_then(|s| s.to_str()).unwrap_or("Untitled").to_string();
                                                let act_bg = if is_act { "var(--bg-active)" } else { "transparent" };
                                                let act_fg = if is_act { "var(--text-primary)" } else { "var(--text-secondary)" };
                                                let p_remove = p.clone();
                                                let p_select = p.clone();
                                                rsx! {
                                                    div {
                                                        key: "{p.display()}",
                                                        style: format!("padding: 3px 8px 3px 18px; border-radius: 4px; display: flex; align-items: center; justify-content: space-between; cursor: pointer; background-color: {act_bg}; color: {act_fg}; font-size: 12px;"),
                                                        onclick: move |_| {
                                                            let mut s = state.write();
                                                            s.active_view = crate::state::ActiveView::Editor;
                                                            let _ = s.select_note(&p_select);
                                                        },
                                                        span {
                                                            style: "overflow: hidden; text-overflow: ellipsis; white-space: nowrap; flex: 1; display: inline-flex; align-items: center; gap: 6px;",
                                                            IconClock { size: 11 }
                                                            span { "{title}" }
                                                        }
                                                        button {
                                                            class: "btn-icon",
                                                            style: "width: 18px; height: 18px; opacity: 0.6;",
                                                            title: "Remove from recent",
                                                            onclick: move |e| {
                                                                e.stop_propagation();
                                                                state.write().remove_recent_note(&p_remove);
                                                            },
                                                            IconClose { size: 10 }
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

                // External Source Projects Section
                div {
                    style: "margin-bottom: 8px; border-bottom: 1px solid var(--border-subtle); padding-bottom: 6px;",
                    div {
                        style: "display: flex; align-items: center; justify-content: space-between; padding: 4px 8px; cursor: pointer; color: var(--text-muted); font-size: 11px; font-weight: 600; text-transform: uppercase; letter-spacing: 0.5px; border-radius: 4px;",
                        onclick: move |_| {
                            let mut s = state.write();
                            s.show_projects_section = !s.show_projects_section;
                        },
                        div { style: "display: flex; align-items: center; gap: 5px;",
                            if app_state.show_projects_section {
                                IconChevronDown { size: 11 }
                            } else {
                                IconChevronRight { size: 11 }
                            }
                            span { "Projects ({app_state.registered_projects.len()})" }
                        }
                        button {
                            class: "btn-icon",
                            style: "width: 18px; height: 18px;",
                            title: "Refresh registered projects",
                            onclick: move |e| {
                                e.stop_propagation();
                                state.write().refresh_registered_projects();
                            },
                            IconRefresh { size: 11 }
                        }
                    }
                    if app_state.show_projects_section {
                        div { style: "display: flex; flex-direction: column; gap: 2px; padding-left: 8px; margin-top: 2px;",
                            if app_state.registered_projects.is_empty() {
                                div { style: "font-size: 11px; color: var(--text-muted); padding: 4px 8px;", "No projects registered" }
                            } else {
                                for proj in &app_state.registered_projects {
                                    {
                                        let proj_id = proj.id.clone();
                                        let is_active = app_state.active_project.as_ref().map(|p| p.id == proj.id).unwrap_or(false);
                                        let name = proj.name.clone();
                                        rsx! {
                                            button {
                                                key: "{proj.id}",
                                                style: if is_active {
                                                    "display: flex; align-items: center; justify-content: space-between; padding: 5px 8px; border-radius: 4px; background: var(--bg-hover); color: var(--accent); font-weight: 600; width: 100%; text-align: left; font-size: 12px;"
                                                } else {
                                                    "display: flex; align-items: center; justify-content: space-between; padding: 5px 8px; border-radius: 4px; color: var(--text-secondary); width: 100%; text-align: left; font-size: 12px;"
                                                },
                                                onclick: move |_| {
                                                    let mut s = state.write();
                                                    let _ = s.select_project(&proj_id);
                                                },
                                                span { style: "display: inline-flex; align-items: center; gap: 6px;",
                                                    IconCode { size: 13 }
                                                    span { "{name}" }
                                                }
                                                if is_active {
                                                    span { style: "font-size: 9px; padding: 1px 4px; border-radius: 3px; background: var(--accent); color: white;", "ACTIVE" }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                // Vault File Tree Header
                div {
                    style: "padding: 4px 8px; font-weight: 600; font-size: 11px; text-transform: uppercase; letter-spacing: 0.5px; color: var(--text-muted); display: flex; align-items: center; justify-content: space-between; margin-bottom: 2px;",
                    span { "Files" }
                    div {
                        style: "position: relative;",
                        button {
                            class: "btn-icon",
                            style: "width: 20px; height: 20px;",
                            title: "New File or Folder",
                            onclick: move |e| {
                                e.stop_propagation();
                                let is_open = *show_header_dropdown.peek();
                                show_header_dropdown.set(!is_open);
                            },
                            IconPlus { size: 12 }
                        }
                        if *show_header_dropdown.read() {
                            div {
                                style: "position: fixed; top: 0; left: 0; right: 0; bottom: 0; z-index: 1098;",
                                onclick: move |_| show_header_dropdown.set(false),
                            }
                            div {
                                class: "context-menu",
                                style: "right: 0; top: 22px; position: absolute; z-index: 1099; min-width: 120px;",
                                button {
                                    class: "context-menu-item",
                                    onclick: move |_| {
                                        show_header_dropdown.set(false);
                                        let mut s = state.write();
                                        s.new_file_target_folder = Some(std::path::PathBuf::new());
                                        s.show_new_note_dialog = true;
                                    },
                                    IconFile { size: 13 }
                                    span { "New File" }
                                }
                                button {
                                    class: "context-menu-item",
                                    onclick: move |_| {
                                        show_header_dropdown.set(false);
                                        let mut s = state.write();
                                        s.new_folder_target_parent = None;
                                        s.show_new_folder_dialog = true;
                                    },
                                    IconFolderPlus { size: 13 }
                                    span { "New Folder" }
                                }
                            }
                        }
                    }
                }

                {
                    let tree = crate::state::build_file_tree(&entries);
                    let mut visible_items = Vec::new();
                    flatten_visible_items(&tree, 0, &app_state.expanded_folders, &mut visible_items);

                    rsx! {
                        for item in visible_items {
                            match item {
                                VisibleTreeItem::Folder { name, relative_path, depth, is_expanded, is_empty } => {
                                    let pad_left = 8 + depth * 14;
                                    let p_click = relative_path.to_path_buf();
                                    let p_ctx = relative_path.to_path_buf();
                                    let name_str = name.to_string();
                                    let rel_display = relative_path.display().to_string();

                                    rsx! {
                                        div {
                                            key: "dir-{rel_display}",
                                            style: format!("padding: 4px 8px; padding-left: {pad_left}px; border-radius: 4px; display: flex; align-items: center; justify-content: space-between; cursor: pointer; color: var(--text-secondary); font-size: 12px; font-weight: 500; user-select: none; gap: 4px;"),
                                            onclick: move |_| {
                                                state.write().toggle_folder_expanded(&p_click);
                                            },
                                            oncontextmenu: move |evt: MouseEvent| {
                                                evt.prevent_default();
                                                evt.stop_propagation();
                                                let coords = evt.client_coordinates();
                                                context_menu.set(Some((ContextMenuTarget::Folder(p_ctx.clone()), coords.x, coords.y)));
                                            },
                                            div {
                                                style: "display: flex; align-items: center; gap: 5px; flex: 1; overflow: hidden;",
                                                span {
                                                    style: "display: inline-flex; align-items: center; opacity: 0.7;",
                                                    if is_expanded {
                                                        IconChevronDown { size: 11 }
                                                    } else {
                                                        IconChevronRight { size: 11 }
                                                    }
                                                }
                                                span {
                                                    style: "display: inline-flex; align-items: center; opacity: 0.8;",
                                                    if is_expanded {
                                                        IconFolderOpen { size: 13 }
                                                    } else {
                                                        IconFolder { size: 13 }
                                                    }
                                                }
                                                span {
                                                    style: "overflow: hidden; text-overflow: ellipsis; white-space: nowrap;",
                                                    title: "{name_str}",
                                                    "{name_str}"
                                                }
                                            }
                                        }
                                        if is_expanded && is_empty {
                                            {
                                                let empty_pad = 8 + (depth + 1) * 14;
                                                rsx! {
                                                    div {
                                                        key: "empty-{rel_display}",
                                                        style: format!("padding: 2px 8px; padding-left: {empty_pad}px; font-size: 11px; color: var(--text-muted); font-style: italic; user-select: none;"),
                                                        "(empty)"
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                                VisibleTreeItem::File { summary, depth } => {
                                    let summary_path = summary.relative_path.clone();
                                    let is_active = active_path.as_ref() == Some(&summary_path);
                                    let click_path = summary_path.clone();
                                    let context_path = summary_path.clone();
                                    let is_bookmarked = app_state.is_bookmarked(&summary_path);
                                    let active_bg = if is_active { "var(--bg-active)" } else { "transparent" };
                                    let active_color = if is_active { "var(--text-primary)" } else { "var(--text-secondary)" };
                                    let pad_left = 8 + depth * 14;
                                    let rel_display = summary_path.display().to_string();
                                    let title_str = summary.title.clone();

                                    rsx! {
                                        div {
                                            key: "file-{rel_display}",
                                            style: format!("padding: 4px 8px; padding-left: {pad_left}px; border-radius: 4px; display: flex; align-items: center; justify-content: space-between; cursor: pointer; background-color: {active_bg}; color: {active_color}; font-size: 13px; user-select: none; gap: 4px;"),
                                            onclick: move |_| {
                                                let mut s = state.write();
                                                s.active_view = crate::state::ActiveView::Editor;
                                                let _ = s.select_note(&click_path);
                                            },
                                            oncontextmenu: move |evt: MouseEvent| {
                                                evt.prevent_default();
                                                evt.stop_propagation();
                                                let coords = evt.client_coordinates();
                                                context_menu.set(Some((ContextMenuTarget::File(context_path.clone()), coords.x, coords.y)));
                                            },
                                            div {
                                                style: "display: flex; align-items: center; gap: 6px; flex: 1; overflow: hidden;",
                                                IconFile { size: 14, class: "opacity-70" }
                                                span {
                                                    style: "overflow: hidden; text-overflow: ellipsis; white-space: nowrap;",
                                                    title: "{title_str}",
                                                    "{title_str}"
                                                }
                                            }
                                            if is_bookmarked {
                                                span {
                                                    style: "color: var(--accent); margin-left: 4px; display: inline-flex;",
                                                    title: "Bookmarked",
                                                    IconBookmark { size: 11 }
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

        // Right-Click Context Menu (File, Folder, or Root)
        if let Some((target, x, y)) = context_menu.read().clone() {
            div {
                    style: "position: fixed; top: 0; left: 0; right: 0; bottom: 0; z-index: 1099;",
                    onclick: move |_| context_menu.set(None),
                    oncontextmenu: move |e| {
                        e.prevent_default();
                        context_menu.set(None);
                    },
                }
                div {
                    class: "context-menu",
                    style: "left: {x}px; top: {y}px; z-index: 1100;",
                    match target {
                        ContextMenuTarget::File(cm_path) => {
                            let open_path = cm_path.clone();
                            let bookmark_path = cm_path.clone();
                            let rename_path = cm_path.clone();
                            let delete_path = cm_path.clone();
                            let copy_path = cm_path.clone();
                            let is_bkmk = app_state.is_bookmarked(&cm_path);

                            rsx! {
                                button {
                                    class: "context-menu-item",
                                    onclick: move |_| {
                                        let mut s = state.write();
                                        s.active_view = crate::state::ActiveView::Editor;
                                        let _ = s.select_note(&open_path);
                                        context_menu.set(None);
                                    },
                                    IconNotes { size: 13 }
                                    span { "Open" }
                                }
                                button {
                                    class: "context-menu-item",
                                    onclick: move |_| {
                                        let mut s = state.write();
                                        s.toggle_bookmark(&bookmark_path);
                                        context_menu.set(None);
                                    },
                                    IconBookmark { size: 13 }
                                    span {
                                        if is_bkmk { "Remove Bookmark" } else { "Bookmark" }
                                    }
                                }
                                button {
                                    class: "context-menu-item",
                                    onclick: move |_| {
                                        let mut s = state.write();
                                        s.note_to_rename = Some(rename_path.clone());
                                        s.show_rename_dialog = true;
                                        context_menu.set(None);
                                    },
                                    IconEdit { size: 13 }
                                    span { "Rename…" }
                                }
                                button {
                                    class: "context-menu-item",
                                    onclick: move |_| {
                                        let p_str = copy_path.display().to_string();
                                        let mut s = state.write();
                                        s.status_message = format!("Copied path: {p_str}");
                                        context_menu.set(None);
                                    },
                                    IconCopy { size: 13 }
                                    span { "Copy Path" }
                                }
                                div { class: "dropdown-divider" }
                                button {
                                    class: "context-menu-item danger",
                                    onclick: move |_| {
                                        let mut s = state.write();
                                        s.note_to_delete = Some(delete_path.clone());
                                        s.show_delete_confirm_dialog = true;
                                        context_menu.set(None);
                                    },
                                    IconTrash { size: 13 }
                                    span { "Delete…" }
                                }
                            }
                        }
                        ContextMenuTarget::Folder(folder_path) => {
                            let new_file_folder = folder_path.clone();
                            let new_subfolder = folder_path.clone();
                            let rename_f = folder_path.clone();
                            let delete_f = folder_path.clone();

                            rsx! {
                                button {
                                    class: "context-menu-item",
                                    onclick: move |_| {
                                        let mut s = state.write();
                                        s.new_file_target_folder = Some(new_file_folder.clone());
                                        s.show_new_note_dialog = true;
                                        context_menu.set(None);
                                    },
                                    IconPlus { size: 13 }
                                    span { "New File" }
                                }
                                button {
                                    class: "context-menu-item",
                                    onclick: move |_| {
                                        let mut s = state.write();
                                        s.new_folder_target_parent = Some(new_subfolder.clone());
                                        s.show_new_folder_dialog = true;
                                        context_menu.set(None);
                                    },
                                    IconFolderPlus { size: 13 }
                                    span { "New Folder" }
                                }
                                button {
                                    class: "context-menu-item",
                                    onclick: move |_| {
                                        let mut s = state.write();
                                        s.folder_to_rename = Some(rename_f.clone());
                                        s.show_rename_folder_dialog = true;
                                        context_menu.set(None);
                                    },
                                    IconEdit { size: 13 }
                                    span { "Rename…" }
                                }
                                div { class: "dropdown-divider" }
                                button {
                                    class: "context-menu-item danger",
                                    onclick: move |_| {
                                        let mut s = state.write();
                                        s.folder_to_delete = Some(delete_f.clone());
                                        s.show_delete_folder_dialog = true;
                                        context_menu.set(None);
                                    },
                                    IconTrash { size: 13 }
                                    span { "Delete…" }
                                }
                            }
                        }
                        ContextMenuTarget::Root => {
                            rsx! {
                                button {
                                    class: "context-menu-item",
                                    onclick: move |_| {
                                        let mut s = state.write();
                                        s.new_file_target_folder = Some(std::path::PathBuf::new());
                                        s.show_new_note_dialog = true;
                                        context_menu.set(None);
                                    },
                                    IconPlus { size: 13 }
                                    span { "New File" }
                                }
                                button {
                                    class: "context-menu-item",
                                    onclick: move |_| {
                                        let mut s = state.write();
                                        s.new_folder_target_parent = None;
                                        s.show_new_folder_dialog = true;
                                        context_menu.set(None);
                                    },
                                    IconFolderPlus { size: 13 }
                                    span { "New Folder" }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
