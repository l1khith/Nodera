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

            // Header toolbar
            div {
                class: "library-header",

                div {
                    style: "display: flex; align-items: center; gap: 14px;",
                    div {
                        style: "width: 38px; height: 38px; border-radius: 8px; background: rgba(91, 108, 255, 0.12); border: 1px solid rgba(91, 108, 255, 0.25); display: flex; align-items: center; justify-content: center; color: var(--accent);",
                        IconLibrary { size: 22 }
                    }
                    div {
                        h2 {
                            style: "margin: 0; font-size: 18px; font-weight: 700; color: var(--text-primary); letter-spacing: -0.2px;",
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
                    div {
                        style: "display: flex; align-items: center; gap: 8px; padding: 6px 12px; border-radius: 6px; border: 1px solid var(--border); background: var(--bg-surface-elevated); width: 240px;",
                        IconSearch { size: 14, class: "opacity-60" }
                        input {
                            r#type: "text",
                            placeholder: placeholders::SEARCH_LIBRARY,
                            value: "{query}",
                            style: "border: none; background: transparent; color: var(--text-primary); font-size: 12px; outline: none; width: 100%;",
                            oninput: move |evt| {
                                state.write().library_search_query = evt.value();
                            }
                        }
                        if !query.is_empty() {
                            button {
                                class: "btn-icon",
                                style: "padding: 2px;",
                                onclick: move |_| {
                                    state.write().library_search_query.clear();
                                },
                                IconClose { size: 12 }
                            }
                        }
                    }

                    // Import PDF button
                    button {
                        class: "btn-primary",
                        style: "display: inline-flex; align-items: center; gap: 7px; padding: 7px 16px; border-radius: 6px; background: var(--accent); color: #ffffff; border: 1px solid transparent; font-size: 13px; font-weight: 600; cursor: pointer; box-shadow: 0 2px 8px rgba(91, 108, 255, 0.35);",
                        onclick: move |_| {
                            state.write().open_pdf_import_modal();
                        },
                        IconImport { size: 15 }
                        span { "{actions::IMPORT_PDF_BTN}" }
                    }
                }
            }

            // Book grid or empty state
            div {
                class: "library-content",

                if books.is_empty() {
                    div {
                        class: "library-empty-state",
                        style: "display: flex; flex-direction: column; align-items: center; justify-content: center; height: 360px; text-align: center; max-width: 440px; margin: 40px auto; padding: 36px 32px; border: 2px dashed var(--border); border-radius: 12px; background: var(--bg-surface);",

                        div {
                            style: "width: 64px; height: 64px; border-radius: 12px; background: var(--bg-surface-elevated); border: 1px solid var(--border); display: flex; align-items: center; justify-content: center; color: var(--text-muted); margin-bottom: 8px;",
                            IconLibrary { size: 32 }
                        }
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
                                class: "btn-primary",
                                style: "display: inline-flex; align-items: center; gap: 8px; padding: 8px 20px; border-radius: 6px; background: var(--accent); color: #ffffff; border: none; font-size: 13px; font-weight: 600; cursor: pointer; box-shadow: 0 2px 8px rgba(91, 108, 255, 0.35);",
                                onclick: move |_| {
                                    state.write().open_pdf_import_modal();
                                },
                                IconImport { size: 15 }
                                span { "{actions::IMPORT_PDF}" }
                            }
                        } else {
                            button {
                                class: "btn-secondary",
                                style: "padding: 6px 16px; border-radius: 6px; border: 1px solid var(--border); background: var(--bg-surface-elevated); color: var(--text-primary); font-size: 13px; cursor: pointer;",
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

                                        // Card Cover Header
                                        div {
                                            class: "library-card-header",

                                            div {
                                                class: "library-card-cover",
                                                IconBook { size: 24 }
                                            }

                                            div {
                                                style: "flex: 1; min-width: 0;",
                                                h4 {
                                                    style: "margin: 0 0 4px 0; font-size: 16px; font-weight: 700; color: var(--text-primary); overflow: hidden; text-overflow: ellipsis; white-space: nowrap; line-height: 1.3;",
                                                    title: "{title}",
                                                    "{title}"
                                                }
                                                if let Some(src) = source {
                                                    div {
                                                        style: "font-size: 11px; color: var(--text-muted); overflow: hidden; text-overflow: ellipsis; white-space: nowrap; margin-top: 2px;",
                                                        title: "{src}",
                                                        "Source: {src}"
                                                    }
                                                }
                                            }
                                        }

                                        // Card Body & Metadata
                                        div {
                                            class: "library-card-body",

                                            // Stats badges
                                            div {
                                                style: "display: flex; flex-wrap: wrap; gap: 8px;",

                                                if pages > 0 {
                                                    span {
                                                        class: "library-badge",
                                                        IconFile { size: 12 }
                                                        span { "{pages} pages" }
                                                    }
                                                }

                                                if chapters > 0 {
                                                    span {
                                                        class: "library-badge",
                                                        IconList { size: 12 }
                                                        span { "{chapters} chapters" }
                                                    }
                                                }

                                                span {
                                                    class: "library-badge",
                                                    IconNotes { size: 12 }
                                                    span { "{word_count} words" }
                                                }
                                            }

                                            // Tags
                                            if !tags.is_empty() {
                                                div {
                                                    style: "display: flex; flex-wrap: wrap; gap: 6px; margin-top: 2px;",
                                                    for tag in tags.iter().take(4) {
                                                        span {
                                                            key: "{tag}",
                                                            class: "library-tag",
                                                            "#{tag}"
                                                        }
                                                    }
                                                    if tags.len() > 4 {
                                                        span {
                                                            style: "font-size: 11px; color: var(--text-muted); align-self: center; font-weight: 500;",
                                                            "+{tags.len() - 4}"
                                                        }
                                                    }
                                                }
                                            }

                                            // Progress bar
                                            div {
                                                style: "margin-top: auto; padding-top: 8px; display: flex; flex-direction: column; gap: 6px;",
                                                div {
                                                    style: "display: flex; justify-content: space-between; align-items: center; font-size: 11px;",
                                                    span { style: "color: var(--text-muted); font-weight: 500;", "Reading progress" }
                                                    span {
                                                        style: "font-weight: 600; font-family: var(--font-editor); color: if progress > 0 { \"var(--accent)\" } else { \"var(--text-muted)\" };",
                                                        "{progress}%"
                                                    }
                                                }
                                                div {
                                                    class: "library-progress-track",
                                                    div {
                                                        class: "library-progress-bar",
                                                        style: "width: {progress}%;",
                                                    }
                                                }
                                            }
                                        }

                                        // Card Footer Actions
                                        div {
                                            class: "library-card-footer",

                                            button {
                                                class: "btn-secondary",
                                                style: "display: inline-flex; align-items: center; gap: 6px; padding: 7px 14px; border-radius: 6px; border: 1px solid var(--border); background: var(--bg-surface); color: var(--text-primary); font-size: 12px; font-weight: 500; cursor: pointer;",
                                                onclick: move |_| {
                                                    let _ = state.write().open_book_in_editor(&book_path_edit);
                                                },
                                                IconEdit { size: 13 }
                                                span { "{actions::EDIT}" }
                                            }

                                            button {
                                                class: "btn-primary",
                                                style: "display: inline-flex; align-items: center; gap: 6px; padding: 7px 18px; border-radius: 6px; background: var(--accent); color: #ffffff; border: 1px solid transparent; font-size: 12px; font-weight: 600; cursor: pointer; box-shadow: 0 2px 6px rgba(91, 108, 255, 0.3);",
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
