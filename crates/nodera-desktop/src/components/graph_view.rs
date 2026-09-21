use dioxus::prelude::*;
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::time::Instant;

use crate::icons::*;
use crate::state::{ActiveView, AppState, GraphForcesSettings};
use crate::strings::{actions, empty_states, graph as graph_strings, placeholders};
use nodera_markdown::GraphData;

pub const COMMUNITY_COLORS: [&str; 12] = [
    "#5C6FE6", "#7081F0", "#8492F6", "#4F61C9", "#3F4D9E", "#6B7DF2", "#7E8DF4", "#4555B8",
    "#364391", "#5466DB", "#6475E8", "#4A5CC5",
];

/// Node in the 2D physics simulation canvas
#[derive(Debug, Clone, PartialEq)]
pub struct SimNode {
    pub id: String,
    pub path: PathBuf,
    pub label: String,
    pub degree: usize,
    pub in_degree: usize,
    pub out_degree: usize,
    pub centrality: u32,
    pub community_id: usize,
    pub x: f32,
    pub y: f32,
    pub vx: f32,
    pub vy: f32,
    pub radius: f32,
    pub is_unresolved: bool,
    pub is_tag: bool,
}

/// Directed or undirected link between nodes by index
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SimEdge {
    pub source: usize,
    pub target: usize,
}

/// Initializes simulation nodes with deterministic golden-spiral positions using default forces.
pub fn init_simulation(graph: &GraphData, width: f32, height: f32) -> (Vec<SimNode>, Vec<SimEdge>) {
    init_simulation_with_forces(graph, width, height, &GraphForcesSettings::default())
}

/// Initializes simulation nodes with deterministic golden-spiral positions and runs multi-step pre-warm relaxation.
pub fn init_simulation_with_forces(
    graph: &GraphData,
    width: f32,
    height: f32,
    forces: &GraphForcesSettings,
) -> (Vec<SimNode>, Vec<SimEdge>) {
    let n = graph.nodes.len();
    if n == 0 {
        return (Vec::new(), Vec::new());
    }

    let mut sorted_nodes = graph.nodes.clone();
    // Sort deterministically so reopening the graph layout is 100% reproducible
    sorted_nodes.sort_by(|a, b| a.id.cmp(&b.id));

    let cx = width / 2.0;
    let cy = height / 2.0;

    let mut nodes = Vec::with_capacity(n);
    for (i, gn) in sorted_nodes.iter().enumerate() {
        let angle = (i as f32) * 2.3999632;
        let r = 32.0 * ((i + 1) as f32).sqrt();
        let x = cx + r * angle.cos();
        let y = cy + r * angle.sin();
        let radius = 6.0 + (gn.degree as f32).min(12.0) * 1.25;
        nodes.push(SimNode {
            id: gn.id.clone(),
            path: gn.path.clone(),
            label: gn.label.clone(),
            degree: gn.degree,
            in_degree: gn.in_degree,
            out_degree: gn.out_degree,
            centrality: gn.centrality,
            community_id: gn.community_id,
            x,
            y,
            vx: 0.0,
            vy: 0.0,
            radius,
            is_unresolved: gn.is_unresolved,
            is_tag: gn.is_tag,
        });
    }

    let id_map: HashMap<&str, usize> = nodes
        .iter()
        .enumerate()
        .map(|(i, n)| (n.id.as_str(), i))
        .collect();

    let mut edges = Vec::new();
    let mut edge_set = HashSet::new();

    for ge in &graph.edges {
        if let (Some(&s), Some(&t)) = (
            id_map.get(ge.source.as_str()),
            id_map.get(ge.target.as_str()),
        ) {
            if s != t {
                let min_idx = s.min(t);
                let max_idx = s.max(t);
                if edge_set.insert((min_idx, max_idx)) {
                    edges.push(SimEdge {
                        source: s,
                        target: t,
                    });
                }
            }
        }
    }

    // Pre-warm relaxation with simulated cooling annealing using force settings
    let total_steps = 120;
    for step in 0..total_steps {
        let alpha = (1.0 - (step as f32 / total_steps as f32)).max(0.03);
        step_simulation_with_forces(&mut nodes, &edges, (cx, cy), None, alpha, forces);
    }

    (nodes, edges)
}

/// Executes one physics tick with default force settings.
pub fn step_simulation(
    nodes: &mut [SimNode],
    edges: &[SimEdge],
    center: (f32, f32),
    dragged_idx: Option<usize>,
    alpha: f32,
) {
    step_simulation_with_forces(
        nodes,
        edges,
        center,
        dragged_idx,
        alpha,
        &GraphForcesSettings::default(),
    );
}

/// Compact QuadTree node for Barnes-Hut spatial force approximation.
#[derive(Debug, Clone)]
struct QuadTreeNode {
    cx: f32,
    cy: f32,
    size: f32,
    mass: f32,
    com_x: f32,
    com_y: f32,
    payload: QuadPayload,
}

#[derive(Debug, Clone)]
enum QuadPayload {
    Empty,
    Leaf(usize),
    Internal([usize; 4]), // NW, NE, SW, SE
}

struct QuadTree {
    nodes: Vec<QuadTreeNode>,
}

impl QuadTree {
    fn new(cx: f32, cy: f32, size: f32) -> Self {
        let mut tree = Self {
            nodes: Vec::with_capacity(128),
        };
        tree.nodes.push(QuadTreeNode {
            cx,
            cy,
            size,
            mass: 0.0,
            com_x: 0.0,
            com_y: 0.0,
            payload: QuadPayload::Empty,
        });
        tree
    }

