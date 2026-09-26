# Nodera Product Catalog

**Document Status**: Authoritative Engineering & Product Capability Inventory  
**Current Release**: v0.6.0  
**Last Updated**: 2026-09-24  
**Maintainer**: Nodera Core Architecture & Product Working Group  

---

## 1. Executive Product Overview

### What is Nodera?
**Nodera** is a local-first, native desktop knowledge and work workspace where human thoughts, task commitments, software projects, books, and literature live in one connected system. 

Built with **Rust** and **Dioxus Desktop**, Nodera treats plain **CommonMark Markdown** as the canonical source of truth for notes and documentation, and external **source code trees** (`Cargo.toml`, `*.rs`) as the canonical source of truth for software projects. All databases, full-text indexes, spatial force graphs, and caches are strictly **derived, self-healing artifacts** that can be wiped and regenerated from disk at any time.

---

## 2. Core Product & Architectural Principles

1. **Local-First & Data Ownership**: No mandatory accounts, telemetry, or network connectivity. Markdown notes and source files remain 100% human-readable outside Nodera.
2. **Canonical Sources vs. Derived Artifacts**:
   - Canonical human content: Plaintext Markdown (`.md`).
   - Canonical code content: Native repository source trees.
   - Derived, disposable state: SQLite (`.db`), Tantivy indexes, and `.nodera/` graph snapshots.
3. **Graph Domain Separation**:
   $$\text{Markdown Knowledge Graph} \neq \text{External Project Graph}$$
   Human wikilink thoughts and software symbol trees never mix into one cluttered graph. They remain distinct domains, joined only via a typed future cross-domain relationship layer.
4. **Unified Domain Model with Multiple Projections**:
   A single coherent domain entity (e.g. a Task, a Reading Session, or a Git Commit) can be projected into multiple views (Today Dashboard, Global Task View, Project Timeline, Knowledge Graph, or Habit Evidence) without duplicating source data.
5. **Activity is Distinct from Planned Work**:
   A task or a calendar entry is an *intention* ($\text{Plan}$). An edit, commit, reading duration, or journal entry is verifiable *evidence* ($\text{Activity}$).
6. **Evidence-Based Habits**: Habits are verified through collected activity telemetry (Git commits, word count deltas, reading minutes) rather than dumb checkmarks.
7. **Native Capabilities Over Plugin Fragility**: Nodera implements core knowledge and workflow primitives natively in safe, high-performance Rust rather than maintaining a fragile, fragmented plugin marketplace.
8. **Calm, Fast, Keyboard-Driven**: Instant startup, sub-5ms indexed search, $O(N \log N)$ Barnes-Hut graph physics, and keyboard shortcuts for every action.

---

## 3. Product Architecture Map

```
                             NODERA WORKSPACE
                                    │
       ┌────────────────────────────┼────────────────────────────┐
       │                            │                            │
   KNOWLEDGE                       WORK                       ACTIVITY
       │                            │                            │
   ┌───┴───┐                    ┌───┴───┐                    ┌───┴───┐
   │ Notes │ Markdown           │ Tasks │ Checkboxes         │ Code  │ Edits/Commits
   │ Books │ PDF Ingestion      │ Goals │ Milestones         │ Read  │ Reading Sessions
   │ Papers│ BibTeX Citations   │ Plans │ Horizons           │ Write │ Word Counts
   │ Code  │ AST Symbols        │ Proj  │ Repositories       │ Log   │ Journal Entries
   └───┬───┘                    └───┬───┘                    └───┬───┘
       │                            │                            │
       └────────────────────────────┼────────────────────────────┘
                                    │
                               REFLECTION
                                    │
                   Daily • Weekly • Monthly • Quarterly
                                    │
                      UNIFIED LOCAL DOMAIN MODEL
                                    │
         ┌──────────────────────────┴──────────────────────────┐
         │                                                     │
   PROJECTIONS & SURFACES                               SEARCH & CONTEXT
         │                                                     │
   ┌─────┴────────────────────────┐              ┌─────────────┴─────────────┐
   │ Today Calendar Dashboard     │              │ Tantivy Full-Text BM25    │
   │ Global Task Board            │              │ SQLite Relational Index   │
   │ Knowledge Graph (Markdown)   │              │ Symbol & Ref Lookup       │
   │ Project Graph (Source Code)  │              │ Context Assembly Engine   │
   │ Library & Reading Cards      │              │ Future Local AI / LLM     │
   │ Review Queue Triage          │              └───────────────────────────┘
   └──────────────────────────────┘
                                    │
         ┌──────────────────────────┴──────────────────────────┐
         │                                                     │
   [Markdown Knowledge Graph]         ≠          [External Project Graph]
   (wikilinks, tags, backlinks)                  (modules, structs, fns, calls)
         │                                                     │
         └──────────────────────────┬──────────────────────────┘
                                    ▼
                 [Future Cross-Domain Relationship Layer]
                 (Notes ↔ Symbols ↔ Tasks ↔ Activities)
```

---

## 4. Controlled Status & Evaluation Taxonomy

### Implementation Status
- **`COMPLETE`**: Implemented, fully tested, integrated into the UI/CLI, and behaving as intended with production quality.
- **`WORKING`**: Implemented and usable in daily workflows, but may have non-blocking functional gaps or known constraints.
- **`PARTIALLY_WORKING`**: Critical components function, but the full user workflow is incomplete or missing end-to-end integration.
- **`IMPLEMENTED_NEEDS_REFINEMENT`**: Functionally working, but requires UX polish, performance tuning, architectural cleanup, or accessibility improvements.
- **`BROKEN`**: Code exists in repository, but is currently unusable, panics, or fails verification.
- **`PLANNED`**: Explicitly agreed and designed; scheduled for implementation.
- **`PROPOSED`**: Product idea accepted for roadmap consideration; architecture defined but not yet committed to a release milestone.
- **`RESEARCH_REQUIRED`**: High conceptual value, but technical or product design investigation is needed before planning.
- **`DEFERRED`**: Intentionally postponed to a later major version.
- **`OUT_OF_SCOPE`**: Explicitly excluded from Nodera’s product direction.

### Quality Flags
- `GOOD`: Meets production quality bar.
- `NEEDS_UI_REFINEMENT`: Visual styling, layout, or typography needs polish.
- `NEEDS_UX_REFINEMENT`: Interaction flow, feedback, or discoverability needs improvement.
- `NEEDS_PERFORMANCE_REVIEW`: Latency, memory, or CPU utilization optimization needed.
- `NEEDS_RELIABILITY_REVIEW`: Edge-case error handling, recovery, or race condition review needed.
- `NEEDS_ARCHITECTURAL_REVIEW`: Structural refactoring needed to prevent technical debt.
- `NEEDS_ACCESSIBILITY_REVIEW`: Keyboard navigation, screen-reader, or high-contrast adjustments needed.
- `NEEDS_DOCUMENTATION`: User or developer documentation incomplete.

### Maturity Levels
- `M0`: Concept / Design Phase
- `M1`: Prototype / Proof of Concept
- `M2`: Functional / Alpha Feature
- `M3`: Integrated / Beta Feature
- `M4`: Production Ready / Hardened

### Priorities
- `P0`: Critical / Core Foundation
- `P1`: High Value / Primary Roadmap
- `P2`: Medium Value / Secondary Roadmap
- `P3`: Low Value / Polish & Niche
- `P4`: Long-Term Horizon

---

## 5. Master Product Capability Index

