# Changelog

All notable changes to this project are documented here, in the
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/) format.

## [Unreleased]

v26.9.29 is intentionally held and unreleased.

### Changed

- Unsigned lease transitions are now `transition_leased_unverified`; the ABI `law` op with a
  `lease` requires `signed_lease` and `trusted_keys` unless `unverified_lease: true` is set.

### Added

- Independent plan admission: `plan::Plan::admit` replays actions and refuses on the first unmet
  precondition or goal, with a receipt per action and a canonical plan digest.
- Receipts and receipt feedback: `receipt::record` / `receipt::require` and
  `Step::RequireReceipt` gate a step on a recorded receipt; native/wasm determinism checks.
- Authority leases (`law::Lease`, `Ceiling`) and an out-of-subject receipt store.
- FOND strong-cyclic policy admission (`policy`, feature `abi`).
- Resource limits on the JSON ABI (`request_bytes`, `json_depth`, `atoms_per_field`,
  `plan_actions`, `policy_entries`, `n3_iterations`).
- Structured refusal `details` objects with a stable `code` on ABI refusals.
- Signed receipts and leases: Ed25519 `attest`, `receipt::record_signed` / `require_signed`,
  `Step::RequireSignedReceipt`, `SignedLease` judged against an injected `Clock` with a skew bound.
- `graphlaw-verify` binary: replays a plan and checks the receipt chain and attestations offline.
- Wasm lease-boundary test and an upstream-ready vendored clock patch.
- `examples/`, `docs/quickstart.md`, `docs/refusals.md`, `SECURITY.md`, `CONTRIBUTING.md`.

## [26.9.28] - 2026-09-28

### Changed

- Semantic reset: GraphLaw delegates RDF/SPARQL/SHACL/ShEx/Datalog/entailment to PurRDF and N3 to
  Eyeron; the bespoke parser, triplestore, reasoner and validators were removed.
- Crate renamed `praxis-graphlaw` to `graphlaw`.

### Added

- Law-state layer, dialect router, differential oracle, knowledge hooks, JSON ABI and a
  self-contained WASI module; vendored Eyeron as `graphlaw-eyeron`.
