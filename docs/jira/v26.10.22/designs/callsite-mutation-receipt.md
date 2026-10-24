# Callsite Mutation Receipt — UNKNOWN-STEP-CAPABILITY court (lane M05)

Date: 2026-10-22. Defect class K10: the typed-refusal test killed only mutation
A (helper revert in isolation); reverting the law.rs:1033 transition callsite
(or the plan.rs:607 `fn authority()`) to a fallback authority broke no test.

## Design

1. `src/law.rs`: new `#[doc(hidden)]` test-only probe step
   `Step::TestCapabilityProbe(&'static str)` on the `#[non_exhaustive]` enum
   (wildcard matches unaffected — `tests/api_stability.rs` still passes). It
   carries an arbitrary capability string, maps to `Ceiling::Observe`, name
   `test:capability-probe`, and is an identity transition (`self.clone()`), so
   the only observable effect is the authority resolution at the law.rs
   transition callsite. This is the minimal surface that lets a court drive the
   PUBLIC `LawState::transition` path with a capability absent from
   `BACKEND_AUTHORITIES`.
2. `tests/law_state.rs::transition_with_unknown_capability_refuses_typed_end_to_end`
   drives `transition(&Step::TestCapabilityProbe("court-unknown-capability"))`
   and asserts the typed `[UNKNOWN-STEP-CAPABILITY]` `LawError::Refused`
   naming the offending capability; a second arm drives a DECLARED capability
   ("RDF/RDFS/OWL-RL entailment") through the same path and asserts the real
   authority readback (`purrdf::entail`, identity child, added = 0). That arm
   is the mutant-killer: any fallback/laundering of unknown capabilities makes
   the first arm return Ok and fail.
3. `scripts/mutation_court.sh`: new leg `callsite-revert`
   (`--leg callsite-revert` for leg-only mode; also runs as the pre-leg of
   run/check modes before cargo-mutants starts). Shim-court protocol:
   sha256 baseline -> python shim replaces the body of
   `authority_for_capability` with an unconditional
   `Ok(BACKEND_AUTHORITIES[0])` (marked SHIM-COURT) -> court must FLIP
   (test FAILED, exit != 0; compile aborts are rejected loudly, not counted as
   flips) -> restore -> byte-exact sha256 verification -> post-restore
   sanity run. Vacuous leg (court still passes under the shim) = exit 7.
   Leg-only mode is dispatched BEFORE the cargo-mutants dirty-tree refusal
   because the leg is self-snapshotting and byte-exact-restoring. Restores use
   plain `cp` (NOT `cp -p`): a preserved mtime let cargo keep the shim-compiled
   rlib as fresh and produced a false post-restore result — this failure was
   witnessed and is now structurally excluded.

## Witnessed execution (2026-10-22, real outputs)

- Court on clean tree: `cargo test --test law_state` -> 8 passed; 0 failed.
- Leg under shim: `transition_with_unknown_capability_refuses_typed_end_to_end`
  FAILED (exit 101) — the flip is the non-vacuity witness.
- Restore: byte-exact, sha256 `ac1ac95533abf8abf05e6d4f6608bb8507ba29a0023e32978be984f6a2eb8bb0` (historical)
  (verified twice: leg output and independent `sha256sum` after the leg).
- Leg exit: 0. Vacuous and compile-abort paths both exit 7 typed.

## Disclosures (compile-freeze SLA)

- `src/law.rs` is outside the originally stated sql-rdf file lease; the
  `#[doc(hidden)]` probe-step constructor was the dispatch-sanctioned minimal
  surface addition, applied here and disclosed.
- Two transient shared-tree events, neither caused by this lane's files:
  a concurrent lane's in-flight `tests/no_mock_markers.rs` briefly broke
  `cargo check --tests` (resolved by its owner within the SLA window), and
  `src/bin/graphlaw_ocel_export.rs` was briefly non-compiling (bounded-poll
  waited for its owner's fix; no edit applied by this lane).
- One DISK-REFUSED window (12 GiB free < 20 GiB gate) was resolved by the
  coordinator's lane-root purge; bounded disk polls were used, no fixed sleeps
  as coordination.
