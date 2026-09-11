# Development Setup

## 1. Required

- Rust toolchain
- Cargo
- Git
- Dioxus CLI/tooling appropriate to the selected Dioxus release
- Platform dependencies required by Dioxus Desktop

Do not pin exact tool versions in this planning document until the implementation branch establishes a tested version matrix.

## 2. Initial commands

Conceptual:

```bash
git clone <repository>
cd nodera

cargo check --workspace
cargo test --workspace
```

Dioxus development commands should follow the version selected in `Cargo.toml`.

## 3. Local development

Recommended workspace commands:

```bash
cargo fmt --all
cargo clippy --workspace --all-targets
cargo test --workspace
```

Run the desktop application using the selected Dioxus CLI.

## 4. Repository structure

Keep:
- source
- docs
- test fixtures
- scripts

Do not commit:
- user vaults
- generated search indexes
- large private PDFs
- local `.nodera` state
- secrets

## 5. Git hooks / CI

CI should run:
1. formatting check
2. clippy
3. unit/integration tests
4. build
5. selected fixture tests

## 6. Development principles

- Prefer small commits.
- Keep core logic independent of UI.
- Add tests with every parser/indexing change.
- Document architectural decisions.
- Avoid speculative abstractions.
