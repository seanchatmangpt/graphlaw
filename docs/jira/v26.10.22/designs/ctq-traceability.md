# CTQ → Court Traceability Matrix (DFLSS Define-lane A record)

Condensed from the CTQ lane. Sources: PRD-P28-SIL §1/§2/§5, designs/
P28-SIL-specs, hot-path-baseline, TC-HOT-benchmarks, TC-MUT-01-mapping.

| # | CTQ (budget) | Surface | Court | Verdict |
|---|---|---|---|---|
| 1 | Typed refusal ≤0.10µs | dialect.rs:152-200 Refusal | TC-HOT-01 refusal leg (timed, printed) | **PARTIAL** — timed but UNGATED (no 100ns assert); RefusalMessage::Static zero-alloc fast path NOT landed (heap String per refusal); 0.10µs figure appears nowhere in code |
| 2 | H_query fold ≤0.15µs | receipt_state fold (sql_ast.rs:400, duckdb_eval.rs:293-311) | tc_hot_01 clone-path leg + bit-identity courts | **PARTIAL — measured MISS**: 184-216ns vs 150ns (1.2-1.4x over; satisfies the 250ns TC-HOT-01 gate which binds only 250ns — the 150ns row has NO gate) |
| 3 | Evidentiary overhead ≤0.25µs | fold + SPSC ring emit | tc_hot_01 gate (p99 ≤250ns) + evidentiary_ring_court | **MET (smoke)**: 0.19-0.22µs fold-path; caveats: 1M not 10M iters, contention ceiling, eval wrapper UNKNOWN (prepare 302-541µs dominates; re-prepares per call) |
| 4 | >12M rows/sec | parquet mount + eval | tc_hot_02 | **PARTIAL — UNCOURTED at scale**: 500k smoke; throughput printed never gated vs 12M; 25M full behind TC_HOT_FULL=1 unrun |
| 5 | p95 <12ms | eval_ast_query | **NONE** | **UNCOURTED** — no test/bench/gate asserts 12ms anywhere; only meaningful at 25M scale (unrun) |
| 6 | 100% receipt coverage / anti-vacuity | receipts + chains + refusal classes | mutation_court + shape_mutation_court + no_mock/no_mutant markers + corpus flips | **PARTIAL** — shape-side operator mutation is E2-coverage not a flip court; aggregate+ProcessScope conjunction row landed post-audit (verify); query-receipt ledger designed-not-built |

## Flags for the SPC court + OCAP court lanes
- CTQ 1: add a 100ns budget assert to the refusal leg OR land
  RefusalMessage::Static first — currently the 0.10µs CTQ is printed-only.
- CTQ 2: the SPC court binds 250ns; file the 150ns §2 row as a tighter
  Phase-2 target with its own gate.
- CTQ 4/5: the 25M-row TC_HOT_FULL run is the acceptance event for both
  budgets; nothing else can close them.
- CTQ 6: conjunction corpus row landed late — confirm it's in the 14/0 count.
