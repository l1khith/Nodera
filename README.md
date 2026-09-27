<p align="center">
  <img src="assets/branding/nodera-logo.png" alt="Nodera Logo" width="120" height="120" />
</p>

<h1 align="center">Nodera</h1>

<p align="center">
  <strong>Rust-native local-first knowledge workspace that turns documents and notes into a searchable, interconnected knowledge base.</strong>
</p>

<p align="center">
  <a href="https://github.com/l1khith/Nodera/actions"><img src="https://img.shields.io/badge/build-passing-brightgreen?style=flat-square" alt="Build Status" /></a>
  <a href="https://www.rust-lang.org/"><img src="https://img.shields.io/badge/rust-1.80%2B-orange?style=flat-square&logo=rust" alt="Rust 1.80+" /></a>
  <a href="#test-suite--quality-verification"><img src="https://img.shields.io/badge/tests-100%25%20passing-blue?style=flat-square" alt="Tests" /></a>
  <a href="#license"><img src="https://img.shields.io/badge/license-AGPL--3.0-blueviolet?style=flat-square" alt="License" /></a>
  <a href="https://dioxuslabs.com/"><img src="https://img.shields.io/badge/GUI-Dioxus%200.6-00D1B2?style=flat-square" alt="GUI Dioxus" /></a>
  <a href="#architecture"><img src="https://img.shields.io/badge/storage-local--first-success?style=flat-square" alt="Local-First" /></a>
</p>

<p align="center">
  <a href="#why-nodera">Why Nodera?</a> •
  <a href="#architecture">Architecture</a> •
  <a href="#current-capabilities-vs-experimental">Capabilities</a> •
  <a href="#measurable-engineering-benchmarks">Benchmarks</a> •
  <a href="#why-rust">Why Rust?</a> •
  <a href="#crate-boundaries">Crates</a> •
  <a href="#architecture-decision-records-adrs">ADRs</a> •
  <a href="#known-limitations">Limitations</a> •
  <a href="#getting-started">Getting Started</a>
</p>

---

## Why Nodera?

Most personal knowledge tools are built on one of two extremes:
1. **Electron-wrapped web apps** with heavy V8/Chromium overhead (300MB+ RAM idle), sluggish startup times, and complex external runtimes.
2. **Proprietary cloud platforms** that trap knowledge in opaque remote databases with vendor lock-in and offline penalties.

**Nodera** is engineered from the ground up as a **Rust-native, local-first knowledge workspace**:
- **Plain Markdown as Source of Truth**: Notes reside as plain `.md` files directly on your filesystem. Zero proprietary silos, zero risk of data loss. Deleting Nodera leaves every note 100% intact.
- **Rebuildable Derived State**: Fast relational metadata (SQLite) and full-text search (Tantivy) are treated as disposable, self-healing caches.
- **Pure-Rust Ingestion Pipeline**: Ingest large PDF books and technical documentation directly into clean, structured Markdown at **~875 pages/second** with zero external Python or C dependencies.
- **Mechanical Sympathy**: Sub-3ms search queries, ~34MB baseline RSS footprint, and multi-threaded background workers powered by Tokio and Rayon.

---

## Architecture

Nodera decouples user-facing interactions from high-throughput document processing and persistent indexing:

```text
             ┌─────────────────────────┐
             │     PDF / Markdown      │
             │   (Filesystem Source)   │
             └────────────┬────────────┘
                          │
                          ▼
             ┌─────────────────────────┐
             │     Rust Ingestion      │
             │  Streaming PDF (lopdf)  │
             │   + Markdown Parser     │
             └────────────┬────────────┘
                          │
                          ▼
             ┌─────────────────────────┐
             │     Knowledge Model     │
             │ AST • Wikilinks • Tasks │
             └────────────┬────────────┘
                          │
              ┌───────────┴───────────┐
              ▼                       ▼
       ┌─────────────┐         ┌─────────────┐
       │   SQLite    │         │   Tantivy   │
       │ persistence │         │ full search │
       │  (rusqlite) │         │ (embedded)  │
       └──────┬──────┘         └──────┬──────┘
              │                       │
              └───────────┬───────────┘
                          │
                          ▼
             ┌─────────────────────────┐
             │     Dioxus Desktop      │
             │  Multi-Tab Workspace    │
             │  Search • Notes • Tasks │
             │  Graph View • Explorer  │
             └─────────────────────────┘
```