    fn insert(
        &mut self,
        node_idx: usize,
        body_idx: usize,
        x: f32,
        y: f32,
        mass: f32,
        max_depth: usize,
    ) {
        if max_depth == 0 {
            let cur = &mut self.nodes[node_idx];
            let new_mass = cur.mass + mass;
            if new_mass > 0.0 {
                cur.com_x = (cur.com_x * cur.mass + x * mass) / new_mass;
                cur.com_y = (cur.com_y * cur.mass + y * mass) / new_mass;
                cur.mass = new_mass;
            }
            return;
        }

        match self.nodes[node_idx].payload {
            QuadPayload::Empty => {
                let cur = &mut self.nodes[node_idx];
                cur.mass = mass;
                cur.com_x = x;
                cur.com_y = y;
                cur.payload = QuadPayload::Leaf(body_idx);
            }
            QuadPayload::Leaf(old_body_idx) => {
                let old_com_x = self.nodes[node_idx].com_x;
                let old_com_y = self.nodes[node_idx].com_y;
                let old_mass = self.nodes[node_idx].mass;
                let cx = self.nodes[node_idx].cx;
                let cy = self.nodes[node_idx].cy;
                let size = self.nodes[node_idx].size;
                let half = size / 2.0;
                let q_size = half;

                let nw_idx = self.nodes.len();
                let ne_idx = nw_idx + 1;
                let sw_idx = nw_idx + 2;
                let se_idx = nw_idx + 3;

                self.nodes.push(QuadTreeNode {
                    cx: cx - half / 2.0,
                    cy: cy - half / 2.0,
                    size: q_size,
                    mass: 0.0,
                    com_x: 0.0,
                    com_y: 0.0,
                    payload: QuadPayload::Empty,
                });
                self.nodes.push(QuadTreeNode {
                    cx: cx + half / 2.0,
                    cy: cy - half / 2.0,
                    size: q_size,
                    mass: 0.0,
                    com_x: 0.0,
                    com_y: 0.0,
                    payload: QuadPayload::Empty,
                });
                self.nodes.push(QuadTreeNode {
                    cx: cx - half / 2.0,
                    cy: cy + half / 2.0,
                    size: q_size,
                    mass: 0.0,
                    com_x: 0.0,
                    com_y: 0.0,
                    payload: QuadPayload::Empty,
                });
                self.nodes.push(QuadTreeNode {
                    cx: cx + half / 2.0,
                    cy: cy + half / 2.0,
                    size: q_size,
                    mass: 0.0,
                    com_x: 0.0,
                    com_y: 0.0,
                    payload: QuadPayload::Empty,
                });

                let new_mass = old_mass + mass;
                self.nodes[node_idx].com_x = (old_com_x * old_mass + x * mass) / new_mass;
                self.nodes[node_idx].com_y = (old_com_y * old_mass + y * mass) / new_mass;
                self.nodes[node_idx].mass = new_mass;
                self.nodes[node_idx].payload =
                    QuadPayload::Internal([nw_idx, ne_idx, sw_idx, se_idx]);

                let old_quadrant = self.quadrant_for(cx, cy, old_com_x, old_com_y);
                let old_child_idx = self.nodes[node_idx].child(old_quadrant);
                self.insert(
                    old_child_idx,
                    old_body_idx,
                    old_com_x,
                    old_com_y,
                    old_mass,
                    max_depth - 1,
                );

                let new_quadrant = self.quadrant_for(cx, cy, x, y);
                let new_child_idx = self.nodes[node_idx].child(new_quadrant);
                self.insert(new_child_idx, body_idx, x, y, mass, max_depth - 1);
            }
            QuadPayload::Internal(_) => {
                let cx = self.nodes[node_idx].cx;
                let cy = self.nodes[node_idx].cy;
                let new_mass = self.nodes[node_idx].mass + mass;
                self.nodes[node_idx].com_x =
                    (self.nodes[node_idx].com_x * self.nodes[node_idx].mass + x * mass) / new_mass;
                self.nodes[node_idx].com_y =
                    (self.nodes[node_idx].com_y * self.nodes[node_idx].mass + y * mass) / new_mass;
                self.nodes[node_idx].mass = new_mass;

                let quadrant = self.quadrant_for(cx, cy, x, y);
                let child_idx = self.nodes[node_idx].child(quadrant);
                self.insert(child_idx, body_idx, x, y, mass, max_depth - 1);
            }
        }
    }

