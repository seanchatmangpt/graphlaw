# Hot-path measured baseline (design record — DMEDI Measure)

Release-mode timing test `tests/p28_hot_path_bench.rs` (min-of-batches; real
runs under concurrent fan-out — absolute numbers are ceilings, ratios are the
signal). 1102-byte canon, 4 typed params.

| Measurement | Result | Budget | Verdict |
|---|---|---|---|
| H_query full rehash (`admitted_statement_hash`, no fold state) | 1673–1748 ns/op | ≤150 ns | **MISS ~11.6x** — cost is the canon-byte absorb (0-param variant ~1630ns proves it) |
| H_query incremental fold (`fold_params_anchored`, one `[u8; 96]` update) | **10M p99 209 ns** (p50 batches 125/166/166) | ≤150 ns (§2, no gate) / ≤250 ns p99 | **p99 inside 250 ns; variance 26.91% FAIL.** The 2KB hasher-clone is not this path: `receipt_state` is 32 bytes, folded in one update |
| Admission (compile_turtle: SHACL + RDFC-1.0) | 1.56–4.97 ms/compile | n/a — once per compile, excluded from hot path per PRD §2 | correctly out of hot-path claims |

## Conclusions
- The ≤0.25 µs TC-HOT-01 budget read as MET on the fold path (~0.19–0.22 µs
  measured) at bench time — the M3 receipt_state landing is what buys it; the full-hash
  path (hand-built CompiledQuery without fold state) remains 11.6x over and
  is the documented §3.1.1 fallback to close. **The 2026-10-22 ERRC run
  replaces the p99 reading:** 10M clone-path p99 209 ns (inside 250) and
  the combined gate still FAILs on variance 26.91%. SPC smoke p99 209 ns
  meets the USL and Cpk 0.001 does not
  (`designs/tc-hot-01-errc-receipt.md`). The CORRECTION below is the earlier
  measurement, kept as history.
- The 2KB hasher-state memcpy is not the live fold. `receipt_state` is a
  32-byte `h_ast`. `fold_params_anchored` absorbs `h_ast || P_params` with
  one `Hasher::update` of a 96-byte stack buffer (2026-10-22 ERRC). Hasher
  reuse and a second `update` were the shapes bcinr measured and reverted;
  they are not reopen items. The 150 ns row stays a phase-2 target with no
  gate.
- Honest caveats: fan-out contention inflates absolutes; benches under
  contention are ceilings not guarantees. The eval-versus-raw DuckDB
  comparison was removed from `tests/p28_hot_path_bench.rs`.

## CORRECTION (SPC court, post-baseline): the 250ns P99 is NOT met
The SPC gate (scripts/spc_court.sh, n=1,000,000, release, bit-identity
verified) measured clone-path **p50=291ns, p99=375ns** — above the 250ns USL
before any injection. Cpk −0.002 (raw σ dominated by OS-scheduler outliers,
max samples 5.5-27ms). Verdict: honest FAIL, correctly exited 1; the
anti-vacuity injection flip was degenerate at the SPC run (2026-10-22; both arms FAIL) until
the budget closes. The earlier "184-216ns met" reading was MIN-of-batches —
min-of-rounds and p99-over-1M answer different questions; the p99 is the
PRD's own criterion and it is not met. Closure belongs to the perf owners
(receipt-surface + clone-memcpy residual), not the gate.
