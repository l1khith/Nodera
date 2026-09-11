# UI/UX Design

## 1. Design direction

Nodera should feel familiar to users of Obsidian-like knowledge tools without copying the exact visual design.

Principles:
- quiet workspace
- dense but readable information
- minimal chrome
- keyboard-first
- resizable panes
- strong focus states
- dark/light themes
- no dashboard-card overload

## 2. Primary shell

```text
┌─────────────────────────────────────────────────────────────────────┐
│ Nodera                                      Search   Command  ⚙    │
├──────────────┬──────────────────────────────────┬───────────────────┤
│ WORKSPACE    │ Tabs / Note                      │ CONTEXT           │
│              │                                  │                   │
│ Search       │ # Note title                     │ Backlinks         │
│ Notes        │                                  │ Related           │
│ Tasks        │ Note content...                  │ Properties        │
│ Library      │                                  │                   │
│              │                                  │                   │
│ Vault tree   │                                  │                   │
│              │                                  │                   │
│ + New        │                                  │                   │
├──────────────┴──────────────────────────────────┴───────────────────┤
│ Ready                                      42 notes · 7 tasks       │
└─────────────────────────────────────────────────────────────────────┘
```

Panes:
- left: navigation/explorer
- center: editor/reader
- right: contextual information

Each side pane can be hidden.

## 3. Main navigation

Top-level areas:
- Notes
- Tasks
- Library

Secondary:
- Search
- Graph
- Settings

Vault tree remains available from the workspace.

## 4. Home behavior

Home is optional.

Default behavior should favor reopening the previous workspace/note. Users can configure a home/start view.

## 5. Note editor

Modes:
- Live editing
- Source Markdown
- Reading

The editor must support:
- headings
- emphasis
- lists
- checkboxes
- code blocks
- links
- Wikilinks
- blockquotes
- frontmatter

## 6. Wikilink interaction

Typing:

```text
[[Rust Ow
```

shows:

```text
Rust Ownership
Rust Ownership Patterns
Rust Ownership Notes
```

Enter selects a suggestion.

If no note exists:
- show "Create note"
- unresolved link remains valid Markdown text

## 7. Backlinks panel

Display:
- source note title
- relevant context/snippet
- link location

Clicking an entry opens the source note.

## 8. Task view

Global task view is derived from Markdown.

```text
TODAY
□ Finish editor
□ Review architecture

UPCOMING
□ Test PDF conversion

NO DATE
□ Research graph layout
```

Each task shows its source note.

Changing a checkbox updates the original Markdown file.

## 9. PDF import UX

Entry points:
- Command palette
- Import button
- Context menu
- Drag-and-drop

Dialog:

```text
Import PDF
────────────────────────────
Drop PDF here
or Choose File

Output:
Books/Book.md

Options:
☑ Detect headings
☑ Remove repeated headers
☑ Remove page numbers
☐ Insert page markers

[Cancel] [Convert]
```

## 10. PDF progress

```text
Converting Book.pdf

██████████████████░░░░  78%

Pages        468 / 600
Status       Extracting text
Elapsed      02:14

[Cancel]
```

Completion:

```text
Conversion complete

600 pages
17 chapters
0 conversion errors

[Open Markdown] [Show in Folder]
```

## 11. Search UX

Global search:
- keyboard shortcut
- command palette
- sidebar

Results grouped by:
- notes
- tasks
- library/documents

Search result should show:
- title
- path
- snippet
- matching location

## 12. Command palette

Suggested default shortcut: `Ctrl/Cmd + P`.

Commands:
- New note
- New task/note
- Search
- Import PDF as Markdown
- Open graph
- Toggle sidebar
- Toggle backlinks
- Reading mode
- Rebuild index
- Settings

## 13. Keyboard shortcuts

Initial proposal:

| Action | Windows/Linux | macOS |
|---|---|---|
| New note | Ctrl+N | Cmd+N |
| Search | Ctrl+Shift+F | Cmd+Shift+F |
| Command palette | Ctrl+P | Cmd+P |
| Save | Ctrl+S | Cmd+S |
| Import PDF | Ctrl+Shift+I | Cmd+Shift+I |
| Toggle reading | Ctrl+E | Cmd+E |
| Toggle graph | Ctrl+G | Cmd+G |

Avoid conflicts after testing against editor/browser conventions.

## 14. Empty states

### Empty vault
Explain:
- create note
- import existing Markdown
- import PDF

### No search results
Show:
- query
- suggestion to broaden search
- current scope

### No backlinks
Use simple text:
"This note is not referenced by other notes yet."

### No tasks
"Tasks are created from Markdown checkboxes."

## 15. Accessibility

- keyboard navigation
- visible focus
- scalable text
- semantic labels
- no color-only status indication
- reduced-motion preference
- accessible dialogs

## 16. Responsive desktop behavior

Although this is desktop-only, support narrower windows:
- hide right pane automatically below threshold
- collapse left pane
- retain editor as primary surface
- never allow core controls to disappear without an alternate command

## 17. Visual system

Create design tokens rather than hard-coding colors throughout components:

```text
background
surface
surface-elevated
border
text-primary
text-secondary
accent
success
warning
error
focus
```

Use a restrained accent and neutral surfaces.

## 18. UX rule

Every destructive action must be explicit:
- delete note
- delete folder
- overwrite output
- discard unsaved changes

Non-destructive navigation should remain fast and low-friction.
