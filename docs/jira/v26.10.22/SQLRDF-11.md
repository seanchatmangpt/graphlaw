# SQLRDF-11 — Chicago extension court through compiled AST graphs

- **Id:** SQLRDF-11
- **Title:** Chicago extension court through compiled AST graphs
- **Status:** Open

## Conformance target

Section 6.2 of the pasted sql-rdf Recommendation, the conformance text for v26.10.22, under the section 6.1 Chicago rules: real in-memory frozen-duckdb, no test doubles, and an anti-vacuity check. Inverting an operator or mutating the table must change the court result.

The five certified checks:

| Extension | Verification | Certified result |
| --- | --- | --- |
| `core_functions` | `SELECT list_sum([1, 2, 3])` | integer `6` |
| `icu` | `SELECT CASE WHEN count(*) > 0 THEN 1 ELSE 0 END FROM icu_calendar_names()` | integer `1` |
| `json` | `SELECT json_extract('{"a": 1}', '$.a')::INTEGER` | integer `1` |
| `parquet` | `COPY (...) TO 'surface.parquet'; SELECT count(*) FROM read_parquet(...)` | integer `1` |
| `tpch` | `CALL dbgen(sf = 0); SELECT count(*) FROM nation` | integer `25` |

## Current behavior

Those five checks are raw SQL strings in [`ggen/sql-dialect/sql_extensions.rs`](../../../ggen/sql-dialect/sql_extensions.rs) (lines 10–22):

- `core_functions`: `SELECT list_sum([1, 2, 3])` expects `6`
- `icu`: `SELECT CASE WHEN count(*) > 0 THEN 1 ELSE 0 END FROM icu_calendar_names()` expects `1`
- `json`: `SELECT json_extract('{"a": 1}', '$.a')::INTEGER` expects `1`
- `parquet`: copy to `/tmp/graphlaw-sql-dialect.parquet`, then `SELECT count(*) FROM read_parquet(...)` expects `1`
- `tpch`: `CALL dbgen(sf = 0)` then `SELECT count(*) FROM nation` expects `25`

`every_source_extension_answers_or_is_absent` in [`tests/duckdb_statement_kinds.rs`](../../../tests/duckdb_statement_kinds.rs) (lines 45–93) runs that table on `GraphLawDuckDB::open_in_memory`. When setup fails with `allow_unsigned_extensions`, the test accepts the error and does not run the check. The unsigned-extension connection already exists: `GraphLawDuckDB::open_in_memory_allowing_unsigned_extensions` in [`src/duckdb_eval.rs`](../../../src/duckdb_eval.rs) (lines 83–87), used by [`tests/chicago_validation.rs`](../../../tests/chicago_validation.rs).

The court feeds SQL text to `execute_batch` / `query_row`. It does not compile an AST graph.

## Required behavior

The five section 6.2 checks run through compiled AST graphs on a real `GraphLawDuckDB::open_in_memory_allowing_unsigned_extensions` connection. Certified results stay `6`, `1`, `1`, `1`, and `25`.

Anti-vacuity: inverting the section 7 comparison operator changes the readback. A court that still passes is a failure.

## Acceptance

`cargo test --test duckdb_statement_kinds`.
