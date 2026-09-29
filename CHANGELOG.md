# Changelog

All notable changes to this project are documented here, in the
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/) format.

## [Unreleased]

v26.9.29 is intentionally held and unreleased.

### Changed

- BREAKING (unreleased): `plan::Action` gains `pre_not`, `plan::Plan` gains `goal_not`, and
  `LawError::PlanRefused` gains `violated_absent`; every in-repo struct literal (tests, examples)
  was updated with `..Default::default()`. Digests of plans without negation are unchanged.
- API stability: growable public enums (`Step`, `Dialect`, `Engine`, `RefusalKind`,
  `PolicyRefusalKind`, `LeaseReason`, `ReceiptReason`, `Ceiling`, `N3Error`, `LawError`,
  `StoreError`, `AttestError`) are `#[non_exhaustive]`; downstream matches need a wildcard arm.
- `#![deny(missing_docs)]` on `graphlaw` and `graphlaw-wasm`; every public item is documented and
  the core types have runnable doctests.

- Unsigned lease transitions are now `transition_leased_unverified`; the ABI `law` op with a
  `lease` requires `signed_lease` and `trusted_keys` unless `unverified_lease: true` is set.

### Added

- Negative preconditions and goals: `Action::pre_not` / `Plan::goal_not` (triples that must be ABSENT,
  PDDL `(not p)`, closed-world over ground atoms), `ActionBuilder::requires_not`,
  `PlanBuilder::requires_not` / `goal_not`, ABI `pre_not` / `goal_not` fields, and
  `details.violated_absent` on `PlanRefused`. Included in the plan digest only when non-empty.
  `tests/plan_negation.rs`.
- `plan::Triple` (validated IRIs, escaped literals), `plan::PlanBuilder` / `Plan::builder()`,
  `plan::ActionBuilder` / `Action::builder()` and `plan::TripleError`; `examples/quickstart_plan.rs`
  uses them. Struct-literal construction is unchanged.
- `docs/api-stability.md` (stability, deprecation, MSRV and `ABI_VERSION` policy),
  `tests/api_stability.rs`, `tests/plan_builder.rs`.
- CI jobs `semver` (cargo-semver-checks, informational while unreleased), `deny` (`deny.toml`) and
  `audit` (cargo-audit).

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
