# ADR: Retire the wasmi Execution Path

- **Status**: Proposed
- **Date**: 2026-10-23
- **Milestone**: v26.10.23
- **Class**: ERRC — Eliminate
- **Scope**: one PR; no code changes before the flip witness lands

## Context

The wasmi-based execution path is dead weight carried by the tree:

- **Upstream blocker**: wasmi 2.0.0 cannot parse EH (exception-handling)
  artifacts. This is vendored-receipt-backed (real probe output recorded in
  this milestone's receipts) and tracked in an open upstream wasmi issue.
  No local patch can close it; the artifact class wasmi would need to accept
  is produced outside this repo.
- **Residual surface**: the only remaining code reachable through the wasmi
  path is `graphlaw_run.rs` plus unused dev-dependencies and 5
  `[profile.*]` blocks in the workspace manifest. Nothing in `src/` or the
  court suite depends on wasmi for admission, evaluation, or receipts.
- **Superset already exists**: `graphlaw-run-eh` is a protocol superset of
  the wasmi runner with stub-compatible exit codes. Every wire behavior the
  wasmi runner provides is available through the EH runner; consumers see
  the same exit-code contract.

Per the ERRC asymmetry rule, deletion outranks maintenance: a retired dead
path is a permanent entropy sink, while keeping it costs compile time,
profile sprawl, and a standing implication that wasmi is a supported
runtime.

## Decision

Retire the wasmi execution path in **one PR**, ordered so the deletion is
witnessed before bytes are removed:

1. **Flip consumers first.** The `tests_py` blackbox suite has 3 call sites
   that exercise the wasmi runner; flip them to `graphlaw-run-eh`. Update
   the conftest runner lookup (the single resolution point for the runner
   binary) to resolve the EH runner.
2. **Witness green.** Run the flipped suite and record the real output
   (exit code, pass counts) in the milestone receipts before any deletion.
   A green witness on the flipped surface is the precondition for step 3;
   no deletion lands on an unrun flip.
3. **Delete.** Remove `graphlaw_run.rs`, the unused dev-dependencies, and
   the 5 profile blocks in the same PR.

No feature flag, no deprecation window: the path is unrunnable against
current artifacts and fully superseded, so a flag would preserve cost
without preserving capability.

## Deletion receipt shape

The deletion receipt is the proof the retirement happened and is
recoverable. It must contain:

- **Absent-from-graph witness**: real output of
  `cargo tree -i wasmi` showing wasmi no longer resolves in the dependency
  graph (post-delete run, embedded verbatim — a receipt with no embedded
  output is not a receipt).
- **Per-file blake3 of pre-delete bytes**: each deleted file's blake3
  digest computed from the pre-delete contents, so the exact removed bytes
  are identifiable and replayable from git history.
- The green witness output from the flipped blackbox suite (step 2).

## Consequences

- The workspace loses the wasmi dev-dependencies and profile blocks;
  compile surface and manifest shrink accordingly.
- `graphlaw-run-eh` becomes the sole runner behind the blackbox contract.
- **Preserved as note**: wasmi is a pure interpreter. If a future host
  needs a no-JIT execution engine (e.g. environments without EH support),
  wasmi remains the candidate; this ADR records that intent so the
  retirement is not misread as a verdict against pure interpretation.
- Reintroduction, if ever needed, is a new ADR — not a revert.

## PURE-artifact pin caveat

The D1a/D1b PURE-artifact pin work stays **open and untouched** by this
retirement. The pin concerns artifact identity and admission guarantees,
not the wasmi runner; no D1a/D1b acceptance depends on `graphlaw_run.rs`.
This ADR neither advances nor closes that ticket.

## See Also

- `docs/jira/v26.10.23/ROADMAP.md`
- `docs/refusals.md` (typed refusal register)
