# ADR-0001: Native Dioxus Desktop

## Status

Accepted for V1.

## Context

Nodera is a desktop-only application with no web client requirement.

Potential stacks considered:
- Dioxus Desktop
- Tauri + web frontend
- other native Rust GUI frameworks

## Decision

Use Dioxus Desktop for the initial UI.

## Rationale

- Rust-first application
- no need for a web client
- avoids a second desktop runtime layer
- keeps UI and core integration straightforward
- supports a component-based desktop UI

## Consequences

Positive:
- simpler V1 architecture
- fewer runtime layers
- Rust-first codebase

Negative:
- Dioxus Desktop ecosystem differs from mature web UI ecosystems
- some advanced UI capabilities may require framework-specific work

Revisit if:
- desktop platform support becomes inadequate
- a web client becomes a firm product requirement
