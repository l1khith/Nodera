# Database and Index Design

## 1. Core rule

The database is **not the source of truth**.

```text
Markdown files
     │
     ▼
Parser/indexer
     │
     ├── SQLite metadata/index
     └── Tantivy full-text index
```

If either index disappears, Nodera rebuilds it.

## 2. Vault layout

```text
MyVault/
├── Notes/
├── Projects/
├── Books/
├── Attachments/
└── .nodera/
    ├── index.sqlite
    ├── tantivy/
    └── state.json
```

The `.nodera` directory is application-managed cache/state.

## 3. SQLite responsibilities

Use SQLite for:
- note metadata
- file hashes/mtime
- normalized paths
- link relationships
- task locations
- tags
- frontmatter metadata
- indexing status

Do not store complete Markdown bodies as the canonical document.

## 4. Proposed schema

```sql
CREATE TABLE files (
    id            TEXT PRIMARY KEY,
    path          TEXT NOT NULL UNIQUE,
    title         TEXT NOT NULL,
    extension     TEXT NOT NULL,
    content_hash  TEXT NOT NULL,
    size_bytes    INTEGER NOT NULL,
    modified_ns   INTEGER NOT NULL,
    indexed_at_ns INTEGER NOT NULL
);

CREATE TABLE links (
    source_id     TEXT NOT NULL,
    target_path   TEXT NOT NULL,
    target_id     TEXT,
    start_offset  INTEGER,
    end_offset    INTEGER,
    PRIMARY KEY (source_id, target_path, start_offset)
);

CREATE TABLE tasks (
    id            TEXT PRIMARY KEY,
    note_id       TEXT NOT NULL,
    line_number   INTEGER NOT NULL,
    checked       INTEGER NOT NULL,
    text          TEXT NOT NULL,
    due_date      TEXT
);

CREATE TABLE tags (
    note_id       TEXT NOT NULL,
    tag           TEXT NOT NULL,
    PRIMARY KEY (note_id, tag)
);

CREATE TABLE properties (
    note_id       TEXT NOT NULL,
    key           TEXT NOT NULL,
    value_json    TEXT NOT NULL,
    PRIMARY KEY (note_id, key)
);
```

Foreign-key and index choices should be finalized during implementation.

## 5. Tantivy

Tantivy is responsible for full-text search.

Indexed fields may include:
- title
- path
- body
- headings
- tags

Potentially stored fields:
- note ID
- path
- title

Do not duplicate unnecessary large bodies into stored fields if they are not needed for result display.

## 6. Index lifecycle

### Startup

```text
Open vault
   ↓
Open SQLite
   ↓
Validate index metadata
   ↓
Scan for changed/new/deleted files
   ↓
Incrementally index
   ↓
Ready
```

### Full rebuild

```text
Delete/recreate derived indexes
        ↓
Walk vault
        ↓
Parse each supported file
        ↓
Populate SQLite
        ↓
Populate Tantivy
        ↓
Commit
```

## 7. File identity

Paths can change, so path alone should not be treated as permanent identity.

Use an internal ID and track:
- normalized relative path
- content hash
- filesystem metadata

Identity reconciliation needs careful testing around renames.

## 8. Concurrency

SQLite writes should be serialized through a dedicated service boundary.

Search reads can be concurrent.

Tantivy commits should be coordinated with the index service.

## 9. Corruption/recovery

If `.nodera/index.sqlite` is corrupt:
1. preserve source files
2. move broken derived index aside
3. recreate
4. rebuild

If Tantivy index is corrupt:
1. delete/recreate Tantivy directory
2. rebuild from SQLite/source files as designed

## 10. Versioning

Store an index schema/version number.

Example:

```text
index_schema_version = 1
```

When incompatible:
- migrate if safe
- otherwise rebuild

## 11. Database rule

No feature may require the database to recover the user's note text if the Markdown file exists.
