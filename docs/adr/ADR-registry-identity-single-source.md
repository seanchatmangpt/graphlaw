# ADR: Registry Identity Single Source

- **Status**: Accepted
- **Date**: 2026-11-25

## Context

Op names, dialects, refusal codes and limits were described in several places that could
drift.

## Decision

`src/registry.rs` and `src/registry/definitions.rs` are the one Rust source for the ABI
surface (`src/registry.rs:1-4`). `registry/capability-registry.json`, `.ttl` and
`registry/mcp-manifest.json` are emitted from those tables, never hand-edited. The
`capabilities` op builds its lists from the same tables. Identity is `surface_blake3` and
`registry_blake3` over canonical sorted-key JSON, not file bytes (`:10-13`). Hidden ops are
disjoint from advertised ops (test `hidden_ops_are_disjoint_from_advertised_ops`).

## Consequences

- Changing the surface means editing definitions and regenerating.
- Digests are stable across pretty-printing.
- Elixir `graphlaw_ex/lib/**` and `contracts/abi-manifest.json` are projections.

## See Also

- `docs/explanation/receipt-fields.md`
