use dioxus::prelude::*;
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

use crate::icons::*;
use crate::state::{ActiveView, AppState};
use crate::strings::{actions, empty_states, placeholders};
use nodera_markdown::GraphData;

/// Node in the 2D physics simulation canvas
#[derive(Debug, Clone, PartialEq)]
pub struct SimNode {
    pub id: String,
    pub path: PathBuf,
    pub label: String,
    pub degree: usize,
    pub x: f32,
    pub y: f32,
    pub vx: f32,
    pub vy: f32,
    pub radius: f32,
}

/// Directed link between nodes by index
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SimEdge {
    pub source: usize,
    pub target: usize,
}

/// Initializes simulation nodes with golden-spiral positions and runs pre-warm relaxation.
pub fn init_simulation(graph: &GraphData, width: f32, height: f32) -> (Vec<SimNode>, Vec<SimEdge>) {
    let n = graph.nodes.len();
    let mut nodes = Vec::with_capacity(n);
    let cx = width / 2.0;
    let cy = height / 2.0;

    for (i, gn) in graph.nodes.iter().enumerate() {
        let angle = (i as f32) * 2.3999632;
        let r = 28.0 * (i as f32).sqrt();
        let x = cx + r * angle.cos();
        let y = cy + r * angle.sin();
        let radius = 6.0 + (gn.degree as f32).min(10.0) * 1.5;
        nodes.push(SimNode {
            id: gn.id.clone(),
            path: gn.path.clone(),
            label: gn.label.clone(),
            degree: gn.degree,
            x,
            y,
            vx: 0.0,
            vy: 0.0,
            radius,
        });
    }

    let id_map: HashMap<&str, usize> = nodes
        .iter()
        .enumerate()
        .map(|(i, n)| (n.id.as_str(), i))
        .collect();

    let mut edges = Vec::new();
    for ge in &graph.edges {
        if let (Some(&s), Some(&t)) = (
            id_map.get(ge.source.as_str()),
            id_map.get(ge.target.as_str()),
        ) {
            if s != t {
                edges.push(SimEdge {
                    source: s,
                    target: t,
                });
            }
        }
    }

    // Pre-warm relaxation so graph is aesthetically spaced out immediately
    for _ in 0..75 {
        step_simulation(&mut nodes, &edges, (cx, cy), None);
    }

    (nodes, edges)
}

/// Executes one physics tick with Coulomb repulsion, Hooke spring attraction, and center gravity.
pub fn step_simulation(
    nodes: &mut [SimNode],
    edges: &[SimEdge],
    center: (f32, f32),
    dragged_idx: Option<usize>,
) {
    let n = nodes.len();
    if n == 0 {
        return;
    }

    let k_rep = 1600.0;
    let k_spring = 0.045;
    let target_len = 80.0;
    let k_center = 0.012;
    let damping = 0.82;

    let mut fx = vec![0.0f32; n];
    let mut fy = vec![0.0f32; n];

    // 1. Repulsion between all pairs
    for i in 0..n {
        for j in (i + 1)..n {
            let dx = nodes[i].x - nodes[j].x;
            let dy = nodes[i].y - nodes[j].y;
            let dist_sq = dx * dx + dy * dy;
            let dist = dist_sq.sqrt().max(1.0);
            let force = k_rep / (dist_sq.max(400.0));
            let f_x = (dx / dist) * force;
            let f_y = (dy / dist) * force;

            fx[i] += f_x;
            fy[i] += f_y;
            fx[j] -= f_x;
            fy[j] -= f_y;
        }
    }

    // 2. Spring attraction along edges
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

    // 4. Update velocity & position
    for i in 0..n {
        if Some(i) == dragged_idx {
            nodes[i].vx = 0.0;
            nodes[i].vy = 0.0;
            continue;
        }
        nodes[i].vx = (nodes[i].vx + fx[i].clamp(-25.0, 25.0)) * damping;
        nodes[i].vy = (nodes[i].vy + fy[i].clamp(-25.0, 25.0)) * damping;
        nodes[i].x += nodes[i].vx;
        nodes[i].y += nodes[i].vy;
    }
}

