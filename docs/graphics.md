# Nodera Hardware-Accelerated Graphics Architecture

**Document Version**: 1.0.0  
**Current Release**: Nodera v0.6.1  
**Maintainer**: Nodera Graphics & Core Architecture Working Group  

---

## 1. Executive Summary & Architectural Philosophy

Nodera's 2D knowledge graph visualizes human thought structures (wikilinks, tags, notes) and software project symbol dependencies (AST functions, structs, traits, modules). On large vaults and codebases exceeding 1,000 nodes and 3,000 edges, legacy SVG/DOM rendering causes DOM element bloat, layout thrashing, and dropped frames.

To ensure instant responsiveness and 60+ FPS navigation on large-scale graphs, Nodera incorporates a **hybrid hardware-accelerated graphics architecture** powered by modern, safe Rust primitives:

```
                                 NODERA DESKTOP
                                        │
                         ┌──────────────┴──────────────┐
                         ▼                             ▼
                 [ Dioxus UI Layer ]          [ Physics Simulation ]
                (Controls, Inspector,         (Barnes-Hut QuadTree,
                 Settings & Overlays)          Off-Thread Relaxer)
                         │                             │
                         └──────────────┬──────────────┘
                                        │
                                        ▼
                             [ GraphScene Abstraction ]
                             (Packed Instanced Nodes,
                              Packed Instanced Edges,
                              Frustum Spatial Culling)
                                        │
                         ┌──────────────┴──────────────┐
                         ▼                             ▼
             [ GpuGraphRenderer ]             [ CpuGraphRenderer ]
              (wgpu v24 Pipeline,            (Zero-Crash Fallback,
               WGSL Instanced Quads,          4-Tier Label LOD,
               Vulkan / DX12 / Metal)         Spatial-Culled SVG)
```

### Core Design Rules
1. **Never Force GPU Blindly**: If a device lacks modern graphics drivers or operates in a headless/virtualized environment, Nodera falls back smoothly to CPU software rendering without panicking or crashing.
2. **Decoupled Scene Architecture**: All node and edge geometry is packaged into an instanced `GraphScene` representation completely independent of the target rasterizer.
3. **Zero UI Thread Blocking**: Hardware adapter probing and shader pipeline initialization execute asynchronously off the UI thread via Tokio worker tasks.
4. **Transparent Diagnostics**: The active adapter, backend API, device kind, driver string, and hardware acceleration status are exposed in both the settings modal and the contextual inspector.
5. **Cross-Platform Portability**: Compatible with Windows (DirectX 12, Vulkan), Linux (Vulkan), and macOS (Metal).

---

## 2. Adapter Detection, Scoring & Fallback Policy

During startup, Nodera's `AdapterDetector` safely probes available system graphics devices via `wgpu::Instance`. To guarantee system stability, enumeration is executed with panic protection (`std::panic::catch_unwind`) and isolated thread boundaries.

### Scoring Policy
When multiple graphics adapters are present (e.g. laptop hybrid graphics with an Intel integrated GPU and an NVIDIA discrete GPU), adapters are scored according to hardware capability:

| Metric | Condition | Score Weight |
|---|---|---|
| **Device Type** | Discrete GPU (`wgpu::DeviceType::DiscreteGpu`) | `+1000 pts` |
| | Integrated GPU (`wgpu::DeviceType::IntegratedGpu`) | `+500 pts` |
| | Virtual GPU (`wgpu::DeviceType::VirtualGpu`) | `+250 pts` |
| | Software Rasterizer (`wgpu::DeviceType::Cpu`) | `+50 pts` |
| **Backend API** | Vulkan, Metal, DirectX 12 | `+200 pts` |
| | OpenGL / WebGL / WebGPU | `+50 pts` |
| **Texture Limit** | $\min(50, \lfloor \text{Max 2D Dimension} / 1024 \rfloor)$ | `0 - 50 pts` |

The highest-scoring adapter is selected as the primary backend. If device initialization fails (e.g., outdated Vulkan loader or driver timeout), Nodera automatically falls back to `CpuGraphRenderer` with a clear status message.

---

## 3. GraphScene Abstraction & Instancing Pipeline

Instead of allocating thousands of heavy DOM nodes, graph data is transformed into a compact, contiguous memory representation:

### Node & Edge Instances
- `GraphNodeInstance`: 48-byte packed struct containing coordinates `(x, y)`, radius, RGBA float color, centrality, degree, and hover/selection/dimming state flags.
- `GraphEdgeInstance`: 40-byte packed struct containing endpoint coordinates `(x1, y1, x2, y2)`, color, stroke width, and dimming state.

