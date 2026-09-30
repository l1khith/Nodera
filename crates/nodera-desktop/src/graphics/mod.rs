pub mod adapter;
pub mod cpu;
pub mod gpu;
pub mod renderer;
pub mod scene;

use std::sync::OnceLock;

pub use adapter::{
    detect_graphics, score_adapter, AdapterKind, DiscoveredAdapterSummary, GraphicsDiagnostics,
};
pub use cpu::{CpuGraphRenderer, LabelLod};
pub use gpu::GpuGraphRenderer;
pub use renderer::{
    GraphRenderer, GraphicsRendererPreference, RendererBackendInfo, RendererConfig, RendererError,
};
pub use scene::{GraphEdgeInstance, GraphNodeInstance, GraphScene, RenderStats, Viewport};

static CACHED_DIAGNOSTICS: OnceLock<GraphicsDiagnostics> = OnceLock::new();

/// Retrieves system graphics diagnostics, evaluating on first access and caching results for the application lifecycle.
pub fn cached_diagnostics() -> &'static GraphicsDiagnostics {
    CACHED_DIAGNOSTICS.get_or_init(detect_graphics)
}

/// Creates and initializes a graph renderer according to user preference, automatically falling back
/// to CPU software rendering if GPU initialization fails for any reason.
pub fn create_graph_renderer(
    preference: GraphicsRendererPreference,
    config: RendererConfig,
) -> (Box<dyn GraphRenderer>, RendererBackendInfo) {
    match preference {
        GraphicsRendererPreference::ForceCpu => {
            tracing::info!("User preference configured for CPU fallback renderer");
            let mut cpu_renderer = CpuGraphRenderer::new();
            let _ = cpu_renderer.initialize(config);
            let info = cpu_renderer.backend_info();
            (Box::new(cpu_renderer), info)
        }
        GraphicsRendererPreference::Auto | GraphicsRendererPreference::ForceGpu => {
            tracing::info!(preference = %preference, "Attempting GPU hardware acceleration initialization...");
            let mut gpu_renderer = GpuGraphRenderer::new();

            match gpu_renderer.initialize(config) {
                Ok(()) => {
                    let info = gpu_renderer.backend_info();
                    tracing::info!(
                        backend = %info.backend,
                        device = %info.device,
                        "Hardware-accelerated graphics renderer active"
                    );
                    (Box::new(gpu_renderer), info)
                }
                Err(err) => {
                    tracing::warn!(
                        error = %err,
                        "GPU graphics initialization failed; falling back to CPU software rasterizer"
                    );
                    let mut cpu_renderer = CpuGraphRenderer::new();
                    let _ = cpu_renderer.initialize(config);
                    let mut info = cpu_renderer.backend_info();
                    info.description = format!("CPU fallback active (GPU unavailable: {})", err);
                    (Box::new(cpu_renderer), info)
                }
            }
        }
    }
}
