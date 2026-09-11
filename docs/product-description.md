# Product Description

## 1. Product name

**Nodera** — working name.

The name should be validated for package/repository/domain/trademark availability before public release.

## 2. Product vision

Nodera is a local-first knowledge workspace where notes, tasks, projects, books, and imported documents live in one connected Markdown-based system.

The product should feel like a calm, fast desktop workspace rather than a dashboard or project-management SaaS.

## 3. Problem

Users often split their knowledge across:
- Markdown notes
- TODO applications
- project documents
- PDFs/books
- browser bookmarks
- disconnected search systems

The result is duplicated context and weak relationships between information and actions.

Nodera addresses this by keeping information in portable Markdown while indexing relationships, tasks, and searchable content.

## 4. Target users

### Primary
- Developers and technical users
- Students and researchers
- Writers and knowledge workers
- Users who prefer local files and Markdown

### Secondary
- Users migrating from proprietary note systems
- Users who maintain personal project documentation
- Readers who want to turn text-heavy PDFs into editable notes

## 5. Product principles

1. **Local first** — no account is required.
2. **User owns the files** — Markdown is the source of truth.
3. **Fast by default** — opening a note should feel instantaneous.
4. **Keyboard friendly** — every common action has a shortcut.
5. **Connected knowledge** — links, backlinks, tasks, and projects share the same model.
6. **Progressive complexity** — simple workflows stay simple.
7. **Graceful failure** — indexes can be rebuilt; files remain usable.
8. **Import, don't imprison** — PDF conversion produces ordinary Markdown.

## 6. Product pillars

### Notes
Create, edit, organize, link, search, and read Markdown notes.

### Tasks
Use standard Markdown checkboxes and provide a global task view.

### Projects
Group notes and tasks around a project without introducing a mandatory proprietary project database.

### Library
Organize books and documents. Imported PDFs become ordinary Markdown notes.

### Knowledge graph
Build relationships from Wikilinks, backlinks, tags, and metadata.

### Document ingestion
Convert supported PDFs to Markdown locally through a pluggable Rust PDF engine.

## 7. What Nodera is not

- Not a cloud-first collaboration suite
- Not Jira
- Not a dedicated PDF reader
- Not a web application
- Not a proprietary database-first notes system
- Not a mandatory AI assistant

## 8. Success criteria for V1

A user should be able to:

1. Create/open a vault.
2. Create and edit Markdown notes.
3. Organize notes in folders.
4. Link notes with `[[wikilinks]]`.
5. Navigate backlinks.
6. Search the vault.
7. Create/check tasks inside notes.
8. See all tasks globally.
9. Import a text-based PDF into Markdown.
10. Continue editing the generated Markdown as a normal note.
11. Close/reopen the application without losing data.
