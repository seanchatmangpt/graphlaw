# M-wave verdict receipts (coordinator-filed from lane reports, 2026-10-22)

The M-lane court/verify lanes report counts rather than receipt files;
N16's GO/NO-GO board marked their rows UNFILED. This record carries the
verdicts. Numbers are transcribed from the lanes' own reported command
output.

## M2 — statement courts (zero edits)

- `cargo test --test sql_rdf_dialect` → 25 passed / 0 failed
- `cargo test --test duckdb_statement_kinds` → 6 passed / 0 failed
- `cargo test --test sql_rdf_conformance` → 7 passed / 0 failed
No src edits; no skips.

## M6 — refusal register audit (2 corrections)

- Codes table verified against carriers (LawError, RefusalKind::DuckDb,
  AstError variants 16–21, regex at src/sql_ast.rs:495).
- CORRECTED: NO-CANONICAL-GRAPH Display quote now includes the
  `(query <iri>)` suffix (src/duckdb_eval.rs:102-105).
- CORRECTED with dated note: L85 ergonomics probe pointer (renamed to
  examples/refusal_probe.rs by L96; N11 later layered the supersession).
- `cargo test --test trust_attest` → 10 passed / 0 failed.

## M9 — authority manifest (PASS, no change)

- `cargo test --test authority_manifest` → 6 passed / 0 failed:
  frozen-duckdb 2.0.2 pin, no stray [patch], Eyeron 0.7.7 vendored
  match, lockfile agreement, no unpublishable path deps.

## M10 — ledger/cache/ring courts (zero edits)

- query_ledger_court → 9/0; query_cache_court → 8/0.
- evidentiary_ring_court bare → typed loud SKIP (exit 101, expected);
  featured run → 10 passed / 0 failed (15.40s), all nine ring courts.

## M11 — old-link docs (positive no-change)

- Greps: no live wasm-link-pending / duckdb-feature / cache-as-SDK
  instructions in README/CONTRIBUTING/docs/how-to/run-the-wasm-module.md.
- readme_first_ten_minutes → 3/0; docs_release_claims → 2/0.

## M4 — wasm boundary (pending at file time)

M4 was still running when this record was filed; its receipt lands in
its own file. Not covered here.
