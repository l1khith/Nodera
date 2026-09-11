# ADR-0002: Markdown as Source of Truth

## Status

Accepted.

## Decision

User notes are stored as ordinary Markdown files. SQLite/Tantivy are derived indexes/cache.

## Rationale

- portability
- ownership
- interoperability
- easy backups
- resilience to application failure

## Consequences

The application must handle:
- external edits
- rename/move reconciliation
- parsing/indexing
- file conflicts

These are acceptable costs for the portability benefit.