### Storage Invariant & Rebuildable Indexes

```text
Filesystem (.md) ──[Atomic Write]──► Disk
       │
       ├──[Sync / Watcher]──► SQLite  (links, tags, tasks, metadata)
       └──[Sync / Watcher]──► Tantivy (lexical tokens, headings, body text)
```

If the SQLite database or Tantivy index is corrupted or deleted, Nodera detects index invalidation on boot and completely reconstructs both derived stores directly from raw Markdown in **under 250ms**.

---

## Current Capabilities vs. Experimental

We maintain an explicit separation between production-ready capabilities and active R&D to provide an honest engineering picture:

### Production-Ready Capabilities (Stable)

- [x] **Local-First Plain Markdown Source of Truth**: Safe relative path traversal, atomic tempfile writes, and real-time filesystem watcher synchronization.
- [x] **Pure-Rust Streaming PDF Ingestion**: Chapter & heading discovery, running header/footer suppression, and hyphenation repair via `nodera-pdf` (~875 pages/sec).
- [x] **Dual-Tier Embedded Indexing**: Relational links, backlinks, and task states indexed in SQLite; ranked BM25 lexical search indexed in Tantivy.
- [x] **Sub-3ms Full-Text Search**: Instant search over note titles, headings, body content, and inline tags with highlighted snippet extraction.
- [x] **Self-Healing Index Recovery**: Automatic cold rebuild from Markdown if transient index stores are absent or corrupted.
- [x] **Multi-Tab Reactive Desktop UI**: Built on Dioxus 0.6 desktop renderer, featuring split-pane editing, live table of contents, and word counts.
- [x] **Universal Task Aggregation**: Scans `- [ ]` / `- [x]` Markdown checkboxes across every note in the vault with filterable dashboard views.
- [x] **7 Calibrated Workspace Themes**: Includes **Graphite** (restrained near-black `#0D0D0D` canvas, `#8E95A5` steel accent, 14.9:1 contrast ratio), Nodera Dark, Light, Midnight, Nord, Dracula, and Solarized with universal dark-adapted scrollbar chrome.

### Experimental Capabilities (Active R&D)

- [ ] **2D Force-Directed Knowledge Graph**: Barnes-Hut $O(N \log N)$ spatial force approximation with thread-local QuadTree scratch buffers. Currently tuning projection budgets to transition from SVG DOM to high-scale WebGL/Canvas backends.
- [ ] **AST Symbol Dependency Graph**: Static parsing of Rust (`syn`) and foreign language codebases into structural knowledge entities via `nodera-project`.
- [ ] **Semantic / Vector Search**: Exploring pure-Rust embedded vector embeddings (HNSW) to complement lexical Tantivy search without external cloud APIs.

---

## Measurable Engineering Benchmarks

All metrics measured on an AMD Ryzen 9 workstation (Windows 11, NVMe SSD). Zero manufactured benchmarks:

| Benchmark Workload | Dataset / Target | Measured Metric | Peak RSS | Status |
| :--- | :--- | :--- | :--- | :--- |
| **PDF Ingestion Throughput** | 600-page academic book (BT001S26) | **685 ms** (~875.7 pages/sec) | 91 MB | **VERIFIED** |
| **Vault Cold Indexing** | 1,000 hierarchical Markdown notes | **908 ms** (~1,101 notes/sec) | 57 MB | **VERIFIED** |
| **Tantivy Lexical Query** | 1,000 indexed notes (warm cache) | **2.38 ms** (target: < 50ms) | — | **VERIFIED** |
| **Index Self-Healing Rebuild** | Corrupted SQLite & Tantivy files | **230 ms** end-to-end recovery | 42 MB | **VERIFIED** |
| **Desktop Startup Footprint** | Cold desktop launch | **34.7 MB RSS** | 35 MB | **VERIFIED** |
| **Physics Relaxation Tick** | 1,000 nodes Barnes-Hut QuadTree | **1.84 ms / tick** (target: < 16ms) | 12 MB | **VERIFIED** |

