# ADR-0003: No Axum in V1

## Status

Accepted.

## Decision

Do not introduce Axum until a real HTTP boundary is required.

## Rationale

There is no web client, remote service, or local API requirement in V1.

## Future trigger

Add an HTTP layer if one or more of these become requirements:
- web client
- separate conversion service
- remote conversion
- external integrations
- local API for plugins
