# API stability

GraphLaw's workspace version is 26.10.21 (latest released); the capability-registry schema
`graphlaw.capability-registry/1` and `ABI_VERSION = 1` were frozen at v26.9.29 and remain additive-only.
This page states what downstream crates and JSON-ABI hosts may
rely on, how deprecations work, and how the minimum supported Rust version moves.

## What is stable

- **Public types and functions** in `graphlaw::{law, plan, policy, receipt, receipt_store, attest,
  dialect, hooks, capability_intake, qualification, receipt_chain, registry, smon}` and, with
  `--features abi`, `graphlaw::abi`. Removing or changing the signature
  of any of them requires a semver-major release (`26.x` calendar versions: a change of the
  first component or an explicit CHANGELOG "Breaking" entry).
- **Growable enums are `#[non_exhaustive]`**: `Step`, `Dialect`, `Engine`, `RefusalKind`,
  `PolicyRefusalKind`, `LeaseReason`, `ReceiptReason`, `Ceiling`, `N3Error`, `LawError`,
  `StoreError`, `AttestError`, `TripleError`. Adding a variant is a minor change; downstream
  `match` expressions must carry a wildcard arm. `tests/api_stability.rs` is a downstream-crate
  falsifier for this, and each enum's rustdoc has a doctest showing the `_ =>` arm.
- **Structs callers construct with literals stay constructible** (`Lease`, `Plan`, `Action`,
  `Problem`, `Entry`, `Outcome`): adding a field to them is a breaking change and is called out
  in the CHANGELOG. Prefer `Plan::builder()`, `Action::builder()` and `Triple` so callers never
  hand-write N-Triples.
- **ABI JSON codes**: the `details.code` values listed in `docs/refusals.md` (for example
  `PlanRefused`, `ReceiptRequired`, `LeaseRefused`) and the stable `as_str()` names of
  `LeaseReason` and `ReceiptReason` never change meaning. New codes may be added; hosts must treat
  an unknown code as a refusal.
- **Re-exports** `graphlaw::purrdf`, `graphlaw::eyeron`, and `graphlaw::duckdb_eval` (with `frozen_duckdb`) are escape hatches; their
  API is that of the pinned upstream version (`purrdf =2.0.2`, `frozen-duckdb 2.0`).

## Deprecation policy

1. A deprecated item gets `#[deprecated(since = "…", note = "use X")]` and a CHANGELOG line under
   "Deprecated".
2. It keeps working for at least one full minor release before removal.
3. Removal is listed under "Removed" with the replacement.

## MSRV policy

`rust-version` in `Cargo.toml` is 1.96. It is raised only in a minor release, and every raise gets
a CHANGELOG line ("MSRV: 1.96 -> 1.9x"). CI builds on the pinned toolchain in
`rust-toolchain.toml`.

## ABI_VERSION contract

`graphlaw::abi::ABI_VERSION` (currently `1`) is reported in every `capabilities` response as `abi` and
`abi_version`.

- It is bumped on any incompatible request or response change (removed or retyped field, changed
  meaning of an existing code).
- Adding an optional request field, a new op, a new response field or a new refusal `code` does
  **not** bump it.
- A host must check `abi_version` equals the value it was written against and refuse to run
  otherwise. The wasm module and the crate are released together, so their `ABI_VERSION` match.

### Capability registry and surface digest

`registry/capability-registry.json` (schema `graphlaw.capability-registry/1`) is emitted from
`src/registry.rs` and described by the `capabilities` response fields `registry_schema`,
`registry_blake3` and `surface_blake3`.

- **Additive-field rule**: registry fields, ops, dialects, refusal codes, limits and enum
  values may be added without bumping `ABI_VERSION` or the schema id. Removing or retyping a
  registry field, or changing the meaning of an existing name, is an ABI break and bumps
  `ABI_VERSION`. `enum` lists are informational; hosts must not reject values outside them.
- **Surface digest**: `surface_blake3` is BLAKE3 over the canonical JSON (sorted keys, compact) of
  `{abi_version, ops, other_dialects, rdf_dialects}` with ops and dialects in registry order. It
  changes only when the public op or dialect surface changes.
- **Older engines**: engines older than v26.9.29 do not report `registry_schema`,
  `registry_blake3` or `surface_blake3`. Consumers must tolerate their absence: compute
  `surface_blake3` host-side from the live `capabilities` response and treat `registry_blake3`
  as unknown.
- **Emission**: the JSON and Turtle files are generated; `graphlaw-registry --check` fails when the
  committed files differ from the Rust table.

### Digest pins vs versions (W09, 2026-10-20)

The wasm artifact digest pins (`registry/ARTIFACTS.blake3`, `priv/graphlaw.wasm.blake3`) and the
crate versions move on independent axes:

- **`ABI_VERSION` bumps only on wire-incompatible changes** to the JSON-over-linear-memory
  contract (removed or retyped field, changed meaning of an existing code). Calendar version
  changes and digest re-pins are never, by themselves, an ABI bump.
- **A crate calendar break without an ABI break ships a new digest pin, not an `ABI_VERSION`
  bump.** Rebuild the wasm artifact, then the coordinator re-pins both `registry/ARTIFACTS.blake3`
  and `priv/graphlaw.wasm.blake3` in the same commit as the version bump.
- **Never ship a crate release whose digest pin predates its own version.** If the pins carry
  `pin-source=<older>` (as the current header does: minted at 26.10.5, crate now 26.10.21), the
  release is blocked until a rebuild produces a fresh digest at the released source state.

## Checks

- `cargo semver-checks` runs in CI on every push and PR (job `semver` in
  `.github/workflows/ci.yml`).
- `#![deny(missing_docs)]` and `cargo doc -D warnings` keep every public item documented.
- `graphlaw-registry --check` keeps the committed registry files equal to the Rust table.
- `cargo deny check` and `cargo audit` gate the dependency tree (`deny.toml`).
