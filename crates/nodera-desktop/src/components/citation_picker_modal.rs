use dioxus::prelude::*;

use crate::icons::{IconClose, IconLibrary, IconPlus, IconQuote, IconSearch};
use crate::state::AppState;

#[component]
pub fn CitationPickerModal(state: Signal<AppState>) -> Element {
    let app_state = state.read();
    if !app_state.show_citation_picker_modal {
        return rsx! {};
    }

    let mut search_query = use_signal(String::new);
    let q = search_query.read().clone();
    let matches: Vec<_> = app_state
        .bib_library
        .search(&q)
        .into_iter()
        .cloned()
        .collect();
    let total_entries = app_state.bib_library.entries.len();

    rsx! {
        div {
            class: "modal-overlay",
            onclick: move |_| {
                state.write().show_citation_picker_modal = false;
            },
            div {
                class: "modal-dialog",
                style: "max-width: 680px; width: 90%; max-height: 80vh; display: flex; flex-direction: column; overflow: hidden; padding: 20px;",
                onclick: move |e| e.stop_propagation(),

                // Header
                div {
                    style: "display: flex; align-items: center; justify-content: space-between; margin-bottom: 14px; border-bottom: 1px solid var(--border); padding-bottom: 12px;",
                    div {
                        style: "display: flex; align-items: center; gap: 10px;",
                        div {
                            style: "width: 32px; height: 32px; border-radius: 8px; background: rgba(91, 108, 255, 0.15); color: var(--accent); display: flex; align-items: center; justify-content: center;",
                            IconQuote { size: 18 }
                        }
                        div {
                            h3 { style: "margin: 0; font-size: 16px; font-weight: 600; color: var(--text-primary);", "Citation & Bibliography Picker" }
                            p { style: "margin: 0; font-size: 12px; color: var(--text-muted);", "Insert Zotero / BibTeX citations into your active note" }
                        }
                    }
                    button {
                        class: "btn-icon",
                        title: "Close (Esc)",
                        onclick: move |_| {
                            state.write().show_citation_picker_modal = false;
                        },
                        IconClose { size: 16 }
                    }
                }

                // Search Bar
                div {
                    style: "position: relative; margin-bottom: 14px;",
                    div {
                        style: "position: absolute; left: 12px; top: 50%; transform: translateY(-50%); color: var(--text-muted); pointer-events: none;",
                        IconSearch { size: 16 }
                    }
                    input {
                        r#type: "text",
                        class: "form-input",
                        style: "width: 100%; padding-left: 36px; padding-top: 8px; padding-bottom: 8px; font-size: 14px; background: var(--bg-surface-elevated); border: 1px solid var(--border); border-radius: 6px;",
                        placeholder: "Search by author, title, year, or citation key...",
                        value: "{search_query}",
                        autofocus: true,
                        oninput: move |e| search_query.set(e.value()),
                    }
                }

                // Status Bar / Count
                div {
                    style: "display: flex; align-items: center; justify-content: space-between; margin-bottom: 10px; font-size: 12px; color: var(--text-muted);",
                    span { "Showing {matches.len()} of {total_entries} references" }
                    if !app_state.bib_library.source_files.is_empty() {
                        span {
                            style: "display: flex; align-items: center; gap: 4px;",
                            IconLibrary { size: 12 }
                            "{app_state.bib_library.source_files.len()} .bib source(s)"
                        }
                    }
                }

                // List of Citations
                div {
                    style: "flex: 1; overflow-y: auto; display: flex; flex-direction: column; gap: 10px; min-height: 150px; max-height: 480px; padding-right: 4px;",

                    if total_entries == 0 {
                        div {
                            style: "text-align: center; padding: 40px 20px; color: var(--text-muted);",
                            div {
                                style: "margin-bottom: 12px; display: flex; justify-content: center; opacity: 0.6;",
                                IconLibrary { size: 36 }
                            }
                            p { style: "font-weight: 500; font-size: 14px; margin-bottom: 6px; color: var(--text-primary);", "No BibTeX References Found" }
                            p { style: "font-size: 12px; max-width: 440px; margin: 0 auto; line-height: 1.5;",
                                "Add any .bib file (e.g. exported from Zotero, Mendeley, or Google Scholar) anywhere in your vault directory. Nodera will automatically index citations."
                            }
                        }
                    } else if matches.is_empty() {
                        div {
                            style: "text-align: center; padding: 30px; color: var(--text-muted); font-size: 13px;",
                            "No references match your query."
                        }
                    } else {
                        for entry in matches {
                            {
                                let citekey = entry.citation_key.clone();
                                let citekey_for_insert = citekey.clone();
                                let citekey_for_full = citekey.clone();
                                let title = entry.title.clone().unwrap_or_else(|| entry.citation_key.clone());
                                let authors_str = if entry.authors.is_empty() {
                                    "Unknown Author".to_string()
                                } else {
                                    entry.authors.join(", ")
                                };
                                let year_str = entry.year.clone().unwrap_or_else(|| "n.d.".to_string());
                                let journal = entry.journal_or_book.clone();

                                rsx! {
                                    div {
                                        key: "{citekey}",
                                        style: "background: var(--bg-surface-elevated); border: 1px solid var(--border); border-radius: 8px; padding: 12px; display: flex; flex-direction: column; gap: 6px; transition: border-color 0.15s ease;",
                                        div {
                                            style: "display: flex; align-items: flex-start; justify-content: space-between; gap: 10px;",
                                            div {
                                                style: "flex: 1;",
                                                div {
                                                    style: "display: flex; align-items: center; gap: 6px; margin-bottom: 4px;",
                                                    span {
                                                        style: "background: rgba(91, 108, 255, 0.12); color: var(--accent); font-family: monospace; font-size: 11px; font-weight: 600; padding: 2px 6px; border-radius: 4px;",
                                                        "@{citekey}"
                                                    }
                                                    span {
                                                        style: "font-size: 11px; background: var(--bg-surface); border: 1px solid var(--border); padding: 1px 6px; border-radius: 4px; color: var(--text-muted);",
                                                        "{year_str}"
                                                    }
                                                    span {
                                                        style: "font-size: 11px; color: var(--text-muted); text-transform: uppercase;",
                                                        "{entry.entry_type}"
                                                    }
                                                }
                                                h4 {
                                                    style: "margin: 0 0 4px 0; font-size: 14px; font-weight: 600; color: var(--text-primary); line-height: 1.4;",
                                                    "{title}"
                                                }
                                                p {
                                                    style: "margin: 0; font-size: 12px; color: var(--text-muted);",
                                                    "{authors_str}"
                                                }
                                                if let Some(ref j) = journal {
                                                    p {
                                                        style: "margin: 2px 0 0 0; font-size: 11px; color: var(--text-secondary); font-style: italic;",
                                                        "{j}"
                                                    }
                                                }
                                            }

                                            // Action Buttons
                                            div {
                                                style: "display: flex; flex-direction: column; gap: 6px; min-width: 140px;",
                                                button {
                                                    class: "btn-primary",
                                                    style: "font-size: 11px; padding: 4px 8px; width: 100%; display: flex; align-items: center; justify-content: center; gap: 4px;",
                                                    onclick: move |_| {
                                                        state.write().insert_citation(&citekey_for_insert, false);
                                                        state.write().show_citation_picker_modal = false;
                                                    },
                                                    IconPlus { size: 12 }
                                                    "Insert [@{citekey}]"
                                                }
                                                button {
                                                    class: "btn-secondary",
                                                    style: "font-size: 11px; padding: 4px 8px; width: 100%; display: flex; align-items: center; justify-content: center; gap: 4px;",
                                                    onclick: move |_| {
                                                        state.write().insert_citation(&citekey_for_full, true);
                                                        state.write().show_citation_picker_modal = false;
                                                    },
                                                    "Full Reference"
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
