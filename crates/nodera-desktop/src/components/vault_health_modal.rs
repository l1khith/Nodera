use dioxus::prelude::*;

use crate::icons::{
    IconActivity, IconClose, IconFile, IconLink, IconPlus, IconReviewQueue, IconTrash,
};
use crate::state::{ActiveView, AppState};

#[derive(Clone, Copy, PartialEq, Eq)]
enum HealthTab {
    BrokenLinks,
    Orphans,
}

#[component]
pub fn VaultHealthModal(state: Signal<AppState>) -> Element {
    let app_state = state.read();
    if !app_state.show_vault_health_modal {
        return rsx! {};
    }

    let mut active_tab = use_signal(|| HealthTab::BrokenLinks);

    let audit_report = app_state.audit_vault().unwrap_or_default();
    let knowledge_stats = app_state.get_knowledge_stats();

    let total_broken = audit_report.broken_links.len();
    let total_orphans = audit_report.orphan_notes.len();
    let total_notes = audit_report.total_notes;
    let total_links = audit_report.total_links;

    rsx! {
        div {
            class: "modal-overlay",
            onclick: move |_| {
                state.write().show_vault_health_modal = false;
            },
            div {
                class: "modal-dialog",
                style: "max-width: 720px; width: 90%; max-height: 85vh; display: flex; flex-direction: column; overflow: hidden; padding: 20px;",
                onclick: move |e| e.stop_propagation(),

                // Header
                div {
                    style: "display: flex; align-items: center; justify-content: space-between; margin-bottom: 16px; border-bottom: 1px solid var(--border); padding-bottom: 12px;",
                    div {
                        style: "display: flex; align-items: center; gap: 10px;",
                        div {
                            style: "width: 32px; height: 32px; border-radius: 8px; background: rgba(91, 108, 255, 0.15); color: var(--accent); display: flex; align-items: center; justify-content: center;",
                            IconActivity { size: 18 }
                        }
                        div {
                            h3 { style: "margin: 0; font-size: 16px; font-weight: 600; color: var(--text-primary);", "Vault Health Doctor" }
                            p { style: "margin: 0; font-size: 12px; color: var(--text-muted);", "Diagnose broken internal wikilinks and manage disconnected orphan notes" }
                        }
                    }
                    button {
                        class: "btn-icon",
                        title: "Close (Esc)",
                        onclick: move |_| {
                            state.write().show_vault_health_modal = false;
                        },
                        IconClose { size: 16 }
                    }
                }

                // Metric Cards Row
                div {
                    style: "display: grid; grid-template-columns: repeat(4, 1fr); gap: 10px; margin-bottom: 16px;",
                    div {
                        style: "background: var(--bg-surface-soft, rgba(255, 255, 255, 0.03)); border: 1px solid var(--border); border-radius: 8px; padding: 10px 14px;",
                        div { style: "font-size: 11px; color: var(--text-muted); text-transform: uppercase; font-weight: 600;", "Total Notes" }
                        div { style: "font-size: 20px; font-weight: 700; color: var(--text-primary); margin-top: 2px;", "{total_notes}" }
                    }
                    div {
                        style: "background: var(--bg-surface-soft, rgba(255, 255, 255, 0.03)); border: 1px solid var(--border); border-radius: 8px; padding: 10px 14px;",
                        div { style: "font-size: 11px; color: var(--text-muted); text-transform: uppercase; font-weight: 600;", "Valid Links" }
                        div { style: "font-size: 20px; font-weight: 700; color: var(--text-primary); margin-top: 2px;", "{total_links}" }
                    }
                    div {
                        style: format!(
                            "background: {}; border: 1px solid {}; border-radius: 8px; padding: 10px 14px;",
                            if total_broken > 0 { "rgba(244, 63, 94, 0.08)" } else { "rgba(16, 185, 129, 0.08)" },
                            if total_broken > 0 { "rgba(244, 63, 94, 0.3)" } else { "rgba(16, 185, 129, 0.3)" },
                        ),
                        div {
                            style: format!("font-size: 11px; text-transform: uppercase; font-weight: 600; color: {};", if total_broken > 0 { "#F43F5E" } else { "#10B981" }),
                            "Broken Links"
                        }
                        div {
                            style: format!("font-size: 20px; font-weight: 700; color: {}; margin-top: 2px;", if total_broken > 0 { "#F43F5E" } else { "#10B981" }),
                            "{total_broken}"
                        }
                    }
                    div {
                        style: format!(
                            "background: {}; border: 1px solid {}; border-radius: 8px; padding: 10px 14px;",
                            if total_orphans > 0 { "rgba(245, 158, 11, 0.08)" } else { "rgba(16, 185, 129, 0.08)" },
                            if total_orphans > 0 { "rgba(245, 158, 11, 0.3)" } else { "rgba(16, 185, 129, 0.3)" },
                        ),
                        div {
                            style: format!("font-size: 11px; text-transform: uppercase; font-weight: 600; color: {};", if total_orphans > 0 { "#F59E0B" } else { "#10B981" }),
                            "Orphan Notes"
                        }
                        div {
                            style: format!("font-size: 20px; font-weight: 700; color: {}; margin-top: 2px;", if total_orphans > 0 { "#F59E0B" } else { "#10B981" }),
                            "{total_orphans}"
                        }
                    }
                }

                // Knowledge Workflow Health Card
                div {
                    style: "display: flex; align-items: center; justify-content: space-between; background: var(--bg-surface-elevated); border: 1px solid var(--border); border-radius: 8px; padding: 12px 16px; margin-bottom: 16px; gap: 16px;",
                    div { style: "display: flex; flex-direction: column; gap: 4px;",
                        div { style: "font-size: 11px; text-transform: uppercase; font-weight: 700; color: var(--text-secondary); letter-spacing: 0.5px;", "Knowledge Workflow Breakdown" }
                        div { style: "display: flex; align-items: center; gap: 12px; font-size: 12px; color: var(--text-muted);",
                            span { strong { style: "color: var(--text-primary);", "{knowledge_stats.rough_count}" } " rough" }
                            span { "·" }
                            span { strong { style: "color: #10B981;", "{knowledge_stats.permanent_count}" } " permanent" }
                            span { "·" }
                            span { strong { style: "color: var(--accent);", "{knowledge_stats.source_count}" } " sources" }
                            span { "·" }
                            span { strong { style: "color: var(--text-primary);", "{knowledge_stats.index_count}" } " index" }
                            span { "·" }
                            span { strong { style: "color: #F59E0B;", "{knowledge_stats.unlinked_count}" } " unlinked thoughts" }
                        }
                    }
                    button {
                        class: "btn-action btn-primary",
                        style: "padding: 6px 12px; font-size: 12px; display: inline-flex; align-items: center; gap: 6px; white-space: nowrap;",
                        title: "Open Review Queue to process rough notes and unlinked thoughts",
                        onclick: move |_| {
                            let mut s = state.write();
                            s.show_vault_health_modal = false;
                            s.active_view = ActiveView::ReviewQueue;
                        },
                        IconReviewQueue { size: 14 }
                        span { "Review Queue" }
                    }
                }

                // Tabs Navigation
                div {
                    style: "display: flex; gap: 8px; border-bottom: 1px solid var(--border); margin-bottom: 12px;",
                    button {
                        class: if *active_tab.read() == HealthTab::BrokenLinks { "tab-btn active" } else { "tab-btn" },
                        style: "padding: 6px 14px; font-size: 13px; font-weight: 500; cursor: pointer;",
                        onclick: move |_| active_tab.set(HealthTab::BrokenLinks),
                        span { "Broken Links ({total_broken})" }
                    }
                    button {
                        class: if *active_tab.read() == HealthTab::Orphans { "tab-btn active" } else { "tab-btn" },
                        style: "padding: 6px 14px; font-size: 13px; font-weight: 500; cursor: pointer;",
                        onclick: move |_| active_tab.set(HealthTab::Orphans),
                        span { "Orphan Notes ({total_orphans})" }
                    }
                }

                // Tab Content Area
                div {
                    style: "flex: 1; overflow-y: auto; display: flex; flex-direction: column; gap: 8px; padding-right: 4px;",

                    if *active_tab.read() == HealthTab::BrokenLinks {
                        if audit_report.broken_links.is_empty() {
                            div {
                                style: "padding: 40px 20px; text-align: center; color: var(--text-muted);",
                                div { style: "font-size: 15px; font-weight: 600; color: #10B981; margin-bottom: 4px;", "No broken links found!" }
                                div { style: "font-size: 13px;", "Every internal [[wikilink]] in your vault successfully resolves to a note." }
                            }
                        } else {
                            for group in audit_report.broken_links.iter() {
                                {
                                    let target_title = group.target.clone();
                                    let occ_count = group.occurrences.len();
                                    let occ_list = group.occurrences.clone();
                                    let target_create = target_title.clone();
                                    let target_unlink = target_title.clone();

                                    rsx! {
                                        div {
                                            key: "{target_title}",
                                            style: "background: var(--bg-surface-soft, rgba(255, 255, 255, 0.02)); border: 1px solid var(--border); border-radius: 8px; padding: 12px 14px; display: flex; flex-direction: column; gap: 8px;",
                                            div {
                                                style: "display: flex; align-items: center; justify-content: space-between; flex-wrap: wrap; gap: 8px;",
                                                div {
                                                    style: "display: flex; align-items: center; gap: 8px;",
                                                    span { style: "font-size: 14px; font-weight: 600; color: var(--text-primary); font-family: monospace;", "[[{target_title}]]" }
                                                    span {
                                                        style: "font-size: 11px; font-weight: 500; background: rgba(244, 63, 94, 0.15); color: #F43F5E; padding: 2px 8px; border-radius: 10px;",
                                                        "{occ_count} references"
                                                    }
                                                }
                                                div {
                                                    style: "display: flex; align-items: center; gap: 6px;",
                                                    button {
                                                        class: "btn-action btn-primary",
                                                        style: "font-size: 12px; padding: 4px 10px; display: inline-flex; align-items: center; gap: 4px;",
                                                        title: "Create note with this title to satisfy links",
                                                        onclick: move |_| {
                                                            let mut s = state.write();
                                                            let _ = s.fix_broken_link_create_note(&target_create);
                                                        },
                                                        IconPlus { size: 12 }
                                                        span { "Create Note" }
                                                    }
                                                    button {
                                                        class: "btn-action",
                                                        style: "font-size: 12px; padding: 4px 10px; display: inline-flex; align-items: center; gap: 4px; color: var(--text-muted);",
                                                        title: "Convert references from [[link]] syntax to plain text",
                                                        onclick: move |_| {
                                                            let mut s = state.write();
                                                            let _ = s.fix_broken_link_unlink(&target_unlink);
                                                        },
                                                        IconLink { size: 12 }
                                                        span { "Unlink" }
                                                    }
                                                }
                                            }
                                            // Referencing notes list
                                            div {
                                                style: "display: flex; flex-direction: column; gap: 3px; font-size: 12px; color: var(--text-muted); border-top: 1px solid rgba(255, 255, 255, 0.05); padding-top: 6px;",
                                                for occ in occ_list.iter() {
                                                    {
                                                        let source_path = occ.source_path.clone();
                                                        rsx! {
                                                            div {
                                                                key: "{occ.source_path.display()}_{occ.line_number}",
                                                                style: "display: flex; align-items: center; gap: 6px;",
                                                                IconFile { size: 12 }
                                                                button {
                                                                    class: "link-item",
                                                                    style: "font-size: 12px; padding: 0; text-align: left;",
                                                                    onclick: move |_| {
                                                                        let mut s = state.write();
                                                                        let _ = s.select_note(&source_path);
                                                                        s.show_vault_health_modal = false;
                                                                    },
                                                                    "{occ.source_path.display()}"
                                                                }
                                                                span { style: "color: var(--text-muted); font-size: 11px;", "(line {occ.line_number})" }
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
                    } else {
                        // Orphan notes tab
                        if audit_report.orphan_notes.is_empty() {
                            div {
                                style: "padding: 40px 20px; text-align: center; color: var(--text-muted);",
                                div { style: "font-size: 15px; font-weight: 600; color: #10B981; margin-bottom: 4px;", "No orphan notes found!" }
                                div { style: "font-size: 13px;", "All notes in your vault are linked and connected within the knowledge graph." }
                            }
                        } else {
                            {
                                let orphan_paths = audit_report.orphan_notes.clone();
                                let batch_all_paths = orphan_paths.clone();

                                rsx! {
                                    div {
                                        style: "display: flex; align-items: center; justify-content: space-between; margin-bottom: 8px; padding: 0 4px;",
                                        span { style: "font-size: 12px; color: var(--text-muted);", "{orphan_paths.len()} notes have 0 incoming and 0 outgoing links." }
                                        button {
                                            class: "btn-action",
                                            style: "font-size: 12px; padding: 4px 10px; color: var(--danger); display: inline-flex; align-items: center; gap: 4px;",
                                            onclick: move |_| {
                                                let mut s = state.write();
                                                let _ = s.batch_trash_orphans(&batch_all_paths);
                                            },
                                            IconTrash { size: 12 }
                                            span { "Move All Orphans to Trash" }
                                        }
                                    }

                                    for path in orphan_paths.iter() {
                                        {
                                            let note_path = path.clone();
                                            let path_open = path.clone();
                                            let path_trash = path.clone();
                                            let title = path.file_stem().and_then(|s| s.to_str()).unwrap_or("Untitled").to_string();

                                            rsx! {
                                                div {
                                                    key: "{note_path.display()}",
                                                    style: "background: var(--bg-surface-soft, rgba(255, 255, 255, 0.02)); border: 1px solid var(--border); border-radius: 8px; padding: 10px 14px; display: flex; align-items: center; justify-content: space-between;",
                                                    div {
                                                        style: "display: flex; align-items: center; gap: 8px;",
                                                        IconFile { size: 14 }
                                                        div {
                                                            div { style: "font-size: 13px; font-weight: 600; color: var(--text-primary);", "{title}" }
                                                            div { style: "font-size: 11px; color: var(--text-muted);", "{note_path.display()}" }
                                                        }
                                                    }
                                                    div {
                                                        style: "display: flex; align-items: center; gap: 6px;",
                                                        button {
                                                            class: "btn-action",
                                                            style: "font-size: 11px; padding: 3px 8px;",
                                                            onclick: move |_| {
                                                                let mut s = state.write();
                                                                let _ = s.select_note(&path_open);
                                                                s.show_vault_health_modal = false;
                                                            },
                                                            "Open"
                                                        }
                                                        button {
                                                            class: "btn-action",
                                                            style: "font-size: 11px; padding: 3px 8px; color: var(--danger);",
                                                            title: "Move to Trash (.trash/)",
                                                            onclick: move |_| {
                                                                let mut s = state.write();
                                                                let _ = s.trash_note(&path_trash);
                                                            },
                                                            IconTrash { size: 12 }
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