---

## Why Rust?

Rust was chosen because Nodera executes CPU-intensive document ingestion, AST parsing, lexical indexing, and graph layout in the same client process where GUI rendering takes place:

| Technical Requirement | Technology Chosen | Alternative Considered | Technical Trade-off & Rationale |
| :--- | :--- | :--- | :--- |
| **Desktop Application** | **Dioxus 0.6 (Wry/Tao)** | Electron / Chromium | Dioxus renders natively via OS webviews with a Rust event loop. Avoids shipping a duplicate Node.js + Chromium runtime (~150MB+ bundle, 300MB+ idle RAM). |
| **PDF Ingestion** | **Pure Rust (`lopdf`)** | Python (`PyMuPDF`) / Poppler | Avoids requiring Python runtimes or external C shared library dependencies. Allows streaming in-process parsing with zero-copy buffer slices. |
| **Full-Text Search** | **Tantivy** | SQLite FTS5 / Meilisearch | Tantivy is an embedded, pure-Rust Lucene equivalent. Delivers configurable BM25 ranking, tokenization, and schema indexing without running an external server daemon. |
| **Relational Metadata** | **SQLite (`rusqlite`)** | RocksDB / sled | Single-file transactional database with ACID durability, queryable relational joins for Wikilinks, and zero background daemons. |
| **Concurrency Model** | **Tokio + Rayon** | Single-threaded async | Heavy batch tasks (PDF parsing, index rebuilds) run on bounded worker thread pools, ensuring the desktop UI event loop stays locked at 60 FPS. |

---

## Crate Boundaries

Nodera enforces strict, unidirectional dependency layers across its multi-crate workspace:

```text
crates/
├── nodera-core/         # Vault domain model, safe relative path resolution, atomic writes
├── nodera-markdown/     # AST parsing (pulldown-cmark), Wikilinks, backlinks, tasks, graph projection
├── nodera-index/        # SQLite relations + Tantivy embedded full-text search & recovery
├── nodera-pdf/          # Streaming PDF extraction, heading heuristics, paragraph reconstruction
├── nodera-project/      # AST code discovery & workspace project graph integration
├── nodera-parser-core/  # Language-agnostic code symbol extraction interfaces
├── nodera-desktop/      # Dioxus 0.6 reactive GUI, theme engine, interactive graph canvas
└── nodera-cli/          # Headless vault indexer, updater, and CLI protocol handler
```

### Dependency Graph

```text
                 nodera-desktop
                       │
         ┌─────────────┼─────────────┐
         ▼             ▼             ▼
   nodera-index   nodera-pdf   nodera-project
         │             │             │
         ▼             │             ▼
  nodera-markdown      │     nodera-parser-core
         │             │             │
         └─────────────┼─────────────┘
                       ▼
                  nodera-core
```

*Coupling Rules:* `nodera-core` has zero internal workspace dependencies. `nodera-pdf` and `nodera-index` never depend on `nodera-desktop`. UI concerns are strictly isolated within the desktop crate.

---

## Architecture Decision Records (ADRs)

Key architectural choices are formally documented in [`docs/decisions/`](docs/decisions/):

- [**ADR-0001: Native Dioxus Desktop over Web/Electron**](docs/decisions/ADR-0001-native-dioxus-desktop.md): Evaluates Electron vs Tauri vs Dioxus Desktop. Selected Dioxus for pure-Rust state management and native OS windowing.
- [**ADR-0002: Markdown Filesystem as Sole Source of Truth**](docs/decisions/ADR-0002-markdown-source-of-truth.md): Rejects opaque database persistence for user notes. Plain `.md` files remain immutable authority.
- [**ADR-0003: Embedded In-Process Engine vs Client-Server/Axum**](docs/decisions/ADR-0003-no-axum-in-v1.md): Avoids local HTTP socket overhead and security attack surfaces by embedding indexing directly in-process.
- [**ADR-0004: In-Process Pure-Rust PDF Ingestion over External Containers**](docs/decisions/ADR-0004-no-container-for-pdf-v1.md): Eliminates Docker or Python runtime prerequisites for PDF digestion.
- [**ADR-0005: Task Data Embedded in Markdown Checkboxes**](docs/decisions/ADR-0005-task-data-in-markdown.md): Implements bidirectional syncing for `- [ ]` tasks without out-of-band database drift.

