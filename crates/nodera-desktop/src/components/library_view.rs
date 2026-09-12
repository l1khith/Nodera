use dioxus::prelude::*;

use crate::icons::*;
use crate::state::AppState;
use crate::strings::{actions, empty_states, nav, placeholders};

/// Library view for browsing long-form documents and imported books.
#[component]
pub fn LibraryView(state: Signal<AppState>) -> Element {
    let app_state = state.read();
    let books = app_state.get_library_books();
    let query = app_state.library_search_query.clone();
    let total_books = books.len();
    let doc_count_label = if total_books == 1 {
        "1 document available".to_string()
    } else {
        format!("{total_books} documents available")
    };

    rsx! {
        div {
            class: "library-view",
            style: "display: flex; flex-direction: column; height: 100%; width: 100%; overflow: hidden; background: var(--bg-primary);",

            // Header toolbar
            div {
                class: "library-header",
                style: "display: flex; align-items: center; justify-content: space-between; padding: 16px 24px; border-bottom: 1px solid var(--border-color); background: var(--bg-secondary);",

                div {
                    style: "display: flex; align-items: center; gap: 12px;",
                    IconLibrary { size: 24 }
                    div {
                        h2 {
                            style: "margin: 0; font-size: 18px; font-weight: 600; color: var(--text-primary);",
                            "{nav::LIBRARY}"
                        }
                        span {
                            style: "font-size: 12px; color: var(--text-muted);",
                            "{doc_count_label}"
                        }
                    }
                }

                div {
                    style: "display: flex; align-items: center; gap: 12px;",

                    // Search input
                    input {
                        r#type: "text",
                        placeholder: placeholders::SEARCH_LIBRARY,
                        value: "{query}",
                        style: "padding: 6px 12px; border-radius: 6px; border: 1px solid var(--border-color); background: var(--bg-primary); color: var(--text-primary); font-size: 13px; width: 220px; outline: none;",
                        oninput: move |evt| {
                            state.write().library_search_query = evt.value();
                        }
                    }

                    // Import PDF button
                    button {
                        class: "btn btn-primary",
                        style: "display: flex; align-items: center; gap: 6px; padding: 6px 14px; border-radius: 6px; background: var(--accent-color); color: #ffffff; border: none; font-size: 13px; font-weight: 500; cursor: pointer;",
                        onclick: move |_| {
                            state.write().open_pdf_import_modal();
                        },
                        IconImport { size: 14 }
                        span { "{actions::IMPORT_PDF_BTN}" }
                    }
                }
            }

            // Book grid or empty state
            div {
                class: "library-content",
                style: "flex: 1; overflow-y: auto; padding: 24px;",

                if books.is_empty() {
                    div {
                        class: "library-empty-state",
                        style: "display: flex; flex-direction: column; align-items: center; justify-content: center; height: 360px; text-align: center; max-width: 420px; margin: 40px auto; padding: 32px; border: 2px dashed var(--border-color); border-radius: 12px; background: var(--bg-secondary);",

                        IconLibrary { size: 48 }
                        h3 {
                            style: "margin: 12px 0 8px 0; font-size: 16px; font-weight: 600; color: var(--text-primary);",
                            if query.is_empty() {
                                "{empty_states::NO_BOOKS_TITLE}"
                            } else {
                                "{empty_states::NO_BOOKS_SEARCH_TITLE}"
                            }
                        }
                        p {
                            style: "margin: 0 0 20px 0; font-size: 13px; line-height: 1.5; color: var(--text-muted);",
                            if query.is_empty() {
                                "{empty_states::NO_BOOKS_DESC}"
                            } else {
                                "{empty_states::NO_BOOKS_SEARCH_DESC}"
                            }
                        }

                        if query.is_empty() {
                            button {
                                class: "btn btn-primary",
                                style: "display: inline-flex; align-items: center; gap: 8px; padding: 8px 18px; border-radius: 6px; background: var(--accent-color); color: #ffffff; border: none; font-size: 13px; font-weight: 500; cursor: pointer;",
                                onclick: move |_| {
                                    state.write().open_pdf_import_modal();
                                },
                                IconImport { size: 14 }
                                span { "{actions::IMPORT_PDF}" }
                            }
                        } else {
                            button {
                                class: "btn btn-secondary",
                                style: "padding: 6px 14px; border-radius: 6px; border: 1px solid var(--border-color); background: var(--bg-primary); color: var(--text-primary); font-size: 13px; cursor: pointer;",
                                onclick: move |_| {
                                    state.write().library_search_query.clear();
                                },
                                "{actions::CLEAR_SEARCH}"
                            }
                        }
                    }
                } else {
                    div {
                        class: "library-grid",
                        style: "display: grid; grid-template-columns: repeat(auto-fill, minmax(320px, 1fr)); gap: 20px;",

                        for book in books {
                            {
                                let book_path_read = book.relative_path.clone();
                                let book_path_edit = book.relative_path.clone();
                                let title = book.title.clone();
                                let pages = book.pages;
                                let chapters = book.chapters;
                                let word_count = book.word_count;
                                let progress = book.reading_progress_pct;
                                let source = book.source.clone();
                                let tags = book.tags.clone();

                                rsx! {
                                    div {
                                        key: "{book.relative_path.to_string_lossy()}",
                                        class: "library-card",
                                        style: "display: flex; flex-direction: column; border: 1px solid var(--border-color); border-radius: 10px; background: var(--bg-secondary); overflow: hidden; box-shadow: 0 2px 6px rgba(0,0,0,0.05); transition: transform 0.15s ease, box-shadow 0.15s ease;",

                                        // Card Cover Header
                                        div {
                                            style: "padding: 16px; background: linear-gradient(135deg, var(--bg-secondary), var(--bg-tertiary, var(--bg-primary))); border-bottom: 1px solid var(--border-color); display: flex; align-items: flex-start; gap: 12px;",

                                            div {
                                                style: "width: 40px; height: 52px; border-radius: 4px; background: var(--accent-color); color: #ffffff; display: flex; align-items: center; justify-content: center; font-size: 20px; flex-shrink: 0; box-shadow: 0 2px 4px rgba(0,0,0,0.2);",
                                                IconBook { size: 22 }
                                            }

                                            div {
                                                style: "flex: 1; min-width: 0;",
                                                h4 {
                                                    style: "margin: 0 0 4px 0; font-size: 15px; font-weight: 600; color: var(--text-primary); overflow: hidden; text-overflow: ellipsis; white-space: nowrap;",
                                                    title: "{title}",
                                                    "{title}"
                                                }
                                                if let Some(src) = source {
                                                    div {
                                                        style: "font-size: 11px; color: var(--text-muted); overflow: hidden; text-overflow: ellipsis; white-space: nowrap;",
                                                        title: "{src}",
                                                        "Source: {src}"
                                                    }
                                                }
                                            }
                                        }

                                        // Card Body & Metadata
                                        div {
                                            style: "padding: 14px 16px; flex: 1; display: flex; flex-direction: column; gap: 10px;",

                                            // Stats badges
                                            div {
                                                style: "display: flex; flex-wrap: wrap; gap: 8px; font-size: 12px; color: var(--text-secondary);",

                                                if pages > 0 {
                                                    span {
                                                        style: "display: inline-flex; align-items: center; gap: 4px; background: var(--bg-primary); padding: 3px 8px; border-radius: 4px; border: 1px solid var(--border-color);",
                                                        IconFile { size: 12 }
                                                        span { "{pages} pages" }
                                                    }
                                                }

                                                if chapters > 0 {
                                                    span {
                                                        style: "display: inline-flex; align-items: center; gap: 4px; background: var(--bg-primary); padding: 3px 8px; border-radius: 4px; border: 1px solid var(--border-color);",
                                                        IconList { size: 12 }
                                                        span { "{chapters} chapters" }
                                                    }
                                                }

                                                span {
                                                    style: "display: inline-flex; align-items: center; gap: 4px; background: var(--bg-primary); padding: 3px 8px; border-radius: 4px; border: 1px solid var(--border-color);",
                                                    IconNotes { size: 12 }
                                                    span { "{word_count} words" }
                                                }
                                            }

                                            // Tags
                                            if !tags.is_empty() {
                                                div {
                                                    style: "display: flex; flex-wrap: wrap; gap: 4px;",
                                                    for tag in tags.iter().take(4) {
                                                        span {
                                                            key: "{tag}",
                                                            style: "font-size: 11px; padding: 2px 6px; border-radius: 3px; background: rgba(59, 130, 246, 0.1); color: var(--accent-color); font-weight: 500;",
                                                            "#{tag}"
                                                        }
                                                    }
                                                    if tags.len() > 4 {
                                                        span {
                                                            style: "font-size: 11px; color: var(--text-muted); align-self: center;",
                                                            "+{tags.len() - 4}"
                                                        }
                                                    }
                                                }
                                            }

                                            // Progress bar
                                            div {
                                                style: "margin-top: auto; display: flex; flex-direction: column; gap: 4px;",
                                                div {
                                                    style: "display: flex; justify-content: space-between; font-size: 11px; color: var(--text-muted);",
                                                    span { "Reading progress" }
                                                    span { "{progress}%" }
                                                }
                                                div {
                                                    style: "height: 6px; border-radius: 3px; background: var(--border-color); overflow: hidden;",
                                                    div {
                                                        style: "height: 100%; width: {progress}%; background: var(--accent-color); border-radius: 3px; transition: width 0.2s ease;",
                                                    }
                                                }
                                            }
                                        }

                                        // Card Footer Actions
                                        div {
                                            style: "padding: 10px 16px; border-top: 1px solid var(--border-color); background: var(--bg-primary); display: flex; justify-content: flex-end; gap: 8px;",

                                            button {
                                                class: "btn btn-secondary",
                                                style: "display: flex; align-items: center; gap: 6px; padding: 6px 12px; border-radius: 5px; border: 1px solid var(--border-color); background: var(--bg-secondary); color: var(--text-primary); font-size: 12px; cursor: pointer;",
                                                onclick: move |_| {
                                                    let _ = state.write().open_book_in_editor(&book_path_edit);
                                                },
                                                IconEdit { size: 13 }
                                                span { "{actions::EDIT}" }
                                            }

                                            button {
                                                class: "btn btn-primary",
                                                style: "display: flex; align-items: center; gap: 6px; padding: 6px 14px; border-radius: 5px; background: var(--accent-color); color: #ffffff; border: none; font-size: 12px; font-weight: 500; cursor: pointer;",
                                                onclick: move |_| {
                                                    let _ = state.write().open_book_in_reader(&book_path_read);
                                                },
                                                IconBook { size: 13 }
                                                span { "{actions::READ}" }
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
