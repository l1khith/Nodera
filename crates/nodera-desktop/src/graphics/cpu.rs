use std::time::Instant;

use crate::graphics::renderer::{
    GraphRenderer, RendererBackendInfo, RendererConfig, RendererError,
};
use crate::graphics::scene::{GraphScene, RenderStats};

/// Level of detail for label presentation based on current zoom factor.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LabelLod {
    /// Full label text rendered (zoom >= 0.75).
    Full,
    /// Truncated to 12 characters (0.45 <= zoom < 0.75).
    Truncated,
    /// Only top-connected hubs and selected nodes display labels (0.25 <= zoom < 0.45).
    MajorHubsOnly,
    /// Pure node dots without text for maximum frame rates (zoom < 0.25).
    None,
}

impl LabelLod {
    pub fn from_zoom(zoom: f32) -> Self {
        if zoom >= 0.75 {
            LabelLod::Full
        } else if zoom >= 0.45 {
            LabelLod::Truncated
        } else if zoom >= 0.25 {
            LabelLod::MajorHubsOnly
        } else {
            LabelLod::None
        }
    }
}

/// Robust, zero-dependency software CPU graph renderer.
/// Guarantees that any host system without a functional GPU continues operating cleanly.
pub struct CpuGraphRenderer {
    backend_info: RendererBackendInfo,
    scene: GraphScene,
    config: RendererConfig,
}

impl Default for CpuGraphRenderer {
    fn default() -> Self {
        Self::new()
    }
}

impl CpuGraphRenderer {
    pub fn new() -> Self {
        Self {
            backend_info: RendererBackendInfo {
                name: "CPU Software Rasterizer".to_string(),
                backend: "Software / SVG DOM".to_string(),
                device: "Host CPU".to_string(),
                is_hardware_accelerated: false,
                description:
                    "Spatial-culled software projection pipeline for 100% platform portability"
                        .to_string(),
            },
            scene: GraphScene::default(),
            config: RendererConfig::default(),
        }
    }

    /// Evaluates which label LOD applies to the active viewport zoom.
    pub fn current_lod(&self) -> LabelLod {
        LabelLod::from_zoom(self.scene.viewport.zoom)
    }
}

impl GraphRenderer for CpuGraphRenderer {
    fn initialize(&mut self, config: RendererConfig) -> Result<(), RendererError> {
        self.config = config;
        tracing::info!("CpuGraphRenderer initialized successfully");
        Ok(())
    }

    fn resize(&mut self, width: u32, height: u32) {
        self.config.width = width;
        self.config.height = height;
        self.scene.viewport.canvas_width = width as f32;
        self.scene.viewport.canvas_height = height as f32;
    }

    fn update_scene(&mut self, scene: &GraphScene) {
        self.scene = scene.clone();
    }

    fn render(&mut self) -> Result<RenderStats, RendererError> {
        let start = Instant::now();

        // Perform fast world-space AABB culling
        let (visible_nodes, visible_edges) = self.scene.spatial_cull(50.0);
        let total_nodes = self.scene.nodes.len();
        let total_edges = self.scene.edges.len();
        let nodes_rendered = visible_nodes.len();
        let nodes_culled = total_nodes.saturating_sub(nodes_rendered);
        let edges_rendered = visible_edges.len();
        let edges_culled = total_edges.saturating_sub(edges_rendered);

        let duration = start.elapsed().as_micros() as u64;

        Ok(RenderStats {
            renderer_name: self.backend_info.name.clone(),
            backend: self.backend_info.backend.clone(),
            nodes_rendered,
            nodes_culled,
            edges_rendered,
            edges_culled,
            render_duration_us: duration,
            is_gpu_accelerated: false,
        })
    }

    fn backend_info(&self) -> RendererBackendInfo {
        self.backend_info.clone()
    }

    fn shutdown(&mut self) {
        // CPU fallback has no external GPU resources to release
    }
}
