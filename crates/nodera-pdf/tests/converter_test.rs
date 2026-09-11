use lopdf::content::{Content, Operation};
use lopdf::{dictionary, Document, Object, Stream};
use nodera_core::error::PdfError;
use nodera_pdf::{CancellationToken, ConversionOptions, NativePdfConverter, PdfConverter};
use std::io::Write;
use std::sync::Arc;
use tempfile::NamedTempFile;

/// Helper to generate a valid PDF with multiple pages of text using standard Helvetica font.
pub fn generate_test_pdf(pages_text: &[&[&str]], title: Option<&str>) -> NamedTempFile {
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

    for lines in pages_text {
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
            content.encode().expect("encode content"),
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
        "Count" => pages_text.len() as i64,
        "Resources" => resources_id,
        "MediaBox" => vec![0.into(), 0.into(), 595.into(), 842.into()],
    };
    doc.set_object(pages_id, pages_dict);

    let catalog_id = doc.add_object(dictionary! {
        "Type" => "Catalog",
        "Pages" => pages_id,
    });
    doc.trailer.set("Root", catalog_id);

    if let Some(t) = title {
        let info_id = doc.add_object(dictionary! {
            "Title" => Object::string_literal(t),
            "Author" => Object::string_literal("Nodera Test Suite"),
        });
        doc.trailer.set("Info", info_id);
    }

    let mut temp = NamedTempFile::new().expect("create temp file");
    doc.save_to(&mut temp).expect("save pdf to temp");
    temp.flush().expect("flush temp");
    temp
}

#[test]
fn test_converter_single_page() {
    let pdf_file = generate_test_pdf(
        &[&[
            "Chapter 1: The Beginning",
            "This is the first line of the book.",
        ]],
        Some("Single Page Guide"),
    );

    let converter = NativePdfConverter::new();
    let metadata = converter.validate(pdf_file.path()).expect("validate");
    assert_eq!(metadata.page_count, 1);
    assert_eq!(metadata.title, Some("Single Page Guide".to_string()));
    assert!(!metadata.is_encrypted);

    let options = ConversionOptions::default();
    let cancellation = CancellationToken::new();
    let result = converter
        .convert(pdf_file.path(), &options, &cancellation, None)
        .expect("convert");

    assert_eq!(result.total_pages, 1);
    assert_eq!(result.chapters_detected, 1);
    assert!(result
        .markdown_content
        .contains("## Chapter 1: The Beginning"));
    assert!(result
        .markdown_content
        .contains("This is the first line of the book."));
    assert!(result
        .markdown_content
        .contains("title: \"Single Page Guide\""));
}

#[test]
fn test_converter_multiple_pages_and_ordering() {
    let pdf_file = generate_test_pdf(
        &[
            &["Page 1 Header", "Content of first page."],
            &["Page 2 Header", "Content of second page."],
            &["Page 3 Header", "Content of third page."],
        ],
        Some("Multi-Page Test"),
    );

    let converter = NativePdfConverter::new();
    let options = ConversionOptions::default();
    let cancellation = CancellationToken::new();

    let result = converter
        .convert(pdf_file.path(), &options, &cancellation, None)
        .expect("convert");

    assert_eq!(result.total_pages, 3);
    let md = &result.markdown_content;
    let pos1 = md.find("Content of first page.").expect("page 1 found");
    let pos2 = md.find("Content of second page.").expect("page 2 found");
    let pos3 = md.find("Content of third page.").expect("page 3 found");

    assert!(pos1 < pos2);
    assert!(pos2 < pos3);
}

