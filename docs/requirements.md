# Requirements

## 1. Functional Requirements (FR)

### FR-001 Vault management
The application shall allow a user to create a new vault directory and open an existing vault directory.

### FR-002 Markdown source of truth
The application shall store notes as `.md` files in the vault.

### FR-003 Note creation
The user shall be able to create a note from the UI and command palette.

### FR-004 Note editing
The application shall provide a Markdown editing experience with syntax-aware behavior.

### FR-005 Auto-save
The application shall save changes automatically according to a configurable policy.

### FR-006 Manual save
The application shall support explicit save.

### FR-007 File explorer
The application shall display folders and Markdown files in a navigable tree.

### FR-008 Rename/move
The user shall be able to rename and move notes while preserving links where safely possible.

### FR-009 Wikilinks
The application shall support `[[Note Name]]` links.

### FR-010 Link suggestions
While typing a Wikilink, the application shall provide matching notes.

### FR-011 Link navigation
Clicking a resolved Wikilink shall open the target note.

### FR-012 Backlinks
The application shall display notes that link to the current note.

### FR-013 Unresolved links
The application shall display unresolved Wikilinks and allow creation of the target note.

### FR-014 Tags
The application shall recognize Markdown-compatible tags according to the documented tag rules.

### FR-015 YAML frontmatter
The application shall parse YAML frontmatter without making it mandatory.

### FR-016 Search
The application shall provide full-text search across indexed notes.

### FR-017 Search result navigation
Selecting a search result shall open the matching note and relevant location.

### FR-018 Tasks
The application shall recognize Markdown task checkboxes such as `- [ ]` and `- [x]`.

### FR-019 Global task view
The application shall provide a global task view derived from Markdown files.

### FR-020 Task navigation
A task in the global task view shall navigate back to its source note and location.

### FR-021 Projects
The application shall support project notes/metadata without requiring a separate project database as the source of truth.

### FR-022 Library
The application shall provide an organizational view for books/documents.

### FR-023 PDF import
The user shall be able to select a PDF and convert it into Markdown.

### FR-024 PDF progress
PDF conversion shall expose progress when the underlying converter can provide it.

### FR-025 PDF output
The generated Markdown shall be written into the selected vault location.

### FR-026 Conversion cancellation
Long-running conversion shall be cancellable where the backend supports cancellation.

### FR-027 Markdown preview
The application shall provide a rendered reading/preview mode.

### FR-028 Command palette
The application shall provide searchable commands.

### FR-029 Keyboard shortcuts
Common operations shall have configurable/documented keyboard shortcuts.

### FR-030 Recent notes
The application shall remember recent notes in local application state.

### FR-031 Workspace layout
Sidebars and panes shall be resizable and hideable.

### FR-032 Settings
The application shall expose user-facing editor, appearance, indexing, and PDF settings.

### FR-033 Index rebuild
The user shall be able to rebuild local indexes from vault files.

### FR-034 External edits
The application shall detect filesystem changes made outside Nodera and reconcile them safely.

### FR-035 Safe writes
The application shall use atomic/temporary-write strategies where appropriate to reduce corruption risk.

---

## 2. Non-Functional Requirements (NFR)

### NFR-001 Local-first
Core note editing, navigation, search, and task workflows shall work without an internet connection.

### NFR-002 Data ownership
The user's Markdown files shall remain readable without Nodera.

### NFR-003 Reliability
A failed index operation must not corrupt the source Markdown files.

### NFR-004 Recoverability
Indexes must be rebuildable from the vault.

### NFR-005 Performance: startup
Target: open the application and reach an interactive state quickly on a normal developer laptop. Exact benchmark thresholds will be established during performance testing.

### NFR-006 Performance: note open
Target: opening a normal-sized note should feel instantaneous.

### NFR-007 Performance: search
Typical indexed searches should return interactively without noticeable delay.

### NFR-008 Large vault
The architecture shall support growth to tens of thousands of Markdown files without requiring a database migration of the user's source files.

### NFR-009 Large documents
The application shall avoid rendering unnecessarily large complete documents in expensive UI structures.

### NFR-010 600-page PDF
A 600-page text-based PDF shall be treated as a long-running job rather than blocking the UI thread.

### NFR-011 Responsiveness
Long-running indexing/import operations shall execute off the UI thread.

### NFR-012 Cross-platform
Initial target: Windows, Linux, and macOS where supported by the selected Dioxus desktop stack and dependencies.

### NFR-013 Accessibility
Keyboard navigation, focus indication, readable typography, and sufficient contrast shall be treated as first-class requirements.

### NFR-014 Security
The application shall not execute Markdown content as code.

### NFR-015 Privacy
No network connection shall be required for core features.

### NFR-016 Observability
Errors and important background operations shall be logged with structured tracing.

### NFR-017 Maintainability
Business logic shall remain independent of Dioxus UI components.

### NFR-018 Testability
Core parsing, linking, indexing, task extraction, and conversion orchestration shall be unit-testable without launching the UI.

### NFR-019 Extensibility
PDF conversion shall be behind a backend interface so alternative engines can be added later.

### NFR-020 Graceful degradation
If an index is missing or corrupt, the application shall rebuild it from source files rather than treating the vault as unusable.
