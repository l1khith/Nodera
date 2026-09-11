use lopdf::content::{Content, Operation};
use lopdf::{dictionary, Document, Object, Stream};
use nodera_desktop::state::AppState;
use nodera_pdf::{CancellationToken, ConversionOptions, NativePdfConverter, PdfImportService};
use std::io::Write;
use std::path::PathBuf;
use std::sync::Arc;
use tempfile::{tempdir, NamedTempFile};

/// Helper to generate a test PDF file.
fn create_test_pdf(title: &str, pages: &[&[&str]]) -> NamedTempFile {
    let mut doc = Document::with_version("1.5");
    let pages_id = doc.new_object_id();

    let font_id = doc.add_object(dictionary! {
        "Type" => "Font",
        "Subtype" => "Type1",
        "BaseFont" => "Helvetica",
    });

    let resources_id = doc.add_object(dictionary! {
        "Font" => dictionary! {
            "F1" => font_id,
        },
    });

    let mut page_ids = Vec::new();

    for lines in pages {
        let mut operations = vec![
            Operation::new("BT", vec![]),
            Operation::new("Tf", vec!["F1".into(), 12.into()]),
            Operation::new("Td", vec![50.into(), 750.into()]),
            Operation::new("TL", vec![14.into()]),
        ];

        for line in *lines {
            operations.push(Operation::new("Tj", vec![Object::string_literal(*line)]));
            operations.push(Operation::new("T*", vec![]));
        }

        operations.push(Operation::new("ET", vec![]));

        let content = Content { operations };
        let content_id = doc.add_object(Stream::new(
            dictionary! {},
            content.encode().expect("encode"),
        ));

        let page_id = doc.add_object(dictionary! {
            "Type" => "Page",
            "Parent" => pages_id,
            "Contents" => content_id,
        });

        page_ids.push(page_id.into());
    }

    let pages_dict = dictionary! {
        "Type" => "Pages",
        "Kids" => page_ids,
        "Count" => pages.len() as i64,
        "Resources" => resources_id,
        "MediaBox" => vec![0.into(), 0.into(), 595.into(), 842.into()],
    };
    doc.set_object(pages_id, pages_dict);

    let catalog_id = doc.add_object(dictionary! {
        "Type" => "Catalog",
        "Pages" => pages_id,
    });
    doc.trailer.set("Root", catalog_id);

    let info_id = doc.add_object(dictionary! {
        "Title" => Object::string_literal(title),
        "Author" => Object::string_literal("Nodera Test Author"),
    });
    doc.trailer.set("Info", info_id);

    let mut temp = NamedTempFile::new().expect("create temp file");
    doc.save_to(&mut temp).expect("save pdf");
    temp.flush().expect("flush temp");
    temp
}

#[test]
fn test_end_to_end_pdf_import_workflow() {
    let tmp = tempdir().expect("create temp dir");
    let vault_path = tmp.path().join("PdfTestVault");

    let mut state = AppState::default();
    state
        .create_vault(&vault_path, Some("PDF Test Vault".to_string()))
        .expect("create vault");

    // 1. Generate test PDF with 3 pages to verify repeated header stripping
    let pdf_file = create_test_pdf(
        "Quantum Mechanics Primer",
        &[
            &[
                "RUNNING HEADER",
                "Chapter 1: The Wavefunction",
                "The state of a quantum particle is described by a complex wavefunction.",
                "1",
            ],
            &[
                "RUNNING HEADER",
                "1.1 Schrodinger Equation",
                "The time-dependent Schrodinger equation governs how quantum states evolve.",
                "2",
            ],
            &[
                "RUNNING HEADER",
                "Chapter 2: Quantum Tunneling",
                "Particles can penetrate potential energy barriers through tunneling.",
                "3",
            ],
        ],
    );

    // 2. Configure import state
    state.open_pdf_import_modal();
    assert!(state.show_pdf_import_modal);
    state.set_pdf_selected_path(pdf_file.path().to_path_buf());

    // 3. Execute conversion via PdfImportService
    let converter = Arc::new(NativePdfConverter::new());
    let service = PdfImportService::new(converter);
    let options = ConversionOptions {
        detect_headings: true,
        remove_repeated_headers: true,
        remove_page_numbers: true,
        add_page_markers: true,
        ..Default::default()
    };
    let cancellation = CancellationToken::new();

    let import_result = service
        .import_to_vault(&vault_path, pdf_file.path(), &options, &cancellation, None)
        .expect("import to vault");

    // 4. Complete import in AppState (triggers re-indexing and entry refresh)
    state.complete_pdf_import(import_result.clone());

    assert!(state.pdf_last_result.is_some());
    let res = state.pdf_last_result.as_ref().unwrap();
    assert_eq!(
        res.relative_vault_path,
        PathBuf::from("Books").join("Quantum Mechanics Primer.md")
    );
    assert_eq!(res.conversion.total_pages, 3);
    assert_eq!(res.conversion.chapters_detected, 2);

    // 5. Verify file on disk
    let written_file = vault_path.join("Books").join("Quantum Mechanics Primer.md");
    assert!(written_file.exists());
    let content = std::fs::read_to_string(&written_file).expect("read imported note");
    assert!(content.contains("title: \"Quantum Mechanics Primer\""));
    assert!(content.contains("## Chapter 1: The Wavefunction"));
    assert!(content.contains("### 1.1 Schrodinger Equation"));
    assert!(content.contains("## Chapter 2: Quantum Tunneling"));
    assert!(content.contains("<!-- nodera:page=1 -->"));
    assert!(content.contains("<!-- nodera:page=2 -->"));
    assert!(content.contains("<!-- nodera:page=3 -->"));
    // Verify running header and page numbers removed
    assert!(!content.contains("RUNNING HEADER"));
    assert!(!content.contains("\n1\n"));
    assert!(!content.contains("\n2\n"));
    assert!(!content.contains("\n3\n"));

    // 6. Verify full-text search index contains imported note
    state.execute_search("wavefunction");
    assert!(!state.search_results.is_empty());
    assert_eq!(state.search_results[0].title, "Quantum Mechanics Primer");

    // 7. Verify opening generated note in editor
    state.open_imported_note();
    assert!(!state.show_pdf_import_modal);
    assert!(state.active_note.is_some());
    assert_eq!(
        state.active_note.as_ref().unwrap().title,
        "Quantum Mechanics Primer"
    );
    assert_eq!(
        state.active_note.as_ref().unwrap().relative_path,
        PathBuf::from("Books").join("Quantum Mechanics Primer.md")
    );
    assert!(state.editor_content.contains("Schrodinger equation"));

    // 8. Verify collision handling on second import
    let import_result_2 = service
        .import_to_vault(&vault_path, pdf_file.path(), &options, &cancellation, None)
        .expect("second import");

    assert_eq!(
        import_result_2.relative_vault_path,
        PathBuf::from("Books").join("Quantum Mechanics Primer (1).md")
    );
    assert!(vault_path
        .join("Books")
        .join("Quantum Mechanics Primer (1).md")
        .exists());
}
