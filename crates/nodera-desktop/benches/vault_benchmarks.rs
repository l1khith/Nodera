use std::path::Path;
use std::time::{Duration, Instant};
use tempfile::tempdir;

use nodera_core::{Vault, VaultService};
use nodera_desktop::components::graph_view::{
    init_simulation_with_forces, step_simulation_with_forces,
};
use nodera_desktop::state::GraphForcesSettings;
use nodera_desktop::AppState;
use nodera_index::VaultIndex;
use nodera_markdown::{parse_document, LinkGraph};

#[derive(Debug, Default, Clone)]
pub struct LatencyStats {
    pub total: Duration,
    pub count: usize,
    pub throughput: f64,
    pub median: Duration,
    pub p95: Duration,
    pub p99: Duration,
    pub min: Duration,
    pub max: Duration,
}

impl LatencyStats {
    pub fn compute(mut samples: Vec<Duration>) -> Self {
        if samples.is_empty() {
            return Self::default();
        }
        samples.sort();
        let count = samples.len();
        let total: Duration = samples.iter().sum();
        let throughput = if total.as_secs_f64() > 0.0 {
            count as f64 / total.as_secs_f64()
        } else {
            0.0
        };
        let median = samples[count / 2];
        let p95 = samples[((count as f64 * 0.95).ceil() as usize).min(count - 1)];
        let p99 = samples[((count as f64 * 0.99).ceil() as usize).min(count - 1)];
        let min = samples[0];
        let max = samples[count - 1];

        Self {
            total,
            count,
            throughput,
            median,
            p95,
            p99,
            min,
            max,
        }
    }

    pub fn print_row(&self, name: &str) {
        println!(
            "| {:<32} | {:>10.2?} | {:>10.1} | {:>10.2?} | {:>10.2?} | {:>10.2?} |",
            name, self.total, self.throughput, self.median, self.p95, self.p99
        );
    }
}

fn create_dataset(vault_path: &Path, count: usize) -> VaultService {
    let vault = Vault::create(vault_path, Some(format!("Benchmark Vault {count}"))).unwrap();
    let service = VaultService::new(vault);

    for i in 0..count {
        let folder = match i % 5 {
            0 => "Projects/Alpha",
            1 => "Projects/Beta",
            2 => "Research/AI",
            3 => "Daily/2026",
            _ => "Archive/Misc",
        };
        let rel_path = format!("{folder}/Note_{i}.md");
        let target_a = (i + 1) % count;
        let target_b = (i + 7) % count;
        let content = format!(
            r#"---
title: Note {i}
tags:
  - benchmark
  - scale
  - tier{tier}
status: active
priority: {priority}
---

# Note {i}

This is a synthetic benchmark note designed for stress-testing Nodera indexing pipelines.
It contains references to [[Projects/Alpha/Note_{target_a}]] and [[Daily/2026/Note_{target_b}|Target B]].

## Tasks
- [{t1}] Benchmark discovery phase
- [{t2}] Benchmark parsing phase
- [ ] Verify index consistency

## Details
Rust provides mechanical sympathy with thread pools and memory predictability.
"#,
            tier = i % 10,
            priority = (i % 5) + 1,
            target_a = target_a,
            target_b = target_b,
            t1 = if i % 2 == 0 { "x" } else { " " },
            t2 = if i % 3 == 0 { "x" } else { " " },
        );
        service.write_note(&rel_path, &content).unwrap();
    }
    service
}

