<p align="center">
  <img src="assets/branding/nodera-logo.png" alt="Nodera Logo" width="120" height="120" />
</p>

<h1 align="center">Nodera</h1>

<p align="center">
  <strong>A high-speed, local-first knowledge workspace and Markdown engine built in pure Rust.</strong>
</p>

<p align="center">
  <a href="https://github.com/l1khith/Nodera/actions"><img src="https://img.shields.io/badge/build-passing-brightgreen?style=flat-square" alt="Build Status" /></a>
  <a href="https://www.rust-lang.org/"><img src="https://img.shields.io/badge/rust-1.80%2B-orange?style=flat-square&logo=rust" alt="Rust 1.80+" /></a>
  <a href="#test-suite--benchmarks"><img src="https://img.shields.io/badge/tests-90%20passed-blue?style=flat-square" alt="Tests" /></a>
  <a href="#license"><img src="https://img.shields.io/badge/license-AGPL--3.0-blueviolet?style=flat-square" alt="License" /></a>
  <a href="https://dioxuslabs.com/"><img src="https://img.shields.io/badge/GUI-Dioxus%200.6-00D1B2?style=flat-square" alt="GUI Dioxus" /></a>
  <a href="#architecture"><img src="https://img.shields.io/badge/storage-local--first-success?style=flat-square" alt="Local-First" /></a>
</p>

<p align="center">
  <a href="#overview">Overview</a> •
  <a href="#key-features">Key Features</a> •
  <a href="#interactive-2d-knowledge-graph">Knowledge Graph</a> •
  <a href="#architecture">Architecture</a> •
  <a href="#benchmarks">Benchmarks</a> •
  <a href="#getting-started">Getting Started</a> •
  <a href="#keyboard-shortcuts">Shortcuts</a> •
  <a href="#documentation">Documentation</a>
</p>

---

## Overview

**Nodera** is a native desktop knowledge workspace designed for speed, deep focus, and longevity. Built entirely in Rust without heavy web wrappers (no Electron), Nodera provides instant note-taking, bidirectional Wikilink graph analysis, lightning-fast full-text search, and automated PDF-to-Markdown digestion.

### Core Philosophy

1. **Plain Markdown as the Source of Truth**: Every note is stored as a standard `.md` file directly on your filesystem. No proprietary databases, no vendor lock-in, and zero risk of data loss. Deleting Nodera leaves your entire knowledge base intact and completely accessible.
2. **Rebuildable Derived State**: Fast indexes (SQLite metadata and Tantivy full-text index) are treated as transient caches. If indexes are corrupted or removed, Nodera self-heals and reconstructs them instantly from your raw Markdown files.
3. **Local-First & Offline**: Everything runs locally on your machine with zero cloud dependencies, zero accounts, and zero telemetry.
4. **Mechanical Sympathy**: Leveraging Rust's memory safety, Rayon multi-threading, and Tokio async runtime to achieve sub-millisecond search latencies and >1,000 notes/sec index throughput.

---

## Key Features

### 📝 Markdown-Native Workspace
- **Distraction-Free Editor**: Responsive editing with live word count, line/column tracking, and unsaved changes indicators.
- **Reading & Edit Modes**: Seamless toggle between raw Markdown and styled preview with formatted tables, task lists, and blockquotes.
- **Dynamic Table of Contents**: Automatically parsed from document headings (`# H1` through `###### H6`) with click-to-jump navigation.
- **Hierarchical Vault Explorer**: Multi-level folder organization with file creation, renaming, deletion, and real-time filesystem watcher synchronization.

### 🕸️ Interactive 2D Knowledge Graph
- **Global 2D Graph View**: Force-directed physics canvas visualizing interconnected concepts across your entire vault.
- **Local Neighborhood Graph**: Dedicated 1-hop interactive sub-graph inside the context inspector centered around the active note.
- **Obsidian-Style Subtle Dotted Canvas**: Uniform low-contrast grid background that scales and pans in hardware-accelerated lockstep with nodes and edges.
- **Dynamic Dot Compensation**: Dot radius dynamically compensates for zoom level (`(0.85 / zoom).clamp(0.4, 1.2)`), ensuring dots stay crisp without visual checkerboard glare.
- **Physics Engine**: Coulomb repulsion, Hooke spring attraction, and viewport center gravity with 75-step synchronous pre-warming.
- **Floating Controls Toolbar**: Frosted glass controls for Zoom In, Zoom Level (`%`), Zoom Out, Center Reset, and Physics Re-layout.
- **Live Search & Highlighting**: Filter nodes in real time; hovering any node highlights direct connections and dims distant notes.

