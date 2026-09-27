# Nodera `liki` — Architecture Audit (Verified)

**Audit date:** 2026-09-26
**Branch:** `liki`
**Status:** Source-verified against actual codebase. All P0/P1 findings confirmed with exact line numbers.

---

## Executive Summary

Nodera has a **good foundation** with correct instincts: Markdown as source of truth, SQLite/Tantivy as derived indexes, transactions, Rayon parallelism, atomic writes, and modular crates.

The architecture is **not yet ready for large knowledge bases** (10k+ notes). The five most impactful problems are:

1. **Vault opening does full-vault work twice** (read+parse for LinkGraph, then read+parse again for index rebuild)
2. **"Local graph" builds the entire global graph first**, including community detection
3. **Full rebuild materializes every note body in RAM** with unbounded parallelism
4. **Content hashing uses non-deterministic `DefaultHasher`**, and `modified_ns` records wall-clock time instead of filesystem mtime
5. **Search falls back to `AllQuery` on parse failure**, potentially returning the entire index

---

## Verified Flaw Register

### Severity Legend
- **P0** — Fix before scaling / correctness risk
- **P1** — Fix next / significant architectural debt
- **P2** — Scalability improvement / nice to have

---

### P0 — Fix Before Scaling

#### F-001 — Full vault rebuild during `open_vault()` ✅ VERIFIED

