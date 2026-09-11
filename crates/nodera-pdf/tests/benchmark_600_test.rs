use lopdf::content::{Content, Operation};
use lopdf::{dictionary, Document, Object, Stream};
use nodera_pdf::{
    CancellationToken, ConversionOptions, ConversionProgress, NativePdfConverter, PdfConverter,
};
use std::io::Write;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Instant;
use tempfile::NamedTempFile;

/// Generates a synthetic 600-page document with realistic chapter headings,
/// paragraphs, running headers, and page numbers.
fn generate_600_page_pdf() -> (NamedTempFile, u64) {
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

    let mut page_ids = Vec::with_capacity(600);

    for page_idx in 1..=600 {
        let mut operations = vec![
            Operation::new("BT", vec![]),
            Operation::new("Tf", vec!["F1".into(), 12.into()]),
            Operation::new("Td", vec![50.into(), 750.into()]),
            Operation::new("TL", vec![14.into()]),
        ];

        // Running header
        operations.push(Operation::new(
            "Tj",
            vec![Object::string_literal("PRINCIPLES OF DISTRIBUTED SYSTEMS")],
        ));
        operations.push(Operation::new("T*", vec![]));

        // Chapter every 30 pages
        if page_idx % 30 == 1 {
            let chapter_num = (page_idx / 30) + 1;
            let chapter_title = format!("Chapter {}: Consensus Protocols", chapter_num);
            operations.push(Operation::new(
                "Tj",
                vec![Object::string_literal(chapter_title)],
            ));
            operations.push(Operation::new("T*", vec![]));
        }

        // Section heading
        if page_idx % 10 == 1 {
            let sec_title = format!("1.{} State Machine Replication", page_idx / 10);
            operations.push(Operation::new(
                "Tj",
                vec![Object::string_literal(sec_title)],
            ));
            operations.push(Operation::new("T*", vec![]));
        }

        // Paragraph text
        operations.push(Operation::new(
            "Tj",
            vec![Object::string_literal(
                "Distributed consensus ensures that multiple nodes agree on state changes",
            )],
        ));
        operations.push(Operation::new("T*", vec![]));
        operations.push(Operation::new(
            "Tj",
            vec![Object::string_literal(
                "even in the presence of unreliable network connections and node crashes.",
            )],
        ));
        operations.push(Operation::new("T*", vec![]));
        operations.push(Operation::new("Tj", vec![Object::string_literal("Raft and Paxos decompose the consensus problem into leader election, log replication,")]));
        operations.push(Operation::new("T*", vec![]));
        operations.push(Operation::new(
            "Tj",
            vec![Object::string_literal(
                "and safety invariants to maintain system linearizability.",
            )],
        ));
        operations.push(Operation::new("T*", vec![]));

        // Page number footer
        operations.push(Operation::new(
            "Tj",
            vec![Object::string_literal(format!("{}", page_idx))],
        ));
        operations.push(Operation::new("T*", vec![]));

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
        "Count" => 600i64,
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
        "Title" => Object::string_literal("Principles of Distributed Systems"),
        "Author" => Object::string_literal("Leslie Lamport & Friends"),
    });
    doc.trailer.set("Info", info_id);

    let mut temp = NamedTempFile::new().expect("create temp file");
    doc.save_to(&mut temp).expect("save pdf to temp");
    temp.flush().expect("flush temp");

    let size = temp.as_file().metadata().map(|m| m.len()).unwrap_or(0);
    (temp, size)
}

#[test]
#[ignore = "Expensive 600-page stress/performance benchmark. Run with: cargo test -p nodera-pdf --test benchmark_600_test -- --ignored --nocapture"]
fn benchmark_600_page_conversion() {
    println!("\n=== Starting 600-Page PDF Stress & Performance Benchmark ===");

    let gen_start = Instant::now();
    let (pdf_file, file_size_bytes) = generate_600_page_pdf();
    let gen_duration = gen_start.elapsed();
    println!("Generated 600-page synthetic PDF:");
    println!(
        "  - File size: {} bytes ({:.2} MB)",
        file_size_bytes,
        file_size_bytes as f64 / (1024.0 * 1024.0)
    );
    println!("  - Generation duration: {:?}", gen_duration);

    let converter = NativePdfConverter::new();
    let validate_start = Instant::now();
    let metadata = converter
        .validate(pdf_file.path())
        .expect("validate 600-page pdf");
    let validate_duration = validate_start.elapsed();
    println!("Validation completed in {:?}:", validate_duration);
    println!("  - Page count: {}", metadata.page_count);
    println!("  - Title: {:?}", metadata.title);
    assert_eq!(metadata.page_count, 600);

    let options = ConversionOptions {
        detect_headings: true,
        remove_repeated_headers: true,
        remove_page_numbers: true,
        add_page_markers: true,
        heading_prefix_depth: 2,
    };
    let cancellation = CancellationToken::new();

    // Monotonicity verification
    let last_reported_page = Arc::new(AtomicUsize::new(0));
    let last_page_clone = last_reported_page.clone();
    let event_count = Arc::new(AtomicUsize::new(0));
    let event_count_clone = event_count.clone();

    let progress_sink: Arc<dyn Fn(ConversionProgress) + Send + Sync> = Arc::new(move |p| {
        event_count_clone.fetch_add(1, Ordering::SeqCst);
        let prev = last_page_clone.load(Ordering::SeqCst);
        assert!(
            p.page_current >= prev,
            "Progress was non-monotonic! prev: {}, current: {}",
            prev,
            p.page_current
        );
        assert!(
            p.page_current <= 600,
            "Page exceeded total! current: {}",
            p.page_current
        );
        last_page_clone.store(p.page_current, Ordering::SeqCst);
    });

    let conv_start = Instant::now();
    let result = converter
        .convert(
            pdf_file.path(),
            &options,
            &cancellation,
            Some(&progress_sink),
        )
        .expect("convert 600-page PDF");
    let conv_duration = conv_start.elapsed();

    println!("Conversion completed successfully in {:?}", conv_duration);
    println!("Performance measurements:");
    println!("  - Total pages processed: {}", result.total_pages);
    println!("  - Chapters detected: {}", result.chapters_detected);
    println!("  - Characters extracted: {}", result.characters_extracted);
    println!(
        "  - Output Markdown size: {} bytes ({:.2} MB)",
        result.markdown_content.len(),
        result.markdown_content.len() as f64 / (1024.0 * 1024.0)
    );
    println!(
        "  - Total progress events emitted: {}",
        event_count.load(Ordering::SeqCst)
    );
    println!(
        "  - Processing throughput: {:.1} pages/sec",
        600.0 / conv_duration.as_secs_f64()
    );

    assert_eq!(result.total_pages, 600);
    assert!(result.chapters_detected >= 20);
    assert!(result
        .markdown_content
        .contains("title: \"Principles of Distributed Systems\""));
    assert!(result.markdown_content.contains("<!-- nodera:page=600 -->"));
    // Verify running header stripped
    assert!(!result
        .markdown_content
        .contains("PRINCIPLES OF DISTRIBUTED SYSTEMS"));

    println!("=== 600-Page Benchmark PASSED ===\n");
}
