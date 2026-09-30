use serde::{Deserialize, Serialize};

/// 2D Viewport transformation defining pan, zoom, and visible canvas dimensions.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Viewport {
    pub pan_x: f32,
    pub pan_y: f32,
    pub zoom: f32,
    pub canvas_width: f32,
    pub canvas_height: f32,
}

impl Default for Viewport {
    fn default() -> Self {
        Self {
            pan_x: 0.0,
            pan_y: 0.0,
            zoom: 1.0,
            canvas_width: 1000.0,
            canvas_height: 700.0,
        }
    }
}

impl Viewport {
    /// Converts a screen/client coordinate (in pixels) into graph world space coordinates.
    #[inline]
    pub fn screen_to_world(&self, sx: f32, sy: f32) -> (f32, f32) {
        let z = if self.zoom.abs() < 1e-4 {
            1.0
        } else {
            self.zoom
        };
        ((sx - self.pan_x) / z, (sy - self.pan_y) / z)
    }

    /// Converts a world coordinate into screen/client pixel coordinates.
    #[inline]
    pub fn world_to_screen(&self, wx: f32, wy: f32) -> (f32, f32) {
        (wx * self.zoom + self.pan_x, wy * self.zoom + self.pan_y)
    }

    /// Calculates the world-space bounding box for current visible viewport with an optional margin in pixels.
    pub fn visible_world_bounds(&self, pixel_margin: f32) -> (f32, f32, f32, f32) {
        let z = if self.zoom.abs() < 1e-4 {
            1.0
        } else {
            self.zoom
        };
        let min_x = (-self.pan_x - pixel_margin) / z;
        let max_x = (self.canvas_width - self.pan_x + pixel_margin) / z;
        let min_y = (-self.pan_y - pixel_margin) / z;
        let max_y = (self.canvas_height - self.pan_y + pixel_margin) / z;
        (min_x, min_y, max_x, max_y)
    }
}

/// Renderer-independent compact instanced node representation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GraphNodeInstance {
    pub id: String,
    pub x: f32,
    pub y: f32,
    pub radius: f32,
    pub color_rgba: [f32; 4],
    pub label: String,
    pub degree: usize,
    pub centrality: u32,
    pub is_hovered: bool,
    pub is_selected: bool,
    pub is_dimmed: bool,
}

/// Renderer-independent compact instanced edge representation.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct GraphEdgeInstance {
    pub source_idx: usize,
    pub target_idx: usize,
    pub x1: f32,
    pub y1: f32,
    pub x2: f32,
    pub y2: f32,
    pub color_rgba: [f32; 4],
    pub width: f32,
    pub is_dimmed: bool,
}

/// Metrics recorded during a rendering pass.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RenderStats {
    pub renderer_name: String,
    pub backend: String,
    pub nodes_rendered: usize,
    pub nodes_culled: usize,
    pub edges_rendered: usize,
    pub edges_culled: usize,
    pub render_duration_us: u64,
    pub is_gpu_accelerated: bool,
}

impl Default for RenderStats {
    fn default() -> Self {
        Self {
            renderer_name: "CPU Fallback".to_string(),
            backend: "Software / SVG".to_string(),
            nodes_rendered: 0,
            nodes_culled: 0,
            edges_rendered: 0,
            edges_culled: 0,
            render_duration_us: 0,
            is_gpu_accelerated: false,
        }
    }
}

/// Standalone scene description holding node and edge geometry with viewport state.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct GraphScene {
    pub nodes: Vec<GraphNodeInstance>,
    pub edges: Vec<GraphEdgeInstance>,
    pub viewport: Viewport,
}

impl GraphScene {
    pub fn new(
        nodes: Vec<GraphNodeInstance>,
        edges: Vec<GraphEdgeInstance>,
        viewport: Viewport,
    ) -> Self {
        Self {
            nodes,
            edges,
            viewport,
        }
    }

    /// Evaluates spatial frustum culling against the current viewport.
    /// Returns `(visible_node_indices, visible_edge_indices)`.
    pub fn spatial_cull(&self, pixel_margin: f32) -> (Vec<usize>, Vec<usize>) {
        let (min_x, min_y, max_x, max_y) = self.viewport.visible_world_bounds(pixel_margin);

        let mut visible_nodes = Vec::with_capacity(self.nodes.len());
        let mut node_visible_mask = vec![false; self.nodes.len()];

        for (idx, node) in self.nodes.iter().enumerate() {
            let r = node.radius;
            if node.x + r >= min_x
                && node.x - r <= max_x
                && node.y + r >= min_y
                && node.y - r <= max_y
            {
                visible_nodes.push(idx);
                node_visible_mask[idx] = true;
            }
        }

        let mut visible_edges = Vec::with_capacity(self.edges.len());
        for (idx, edge) in self.edges.iter().enumerate() {
            // An edge is considered visible if either endpoint is visible or if line intersects AABB
            let src_vis = node_visible_mask
                .get(edge.source_idx)
                .copied()
                .unwrap_or(false);
            let tgt_vis = node_visible_mask
                .get(edge.target_idx)
                .copied()
                .unwrap_or(false);

            if src_vis || tgt_vis {
                visible_edges.push(idx);
            } else {
                // Check edge segment bounding box against viewport AABB
                let e_min_x = edge.x1.min(edge.x2);
                let e_max_x = edge.x1.max(edge.x2);
                let e_min_y = edge.y1.min(edge.y2);
                let e_max_y = edge.y1.max(edge.y2);

                if e_max_x >= min_x && e_min_x <= max_x && e_max_y >= min_y && e_min_y <= max_y {
                    visible_edges.push(idx);
                }
            }
        }

        (visible_nodes, visible_edges)
    }
}