fn run_benchmark_tier(note_count: usize) {
    println!("\n================================================================================");
    println!("RUNNING RELEASE BENCHMARK: {} NOTES", note_count);
    println!("================================================================================");

    let dir = tempdir().unwrap();
    let vault_path = dir.path().to_path_buf();

    // 0. Dataset Creation
    let start_create = Instant::now();
    let service = create_dataset(&vault_path, note_count);
    let create_dur = start_create.elapsed();
    println!("Dataset generated in {:.2?}", create_dur);

    println!(
        "\n| {:<32} | {:>10} | {:>10} | {:>10} | {:>10} | {:>10} |",
        "Operation", "Total", "Notes/sec", "Median", "p95", "p99"
    );
    println!(
        "|{:-<34}|{:-<12}|{:-<12}|{:-<12}|{:-<12}|{:-<12}|",
        "", "", "", "", "", ""
    );

    // 1. Vault Scan
    let start_scan = Instant::now();
    let entries = service.list_entries().unwrap();
    let scan_dur = start_scan.elapsed();
    let scan_stats = LatencyStats {
        total: scan_dur,
        count: entries.len(),
        throughput: entries.len() as f64 / scan_dur.as_secs_f64().max(0.0001),
        median: scan_dur,
        p95: scan_dur,
        p99: scan_dur,
        min: scan_dur,
        max: scan_dur,
    };
    scan_stats.print_row("1. Vault Scan (list_entries)");

    // Read all notes in memory for subsequent granular tests
    let mut note_contents = Vec::with_capacity(note_count);
    let mut note_paths = Vec::with_capacity(note_count);
    for entry in &entries {
        if let nodera_core::VaultEntry::Note(summary) = entry {
            if let Ok(note) = service.read_note(&summary.relative_path) {
                note_paths.push(summary.relative_path.clone());
                note_contents.push(note.content);
            }
        }
    }

    // 2. Markdown Parsing & Entity Extraction
    let mut parse_samples = Vec::with_capacity(note_contents.len());
    let mut parsed_docs = Vec::with_capacity(note_contents.len());
    for content in &note_contents {
        let t0 = Instant::now();
        let doc = parse_document(content).unwrap();
        parse_samples.push(t0.elapsed());
        parsed_docs.push(doc);
    }
    let parse_stats = LatencyStats::compute(parse_samples);
    parse_stats.print_row("2. Markdown Parsing (serial)");

    // 3. Graph Construction
    let start_graph = Instant::now();
    let mut link_graph = LinkGraph::new();
    for (i, doc) in parsed_docs.iter().enumerate() {
        link_graph.update_note_links(note_paths[i].clone(), doc.wikilinks.clone());
    }
    let graph_dur = start_graph.elapsed();
    let graph_stats = LatencyStats {
        total: graph_dur,
        count: parsed_docs.len(),
        throughput: parsed_docs.len() as f64 / graph_dur.as_secs_f64().max(0.0001),
        median: graph_dur,
        p95: graph_dur,
        p99: graph_dur,
        min: graph_dur,
        max: graph_dur,
    };
    graph_stats.print_row("3. Graph Construction");

    // 4. Graph Layout Simulation Tick
    let graph_data = link_graph.to_graph_data(&note_paths);
    let forces = GraphForcesSettings::default();
    let (mut sim_nodes, sim_edges) =
        init_simulation_with_forces(&graph_data, 1000.0, 700.0, &forces);
    let mut tick_samples = Vec::with_capacity(20);
    for _ in 0..20 {
        let t0 = Instant::now();
        step_simulation_with_forces(
            &mut sim_nodes,
            &sim_edges,
            (500.0, 350.0),
            None,
            0.5,
            &forces,
        );
        tick_samples.push(t0.elapsed());
    }
    let tick_stats = LatencyStats::compute(tick_samples);
    tick_stats.print_row("4. Graph Sim Tick (Barnes-Hut)");

    // 5. Full Index Rebuild (SQLite + Tantivy)
    let start_rebuild = Instant::now();
    let mut vault_index = VaultIndex::open(&vault_path).unwrap();
    let rebuild_count = vault_index.rebuild(&service).unwrap();
    let rebuild_dur = start_rebuild.elapsed();
    let rebuild_stats = LatencyStats {
        total: rebuild_dur,
        count: rebuild_count,
        throughput: rebuild_count as f64 / rebuild_dur.as_secs_f64().max(0.0001),
        median: rebuild_dur,
        p95: rebuild_dur,
        p99: rebuild_dur,
        min: rebuild_dur,
        max: rebuild_dur,
    };
    rebuild_stats.print_row("5. Index Rebuild (Current)");

    // 6. Full-Text Search Queries
    let queries = [
        "benchmark",
        "scale",
        "mechanical sympathy",
        "Note 42",
        "Projects/Alpha",
        "synthetic",
        "pipeline",
        "tier5",
    ];
    let mut search_samples = Vec::new();
    for _ in 0..10 {
        for q in &queries {
            let t0 = Instant::now();
            let _ = vault_index.search(q, 20).unwrap();
            search_samples.push(t0.elapsed());
        }
    }
    let search_stats = LatencyStats::compute(search_samples);
    search_stats.print_row("6. Full-Text Search (Tantivy)");

    // 7. Incremental Note Edit -> Index Update
    let mut inc_samples = Vec::new();
    for target_path in note_paths.iter().take(50) {
        let note = service.read_note(target_path).unwrap();
        let modified = format!("{}\n\nUpdated at {:?}", note.content, Instant::now());
        let doc = parse_document(&modified).unwrap();
        let t0 = Instant::now();
        vault_index.index_note(&note, &doc).unwrap();
        inc_samples.push(t0.elapsed());
    }
    let inc_stats = LatencyStats::compute(inc_samples);
    inc_stats.print_row("7. Incremental Note Index");

    // 8. Incremental Note Edit -> Graph Update
    let mut graph_inc_samples = Vec::new();
    for target_path in note_paths.iter().take(50) {
        let new_links = vec![nodera_markdown::Wikilink {
            raw: "[[Projects/Beta/Note_99]]".to_string(),
            target: "Projects/Beta/Note_99".to_string(),
            display_text: None,
            start: 0,
            end: 25,
        }];
        let t0 = Instant::now();
        link_graph.update_note_links(target_path.clone(), new_links);
        graph_inc_samples.push(t0.elapsed());
    }
    let graph_inc_stats = LatencyStats::compute(graph_inc_samples);
    graph_inc_stats.print_row("8. Incremental Graph Update");

    // 9. Cold AppState Open
    let mut app_state = AppState::default();
    let start_open = Instant::now();
    app_state.open_vault(&vault_path).unwrap();
    let open_dur = start_open.elapsed();
    let open_stats = LatencyStats {
        total: open_dur,
        count: 1,
        throughput: 1.0 / open_dur.as_secs_f64().max(0.0001),
        median: open_dur,
        p95: open_dur,
        p99: open_dur,
        min: open_dur,
        max: open_dur,
    };
    open_stats.print_row("9. Cold AppState::open_vault");
}

fn main() {
    println!("=== NODERA PERFORMANCE PHASE BASELINE PROFILER ===");
    run_benchmark_tier(1_000);

    // Run 10K tier
    run_benchmark_tier(10_000);
}
