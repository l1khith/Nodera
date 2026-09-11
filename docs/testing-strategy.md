# Testing Strategy

## 1. Testing layers

```text
Unit tests
   ↓
Integration tests
   ↓
Filesystem/vault tests
   ↓
PDF fixture tests
   ↓
UI/component tests
   ↓
End-to-end desktop tests
```

## 2. Unit tests

Cover:
- Wikilink parsing
- link resolution
- heading extraction
- tag extraction
- frontmatter parsing
- task parsing
- Markdown rendering
- path normalization
- filename sanitization
- PDF Markdown cleanup
- search query parsing

## 3. Integration tests

Cover:
- create note → save → reopen
- rename note → link resolution
- move note → index update
- external file edit → refresh
- index rebuild
- search after modification
- task toggle → source file update
- PDF import → Markdown → indexing

## 4. Failure tests

Explicitly test:
- invalid Markdown
- malformed YAML
- unreadable file
- permission denied
- disk-full simulation where practical
- corrupt index
- interrupted write
- duplicate filenames
- unresolved links
- invalid PDF
- cancelled PDF conversion

## 5. UI tests

Test:
- pane visibility
- note selection
- editor changes
- command palette
- search navigation
- task toggling
- import dialog
- progress state
- error dialogs
- keyboard shortcuts

## 6. PDF fixtures

Maintain a small fixture suite:
- simple text PDF
- heading-heavy PDF
- header/footer PDF
- multi-column PDF
- large PDF

A full 600-page test file may be generated/downloaded in CI or stored outside Git depending on size/licensing.

## 7. Performance tests

Measure:
- startup
- open note
- index 1k/10k/50k notes
- search latency
- graph construction
- 600-page PDF conversion
- memory usage during conversion

Do not set arbitrary hard limits before measuring realistic workloads.

## 8. Regression tests

Every PDF conversion bug should add a minimized fixture or golden output.

Every Markdown parsing bug should add an input/output test.

## 9. Golden tests

For deterministic Markdown transformations:

```text
input.pdf/text fixture
        ↓
expected.md
```

Compare normalized output.

## 10. Test data rule

Never put private user notes into tests.

Use synthetic or properly licensed fixtures.
