use std::collections::HashMap;
use std::path::PathBuf;
use std::time::{Duration, Instant};

use nodera_desktop::components::graph_view::{
    init_simulation_with_forces, step_simulation_with_forces,
};
use nodera_desktop::state::GraphForcesSettings;
use nodera_markdown::{GraphFilterOptions, LinkGraph, Wikilink};

#[derive(Debug, Default, Clone)]
pub struct BenchMetric {
    pub name: String,
    pub node_count: usize,
    pub edge_count: usize,
    pub duration: Duration,
    pub iterations: usize,
    pub avg_tick_ms: f64,
}

impl BenchMetric {
    pub fn print(&self) {
        println!(
            "| {:<28} | {:>6} nodes | {:>6} edges | {:>10.3?} | {:>8.3} ms/step |",
            self.name,
            self.node_count,
            self.edge_count,
            self.duration,
            self.avg_tick_ms
        );
    }
}

/// Generates a synthetic LinkGraph with realistic average degree (3-5 edges per node).
fn generate_synthetic_graph(node_count: usize) -> (LinkGraph, Vec<PathBuf>) {
    let mut all_paths = Vec::with_capacity(node_count);
    let mut note_links = Vec::with_capacity(node_count);

    for i in 0..node_count {
        let path = PathBuf::from(format!("Notes/Note_{:05}.md", i));
        all_paths.push(path.clone());

        // Connect to 3-5 neighbors (local clusters + some hubs)
        let mut links = Vec::new();
        let target_1 = (i + 1) % node_count;
        let target_2 = (i + 3) % node_count;
        let hub_target = i % 25; // Create periodic hubs

        links.push(Wikilink {
            raw: format!("[[Note_{:05}]]", target_1),
            target: format!("Note_{:05}", target_1),
            display_text: None,
            start: 0,
            end: 0,
        });
        links.push(Wikilink {
            raw: format!("[[Note_{:05}]]", target_2),
            target: format!("Note_{:05}", target_2),
            display_text: None,
            start: 0,
            end: 0,
        });
        if i != hub_target {
            links.push(Wikilink {
                raw: format!("[[Note_{:05}]]", hub_target),
                target: format!("Note_{:05}", hub_target),
                display_text: None,
                start: 0,
                end: 0,
            });
        }

        note_links.push((path, links));
    }

    let link_graph = LinkGraph::build(&all_paths, note_links);
    (link_graph, all_paths)
}

fn benchmark_tier(node_count: usize) {
    println!("\n==========================================================================");
    println!("  BENCHMARK TIER: {} NODES", node_count);
    println!("==========================================================================");

    let (link_graph, paths) = generate_synthetic_graph(node_count);
    let filter_options = GraphFilterOptions {
        existing_files_only: true,
        orphans: true,
        tags: false,
        attachments: false,
    };
    let titles = HashMap::new();
    let note_tags = HashMap::new();

    // 1. Full Graph Generation (Linear map + LPA Community Detection + Degree Centrality)
    let t0 = Instant::now();
    let full_graph = link_graph.to_graph_data_with_options(
        &paths,
        &titles,
        &note_tags,
        &filter_options,
    );
    let full_gen_dur = t0.elapsed();
    BenchMetric {
        name: "1. Full Graph Gen (LPA+Centr)".to_string(),
        node_count: full_graph.nodes.len(),
        edge_count: full_graph.edges.len(),
        duration: full_gen_dur,
        iterations: 1,
        avg_tick_ms: full_gen_dur.as_secs_f64() * 1000.0,
    }
    .print();

    // 2. Local Graph Projection (2 hops from Note 0)
    let active_note = &paths[0];
    let t0 = Instant::now();
    let local_graph = link_graph.to_local_graph_data_with_options(
        active_note,
        &paths,
        &titles,
        &note_tags,
        &filter_options,
        2, // depth 2
    );
    let local_gen_dur = t0.elapsed();
    BenchMetric {
        name: "2. Local 2-Hop Projection".to_string(),
        node_count: local_graph.nodes.len(),
        edge_count: local_graph.edges.len(),
        duration: local_gen_dur,
        iterations: 1,
        avg_tick_ms: local_gen_dur.as_secs_f64() * 1000.0,
    }
    .print();

    // 3. Physics Simulation on Full Graph (Barnes-Hut step latency)
    let forces = GraphForcesSettings::default();
    let (mut sim_nodes, sim_edges) =
        init_simulation_with_forces(&full_graph, 1200.0, 800.0, &forces);

    let steps = if node_count > 5000 { 5 } else { 15 };
    let mut total_step_dur = Duration::ZERO;
    for _ in 0..steps {
        let t0 = Instant::now();
        step_simulation_with_forces(
            &mut sim_nodes,
            &sim_edges,
            (600.0, 400.0),
            None,
            0.5,
            &forces,
        );
        total_step_dur += t0.elapsed();
    }
    let avg_step_ms = (total_step_dur.as_secs_f64() / steps as f64) * 1000.0;
    BenchMetric {
        name: format!("3. Physics Step (x{})", steps),
        node_count: sim_nodes.len(),
        edge_count: sim_edges.len(),
        duration: total_step_dur,
        iterations: steps,
        avg_tick_ms: avg_step_ms,
    }
    .print();

    // 4. UI Frame-Budget Evaluation
    let est_fps = if avg_step_ms > 0.0 {
        1000.0 / avg_step_ms
    } else {
        1000.0
    };
    println!(
        "   └─ Physics Compute: {:.2} ms/frame (~{:.1} FPS theoretical physics cap)",
        avg_step_ms, est_fps
    );
    if full_graph.nodes.len() > 500 {
        println!(
            "   └─ WARNING: {} SVG DOM elements would exceed 60 FPS rendering budget!",
            full_graph.nodes.len() * 3 + full_graph.edges.len()
        );
    }
}

fn main() {
    println!("\n╔═════════════════════════════════════════════════════════════════════════════╗");
    println!("║       NODERA G0: GRAPH SCALE & RENDERING BOTTLENECK BENCHMARK               ║");
    println!("╚═════════════════════════════════════════════════════════════════════════════╝");

    let tiers = [100, 500, 1_000, 2_000, 5_000, 10_000];
    for tier in tiers {
        benchmark_tier(tier);
    }

    println!("\n==========================================================================");
    println!("  G0 BENCHMARK SUMMARY & KNEE-POINT ANALYSIS COMPLETE");
    println!("==========================================================================");
}
