# ADR: Lease Policy Default

- **Status**: Accepted
- **Date**: 2026-11-25

## Context

Lease-less rdfs/shacl/n3 chains exist in the Rust kernel (`src/abi/lease.rs:9-11`). Hosts that
need authority on every `law` request had no typed way to demand it.

## Decision

Rust keeps `NoneAllowed` as the process default and adds `set_lease_policy` plus request-level
`"lease_policy": "required"` (tighten-only). The Elixir wrapper defaults to `:required`
(`graphlaw_ex/config/config.exs:3`); test env uses `:none_allowed`. Missing lease under
`Required` refuses with `LeaseRequired`.

## Consequences

- Wrapper users fail closed by default; Rust embedders must opt in.
- Wrapper test suites must set `:none_allowed` explicitly.
- The two defaults differ by design; docs must state both.

## See Also

- `docs/explanation/lease-policy-and-fail-closed-defaults.md`
