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
- [ ] File tree
- [ ] Create note
- [ ] Open note
- [ ] Edit note
- [ ] Save note
- [ ] Rename
- [ ] Move
- [ ] Delete confirmation
- [ ] Recent notes

## Markdown
- [ ] Parser
- [ ] Renderer
- [ ] Wikilinks
- [ ] Link resolution
- [ ] Backlinks
- [ ] Tags
- [ ] Frontmatter
- [ ] Tasks

## Index
- [ ] SQLite schema
- [ ] Incremental index
- [ ] Rebuild
- [ ] Tantivy index
- [ ] Search
- [ ] Result snippets

## Tasks
- [ ] Global task view
- [ ] Toggle from task view
- [ ] Navigate to source
- [ ] Date grouping only after syntax decision

## PDF
- [ ] Converter trait
- [ ] Rust PDF backend
- [ ] Conversion job
- [ ] Progress
- [ ] Cancellation
- [ ] Header/footer cleanup
- [ ] Heading detection
- [ ] Output naming
- [ ] 600-page benchmark

## UX
- [ ] Three-pane shell
- [ ] Resizable panes
- [ ] Hideable panes
- [ ] Command palette
- [ ] Keyboard shortcuts
- [ ] Empty states
- [ ] Error states
- [ ] Settings
- [ ] Light/dark themes
- [ ] Accessibility

## Hardening
- [ ] External file changes
- [ ] Crash recovery
- [ ] Index recovery
- [ ] Large-vault tests
- [ ] Performance profiling
- [ ] Packaging
- [ ] Documentation