### 🔗 Wikilinks & Backlinks Engine
- **Bidirectional Linking**: Instant resolution of `[[Target Note]]` and aliased links `[[Target Note|Custom Display Text]]`.
- **Automatic Backlinks Detection**: Live indexation of incoming references with context preview snippets.
- **Broken Link Detection**: Visual badges distinguishing resolved notes from dangling references.

### ⚡ Full-Text Search (Tantivy & SQLite)
- **Sub-3ms Search Latency**: Tantivy-powered search engine indexing titles, headings, body text, and tags.
- **Dual-Index Architecture**: SQLite manages relational metadata, link edges, and task states; Tantivy handles lexical full-text scoring.
- **Self-Healing Recovery**: Automatic detection and reconstruction from damaged or corrupted index files.

### ✅ Global Tasks Dashboard
- **Universal Task Aggregation**: Automatically extracts all `- [ ]` and `- [x]` Markdown checkboxes across every note in your vault.
- **Filter Views**: Switch effortlessly between **All**, **To Do**, and **Completed** tasks.
- **Interactive Toggling**: Checking a task updates the source `.md` file atomically while preserving formatting.
- **Origin Links**: Click any task badge to navigate directly to the note and line where it was defined.

### 📄 High-Throughput PDF-to-Markdown Ingestion
- **Streaming Pipeline**: Digestion throughput of **~875 pages/second** (600-page book converted in ~685ms).
- **Intelligent Structure Recognition**: Automatic detection of chapters, headings, repeated running headers, and page numbers.
- **Hyphenation & Paragraph Reconstruction**: Heuristic-based repair of line-wrap hyphenations and fragmented paragraphs into flowing Markdown.
- **Monotonic Progress Events**: Live progress bar with cancellation support and collision-free vault import.

### 🎨 Pure-Rust Vector Asset & Design System
- **Official Nodera Logo**: High-DPI folded-book `N` mark embedded across window icons, Windows taskbar, and PE executable resources.
- **Zero Hardcoded Emojis**: 30+ dedicated pure-Rust SVG vector icons adapting automatically across light and dark themes.
- **Centralized Design Tokens**: Strict semantic color tokens ([`docs/design-system.md`](docs/design-system.md)) with dark theme base (`#0D0F14`) and primary violet-indigo accents (`#5B6CFF`, `#9A4BFF`).

---

## Interactive 2D Knowledge Graph

```text
Nodera Graph Architecture
│
├── Global 2D Graph (Dedicated Canvas View)
│   ├── Physics: Coulomb Repulsion (1/r²) + Hooke Spring (k=0.045) + Gravity (k=0.012)
│   ├── Canvas: Infinite SVG with <pattern id="graph-dot-grid">
│   ├── Navigation: Smooth Pan (drag), Zoom (wheel 0.2x–3.5x), Drag & Drop Nodes
│   └── Controls: Frosted top-right pill (Zoom In / Out, Reset, Re-simulate)
│
└── Local 2D Graph (Context Inspector Panel)
    ├── Breadth-First Expansion (1-hop radius around active note)
    ├── Violet Active Node Highlight (#9A4BFF)
    └── Quick Jump: Click any connected neighbor to switch active note
```

---

## Architecture

```text
nodera/
├── crates/
│   ├── nodera-core/         # Vault filesystem operations, path resolution, models
│   ├── nodera-markdown/     # AST parsing, Wikilinks, backlinks, tasks, graph models
│   ├── nodera-index/        # SQLite metadata index + Tantivy full-text search
│   ├── nodera-pdf/          # High-speed PDF text extraction & Markdown converter
│   └── nodera-desktop/      # Dioxus 0.6 Desktop UI, state store, graph view, theme
├── assets/
│   └── branding/            # Canonical logos, multi-res .ico, .icns, and PNG assets
└── docs/                    # Architectural decisions, design system, specifications
```

### Module Responsibilities

