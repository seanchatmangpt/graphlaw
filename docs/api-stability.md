# API stability

GraphLaw v26.9.29 (unreleased). This page states what downstream crates and JSON-ABI hosts may
rely on, how deprecations work, and how the minimum supported Rust version moves.

## What is stable

- **Public types and functions** in `graphlaw::{law, plan, policy, receipt, receipt_store, attest,
  dialect, hooks}` and, with `--features abi`, `graphlaw::abi`. Removing or changing the signature
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
- **Re-exports** `graphlaw::purrdf` and `graphlaw::eyeron` are unstable escape hatches; their
  API is that of the pinned upstream version (`purrdf =2.0.2`).

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

`graphlaw::abi::ABI_VERSION` (currently `1`) is reported in every `info` response as `abi` and
`abi_version`.

- It is bumped on any incompatible request or response change (removed or retyped field, changed
  meaning of an existing code).
- Adding an optional request field, a new op, a new response field or a new refusal `code` does
  **not** bump it.
- A host must check `abi_version` equals the value it was written against and refuse to run
  otherwise. The wasm module and the crate are released together, so their `ABI_VERSION` match.

## Checks

- `cargo semver-checks` runs in CI (informational while v26.9.29 is unreleased and expected to
  differ from 26.9.28).
- `#![deny(missing_docs)]` and `cargo doc -D warnings` keep every public item documented.
- `cargo deny check` and `cargo audit` gate the dependency tree (`deny.toml`).
