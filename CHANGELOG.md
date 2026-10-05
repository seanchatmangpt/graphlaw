# Changelog

All notable changes to this project are documented here, in the
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/) format.

## [Unreleased]

Nothing.

## [26.10.5] - 2026-10-05

Published as crates.io `graphlaw` 26.10.5 (`graphlaw-eyeron` remains 0.7.8; it is independently
versioned upstream-vendored and is not coupled to the workspace calendar version).

publish: BLOCKED:crates_io_credentials

### Added

- `graphlaw::capability_intake` (CASTLE donor intake): a descriptive, side-effect-free registry of
  the donor subjects GraphLaw may wrap or absorb while remaining the sole semantic-law owner.
  `DONORS` carries three projected donors (unrdf `WRAP`, praxis `ABSORB`, mfw `CANDIDATE_WRAP`);
  `donor(repository)` looks one up by exact repository identity; the consts `PROJECTION_SOURCE`
  (the admitting ggen-ecosystem projection, `@50fdfa2`), `OWNER_CAPABILITY`
  (`SEMANTIC_LAW_DERIVATION`) and `AUTHORITY_CEILING` (`CONSTRUCT`) bound the intake, and
  `consequence_authority` is `false` for every donor — projected donor capability never carries
  GraphLaw or CASTLE DO authority.

### Changed

- Capability registry (additive, schema id unchanged): `limits` now lists all 15 engine limits
  (adds `n3_max_derived_facts`, `n3_max_total_bytes`, `n3_max_term_bytes`, `n3_max_match_steps`,
  `max_plan_total_atoms`, `hooks_max_rounds`, `hooks_max_firings`, `hooks_max_state_quads`,
  `max_outstanding_alloc_bytes`), each read from its source constant; new `limit_meta` (scope,
  unit, source, refusal name), `models` (Lease, SignedLease, Receipt, Attestation, Plan, Action,
  PolicyEntry, PolicyOutcome) and `model_enums`; the Turtle projection gains `gac:Model*` and
  limit scope/unit/source. The wasm 256 MiB outstanding-alloc cap is now
  `graphlaw::abi::MAX_OUTSTANDING_ALLOC_BYTES`. Registry and wasm pins in `ARTIFACTS.sha256`
  were regenerated.
- Resource safety: the registry Turtle projection can no longer loop (a leaked cargo-mutants edit
  had made it spin and exhaust memory); Eyeron gains run-wide caps and `ReasonerLimit::ClosureSize`
  / `ClosureFacts` (`graphlaw-eyeron` 0.7.8); N3 refusals report their real limit and ceiling
  (`n3_iterations`, `n3_derived_facts`, `n3_total_bytes`, `n3_term_bytes`, `n3_match_steps`);
  `plan_total_atoms` caps a plan's cumulative atoms; hooks cap the state at 1,000,000 quads; the
  wasm module caps outstanding `gl_alloc` bytes at 256 MiB; `N3Error::Limit` gains `limit` and `max`.
