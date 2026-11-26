# Index

The Diataxis map of `docs/`. Every doc file under `docs/` (outside `docs/jira/` and
`docs/archive/`, which have their own sections) is listed in exactly one quadrant.
Placement was decided by the doc's purpose, not its directory; where a file's path disagrees
with its quadrant, the Placement notes section says so. Counts and statuses live in the linked
registers, not here.

| Quadrant | You want to | Section |
|---|---|---|
| Tutorials | learn by doing, with one guaranteed path | [Tutorials](#tutorials) |
| How-to | accomplish a specific task | [How-to guides](#how-to-guides) |
| Reference | look up exact facts | [Reference](#reference) |
| Explanation | understand why | [Explanation](#explanation) |
| Archive | find superseded or historical records | [Archive and program records](#archive-and-program-records) |

## Tutorials

- [quickstart.md](quickstart.md) — build, admit a plan, read the receipt, see a refusal.
- [tutorials/first-law-admission.md](tutorials/first-law-admission.md) — the same path with
  the receipt fields explained.
- [python/diataxis/tutorials/01-first-session.md](python/diataxis/tutorials/01-first-session.md)
  — first Python session against the runner.
- [python/diataxis/tutorials/02-admit-a-plan-with-receipts.md](python/diataxis/tutorials/02-admit-a-plan-with-receipts.md)
  — admit a plan and verify its receipts from Python.

## How-to guides

Core (Rust, wasm, SQL):

- [how-to/run-the-wasm-module.md](how-to/run-the-wasm-module.md)
- [how-to/first-in-wasm-sql-lifecycle.md](how-to/first-in-wasm-sql-lifecycle.md)
- [how-to/persistent-duckdb-sessions.md](how-to/persistent-duckdb-sessions.md)
- [how-to/document-sequences.md](how-to/document-sequences.md)
- [how-to/egress-and-attach-policy.md](how-to/egress-and-attach-policy.md)
- [how-to/how-to-edge-ingestion-and-compaction.md](how-to/how-to-edge-ingestion-and-compaction.md)
- [how-to/authorize-with-signed-leases.md](how-to/authorize-with-signed-leases.md)
- [how-to/verify-without-trusting-the-operator.md](how-to/verify-without-trusting-the-operator.md)
- [how-to/use-knowledge-hooks.md](how-to/use-knowledge-hooks.md)
- [how-to/use-from-javascript.md](how-to/use-from-javascript.md)
- [how-to/release-publish.md](how-to/release-publish.md)
- [WASM.md](WASM.md) — wasm runbook: build, verify, embed, upgrade.
- [refusal-playbook.md](refusal-playbook.md) — symptom, refusal, cause, fix.

Python (`docs/python/diataxis/`):

- [how-to/query-and-validate](python/diataxis/how-to/query-and-validate.md)
- [how-to/reason-and-derive](python/diataxis/how-to/reason-and-derive.md)
- [how-to/govern-with-law-steps](python/diataxis/how-to/govern-with-law-steps.md)
- [how-to/receipts-and-attestation](python/diataxis/how-to/receipts-and-attestation.md)
- [how-to/duckdb-sessions](python/diataxis/how-to/duckdb-sessions.md)
- [how-to/handle-refusals](python/diataxis/how-to/handle-refusals.md)
- [how-to/stay-inside-the-limits](python/diataxis/how-to/stay-inside-the-limits.md)
- [how-to/give-an-agent-graphlaw](python/diataxis/how-to/give-an-agent-graphlaw.md)
- [python/diataxis/index.md](python/diataxis/index.md) — the Python sub-map;
  [python/diataxis/coverage.json](python/diataxis/coverage.json) is its coverage data.

## Reference

Wire and API:

- [abi-reference.md](abi-reference.md) — JSON envelope, ops, error kinds.
- [capability-registry.md](capability-registry.md) — op surface and digest rule.
- [api-stability.md](api-stability.md) — semver policy, `#[non_exhaustive]` list, MSRV.
- [DIALECTS.md](DIALECTS.md) — dialect routing table.
- [reference/typescript-api.md](reference/typescript-api.md)
- [python/diataxis/reference/api.md](python/diataxis/reference/api.md)
- [python/diataxis/reference/capability-matrix.md](python/diataxis/reference/capability-matrix.md)
- [wasm-integration.md](wasm-integration.md) — WASI shell vs safe core.
- [reference/runner-exit-codes.md](reference/runner-exit-codes.md)
- [reference/knowledge-hook-projection.md](reference/knowledge-hook-projection.md)
- [qme-1-reference.md](qme-1-reference.md)

Refusals:

- [refusals.md](refusals.md) — normative typed refusal register, append-only.
- [reference/refusal-catalog.md](reference/refusal-catalog.md) — user-facing view.

Conformance and courts:

- [reference/sql-rdf-rec.md](reference/sql-rdf-rec.md) — sql-rdf Recommendation text.
- [reference-generated/sql-rdf-conformance.md](reference-generated/sql-rdf-conformance.md) —
  generated; regenerate, never patch.
- [reference/conformance_claim.md](reference/conformance_claim.md)
- [CHICAGO_N3_DATALOG.md](CHICAGO_N3_DATALOG.md)
- [CHICAGO_FIBO.md](CHICAGO_FIBO.md)
- [CHICAGO_CROSS_AUTHORITY.md](CHICAGO_CROSS_AUTHORITY.md)
- [reference/n3-audit-100-report.md](reference/n3-audit-100-report.md)
- [reference/wasm-gap-register.md](reference/wasm-gap-register.md) — typed open gaps.

## Explanation

- [explanation/receipt-fields.md](explanation/receipt-fields.md)
- [explanation/eh-closure-story.md](explanation/eh-closure-story.md)
- [explanation/edge-sovereignty-and-zero-trust-compaction.md](explanation/edge-sovereignty-and-zero-trust-compaction.md)
- [QUALIFICATION_ALGEBRA.md](QUALIFICATION_ALGEBRA.md) — standing as executable falsification.
- [python/diataxis/explanation/combinatorial-maximalism.md](python/diataxis/explanation/combinatorial-maximalism.md)
- [python/diataxis/explanation/state-and-persistence.md](python/diataxis/explanation/state-and-persistence.md)
- [adr/ADR-wasmi-retirement.md](adr/ADR-wasmi-retirement.md) — decision record.
- [reference/design-read-frame-bound.md](reference/design-read-frame-bound.md) — design
  rationale for the read_frame bound.
- [reference/design-sniff-fixes.md](reference/design-sniff-fixes.md) — design rationale for
  the dialect sniff fixes.

## Archive and program records

- [archive/README.md](archive/README.md) — what has been archived and the pointer-stub
  convention. Superseded docs live under `docs/archive/`; a stub is left at the old path.
- `docs/jira/` — milestone windows. The newest directory is the active window; older windows
  carry an `ARCHIVED.md` pointer to `docs/archive/jira/`. Read `ls docs/jira` at use time.

## Placement notes

- `how-to/first-in-wasm-sql-lifecycle.md` is a walkthrough but stays How-to: it assumes a
  built wasm module and covers a task, not first contact.
- `WASM.md` and `refusal-playbook.md` are runbooks (task-oriented), so How-to, despite sitting
  in `docs/` root.
- `QUALIFICATION_ALGEBRA.md` is Explanation: it argues the standing model; its executable
  binding section is a pointer, not a lookup table.
- `reference/design-*.md` live under `reference/` but read as design rationale; placed under
  Explanation. Paths are unchanged.
- `reference/n3-audit-100-report.md` is a dated report of findings, so Reference (records).
- `CHICAGO_*.md` describe court contracts to look up, so Reference.
- The Python quadrant docs are listed by role; the Python package surface is described in
  `docs/python/diataxis/` and was not re-read by this lane against code.

## See Also

- [Quickstart](quickstart.md)
- [Refusals](refusals.md)
- [Repository README](../README.md)
- [Archive](archive/README.md)
