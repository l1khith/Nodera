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