- `plan::Action` gains `pre_not`, `plan::Plan` gains `goal_not`, and `LawError::PlanRefused`
  gains `violated_absent`; every in-repo struct literal (tests, examples) was updated with
  `..Default::default()`. Digests of plans without negation are unchanged.
  Present since v26.9.29 (misfiled as breaking in this section; verified via
  cargo-semver-checks 196/196 against the v26.9.29 baseline).
  Migration notes (introduced in 797056b, "Negative preconditions and negative goals in plan
  admission"):
  - `plan::Action` — exhaustive struct literals no longer compile; add `pre_not` or spread
    `..Default::default()`:

    ```rust
    // before (26.9.28)
    let a = graphlaw::plan::Action {
        name: "open".into(),
        pre: "<urn:d> <urn:p:is> <urn:v:closed> .".into(),
        add: "<urn:d> <urn:p:is> <urn:v:open> .".into(),
        del: "<urn:d> <urn:p:is> <urn:v:closed> .".into(),
    };
    // after (26.10.5)
    let a = graphlaw::plan::Action {
        name: "open".into(),
        pre: "<urn:d> <urn:p:is> <urn:v:closed> .".into(),
        pre_not: "<urn:d> <urn:p:is> <urn:v:locked> .".into(),
        add: "<urn:d> <urn:p:is> <urn:v:open> .".into(),
        del: "<urn:d> <urn:p:is> <urn:v:closed> .".into(),
    };
    // or: ..Default::default() after `del` to keep the old shape.
    ```

  - `plan::Plan` — same shape change for `goal_not` (or prefer `Plan::builder()`, which is
    unaffected):

    ```rust
    // before (26.9.28)
    let p = graphlaw::plan::Plan { actions, goal: goal_nt.into() };
    // after (26.10.5)
    let p = graphlaw::plan::Plan {
        actions,
        goal: goal_nt.into(),
        goal_not: String::new(),
    };
    ```

  - `LawError::PlanRefused` — the variant is `#[non_exhaustive]`-exempt (it is a struct
    variant), so exhaustive matches gain a new binding; add the field or a wildcard:

    ```rust
    // before (26.9.28)
    LawError::PlanRefused { index, action, missing } => { ... }
    // after (26.10.5)
    LawError::PlanRefused { index, action, missing, violated_absent } => { ... }
    // or: LawError::PlanRefused { index, action, missing, .. } => { ... }
    ```
- API stability: growable public enums (`Step`, `Dialect`, `Engine`, `RefusalKind`,
  `PolicyRefusalKind`, `LeaseReason`, `ReceiptReason`, `Ceiling`, `N3Error`, `LawError`,
  `StoreError`, `AttestError`) are `#[non_exhaustive]`; downstream matches need a wildcard arm.
- `#![deny(missing_docs)]` on `graphlaw` and `graphlaw-wasm`; every public item is documented and
  the core types have runnable doctests.

- Documentation fixes accompanying the registry (op reference, refusal codes and limits now agree
  with the emitted registry).
- CI and release workflows: every action reference is pinned to a commit SHA and cargo build/test
  steps use `--locked`.

- Unsigned lease transitions are now `transition_leased_unverified`; the ABI `law` op with a
  `lease` requires `signed_lease` and `trusted_keys` unless `unverified_lease: true` is set.

### Added

- Capability registry (`graphlaw.capability-registry/1`): the Rust table in `src/registry.rs`
  (feature `abi`) is the single source of truth for the 14 ABI ops, their request and response
  fields, dialects, regimes, law steps, refusal codes, limits and authorities. It is emitted to
  `registry/capability-registry.json` (canonical, sorted keys), `registry/capability-registry.ttl`
  (deterministic Turtle projection), with `registry/capability-registry.schema.json` and
  `registry/capability-registry.shapes.ttl` (SHACL, validated by GraphLaw's own `shacl` op in
  tests). The `graphlaw-registry` binary supports `--write`, `--check`, `--print-json`,
  `--print-ttl` and `--print-digests`; CI runs `--check`.
- `capabilities` op additive response fields `registry_schema`, `registry_sha256` and
  `surface_sha256`. Existing fields keep their names and values; `ops`, `rdf_dialects` and
  `other_dialects` in `abi.rs` are now built from the registry rather than a second literal.
  `ABI_VERSION` stays `1`; the change is additive only.
- Op-examples corpus `registry/op-examples.json` (`graphlaw.op-examples/1`, with schema): every op
  has an ok example listed before its refused examples, and every op except `capabilities` has at
  least one refused example. Examples are not part of the registry digest.
- Registry-vs-dispatch court: tests parse the `"<op>" =>` arms of `abi::dispatch` and require them
  to equal the registry op list and order, with the `unknown op` fallthrough intact.
- Native mutation court over the registry and dispatch surface.
- Per-op differential tests (each op against the registry and op-examples) and forward-compat
  tests (unknown request and response fields are tolerated, unknown refusal codes remain
  refusals).
- Release assets: `capability-registry.json`, `capability-registry.ttl` and `op-examples.json`,
  each with a `.sha256`, are attached next to `graphlaw.wasm` (release remains held by
  `RELEASE_HOLD`).
- `docs/api-stability.md`: registry additive-field rule and `surface_sha256` compatibility rule.
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
- CI jobs `semver` (cargo-semver-checks, informational), `deny` (`deny.toml`) and
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
