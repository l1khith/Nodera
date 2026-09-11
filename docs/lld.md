# Low-Level Design (LLD)

## 1. Workspace/crate layout

```text
nodera/
├── Cargo.toml
├── crates/
│   ├── nodera-core/
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── note.rs
│   │       ├── task.rs
│   │       ├── project.rs
│   │       ├── link.rs
│   │       ├── metadata.rs
│   │       ├── vault.rs
│   │       └── error.rs
│   │
│   ├── nodera-markdown/
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── parser.rs
│   │       ├── renderer.rs
│   │       ├── wikilink.rs
│   │       ├── task_parser.rs
│   │       └── frontmatter.rs
│   │
│   ├── nodera-index/
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── database.rs
│   │       ├── search.rs
│   │       ├── links.rs
│   │       ├── tasks.rs
│   │       └── rebuild.rs
│   │
│   ├── nodera-pdf/
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── converter.rs
│   │       ├── job.rs
│   │       ├── progress.rs
│   │       └── backends/
│   │
│   └── nodera-desktop/
│       └── src/
│           ├── main.rs
│           ├── app.rs
│           ├── state.rs
│           ├── commands.rs
│           └── components/
│               ├── shell.rs
│               ├── sidebar.rs
│               ├── editor.rs
│               ├── search.rs
│               ├── tasks.rs
│               ├── library.rs
│               ├── import.rs
│               └── settings.rs
```

## 2. Core types

Conceptual types:

```rust
struct NoteId(String);

struct Note {
    id: NoteId,
    path: PathBuf,
    title: String,
    content: String,
}

struct Link {
    source: NoteId,
    target_path: String,
    target_id: Option<NoteId>,
    start: usize,
    end: usize,
}

struct Task {
    id: String,
    note_id: NoteId,
    checked: bool,
    text: String,
    line: u32,
}

struct Project {
    note_id: NoteId,
    status: ProjectStatus,
}
```

Exact representation should be decided after implementation experiments.

## 3. Services

### VaultService

Responsibilities:
- open/validate vault
- list files
- create note
- rename note
- move note
- read note
- write note
- delete note with explicit confirmation

### LinkService

Responsibilities:
- parse Wikilinks
- resolve targets
- generate backlinks
- update links after rename/move where supported

### TaskService

Responsibilities:
- parse checkbox tasks
- update checkbox state in source Markdown
- query global task index

### SearchService

Responsibilities:
- index note content
- execute queries
- return ranked results
- expose snippets/highlights

### PdfConversionService

Responsibilities:
- validate PDF
- create job
- stream progress
- return Markdown result
- report conversion errors
- support cancellation if backend supports it

## 4. Event model

Recommended internal events:

```text
VaultOpened
NoteCreated
NoteChanged
NoteRenamed
NoteMoved
NoteDeleted
IndexStarted
IndexProgress
IndexCompleted
PdfImportStarted
PdfImportProgress
PdfImportCompleted
PdfImportFailed
TaskChanged
```

Events should be lightweight and should not carry giant document bodies unnecessarily.

## 5. Command model

UI invokes application commands:

```text
CreateNote
OpenNote
SaveNote
RenameNote
MoveNote
DeleteNote
Search
ToggleTask
ImportPdf
RebuildIndex
OpenGraph
TogglePane
```

The UI should not directly manipulate SQLite or filesystem paths.

## 6. Error model

Use domain-specific errors and preserve actionable context.

Example categories:

```text
VaultError
FileError
ParseError
IndexError
SearchError
PdfError
ValidationError
OperationCancelled
```

Errors should reach the UI as human-readable messages plus structured diagnostic information for logs.

## 7. Threading

- UI state: UI thread/runtime.
- File I/O: async or blocking task depending on operation.
- PDF conversion: background worker.
- Index rebuild: background worker.
- Search: background/index service.
- Rendering: avoid blocking on full-vault operations.

## 8. Atomic writes

Preferred strategy:

```text
write temp file
    ↓
flush/sync where appropriate
    ↓
rename temp → destination
```

The exact durability guarantees should be documented per platform.

## 9. Rename semantics

When a note is renamed:
1. Update filename.
2. Update its index record.
3. Resolve existing links again.
4. Optionally rewrite links if the user enables safe link maintenance.
5. Never silently rewrite arbitrary text that only resembles a link.

## 10. Open-note state

The UI should track:
- note ID/path
- dirty state
- cursor/selection
- scroll position
- editor mode
- save status

This state is transient and not the source of truth.