/// Global 2D Knowledge Graph View
#[component]
pub fn GraphView(state: Signal<AppState>) -> Element {
    let app_state = state.read();
    let graph_data = app_state.get_full_graph_data();
    let total_notes = graph_data.nodes.len();
    let total_edges = graph_data.edges.len();

    let mut nodes_state = use_signal(|| {
        let (nodes, _) = init_simulation(&graph_data, 1000.0, 700.0);
        nodes
    });

    let mut edges_state = use_signal(|| {
        let (_, edges) = init_simulation(&graph_data, 1000.0, 700.0);
        edges
    });

    let mut pan_x = use_signal(|| 0.0f32);
    let mut pan_y = use_signal(|| 0.0f32);
    let mut zoom = use_signal(|| 1.0f32);
    let mut is_panning = use_signal(|| false);
    let mut pan_start = use_signal(|| (0.0f64, 0.0f64));
    let mut dragged_node = use_signal(|| None::<usize>);
    let mut hovered_node = use_signal(|| None::<usize>);
    let mut search_query = use_signal(String::new);

    // If vault notes count changed, re-sync simulation
    use_effect(move || {
        let current_graph = state.read().get_full_graph_data();
        let (n, e) = init_simulation(&current_graph, 1000.0, 700.0);
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
    let query = search_query.read().trim().to_lowercase();
    let zoom_val = *zoom.read();
    let zoom_pct = (zoom_val * 100.0).round() as u32;
    let dot_radius = (0.85 / zoom_val).clamp(0.4, 1.2);

    // Determine connected nodes for hover highlighting
    let mut connected_indices = HashSet::new();
    if let Some(h_idx) = current_hovered {
        connected_indices.insert(h_idx);
        for edge in edges.iter() {
            if edge.source == h_idx {
                connected_indices.insert(edge.target);
            } else if edge.target == h_idx {
                connected_indices.insert(edge.source);
            }
        }
    }

    rsx! {
        div {
            class: "graph-container",
            tabindex: "0",
            onmousedown: move |evt: MouseEvent| {
                is_panning.set(true);
                pan_start.set((evt.client_coordinates().x, evt.client_coordinates().y));
            },
            onmousemove: move |evt: MouseEvent| {
                let cur_x = evt.client_coordinates().x;
                let cur_y = evt.client_coordinates().y;

                if *is_panning.read() {
                    let start = *pan_start.read();
                    let dx = (cur_x - start.0) as f32;
                    let dy = (cur_y - start.1) as f32;
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
                        let es = edges_state.read();
                        step_simulation(&mut ns, &es, (500.0, 350.0), Some(drag_idx));
                    }
                }
            },
            onmouseup: move |_| {
                is_panning.set(false);
                dragged_node.set(None);
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
                        search_query.set(evt.value());
                    },
                }
                if !search_query.read().is_empty() {
                    button {
                        class: "btn-icon",
                        style: "padding: 2px;",
                        onclick: move |_| {
                            search_query.set(String::new());
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
                        let current_graph = state.read().get_full_graph_data();
                        let (n, e) = init_simulation(&current_graph, 1000.0, 700.0);
                        nodes_state.set(n);
                        edges_state.set(e);
                    },
                    IconRefresh { size: 13 }
                }
            }

            // Bottom-left Stats Badge
            div {
                class: "graph-stats",
                span {
                    style: "display: inline-flex; align-items: center; gap: 6px; font-weight: 500; color: var(--text-primary);",
                    IconGraph { size: 13 }
                    span { "{total_notes} notes" }
                }
                span { style: "color: var(--text-muted);", "•" }
                span { style: "color: var(--text-secondary);", "{total_edges} connections" }
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
                }
                g {
                    transform: format!("translate({}, {}) scale({})", *pan_x.read(), *pan_y.read(), *zoom.read()),

                    // Infinite subtle dotted background grid
                    rect {
                        x: "-100000",
                        y: "-100000",
                        width: "200000",
                        height: "200000",
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
                                let is_highlighted = if let Some(h) = current_hovered {
                                    s == h || t == h
                                } else {
                                    false
                                };

                                let edge_color = if is_highlighted {
                                    "var(--graph-edge-highlight, #7182FF)"
                                } else {
                                    "var(--graph-edge, #444A5B)"
                                };

                                let edge_opacity = if current_hovered.is_some() {
                                    if is_highlighted { "1.0" } else { "0.15" }
                                } else {
                                    "0.7"
                                };

                                let edge_width = if is_highlighted { "2.5" } else { "1.2" };

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
                            let is_hovered = current_hovered == Some(idx);
                            let is_connected = connected_indices.contains(&idx);
                            let matches_query = if query.is_empty() {
                                true
                            } else {
                                node.label.to_lowercase().contains(&query)
                            };

                            let node_color = if is_current {
                                "var(--graph-node-current, #9A4BFF)"
                            } else if is_hovered {
                                "var(--graph-node-hover, #7182FF)"
                            } else if is_connected {
                                "var(--graph-node-connected, #7E8CFF)"
                            } else {
                                "var(--graph-node, #5B6CFF)"
                            };

                            let node_opacity = if current_hovered.is_some() {
                                if is_hovered || is_connected { "1.0" } else { "0.25" }
                            } else if !matches_query {
                                "0.2"
                            } else {
                                "1.0"
                            };

                            let stroke_color = if is_current {
                                "#D5C7FF"
                            } else if is_hovered {
                                "#FFFFFF"
                            } else {
                                "var(--border, #292E3A)"
                            };

                            let stroke_width = if is_hovered || is_current { "2.5" } else { "1.5" };
                            let path_click = node.path.clone();

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
                                        let mut s = state.write();
                                        let _ = s.select_note(&path_click);
                                        s.active_view = ActiveView::Editor;
                                    },
                                    circle {
                                        cx: "{node.x}",
                                        cy: "{node.y}",
                                        r: "{node.radius}",
                                        fill: "{node_color}",
                                        stroke: "{stroke_color}",
                                        stroke_width: "{stroke_width}",
                                    }
                                    if *zoom.read() >= 0.75 || is_hovered || is_connected || total_notes <= 50 {
                                        text {
                                            x: "{node.x}",
                                            y: format!("{}", node.y + node.radius + 12.0),
                                            text_anchor: "middle",
                                            fill: "var(--graph-label, #D7DBE6)",
                                            font_size: "11",
                                            font_weight: if is_hovered || is_current { "600" } else { "400" },
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

/// Compact Local 2D Graph for Context Panel
#[component]
pub fn LocalGraphView(state: Signal<AppState>) -> Element {
    let app_state = state.read();
    let local_graph = app_state.get_local_graph_data(1);
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

    // Re-sync when active note changes
    use_effect(move || {
        let current_local = state.read().get_local_graph_data(1);
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
            // Header with expand button
            div {
                style: "position: absolute; top: 6px; right: 6px; z-index: 5;",
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
                        let fill_color = if is_current {
                            "var(--graph-node-current, #9A4BFF)"
                        } else if is_hovered {
                            "var(--graph-node-hover, #7182FF)"
                        } else {
                            "var(--graph-node, #5B6CFF)"
                        };
                        let path_click = node.path.clone();

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
                                    let _ = s.select_note(&path_click);
                                },
                                circle {
                                    cx: "{node.x}",
                                    cy: "{node.y}",
                                    r: if is_current { "8" } else { "6" },
                                    fill: "{fill_color}",
                                    stroke: if is_current { "#D5C7FF" } else { "var(--border, #292E3A)" },
                                    stroke_width: "1.5",
                                }
                                text {
                                    x: "{node.x}",
                                    y: format!("{}", node.y + if is_current { 18.0 } else { 15.0 }),
                                    text_anchor: "middle",
                                    fill: "var(--graph-label, #D7DBE6)",
                                    font_size: "10",
                                    font_weight: if is_current { "600" } else { "400" },
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
