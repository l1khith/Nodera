use serde::{Deserialize, Serialize};
use std::fmt;

/// Type/category of graphics adapter discovered on the host system.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AdapterKind {
    /// Dedicated/discrete high-performance GPU (NVIDIA, AMD Radeon dGPU, Apple Pro/Max).
    DiscreteGpu,
    /// Processor-integrated GPU (Intel Iris/UHD, AMD APU, Apple standard).
    IntegratedGpu,
    /// Virtualized graphics device in cloud or virtual machine environments.
    VirtualGpu,
    /// Software rasterizer / CPU fallback (e.g. WARP, llvmpipe, SwiftShader).
    Cpu,
    /// Other or unclassified device.
    Other,
}

impl fmt::Display for AdapterKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AdapterKind::DiscreteGpu => write!(f, "Discrete GPU"),
            AdapterKind::IntegratedGpu => write!(f, "Integrated GPU"),
            AdapterKind::VirtualGpu => write!(f, "Virtual GPU"),
            AdapterKind::Cpu => write!(f, "CPU Software Fallback"),
            AdapterKind::Other => write!(f, "Other Graphics Device"),
        }
    }
}

impl From<wgpu::DeviceType> for AdapterKind {
    fn from(device_type: wgpu::DeviceType) -> Self {
        match device_type {
            wgpu::DeviceType::DiscreteGpu => AdapterKind::DiscreteGpu,
            wgpu::DeviceType::IntegratedGpu => AdapterKind::IntegratedGpu,
            wgpu::DeviceType::VirtualGpu => AdapterKind::VirtualGpu,
            wgpu::DeviceType::Cpu => AdapterKind::Cpu,
            wgpu::DeviceType::Other => AdapterKind::Other,
        }
    }
}

/// Summary of an individual adapter discovered during graphics subsystem probing.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DiscoveredAdapterSummary {
    pub name: String,
    pub backend: String,
    pub kind: AdapterKind,
    pub score: u32,
    pub is_selected: bool,
    pub driver: String,
    pub driver_info: String,
}

/// Comprehensive diagnostics on host graphics capabilities, chosen adapter, and hardware acceleration status.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GraphicsDiagnostics {
    pub adapter_name: String,
    pub backend: String,
    pub device_type: AdapterKind,
    pub driver: String,
    pub driver_info: String,
    pub vendor_id: u32,
    pub device_id: u32,
    pub is_hardware_accelerated: bool,
    pub max_texture_dimension_2d: u32,
    pub supported_features_count: usize,
    pub score: u32,
    pub detected_adapters: Vec<DiscoveredAdapterSummary>,
    pub status_message: String,
}

impl Default for GraphicsDiagnostics {
    fn default() -> Self {
        Self {
            adapter_name: "CPU Software Rasterizer".to_string(),
            backend: "Software / SVG DOM".to_string(),
            device_type: AdapterKind::Cpu,
            driver: "Native Host CPU".to_string(),
            driver_info: "Fallback Software Engine".to_string(),
            vendor_id: 0,
            device_id: 0,
            is_hardware_accelerated: false,
            max_texture_dimension_2d: 4096,
            supported_features_count: 0,
            score: 50,
            detected_adapters: vec![DiscoveredAdapterSummary {
                name: "CPU Software Rasterizer".to_string(),
                backend: "Software / SVG DOM".to_string(),
                kind: AdapterKind::Cpu,
                score: 50,
                is_selected: true,
                driver: "Native Host CPU".to_string(),
                driver_info: "Fallback Software Engine".to_string(),
            }],
            status_message: "Fallback CPU rendering active".to_string(),
        }
    }
}

