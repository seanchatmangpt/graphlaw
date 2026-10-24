# INDEX — milestone window v26.10.23

Registered 2026-10-24. This window is the post-v26.10.22 wave: ADRs, design
specs, release gates, and the compliance program. Every markdown doc in
`docs/jira/v26.10.23/` is registered below — 23 docs, read from disk at use
time. Statuses are the docs' own self-declared standing on disk, not verdicts
by this index.

`PUBLISH-STATUS.md` landed in this window at 2026-10-24 03:20, while this
register was being written; it is registered below (the dispatch listed it
as "if landed").

## Register: roadmap and release gates

- [ROADMAP.md](ROADMAP.md) — Active plan.
  Milestone v26.10.23 wave plan: wasm gap closures + Phase-2 CanonArrow,
  derived from the approved plan; v26.10.22 witnessed end-to-end on artifact
  identity `472d5557…`, solo mutation-court verdict outstanding.
- [GO-NO-GO-v26.10.22.md](GO-NO-GO-v26.10.22.md) — NO-GO (v3, read window
  2026-10-24 00:45–01:15 PDT, HEAD `907e663`).
  Final go/no-go for the v26.10.22 tag: red on four evidence-cited grounds —
  uncommitted security-wave residue, mutation sweep without a completed
  verdict (exit 137 at 18/88), last FULL battery RED (2 failed), open R-1/R-2
  pre-tag gates.
- [TAG-PREP.md](TAG-PREP.md) — PREPARED (2026-10-24).
  v26.10.22 annotated-tag preparation: version triple verified on disk
  (`26.10.22` / wasm / eyeron), eyeron publishes as `0.7.9` (0.7.8 collision);
  no tag commands run by the lane.
- [PUBLISH-STATUS.md](PUBLISH-STATUS.md) — Recorded (2026-10-24, HEAD
  `4df19b5`).
  Publish state of v26.10.22 by live query: git tag present at `29c6d98`,
  npm `26.10.14` matches until Trusted Publisher lands, crates.io at
  eyeron `0.7.8` / graphlaw `26.10.21` with local `0.7.9` / `26.10.22`
  publishable; npm publish CI run `40084544193` RED (E404, typed-blocked on
  operator act).

## Register: ADRs

All four ADRs are Proposed; none has a flip witness or landed code yet.

- [ADR-wasmi-retirement.md](ADR-wasmi-retirement.md) — Proposed (2026-10-23).
  Retire the wasmi execution path (ERRC — Eliminate): one PR, no code changes
  before the flip witness lands.
- [ADR-crate-splitting.md](ADR-crate-splitting.md) — Proposed.
  Split the law kernel into workspace crates; workspace restructuring only,
  no behavioral change to admission, SQL compilation, evaluation, receipts.
- [ADR-release-lease.md](ADR-release-lease.md) — Proposed (2026-10-23).
  Attest-signed release lease replacing the RELEASE_HOLD console-variable
  gate; gate semantics for `release.yml`, `publish-npm.yml`, `js.yml`;
  prose only.
- [ADR-egress-op.md](ADR-egress-op.md) — Proposed (2026-10-23).
  Promote egress to a first-class ABI op; consumes the ERRC-raise findings in
  ROADMAP.md and the CanonArrow Phase 2 wire format
  (`v26.10.22/ADR-canonarrow-dual-witness-egress.md`).

## Register: design specs

- [design-pre-dispatch-drift-gate.md](design-pre-dispatch-drift-gate.md) —
  PROPOSED (DESIGN-PDDG-01).
  Pre-dispatch drift gate: one PreToolUse hook reusing the attest-hook
  pattern; no `src/` or manifest changes.
- [design-hot-file-lease-locks.md](design-hot-file-lease-locks.md) — DRAFT.
  Mechanical hot-file lease locks (ERRC-Reduce Fix 2): replace a prose
  invariant with a mechanism; relates to the UNWIRED `src/abi/lease.rs` L67
  draft.
- [design-env-admission-court.md](design-env-admission-court.md) — DRAFT.
  Pre-wave environment-admission court (ERRC intervention I-3): ~20 recurring
  BLOCKED rows in the v26.10.22 ledger were environmental, not logical; lanes
  must pass an env court before dispatch.
- [design-dev-loop-self-hosting.md](design-dev-loop-self-hosting.md) — DRAFT.
  Dev-loop self-hosting, the process-mining endgame: the repo emits events
  but never admits its own dev loop through the law kernel.
- [design-sniff-fixes.md](design-sniff-fixes.md) — Spec (unimplemented).
  Sniff-layer fixes for the G8 root-cause class behind `corpus_conformance`
  failures: one file (`src/dialect/sniff.rs`) plus a new
  `tests/sniff_court.rs`.
- [design-read-frame-bound.md](design-read-frame-bound.md) — SPEC
  (unimplemented).
  `read_frame` length-prefix bound (G4) in both stdio runners
  (`graphlaw_run.rs`, `graphlaw_run_eh.rs`); current behavior verified on
  disk.
- [ATTEST-API.md](ATTEST-API.md) — DESIGN (nothing claimed landed).
  `graphlaw-attest` verification CLI and the self-contained replay-JSON wire
  shape, grounded in `src/attest.rs::receipt_fold` and `src/egress/arrow.rs`
  as on disk.

## Register: compliance program

Six matrices, one closure plan, one consolidated verdict (the dispatch
projected 7 matrices; disk holds 6 — all are registered).

- [compliance-eu-ai-act.md](compliance-eu-ai-act.md) — Matrix.
  EU AI Act Art. 9 risk-management evidence matrix, GraphLaw v26.10.22,
  opening with the system-classification argument.
- [compliance-eu-ai-act-logging.md](compliance-eu-ai-act-logging.md) —
  Matrix.
  Art. 12 record-keeping (automatic event logging via ExecutionReceipt) and
  Art. 11 technical documentation mapping.
- [compliance-eu-ai-act-robustness.md](compliance-eu-ai-act-robustness.md) —
  Matrix.
  Art. 15 accuracy, robustness, cybersecurity mapping, incl. the memory-wall
  plateau under adversarial load.
- [compliance-nist-gov-map.md](compliance-nist-gov-map.md) — Matrix.
  NIST AI RMF GOVERN + MAP functions mapped to repo evidence.
- [compliance-nist-measure-manage.md](compliance-nist-measure-manage.md) —
  Matrix.
  NIST AI RMF MEASURE + MANAGE functions, incl. Divan performance baselines
  with real output embedded.
- [compliance-transparency.md](compliance-transparency.md) — Mapping.
  Compliance and transparency map for v26.10.23: public docs completeness,
  artifact transparency, claims audit trail; read-only lane.
- [compliance-gap-closure-plan.md](compliance-gap-closure-plan.md) — Plan
  (as-of 2026-10-23).
  Gap closure plan for EU AI Act + NIST AI RMF against graphlaw v26.10.22;
  opens with the subject-classification argument.
- [compliance-final-verdict.md](compliance-final-verdict.md) — Verdict.
  Consolidated due-diligence evidence verdict (not a certificate), subject
  HEAD `7e92175`; Art. 9 process shape graded PARTIAL with residual register
  and typed refusals cited.

## Counts

- Docs on disk in this window: 23 (all registered above; this INDEX.md is the
  24th .md file and is the register itself).
- PUBLISH-STATUS.md: landed 2026-10-24 03:20 — registered.
