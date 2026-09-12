# Implementation Checklist

## Foundation
- [x] Create Cargo workspace
- [x] Create core crate
- [x] Create desktop crate
- [x] Add structured errors
- [x] Add tracing
- [x] Implement vault open/create
- [x] Implement atomic file writes

## Notes
- [x] File tree
- [x] Create note
- [x] Open note
- [x] Edit note
- [x] Save note
- [x] Rename
- [x] Move
- [x] Delete confirmation
- [x] Recent notes

## Markdown
- [x] Parser
- [x] Renderer
- [x] Wikilinks
- [x] Link resolution
- [x] Backlinks
- [x] Tags
- [x] Frontmatter
- [x] Tasks

## Index
- [x] SQLite schema
- [x] Incremental index
- [x] Rebuild
- [x] Tantivy index
- [x] Search
- [x] Result snippets

## Tasks
- [x] Global task view
- [x] Toggle from task view
- [x] Navigate to source
- [x] Date grouping only after syntax decision

## PDF
- [x] Converter trait
- [x] Rust PDF backend
- [x] Conversion job
- [x] Progress
- [x] Cancellation
- [x] Header/footer cleanup
- [x] Heading detection
- [x] Output naming
- [x] 600-page benchmark

## UX
- [x] Three-pane shell
- [x] Resizable panes
- [x] Hideable panes
- [x] Command palette
- [x] Keyboard shortcuts
- [x] Empty states
- [x] Error states
- [x] Settings
- [x] Light/dark themes
- [x] Accessibility

## Hardening
- [x] External file changes
- [x] Crash recovery
- [x] Index recovery
- [x] Large-vault tests
- [x] Performance profiling
- [x] Packaging
- [x] Documentation
