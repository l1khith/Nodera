use dioxus::prelude::*;
use std::path::PathBuf;

use crate::icons::{IconFile, IconTasks};
use crate::state::{ActiveView, AppState};
use crate::strings::{actions, empty_states, nav, placeholders};

#[component]
pub fn TaskView(state: Signal<AppState>) -> Element {
    let app_state = state.read();
    let tasks = app_state.get_vault_tasks();
    let total_tasks = tasks.len();
    let completed_count = tasks.iter().filter(|t| t.checked).count();
    let active_count = total_tasks.saturating_sub(completed_count);
    let current_filter = app_state.task_filter.clone();

    rsx! {
        main { class: "pane-center",
            // Task View Toolbar
            header {
                style: "height: 48px; border-bottom: 1px solid var(--border); background-color: var(--bg-surface); display: flex; align-items: center; justify-content: space-between; padding: 0 20px;",
                div { style: "display: flex; align-items: center; gap: 12px;",
                    span { style: "font-size: 16px; font-weight: 700; color: var(--text-primary);", "{nav::GLOBAL_TASKS}" }
                    span {
                        class: "vault-badge",
                        "{active_count} active · {completed_count} completed"
                    }
                }

                div { style: "display: flex; align-items: center; gap: 8px;",
                    // Filter tabs
                    div {
                        style: "display: flex; border: 1px solid var(--border); border-radius: 6px; overflow: hidden; background-color: var(--bg-surface-elevated);",
                        button {
                            style: if current_filter.checked.is_none() { "padding: 4px 10px; font-size: 11px; font-weight: 600; background-color: var(--accent); color: #fff;" } else { "padding: 4px 10px; font-size: 11px; color: var(--text-secondary);" },
                            onclick: move |_| {
                                let mut s = state.write();
                                s.task_filter.checked = None;
                            },
                            "{actions::FILTER_ALL}"
                        }
                        button {
                            style: if current_filter.checked == Some(false) { "padding: 4px 10px; font-size: 11px; font-weight: 600; background-color: var(--accent); color: #fff;" } else { "padding: 4px 10px; font-size: 11px; color: var(--text-secondary);" },
                            onclick: move |_| {
                                let mut s = state.write();
                                s.task_filter.checked = Some(false);
                            },
                            "{actions::FILTER_TODO} ({active_count})"
                        }
                        button {
                            style: if current_filter.checked == Some(true) { "padding: 4px 10px; font-size: 11px; font-weight: 600; background-color: var(--accent); color: #fff;" } else { "padding: 4px 10px; font-size: 11px; color: var(--text-secondary);" },
                            onclick: move |_| {
                                let mut s = state.write();
                                s.task_filter.checked = Some(true);
                            },
                            "{actions::FILTER_DONE} ({completed_count})"
                        }
                    }

                    // Search filter within tasks
                    input {
                        style: "padding: 4px 8px; border-radius: 4px; border: 1px solid var(--border); background-color: var(--bg-surface-elevated); font-size: 12px; width: 160px; color: var(--text-primary);",
                        placeholder: placeholders::FILTER_TASKS,
                        value: "{current_filter.search_query.as_deref().unwrap_or_default()}",
                        oninput: move |evt| {
                            let mut s = state.write();
                            let val = evt.value();
                            s.task_filter.search_query = if val.trim().is_empty() { None } else { Some(val) };
                        }
                    }
                }
            }

            // Task List Body
            div {
                style: "flex: 1; overflow-y: auto; padding: 24px 36px; display: flex; flex-direction: column; gap: 8px;",
                if tasks.is_empty() {
                    div {
                        style: "padding: 60px 20px; text-align: center; color: var(--text-muted); display: flex; flex-direction: column; align-items: center; gap: 12px;",
                        div { style: "color: var(--text-muted); opacity: 0.6;",
                            IconTasks { size: 48 }
                        }
                        h3 { style: "font-size: 16px; color: var(--text-secondary);", "{empty_states::NO_TASKS_TITLE}" }
                        p { "{empty_states::NO_TASKS_DESC}" }
                    }
                } else {
                    for task in tasks {
                        {
                            let note_path = PathBuf::from(&task.note_path);
                            let note_path_nav = note_path.clone();
                            let line_num = task.line_number;
                            let is_checked = task.checked;
                            rsx! {
                                div {
                                    key: "{task.id}",
                                    style: "display: flex; align-items: center; justify-content: space-between; padding: 10px 14px; border-radius: 6px; background-color: var(--bg-surface); border: 1px solid var(--border-subtle); transition: background-color 0.1s;",
                                    div { style: "display: flex; align-items: center; gap: 12px; flex: 1;",
                                        input {
                                            r#type: "checkbox",
                                            style: "cursor: pointer; width: 16px; height: 16px; accent-color: var(--accent);",
                                            checked: is_checked,
                                            onchange: move |_| {
                                                let mut s = state.write();
                                                let _ = s.toggle_task_and_sync(&note_path, line_num);
                                            }
                                        }
                                        span {
                                            style: if is_checked {
                                                "color: var(--text-muted); text-decoration: line-through; font-size: 13px;"
                                            } else {
                                                "color: var(--text-primary); font-size: 13px;"
                                            },
                                            "{task.text}"
                                        }
                                    }

                                    // Origin note link
                                    button {
                                        style: "display: inline-flex; align-items: center; gap: 5px; font-size: 11px; color: var(--text-secondary); padding: 2px 8px; border-radius: 4px; background-color: var(--bg-surface-elevated); border: 1px solid var(--border-subtle); cursor: pointer;",
                                        title: "Navigate to note line {task.line_number}",
                                        onclick: move |_| {
                                            let mut s = state.write();
                                            s.active_view = ActiveView::Editor;
                                            let _ = s.select_note(&note_path_nav);
                                        },
                                        IconFile { size: 12 }
                                        span { "{task.note_title}:{task.line_number}" }
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
