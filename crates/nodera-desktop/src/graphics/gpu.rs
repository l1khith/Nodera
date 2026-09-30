use std::sync::Arc;
use std::time::Instant;

use crate::graphics::renderer::{
    GraphRenderer, RendererBackendInfo, RendererConfig, RendererError,
};
use crate::graphics::scene::{GraphScene, RenderStats};

/// WGSL shader code for instanced 2D graph node and edge rendering
const GRAPH_WGSL: &str = r#"
struct ViewportUniform {
    pan_x: f32,
    pan_y: f32,
    zoom: f32,
    _pad0: f32,
    canvas_width: f32,
    canvas_height: f32,
    _pad1: f32,
    _pad2: f32,
};

@group(0) @binding(0)
var<uniform> u_viewport: ViewportUniform;

struct VertexInput {
    @location(0) quad_pos: vec2<f32>,
};

struct InstanceInput {
    @location(1) center: vec2<f32>,
    @location(2) radius: f32,
    @location(3) color: vec4<f32>,
    @location(4) is_dimmed: f32,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) local_uv: vec2<f32>,
    @location(1) color: vec4<f32>,
    @location(2) is_dimmed: f32,
};

@vertex
fn vs_main(model: VertexInput, instance: InstanceInput) -> VertexOutput {
    var out: VertexOutput;
    out.local_uv = model.quad_pos;
    out.color = instance.color;
    out.is_dimmed = instance.is_dimmed;

    // Transform world to screen coordinates: (world * zoom) + pan
    let world_pos = instance.center + (model.quad_pos * instance.radius * 1.5);
    let screen_x = world_pos.x * u_viewport.zoom + u_viewport.pan_x;
    let screen_y = world_pos.y * u_viewport.zoom + u_viewport.pan_y;

    // Screen to Normalized Device Coordinates (NDC) [-1, 1]
    let ndc_x = (screen_x / u_viewport.canvas_width) * 2.0 - 1.0;
    let ndc_y = 1.0 - (screen_y / u_viewport.canvas_height) * 2.0;

    out.clip_position = vec4<f32>(ndc_x, ndc_y, 0.0, 1.0);
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let dist_sq = dot(in.local_uv, in.local_uv);
    if dist_sq > 1.0 {
        discard;
    }

    // Antialiased border feathering
    let alpha_edge = 1.0 - smoothstep(0.85, 1.0, dist_sq);
    var col = in.color;
    if in.is_dimmed > 0.5 {
        col.a = col.a * 0.25;
    }
    col.a = col.a * alpha_edge;
    return col;
}
"#;

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct GpuViewportUniform {
    pan_x: f32,
    pan_y: f32,
    zoom: f32,
    _pad0: f32,
    canvas_width: f32,
    canvas_height: f32,
    _pad1: f32,
    _pad2: f32,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct GpuNodeVertex {
    quad_pos: [f32; 2],
}

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct GpuNodeInstance {
    center: [f32; 2],
    radius: f32,
    color: [f32; 4],
    is_dimmed: f32,
}

const QUAD_VERTICES: [GpuNodeVertex; 6] = [
    GpuNodeVertex {
        quad_pos: [-1.0, -1.0],
    },
    GpuNodeVertex {
        quad_pos: [1.0, -1.0],
    },
    GpuNodeVertex {
        quad_pos: [1.0, 1.0],
    },
    GpuNodeVertex {
        quad_pos: [-1.0, -1.0],
    },
    GpuNodeVertex {
        quad_pos: [1.0, 1.0],
    },
    GpuNodeVertex {
        quad_pos: [-1.0, 1.0],
    },
];

/// Hardware-accelerated GPU graph renderer utilizing wgpu.
pub struct GpuGraphRenderer {
    device: Option<Arc<wgpu::Device>>,
    queue: Option<Arc<wgpu::Queue>>,
    render_pipeline: Option<wgpu::RenderPipeline>,
    uniform_buffer: Option<wgpu::Buffer>,
    vertex_buffer: Option<wgpu::Buffer>,
    instance_buffer: Option<wgpu::Buffer>,
    instance_buffer_capacity: usize,
    bind_group: Option<wgpu::BindGroup>,
    backend_info: RendererBackendInfo,
    scene: GraphScene,
    config: RendererConfig,
}

impl Default for GpuGraphRenderer {
    fn default() -> Self {
        Self::new()
    }
}

impl GpuGraphRenderer {
    pub fn new() -> Self {
        Self {
            device: None,
            queue: None,
            render_pipeline: None,
            uniform_buffer: None,
            vertex_buffer: None,
            instance_buffer: None,
            instance_buffer_capacity: 0,
            bind_group: None,
            backend_info: RendererBackendInfo {
                name: "WGPU Hardware Acceleration".to_string(),
                backend: "Uninitialized".to_string(),
                device: "Uninitialized".to_string(),
                is_hardware_accelerated: true,
                description: "Modern low-overhead GPU pipeline for high-density knowledge graphs"
                    .to_string(),
            },
            scene: GraphScene::default(),
            config: RendererConfig::default(),
        }
    }
}

impl GraphRenderer for GpuGraphRenderer {
    fn initialize(&mut self, config: RendererConfig) -> Result<(), RendererError> {
        self.config = config;

        // Execute asynchronous GPU adapter and device discovery on an isolated OS thread
        // to guarantee safety whether called from sync or async Tokio execution context.
        let init_result = std::thread::spawn(move || {
            let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
                backends: wgpu::Backends::all(),
                flags: wgpu::InstanceFlags::empty(),
                backend_options: wgpu::BackendOptions::default(),
            });

            let adapters = instance.enumerate_adapters(wgpu::Backends::all());
            if adapters.is_empty() {
                return Err(RendererError::AdapterNotFound(
                    "No functional GPU adapters discovered on system".to_string(),
                ));
            }

            // Pick highest scoring adapter
            let mut scored = adapters
                .into_iter()
                .map(|a| {
                    let info = a.get_info();
                    let limits = a.limits();
                    let score = crate::graphics::adapter::score_adapter(&info, Some(&limits));
                    (a, info, score)
                })
                .collect::<Vec<_>>();
            scored.sort_by_key(|a| std::cmp::Reverse(a.2));

            let (adapter, info, _) = scored.remove(0);

            // Block for device and queue creation
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .map_err(|e| RendererError::DeviceCreationFailed(e.to_string()))?;

            let (device, queue) = rt
                .block_on(adapter.request_device(
                    &wgpu::DeviceDescriptor {
                        label: Some("nodera_gpu_device"),
                        required_features: wgpu::Features::empty(),
                        required_limits: wgpu::Limits::downlevel_webgl2_defaults(),
                        memory_hints: wgpu::MemoryHints::Performance,
                    },
                    None,
                ))
                .map_err(|e| RendererError::DeviceCreationFailed(e.to_string()))?;

            Ok((device, queue, info))
        })
        .join()
        .map_err(|_| {
            RendererError::ExecutionFailed("Thread panic during GPU initialization".to_string())
        })??;

        let (device, queue, info) = init_result;
        let device = Arc::new(device);
        let queue = Arc::new(queue);

        // Compile WGSL shader module
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("nodera_graph_shader"),
            source: wgpu::ShaderSource::Wgsl(GRAPH_WGSL.into()),
        });

        // Create uniform buffer
        let uniform_size = std::mem::size_of::<GpuViewportUniform>() as wgpu::BufferAddress;
        let uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("nodera_viewport_uniform"),
            size: uniform_size,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("nodera_bind_group_layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("nodera_bind_group"),
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding(),
            }],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("nodera_pipeline_layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        use wgpu::util::DeviceExt;
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("nodera_quad_vertices"),
            contents: bytemuck::cast_slice(&QUAD_VERTICES),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let vertex_buffer_layout = wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<GpuNodeVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[wgpu::VertexAttribute {
                offset: 0,
                shader_location: 0,
                format: wgpu::VertexFormat::Float32x2,
            }],
        };

        let instance_buffer_layout = wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<GpuNodeInstance>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x2,
                },
                wgpu::VertexAttribute {
                    offset: 8,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float32,
                },
                wgpu::VertexAttribute {
                    offset: 12,
                    shader_location: 3,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: 28,
                    shader_location: 4,
                    format: wgpu::VertexFormat::Float32,
                },
            ],
        };

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("nodera_graph_pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[vertex_buffer_layout, instance_buffer_layout],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: wgpu::TextureFormat::Rgba8UnormSrgb,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        });

        self.backend_info = RendererBackendInfo {
            name: "WGPU Hardware Accelerated".to_string(),
            backend: format!("{:?}", info.backend),
            device: info.name.clone(),
            is_hardware_accelerated: info.device_type != wgpu::DeviceType::Cpu,
            description: format!(
                "GPU instanced pipeline running on {} ({:?})",
                info.name, info.backend
            ),
        };

        self.device = Some(device);
        self.queue = Some(queue);
        self.render_pipeline = Some(render_pipeline);
        self.uniform_buffer = Some(uniform_buffer);
        self.vertex_buffer = Some(vertex_buffer);
        self.bind_group = Some(bind_group);

        tracing::info!(
            backend = %self.backend_info.backend,
            device = %self.backend_info.device,
            "GpuGraphRenderer successfully initialized"
        );

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

        let device = self
            .device
            .as_ref()
            .ok_or_else(|| RendererError::ExecutionFailed("Device not initialized".to_string()))?;
        let queue = self
            .queue
            .as_ref()
            .ok_or_else(|| RendererError::ExecutionFailed("Queue not initialized".to_string()))?;
        let pipeline = self.render_pipeline.as_ref().ok_or_else(|| {
            RendererError::ExecutionFailed("Pipeline not initialized".to_string())
        })?;
        let uniform_buffer = self.uniform_buffer.as_ref().ok_or_else(|| {
            RendererError::ExecutionFailed("Uniform buffer not initialized".to_string())
        })?;
        let vertex_buffer = self.vertex_buffer.as_ref().ok_or_else(|| {
            RendererError::ExecutionFailed("Vertex buffer not initialized".to_string())
        })?;
        let bind_group = self.bind_group.as_ref().ok_or_else(|| {
            RendererError::ExecutionFailed("Bind group not initialized".to_string())
        })?;

        // Perform spatial frustum culling
        let (visible_nodes, visible_edges) = self.scene.spatial_cull(50.0);
        let total_nodes = self.scene.nodes.len();
        let total_edges = self.scene.edges.len();
        let nodes_rendered = visible_nodes.len();
        let nodes_culled = total_nodes.saturating_sub(nodes_rendered);
        let edges_rendered = visible_edges.len();
        let edges_culled = total_edges.saturating_sub(edges_rendered);

        // Upload viewport uniforms
        let uniforms = GpuViewportUniform {
            pan_x: self.scene.viewport.pan_x,
            pan_y: self.scene.viewport.pan_y,
            zoom: self.scene.viewport.zoom,
            _pad0: 0.0,
            canvas_width: self.scene.viewport.canvas_width,
            canvas_height: self.scene.viewport.canvas_height,
            _pad1: 0.0,
            _pad2: 0.0,
        };
        queue.write_buffer(uniform_buffer, 0, bytemuck::cast_slice(&[uniforms]));

        // Prepare instance data for visible nodes
        let mut instances = Vec::with_capacity(nodes_rendered);
        for &node_idx in &visible_nodes {
            let n = &self.scene.nodes[node_idx];
            instances.push(GpuNodeInstance {
                center: [n.x, n.y],
                radius: n.radius,
                color: n.color_rgba,
                is_dimmed: if n.is_dimmed { 1.0 } else { 0.0 },
            });
        }

        // Reallocate or write instance buffer
        if instances.len() > self.instance_buffer_capacity || self.instance_buffer.is_none() {
            let new_cap = (instances.len() * 2).max(64);
            let byte_size =
                (new_cap * std::mem::size_of::<GpuNodeInstance>()) as wgpu::BufferAddress;
            let new_buf = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("nodera_instance_buffer"),
                size: byte_size,
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
            self.instance_buffer = Some(new_buf);
            self.instance_buffer_capacity = new_cap;
        }

        if let Some(ref inst_buf) = self.instance_buffer {
            if !instances.is_empty() {
                queue.write_buffer(inst_buf, 0, bytemuck::cast_slice(&instances));
            }
        }

        // Create an offscreen render target to validate and drive the GPU command queue
        let target_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("nodera_render_target"),
            size: wgpu::Extent3d {
                width: self.config.width.max(1),
                height: self.config.height.max(1),
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        });
        let target_view = target_texture.create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("nodera_command_encoder"),
        });

        {
            let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("nodera_render_pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &target_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            if !instances.is_empty() {
                if let Some(ref inst_buf) = self.instance_buffer {
                    rpass.set_pipeline(pipeline);
                    rpass.set_bind_group(0, bind_group, &[]);
                    rpass.set_vertex_buffer(0, vertex_buffer.slice(..));
                    rpass.set_vertex_buffer(1, inst_buf.slice(..));
                    rpass.draw(0..6, 0..instances.len() as u32);
                }
            }
        }

        queue.submit(Some(encoder.finish()));

        let duration = start.elapsed().as_micros() as u64;

        Ok(RenderStats {
            renderer_name: self.backend_info.name.clone(),
            backend: self.backend_info.backend.clone(),
            nodes_rendered,
            nodes_culled,
            edges_rendered,
            edges_culled,
            render_duration_us: duration,
            is_gpu_accelerated: true,
        })
    }

    fn backend_info(&self) -> RendererBackendInfo {
        self.backend_info.clone()
    }

    fn shutdown(&mut self) {
        self.device = None;
        self.queue = None;
        self.render_pipeline = None;
        self.uniform_buffer = None;
        self.vertex_buffer = None;
        self.instance_buffer = None;
        self.bind_group = None;
        self.instance_buffer_capacity = 0;
    }
}
