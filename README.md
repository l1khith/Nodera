# Nodera

> **Working name:** Nodera  
> **Product:** A local-first, Markdown-native workspace for notes, tasks, projects, knowledge, and document ingestion.

Nodera is a native desktop application written in Rust. Its primary data format is Markdown, so users own portable files rather than being locked into a proprietary notes database.

The product is inspired by the interaction model of knowledge-management tools such as Obsidian, but it is **not intended to be a pixel-for-pixel clone**.

## Core capabilities

- Markdown notes
- Folders and vaults
- Wikilinks (`[[Note Name]]`)
- Backlinks
- Tags and YAML frontmatter
- Full-text search
- Global task/TODO view backed by Markdown checkboxes
- Lightweight project organization
- Library/document organization
- PDF → Markdown import
- Reading mode and Markdown preview
- Command palette
- Keyboard-first workflows
- Local filesystem as the source of truth

## Technology direction

| Area | Choice |
|---|---|
| Language | Rust |
| Desktop UI | Dioxus Desktop |
| Async/runtime | Tokio |
| Serialization | Serde |
| Markdown | Comrak / compatible Markdown AST tooling |
| Search | Tantivy |
| Metadata/index | SQLite |
| File watching | notify |
| PDF | Pluggable Rust PDF extraction backend |
| Logging | tracing |
| Initial networking | None |
| Web client | None |
| Tauri | Not used initially |
| Axum | Not used initially |
| Containers | Not required initially |

## Design principle

```text
Markdown files = source of truth
SQLite/Tantivy  = rebuildable indexes/cache
```

Deleting Nodera must never destroy the user's notes.

## Documentation map

- [Product Description](docs/product-description.md)
- [Requirements: FR/NFR](docs/requirements.md)
- [HLD](docs/hld.md)
- [LLD](docs/lld.md)
- [Architecture](docs/architecture.md)
- [UI/UX Design](docs/design.md)
- [Database and Indexing](docs/database.md)
- [Markdown Specification](docs/markdown-spec.md)
- [PDF Import](docs/pdf-import.md)
- [Search](docs/search-index.md)
- [Tasks](docs/task-model.md)
- [Testing Strategy](docs/testing-strategy.md)
- [Project Plan](docs/project-plan.md)
- [Development Setup](docs/development-setup.md)
- [Architecture Decisions](docs/decisions/)

## Scope rule

Do not add a technology merely because it is popular. Every dependency must solve a demonstrated requirement.

Initial scope deliberately excludes:
- cloud sync
- accounts/authentication
- web client
- collaboration
- plugin marketplace
- remote API
- AI features
