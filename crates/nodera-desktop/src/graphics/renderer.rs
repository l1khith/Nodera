use serde::{Deserialize, Serialize};
use std::fmt;

use crate::graphics::scene::{GraphScene, RenderStats};

/// Errors encountered in the graphics rendering pipeline or device initialization.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RendererError {
    AdapterNotFound(String),
    DeviceCreationFailed(String),
    SurfaceCreationFailed(String),
    ShaderCompilationFailed(String),
    BufferAllocationFailed(String),
    OutOfMemory(String),
    ExecutionFailed(String),
}

impl fmt::Display for RendererError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RendererError::AdapterNotFound(s) => {
                write!(f, "No compatible graphics adapter discovered: {s}")
            }
            RendererError::DeviceCreationFailed(s) => {
                write!(f, "Failed to create graphics device and queue: {s}")
            }
            RendererError::SurfaceCreationFailed(s) => {
                write!(f, "Failed to initialize rendering surface: {s}")
            }
            RendererError::ShaderCompilationFailed(s) => {
                write!(f, "Shader compilation or pipeline assembly error: {s}")
            }
            RendererError::BufferAllocationFailed(s) => {
                write!(f, "Failed to allocate GPU buffer: {s}")
            }
            RendererError::OutOfMemory(s) => write!(f, "Graphics hardware out of memory: {s}"),
            RendererError::ExecutionFailed(s) => write!(f, "Rendering execution error: {s}"),
        }
    }
}

impl std::error::Error for RendererError {}

/// Information identifying an active graphics backend and device.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RendererBackendInfo {
    pub name: String,
    pub backend: String,
    pub device: String,
    pub is_hardware_accelerated: bool,
    pub description: String,
}

impl Default for RendererBackendInfo {
    fn default() -> Self {
        Self {
            name: "CPU Fallback Renderer".to_string(),
            backend: "Software / SVG DOM".to_string(),
            device: "Host CPU".to_string(),
            is_hardware_accelerated: false,
            description: "Zero-dependency CPU software rasterizer with spatial culling".to_string(),
        }
    }
}

/// Configuration supplied when initializing or reconfiguring a graph renderer.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct RendererConfig {
    pub width: u32,
    pub height: u32,
    pub dpi_scale: f32,
    pub prefer_low_power: bool,
}

impl Default for RendererConfig {
    fn default() -> Self {
        Self {
            width: 1000,
            height: 700,
            dpi_scale: 1.0,
            prefer_low_power: false,
        }
    }
}

/// User preference for graphics backend selection.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum GraphicsRendererPreference {
    /// Automatically select highest-scoring GPU adapter, falling back to CPU if unavailable.
    #[default]
    Auto,
    /// Force GPU hardware-accelerated rendering; automatically falls back with warning if unavailable.
    ForceGpu,
    /// Force CPU software rendering regardless of available hardware acceleration.
    ForceCpu,
}

impl fmt::Display for GraphicsRendererPreference {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GraphicsRendererPreference::Auto => write!(f, "Auto (Prefer GPU)"),
            GraphicsRendererPreference::ForceGpu => write!(f, "Force GPU"),
            GraphicsRendererPreference::ForceCpu => write!(f, "Force CPU Fallback"),
        }
    }
}

/// Universal contract for Nodera graph renderers across GPU and CPU backends.
pub trait GraphRenderer: Send + Sync {
    /// Initializes graphics resources, device, queues, and shaders.
    fn initialize(&mut self, config: RendererConfig) -> Result<(), RendererError>;

    /// Updates viewport dimensions and projection matrices.
    fn resize(&mut self, width: u32, height: u32);

    /// Synchronizes the latest node and edge instances for drawing.
    fn update_scene(&mut self, scene: &GraphScene);

    /// Executes the drawing pass, returning performance and culling statistics.
    fn render(&mut self) -> Result<RenderStats, RendererError>;

    /// Queries information describing the active backend and adapter.
    fn backend_info(&self) -> RendererBackendInfo;

    /// Checks if this renderer actively employs hardware acceleration.
    fn is_hardware_accelerated(&self) -> bool {
        self.backend_info().is_hardware_accelerated
    }

    /// Releases resources and cleans up pipeline structures.
    fn shutdown(&mut self);
}