---

## Known Limitations

We believe in engineering transparency. Current known limitations include:

1. **PDF Structure Sensitivity**: PDF text extraction relies on layout positioning heuristics. Two-column academic papers and complex tables extract well, but non-standard text streams or scanned non-OCR PDFs require external pre-processing.
2. **SVG Graph DOM Overhead at Extreme Scale**: The SVG knowledge graph performs smoothly up to ~1,500 visible nodes. For massive 10,000+ node graphs, Nodera enforces a projection render budget while active development focuses on a dedicated Canvas/WebGL rendering pipeline.
3. **Filesystem Watcher Timing**: OS-level directory watchers (ReadDirectoryChangesW on Windows, inotify on Linux) require debouncing logic to prevent self-triggering loops during batch atomic writes.

---

## Getting Started

### Prerequisites

- [Rust 1.80+ Toolchain](https://rustup.rs/) (`cargo`, `rustc`)
- Supported Platforms: Windows 10/11, macOS (Apple Silicon / Intel), Linux

### Build & Run

```bash
# 1. Clone repository
git clone https://github.com/l1khith/Nodera.git
cd Nodera

# 2. Build desktop binary in release mode
cargo build --release -p nodera-desktop

# 3. Launch application
./target/release/nodera-desktop
```

### Test Suite & Quality Verification

Nodera enforces a strict zero-warning policy across Clippy, formatting, and tests:

```bash
# Run the complete test suite across all workspace crates
cargo test --workspace

# Run Clippy lints with zero warnings allowed
cargo clippy --workspace --all-targets --all-features -- -D warnings

# Verify formatting
cargo fmt --all -- --check
```

---

## Keyboard Shortcuts

| Shortcut | Action | Scope |
| :--- | :--- | :--- |
| <kbd>Ctrl</kbd> + <kbd>N</kbd> | Create a new note | Global |
| <kbd>Ctrl</kbd> + <kbd>S</kbd> | Save current note | Editor |
| <kbd>Ctrl</kbd> + <kbd>P</kbd> | Open Command Palette / Quick Search | Global |
| <kbd>Ctrl</kbd> + <kbd>E</kbd> | Toggle Reading / Source View | Editor |
| <kbd>Ctrl</kbd> + <kbd>G</kbd> | Toggle Inspector Panel (Local Graph) | Global |
| <kbd>Ctrl</kbd> + <kbd>I</kbd> | Import PDF Document as Markdown | Global |
| <kbd>Ctrl</kbd> + <kbd>,</kbd> | Open Settings & Appearance Palette | Global |
| <kbd>Escape</kbd> | Dismiss active modal or palette | Dialogs |

---

## Documentation Index

- [`docs/architecture.md`](docs/architecture.md) — Comprehensive concurrency, state store, and storage architecture.
- [`docs/design-system.md`](docs/design-system.md) — Canonical design tokens, typography, and theme definitions.
- [`docs/product-catalog.md`](docs/product-catalog.md) — Complete 20-domain product and engineering status catalog.
- [`docs/pdf-import.md`](docs/pdf-import.md) — Ingestion pipeline, paragraph reconstruction, and heading heuristics.
- [`docs/performance.md`](docs/performance.md) — Scalability benchmarks and flamegraph profiling instructions.
- [`docs/decisions/`](docs/decisions/) — Architecture Decision Records (ADRs).

---

## License

Nodera is free and open-source software licensed under the **GNU Affero General Public License v3.0** ([AGPL-3.0](LICENSE)).
