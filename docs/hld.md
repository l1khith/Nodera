# High-Level Design (HLD)

## 1. Architectural goal

Build a native Rust desktop application whose UI is replaceable without rewriting the domain/core layer.

```text
┌───────────────────────────────────────────────────────────────┐
│                         Dioxus Desktop                        │
│ UI / navigation / editor / dialogs / task views / graph      │
└──────────────────────────────┬────────────────────────────────┘
                               │
                        Application Layer
                               │
        ┌──────────────────────┼───────────────────────┐
        │                      │                       │
     Vault/Notes            Tasks/Projects         Import Jobs
        │                      │                       │
        └──────────────────────┼───────────────────────┘
                               │
                          Domain/Core
                               │
      ┌──────────────┬─────────┼──────────┬───────────────┐
      │              │         │          │               │
 Filesystem      Markdown    Links      Indexing       PDF Engine
      │              │         │          │               │
      ▼              ▼         ▼          ▼               ▼
   .md files       AST      graph     SQLite/Tantivy   Rust backend
```

## 2. Major components

### Desktop UI
Dioxus components, application state, routing/workspace layout, keyboard handling, dialogs.

### Application layer
Coordinates commands and use cases. It should not contain UI rendering code.

Examples:
- CreateNote
- OpenNote
- SaveNote
- SearchVault
- ToggleTask
- ImportPdf
- RebuildIndex

### Domain/core
Defines note, link, task, project, document, metadata, and vault concepts.

### Filesystem layer
Reads/writes files, watches external changes, performs atomic writes, normalizes paths.

### Markdown layer
Parses Markdown and extracts:
- headings
- links
- tasks
- tags
- frontmatter

### Index layer
Maintains rebuildable local indexes.

### PDF layer
Defines a converter interface. The first implementation may use a Rust-native PDF extraction library. More advanced engines can be added later.

## 3. Data flow: opening a note

```text
User clicks note
      ↓
Application command: OpenNote
      ↓
Vault service resolves path
      ↓
Filesystem reads .md
      ↓
Markdown parser builds document model
      ↓
UI receives note state
      ↓
Editor renders content
```

## 4. Data flow: editing

```text
Editor change
    ↓
Debounced save scheduler
    ↓
Atomic filesystem write
    ↓
Indexer update
    ↓
Search/link/task indexes refreshed
```

## 5. Data flow: external file change

```text
Filesystem watcher
      ↓
Changed path
      ↓
Debounce/coalesce events
      ↓
Read changed file
      ↓
Reparse
      ↓
Update indexes
      ↓
Refresh open document if safe
```

## 6. Data flow: PDF import

```text
User selects PDF
      ↓
Validate input
      ↓
Create ConversionJob
      ↓
PDF backend
      ↓
Extract document structure/text
      ↓
Normalize Markdown
      ↓
Write .md atomically
      ↓
Index generated note
      ↓
Open generated note
```

## 7. Concurrency

Use Tokio where asynchronous orchestration is useful, but do not force all CPU-heavy parsing into async functions.

CPU-heavy operations should run in appropriate blocking/task pools.

The UI must never wait synchronously for:
- PDF conversion
- full vault indexing
- large search index rebuilds
- large file parsing

## 8. Failure boundaries

A failed operation should be isolated.

Examples:

```text
PDF conversion failure
    ≠
vault failure

Index corruption
    ≠
Markdown corruption

UI rendering error
    ≠
source file loss
```

## 9. Future architecture

Potential later additions:
- optional local HTTP service
- optional remote conversion backend
- sync
- plugins
- collaboration

None are dependencies of V1.