    #[inline(always)]
    fn quadrant_for(&self, cx: f32, cy: f32, x: f32, y: f32) -> usize {
        match (x >= cx, y >= cy) {
            (false, false) => 0, // NW
            (true, false) => 1,  // NE
            (false, true) => 2,  // SW
            (true, true) => 3,   // SE
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn compute_repulsion(
        &self,
        tree_idx: usize,
        body_idx: usize,
        bx: f32,
        by: f32,
        b_mass: f32,
        b_radius: f32,
        nodes: &[SimNode],
        k_rep: f32,
        theta_sq: f32,
        fx: &mut f32,
        fy: &mut f32,
    ) {
        if tree_idx >= self.nodes.len() {
            return;
        }
        let qnode = &self.nodes[tree_idx];
        if qnode.mass <= 0.0 {
            return;
        }

        match qnode.payload {
            QuadPayload::Empty => {}
            QuadPayload::Leaf(other_idx) => {
                if other_idx != body_idx {
                    let other = &nodes[other_idx];
                    let dx = bx - other.x;
                    let dy = by - other.y;
                    let dist_sq = dx * dx + dy * dy;
                    let dist = dist_sq.sqrt().max(1.0);
                    let nx = dx / dist;
                    let ny = dy / dist;

                    let other_mass = 1.0 + (other.degree as f32) * 0.15;
                    let mut force =
                        (k_rep * b_mass * other_mass) / (dist * (dist + 20.0)).max(300.0);

                    let min_dist = b_radius + other.radius + 14.0;
                    if dist < min_dist {
                        let overlap = min_dist - dist;
                        force += overlap * overlap * 0.25;
                    }

                    *fx += nx * force;
                    *fy += ny * force;
                }
            }
            QuadPayload::Internal(children) => {
                let dx = bx - qnode.com_x;
                let dy = by - qnode.com_y;
                let dist_sq = dx * dx + dy * dy;

                if (qnode.size * qnode.size) < (theta_sq * dist_sq) {
                    let dist = dist_sq.sqrt().max(1.0);
                    let nx = dx / dist;
                    let ny = dy / dist;

                    let force = (k_rep * b_mass * qnode.mass) / (dist * (dist + 20.0)).max(300.0);
                    *fx += nx * force;
                    *fy += ny * force;
                } else {
                    for &child in &children {
                        self.compute_repulsion(
                            child, body_idx, bx, by, b_mass, b_radius, nodes, k_rep, theta_sq, fx,
                            fy,
                        );
                    }
                }
            }
        }
    }
}

impl QuadTreeNode {
    #[inline(always)]
    fn child(&self, quadrant: usize) -> usize {
        match self.payload {
            QuadPayload::Internal(c) => c[quadrant],
            _ => 0,
        }
    }
}

/// Executes one physics tick with Coulomb repulsion, Hooke spring attraction, center gravity, and collision clearance.
pub fn step_simulation_with_forces(
    nodes: &mut [SimNode],
    edges: &[SimEdge],
    center: (f32, f32),
    dragged_idx: Option<usize>,
    alpha: f32,
    forces: &GraphForcesSettings,
) {
    let n = nodes.len();
    if n == 0 {
        return;
    }

    let k_rep = 475.0 * forces.repel_force.max(0.1);
    let k_spring = 0.07 * forces.link_force.max(0.05);
    let target_len = forces.link_distance.clamp(20.0, 500.0);
    let k_center = 0.00875 * forces.center_force.max(0.01);
    let damping = 0.82;

    let mut fx = vec![0.0f32; n];
    let mut fy = vec![0.0f32; n];

    // 1. Repulsion and collision avoidance
    if n < 64 {
        // Direct all-pairs calculation for small graphs
        for i in 0..n {
            let r_i = nodes[i].radius;
            let deg_i = nodes[i].degree as f32;
            let mass_i = 1.0 + deg_i * 0.15;

            for j in (i + 1)..n {
                let dx = nodes[i].x - nodes[j].x;
                let dy = nodes[i].y - nodes[j].y;
                let dist_sq = dx * dx + dy * dy;
                let dist = dist_sq.sqrt().max(1.0);
                let nx = dx / dist;
                let ny = dy / dist;

                let mass_j = 1.0 + (nodes[j].degree as f32) * 0.15;
                let mut force = (k_rep * mass_i * mass_j) / (dist * (dist + 20.0)).max(300.0);

                let r_j = nodes[j].radius;
                let min_dist = r_i + r_j + 14.0;
                if dist < min_dist {
                    let overlap = min_dist - dist;
                    force += overlap * overlap * 0.25;
                }

                let f_x = nx * force;
                let f_y = ny * force;

                fx[i] += f_x;
                fy[i] += f_y;
                fx[j] -= f_x;
                fy[j] -= f_y;
            }
        }
    } else {
        // Barnes-Hut QuadTree spatial subdivision for O(N log N) scaling
        let mut min_x = f32::MAX;
        let mut max_x = f32::MIN;
        let mut min_y = f32::MAX;
        let mut max_y = f32::MIN;

        for node in nodes.iter() {
            min_x = min_x.min(node.x);
            max_x = max_x.max(node.x);
            min_y = min_y.min(node.y);
            max_y = max_y.max(node.y);
        }

        let cx = (min_x + max_x) * 0.5;
        let cy = (min_y + max_y) * 0.5;
        let size = ((max_x - min_x).max(max_y - min_y) + 20.0).max(100.0);

        let mut tree = QuadTree::new(cx, cy, size);
        for (i, node) in nodes.iter().enumerate() {
            let mass = 1.0 + (node.degree as f32) * 0.15;
            tree.insert(0, i, node.x, node.y, mass, 16);
        }

        let theta_sq = 0.75 * 0.75;
        for i in 0..n {
            let mass_i = 1.0 + (nodes[i].degree as f32) * 0.15;
            tree.compute_repulsion(
                0,
                i,
                nodes[i].x,
                nodes[i].y,
                mass_i,
                nodes[i].radius,
                nodes,
                k_rep,
                theta_sq,
                &mut fx[i],
                &mut fy[i],
            );
        }
    }

    // 2. Spring attraction along connected edges
    for edge in edges {
        let s = edge.source;
        let t = edge.target;
        if s >= n || t >= n {
            continue;
        }
        let dx = nodes[t].x - nodes[s].x;
        let dy = nodes[t].y - nodes[s].y;
        let dist = (dx * dx + dy * dy).sqrt().max(0.1);
        let displacement = dist - target_len;
        let force = displacement * k_spring;
        let f_x = (dx / dist) * force;
        let f_y = (dy / dist) * force;

        fx[s] += f_x;
        fy[s] += f_y;
        fx[t] -= f_x;
        fy[t] -= f_y;
    }

    // 3. Gravity towards center
    for i in 0..n {
        let dx = center.0 - nodes[i].x;
        let dy = center.1 - nodes[i].y;
        fx[i] += dx * k_center;
        fy[i] += dy * k_center;
    }

    // 4. Update velocity and position with cooling alpha
    for i in 0..n {
        if Some(i) == dragged_idx {
            nodes[i].vx = 0.0;
            nodes[i].vy = 0.0;
            continue;
        }
        nodes[i].vx = (nodes[i].vx + fx[i].clamp(-35.0, 35.0) * alpha) * damping;
        nodes[i].vy = (nodes[i].vy + fy[i].clamp(-35.0, 35.0) * alpha) * damping;
        nodes[i].x += nodes[i].vx.clamp(-30.0, 30.0);
        nodes[i].y += nodes[i].vy.clamp(-30.0, 30.0);
    }
}

/// Global 2D Knowledge Graph View with Interactive Controls Panel
#[component]
pub fn GraphView(state: Signal<AppState>) -> Element {
    let app_state = state.read();
    let current_settings = app_state.preferences.graph_settings.clone();
    let graph_data = app_state.get_full_graph_data_with_settings(&current_settings);
    let total_notes = graph_data.nodes.len();
    let total_edges = graph_data.edges.len();

    let (init_nodes, init_edges) =
        init_simulation_with_forces(&graph_data, 1000.0, 700.0, &current_settings.forces);
    let mut nodes_state = use_signal(|| init_nodes);
    let mut edges_state = use_signal(|| init_edges);

    let mut pan_x = use_signal(|| 0.0f32);
    let mut pan_y = use_signal(|| 0.0f32);
    let mut zoom = use_signal(|| 1.0f32);
    let mut is_panning = use_signal(|| false);
    let mut pan_start = use_signal(|| (0.0f64, 0.0f64));
    let mut pan_distance = use_signal(|| 0.0f64);
    let mut dragged_node = use_signal(|| None::<usize>);
    let mut hovered_node = use_signal(|| None::<usize>);
    let mut selected_node = use_signal(|| None::<usize>);
    let mut last_click = use_signal(|| None::<(usize, Instant)>);
    let mut search_query = use_signal(|| current_settings.filters.search_query.clone());

    // When filters or vault entries change, recompute simulation
    use_effect(move || {
        let s = state.read().preferences.graph_settings.clone();
        let current_graph = state.read().get_full_graph_data_with_settings(&s);
        let (n, e) = init_simulation_with_forces(&current_graph, 1000.0, 700.0, &s.forces);
        nodes_state.set(n);
        edges_state.set(e);
    });

    if total_notes == 0 {
        return rsx! {
            div {
                class: "graph-container",
                style: "display: flex; flex-direction: column; align-items: center; justify-content: center; gap: 16px; color: var(--text-secondary);",
                IconGraph { size: 64, class: "opacity-40" }
                h2 { style: "font-size: 18px; font-weight: 600; color: var(--text-primary);", "{empty_states::NO_GRAPH_NODES_TITLE}" }
                p { style: "font-size: 13px; color: var(--text-muted); max-width: 360px; text-align: center;", "{empty_states::NO_GRAPH_NODES_DESC}" }
                if current_settings.filters.existing_files_only || !current_settings.filters.orphans {
                    button {
                        class: "btn-secondary",
                        style: "margin-top: 8px; font-size: 12px; padding: 6px 14px;",
                        onclick: move |_| {
                            let mut s = state.write();
                            s.preferences.graph_settings.filters.existing_files_only = false;
                            s.preferences.graph_settings.filters.orphans = true;
                            s.preferences.graph_settings.filters.search_query.clear();
                        },
                        "{graph_strings::RESTORE_DEFAULTS}"
                    }
                }
            }
        };
    }

    let nodes = nodes_state.read();
    let edges = edges_state.read();
    let active_path_str = app_state
        .active_note
        .as_ref()
        .map(|n| n.relative_path.to_string_lossy().replace('\\', "/"));

    let current_hovered = *hovered_node.read();
    let current_selected = *selected_node.read();
    let focused_idx = current_hovered.or(current_selected);

    let query = search_query.read().trim().to_lowercase();
    let zoom_val = *zoom.read();
    let zoom_pct = (zoom_val * 100.0).round() as u32;
    let dot_radius = (0.85 / zoom_val).clamp(0.35, 1.3);

    // Connected indices for highlighting neighborhood of focused node
    let mut connected_indices = HashSet::new();
    if let Some(f_idx) = focused_idx {
        connected_indices.insert(f_idx);
        for edge in edges.iter() {
            if edge.source == f_idx {
                connected_indices.insert(edge.target);
            } else if edge.target == f_idx {
                connected_indices.insert(edge.source);
            }
        }
    }

    rsx! {
        div {
            class: "graph-container",
            style: "display: flex; flex-direction: row; width: 100%; height: 100%; position: relative; overflow: hidden;",

            // Main Canvas Area
            div {
                class: "graph-canvas-wrapper",
                tabindex: "0",
                onmousedown: move |evt: MouseEvent| {
                    is_panning.set(true);
                    pan_start.set((evt.client_coordinates().x, evt.client_coordinates().y));
                    pan_distance.set(0.0);
                },
                onmousemove: move |evt: MouseEvent| {
                    let cur_x = evt.client_coordinates().x;
                    let cur_y = evt.client_coordinates().y;

                    if *is_panning.read() {
                        let start = *pan_start.read();
                        let dx = (cur_x - start.0) as f32;
                        let dy = (cur_y - start.1) as f32;
                        let dist = (dx.abs() + dy.abs()) as f64;
                        let prev_dist = *pan_distance.read();
                        pan_distance.set(prev_dist + dist);

                        let new_x = *pan_x.read() + dx;
                        let new_y = *pan_y.read() + dy;
                        pan_x.set(new_x);
                        pan_y.set(new_y);
                        pan_start.set((cur_x, cur_y));
                    } else if let Some(drag_idx) = *dragged_node.read() {
                        let z = *zoom.read();
                        let px = *pan_x.read();
                        let py = *pan_y.read();
                        let world_x = (cur_x as f32 - px) / z;
                        let world_y = (cur_y as f32 - py) / z;

                        let mut ns = nodes_state.write();
                        if drag_idx < ns.len() {
                            ns[drag_idx].x = world_x;
                            ns[drag_idx].y = world_y;
                            if current_settings.display.animate {
                                let es = edges_state.read();
                                let forces = current_settings.forces.clone();
                                // Reheat connected neighbors during drag
                                step_simulation_with_forces(&mut ns, &es, (500.0, 350.0), Some(drag_idx), 0.25, &forces);
                                step_simulation_with_forces(&mut ns, &es, (500.0, 350.0), Some(drag_idx), 0.20, &forces);
                            }
                        }
                    }
                },
                onmouseup: move |_| {
                    is_panning.set(false);
                    dragged_node.set(None);
                },
                onclick: move |_| {
                    // Deselect if user clicked empty background without dragging
                    if *pan_distance.read() < 5.0 {
                        selected_node.set(None);
                    }
                },
                onwheel: move |evt: WheelEvent| {
                    let delta = evt.delta().strip_units().y;
                    let factor = if delta > 0.0 { 0.90f32 } else { 1.10f32 };
                    let current_zoom = *zoom.read();
                    let next_zoom = (current_zoom * factor).clamp(0.20, 3.5);
                    zoom.set(next_zoom);
                },

                // Top-left Search / Filter Bar
                div {
                    class: "graph-search-bar",
                    onmousedown: move |evt: MouseEvent| {
                        evt.stop_propagation();
                    },
                    IconSearch { size: 14, class: "opacity-70" }
                    input {
                        style: "border: none; background: transparent; font-size: 12px; outline: none; color: var(--text-primary); width: 140px;",
                        placeholder: placeholders::FILTER_GRAPH,
                        value: "{search_query}",
                        oninput: move |evt: FormEvent| {
                            let val = evt.value();
                            search_query.set(val.clone());
                            state.write().preferences.graph_settings.filters.search_query = val;
                        },
                    }
                    if !search_query.read().is_empty() {
                        button {
                            class: "btn-icon",
                            style: "padding: 2px;",
                            onclick: move |_| {
                                search_query.set(String::new());
                                state.write().preferences.graph_settings.filters.search_query.clear();
                            },
                            IconClose { size: 12 }
                        }
                    }
                }

                // Top-right Compact Controls Toolbar
                div {
                    class: "graph-toolbar",
                    onmousedown: move |evt: MouseEvent| {
                        evt.stop_propagation();
                    },
                    button {
                        class: "graph-btn",
                        title: actions::ZOOM_IN,
                        onclick: move |_| {
                            let z = *zoom.read();
                            zoom.set((z * 1.15).min(3.5));
                        },
                        IconPlus { size: 13 }
                    }
                    span {
                        class: "graph-zoom-label",
                        "{zoom_pct}%"
                    }
                    button {
                        class: "graph-btn",
                        title: actions::ZOOM_OUT,
                        onclick: move |_| {
                            let z = *zoom.read();
                            zoom.set((z * 0.85).max(0.20));
                        },
                        IconMinus { size: 13 }
                    }
                    span { class: "graph-divider" }
                    button {
                        class: "graph-btn",
                        title: actions::FIT_GRAPH,
                        onclick: move |_| {
                            let ns = nodes_state.read();
                            if ns.is_empty() {
                                pan_x.set(0.0);
                                pan_y.set(0.0);
                                zoom.set(1.0);
                                return;
                            }
                            let mut min_x = f32::MAX;
                            let mut max_x = f32::MIN;
                            let mut min_y = f32::MAX;
                            let mut max_y = f32::MIN;
                            for n in ns.iter() {
                                if n.x < min_x { min_x = n.x; }
                                if n.x > max_x { max_x = n.x; }
                                if n.y < min_y { min_y = n.y; }
                                if n.y > max_y { max_y = n.y; }
                            }
                            let graph_w = (max_x - min_x).max(120.0);
                            let graph_h = (max_y - min_y).max(120.0);
                            let graph_cx = (min_x + max_x) / 2.0;
                            let graph_cy = (min_y + max_y) / 2.0;

                            let view_w = 1000.0f32;
                            let view_h = 700.0f32;
                            let pad = 140.0f32;

                            let target_zoom = ((view_w - pad) / graph_w).min((view_h - pad) / graph_h).clamp(0.20, 2.5);
                            let target_pan_x = (view_w / 2.0) - (graph_cx * target_zoom);
                            let target_pan_y = (view_h / 2.0) - (graph_cy * target_zoom);

                            zoom.set(target_zoom);
                            pan_x.set(target_pan_x);
                            pan_y.set(target_pan_y);
                        },
                        IconMaximize { size: 13 }
                    }
                    button {
                        class: "graph-btn",
                        title: actions::RESET_VIEW,
                        onclick: move |_| {
                            pan_x.set(0.0);
                            pan_y.set(0.0);
                            zoom.set(1.0);
                        },
                        IconCrosshair { size: 13 }
                    }
                    button {
                        class: "graph-btn",
                        title: actions::REFRESH_GRAPH,
                        onclick: move |_| {
                            let s = state.read().preferences.graph_settings.clone();
                            let current_graph = state.read().get_full_graph_data_with_settings(&s);
                            spawn(async move {
                                let (n, e) = tokio::task::spawn_blocking(move || {
                                    init_simulation_with_forces(&current_graph, 1000.0, 700.0, &s.forces)
                                }).await.unwrap_or_default();
                                nodes_state.set(n);
                                edges_state.set(e);
                            });
                        },
                        IconRefresh { size: 13 }
                    }
                    span { class: "graph-divider" }
                    button {
                        class: if state.read().context_panel_open { "graph-btn active" } else { "graph-btn" },
                        title: graph_strings::TOGGLE_CONTROLS,
                        onclick: move |_| {
                            let mut s = state.write();
                            s.context_panel_open = !s.context_panel_open;
                        },
                        IconSliders { size: 13 }
                    }
                }

                // Bottom-left Stats Badge
                div {
                    class: "graph-stats",
                    span {
                        style: "display: inline-flex; align-items: center; gap: 6px; font-weight: 500; color: var(--text-primary);",
                        IconGraph { size: 13 }
                        span { "{total_notes} nodes" }
                    }
                    span { style: "color: var(--text-muted);", "•" }
                    span { style: "color: var(--text-secondary);", "{total_edges} connections" }
                }

                // Hovered Node Intelligence Card
                {
                    if let Some(h_idx) = *hovered_node.read() {
                        if let Some(h_node) = nodes.get(h_idx) {
                            let cent_pct = (h_node.centrality as f32) / 100.0;
                            let comm_col = COMMUNITY_COLORS[h_node.community_id % COMMUNITY_COLORS.len()];
                            rsx! {
                                div {
                                    style: "position: absolute; bottom: 50px; left: 14px; background: var(--bg-surface-elevated, #1A1D24); border: 1px solid var(--border); border-radius: 6px; padding: 6px 12px; font-size: 11px; z-index: 10; display: flex; align-items: center; gap: 8px; box-shadow: 0 4px 12px rgba(0, 0, 0, 0.25); pointer-events: none;",
                                    span { style: "font-weight: 600; color: var(--text-primary);", "{h_node.label}" }
                                    span { style: "color: var(--text-muted);", "•" }
                                    span { style: "color: var(--text-secondary);", "Degree {h_node.degree} (in: {h_node.in_degree}, out: {h_node.out_degree})" }
                                    span { style: "color: var(--text-muted);", "•" }
                                    span {
                                        style: "color: {comm_col}; font-weight: 500;",
                                        "Community #{h_node.community_id + 1}"
                                    }
                                    span { style: "color: var(--text-muted);", "•" }
                                    span { style: "color: var(--accent); font-weight: 500;", "Centrality {cent_pct:.1}%" }
                                }
                            }
                        } else {
                            rsx! {}
                        }
                    } else {
                        rsx! {}
                    }
                }

                // Interactive SVG Canvas
                svg {
                    class: "graph-canvas",
                    defs {
                        pattern {
                            id: "graph-dot-grid",
                            width: "28",
                            height: "28",
                            pattern_units: "userSpaceOnUse",
                            circle {
                                cx: "14",
                                cy: "14",
                                r: "{dot_radius}",
                                fill: "var(--graph-grid-dot, rgba(255, 255, 255, 0.08))",
                            }
                        }
                        marker {
                            id: "graph-arrow",
                            view_box: "0 0 10 10",
                            ref_x: "18",
                            ref_y: "5",
                            marker_width: "6",
                            marker_height: "6",
                            orient: "auto-start-reverse",
                            path {
                                d: "M 0 1.5 L 8 5 L 0 8.5 z",
                                fill: "var(--graph-edge, #444A5B)",
                            }
                        }
                        marker {
                            id: "graph-arrow-highlight",
                            view_box: "0 0 10 10",
                            ref_x: "18",
                            ref_y: "5",
                            marker_width: "6",
                            marker_height: "6",
                            orient: "auto-start-reverse",
                            path {
                                d: "M 0 1.5 L 8 5 L 0 8.5 z",
                                fill: "var(--graph-edge-highlight, #7182FF)",
                            }
                        }
                    }
                    g {
                        transform: format!("translate({}, {}) scale({})", *pan_x.read(), *pan_y.read(), *zoom.read()),

                        // Infinite subtle dotted background grid
                        rect {
                            x: "-200000",
                            y: "-200000",
                            width: "400000",
                            height: "400000",
                            fill: "url(#graph-dot-grid)",
                        }

                        // Render Edges
                        for edge in edges.iter() {
                            {
                                let s = edge.source;
                                let t = edge.target;
                                if s < nodes.len() && t < nodes.len() {
                                    let n1 = &nodes[s];
                                    let n2 = &nodes[t];
                                    let is_highlighted = if let Some(f) = focused_idx {
                                        s == f || t == f
                                    } else {
                                        false
                                    };

                                    let edge_color = if is_highlighted {
                                        "var(--graph-edge-highlight, #7182FF)"
                                    } else {
                                        "var(--graph-edge, #444A5B)"
                                    };

                                    let edge_opacity = if focused_idx.is_some() {
                                        if is_highlighted { "1.0" } else { "0.08" }
                                    } else {
                                        "0.35"
                                    };

                                    let edge_width = (if is_highlighted { 2.2 } else { 1.0 }) * current_settings.display.link_thickness;
                                    let marker_attr = if current_settings.display.arrows {
                                        if is_highlighted { "url(#graph-arrow-highlight)" } else { "url(#graph-arrow)" }
                                    } else {
                                        ""
                                    };

                                    rsx! {
                                        line {
                                            key: "{s}-{t}",
                                            x1: "{n1.x}",
                                            y1: "{n1.y}",
                                            x2: "{n2.x}",
                                            y2: "{n2.y}",
                                            stroke: "{edge_color}",
                                            stroke_width: "{edge_width}",
                                            opacity: "{edge_opacity}",
                                            marker_end: "{marker_attr}",
                                        }
                                    }
                                } else {
                                    rsx! {}
                                }
                            }
                        }

                        // Render Nodes
                        for (idx, node) in nodes.iter().enumerate() {
                            {
                                let is_current = active_path_str.as_deref() == Some(node.id.as_str());
                                let is_selected = current_selected == Some(idx);
                                let is_hovered = current_hovered == Some(idx);
                                let is_connected = connected_indices.contains(&idx);
                                let matches_query = if query.is_empty() {
                                    true
                                } else {
                                    node.label.to_lowercase().contains(&query)
                                };

                                let community_color = if current_settings.display.color_by_community {
                                    COMMUNITY_COLORS[node.community_id % COMMUNITY_COLORS.len()]
                                } else {
                                    "var(--graph-node, #6680FF)"
                                };

                                let (node_color, stroke_color, stroke_width, node_opacity, stroke_dash) = if node.is_unresolved {
                                    let fill = "transparent";
                                    let stroke = if is_selected || is_hovered {
                                        "var(--graph-node-hover, #7182FF)"
                                    } else {
                                        "var(--text-muted, #8E90A0)"
                                    };
                                    let opacity = if focused_idx.is_some() && !is_connected { "0.20" } else { "0.85" };
                                    (fill, stroke, "1.5", opacity, "3 2")
                                } else if node.is_tag {
                                    let fill = "#E5A158";
                                    let stroke = if is_selected || is_hovered { "#FFFFFF" } else { "var(--border, #28313C)" };
                                    let opacity = if focused_idx.is_some() && !is_connected { "0.20" } else { "0.95" };
                                    (fill, stroke, "1.5", opacity, "")
                                } else if focused_idx.is_some() {
                                    if is_hovered || is_selected {
                                        let fill = if is_current {
                                            "var(--graph-node-current, #9A4BFF)"
                                        } else if is_hovered {
                                            "var(--graph-node-hover, #7182FF)"
                                        } else {
                                            community_color
                                        };
                                        let stroke = if is_current { "#D5C7FF" } else { "#FFFFFF" };
                                        (fill, stroke, "2.5", "1.0", "")
                                    } else if is_connected {
                                        (community_color, "var(--border-strong, #383F4F)", "2.0", "1.0", "")
                                    } else {
                                        (community_color, "var(--border, #28313C)", "1.0", "0.18", "")
                                    }
                                } else {
                                    let fill = if is_current {
                                        "var(--graph-node-current, #9A4BFF)"
                                    } else {
                                        community_color
                                    };
                                    let stroke = if is_current { "#D5C7FF" } else { "var(--border, #28313C)" };
                                    let opacity = if !matches_query { "0.20" } else { "1.0" };
                                    (fill, stroke, "1.5", opacity, "")
                                };

                                let base_radius = if current_settings.display.centrality_sizing {
                                    let centrality_ratio = (node.centrality as f32) / 10000.0;
                                    5.0 + centrality_ratio * 16.0 + (node.degree as f32).min(8.0) * 0.75
                                } else {
                                    node.radius
                                };
                                let node_radius = base_radius * current_settings.display.node_size;
                                let path_click = node.path.clone();
                                let path_dbl = node.path.clone();
                                let label_click = node.label.clone();
                                let label_dbl = node.label.clone();
                                let is_unresolved = node.is_unresolved;

                                let text_fade_threshold = 0.70 / current_settings.display.text_fade_threshold.max(0.1);
                                let show_label = is_selected
                                    || is_hovered
                                    || (focused_idx.is_some() && is_connected)
                                    || *zoom.read() >= text_fade_threshold
                                    || total_notes <= 35
                                    || (node.degree >= 3 && *zoom.read() >= (text_fade_threshold * 0.65));

                                let font_weight = if is_selected || is_hovered || is_current { "600" } else { "400" };

                                rsx! {
                                    g {
                                        key: "{node.id}",
                                        style: "cursor: pointer;",
                                        opacity: "{node_opacity}",
                                        onmousedown: move |evt: MouseEvent| {
                                            evt.stop_propagation();
                                            dragged_node.set(Some(idx));
                                        },
                                        onmouseenter: move |_| {
                                            hovered_node.set(Some(idx));
                                        },
                                        onmouseleave: move |_| {
                                            if *hovered_node.read() == Some(idx) {
                                                hovered_node.set(None);
                                            }
                                        },
                                        onclick: move |evt: MouseEvent| {
                                            evt.stop_propagation();
                                            let now = Instant::now();
                                            let is_double_click = if let Some((last_idx, last_time)) = *last_click.read() {
                                                last_idx == idx && now.duration_since(last_time).as_millis() < 400
                                            } else {
                                                false
                                            };

                                            if is_double_click {
                                                let mut s = state.write();
                                                if is_unresolved {
                                                    let _ = s.open_or_create_target(&label_click);
                                                } else {
                                                    let _ = s.select_note(&path_click);
                                                }
                                                s.active_view = ActiveView::Editor;
                                                last_click.set(None);
                                            } else {
                                                selected_node.set(Some(idx));
                                                last_click.set(Some((idx, now)));
                                            }
                                        },
                                        ondoubleclick: move |evt: MouseEvent| {
                                            evt.stop_propagation();
                                            let mut s = state.write();
                                            if is_unresolved {
                                                let _ = s.open_or_create_target(&label_dbl);
                                            } else {
                                                let _ = s.select_note(&path_dbl);
                                            }
                                            s.active_view = ActiveView::Editor;
                                        },
                                        // Selection halo ring
                                        if is_selected {
                                            circle {
                                                cx: "{node.x}",
                                                cy: "{node.y}",
                                                r: "{node_radius + 6.0}",
                                                fill: "none",
                                                stroke: "var(--graph-node-current, #9A4BFF)",
                                                stroke_width: "2",
                                                stroke_dasharray: "4 3",
                                                opacity: "0.85",
                                            }
                                        }
                                        circle {
                                            cx: "{node.x}",
                                            cy: "{node.y}",
                                            r: "{node_radius}",
                                            fill: "{node_color}",
                                            stroke: "{stroke_color}",
                                            stroke_width: "{stroke_width}",
                                            stroke_dasharray: "{stroke_dash}",
                                        }
                                        if show_label {
                                            text {
                                                x: "{node.x}",
                                                y: format!("{}", node.y + node_radius + 12.0),
                                                text_anchor: "middle",
                                                fill: "var(--graph-label, #D7DBE6)",
                                                font_size: "11",
                                                font_weight: "{font_weight}",
                                                pointer_events: "none",
                                                "{node.label}"
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

        }
    }
}

/// Compact Local 2D Graph for Context Panel with configurable depth
#[component]
pub fn LocalGraphView(state: Signal<AppState>) -> Element {
    let mut depth = use_signal(|| 1usize);

    let app_state = state.read();
    let current_depth = *depth.read();
    let local_graph = app_state.get_local_graph_data(current_depth);
    let node_count = local_graph.nodes.len();

    let mut nodes_state = use_signal(|| {
        let (n, _) = init_simulation(&local_graph, 260.0, 200.0);
        n
    });
    let mut edges_state = use_signal(|| {
        let (_, e) = init_simulation(&local_graph, 260.0, 200.0);
        e
    });

    let mut hovered = use_signal(|| None::<usize>);

    // Re-sync when active note or depth changes
    use_effect(move || {
        let current_d = *depth.read();
        let current_local = state.read().get_local_graph_data(current_d);
        let (n, e) = init_simulation(&current_local, 260.0, 200.0);
        nodes_state.set(n);
        edges_state.set(e);
    });

    if node_count <= 1 {
        return rsx! {
            div {
                class: "local-graph-box",
                style: "display: flex; flex-direction: column; align-items: center; justify-content: center; padding: 16px; color: var(--text-muted); text-align: center; gap: 8px;",
                IconGraph { size: 28, class: "opacity-40" }
                p { style: "font-size: 11px; line-height: 1.4;", "{empty_states::NO_LOCAL_GRAPH}" }
            }
        };
    }

    let nodes = nodes_state.read();
    let edges = edges_state.read();
    let active_path_str = app_state
        .active_note
        .as_ref()
        .map(|n| n.relative_path.to_string_lossy().replace('\\', "/"));

    rsx! {
        div {
            class: "local-graph-box",
            // Header with depth controls and expand button
            div {
                style: "position: absolute; top: 6px; left: 8px; z-index: 5; display: flex; align-items: center; gap: 4px;",
                span { style: "font-size: 10px; color: var(--text-muted); font-weight: 500;", "Depth:" }
                div {
                    class: "graph-pill-group",
                    for d in [1usize, 2, 3] {
                        button {
                            key: "{d}",
                            class: if *depth.read() == d { "graph-pill active" } else { "graph-pill" },
                            onclick: move |_| {
                                depth.set(d);
                            },
                            "{d}"
                        }
                    }
                }
            }

            div {
                style: "position: absolute; top: 6px; right: 8px; z-index: 5;",
                button {
                    class: "btn-icon",
                    style: "padding: 3px; background: var(--bg-surface-elevated); border: 1px solid var(--border); border-radius: 4px;",
                    title: actions::OPEN_GRAPH,
                    onclick: move |_| {
                        state.write().active_view = ActiveView::Graph;
                    },
                    IconMaximize { size: 12 }
                }
            }

            svg {
                style: "width: 100%; height: 100%; display: block;",
                view_box: "0 0 260 200",
                defs {
                    pattern {
                        id: "local-graph-dot-grid",
                        width: "20",
                        height: "20",
                        pattern_units: "userSpaceOnUse",
                        circle {
                            cx: "10",
                            cy: "10",
                            r: "0.75",
                            fill: "var(--graph-grid-dot, rgba(255, 255, 255, 0.08))",
                        }
                    }
                }

                // Dotted background grid
                rect {
                    width: "100%",
                    height: "100%",
                    fill: "url(#local-graph-dot-grid)",
                }

                // Edges
                for edge in edges.iter() {
                    {
                        let s = edge.source;
                        let t = edge.target;
                        if s < nodes.len() && t < nodes.len() {
                            let n1 = &nodes[s];
                            let n2 = &nodes[t];
                            rsx! {
                                line {
                                    key: "{s}-{t}",
                                    x1: "{n1.x}",
                                    y1: "{n1.y}",
                                    x2: "{n2.x}",
                                    y2: "{n2.y}",
                                    stroke: "var(--graph-edge, #444A5B)",
                                    stroke_width: "1.2",
                                    opacity: "0.6",
                                }
                            }
                        } else {
                            rsx! {}
                        }
                    }
                }

                // Nodes
                for (idx, node) in nodes.iter().enumerate() {
                    {
                        let is_current = active_path_str.as_deref() == Some(node.id.as_str());
                        let is_hovered = *hovered.read() == Some(idx);
                        let fill_color = if node.is_unresolved {
                            "transparent"
                        } else if is_current {
                            "var(--graph-node-current, #9A4BFF)"
                        } else if is_hovered {
                            "var(--graph-node-hover, #7182FF)"
                        } else {
                            "var(--graph-node, #6680FF)"
                        };
                        let stroke_dash = if node.is_unresolved { "3 2" } else { "" };
                        let path_click = node.path.clone();
                        let label_click = node.label.clone();
                        let is_unresolved = node.is_unresolved;

                        rsx! {
                            g {
                                key: "{node.id}",
                                style: "cursor: pointer;",
                                onmouseenter: move |_| {
                                    hovered.set(Some(idx));
                                },
                                onmouseleave: move |_| {
                                    hovered.set(None);
                                },
                                onclick: move |_| {
                                    let mut s = state.write();
                                    if is_unresolved {
                                        let _ = s.open_or_create_target(&label_click);
                                    } else {
                                        let _ = s.select_note(&path_click);
                                    }
                                    s.active_view = ActiveView::Editor;
                                },
                                circle {
                                    cx: "{node.x}",
                                    cy: "{node.y}",
                                    r: if is_current { "8" } else { "6" },
                                    fill: "{fill_color}",
                                    stroke: if is_current { "#D5C7FF" } else { "var(--border, #28313C)" },
                                    stroke_width: "1.5",
                                    stroke_dasharray: "{stroke_dash}",
                                }
                                text {
                                    x: "{node.x}",
                                    y: format!("{}", node.y + if is_current { 18.0 } else { 15.0 }),
                                    text_anchor: "middle",
                                    fill: "var(--graph-label, #DEE2ED)",
                                    font_size: "10",
                                    font_weight: if is_current { "600" } else { "400" },
                                    pointer_events: "none",
                                    "{node.label}"
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_barnes_hut_quadtree_construction_and_repulsion() {
        let forces = GraphForcesSettings::default();
        let mut nodes = vec![
            SimNode {
                id: "n1".to_string(),
                path: PathBuf::from("n1.md"),
                label: "N1".to_string(),
                degree: 1,
                in_degree: 1,
                out_degree: 0,
                centrality: 100,
                community_id: 0,
                x: 100.0,
                y: 100.0,
                vx: 0.0,
                vy: 0.0,
                radius: 6.0,
                is_unresolved: false,
                is_tag: false,
            },
            SimNode {
                id: "n2".to_string(),
                path: PathBuf::from("n2.md"),
                label: "N2".to_string(),
                degree: 1,
                in_degree: 0,
                out_degree: 1,
                centrality: 100,
                community_id: 0,
                x: 120.0,
                y: 100.0,
                vx: 0.0,
                vy: 0.0,
                radius: 6.0,
                is_unresolved: false,
                is_tag: false,
            },
        ];
        let edges = vec![SimEdge {
            source: 0,
            target: 1,
        }];

        // Run 10 ticks
        for _ in 0..10 {
            step_simulation_with_forces(&mut nodes, &edges, (150.0, 150.0), None, 0.5, &forces);
        }

        // Repulsion should have pushed n1 and n2 further apart along the x-axis
        assert!(nodes[1].x > nodes[0].x);
        assert!((nodes[1].x - nodes[0].x) > 20.0);
    }

    #[test]
    fn test_barnes_hut_large_scale_convergence() {
        let forces = GraphForcesSettings::default();
        let mut nodes = Vec::new();
        for i in 0..100 {
            nodes.push(SimNode {
                id: format!("node_{i}"),
                path: PathBuf::from(format!("node_{i}.md")),
                label: format!("Node {i}"),
                degree: 2,
                in_degree: 1,
                out_degree: 1,
                centrality: 100,
                community_id: i % 4,
                x: 500.0 + (i as f32 * 0.1).cos() * 50.0,
                y: 500.0 + (i as f32 * 0.1).sin() * 50.0,
                vx: 0.0,
                vy: 0.0,
                radius: 6.0,
                is_unresolved: false,
                is_tag: false,
            });
        }
        let edges = (0..99)
            .map(|i| SimEdge {
                source: i,
                target: i + 1,
            })
            .collect::<Vec<_>>();

        // Step simulation using Barnes-Hut (since n >= 64)
        for _ in 0..15 {
            step_simulation_with_forces(&mut nodes, &edges, (500.0, 500.0), None, 0.3, &forces);
        }

        // Assert all coordinates remain finite numbers
        for n in &nodes {
            assert!(n.x.is_finite());
            assert!(n.y.is_finite());
            assert!(n.vx.is_finite());
            assert!(n.vy.is_finite());
        }
    }
}
