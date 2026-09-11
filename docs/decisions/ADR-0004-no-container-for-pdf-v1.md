# ADR-0004: No Podman/Docker Dependency for PDF V1

## Status

Accepted.

## Decision

Start with a Rust-native PDF conversion backend.

## Rationale

The current target is a text-heavy, selectable-text PDF. A container is not inherently required.

## Future trigger

Introduce an optional containerized backend if:
- Rust-native extraction quality is inadequate
- a mature external engine provides substantially better output
- users need advanced OCR/layout reconstruction

The PDF abstraction must allow this without changing the UI.