| Crate | Responsibility | Primary Technologies |
| :--- | :--- | :--- |
| **`nodera-core`** | Vault creation, safe relative path resolution, atomic writes | `thiserror`, `uuid`, `serde` |
| **`nodera-markdown`** | Parser, Wikilinks, backlinks, task parser, graph engine | `pulldown-cmark`, `serde_yaml` |
| **`nodera-index`** | Dual-tier indexer, SQLite relations, Tantivy FTS, recovery | `rusqlite`, `tantivy` |
| **`nodera-pdf`** | Streaming extraction, heading detection, paragraph repair | `lopdf`, `regex`, `tokio` |
| **`nodera-desktop`** | Reactive desktop UI, 2D force-directed canvas, icons, theme | `dioxus 0.6`, `tao`, `wry`, `winres` |

---

## Benchmarks

Tested on standard development workstation (AMD Ryzen 9 / Windows 11):

| Benchmark Scenario | Dataset / Workload | Metric / Latency | Result |
| :--- | :--- | :--- | :--- |
| **PDF Ingestion Throughput** | 600-page academic book (BT001S260218049) | **~875.7 pages/sec** (685ms total) | **PASS** |
| **Vault Cold Indexing** | 1,000 hierarchical Markdown notes | **~1,100 notes/sec** (908ms total) | **PASS** |
| **Full-Text Search Latency** | 1,000 indexed notes (Tantivy query) | **2.38ms** (target: < 50ms) | **PASS** |
| **Index Self-Healing Recovery** | Corrupted SQLite & Tantivy headers | Auto-rebuilt & verified in **230ms** | **PASS** |
| **Startup Memory Footprint** | Fresh desktop launch | **~24MB RSS** | **PASS** |

---

## Getting Started

### Prerequisites
- [Rust 1.80+](https://rustup.rs/) toolchain installed.
- Windows, macOS, or Linux.

### Installation & Build

```bash
# 1. Clone the repository
git clone https://github.com/l1khith/Nodera.git
cd Nodera

# 2. Build the desktop release binary
cargo build --workspace --release

# 3. Launch Nodera
./target/release/nodera-desktop
```

### Running Tests & Quality Verification

```bash
# Run the complete test suite (90 unit, integration, and stress tests)
cargo test --workspace

# Run Clippy lints (zero-warning policy)
cargo clippy --workspace --all-targets -- -D warnings

# Check code formatting
cargo fmt --all -- --check
```

---

## Keyboard Shortcuts

| Shortcut | Action | Scope |
| :--- | :--- | :--- |
| <kbd>Ctrl</kbd> + <kbd>N</kbd> | Create a new note | Global |
| <kbd>Ctrl</kbd> + <kbd>S</kbd> | Save current note | Editor |
| <kbd>Ctrl</kbd> + <kbd>P</kbd> | Open Command Palette / Quick Search | Global |
| <kbd>Ctrl</kbd> + <kbd>E</kbd> | Toggle Reading / Edit Mode | Editor |
| <kbd>Ctrl</kbd> + <kbd>G</kbd> | Toggle Right Inspector Panel (Local Graph) | Global |
| <kbd>Ctrl</kbd> + <kbd>I</kbd> | Import PDF as Markdown | Global |
| <kbd>Ctrl</kbd> + <kbd>,</kbd> | Open Settings & Vault Preferences | Global |
| <kbd>Escape</kbd> | Close Modals / Command Palette | Dialogs |

---

## Documentation

Comprehensive architecture, design decisions, and specifications are located in [`docs/`](docs/):

- [`docs/design-system.md`](docs/design-system.md): Canonical color tokens, typography, and UI guidelines.
- [`docs/architecture.md`](docs/architecture.md): Architectural layers, concurrency model, and data flow.
- [`docs/database.md`](docs/database.md): SQLite schema, relational indexing, and Tantivy document schemas.
- [`docs/pdf-import.md`](docs/pdf-import.md): PDF processing pipeline, paragraph reconstruction heuristics.
- [`docs/decisions/`](docs/decisions/): Architecture Decision Records (ADRs).

---

## Contributing

Contributions, bug reports, and feature requests are welcome!

1. Fork the repository.
2. Create your feature branch (`git checkout -b feature/amazing-feature`).
3. Ensure formatting and clippy pass (`cargo fmt --all && cargo clippy --workspace --all-targets -- -D warnings`).
4. Ensure tests pass (`cargo test --workspace`).
5. Commit your changes (`git commit -m 'Add amazing feature'`).
6. Push to the branch (`git push origin feature/amazing-feature`).
7. Open a Pull Request.

---

## License

Nodera is free and open-source software licensed under the **GNU Affero General Public License v3.0** ([AGPL-3.0](LICENSE)).

See the [LICENSE](LICENSE) file for the full license text.