#[test]
fn test_converter_repeated_headers_and_page_numbers() {
    let pdf_file = generate_test_pdf(
        &[
            &["RUNNING BOOK TITLE", "First chapter text begins here.", "1"],
            &[
                "RUNNING BOOK TITLE",
                "Second chapter continuation text.",
                "2",
            ],
            &[
                "RUNNING BOOK TITLE",
                "Third page text concluding story.",
                "3",
            ],
        ],
        Some("Book With Headers"),
    );

    let converter = NativePdfConverter::new();
    let options = ConversionOptions {
        remove_repeated_headers: true,
        remove_page_numbers: true,
        ..Default::default()
    };
    let cancellation = CancellationToken::new();

    let result = converter
        .convert(pdf_file.path(), &options, &cancellation, None)
        .expect("convert");

    assert_eq!(result.total_pages, 3);
    // Running header "RUNNING BOOK TITLE" should be stripped
    assert!(!result.markdown_content.contains("RUNNING BOOK TITLE"));
    // Standalone page numbers should be stripped
    assert!(!result.markdown_content.contains("\n1\n"));
    assert!(!result.markdown_content.contains("\n2\n"));
    assert!(!result.markdown_content.contains("\n3\n"));
    // Main content preserved
    assert!(result
        .markdown_content
        .contains("First chapter text begins here."));
    assert!(result
        .markdown_content
        .contains("Second chapter continuation text."));
}

#[test]
fn test_converter_headings_and_chapters() {
    let pdf_file = generate_test_pdf(
        &[
            &[
                "Chapter 1: Quantum Theory",
                "The origins of quantum physics.",
            ],
            &[
                "1.1 Wave Particle Duality",
                "Photons exhibit both wave and particle traits.",
            ],
            &[
                "Chapter 2: Relativity",
                "Einstein introduced spacetime curvature.",
            ],
        ],
        Some("Physics Compendium"),
    );

    let converter = NativePdfConverter::new();
    let options = ConversionOptions::default();
    let cancellation = CancellationToken::new();

    let result = converter
        .convert(pdf_file.path(), &options, &cancellation, None)
        .expect("convert");

    assert_eq!(result.total_pages, 3);
    assert_eq!(result.chapters_detected, 2);
    assert!(result
        .markdown_content
        .contains("## Chapter 1: Quantum Theory"));
    assert!(result
        .markdown_content
        .contains("### 1.1 Wave Particle Duality"));
    assert!(result.markdown_content.contains("## Chapter 2: Relativity"));
}

#[test]
fn test_converter_malformed_pdf() {
    let mut temp = NamedTempFile::new().unwrap();
    temp.write_all(b"NOT A VALID PDF FILE AT ALL").unwrap();
    temp.flush().unwrap();

    let converter = NativePdfConverter::new();
    let result = converter.validate(temp.path());

    assert!(result.is_err());
    match result.unwrap_err() {
        PdfError::InvalidPdf { reason, .. } => {
            assert!(reason.contains("magic bytes") || reason.contains("header"));
        }
        other => panic!("Expected InvalidPdf error, got {:?}", other),
    }
}

#[test]
fn test_converter_deterministic_cancellation() {
    let pdf_file = generate_test_pdf(
        &[
            &["Page 1 Content"],
            &["Page 2 Content"],
            &["Page 3 Content"],
            &["Page 4 Content"],
            &["Page 5 Content"],
        ],
        Some("Cancellation Test"),
    );

    let converter = NativePdfConverter::new();
    let options = ConversionOptions::default();
    let cancellation = CancellationToken::new();

    // Cancellation hook: cancel deterministically when page 2 progress event is received
    let cancel_token = cancellation.clone();
    let progress_sink: Arc<dyn Fn(nodera_pdf::ConversionProgress) + Send + Sync> =
        Arc::new(move |p| {
            if p.page_current == 2 {
                cancel_token.cancel();
            }
        });

    let result = converter.convert(
        pdf_file.path(),
        &options,
        &cancellation,
        Some(&progress_sink),
    );

    assert!(result.is_err());
    match result.unwrap_err() {
        PdfError::Cancelled => {}
        other => panic!("Expected PdfError::Cancelled, got {:?}", other),
    }
}