| Domain | Feature | Status | Maturity | Quality | Priority | Primary Source / Evidence |
|---|---|---|---|---|---|---|
| Core Workspace | Vault Management & Discovery | COMPLETE | M4 | GOOD | P0 | `nodera-core::Vault`, `VaultService` |
| Core Workspace | Three-Pane Desktop Shell | COMPLETE | M4 | GOOD | P0 | `nodera-desktop::app`, `sidebar`, `inspector` |
| Core Workspace | Workspace Preferences Persistence | COMPLETE | M4 | GOOD | P1 | `nodera-desktop::state::AppPreferences` |
| Core Workspace | Multi-Tab Document Workspace | COMPLETE | M4 | GOOD | P0 | `nodera-desktop::components::TabBar` |
| Core Workspace | Navigation History & Breadcrumbs | COMPLETE | M4 | GOOD | P0 | `nodera-desktop::state::nav_history` |
| Markdown & Notes | Markdown AST Parser & Safe HTML | COMPLETE | M4 | GOOD | P0 | `nodera-markdown::parser`, `renderer` |
| Markdown & Notes | Note Editor & Keystroke Auto-Save | COMPLETE | M4 | GOOD | P0 | `nodera-desktop::components::editor` |
| Markdown & Notes | YAML Frontmatter & Properties | COMPLETE | M4 | GOOD | P0 | `nodera-markdown::frontmatter`, `inspector` |
| Markdown & Notes | Callouts, Syntax Code & LaTeX Math | COMPLETE | M4 | GOOD | P1 | `nodera-markdown::renderer`, `syntect` |
| Markdown & Notes | Note Lifecycle & Safe Atomic Writes | COMPLETE | M4 | GOOD | P0 | `nodera-core::service::save_note_atomic` |
| Markdown & Notes | Note Templates Engine | COMPLETE | M4 | GOOD | P1 | `nodera-desktop::components::dialogs` |
| Markdown & Notes | Soft Delete & Trash Bin Recovery | COMPLETE | M4 | GOOD | P1 | `nodera-core::service::trash`, `dialogs` |
| Knowledge Hygiene| Knowledge Review Queue | COMPLETE | M3 | GOOD | P1 | `nodera-desktop::components::review_queue` |
| Knowledge Hygiene| Quick Thought Capture Modal | COMPLETE | M3 | GOOD | P1 | `nodera-desktop::components::quick_capture` |
| Knowledge Hygiene| Vault Health Doctor (Audits) | COMPLETE | M3 | GOOD | P1 | `nodera-desktop::components::vault_health_modal` |
| Knowledge Graph | Wikilinks & Backlinks Resolution | COMPLETE | M4 | GOOD | P0 | `nodera-markdown::links::LinkGraph` |
| Knowledge Graph | Algorithmic Related Notes Engine | COMPLETE | M3 | GOOD | P1 | `nodera-index::similarity`, `inspector` |
| Knowledge Graph | 2D Force-Directed Simulation | COMPLETE | M4 | GOOD | P0 | `nodera-desktop::components::graph_view` |
| Knowledge Graph | Barnes-Hut QuadTree Physics | COMPLETE | M4 | GOOD | P0 | `graph_view::barnes_hut` ($O(N \log N)$) |
| Knowledge Graph | Graph Controls & Filter Drawer | COMPLETE | M4 | GOOD | P1 | `nodera-desktop::components::inspector` |
| Knowledge Graph | Contextual Local Graph View | COMPLETE | M4 | GOOD | P1 | `nodera-desktop::components::graph_view` |
| Knowledge Graph | Graph Domain Separation | COMPLETE | M4 | GOOD | P0 | `nodera-desktop::state::active_project` |
| Knowledge Graph | Cross-Domain Relationship Layer | PROPOSED | M0 | - | P2 | Product Research / Architecture |
| Search & Index | Full-Text Search (Tantivy BM25) | COMPLETE | M4 | GOOD | P0 | `nodera-index::tantivy` |
| Search & Index | SQLite Relational Vault Index | COMPLETE | M4 | GOOD | P0 | `nodera-index::sqlite` |
| Search & Index | Parallel Parsing Pipeline (Rayon) | COMPLETE | M4 | GOOD | P0 | `nodera-index::coordinator` |
| Search & Index | Self-Healing Index Recovery | COMPLETE | M4 | GOOD | P0 | `nodera-index::coordinator::recover` |
| Search & Index | Fuzzy Command Palette (`Ctrl+P`) | COMPLETE | M4 | GOOD | P0 | `nodera-desktop::command_palette` |
| Search & Index | Semantic Vector Embedding Search | PROPOSED | M0 | - | P3 | Product Research |
| PDF Intelligence| Local PDF-to-Markdown Ingestion | COMPLETE | M4 | GOOD | P0 | `nodera-pdf::converter::NativePdfConverter` |
| PDF Intelligence| PDF Layout & Chapter Cleaning | COMPLETE | M4 | GOOD | P0 | `nodera-pdf::cleanup` |
| PDF Intelligence| PDF Annotation & Highlight Extractor| COMPLETE | M3 | GOOD | P1 | `nodera-pdf::annotations`, `modal` |
| PDF Intelligence| OCR for Scanned Documents | RESEARCH_REQUIRED | M0 | - | P3 | Product Research |
| Project Intel | Cargo Project & Workspace Discovery | COMPLETE | M4 | GOOD | P0 | `nodera-project::discovery` |
| Project Intel | Rust AST Parser Core (`syn`) | COMPLETE | M4 | GOOD | P0 | `nodera-parser-core::languages::rust` |
| Project Intel | Derived Project Graph Schema | COMPLETE | M4 | GOOD | P0 | `nodera-project::graph::ProjectGraph` |
| Project Intel | SQLite Project Symbol Index | COMPLETE | M4 | GOOD | P0 | `nodera-project::index::ProjectIndex` |
| Project Intel | Incremental Sync (`nodera update`) | COMPLETE | M4 | GOOD | P0 | `nodera-project::sync` |
| Project Intel | Desktop Project Graph Visualization | COMPLETE | M3 | GOOD | P0 | `nodera-desktop::components::graph_view` |
| Project Intel | Symbol Context Panel Inspector | COMPLETE | M3 | GOOD | P1 | `nodera-desktop::components::inspector::SymbolInspector` |
| Project Intel | Jump-to-Source in Local IDE | COMPLETE | M3 | GOOD | P1 | `nodera-desktop::components::inspector::open_file_in_editor` |
| Project Intel | Multi-Language Parser Support | RESEARCH_REQUIRED | M0 | - | P2 | Product Research |
| Tasks & Work | Markdown Checkbox Tasks Sync | COMPLETE | M4 | GOOD | P0 | `nodera-markdown::task_parser` |
| Tasks & Work | Global Task Board (`TaskView`) | COMPLETE | M4 | GOOD | P0 | `nodera-desktop::components::task_view` |
| Tasks & Work | Due Dates & Scheduling Syntax | COMPLETE | M3 | GOOD | P1 | `nodera-markdown::task_parser`, `nodera-index::sqlite` |
| Tasks & Work | Goal & Milestone Lineage | PROPOSED | M0 | - | P1 | Product Research — Obsidian/Compass |
| Planning & Goals| Multi-Scale Planning Hierarchy | PROPOSED | M0 | - | P1 | Product Research — Obsidian/Compass |
| Planning & Goals| Quarterly Personal Retreat System | PROPOSED | M0 | - | P2 | Product Research — Obsidian/Compass |
| Activity Engine | Universal Activity Model | COMPLETE | M3 | GOOD | P1 | `nodera-index::models::Activity`, `nodera-index::sqlite` |
| Activity Engine | Coding Activity Collector (Git) | PROPOSED | M0 | - | P1 | Product Research — Obsidian/Compass |
| Activity Engine | Reading Duration Collector | PARTIALLY_WORKING | M2 | NEEDS_ARCHITECTURAL_REVIEW | P1 | `nodera-desktop::state::reading_progress` |
| Activity Engine | Writing Volume Collector | PROPOSED | M0 | - | P2 | Product Research — Obsidian/Compass |
| Habits & Routines| Habit Entity Definitions & Cadence | PROPOSED | M0 | - | P1 | Product Research — Obsidian/Compass |
| Habits & Routines| Evidence-Based Habit Evaluation | RESEARCH_REQUIRED | M0 | - | P1 | Product Research — Obsidian/Compass |
| Habits & Routines| Streak Analytics & Heatmap Grid | PROPOSED | M0 | - | P2 | Product Research — Obsidian/Compass |
| Reflection | Daily Notes Calendar (`TodayView`) | COMPLETE | M4 | GOOD | P0 | `nodera-desktop::calendar`, `today_view` |
| Reflection | Structured Daily Questions | PARTIALLY_WORKING | M2 | NEEDS_UX_REFINEMENT | P1 | `nodera-desktop::templates` |
| Reflection | Cadenced Reviews (Weekly/Monthly) | PROPOSED | M0 | - | P1 | Product Research — Obsidian/Compass |
| Reading & Lit | Book Library View & Progress Bars | COMPLETE | M4 | GOOD | P1 | `nodera-desktop::components::library_view` |
| Reading & Lit | BibTeX / Zotero Citation Picker | COMPLETE | M3 | GOOD | P1 | `nodera-core::bib`, `citation_picker_modal` |
| Reading & Lit | Progressive Summarization Tooling | PROPOSED | M0 | - | P2 | Product Research — Obsidian/Compass |
| Writing | Distraction-Free Long-Form Editor | COMPLETE | M3 | GOOD | P0 | `nodera-desktop::components::editor` |
| Writing | Document Statistics & Reading Time | COMPLETE | M4 | GOOD | P1 | `nodera-desktop::components::statusbar` |
| Writing | Multi-Chapter Manuscript Compiler | PROPOSED | M0 | - | P2 | Product Roadmap |
| Dashboards | Today Command Center Dashboard | COMPLETE | M3 | GOOD | P0 | `nodera-desktop::components::today_view` |
| Dashboards | Project Overview Dashboard | PARTIALLY_WORKING | M2 | NEEDS_UX_REFINEMENT | P1 | `nodera-desktop::sidebar`, `graph_view` |
| Dashboards | Weekly Review Dashboard Projection | PROPOSED | M0 | - | P1 | Product Research — Obsidian/Compass |
| Context & AI | Context Assembly Engine | PARTIALLY_WORKING | M2 | NEEDS_ARCHITECTURAL_REVIEW | P1 | `nodera-index::similarity`, `inspector` |
| Context & AI | Embedded Local LLM Assistant | RESEARCH_REQUIRED | M0 | - | P3 | Product Research |
| Headless CLI | Headless Vault Search, Index, Stats | COMPLETE | M4 | GOOD | P1 | `nodera-cli::main` |
| Headless CLI | Headless Project Init, Update, Status | COMPLETE | M4 | GOOD | P0 | `nodera-cli::main` |
| Headless CLI | Windows Protocol Handler (`nodera://`)| COMPLETE | M4 | GOOD | P1 | `nodera-core::protocol`, `nodera-cli` |
| Performance | Sub-5ms 10K Vault Benchmarking | COMPLETE | M4 | GOOD | P0 | `benches/vault_benchmarks.rs` |
| Storage & Sync | Atomic Tempfile Sync & Rollback | COMPLETE | M4 | GOOD | P0 | `nodera-core::service::save_note_atomic` |
| Storage & Sync | File Watcher Self-Write Suppression | COMPLETE | M4 | GOOD | P0 | `nodera-desktop::watcher` |
| Appearance & Theme | Centralized Theme Token Model | COMPLETE | M4 | GOOD | P0 | `nodera-desktop::theme::Theme`, `Color` |
| Appearance & Theme | 6 Built-in Calibrated Themes | COMPLETE | M4 | GOOD | P0 | Nodera Dark, Light, Midnight, Nord, Dracula, Solarized |
| Appearance & Theme | Settings Appearance Theme Selector | COMPLETE | M4 | GOOD | P0 | `nodera-desktop::components::settings_modal` |
| Appearance & Theme | Theme Persistence & Lossless Fallback | COMPLETE | M4 | GOOD | P0 | `nodera-desktop::state::AppPreferences::theme` |

