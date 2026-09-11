# PDF → Markdown Design

## 1. Scope

PDF → Markdown is a feature of Nodera, not the product's entire identity.

Initial target:
- text-based PDFs
- long documents
- approximately 600 pages
- no image reconstruction requirement

Scanned PDFs and highly complex layouts are not V1 quality targets.

## 2. Pipeline

```text
PDF
 ↓
PDF validation
 ↓
Page/text extraction
 ↓
Reading-order normalization
 ↓
Header/footer cleanup
 ↓
Heading detection
 ↓
Paragraph reconstruction
 ↓
Markdown generation
 ↓
Optional cleanup
 ↓
Atomic .md write
 ↓
Index
```

## 3. Converter interface

```rust
pub trait PdfConverter {
    fn convert(
        &self,
        input: &Path,
        options: ConversionOptions,
        progress: ProgressSink,
    ) -> Result<ConversionResult>;
}
```

## 4. Options

Conceptual:

```rust
struct ConversionOptions {
    detect_headings: bool,
    remove_repeated_headers: bool,
    remove_page_numbers: bool,
    add_page_markers: bool,
}
```

## 5. Job lifecycle

```text
Created
  ↓
Validating
  ↓
Converting
  ↓
Writing
  ↓
Indexing
  ↓
Completed
```

Failure:

```text
Any state → Failed
```

Cancellation:

```text
Converting → Cancelling → Cancelled
```

## 6. Progress

Progress should use a structured event:

```text
page_current
page_total
stage
bytes_processed (optional)
message
```

The UI must not infer fake percentages when the backend cannot provide reliable progress.

## 7. 600-page handling

Never process the entire document synchronously on the UI thread.

Use:
- background worker
- bounded memory where possible
- incremental page processing if supported
- streaming/temporary output where appropriate

## 8. Output

Default output:

```text
Books/<sanitized-book-title>.md
```

If a file exists:
- ask before overwrite
- offer a new name

## 9. Quality checks

After conversion:
- output exists
- valid UTF-8
- Markdown is parseable
- page count/result metadata is internally consistent where available
- no accidental binary content
- no excessive repeated headers/footers
- no obvious empty output

## 10. Testing strategy

Create representative PDFs:
- simple single-column text
- multi-column text
- headings
- footnotes
- repeated headers
- repeated page numbers
- 100+ pages
- 600-page stress fixture

The 600-page fixture should be used for performance tests, not necessarily committed to the repository.

## 11. Advanced engines

Future adapters may include:
- Docling
- Marker
- MinerU
- external/local service

The core app should not depend on one engine.
