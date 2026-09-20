use dioxus::prelude::*;
use std::path::PathBuf;

use crate::icons::{IconClose, IconFile, IconPlus, IconQuote};
use crate::state::AppState;

#[component]
pub fn PdfAnnotationModal(state: Signal<AppState>) -> Element {
    let app_state = state.read();
    if !app_state.show_pdf_annotation_modal {
        return rsx! {};
    }

    let mut selected_pdf = use_signal(|| None::<PathBuf>);
    let mut status_msg = use_signal(|| None::<String>);

    // Scan vault for existing .pdf files
    let mut vault_pdfs = Vec::new();
    if let Some(ref v) = app_state.vault_service {
        let vault_root = v.vault().root();
        find_pdf_files(vault_root, &mut vault_pdfs);
    }

    let current_pdf = selected_pdf.read().clone();
    let annot_report = current_pdf
        .as_ref()
        .and_then(|p| nodera_pdf::extract_annotations(p).ok());

    rsx! {
        div {
            class: "modal-overlay",
            onclick: move |_| {
                state.write().show_pdf_annotation_modal = false;
            },
            div {
                class: "modal-dialog",
                style: "max-width: 680px; width: 90%; max-height: 85vh; display: flex; flex-direction: column; overflow: hidden; padding: 20px;",
                onclick: move |e| e.stop_propagation(),

                // Header
                div {
                    style: "display: flex; align-items: center; justify-content: space-between; margin-bottom: 16px; border-bottom: 1px solid var(--border); padding-bottom: 12px;",
                    div {
                        style: "display: flex; align-items: center; gap: 10px;",
                        div {
                            style: "width: 32px; height: 32px; border-radius: 8px; background: rgba(91, 108, 255, 0.15); color: var(--accent); display: flex; align-items: center; justify-content: center;",
                            IconFile { size: 18 }
                        }
                        div {
                            h3 { style: "margin: 0; font-size: 16px; font-weight: 600; color: var(--text-primary);", "PDF Annotation Extractor" }
                            p { style: "margin: 0; font-size: 12px; color: var(--text-muted);", "Extract highlights, comments, and sticky notes into structured Markdown" }
                        }
                    }
                    button {
                        class: "btn-icon",
                        title: "Close (Esc)",
                        onclick: move |_| {
                            state.write().show_pdf_annotation_modal = false;
                        },
                        IconClose { size: 16 }
                    }
                }

                // File Selection Row
                div {
                    style: "display: flex; gap: 10px; align-items: center; margin-bottom: 14px;",
                    button {
                        class: "btn-secondary",
                        style: "font-size: 12px; display: flex; align-items: center; gap: 6px; white-space: nowrap;",
                        onclick: move |_| {
                            spawn(async move {
                                if let Some(file) = rfd::AsyncFileDialog::new()
                                    .add_filter("PDF Document", &["pdf"])
                                    .pick_file()
                                    .await
                                {
                                    selected_pdf.set(Some(file.path().to_path_buf()));
                                    status_msg.set(None);
                                }
                            });
                        },
                        IconFile { size: 14 }
                        "Browse PDF..."
                    }

                    if !vault_pdfs.is_empty() {
                        select {
                            class: "form-input",
                            style: "flex: 1; font-size: 12px; padding: 6px 10px; background: var(--bg-surface-elevated); border: 1px solid var(--border); border-radius: 6px;",
                            onchange: move |e| {
                                let val = e.value();
                                if !val.is_empty() {
                                    selected_pdf.set(Some(PathBuf::from(val)));
                                    status_msg.set(None);
                                }
                            },
                            option { value: "", "— Or select PDF in vault —" }
                            for pdf in vault_pdfs {
                                {
                                    let path_str = pdf.to_string_lossy().to_string();
                                    let name = pdf.file_name().map(|f| f.to_string_lossy().to_string()).unwrap_or_default();
                                    rsx! {
                                        option { value: "{path_str}", "{name}" }
                                    }
                                }
                            }
                        }
                    }
                }

                // Report Preview or Placeholder
                div {
                    style: "flex: 1; overflow-y: auto; display: flex; flex-direction: column; gap: 12px; min-height: 200px; max-height: 480px; padding-right: 4px;",

                    if let Some(ref report) = annot_report {
                        div {
                            style: "background: var(--bg-surface-elevated); border: 1px solid var(--border); border-radius: 8px; padding: 14px; display: flex; flex-direction: column; gap: 10px;",

                            div {
                                style: "display: flex; align-items: center; justify-content: space-between;",
                                div {
                                    h4 { style: "margin: 0; font-size: 14px; font-weight: 600; color: var(--text-primary);", "{report.pdf_filename}" }
                                    p { style: "margin: 2px 0 0 0; font-size: 12px; color: var(--text-muted);", "{report.total_pages} pages in document" }
                                }
                                div {
                                    style: "background: rgba(91, 108, 255, 0.12); color: var(--accent); font-size: 12px; font-weight: 600; padding: 4px 10px; border-radius: 20px;",
                                    "{report.annotations.len()} annotations found"
                                }
                            }

                            if report.annotations.is_empty() {
                                div {
                                    style: "padding: 20px; text-align: center; color: var(--text-muted); font-size: 12px;",
                                    "No annotations (highlights, underlines, or sticky notes) found in this PDF."
                                }
                            } else {
                                div {
                                    style: "display: flex; flex-direction: column; gap: 8px; max-height: 280px; overflow-y: auto; padding-right: 4px;",
                                    for annot in &report.annotations {
                                        {
                                            let page = annot.page;
                                            let kind_str = format!("{}", annot.kind);
                                            let text = annot.contents.clone().unwrap_or_else(|| "*(highlight without comment)*".to_string());
                                            let color_hex = annot.color_hex.clone();

                                            rsx! {
                                                div {
                                                    style: "background: var(--bg-surface); border: 1px solid var(--border); border-radius: 6px; padding: 8px 10px; font-size: 12px;",
                                                    div {
                                                        style: "display: flex; align-items: center; justify-content: space-between; margin-bottom: 4px;",
                                                        div {
                                                            style: "display: flex; align-items: center; gap: 6px;",
                                                            if let Some(hex) = color_hex {
                                                                span {
                                                                    style: "display: inline-block; width: 10px; height: 10px; border-radius: 50%; background: {hex};",
                                                                }
                                                            }
                                                            span { style: "font-weight: 600; color: var(--text-primary);", "{kind_str}" }
                                                        }
                                                        span { style: "color: var(--text-muted); font-size: 11px;", "Page {page}" }
                                                    }
                                                    p { style: "margin: 0; color: var(--text-secondary); line-height: 1.4;", "{text}" }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    } else if current_pdf.is_some() {
                        div {
                            style: "text-align: center; padding: 40px; color: var(--text-danger, #ff5555); font-size: 13px;",
                            "Failed to parse annotations from selected PDF."
                        }
                    } else {
                        div {
                            style: "text-align: center; padding: 50px 20px; color: var(--text-muted);",
                            div {
                                style: "margin-bottom: 12px; display: flex; justify-content: center; opacity: 0.6;",
                                IconQuote { size: 36 }
                            }
                            p { style: "font-weight: 500; font-size: 14px; margin-bottom: 6px; color: var(--text-primary);", "No PDF Document Selected" }
                            p { style: "font-size: 12px; max-width: 400px; margin: 0 auto; line-height: 1.5;",
                                "Select a PDF file from your vault or click 'Browse PDF...' to inspect and extract highlights and annotations directly into a research note."
                            }
                        }
                    }
                }

                // Footer Actions
                div {
                    style: "display: flex; align-items: center; justify-content: space-between; margin-top: 16px; border-top: 1px solid var(--border); padding-top: 14px;",
                    div {
                        if let Some(ref msg) = *status_msg.read() {
                            span { style: "font-size: 12px; color: var(--accent);", "{msg}" }
                        }
                    }

                    div {
                        style: "display: flex; gap: 8px;",
                        button {
                            class: "btn-secondary",
                            onclick: move |_| {
                                state.write().show_pdf_annotation_modal = false;
                            },
                            "Cancel"
                        }
                        if let Some(ref pdf_path) = current_pdf {
                            {
                                let path_to_extract = pdf_path.clone();
                                let has_annots = annot_report.as_ref().map(|r| !r.annotations.is_empty()).unwrap_or(false);

                                rsx! {
                                    button {
                                        class: "btn-primary",
                                        disabled: !has_annots,
                                        style: "display: flex; align-items: center; gap: 6px;",
                                        onclick: move |_| {
                                            let res = state.write().extract_pdf_annotations_to_note(&path_to_extract);
                                            match res {
                                                Ok(_note_path) => {
                                                    state.write().show_pdf_annotation_modal = false;
                                                }
                                                Err(e) => {
                                                    status_msg.set(Some(format!("Error: {}", e)));
                                                }
                                            }
                                        },
                                        IconPlus { size: 14 }
                                        "Extract to Markdown Note"
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

fn find_pdf_files(dir: &std::path::Path, acc: &mut Vec<PathBuf>) {
    let entries = match std::fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return,
    };

    for entry in entries.flatten() {
        let path = entry.path();
        let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
        if name.starts_with('.') || name == "target" || name == "node_modules" {
            continue;
        }

        if path.is_dir() {
            find_pdf_files(&path, acc);
        } else if path.extension().and_then(|ext| ext.to_str()) == Some("pdf") {
            acc.push(path);
        }
    }
}