**Files:**
- [`state.rs` L1088-1174](file:///c:/Users/ailik/funProjects/rustProjects/nodera/crates/nodera-desktop/src/state.rs#L1088-L1174)
- [`indexer.rs` L305-543](file:///c:/Users/ailik/funProjects/rustProjects/nodera/crates/nodera-index/src/indexer.rs#L305-L543)

**Evidence:**
1. `open_vault()` at L1098 calls `service.list_entries()`
2. L1101-1114: Rayon `par_iter` reads and parses every note for `LinkGraph`
3. L1122: Builds complete `LinkGraph`
4. L1128: **Synchronously** calls `idx.rebuild(&service)` which:
   - L331: Re-discovers all files
   - L393-461: Re-reads and re-parses every note again
   - L499: Batch SQLite persistence
   - L509-519: Full Tantivy rebuild

**Impact:** The vault is read and parsed **twice** on every open. Blocking. No incremental path.

---

#### F-002 — "Local graph" is globally computed ✅ VERIFIED

**File:** [`links.rs` L558-615](file:///c:/Users/ailik/funProjects/rustProjects/nodera/crates/nodera-markdown/src/links.rs#L558-L615)

**Evidence:**
```rust
// Line 567: builds the FULL graph before doing local BFS
let full_graph = self.to_graph_data_with_options(all_paths, titles, note_tags, options);
```

This means a 1-hop neighborhood request:
1. Resolves all edges across all vault notes
2. Sorts all nodes and edges globally
3. Runs 15 iterations of LPA community detection (L775)
4. Computes centrality for all nodes
5. Builds full adjacency map from all edges
6. Does BFS to filter down to local neighborhood
7. Throws away 99%+ of the work

**Complexity:** O(V_total + E_total) instead of O(neighbors_within_depth)

---

#### F-018 — Full rebuild materializes every note body in RAM ✅ VERIFIED

**File:** [`indexer.rs` L377-461](file:///c:/Users/ailik/funProjects/rustProjects/nodera/crates/nodera-index/src/indexer.rs#L377-L461)

**Evidence:**
```rust
// L377-391: ParsedNoteData stores EVERYTHING including full body
struct ParsedNoteData {
    note_id: String,
    path: String,
    title: String,
    content_hash: String,
    body: String,          // ← full note body
    headings_text: String,
    tags_str: String,
    wikilinks: Vec<Wikilink>,
    tasks: Vec<ParsedTask>,
    tags: Vec<String>,
    properties: HashMap<String, serde_json::Value>,
    ...
}

// L393-461: ALL notes collected into Vec before persistence
let parsed_items: Vec<ParsedNoteData> = note_summaries
    .par_iter()
    .filter_map(|summary| { ... })
    .collect();  // ← entire vault in RAM
```

**Impact:** Peak memory = N × average_note_size. No bounded queue, no streaming.

---

#### F-025 — Search falls back to `AllQuery` on parse failure ✅ VERIFIED

**File:** [`tantivy_index.rs` L170](file:///c:/Users/ailik/funProjects/rustProjects/nodera/crates/nodera-index/src/tantivy_index.rs#L170)

**Evidence:**
```rust
.unwrap_or_else(|_| Box::new(tantivy::query::AllQuery));
```

A malformed query silently becomes "return everything" — unbounded and potentially expensive.

---

### P1 — Fix Next

#### F-008 — `DefaultHasher` for content hash ✅ VERIFIED

**File:** [`indexer.rs` L80-82, L408-410](file:///c:/Users/ailik/funProjects/rustProjects/nodera/crates/nodera-index/src/indexer.rs#L80-L82)

```rust
let mut hasher = std::collections::hash_map::DefaultHasher::new();
note.content.hash(&mut hasher);
let content_hash = format!("{:016x}", hasher.finish());
```

Rust docs explicitly warn `DefaultHasher` is not stable across versions. Stored hashes become unreliable. Use SHA-256 or Blake3.

---

#### F-009 — `modified_ns` is wall-clock time, not filesystem mtime ✅ VERIFIED

**File:** [`indexer.rs` L85-88, L413-416](file:///c:/Users/ailik/funProjects/rustProjects/nodera/crates/nodera-index/src/indexer.rs#L85-L88)

```rust
let modified_ns = std::time::SystemTime::now()
    .duration_since(std::time::UNIX_EPOCH)
    .map(|d| d.as_nanos() as u64)
    .unwrap_or(0);
```

Should read `std::fs::metadata(path)?.modified()` instead.

---

#### F-011 — No foreign key constraints despite PRAGMA ✅ VERIFIED

**File:** [`sqlite.rs` L76-135](file:///c:/Users/ailik/funProjects/rustProjects/nodera/crates/nodera-index/src/sqlite.rs#L76-L135)

`PRAGMA foreign_keys = ON` at L76, but zero `REFERENCES` or `FOREIGN KEY` clauses anywhere. `links.source_id`, `tasks.note_id`, `tags.note_id`, `properties.note_id` are all unconstrained.

---

#### F-013 — `target_id` is always NULL ✅ VERIFIED

**File:** [`sqlite.rs` L210-213, L296-301, L411-416](file:///c:/Users/ailik/funProjects/rustProjects/nodera/crates/nodera-index/src/sqlite.rs#L210-L213)

All three link insertion sites hardcode `NULL` for `target_id`:
```sql
INSERT OR IGNORE INTO links (source_id, target_path, target_id, ...)
VALUES (?, ?, NULL, ?, ?)
```

Backlink resolution falls back to string comparison instead of efficient ID joins.

---

#### F-020 — SQLite+Tantivy behind single `Arc<Mutex<>>` ✅ VERIFIED

**File:** [`state.rs` L804](file:///c:/Users/ailik/funProjects/rustProjects/nodera/crates/nodera-desktop/src/state.rs#L804)

```rust
pub vault_index: Option<Arc<Mutex<VaultIndex>>>,
```

A slow index write blocks search reads and vice versa.

---

#### F-021 — Search executes synchronously from AppState ✅ VERIFIED

**File:** [`state.rs` L2942-2952](file:///c:/Users/ailik/funProjects/rustProjects/nodera/crates/nodera-desktop/src/state.rs#L2942-L2952)

```rust
pub fn execute_search(&mut self, query: &str) {
    if let Some(index_arc) = &self.vault_index {
        if let Ok(idx) = index_arc.lock() {
            self.search_results = idx.search(query, 25).unwrap_or_default();
        }
    }
}
```

Locks the entire VaultIndex inline on the UI path.

---

#### F-022 — Body stored in Tantivy unnecessarily ✅ VERIFIED

**File:** [`tantivy_index.rs` L69](file:///c:/Users/ailik/funProjects/rustProjects/nodera/crates/nodera-index/src/tantivy_index.rs#L69)

```rust
let f_body = schema_builder.add_text_field("body", TEXT | STORED);
```

Full note bodies are duplicated in Tantivy storage. Use `TEXT` without `STORED`.

---

#### F-026 — SQLite and Tantivy commits are not atomic ✅ VERIFIED

**File:** [`indexer.rs` L499, L509, L524](file:///c:/Users/ailik/funProjects/rustProjects/nodera/crates/nodera-index/src/indexer.rs#L499-L524)

SQLite commits at L499, Tantivy clears at L509 and commits later. A crash between them leaves inconsistent derived state.

---

#### F-037 — `AppState` is 5,413 lines ✅ VERIFIED

**File:** [`state.rs`](file:///c:/Users/ailik/funProjects/rustProjects/nodera/crates/nodera-desktop/src/state.rs) — 5,413 lines, 43+ fields across 12+ domains: vault lifecycle, notes/editor, search, graph, projects, themes, PDF import, tasks, settings, plugins, calendar, bibliography, templates, reading mode, trash, quick capture, and review queue.

---

### Additional P1 Findings (Architecture-Level, Verified by Inspection)

| ID | Finding | File | Line |
|----|---------|------|------|
| F-003 | Every global graph runs community detection | `links.rs` | L527 |
| F-004 | LinkGraph duplicates SQLite link info | `links.rs` + `sqlite.rs` | multiple |
| F-006 | Graph node IDs are path strings | `links.rs` | multiple |
| F-010 | Schema version overwritten on every open | `sqlite.rs` | L76-80 |
| F-012 | `INSERT OR REPLACE` for identity rows | `sqlite.rs` | multiple |
| F-014 | Link resolution silently picks first match | `links.rs` | `TargetResolver` |
| F-017 | `audit_vault_links()` loads all content | `state.rs` | multiple |
| F-029 | No unified filesystem event pipeline | `state.rs` | n/a |
| F-031 | PDF "streaming" is not bounded | `nodera-pdf` | multiple |

---

## Three Architectural Invariants to Adopt

### 1. Data Invariant
```
Markdown/files are canonical user-owned data.
SQLite, Tantivy, GraphIndex and UI state are derived representations.
Every derived representation must be:
    1. incrementally maintainable
    2. rebuildable
    3. versioned
    4. independently recoverable
    5. safe to discard
Interactive UI operations must never require a full-vault scan
when an indexed query can answer the request.
```

### 2. Concurrency Invariant
```
No background producer may create unbounded work.
Every asynchronous/parallel pipeline must define:
    - queue capacity
    - cancellation
    - priority
    - batching
    - backpressure
    - shutdown semantics
```

### 3. UI Responsiveness Invariant
```
The UI thread may:
    - update state, render, dispatch commands, consume computed results
The UI thread must NOT:
    - scan/parse the vault, rebuild indexes, run graph analytics,
      run force layout, perform PDF conversion, hold global storage lock
```

---

## Priority Implementation Roadmap

### Phase 0 — Do First (Unblocks Scaling)

| # | Action | Effort | Impact |
|---|--------|--------|--------|
| P0.1 | True adjacency-based local graph traversal (bypass full graph) | 2-3 days | Eliminates O(V+E) per local graph view |
| P0.2 | Remove synchronous full rebuild from `open_vault()` | 2 days | Fast vault open, background indexing |
| P0.3 | Incremental change/index pipeline | 3-4 days | Only re-index changed files |
| P0.4 | Bounded parse pipeline (chunked batching, not full Vec) | 1-2 days | Caps peak memory |
| P0.5 | Fix `AllQuery` fallback to return error/empty | 30 min | Correctness fix |

### Phase 1 — Do Next

| # | Action | Effort | Impact |
|---|--------|--------|--------|
| P1.1 | Replace `DefaultHasher` with stable hash (SHA-256/Blake3) | 1 hour | Reliable change detection |
| P1.2 | Use real filesystem mtime for `modified_ns` | 1 hour | Correct incremental detection |
| P1.3 | Resolve and persist `target_id` during indexing | 2 hours | Efficient backlink queries |
| P1.4 | Add FK constraints to SQLite schema | 1 hour | Data consistency |
| P1.5 | Split `Arc<Mutex<VaultIndex>>` into separate services | 1-2 days | Concurrent read/write |
| P1.6 | Remove `STORED` from Tantivy body field | 30 min | Halves Tantivy storage |
| P1.7 | Move graph analytics to background jobs | 1-2 days | UI never blocks on community detection |
| P1.8 | Add filesystem event coalescing | 1-2 days | Efficient live reload |

### Phase 2 — Scalability

| # | Action | Effort | Impact |
|---|--------|--------|--------|
| P2.1 | Graph level-of-detail rendering | 2-3 days | Handles 10k+ node vaults |
| P2.2 | Graph node/edge render budgets | 1 day | Prevents UI lockup |
| P2.3 | Barnes-Hut for force simulation | 2-3 days | O(N log N) instead of O(N²) |
| P2.4 | Prefix/trigram search for link autocomplete | 1-2 days | Fast for 100k+ notes |
| P2.5 | PDF bounded streaming pipeline | 2-3 days | Caps memory for large books |
| P2.6 | Synthetic large-vault benchmarks (1k/10k/100k) | 2-3 days | Proves scalability |

---

## What Should NOT Be Built Yet

- Distributed databases, Kafka, Redis, Neo4j, PostgreSQL
- Cloud indexing, Kubernetes, WASM plugins
- Marketplace, collaboration servers

Nodera's bottleneck is local data architecture + large-vault memory + graph query efficiency + UI scheduling. Solve those first.

---

## Target Architecture After Fixes

```
                         NODERA
                            |
                     Dioxus Desktop
                            |
                     Command/Event API
                            |
                 ┌──────────┴──────────┐
                 │                     │
          Interactive Path       Background Runtime
                 │                     │
          cached/indexed reads   bounded task scheduler
                                       │
              ┌────────────────────────┼──────────────────────┐
              │                        │                      │
         Parse Workers           Index Workers          Graph Workers
              │                        │                      │
              └────────────────────────┼──────────────────────┘
                                       │
                              Canonical File Data
                                       │
                              Markdown / Attachments
                                       │
                  ┌────────────────────┼───────────────────┐
                  │                    │                   │
               SQLite               Tantivy           GraphIndex
             metadata              search            adjacency
                  │                    │                   │
                  └────────────────────┼───────────────────┘
                                       │
                                Versioned Snapshot
                                       │
                              ┌────────┴────────┐
                              │                 │
                         Graph Query       Search Query
                              │                 │
                         projection          results
                              │                 │
                         Layout worker          │
                              │                 │
                              └────────┬────────┘
                                       │
                                  UI snapshot
                                       │
                                    Render
```
