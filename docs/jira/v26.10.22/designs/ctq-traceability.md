# CTQ → Court Traceability Matrix (DFLSS Define-lane A record)

Condensed from the CTQ lane. Sources: PRD-P28-SIL §1/§2/§5, designs/
P28-SIL-specs, hot-path-baseline, TC-HOT-benchmarks, TC-MUT-01-mapping.

| # | CTQ (budget) | Surface | Court | Verdict |
|---|---|---|---|---|
| 1 | Typed refusal ≤0.10µs | dialect.rs RefusalMessage::Static | TC-HOT-01 refusal leg | **PASS** — 10M 2026-10-22 p50 41 ns / p99 42 ns ≤ 100 ns. Timed path is two `write_str` copies (static class label + `RefusalMessage::Static`); `Display` is outside the leg (`designs/tc-hot-01-errc-receipt.md`) |
| 2 | H_query fold ≤0.15µs | receipt_state fold (sql_ast.rs:400, duckdb_eval.rs:293-311) | tc_hot_01 clone-path leg + bit-identity courts | **PARTIAL — measured MISS**: 184-216ns vs 150ns (1.2-1.4x over; satisfies the 250ns TC-HOT-01 gate which binds only 250ns — the 150ns row has NO gate) |
| 3 | Evidentiary overhead ≤0.25µs | fold_params_anchored one update | tc_hot_01 gate (p99 ≤250ns) + spc_court.sh | **FAIL** — 10M clone-path p99 209 ns ≤ 250 ns, variance 26.91% > 5% (batch p50s 125/166/166), combined gate exit 1. SPC smoke p99 209 ns ≤ 250 ns and Cpk 0.001 < 1.33 (max 21,413,887 ns), exit 1 (`receipts/spc-2026-10-22T191548.json`). Prior FAIL receipts left in place |
| 4 | >12M rows/sec | parquet scan | removed | **REMOVED** — `benches/tc_hot_02.rs` timed a DuckDB parquet scan. That is the engine, not a graphlaw measurement. The bench is deleted |
| 5 | p95 <12ms | eval_ast_query | removed | **REMOVED** — the same bench's cold `eval_ast_query` p95 is DuckDB prepare plus the scan. Not a graphlaw budget |
| 6 | 100% receipt coverage / anti-vacuity | receipts + chains + refusal classes | mutation_court + shape_mutation_court + no_mock/no_mutant markers + corpus flips | **PARTIAL** — shape-side operator mutation is E2-coverage not a flip court; aggregate+ProcessScope conjunction row landed post-audit (verify); query-receipt ledger designed-not-built |

## Flags for the SPC court + OCAP court lanes
- CTQ 1: closed on the 10M run (p99 42 ns). The timed leg copies two
  static strings; `Display` remains outside that leg.
- CTQ 2: the SPC court binds 250ns; file the 150ns §2 row as a tighter
  Phase-2 target with its own gate.
- CTQ 4/5: removed with `benches/tc_hot_02.rs`. Those rows timed DuckDB.
- CTQ 6: conjunction corpus row landed late — confirm it's in the 14/0 count.