### Frustum Spatial Culling
Before submission to the draw pass, `GraphScene::spatial_cull(pixel_margin)` computes the visible world-space axis-aligned bounding box (AABB):

$$\text{min\_x} = \frac{-\text{pan\_x} - \text{margin}}{\text{zoom}}, \quad \text{max\_x} = \frac{\text{width} - \text{pan\_x} + \text{margin}}{\text{zoom}}$$
$$\text{min\_y} = \frac{-\text{pan\_y} - \text{margin}}{\text{zoom}}, \quad \text{max\_y} = \frac{\text{height} - \text{pan\_y} + \text{margin}}{\text{zoom}}$$

Nodes outside this bounding box are discarded in microseconds, reducing downstream draw workload by $60\%\text{--}95\%$ when zoomed in.

### 4-Tier Label Level of Detail (LOD)
To eliminate text rasterization bottlenecks:

| Zoom Level | LOD Tier | Presentation Behavior |
|---|---|---|
| $\text{Zoom} \ge 0.75$ | `LabelLod::Full` | Full node title rendered with crisp typography |
| $0.45 \le \text{Zoom} < 0.75$ | `LabelLod::Truncated` | Truncated to 12 characters with ellipsis |
| $0.25 \le \text{Zoom} < 0.45$ | `LabelLod::MajorHubsOnly` | Text rendered only for top $10\%$ degree hubs and selected nodes |
| $\text{Zoom} < 0.25$ | `LabelLod::None` | Zero text rasterization; pure dot clusters for peak FPS |

---

## 4. Hardware Pipeline (WGSL & WGPU)

The `GpuGraphRenderer` utilizes a custom WebGPU Shading Language (WGSL) shader pipeline:

- **Instanced Circle Quad Geometry**: A single 6-vertex quad `[-1.0, 1.0]` is bound once and reused for all visible nodes.
- **Antialiased Edge Feathering**: Fragment shader computes radial distance from center:
  $$\text{dist}^2 = u^2 + v^2$$
  Fragments outside the unit circle are discarded; border smoothing is computed via `smoothstep(0.85, 1.0, dist_sq)`.
- **Dynamic Instance Buffer Growth**: The instance buffer automatically doubles in size when scene density increases, avoiding per-frame GPU reallocations.

---

## 5. UI Surfaces & Diagnostic Transparency

Nodera provides three user-facing control and diagnostic touchpoints:

### 1. Settings Modal — "Graphics & GPU" Tab
- **Renderer Selection Cards**: Select between `Auto (Recommended)`, `Force GPU`, and `Force CPU Fallback`.
- **Active Adapter Status Card**: Displays selected adapter name, backend API, device category, driver information, and max 2D texture size.
- **Hardware Acceleration Badge**: Green `⚡ HARDWARE ACCELERATED` or amber `⚙️ CPU SOFTWARE FALLBACK`.
- **Discovered Adapters Table**: Full enumeration of all discovered host devices with performance scores.
- **Re-scan Hardware**: Trigger live re-probing of system adapters.

### 2. Contextual Inspector — Section 6 "Graphics & GPU"
- Collapsible drawer in the right inspector while in Graph mode.
- One-click toggle between `Auto`, `GPU`, and `CPU` modes.
- Direct link to open the full diagnostics modal.

### 3. Canvas Status Badge
- Bottom-left status badge on the graph canvas displaying the live renderer:
  `⚡ GPU (Dx12)` or `⚙️ CPU Fallback`.

---

## 6. Verification & Quality Gates

The graphics architecture is validated via dedicated unit tests and benchmarks:

- `test_adapter_detection_and_diagnostics`: Validates safe adapter probing across host platforms.
- `test_adapter_kind_display`: Validates adapter classification strings.
- `test_viewport_coordinate_invariants`: Verifies world-to-screen and screen-to-world round-trip precision.
- `test_spatial_frustum_culling`: Verifies accurate culling of out-of-bounds nodes and edges.
- `test_label_lod_thresholds`: Verifies smooth LOD transitions.
- `test_cpu_renderer_lifecycle`: Verifies zero-crash CPU fallback execution.
- `test_renderer_factory_preferences`: Verifies user preference routing and fallback safety.
- `test_graphics_diagnostics_serialization`: Verifies JSON persistence and IPC integrity.
