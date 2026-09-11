# Project Plan

## Phase 0 — Foundation

Goal: prove the architecture before building the full UI.

Deliver:
- Cargo workspace
- core crate
- error model
- basic vault abstraction
- basic file read/write
- logging
- configuration skeleton

Exit criteria:
- create/open vault
- create/read/write Markdown note
- tests pass

## Phase 1 — Notes MVP

Deliver:
- Dioxus shell
- file explorer
- editor
- save/autosave
- create/rename/delete
- tabs or recent-note navigation
- light/dark theme

Exit criteria:
- normal daily note-taking is usable.

## Phase 2 — Knowledge Layer

Deliver:
- Markdown parser
- Wikilinks
- resolution
- backlinks
- tags
- frontmatter
- basic graph data

Exit criteria:
- notes form a connected knowledge base.

## Phase 3 — Search and Tasks

Deliver:
- SQLite index
- Tantivy
- global search
- task extraction
- global task view
- task navigation
- command palette

Exit criteria:
- user can find information and actions from one search system.

## Phase 4 — PDF Import

Deliver:
- converter trait
- Rust-native PDF backend
- conversion job system
- progress UI
- Markdown output
- indexing
- 600-page performance test

Exit criteria:
- selected text-based 600-page PDF converts successfully without freezing the UI.

## Phase 5 — Library/Reading

Deliver:
- library view
- document metadata
- chapter navigation where derivable
- reading mode
- import history/status

## Phase 6 — Hardening

Deliver:
- external file change handling
- recovery/index rebuild
- atomic writes
- crash/failure testing
- performance profiling
- accessibility pass
- packaging

## Phase 7 — Future / Not V1

Candidates:
- plugin API
- sync
- encryption
- optional remote conversion
- collaboration
- local HTTP service
- mobile/web clients

Do not commit to these until V1 demonstrates product value.

## Milestone rule

Each phase must produce a usable increment. Avoid a long period where only infrastructure is being built.