---

## 6. Detailed Domain Specifications

```
===============================================================================
DOMAIN 01: CORE WORKSPACE & APPLICATION SHELL
===============================================================================
```

### Feature: Vault Management & Discovery
- **Status**: `COMPLETE`
- **Maturity**: `M4`
- **Quality**: `GOOD`
- **Priority**: `P0`
- **User Value**: Users can open any local folder as a self-contained knowledge vault. No hidden proprietary lock-in.
- **Current Behavior**: `Vault::create` and `Vault::open_existing` validate directory structure, create/read `.nodera/` metadata, and initialize local database connections. Folder picking utilizes native OS dialogs via `rfd`.
- **Sub-features**:
  - Vault creation with default configuration — `COMPLETE`
  - Opening existing vaults with schema verification — `COMPLETE`
  - Cross-vault state isolation — `COMPLETE`
  - Empty vault onboarding guidance — `COMPLETE`
- **Dependencies**: `nodera-core`, `rfd`
- **Evidence**: `crates/nodera-core/src/vault.rs`, `crates/nodera-desktop/tests/desktop_smoke_test.rs`
- **Next Action**: Monitor native dialog performance across macOS and Linux distributions.

---

### Feature: Three-Pane Desktop Shell
- **Status**: `COMPLETE`
- **Maturity**: `M4`
- **Quality**: `GOOD`
- **Priority**: `P0`
- **User Value**: Provides a structured, distraction-free environment with resizable sidebars, quick navigation, and contextual awareness.
- **Current Behavior**: Left sidebar provides Workspace routes, external Projects list, search, and hierarchical file tree. Center pane renders the active view. Right sidebar conditionally renders the context Inspector.
- **Sub-features**:
  - Resizable sidebar with persistent width — `COMPLETE`
  - Resizable contextual inspector drawer — `COMPLETE`
  - Collapsible panels (`Ctrl+\`) — `COMPLETE`
  - Split view editor (Side-by-side notes) — `WORKING`
- **Known Limitations**: Split view currently supports dual note inspection but lacks synchronized scrolling in preview mode.
- **UI/UX Improvements**: Add a split drag divider handle for fluid resizing between split editor panes.
- **Dependencies**: `dioxus-desktop`
- **Evidence**: `crates/nodera-desktop/src/app.rs`, `crates/nodera-desktop/src/components/sidebar.rs`
- **Next Action**: Polish split-view drag divider styles.

---

### Feature: Multi-Tab Document Workspace
- **Status**: `COMPLETE`
- **Maturity**: `M4`
- **Quality**: `GOOD`
- **Priority**: `P0`
- **User Value**: Allows knowledge workers to keep multiple documents open, pin reference sheets, and switch contexts instantaneously.
- **Current Behavior**: Implements `TabBar` above the editor with dirty state indicator dots, tab close buttons (`Ctrl+W`), pin toggles, close other tabs, and keyboard tab cycling.
- **Sub-features**:
  - Open new tab on note selection — `COMPLETE`
  - Pinned tabs (protected from accidental close) — `COMPLETE`
  - Dirty note indicator dot — `COMPLETE`
  - Tab overflow horizontal scrolling — `COMPLETE`
- **Dependencies**: `nodera-desktop::state`
- **Evidence**: `crates/nodera-desktop/src/components/editor.rs`, `tests/desktop_smoke_test.rs::test_multi_tab_workflow`
- **Next Action**: Implement drag-and-drop tab reordering.

---

```
===============================================================================
DOMAIN 02: MARKDOWN & NOTES ENGINE
===============================================================================
```

### Feature: Markdown AST Parser & Safe HTML Renderer
- **Status**: `COMPLETE`
- **Maturity**: `M4`
- **Quality**: `GOOD`
- **Priority**: `P0`
- **User Value**: Fast, standards-compliant CommonMark rendering with rich extensions for technical and academic writing.
- **Current Behavior**: Parses Markdown via `pulldown-cmark`. Injects syntax-highlighted code blocks via `syntect`, renders LaTeX equations via KaTeX math wrappers, transforms Obsidian-style callouts (`> [!NOTE]`), and safely renders HTML without script execution.
- **Sub-features**:
  - CommonMark standard syntax parsing — `COMPLETE`
  - Multi-language syntax highlighting (`syntect`) — `COMPLETE`
  - Mathematical formula rendering (`$...$`, `$$...$$`) — `COMPLETE`
  - Callout blocks (`[!NOTE]`, `[!TIP]`, `[!WARNING]`, `[!IMPORTANT]`, `[!CAUTION]`) — `COMPLETE`
  - Transclusion embeds (`![[note#section]]`) — `WORKING`
- **Known Limitations**: Transclusion embeds currently render entire target note bodies; section-specific transclusion (`#heading`) is pending parser refinement.
- **Dependencies**: `nodera-markdown`, `pulldown-cmark`, `syntect`
- **Evidence**: `crates/nodera-markdown/src/renderer.rs`, `tests/rust_parser_test.rs`
- **Next Action**: Add block-level transclusion support (`![[note#^blockid]]`).

---

### Feature: Visual Frontmatter & Properties Editor
- **Status**: `COMPLETE`
- **Maturity**: `M4`
- **Quality**: `GOOD`
- **Priority**: `P0`
- **User Value**: Allows users to treat notes as structured database entries with typed metadata without corrupting plain text.
- **Current Behavior**: Parses and injects YAML frontmatter. Right context inspector exposes a visual property grid with text, list, date, number, and checkbox types. Edits update the YAML block atomically.
- **Sub-features**:
  - YAML frontmatter parsing and formatting — `COMPLETE`
  - Visual property row add/remove/edit — `COMPLETE`
  - Array and comma-separated tags recognition — `COMPLETE`
  - System metadata auto-generation (`created_at`, `updated_at`) — `COMPLETE`
- **Dependencies**: `nodera-markdown::frontmatter`, `serde_yaml`
- **Evidence**: `crates/nodera-markdown/src/frontmatter.rs`, `crates/nodera-desktop/src/components/inspector.rs`
- **Next Action**: Add multi-select dropdown type for properties constrained by vault schemas.

---

### Feature: Note Lifecycle & Safe Atomic Writes
- **Status**: `COMPLETE`
- **Maturity**: `M4`
- **Quality**: `GOOD`
- **Priority**: `P0`
- **User Value**: Guarantees zero data loss or partial writes during sudden power losses, crashes, or disk failures.
- **Current Behavior**: Writes are conducted into a hidden sibling temporary file (`.filename.tmp`) and flushed to disk before executing an atomic OS rename. Deletes route through `.nodera/trash/` for recoverable soft deletion.
- **Sub-features**:
  - Atomic write via tempfile rename — `COMPLETE`
  - Safe rename with link preservation — `COMPLETE`
  - Soft deletion to `.trash/` — `COMPLETE`
  - Trash bin viewer modal with single-click restore — `COMPLETE`
  - Permanent purge action — `COMPLETE`
- **Dependencies**: `nodera-core::service`, `tempfile`
- **Evidence**: `crates/nodera-core/src/service.rs`, `crates/nodera-desktop/src/components/dialogs.rs`
- **Next Action**: Add configurable trash auto-purge retention policies (e.g. purge items older than 30 days).

---

```
===============================================================================
DOMAIN 03: KNOWLEDGE HYGIENE & REVIEW
===============================================================================
```

### Feature: Knowledge Review Queue
- **Status**: `COMPLETE`
- **Maturity**: `M3`
- **Quality**: `GOOD`
- **Priority**: `P1`
- **User Value**: Prevents the "collector’s fallacy" by providing a dedicated triage queue for rough notes, literature clippings, and unlinked thoughts.
- **Current Behavior**: Dedicated workspace route (`ReviewQueueView`) categorizes notes using tags and structural markers: Rough Notes (`#rough`, `#fleeting`), Source Notes (`#source`, `#lit`), and Permanent Notes (`#permanent`). Provides single-click promote actions that automatically adjust tags and frontmatter.
- **Sub-features**:
  - Categorized review tabs with live badge counters — `COMPLETE`
  - Rough notes triage panel — `COMPLETE`
  - Source synthesis queue — `COMPLETE`
  - Safe note promotion preserves wikilinks and paths — `COMPLETE`
- **Dependencies**: `nodera-index::review`, `nodera-desktop::components::review_queue`
- **Evidence**: `crates/nodera-desktop/src/components/review_queue.rs`, `tests/knowledge_layer_test.rs`
- **Next Action**: Add scheduled review intervals (e.g. Spaced Repetition review cadences).

---

### Feature: Vault Health Doctor
- **Status**: `COMPLETE`
- **Maturity**: `M3`
- **Quality**: `GOOD`
- **Priority**: `P1`
- **User Value**: Keeps knowledge graphs clean and resilient by finding dead ends, broken wikilinks, and orphaned notes.
- **Current Behavior**: Modal dialog (`VaultHealthModal`) audits the entire link graph in $O(N+E)$ linear time using `TargetResolver`. Lists broken wikilinks with source files and provides one-click note creation to resolve them. Displays disconnected orphan notes with options to link or clean up.
- **Sub-features**:
  - Full-vault broken wikilinks scan — `COMPLETE`
  - Orphan notes inventory — `COMPLETE`
  - One-click target note generator — `COMPLETE`
  - Audit report export — `WORKING`
- **Dependencies**: `nodera-markdown::links`, `nodera-desktop::vault_health_modal`
- **Evidence**: `crates/nodera-desktop/src/components/vault_health_modal.rs`
- **Next Action**: Add automatic suggestion of existing similar note names for broken links (fuzzy link healing).

---

```
===============================================================================
DOMAIN 04: KNOWLEDGE GRAPH & LINKS
===============================================================================
```

### Feature: 2D Force-Directed Simulation & Barnes-Hut Optimization
- **Status**: `COMPLETE`
- **Maturity**: `M4`
- **Quality**: `GOOD`
- **Priority**: `P0`
- **User Value**: Enables users to explore high-level conceptual clusters, discover emergent connections, and navigate notes visually.
- **Current Behavior**: Real force-directed physics engine computing Coulomb repulsion, Hooke spring attraction along wikilinks, center gravity, and label collision prevention. Employs 2D Barnes-Hut spatial decomposition ($\theta = 0.75$) with flat array arena allocation, maintaining $>45\text{ FPS}$ on 10,000 nodes.
- **Sub-features**:
  - Barnes-Hut $O(N \log N)$ QuadTree acceleration — `COMPLETE`
  - Deterministic golden ratio phyllotaxis initial placement — `COMPLETE`
  - Interactive pan, smooth zoom, node drag reheat — `COMPLETE`
  - Connected neighborhood highlighting on hover/select — `COMPLETE`
  - Double-click to open note in editor — `COMPLETE`
  - Unresolved note conceptual nodes with dashed styling — `COMPLETE`
  - Modularity community detection & coloring — `COMPLETE`
  - Graph controls drawer (forces, filters, display thresholds) — `COMPLETE`
- **Dependencies**: `nodera-markdown::links`, `nodera-desktop::components::graph_view`
- **Evidence**: `crates/nodera-desktop/src/components/graph_view.rs`, `tests/graph_behavior_test.rs`
- **Next Action**: Implement 3D WebGL / GPU-accelerated graph projection for 100K+ node vaults.

---

### Feature: Graph Domain Separation
- **Status**: `COMPLETE`
- **Maturity**: `M4`
- **Quality**: `GOOD`
- **Priority**: `P0`
- **User Value**: Protects the personal knowledge graph from being overwhelmed by tens of thousands of compiler-generated code symbols.
- **Current Behavior**: The Markdown Knowledge Graph and External Project Graph are maintained in completely separate domain structures. Switching to an external project switches the graph data source via an adapter without mutating the vault database. Notes and symbols never collide.
- **Sub-features**:
  - Independent project graph model (`ProjectGraph`) — `COMPLETE`
  - Project graph renderer adapter (`to_graph_data`) — `COMPLETE`
  - Active project indicator banner with "Return to Vault" button — `COMPLETE`
  - Zero cross-contamination test guarantees — `COMPLETE`
- **Dependencies**: `nodera-project`, `nodera-desktop::state`
- **Evidence**: `crates/nodera-desktop/tests/external_project_graph_test.rs`
- **Next Action**: Design the schema for the typed Cross-Domain Relationship Layer.

---

### Feature: Cross-Domain Relationship Layer
- **Status**: `PROPOSED`
- **Maturity**: `M0`
- **Quality**: `-`
- **Priority**: `P2`
- **User Value**: Allows a user to write a note discussing an architectural decision and explicitly link to a concrete Rust struct or function symbol without polluting either graph.
- **Current Behavior**: Not yet implemented. Wikilinks currently resolve only to Markdown files in the vault.
- **Sub-features**:
  - Symbol URI syntax (e.g. `[[sym:fungame::Enemy]]` or `@code(fungame, Enemy)`) — `PROPOSED`
  - Cross-domain edge table in SQLite — `PROPOSED`
  - Visual edge styling linking note clusters to code clusters — `PROPOSED`
- **Dependencies**: `nodera-project`, `nodera-markdown`, `nodera-index`
- **Source**: Architecture specification
- **Next Action**: Draft RFC for cross-domain URI link syntax.

---

```
===============================================================================
DOMAIN 05: SEARCH & INDEXING ENGINE
===============================================================================
```

### Feature: Unified Full-Text & Relational Indexing
- **Status**: `COMPLETE`
- **Maturity**: `M4`
- **Quality**: `GOOD`
- **Priority**: `P0`
- **User Value**: Instantaneous, sub-5ms search across massive knowledge vaults with ranking, snippets, and relational querying.
- **Current Behavior**: Combines Tantivy for BM25 inverted full-text search (with field boosts for titles, tags, and headings) with SQLite for structured queries (links, backlinks, tasks, frontmatter). Rayon parallel parsing achieves $>4,300\text{ notes/sec}$ indexing throughput.
- **Sub-features**:
  - BM25 full-text indexing with snippet highlights — `COMPLETE`
  - Relational SQLite schema with foreign key cascades — `COMPLETE`
  - Batch transaction indexing with atomic rollback — `COMPLETE`
  - Multi-threaded Rayon disk/AST parsing pipeline — `COMPLETE`
  - Live search input in sidebar — `COMPLETE`
  - Fuzzy command palette (`Ctrl+P`) — `COMPLETE`
  - Automatic index corruption quarantine and background rebuild — `COMPLETE`
- **Dependencies**: `nodera-index`, `tantivy`, `rusqlite`, `rayon`
- **Evidence**: `crates/nodera-index/src/coordinator.rs`, `benches/vault_benchmarks.rs`
- **Next Action**: Add search query syntax filters (e.g. `tag:rust path:Daily/ is:task`).

---

```
===============================================================================
DOMAIN 06: PDF & DOCUMENT INTELLIGENCE
===============================================================================
```

### Feature: Native PDF-to-Markdown Ingestion
- **Status**: `COMPLETE`
- **Maturity**: `M4`
- **Quality**: `GOOD`
- **Priority**: `P0`
- **User Value**: Transforms static, non-editable text PDFs and academic books into first-class, linkable Markdown notes.
- **Current Behavior**: Pure-Rust extraction pipeline using `lopdf`. Unwraps broken hyphenated words, reconstructs paragraphs, strips running headers/footers and isolated page numbers, identifies chapters/headings, and writes clean Markdown under `Books/`. Monotonic progress streaming with responsive cancellation.
- **Sub-features**:
  - Pure-Rust extraction without external tool dependencies — `COMPLETE`
  - Running header and page number deduplication — `COMPLETE`
  - Hyphenated line unwrapping & paragraph formation — `COMPLETE`
  - 600-page book conversion in $<700\text{ms}$ (~875 pages/sec) — `COMPLETE`
  - Cooperative cancellation via `CancellationToken` — `COMPLETE`
  - Background execution off the UI thread — `COMPLETE`
  - Automatic indexing of generated book notes — `COMPLETE`
- **Dependencies**: `nodera-pdf`, `lopdf`, `tokio`
- **Evidence**: `crates/nodera-pdf/src/service.rs`, `tests/converter_test.rs`, `tests/benchmark_600_test.rs`
- **Next Action**: Improve table extraction heuristics from PDF text stream coordinates.

---

### Feature: PDF Annotation & Highlight Extractor
- **Status**: `COMPLETE`
- **Maturity**: `M3`
- **Quality**: `GOOD`
- **Priority**: `P1`
- **User Value**: Extracts highlights, sticky notes, and comments created in external PDF readers directly into structured literature notes.
- **Current Behavior**: Modal interface (`PdfAnnotationModal`) extracts `/Highlight`, `/Text`, and `/Underline` annotations from PDF dictionaries and formats them into categorized Markdown quote blocks with page citations.
- **Sub-features**:
  - Scanning vault for PDF attachments — `COMPLETE`
  - Extracting highlight color, text, and user comments — `COMPLETE`
  - Outputting formatted markdown literature cards — `COMPLETE`
- **Dependencies**: `nodera-pdf::annotations`, `nodera-desktop`
- **Evidence**: `crates/nodera-desktop/src/components/pdf_annotation_modal.rs`
- **Next Action**: Support direct jump from an extracted annotation back to the source PDF page.

---

```
===============================================================================
DOMAIN 07: EXTERNAL PROJECT INTELLIGENCE
===============================================================================
```

### Feature: Rust Project Discovery & Manifest
- **Status**: `COMPLETE`
- **Maturity**: `M4`
- **Quality**: `GOOD`
- **Priority**: `P0`
- **User Value**: Allows developers to track their real software projects inside Nodera without altering source files.
- **Current Behavior**: Detects single packages and virtual workspaces by inspecting `Cargo.toml`. Resolves package members, identifies source roots (`src/`, `tests/`, `examples/`), and writes `.nodera/project.toml` with deterministic IDs. Skips `target/`, `.git/`, `node_modules/`.
- **Sub-features**:
  - Single package Cargo detection — `COMPLETE`
  - Virtual workspace glob resolution (`crates/*`) — `COMPLETE`
  - Source root and `.rs` discovery with exclusion pruning — `COMPLETE`
  - Global registration in `projects_registry.json` — `COMPLETE`
  - Idempotent `nodera init [path]` CLI command — `COMPLETE`
- **Dependencies**: `nodera-project`, `nodera-cli`
- **Evidence**: `crates/nodera-project/src/discovery.rs`, `crates/nodera-project/tests/project_discovery_test.rs`
- **Next Action**: Add project tag and category customization in `project.toml`.

---

### Feature: AST Parsing Core & Symbol Graph Generation
- **Status**: `COMPLETE`
- **Maturity**: `M4`
- **Quality**: `GOOD`
- **Priority**: `P0`
- **User Value**: Converts raw source code into high-level visual symbol architectures (functions, structs, traits, implementations, call edges).
- **Current Behavior**: `nodera-parser-core` uses `syn` to parse Rust ASTs into a normalized Intermediate Representation (`SourceFile`, `Symbol`, `SymbolReference`). Emits `.nodera/graph/project.json` and populates SQLite `.nodera/index/project.db`.
- **Sub-features**:
  - Normalized AST symbol extraction — `COMPLETE`
  - Call and type reference resolution — `COMPLETE`
  - Stable symbol node ID generation — `COMPLETE`
  - SQLite symbol index for instant lookups — `COMPLETE`
  - Headless `nodera parse <file> [--json]` command — `COMPLETE`
- **Dependencies**: `nodera-parser-core`, `syn`, `nodera-project::graph`
- **Evidence**: `crates/nodera-parser-core/src/languages/rust.rs`, `crates/nodera-project/tests/project_graph_test.rs`
- **Next Action**: Extract doc-comments into symbol metadata inspector cards.

---

### Feature: Incremental Synchronization (`nodera update`)
- **Status**: `COMPLETE`
- **Maturity**: `M4`
- **Quality**: `GOOD`
- **Priority**: `P0`
- **User Value**: Synchronizes code changes in milliseconds without re-parsing untouched files.
- **Current Behavior**: Compares source directory against `.nodera/state/source_state.json` (mtime, size, SHA-256). Identifies Added, Modified, and Deleted files. Reparses only affected files, purges stale symbols, updates graph/index, and auto-rebuilds cleanly if derived files were deleted.
- **Sub-features**:
  - Multi-attribute change detection (mtime, size, SHA-256) — `COMPLETE`
  - Incremental re-indexing of modified files — `COMPLETE`
  - Cascading deletion of removed files and symbols — `COMPLETE`
  - Corrupted or deleted state recovery — `COMPLETE`
  - `nodera update [path]` CLI command — `COMPLETE`
  - `nodera status [path]` inspection summary — `COMPLETE`
- **Dependencies**: `nodera-project::sync`, `nodera-cli`
- **Evidence**: `crates/nodera-project/src/sync.rs`, `crates/nodera-project/tests/project_sync_test.rs`
- **Next Action**: Wire background filesystem watcher for external projects in desktop mode.

---

### Feature: Desktop Project Selection & Graph Visualization
- **Status**: `COMPLETE`
- **Maturity**: `M3`
- **Quality**: `GOOD`
- **Priority**: `P0`
- **User Value**: Allows instant switching between personal knowledge notes and codebase architectures with dedicated visual distinction.
- **Current Behavior**: Prominent **PROJECTS** section in the left sidebar lists registered projects (`fungame`, `nodera`). Selecting a project loads `.nodera/graph/project.json`, switches the 2D graph view to code symbols, and presents an active project banner.
- **Sub-features**:
  - Prominent sidebar Projects list with active indicators — `COMPLETE`
  - Click-to-load project graph — `COMPLETE`
  - Banner indicator with "Return to Vault" trigger — `COMPLETE`
  - Live refresh button for external CLI registrations — `COMPLETE`
- **Dependencies**: `nodera-desktop::sidebar`, `nodera-desktop::graph_view`
- **Evidence**: `crates/nodera-desktop/src/components/sidebar.rs`, `tests/external_project_graph_test.rs`
- **Next Action**: Wire background filesystem watcher for external projects in desktop mode.

---

### Feature: Symbol Context Panel Inspector & Jump-to-Source in Local IDE
- **Status**: `COMPLETE`
- **Maturity**: `M3`
- **Quality**: `GOOD`
- **Priority**: `P1`
- **User Value**: Clicking any code node in the external project graph displays its signature, docstring, enclosing file location, and opens the file in the user's local code editor.
- **Current Behavior**: `ProjectInspector` in the right context drawer dispatches between `ProjectOverviewInspector` (metrics, file counts, symbol breakdown) and `SymbolInspector` (kind badges, visibility, file:line range, syntax-styled signature, docstring, and incoming/outgoing call/trait edge chips). Includes "Open in Editor" (`code -g` or `$EDITOR`) and "Reveal" (Explorer/Finder).
- **Sub-features**:
  - Contextual symbol drawer dispatch on graph node selection — `COMPLETE`
  - Symbol kind badge and visibility indicator — `COMPLETE`
  - Monospace signature display block — `COMPLETE`
  - Docstring comments extractor — `COMPLETE`
  - Cross-symbol edge chips for interactive graph traversal — `COMPLETE`
  - Jump-to-Source in local IDE (`code -g path:line`) — `COMPLETE`
  - Reveal in OS file manager (Windows Explorer, Finder, Nautilus) — `COMPLETE`
  - High-level project metrics overview when no node is selected — `COMPLETE`
- **Dependencies**: `nodera-desktop::components::inspector`, `nodera-project`
- **Evidence**: `crates/nodera-desktop/src/components/inspector.rs`
- **Next Action**: Add syntax highlighting themes to signature inspection box.

---

```
===============================================================================
DOMAIN 08: TASKS & WORK MANAGEMENT
===============================================================================
```

### Feature: Markdown Task Synchronization
- **Status**: `COMPLETE`
- **Maturity**: `M4`
- **Quality**: `GOOD`
- **Priority**: `P0`
- **User Value**: Users can write tasks anywhere in their notes using standard Markdown without maintaining a secondary TODO database.
- **Current Behavior**: Parses `- [ ]` and `- [x]` checkboxes. Provides `toggle_task_at_line` to atomically toggle state directly in source files. Synchronizes state in SQLite `tasks` table.
- **Sub-features**:
  - Standard checkbox syntax parsing — `COMPLETE`
  - In-place atomic file line toggling — `COMPLETE`
  - SQLite task index synchronization — `COMPLETE`
- **Dependencies**: `nodera-markdown::task_parser`, `nodera-index`
- **Evidence**: `crates/nodera-markdown/src/task_parser.rs`
- **Next Action**: Support cancel / cancelled task syntax (`- [-]`).

---

### Feature: Global Task Board (`TaskView`)
- **Status**: `COMPLETE`
- **Maturity**: `M4`
- **Quality**: `GOOD`
- **Priority**: `P0`
- **User Value**: Aggregates action items scattered across hundreds of notes into a single, cohesive command center.
- **Current Behavior**: Workspace route (`TaskView`) lists all tasks across the vault. Supports filtering by pending, completed, or search queries. Checking a box updates both the source Markdown note on disk and the SQLite index instantly. Clicking a task jumps to the exact line in the note.
- **Sub-features**:
  - Vault-wide task aggregation — `COMPLETE`
  - Filter by pending, completed, and text query — `COMPLETE`
  - Interactive two-way checkbox synchronization — `COMPLETE`
  - Jump to source note and line location — `COMPLETE`
- **Dependencies**: `nodera-desktop::components::task_view`, `nodera-index`
- **Evidence**: `crates/nodera-desktop/src/components/task_view.rs`
- **Next Action**: Add grouping by parent folder, tag, or due date.

---

### Feature: Due Dates & Scheduling Syntax
- **Status**: `COMPLETE`
- **Maturity**: `M3`
- **Quality**: `GOOD`
- **Priority**: `P1`
- **User Value**: Allows time-sensitive tasks to integrate directly into daily calendars and agenda projections.
- **Current Behavior**: Parses `📅 YYYY-MM-DD` and `@due(YYYY-MM-DD)` from Markdown task lines via `extract_due_date` in `nodera-markdown`. Persists `due_date` directly into SQLite `tasks` table and enables filtering by due date via `TaskFilter`.
- **Sub-features**:
  - Natural emoji scheduling syntax (`📅 YYYY-MM-DD`) — `COMPLETE`
  - Alternative attribute syntax (`@due(YYYY-MM-DD)`) — `COMPLETE`
  - SQLite task due date persistence — `COMPLETE`
  - TaskFilter query by exact due date — `COMPLETE`
  - Calendar integration: rendering scheduled tasks in Today view — `PLANNED`
  - Overdue task warning indicators — `PLANNED`
- **Dependencies**: `nodera-markdown::task_parser`, `nodera-index::sqlite`
- **Evidence**: `crates/nodera-markdown/src/task_parser.rs`, `crates/nodera-index/src/sqlite.rs`
- **Next Action**: Render scheduled tasks on the Today view calendar date corresponding to their due date.

---

```
===============================================================================
DOMAIN 09: PLANNING & GOALS (PRODUCTIVITY SYSTEM)
===============================================================================
```

### Feature: Multi-Scale Planning Hierarchy
- **Status**: `PROPOSED`
- **Maturity**: `M0`
- **Quality**: `-`
- **Priority**: `P1`
- **User Value**: Bridges daily task execution with long-term aspirations across five connected horizons of time.
- **Current Behavior**: Planning is currently done manually via unstructured notes and templates. No formal entity relationship exists connecting life horizons.
- **Sub-features**:
  - Vision / Multi-Year Horizon notes — `PROPOSED`
  - Annual Themes & Goals — `PROPOSED`
  - Quarterly Milestones / OKRs — `PROPOSED`
  - Monthly Objectives & Commitments — `PROPOSED`
  - Weekly Sprint Planning projection — `PROPOSED`
  - Lineage tracking: Goal $\rightarrow$ Project $\rightarrow$ Milestone $\rightarrow$ Task — `PROPOSED`
- **Dependencies**: `nodera-markdown`, `nodera-index`, Domain Model
- **Source**: Product Research — Obsidian/Compass workflow
- **Next Action**: Define Markdown frontmatter schema for Goal entities (`type: goal`, `horizon: quarter`).

---

### Feature: Quarterly Personal Retreat System
- **Status**: `PROPOSED`
- **Maturity**: `M0`
- **Quality**: `-`
- **Priority**: `P2`
- **User Value**: Provides a structured ritual for knowledge workers to pause every 90 days, review accomplishments, recalibrate priorities, and set quarterly focus.
- **Current Behavior**: Partially supported through ad-hoc user note templates, but lacks dedicated workflow orchestration.
- **Sub-features**:
  - Retreat guided reflection wizard — `PROPOSED`
  - Automated quarterly metrics compilation (tasks done, books read, commits) — `PROPOSED`
  - Milestone renewal & backlog grooming — `PROPOSED`
- **Dependencies**: `Planning Hierarchy`, `Activity Engine`, `Dashboards`
- **Source**: Product Research — Obsidian/Compass workflow
- **Next Action**: Create built-in "Quarterly Retreat" note template with structured questionnaires.

---

```
===============================================================================
DOMAIN 10: ACTIVITY & EVIDENCE ENGINE
===============================================================================
```

### Feature: Universal Activity Model
- **Status**: `COMPLETE`
- **Maturity**: `M3`
- **Quality**: `GOOD`
- **Priority**: `P1`
- **User Value**: Differentiates intentions from actual reality, providing verifiable telemetry on where time and focus were spent.
- **Current Behavior**: Typed `Activity` model in `nodera-index::models` (`id`, `timestamp_secs`, `kind`, `duration_secs`, `source`, `project_id`, `task_id`, `note_path`, `metadata`) persisted into SQLite `activities` table with indexes on timestamp, kind, and project. Supports range querying, filtering, and deletion via `SqliteIndex` and `VaultIndex`.
- **Sub-features**:
  - Typed `Activity` and `ActivityKind` domain entities — `COMPLETE`
  - SQLite `activities` table and indexes (`timestamp_secs`, `kind`, `project_id`) — `COMPLETE`
  - Atomic activity persistence (`record_activity`) — `COMPLETE`
  - Multi-attribute query filtering (`ActivityFilter`) — `COMPLETE`
  - Activity event deletion and bulk clearing — `COMPLETE`
  - VaultIndex delegation interface — `COMPLETE`
  - Collector integrations (Git, Reading, Task completions) — `PLANNED`
- **Dependencies**: `nodera-index::sqlite`, `nodera-index::models`
- **Evidence**: `crates/nodera-index/src/models.rs`, `crates/nodera-index/src/sqlite.rs`
- **Next Action**: Wire task completion events to record `ActivityKind::TaskCompletion` automatically.

---

### Feature: Evidence-Based Habit Evaluation
- **Status**: `RESEARCH_REQUIRED`
- **Maturity**: `M0`
- **Quality**: `-`
- **Priority**: `P1`
- **User Value**: Eliminates fake streak maintenance by verifying habits against real activity evidence (e.g. coding habit confirmed by Git commit, reading habit confirmed by e-book session).
- **Current Behavior**: Habit tracking does not yet exist in Nodera.
- **Sub-features**:
  - Habit definition schema with target metric rules — `PROPOSED`
  - Evidence rule matcher (e.g. `rule: CodingActivity.duration >= 45m`) — `RESEARCH_REQUIRED`
  - Automatic daily streak computation — `PROPOSED`
  - Grace day and streak freeze allowances — `PROPOSED`
  - Manual checkoff fallback for non-digital habits (e.g. Exercise, Meditation) — `PROPOSED`
- **Dependencies**: `Universal Activity Model`, `nodera-index`
- **Source**: Product Research — Obsidian/Compass workflow
- **Next Action**: Research local privacy-preserving evidence collection strategies.

---

```
===============================================================================
DOMAIN 11: JOURNAL & REFLECTION
===============================================================================
```

### Feature: Daily Notes Calendar Workspace (`TodayView`)
- **Status**: `COMPLETE`
- **Maturity**: `M4`
- **Quality**: `GOOD`
- **Priority**: `P0`
- **User Value**: Provides a calm, daily anchor for planning the day, logging thoughts, and reviewing immediate tasks.
- **Current Behavior**: Dedicated route (`TodayView`) renders a full monthly calendar grid with today highlights, daily note existence dots, and selection highlights. Arrow key navigation. Single-click or Enter creates or opens `Daily/YYYY-MM-DD.md` with default frontmatter and daily templates.
- **Sub-features**:
  - Full calendar month grid with keyboard navigation — `COMPLETE`
  - Quick open today’s daily note (`Ctrl+Shift+D`) — `COMPLETE`
  - Go-to-Date modal dialog (`Ctrl+G`) — `COMPLETE`
  - Daily note template auto-application — `COMPLETE`
- **Dependencies**: `nodera-desktop::calendar`, `nodera-desktop::today_view`
- **Evidence**: `crates/nodera-desktop/src/calendar.rs`, `crates/nodera-desktop/src/components/today_view.rs`
- **Next Action**: Embed today's scheduled tasks list directly beneath the calendar grid.

---

### Feature: Cadenced Reviews (Weekly & Monthly)
- **Status**: `PROPOSED`
- **Maturity**: `M0`
- **Quality**: `-`
- **Priority**: `P1`
- **User Value**: Guides the knowledge worker through systematic weekly consolidation (triage inbox, audit tasks, review commitments) and monthly retrospectives.
- **Current Behavior**: Weekly reviews are currently managed manually via the built-in "Weekly Review" note template.
- **Sub-features**:
  - Weekly review checklist wizard — `PROPOSED`
  - Auto-compilation of notes created, tasks completed, and books read during the week — `PROPOSED`
  - Review status indicators on the calendar view — `PROPOSED`
- **Dependencies**: `Universal Activity Model`, `Tasks`, `TodayView`
- **Source**: Product Research — Obsidian/Compass workflow
- **Next Action**: Enhance Weekly Review template with dynamic aggregation queries.

---

```
===============================================================================
DOMAIN 12: READING & LITERATURE
===============================================================================
```

### Feature: Library Bookshelf & Reading Progress
- **Status**: `COMPLETE`
- **Maturity**: `M4`
- **Quality**: `GOOD`
- **Priority**: `P1`
- **User Value**: Manages long-form reading materials, converted PDF books, and reading states in an elegant visual bookshelf.
- **Current Behavior**: `LibraryView` renders high-contrast book cards with thumbnail covers, chapter counts, word metrics, and persistent reading progress bars. Direct "Read" mode switches to distraction-free reading with a floating/docked Table of Contents.
- **Sub-features**:
  - Document grid cards with thumbnail art — `COMPLETE`
  - Visual reading progress track and percentage calculation — `COMPLETE`
  - Distraction-free reading mode with clean typography — `COMPLETE`
  - Table of Contents sidebar extracted from Markdown headings — `COMPLETE`
  - Persistent reading progress in preferences — `COMPLETE`
- **Dependencies**: `nodera-desktop::components::library_view`, `nodera-desktop::components::editor`
- **Evidence**: `crates/nodera-desktop/src/components/library_view.rs`
- **Next Action**: Add reading speed estimator (words per minute) to project remaining chapter time.

---

### Feature: BibTeX & Zotero Citation Picker
- **Status**: `COMPLETE`
- **Maturity**: `M3`
- **Quality**: `GOOD`
- **Priority**: `P1`
- **User Value**: Seamless academic workflow allowing researchers and students to cite papers and literature directly into their notes.
- **Current Behavior**: Parses `.bib` files in the vault into `BibLibrary`. `CitationPickerModal` (`Ctrl+Shift+C`) provides fuzzy search across authors, titles, and keys. Inserting an entry generates standard `@citation_key` references rendered as interactive citation chips in reading mode.
- **Sub-features**:
  - Native BibTeX parser (`BibEntry`) — `COMPLETE`
  - Fuzzy citation picker modal (`Ctrl+Shift+C`) — `COMPLETE`
  - Visual citation chip rendering in preview mode — `COMPLETE`
- **Dependencies**: `nodera-core::bib`, `nodera-desktop::citation_picker_modal`
- **Evidence**: `crates/nodera-core/src/bib.rs`, `crates/nodera-desktop/src/components/citation_picker_modal.rs`
- **Next Action**: Add automatic `.bib` export auto-refresh from local Zotero storage directories.

---

```
===============================================================================
DOMAIN 13: WRITING & PUBLISHING
===============================================================================
```

### Feature: Distraction-Free Long-Form Editor
- **Status**: `COMPLETE`
- **Maturity**: `M3`
- **Quality**: `GOOD`
- **Priority**: `P0`
- **User Value**: A calm, focused writing environment designed for deep creative and technical writing.
- **Current Behavior**: Centered writing canvas bounded to readable character widths (~780px). Statusbar displays live word count, character count, and estimated reading time. Quick toggle between edit and preview modes.
- **Sub-features**:
  - Live document metrics in statusbar — `COMPLETE`
  - Resizable sidebars allow 100% focused editor canvas — `COMPLETE`
  - Full keyboard editing shortcuts — `COMPLETE`
  - Typewriter scrolling mode — `PROPOSED`
  - Focus mode (hide all panels with single hotkey) — `PARTIALLY_WORKING`
- **Dependencies**: `nodera-desktop::components::editor`, `nodera-desktop::components::statusbar`
- **Evidence**: `crates/nodera-desktop/src/components/editor.rs`
- **Next Action**: Implement pure Zen focus mode shortcut (`F11` / `Ctrl+Shift+F`) hiding all chrome.

---

### Feature: Multi-Chapter Manuscript Compiler
- **Status**: `PROPOSED`
- **Maturity**: `M0`
- **Quality**: `-`
- **Priority**: `P2`
- **User Value**: Allows authors, technical writers, and researchers to assemble multi-file draft notes into a single manuscript for export.
- **Current Behavior**: Notes currently exist as individual discrete files.
- **Sub-features**:
  - Manuscript project definition (ordered note list) — `PROPOSED`
  - Single-file unified Markdown compilation — `PROPOSED`
  - Export to standalone formatted PDF / HTML — `PLANNED`
- **Dependencies**: `nodera-core`, `nodera-markdown`
- **Source**: Product Roadmap
- **Next Action**: Define `manuscript.toml` configuration format.

---

```
===============================================================================
DOMAIN 14: DASHBOARDS & PROJECTIONS
===============================================================================
```

### Feature: Projection-Oriented Dashboard Architecture
- **Status**: `COMPLETE`
- **Maturity**: `M3`
- **Quality**: `GOOD`
- **Priority**: `P0`
- **User Value**: Eliminates fractured data silos by rendering the same underlying information across specialized visual dashboards.
- **Current Behavior**: Dashboards exist as clean projections over SQLite and Tantivy data:
  - `TodayView`: Projects daily notes, calendar dates, and daily reflection.
  - `TaskView`: Projects all checkboxes across all notes into an actionable board.
  - `LibraryView`: Projects books, reading progress, and document chapters.
  - `ReviewQueueView`: Projects unlinked notes and rough thoughts for triage.
  - `GraphView`: Projects relational wikilinks and code symbols into spatial clusters.
- **Sub-features**:
  - Today Calendar Dashboard — `COMPLETE`
  - Global Task Board Dashboard — `COMPLETE`
  - Bookshelf & Reading Dashboard — `COMPLETE`
  - Knowledge Triage Dashboard — `COMPLETE`
  - Code Architecture Dashboard — `COMPLETE`
  - Habits & Streaks Dashboard Projection — `PROPOSED`
  - Weekly Review Dashboard Projection — `PROPOSED`
- **Dependencies**: `nodera-desktop::state`, `nodera-index`
- **Evidence**: `crates/nodera-desktop/src/components/`
- **Next Action**: Create modular dashboard widget containers for customized user home screens.

---

```
===============================================================================
DOMAIN 15: CONTEXT ENGINE & LOCAL AI
===============================================================================
```

### Feature: Local Context Assembly Engine
- **Status**: `PARTIALLY_WORKING`
- **Maturity**: `M2`
- **Quality**: `NEEDS_ARCHITECTURAL_REVIEW`
- **Priority**: `P1`
- **User Value**: Gathers all relevant context surrounding an active thought or code task (backlinks, shared tags, AST symbols, recent edits) for instant human recall or AI assistance.
- **Current Behavior**: Note Inspector calculates related notes via TF-IDF content overlap, shared tags, and link connectivity. `ProjectIndex` supports symbol reference querying. However, a unified context assembler combining note content, tasks, and code symbols into a coherent prompt/context window is not yet formalized.
- **Sub-features**:
  - Related notes scoring algorithm — `COMPLETE`
  - Outgoing and incoming backlinks extraction — `COMPLETE`
  - AST symbol reference lookups — `COMPLETE`
  - Unified Context Assembler API — `PLANNED`
- **Dependencies**: `nodera-index`, `nodera-project`
- **Evidence**: `crates/nodera-index/src/similarity.rs`, `crates/nodera-desktop/src/components/inspector.rs`
- **Next Action**: Create `nodera-context` module to assemble structured context windows.

---

### Feature: Embedded Local LLM Assistant
- **Status**: `RESEARCH_REQUIRED`
- **Maturity**: `M0`
- **Quality**: `-`
- **Priority**: `P3`
- **User Value**: Provides private, on-device AI synthesis, summarization, and query assistance without sending sensitive notes or proprietary code to cloud servers.
- **Current Behavior**: Nodera explicitly does not include mandatory cloud AI. Local inference via `llama.cpp` bindings or Ollama integration is under research.
- **Sub-features**:
  - Local inference backend interface — `RESEARCH_REQUIRED`
  - RAG search over Tantivy full-text index — `PROPOSED`
  - Note auto-summarization and link suggestion — `PROPOSED`
- **Dependencies**: `Context Assembly Engine`, Local model runtime
- **Source**: Product Principles (Zero mandatory cloud lock-in)
- **Next Action**: Evaluate CPU/NPU performance of quantized local models (e.g. Llama 3.2 1B/3B) in pure Rust.

---

```
===============================================================================
DOMAIN 16: HEADLESS CLI & SYSTEM INTEGRATION
===============================================================================
```

### Feature: Headless CLI Engine (`nodera-cli`)
- **Status**: `COMPLETE`
- **Maturity**: `M4`
- **Quality**: `GOOD`
- **Priority**: `P0`
- **User Value**: Enables automated workflows, shell scripting, terminal-first users, and external tooling integration without opening the desktop GUI.
- **Current Behavior**: High-performance headless CLI binary (`nodera.exe`) installed in `~/.cargo/bin`. Supports project initialization, synchronization, status reporting, AST parsing, vault searches, atomic note creation, and deep link execution.
- **Sub-features**:
  - `nodera init [path]` (project setup) — `COMPLETE`
  - `nodera update [path]` (incremental sync) — `COMPLETE`
  - `nodera status [path]` (metadata & change inspection) — `COMPLETE`
  - `nodera parse <file> [--json]` (AST parsing) — `COMPLETE`
  - `nodera search <vault> "<q>"` (headless BM25 search) — `COMPLETE`
  - `nodera index <vault>` (headless index rebuild) — `COMPLETE`
  - `nodera stats <vault>` (vault health metrics) — `COMPLETE`
  - `nodera create <vault> "<title>"` (atomic note creation) — `COMPLETE`
  - `nodera uri "<nodera://...>"` (deep link dispatch) — `COMPLETE`
  - `nodera register-protocol` / `unregister-protocol` — `COMPLETE`
- **Dependencies**: `nodera-cli`, `nodera-project`, `nodera-core`, `nodera-index`
- **Evidence**: `crates/nodera-cli/src/main.rs`, `crates/nodera-cli/tests/`
- **Next Action**: Add shell completion generators (`bash`, `zsh`, `fish`, `powershell`).

---

### Feature: System Deep Link Protocol (`nodera://`)
- **Status**: `COMPLETE`
- **Maturity**: `M4`
- **Quality**: `GOOD`
- **Priority**: `P1`
- **User Value**: Allows external tools, browser bookmarks, terminal scripts, and calendar apps to deep-link directly into specific notes or projects inside Nodera.
- **Current Behavior**: Registers `nodera://` protocol handler in Windows registry (HKCU). CLI dispatches incoming URLs (`nodera://open?vault=...&note=...`) directly to active desktop instances or opens target headlessly.
- **Sub-features**:
  - Windows registry protocol installer/uninstaller — `COMPLETE`
  - URL route parser (`open`, `create`, `search`) — `COMPLETE`
  - Inter-process dispatch to running desktop instance — `WORKING`
- **Dependencies**: `nodera-core::protocol`, `nodera-cli`
- **Evidence**: `crates/nodera-core/src/protocol.rs`, `crates/nodera-cli/src/main.rs`
- **Next Action**: Register desktop protocol handlers for Linux (`.desktop` mime) and macOS (`Info.plist`).

---

```
===============================================================================
DOMAIN 17: PERFORMANCE & BENCHMARKING
===============================================================================
```

### Feature: Empirical Benchmark Suite & Algorithmic Scalability
- **Status**: `COMPLETE`
- **Maturity**: `M4`
- **Quality**: `GOOD`
- **Priority**: `P0`
- **User Value**: Guarantees that Nodera never slows down as personal vaults grow to tens of thousands of notes and large codebases.
- **Current Behavior**: Verified release-mode benchmarks in `benches/vault_benchmarks.rs` testing 1,000 and 10,000 note datasets across 12 distinct operations. Achieved strict linear $O(N)$ or $O(N \log N)$ complexity:
  - 10K notes parallel index rebuild: **$2.30\text{ s}$** ($>4,300\text{ notes/s}$)
  - 10K notes cold vault open: **$3.18\text{ s}$**
  - 10K graph layout generation: **$42.91\text{ ms}$**
  - 10K graph simulation tick: **$22.10\text{ ms}$** ($>45\text{ FPS}$)
  - 10K vault link health audit: **$59.77\text{ ms}$**
  - 10K search latency: **$2.3\text{ ms}$**
  - 600-page PDF conversion: **$685\text{ ms}$** (~875 pages/sec)
- **Sub-features**:
  - Rayon bounded parallel parser pipeline — `COMPLETE`
  - SQLite single-transaction batch indexing — `COMPLETE`
  - Barnes-Hut spatial decomposition QuadTree — `COMPLETE`
  - $O(1)$ bidirectional backlink index — `COMPLETE`
  - Strict zero unsafe blocks and zero SIMD requirement — `COMPLETE`
- **Dependencies**: `nodera-core`, `nodera-markdown`, `nodera-index`, `nodera-pdf`
- **Evidence**: `benches/vault_benchmarks.rs`, `docs/performance.md`
- **Next Action**: Add automated performance regression test in CI pipelines.

```
===============================================================================
DOMAIN 18: APPEARANCE & THEME SYSTEM
===============================================================================
```

### 18.1 Theme Architecture & Semantic Design Tokens
- **Status**: `COMPLETE`
- **Maturity**: M4 — Production Quality
- **Quality**: `GOOD`
- **Description**: Centralized design token model that completely decouples visual themes from component rendering logic. Defines strongly typed color tokens (`Color`, `Theme`, `ThemeId`) encompassing core surfaces, elevated layers, 1px hairline borders, text hierarchy, primary & secondary accents, semantic state containers, knowledge graph nodes/edges, and editor scrollbars. All components resolve colors via dynamic CSS custom properties (`var(--bg-app)`, `var(--text-primary)`, `var(--accent)`, etc.) and typed Rust token methods.
- **Capabilities**:
  - Strongly typed `Color` struct with WCAG 2.1 relative luminance and contrast ratio calculations.
  - Strongly typed `Theme` token struct with `const fn` constructors for instant zero-allocation resolution.
  - Stable kebab-case theme identity (`nodera-dark`, `nodera-light`, `midnight`, `nord`, `dracula`, `solarized`).
  - Lossy fallback deserializer ensuring unrecognized, malformed, or legacy theme strings safely default to `NoderaDark` without application panic.
- **Dependencies**: `nodera-desktop::theme`
- **Evidence**: `crates/nodera-desktop/src/theme.rs`, unit tests (`theme::tests`), `docs/design-system.md`

### 18.2 Built-in Themes (6 Calibrated Palettes)
- **Status**: `COMPLETE`
- **Maturity**: M4 — Production Quality
- **Quality**: `GOOD`
- **Description**: Six built-in themes covering diverse ambient workspace environments:
  1. **Nodera Dark**: Reference design language with deep slate void canvas (`#0B0F14`), cobalt interactive focus (`#6680FF`), and relational violet (`#9A4BFF`).
  2. **Nodera Light**: Clean, readable light workspace with high-contrast prose (`#171C23`), soft elevated cards (`#F8FAFC`), and crisp borders.
  3. **Midnight**: Ultra-deep OLED-friendly dark workspace (`#020408`) with electric sapphire accent (`#38BDF8`) and high surface depth separation.
  4. **Nord**: Cool, muted arctic developer palette (`#2E3440`) with frost cyan interactive accent (`#88C0D0`) and aurora purple (`#B48EAD`).
  5. **Dracula**: High-contrast developer palette with rich dark canvas (`#21222C`/`#282A36`), vibrant purple accent (`#BD93F9`), and pink relational links (`#FF79C6`).
  6. **Solarized**: Low-contrast warm reading palette (`#FDF6E3`) engineered for eye comfort during marathon research and writing sessions.
- **Accessibility**: All themes pass automated WCAG 2.1 contrast ratio assertions ($\ge 4.5:1$ for body text on app and surface layers).
- **Evidence**: `crates/nodera-desktop/src/theme.rs`, `theme::tests::test_contrast_ratios`

### 18.3 Settings Appearance Theme UI & Token Preview Cards
- **Status**: `COMPLETE`
- **Maturity**: M4 — Production Quality
- **Quality**: `GOOD`
- **Description**: Interactive theme selection interface situated in `Settings -> Appearance -> Theme Palette`. Renders a responsive two-column grid displaying all 6 themes with selection state indicators, active badges, and live miniature preview cards constructed directly from each theme's own tokens (header, prose, hairline divider, action button, and wikilink badge). Clicking any card immediately applies the theme to the entire desktop window and persists the preference.
- **Evidence**: `crates/nodera-desktop/src/components/settings_modal.rs`

### 18.4 Theme Persistence & Lossless Fallback
- **Status**: `COMPLETE`
- **Maturity**: M4 — Production Quality
- **Quality**: `GOOD`
- **Description**: Persists the user's active theme across application restarts in `preferences.json`. Supports quick toggling via dropdown button (`Ctrl` shortcut compatible) and gracefully restores user choice on launch.
- **Evidence**: `crates/nodera-desktop/src/state.rs::AppPreferences`, `theme::tests::test_preferences_persistence_roundtrip`

---

## 7. Major Product Gaps & Opportunities

### 1. Planning & Goal Lineage
- **Gap**: Nodera currently provides outstanding low-level execution tools (daily notes, markdown tasks) and high-level knowledge representation (graphs, libraries), but lacks the **middle layer** connecting Long-Term Goals $\rightarrow$ Quarterly Milestones $\rightarrow$ Weekly Projects $\rightarrow$ Daily Tasks.
- **Opportunity**: Implement a lightweight Markdown frontmatter goal convention and introduce a Goal Progress Dashboard widget that aggregates task completion states automatically.

### 2. Activity & Evidence Stream
- **Gap**: There is currently no historical telemetry recording when work actually occurred. A completed task loses its completion timestamp; coding sessions are not logged.
- **Opportunity**: Introduce a lightweight SQLite `activities` event stream that logs timestamped events (git commits, note edit durations, reading minutes, task completions) to power verifiable habits and retrospective reviews.

### 3. Cross-Domain Relationship Links
- **Gap**: The Markdown knowledge graph and External Project code graph are strictly separated (which is correct), but users cannot yet create a formal link between a design note and a Rust AST symbol.
- **Opportunity**: Design a typed URI scheme (e.g. `[[sym:fungame::Enemy]]`) that renders as a distinct code chip in notes and highlights connected symbol nodes in the project graph without merging graph databases.

### 4. Symbol Inspector & Jump-to-Source in Desktop
- **Gap**: Selecting an external project in Nodera Desktop renders the code architecture graph, but clicking an individual symbol node does not yet display its docstrings, signature, or open the file in the user’s local code editor.
- **Opportunity**: Implement a Symbol Inspector in the right context drawer and add an "Open in Editor" button invoking the user's default editor (VS Code, RustRover, Zed) at the exact line and column.

---

## 8. Catalog Maintenance & Governance Rules

To ensure this Product Catalog remains the authoritative single source of truth:

1. **Before Claiming Implementation Status**: Inspect actual repository source code and tests. Never base status on documentation, plans, or conversational memory.
2. **When Adding a Feature**:
   - Add feature and its sub-features to Section 6.
   - Assign primary status using the controlled vocabulary.
   - Assign quality flags and maturity level.
   - Register the feature in the Master Index Table (Section 5).
   - Add an entry to the Changelog (Section 9).
3. **When Modifying Architecture**: Update the Product Architecture Map (Section 3).
4. **When Deferring or Rejecting a Feature**: Mark as `DEFERRED` or `OUT_OF_SCOPE` with a clear explanation rather than deleting the historical record.

---

## 9. Product Catalog Changelog

### 2026-09-24 — Controlled Implementation Slice: Symbol Inspector, Task Scheduling, Activity Engine
- **Task Scheduling Syntax**:
  - Implemented `extract_due_date` in `nodera-markdown::task_parser` supporting both natural emoji syntax (`📅 YYYY-MM-DD`) and attribute syntax (`@due(YYYY-MM-DD)`).
  - Populated `due_date` into `ParsedTask` and SQLite `tasks` table across single and batch indexing pipelines.
  - Added due date filtering support to `TaskFilter` and `query_tasks`.
- **Universal Activity Model & SQLite Persistence**:
  - Implemented `Activity`, `ActivityKind`, and `ActivityFilter` domain entities in `nodera-index::models`.
  - Created SQLite `activities` table with indices on `timestamp_secs`, `kind`, and `project_id`.
  - Implemented `record_activity`, `query_activities`, `delete_activity`, and `clear_activities` on `SqliteIndex` and `VaultIndex`.
  - Added comprehensive unit tests validating event recording, multi-attribute querying, and range filtering.
- **External Project Symbol Inspector & Jump-to-Source**:
  - Implemented `ProjectInspector` in `nodera-desktop::components::inspector` rendering when viewing external projects in Graph mode.
  - Implemented `SymbolInspector` displaying symbol kind badges (color-coded per AST kind), visibility, file:line location, syntax-styled signature, docstrings, and cross-symbol reference chips (calls, called-by, implements).
  - Added "Open in Editor" action launching local code editor (`code -g path:line` or `$EDITOR`) and "Reveal" action in OS file manager.
  - Implemented `ProjectOverviewInspector` displaying project metrics, file counts, and symbol inventory breakdown when no node is selected.
  - Embedded collapsible Graph Controls drawer for physics tuning while in project mode.
- **Zero Regressions & Quality Assurance**: All unit tests in `nodera-markdown` and `nodera-index` passed with 0 warnings.

### 2026-09-24 — Catalog Inception & Productivity System Integration
- **Established**: First-class authoritative `docs/product-catalog.md` artifact.
- **Catalogued Existing Domains**: Audited and catalogued 18 existing product domains across `nodera-core`, `nodera-markdown`, `nodera-index`, `nodera-pdf`, `nodera-parser-core`, `nodera-project`, `nodera-cli`, and `nodera-desktop`.
- **Integrated External Project Intelligence**: Documented Rust/Cargo project discovery, AST parsing, derived graph/index schemas, incremental `nodera update`, and desktop graph domain separation.
- **Incorporated Productivity System Research**: Translated research from Obsidian/Compass workflows into native Nodera candidate capabilities:
  - Multi-Scale Planning Hierarchy (Horizons, Goals, Milestones)
  - Quarterly Personal Retreat Workflow
  - Universal Activity Model (`Plan != Activity`)
  - Evidence-Based Habit Evaluation Engine
  - Cadenced Reviews (Weekly & Monthly Retrospectives)
  - Unified Domain Model with Multiple Projections
- **Enforced Architectural Guarantees**: Documented strict separation between Markdown Knowledge Graph and External Project Graph, and defined roadmap for the future Cross-Domain Relationship Layer.
