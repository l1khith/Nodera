use dioxus::prelude::*;
use nodera_pdf::{CancellationToken, ConversionProgress, NativePdfConverter, PdfImportService};
use std::path::PathBuf;
use std::sync::Arc;

use crate::state::AppState;

#[component]
pub fn PdfImportModal(mut state: Signal<AppState>) -> Element {
    let app_state = state.read();
    if !app_state.show_pdf_import_modal {
        return rsx! {};
    }

    let has_vault = app_state.vault_service.is_some();
    let selected_path = app_state.pdf_selected_path.clone();
    let is_converting = app_state.pdf_cancellation.is_some();
    let last_result = app_state.pdf_last_result.clone();
    let progress = app_state.pdf_progress.clone();
    let error = app_state.pdf_error.clone();
    let options = app_state.pdf_import_options.clone();

    rsx! {
        div {
            class: "modal-overlay",
            onclick: move |_| {
                if !is_converting {
                    let mut s = state.write();
                    s.close_pdf_import_modal();
                }
            },
            div {
                class: "modal-dialog",
                style: "width: 520px;",
                onclick: move |evt| {
                    evt.stop_propagation();
                },

                // Header
                div {
                    style: "display: flex; align-items: center; justify-content: space-between; margin-bottom: 16px; border-bottom: 1px solid var(--border); padding-bottom: 12px;",
                    span { style: "font-weight: 600; font-size: 15px; color: var(--text-primary);",
                        if last_result.is_some() {
                            "✓ Conversion Complete"
                        } else if is_converting {
                            "Converting PDF..."
                        } else {
                            "Import PDF as Markdown"
                        }
                    }
                    if !is_converting {
                        button {
                            class: "btn-icon",
                            style: "font-size: 12px;",
                            onclick: move |_| {
                                let mut s = state.write();
                                s.close_pdf_import_modal();
                            },
                            "✕"
                        }
                    }
                }

                // 1. Completion View
                if let Some(res) = last_result {
                    div {
                        style: "display: flex; flex-direction: column; gap: 14px;",
                        div {
                            style: "padding: 14px; background-color: var(--bg-surface-elevated); border-radius: 6px; border: 1px solid var(--border); display: flex; flex-direction: column; gap: 8px;",
                            div { style: "font-size: 13px; font-weight: 600; color: var(--text-primary);", "Note Created" }
                            div { style: "font-size: 12px; color: var(--accent); word-break: break-all;", "{res.relative_vault_path.display()}" }
                            div {
                                style: "display: grid; grid-template-columns: repeat(3, 1fr); gap: 8px; margin-top: 6px; padding-top: 8px; border-top: 1px solid var(--border-subtle); font-size: 12px;",
                                div {
                                    span { style: "color: var(--text-muted);", "Pages: " }
                                    strong { "{res.conversion.total_pages}" }
                                }
                                div {
                                    span { style: "color: var(--text-muted);", "Chapters: " }
                                    strong { "{res.conversion.chapters_detected}" }
                                }
                                div {
                                    span { style: "color: var(--text-muted);", "Characters: " }
                                    strong { "{res.conversion.characters_extracted}" }
                                }
                            }
                        }

                        div {
                            style: "display: flex; justify-content: flex-end; gap: 8px; margin-top: 8px;",
                            button {
                                class: "btn-secondary",
                                onclick: move |_| {
                                    let mut s = state.write();
                                    s.close_pdf_import_modal();
                                },
                                "Close"
                            }
                            button {
                                class: "btn-primary",
                                onclick: move |_| {
                                    let mut s = state.write();
                                    s.open_imported_note();
                                },
                                "📝 Open Markdown Note"
                            }
                        }
                    }
                }
                // 2. Active Converting View
                else if is_converting {
                    {
                        let (curr, total, stage_text, msg) = if let Some(ref p) = progress {
                            (p.page_current, p.page_total, p.stage.to_string(), p.message.clone())
                        } else {
                            (0, 0, "Initializing...".to_string(), "Preparing extraction...".to_string())
                        };

                        let pct = if total > 0 {
                            ((curr as f64 / total as f64) * 100.0).clamp(0.0, 100.0) as usize
                        } else {
                            0
                        };

                        rsx! {
                            div {
                                style: "display: flex; flex-direction: column; gap: 12px; padding: 8px 0;",
                                if let Some(ref path) = selected_path {
                                    div { style: "font-weight: 500; font-size: 13px;", "{path.file_name().and_then(|s| s.to_str()).unwrap_or(\"document.pdf\")}" }
                                }

                                // Progress Bar
                                div {
                                    style: "background-color: var(--border); border-radius: 4px; height: 12px; overflow: hidden; position: relative;",
                                    div {
                                        style: "background-color: var(--accent); height: 100%; width: {pct}%; transition: width 0.15s ease;"
                                    }
                                }

                                div {
                                    style: "display: flex; justify-content: space-between; font-size: 12px; color: var(--text-muted);",
                                    span { "Pages {curr} / {total} ({pct}%)" }
                                    span { "{stage_text}" }
                                }

                                p { style: "font-size: 11px; color: var(--text-secondary); margin: 0; font-family: monospace;", "{msg}" }

                                div {
                                    style: "display: flex; justify-content: flex-end; margin-top: 12px;",
                                    button {
                                        class: "btn-secondary",
                                        style: "color: #ef4444; border-color: rgba(239, 68, 68, 0.4);",
                                        onclick: move |_| {
                                            let mut s = state.write();
                                            s.cancel_pdf_import();
                                        },
                                        "Cancel Conversion"
                                    }
                                }
                            }
                        }
                    }
                }
                // 3. Configuration & Selection View
                else {
                    div {
                        style: "display: flex; flex-direction: column; gap: 16px;",

                        // File Selector Box
                        if let Some(ref path) = selected_path {
                            div {
                                style: "padding: 12px; background-color: var(--bg-surface-elevated); border: 1px solid var(--border); border-radius: 6px; display: flex; align-items: center; justify-content: space-between;",
                                div {
                                    style: "display: flex; flex-direction: column; gap: 2px;",
                                    span { style: "font-weight: 500; font-size: 13px;", "📄 {path.file_name().and_then(|s| s.to_str()).unwrap_or(\"document.pdf\")}" }
                                    span { style: "font-size: 11px; color: var(--text-muted);", "Destination: Books/{path.file_stem().and_then(|s| s.to_str()).unwrap_or(\"document\")}.md" }
                                }
                                button {
                                    class: "btn-secondary",
                                    style: "font-size: 11px; padding: 4px 8px;",
                                    onclick: move |_| {
                                        spawn(async move {
                                            if let Some(file) = rfd::AsyncFileDialog::new().add_filter("PDF files", &["pdf"]).pick_file().await {
                                                let mut s = state.write();
                                                s.set_pdf_selected_path(PathBuf::from(file.path()));
                                            }
                                        });
                                    },
                                    "Change"
                                }
                            }
                        } else {
                            div {
                                style: "border: 2px dashed var(--border); border-radius: 8px; padding: 24px; text-align: center; display: flex; flex-direction: column; align-items: center; gap: 10px;",
                                span { style: "font-size: 28px;", "📄" }
                                span { style: "font-size: 13px; color: var(--text-secondary);", "Select a text-based PDF to import as Markdown" }
                                button {
                                    class: "btn-secondary",
                                    onclick: move |_| {
                                        spawn(async move {
                                            if let Some(file) = rfd::AsyncFileDialog::new().add_filter("PDF files", &["pdf"]).pick_file().await {
                                                let mut s = state.write();
                                                s.set_pdf_selected_path(PathBuf::from(file.path()));
                                            }
                                        });
                                    },
                                    "Choose PDF File..."
                                }
                            }
                        }

                        // Options Checkboxes
                        div {
                            style: "display: flex; flex-direction: column; gap: 8px; padding: 10px 14px; background-color: var(--bg-surface-elevated); border-radius: 6px; border: 1px solid var(--border-subtle);",
                            span { style: "font-size: 12px; font-weight: 600; color: var(--text-secondary); margin-bottom: 2px;", "Conversion Options" }
                            label {
                                style: "display: flex; align-items: center; gap: 8px; font-size: 12px; cursor: pointer;",
                                input {
                                    r#type: "checkbox",
                                    checked: options.detect_headings,
                                    onchange: move |evt| {
                                        let mut s = state.write();
                                        s.pdf_import_options.detect_headings = evt.checked();
                                    }
                                }
                                "Detect chapter & section headings"
                            }
                            label {
                                style: "display: flex; align-items: center; gap: 8px; font-size: 12px; cursor: pointer;",
                                input {
                                    r#type: "checkbox",
                                    checked: options.remove_repeated_headers,
                                    onchange: move |evt| {
                                        let mut s = state.write();
                                        s.pdf_import_options.remove_repeated_headers = evt.checked();
                                    }
                                }
                                "Remove repeated running headers & footers"
                            }
                            label {
                                style: "display: flex; align-items: center; gap: 8px; font-size: 12px; cursor: pointer;",
                                input {
                                    r#type: "checkbox",
                                    checked: options.remove_page_numbers,
                                    onchange: move |evt| {
                                        let mut s = state.write();
                                        s.pdf_import_options.remove_page_numbers = evt.checked();
                                    }
                                }
                                "Remove standalone page numbers"
                            }
                            label {
                                style: "display: flex; align-items: center; gap: 8px; font-size: 12px; cursor: pointer;",
                                input {
                                    r#type: "checkbox",
                                    checked: options.add_page_markers,
                                    onchange: move |evt| {
                                        let mut s = state.write();
                                        s.pdf_import_options.add_page_markers = evt.checked();
                                    }
                                }
                                "Insert page comment markers (<!-- nodera:page=N -->)"
                            }
                        }

                        // Error message banner
                        if let Some(err_msg) = error {
                            div {
                                style: "padding: 8px 12px; background-color: rgba(239, 68, 68, 0.15); border: 1px solid rgba(239, 68, 68, 0.4); border-radius: 6px; color: #ef4444; font-size: 12px;",
                                "Error: {err_msg}"
                            }
                        }

                        // Action Buttons
                        div {
                            style: "display: flex; justify-content: flex-end; gap: 8px; margin-top: 4px;",
                            button {
                                class: "btn-secondary",
                                onclick: move |_| {
                                    let mut s = state.write();
                                    s.close_pdf_import_modal();
                                },
                                "Cancel"
                            }
                            button {
                                class: "btn-primary",
                                disabled: selected_path.is_none() || !has_vault,
                                onclick: move |_| {
                                    let (vault_root, input_pdf, conv_opts) = {
                                        let current_state = state.read();
                                        let vr = match current_state.vault_path.clone() {
                                            Some(p) => p,
                                            None => return,
                                        };
                                        let ip = match current_state.pdf_selected_path.clone() {
                                            Some(p) => p,
                                            None => return,
                                        };
                                        let co = current_state.pdf_import_options.clone();
                                        (vr, ip, co)
                                    };

                                    let cancel_token = CancellationToken::new();

                                    {
                                        let mut s = state.write();
                                        s.pdf_cancellation = Some(cancel_token.clone());
                                        s.pdf_error = None;
                                        s.pdf_progress = Some(ConversionProgress {
                                            page_current: 0,
                                            page_total: 0,
                                            stage: nodera_pdf::ConversionStage::Validating,
                                            bytes_processed: None,
                                            message: "Validating PDF...".to_string(),
                                        });
                                    }

                                    spawn(async move {
                                        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<ConversionProgress>();
                                        let token_for_worker = cancel_token.clone();
                                        let vr = vault_root.clone();
                                        let ip = input_pdf.clone();
                                        let co = conv_opts.clone();

                                        let worker = tokio::task::spawn_blocking(move || {
                                            let converter = Arc::new(NativePdfConverter::new());
                                            let service = PdfImportService::new(converter);
                                            let sink: Arc<dyn Fn(ConversionProgress) + Send + Sync> = Arc::new(move |p| {
                                                let _ = tx.send(p);
                                            });
                                            service.import_to_vault(&vr, &ip, &co, &token_for_worker, Some(&sink))
                                        });

                                        // Listen for progress events while worker is running
                                        let progress_listener = async {
                                            while let Some(prog) = rx.recv().await {
                                                state.write().pdf_progress = Some(prog);
                                            }
                                        };

                                        let (worker_res, _) = tokio::join!(worker, progress_listener);

                                        match worker_res {
                                            Ok(Ok(import_res)) => {
                                                state.write().complete_pdf_import(import_res);
                                            }
                                            Ok(Err(err)) => {
                                                state.write().set_pdf_error(err.to_string());
                                            }
                                            Err(join_err) => {
                                                state.write().set_pdf_error(format!("Worker failed: {}", join_err));
                                            }
                                        }
                                    });
                                },
                                "Convert to Markdown"
                            }
                        }
                    }
                }
            }
        }
    }
}
