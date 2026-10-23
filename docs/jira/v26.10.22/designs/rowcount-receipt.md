# Design: Rowcount Receipt Strengthening (K1 H2)

Lane: M07 (GUARD BUILDER) · 2026-10-22 · Lease: `tests/chicago_validation.rs` (extend only),
this doc.

## Defect class

`rows_emitted >= 1` floors in the sql-rdf execution courts cannot detect a dropped
filter (WHERE removed -> all rows) or an inverted predicate (`> 400` -> `< 400`).
A floor passes for any non-empty result, so a compiler defect that weakens the
compiled WHERE/HAVING is invisible.

## Fix shape

Courts assert the **exact** rowcount and the **exact values** of the compiled
query's own output rows, read back from `ExecutionReceipt.result_triples` — the
projection `eval_ast_query` materializes from the real frozen-duckdb rows
(`src/duckdb_eval.rs` step 3, "Vectorized execution and columnar projection").
No twin SQL, no hand-written second statement, no mocks (Chicago: real
connection, real table readback).

Mutations each court must flip on (falsifiers):

| Mutation | Old floor | New court |
|---|---|---|
| WHERE dropped | pass | `rows_emitted == 2` fails (3) + `beta` re-admitted fails value assert |
| WHERE inverted (`< 400`) | pass | only `beta` -> value assert fails |
| HAVING dropped | pass | 3 rows + `beta` (total 10) fails |
| total inverted | pass | totals `["480","900"]` fails |

## Courts

New module `rowcount_receipt_courts` in `tests/chicago_validation.rs`:

1. `admitted_select_exact_rowcount_and_exact_values_readback` — admitted tier
   compile (`admit_bytes_tiered`), exact `rows_emitted == 2`, exact batch_id
   values `["alpha", "gamma"]`, `beta` absent. Fixture: telemetry_events
   (alpha 480, beta 10, gamma 900), predicate `latency_ms > 400`.
2. `compiled_group_having_order_by_reads_back_its_own_rows` — compiled
   GROUP BY + HAVING SUM >= $1 + ORDER BY batch_id DESC executed via
   `eval_ast_query`; asserts exact rows (2), exact batch_ids
   `["alpha", "gamma"]`, exact totals `["480", "900"]`, `beta` absent.
   Replaces the twin-SQL readback pattern (hand-written copy of the query
   executed via `execute_batch` + separate verification SQL), which could
   pass while the compiled statement diverged from the twin.

## Note on file location

The dispatch named the weak tests at `tests/chicago_validation.rs:155/:162`;
they actually live at `tests/sql_rdf_dialect.rs:137/:162` (outside this lane's
lease). Per lithography, the strengthened equivalents were added in the leased
file; the sql_rdf_dialect.rs edits are proposed, not applied.

## Receipt

See the lane report for the real `CARGO_TARGET_DIR=target-m7 cargo test
--test chicago_validation` output.