/// Scoring policy prioritizing discrete GPUs, modern backends, and high-performance feature sets.
pub fn score_adapter(info: &wgpu::AdapterInfo, limits: Option<&wgpu::Limits>) -> u32 {
    let mut score = match info.device_type {
        wgpu::DeviceType::DiscreteGpu => 1000,
        wgpu::DeviceType::IntegratedGpu => 500,
        wgpu::DeviceType::VirtualGpu => 250,
        wgpu::DeviceType::Cpu => 50,
        wgpu::DeviceType::Other => 100,
    };

    // Backend bonus: Modern low-overhead graphics APIs
    score += match info.backend {
        wgpu::Backend::Vulkan => 200,
        wgpu::Backend::Metal => 200,
        wgpu::Backend::Dx12 => 200,
        wgpu::Backend::Gl => 50,
        wgpu::Backend::BrowserWebGpu => 50,
        _ => 0,
    };

    // Limits bonus: Texture dimensions and capability scaling
    if let Some(lim) = limits {
        score += (lim.max_texture_dimension_2d / 1024).min(50);
    }

    score
}

/// Safely inspects the host platform for graphics adapters without crashing or throwing unhandled exceptions.
pub fn detect_graphics() -> GraphicsDiagnostics {
    tracing::info!("Probing system graphics adapters via wgpu...");

    // Catch potential driver crashes or panics during enumeration
    let result = std::panic::catch_unwind(|| {
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            flags: wgpu::InstanceFlags::empty(),
            backend_options: wgpu::BackendOptions::default(),
        });

        let adapters = instance.enumerate_adapters(wgpu::Backends::all());
        if adapters.is_empty() {
            tracing::warn!(
                "No graphics adapters returned by wgpu::enumerate_adapters; falling back to CPU."
            );
            return None;
        }

        let mut scored_adapters = Vec::with_capacity(adapters.len());
        for adapter in adapters {
            let info = adapter.get_info();
            let limits = adapter.limits();
            let score = score_adapter(&info, Some(&limits));
            scored_adapters.push((adapter, info, limits, score));
        }

        // Sort descending by score
        scored_adapters.sort_by_key(|a| std::cmp::Reverse(a.3));

        Some(scored_adapters)
    });

    match result {
        Ok(Some(scored)) if !scored.is_empty() => {
            let best_score = scored[0].3;
            let mut detected_summaries = Vec::with_capacity(scored.len());

            for (idx, (_, info, _, score)) in scored.iter().enumerate() {
                let kind = AdapterKind::from(info.device_type);
                let backend_str = format!("{:?}", info.backend);
                detected_summaries.push(DiscoveredAdapterSummary {
                    name: info.name.clone(),
                    backend: backend_str,
                    kind,
                    score: *score,
                    is_selected: idx == 0,
                    driver: info.driver.clone(),
                    driver_info: info.driver_info.clone(),
                });
            }

            let (_, best_info, best_limits, _) = &scored[0];
            let best_kind = AdapterKind::from(best_info.device_type);
            let is_hw = best_kind != AdapterKind::Cpu;
            let backend_name = format!("{:?}", best_info.backend);

            tracing::info!(
                adapter = %best_info.name,
                backend = %backend_name,
                kind = %best_kind,
                hardware_accelerated = is_hw,
                "Selected primary graphics adapter"
            );

            GraphicsDiagnostics {
                adapter_name: best_info.name.clone(),
                backend: backend_name.clone(),
                device_type: best_kind,
                driver: best_info.driver.clone(),
                driver_info: best_info.driver_info.clone(),
                vendor_id: best_info.vendor,
                device_id: best_info.device,
                is_hardware_accelerated: is_hw,
                max_texture_dimension_2d: best_limits.max_texture_dimension_2d,
                supported_features_count: 0,
                score: best_score,
                detected_adapters: detected_summaries,
                status_message: if is_hw {
                    format!("Hardware acceleration active ({best_kind} - {backend_name})")
                } else {
                    "Software rasterizer active (CPU fallback)".to_string()
                },
            }
        }
        Ok(_) => {
            tracing::info!("No suitable GPU adapter detected; using CPU software fallback.");
            GraphicsDiagnostics::default()
        }
        Err(err) => {
            tracing::error!(
                error = ?err,
                "Panic during graphics adapter enumeration; falling back to CPU safely."
            );
            GraphicsDiagnostics {
                status_message: "Graphics probe error - safe CPU fallback active".to_string(),
                ..Default::default()
            }
        }
    }
}
