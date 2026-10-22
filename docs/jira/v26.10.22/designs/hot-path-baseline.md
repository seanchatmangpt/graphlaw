# Hot-path measured baseline (design record — DMEDI Measure)

Release-mode timing test `tests/p28_hot_path_bench.rs` (min-of-batches; real
runs under concurrent fan-out — absolute numbers are ceilings, ratios are the
signal). 1102-byte canon, 4 typed params.

| Measurement | Result | Budget | Verdict |
|---|---|---|---|
| H_query full rehash (`admitted_statement_hash`, no fold state) | 1673–1748 ns/op | ≤150 ns | **MISS ~11.6x** — cost is the canon-byte absorb (0-param variant ~1630ns proves it) |
| H_query incremental fold (`receipt_state` clone + param fold + finalize) | **184–216 ns/op** | ≤150 ns (§2) / ≤250 ns (TC-HOT-01) | **~8x improvement; INSIDE the 0.25µs TC-HOT-01 budget, marginally over the 0.15µs §2 row** — residual gap is the ~2KB hasher-state memcpy in clone() |
| Admission (compile_turtle: SHACL + RDFC-1.0) | 1.56–4.97 ms/compile | n/a — once per compile, excluded from hot path per PRD §2 | correctly out of hot-path claims |
| eval_ast_query wrapper overhead | UNKNOWN at sub-µs | — | duckdb prepare dominates (302–541 µs/call, swings exceed wrapper cost; negative-overhead artifacts witnessed); needs pinned-core criterion harness or a prepared-statement reuse seam (eval re-prepares per call, duckdb_eval.rs:204) |

## Conclusions
- The ≤0.25 µs TC-HOT-01 budget is MET on the fold path (~0.19–0.22 µs
  measured) — the M3 receipt_state landing is what buys it; the full-hash
  path (hand-built CompiledQuery without fold state) remains 11.6x over and
  is the documented §3.1.1 fallback to close.
- Remaining optimization lever: avoid the 2KB state memcpy (e.g. pre-finalize
  the canon-only prefix into a mid-state, or hash-table the per-graph state).
- Honest caveats: fan-out contention inflates absolutes; sub-µs wrapper
  overhead is unmeasurable with the current harness; benches under
  contention are ceilings not guarantees.

## CORRECTION (SPC court, post-baseline): the 250ns P99 is NOT met
The SPC gate (scripts/spc_court.sh, n=1,000,000, release, bit-identity
verified) measured clone-path **p50=291ns, p99=375ns** — above the 250ns USL
before any injection. Cpk −0.002 (raw σ dominated by OS-scheduler outliers,
max samples 5.5-27ms). Verdict: honest FAIL, correctly exited 1; the
anti-vacuity injection flip is currently degenerate (both arms FAIL) until
the budget closes. The earlier "184-216ns met" reading was MIN-of-batches —
min-of-rounds and p99-over-1M answer different questions; the p99 is the
PRD's own criterion and it is not met. Closure belongs to the perf owners
(receipt-surface + clone-memcpy residual), not the gate.
