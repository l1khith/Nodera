# Architecture

## Dependency direction

The dependency direction should be:

```text
desktop UI
    ↓
application/use cases
    ↓
domain/core
    ↓
adapters (filesystem, database, search, PDF)
```

The domain must not depend on Dioxus.

## Suggested Cargo workspace

```toml
[workspace]
members = [
  "crates/nodera-core",
  "crates/nodera-markdown",
  "crates/nodera-index",
  "crates/nodera-pdf",
  "crates/nodera-desktop",
]
resolver = "2"
```

## Why no Axum?

There is no web client and no remote API requirement in V1.

Adding Axum would create:

```text
Dioxus → HTTP → Axum → Core
```

without a demonstrated benefit.

If a future local/remote service is required, introduce it as a separate adapter.

## Why no Tauri?

The selected direction is native Dioxus Desktop. Tauri would add another desktop runtime layer without solving a current requirement.

## Why no Podman?

Podman is useful when an external engine has complex dependencies. It is not necessary for the initial native Rust architecture.

The PDF converter should be an interface so a containerized engine can be added later if needed.

## Why Markdown files?

- portable
- inspectable
- version-control friendly
- user-owned
- easy backup
- compatible with existing Markdown tooling

## Why SQLite?

SQLite provides durable structured indexing without making the user's notes database-first.

## Why Tantivy?

It provides a local full-text search engine suitable for a desktop application and can be rebuilt from source documents.

## Why a converter interface?

PDF extraction quality varies. Keeping a stable interface lets the product start with a Rust-native implementation and later support more advanced engines.

```rust
trait PdfConverter {
    fn convert(
        &self,
        input: &Path,
        output: &Path,
        options: ConversionOptions,
    ) -> Result<ConversionResult>;
}
```

## Security boundary

Treat imported Markdown and frontmatter as data.

Never execute:
- Markdown
- embedded scripts
- shell commands from note content

External process execution, if ever introduced, must be explicit and validated.

## Graphics & Knowledge Graph Architecture

Nodera decouples knowledge graph simulation and spatial representation from rasterization:

```text
Simulation Engine (Barnes-Hut, QuadTree)
    ↓
GraphScene Geometry (Packed Node & Edge Instances, Viewport)
    ↓
Spatial Frustum Culling (World-space AABB)
    ↓
GraphRenderer Interface
    ├─ GpuGraphRenderer (wgpu v24, WGSL, Vulkan / DX12 / Metal)
    └─ CpuGraphRenderer (Zero-crash fallback, 4-tier label LOD, Software SVG)
```

### Why a hybrid GPU/CPU renderer?
- **Zero-Crash Portability**: Graphics drivers vary widely across Windows, Linux, and macOS. If a host system has missing or outdated GPU drivers, Nodera automatically falls back to CPU software rendering without panicking or dropping the session.
- **Decoupled Scene Memory**: Node and edge instances are stored in packed, contiguous buffers (`GraphNodeInstance`, `GraphEdgeInstance`), bypassing DOM overhead.
- **Non-blocking UI Thread**: Hardware enumeration and pipeline validation run off the main UI thread via Tokio worker pools.
- **Detailed Reference**: See [docs/graphics.md](file:///c:/Users/ailik/funProjects/rustProjects/nodera/docs/graphics.md).

