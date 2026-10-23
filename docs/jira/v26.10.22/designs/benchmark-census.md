# Benchmark census (2026-10-22)

This file gathers every performance harness and every numeric receipt that
was on disk when it was written. Nothing in this census was re-run. The
source-of-truth checker (`pending-decisions.md` item count = 6) exited 0
and does not cover these numbers. The numbers below were re-read from the
named files in this session.

A later sentence that copies one of these figures into a second file will
go stale the moment a receipt is overwritten. `receipts/spc-2026-10-22.json`
already has been. Several prose files still quote the bytes it used to hold.

## How to read a number

Five instruments time related work and answer different questions. Mixing
their outputs is how the window ended up with Cpk 1.44, Cpk −0.05, Cpk
0.2948, and Cpk 0.001 presented as if they were one measurement.

| Instrument | What one sample is | What the summary is | Where it lives |
|---|---|---|---|
| `tests/p28_hot_path_bench.rs::p28_hot_path_baseline` | A batch of folds, then nanoseconds per op | Minimum batch. A ceiling under fan-out, not a percentile | Printed by the test. The 184–216 ns band is this instrument |
| `p28_hot_path_capability` | One `Instant` pair around one `fold_params_anchored` | Mean, sigma, Cpk, empirical percentiles over 1,000,000 samples | `designs/cpk-correction.md`. Includes the clock pair |
| `benches/tc_hot_01.rs` | One iteration into an HDR histogram | p50 / p95 / p99 / p99.9, plus a 3-batch p50 spread | Smoke is 1,000,000 iterations. `TC_HOT_FULL=1` is 10,000,000 |
| `scripts/spc_court.sh` | The same `tc_hot_01` clone-path histogram, then μ, σ, Cpk | Exits 1 when Cpk < 1.33 or p99 > 250 ns | `receipts/spc-*.json` |
| `benches/tc_hot_02.rs` | A DuckDB `eval_ast_query` over a parquet corpus | p50/p95/p99 in microseconds, PCE, scanned rows/sec | Source file is on disk. `Cargo.toml` does not register it |

Cpk in the SPC receipts is `(USL − mean) / (3 · stddev)` with USL = 250 ns
and no lower spec. One far sample inflates stddev and collapses Cpk while
p50 stays near 167 ns. That is why a receipt can show p99 = 209 ns and
Cpk = 0.001 in the same object.

## Harness inventory

Registered bench target, `Cargo.toml` lines 112–117:

- `[[bench]] name = "tc_hot_01"`, `harness = false`. Custom `main`. Times
  the receipt fold and the refusal render. The comment above the target
  says DuckDB scan and prepare timing is not a graphlaw bench.

On disk and not registered:

- `benches/tc_hot_02.rs`, 311 lines. Header says SMOKE is 500,000 rows and
  the 25,000,000-row corpus is behind `TC_HOT_FULL=1`. `rg tc_hot_02 Cargo.toml`
  finds no match. `designs/TC-HOT-benchmarks.md` and
  `designs/ctq-traceability.md` say the file is deleted. Both sentences
  are false against the working tree. The file is not a `cargo bench` target.

Timing test, not a bench target:

- `tests/p28_hot_path_bench.rs`, 182 lines. Two tests:
  `p28_hot_path_baseline` (min-of-batches, verdict printed, not a hard
  assert on the budget) and `p28_hot_path_capability` (1,000,000
  per-sample timings, Cpk printed).
- `tests/evidentiary_ring_court.rs`. Dev leg is a ratio fence. Release
  leg asserts stalled max push < 10 µs.
- `tests/wasm_size_court.rs`. Ceiling `MAX_BYTES = 67,108,864` (64 MiB).
  This is a size court, not a latency bench. The pinned module cited
  throughout the window is 6,793,617 bytes.

Court script:

- `scripts/spc_court.sh`. Default SMOKE, 1,000,000 iterations.
  `SPC_MODE=FULL` sets `TC_HOT_FULL=1` (10,000,000).
  `SPC_INJECT_OUTLIER=1` busy-spins a 2 µs outlier into every recorded
  clone-path sample. Default `CARGO_TARGET_DIR` is `target-dflss12`.
  USL default 250 ns. Cpk minimum default 1.33. The script writes
  `receipts/spc-<date>.json`. Timestamped copies on disk are lanes
  preserving an earlier file before the undated name was overwritten.

No Criterion harness. `hdrhistogram` is a dev-dependency.

## Budgets, as coded

From `benches/tc_hot_01.rs`:

| Constant | Value | Gate |
|---|---|---|
| `BUDGET_NS` | 250 | clone-path p99, and the SPC USL |
| `REFUSAL_BUDGET_NS` | 100 | refusal p50 and p99. The timed body is two `write_str` calls into a 192-byte stack buffer. `Display` is outside the leg |
| `PARAM_BUDGET_NS` | 40 | `ParamBytes::encode` plus one `Hasher::update`, small (inline < 64 B) and large amortized (spill > 64 B) |
| variance | 5% | spread of three batch p50s. `VARIANCE_SELF_CHECK` |
| timestamp | 10 ns | `CoarseClock` coarse-now, median, and monotonic |
| SPC `cpk_min` | 1.33 | `scripts/spc_court.sh`, not a constant in the bench |

From `tests/p28_hot_path_bench.rs`:

| Row | Value | Enforced? |
|---|---|---|
| Full rehash `admitted_statement_hash` | 150 ns, printed | No. A miss prints `BUDGET-MISS` and the test still passes |
| `fold_params_anchored` vs 150 ns | same print | No gate |
| Capability USL | 250 ns | Printed Cpk. The test does not `assert` Cpk ≥ 1.33 |

From the ring court:

| Leg | Fence |
|---|---|
| Dev, `no_blocking_property_1m_pushes_vs_stalled_drainer` | Stalled max push ≤ 10× the unstalled baseline in the same binary. Baseline floored at 100 ns |
| Release, `no_blocking_release_absolute_bound` | Stalled max push < 10 µs. `debug_assertions` off, feature `evidentiary-ring` |

Charter figures that these gates do not implement:

- Cpk 1.44 at μ = 0.198 µs, σ = 0.012 µs. `designs/cpk-correction.md`
  records this as not witnessed.
- Substrate overhead < 2.8%. `designs/pce-baseline.md` records 42.12%
  at its operating point.
- H_query ≤ 150 ns as a passing gate. The 150 ns row is a phase-2
  target. The executable gate is 250 ns.
- 10,000,000-iteration SPC. `SPC_MODE=FULL` exists in the script.
  Every JSON receipt on disk has `"mode": "SMOKE"` and
  `"iterations": 1000000`.

## TC-HOT-01, run by run

Machine for the L20 smoke, copied from
`designs/tc-hot-baseline.md`: Apple M3 Max, `aarch64`, 16 cores, no
thermal warning recorded. Canon 649 bytes, 5 params, bit-identity
`VERIFIED`. Warmup 50,000 untimed iterations. Thread affinity is not
applied (`core_affinity` is not a dependency). Priority elevation is
not applied.

### L20 smoke, 1,000,000 iterations, exit 1

Command: `CARGO_TARGET_DIR=target-lane20 cargo bench --bench tc_hot_01`.
Compile of that attempt was 5m44s after a 30-minute background kill.
Source: `designs/tc-hot-baseline.md`.

| Leg | p50 | p95 | p99 | p99.9 | Gate |
|---|---|---|---|---|---|
| `re_absorb_baseline` | 1209 ns | 1583 ns | 1625 ns | 1917 ns | reported, not the 250 ns gate |
| `clone_path` | 208 ns | 291 ns | 292 ns | 375 ns | FAIL (292 > 250) |
| `refusal_invariant` | 250 ns | 334 ns | 416 ns | 541 ns | FAIL (250/416 > 100) |

Variance: batch p50s `[208, 208, 208]`, spread 0.00%, PASS.
Param small net 23.00 ns (gross 158.39, scaffold 135.38), PASS against 40 ns.
Param large recompute net 1362.54 ns. Param large amortized net −28.10 ns
(gross 107.28 against scaffold 135.38), PASS against 40 ns. A negative
net means the scaffold batch was slower than the measured batch. It is
not a negative fold.
Timestamp: coarse-now 0.7 ns, `Instant` baseline 25.6 ns, monotonic, PASS.
Throughput line: 518,235.2 full-legs/sec, counting all three legs per
iteration.
The refusal note in that receipt says the measured cost was the
heap-`String` message path. The later ERRC run changed the timed body
to two `write_str` copies. The 416 ns refusal p99 and the 42 ns refusal
p99 are different functions.

### ERRC 10,000,000 iterations, exit 1

Command: `CARGO_TARGET_DIR=/tmp/gl-target TC_HOT_FULL=1 cargo bench --bench tc_hot_01 --offline`.
One run. Not retried. Source: `designs/tc-hot-01-errc-receipt.md`.
Bit-identity court `receipt_state_clone_path_bit_identical_to_reference`
exited 0 in the same note (1 passed).

| Leg | p50 | p95 | p99 | p99.9 | Gate |
|---|---|---|---|---|---|
| `re_absorb_baseline` | 1125 ns | 1625 ns | 1625 ns | 3667 ns | reported |
| `clone_path` | 125 ns | 208 ns | 209 ns | 250 ns | p99 inside 250 |
| `refusal_invariant` | 41 ns | 42 ns | 42 ns | 83 ns | PASS |

Variance: batches of 3,333,333, batch p50s `[125, 166, 166]`, spread
26.91% against 5%, `VARIANCE-EXCEEDED`. Combined clone-path gate FAIL
because variance is in the conjunction, even though p99 is 209 ns.
Param small net −52.05 ns, large amortized net −68.24 ns, printed PASS.
The note gives the batch times behind those nets: scaffold 168.87 ns,
small 116.82 ns, amortized 100.63 ns.
Timestamp 2.7 ns, monotonic, PASS.
`fold_params_anchored` at this run copies `h_ast` and the inline param
bytes into one `[u8; 96]` and calls `Hasher::update` once.

### ERRC SPC smoke that accompanied the 10M run, exit 1

Same note. Command: `CARGO_TARGET_DIR=/tmp/gl-target bash scripts/spc_court.sh`.
The JSON for this invocation is `receipts/spc-2026-10-22T191548.json`.

Bench lines embedded in the design:

- Variance spread 0.60%, batch p50s `[166, 166, 167]`, PASS.
- clone_path p50 166 / p95 208 / p99 209 / p99.9 250. Combined bench gate PASS.
- refusal p50 41 / p99 42, PASS.
- param small net 37.66 ns, large amortized 13.11 ns, PASS.
- timestamp 0.7 ns, PASS.

SPC object, re-parsed this session: mean 185.686 ns, stddev 21,610.119 ns,
Cpk 0.001, p99 209 ns, max 21,413,887 ns, samples 999,999, verdict FAIL,
reason `cpk 0.0010 < 1.33`. Shewhart rule 1 flagged 27 samples (LCL
−64,644.671 ns, UCL 65,016.043 ns). Rule 2 (8-long monotonic run) did
not flag; longest run 5, direction down. Assignable cause true.
p99 meets the USL. Cpk does not, because one sample at 21.4 ms sets σ.

## Every SPC receipt, re-parsed this session

All six files: bench `tc_hot_01`, leg `clone_path`, mode `SMOKE`,
USL 250.0, Cpk minimum 1.33, iterations 1,000,000, samples 999,999
(the histogram drops one sample; S4 confirmed the count). Verdict FAIL
on every file. Rule 2 never flagged. Assignable cause true on every file.
Cpk recomputed by S4 as `(250 − mean) / (3 · σ)` matched the stored
field on the five non-inject receipts.

| File | inject | mean ns | stddev ns | Cpk | p50 | p95 | p99 | p99.9 | max ns | rule1 count | FAIL reasons |
|---|---|---|---|---|---|---|---|---|---|---|---|
| `spc-2026-10-22.json` | no | 187.979 | 70.117 | 0.2948 | 167 | 209 | 250 | 292 | 29,759 | 70 | Cpk only. p99 equals the USL, so the p99 clause does not fire |
| `spc-2026-10-22-inject.json` | yes | 304.262 | 15,942.525 | −0.0011 | 250 | 292 | 333 | 1,167 | 12,435,455 | 120 | Cpk and p99 333 > 250 |
| `spc-2026-10-22T173758.json` | no | 180.476 | 5,152.504 | 0.0045 | 167 | 208 | 291 | 292 | 4,075,519 | 18 | Cpk and p99 291 > 250 |
| `spc-2026-10-22T173809.json` | no | 182.680 | 7,116.309 | 0.0032 | 167 | 209 | 292 | 292 | 6,639,615 | 16 | Cpk and p99 292 > 250 |
| `spc-2026-10-22T184929.json` | no | 168.670 | 48.847 | 0.555 | 167 | 208 | 250 | 292 | 39,615 | 580 | Cpk only. Tightest σ on disk. Still under 1.33 |
| `spc-2026-10-22T191548.json` | no | 185.686 | 21,610.119 | 0.001 | 166 | 208 | 209 | 250 | 21,413,887 | 27 | Cpk only |

Shewhart LCL/UCL pairs, same order:

- undated: LCL −22.372, UCL 398.33. A negative LCL is what a 70 ns σ
  does to a 188 ns mean. Latency cannot be negative. The limit is the
  formula, and it still flags 70 points above 398 ns.
- inject: LCL −47,523.313, UCL 48,131.837.
- T173758: LCL −15,277.036, UCL 15,637.988.
- T173809: LCL −21,166.247, UCL 21,531.607.
- T184929: LCL 22.129, UCL 315.211. This is the only receipt whose LCL
  is positive. It is also the receipt with 580 rule-1 hits, because a
  49 ns σ makes 315 ns a 3σ breach, and the p99.9 shelf at 292 ns sits
  just under that UCL while the max of 39,615 ns does not.
- T191548: LCL −64,644.671, UCL 65,016.043.

S4 (`designs/s4-spc-design-verify.md`) re-read five receipts and corrected
the remediation design:

- p99.9 = 292 ns in four receipts, not three. T191548's p99.9 is 250 ns,
  which is the run whose body got fast enough that the shelf moved.
- p99 = 291 belongs only to T173758. T173809's p99 is 292.
- The "p99 is 208–250 on all five" sentence is false. Two receipts have
  p99 at 291 and 292.
- Rule-1 counts of 18 and 16 are in the JSON. The design had called them
  unreported.
- Intrinsic-body arithmetic S4 confirmed: a normal fit
  `(208 − 167) / 1.645 ≈ 24.9 ns`, then
  `(250 − 170) / (3 × 25) ≈ 1.07`. That 1.07 is a model of the body
  with the far tail removed. It is not a receipt field. S4's conclusion
  was that raw Cpk ≥ 1.33 is not reached by discarding the tail, and
  that the composite gate still needs a far-tail term. The gate was
  not changed. USL, the 5% variance budget, and the 1.33 floor are
  the constants in the tree today.

No receipt on disk has Cpk ≥ 1.33. The highest stored Cpk is 0.555.
The inject receipt is the anti-vacuity arm: it FAILs, and it FAILs for
both Cpk and p99. Early prose called the flip degenerate because the
uninjected arm also FAILed. That description matches the undated file's
current bytes only halfway: the current uninjected file FAILs on Cpk
while its p99 is 250 ns, so an injection that moves p99 from 250 to 333
does change the p99 clause. It does not change the verdict word, which
is FAIL on both files.

## What the prose still says about `spc-2026-10-22.json`

The file's current bytes are mean 187.979, σ 70.117, Cpk 0.2948, p99 250,
max 29,759. S4 parsed those same bytes.

These sentences describe a different object and name this filename:

- `docs/jira/v26.10.22/DISSERTATION.md` says the SPC court verdict is
  FAIL with cpk −0.0018 and p99 375 ns, and that the 10,000,000-operation
  trial is unexecuted. The 10M trial has a receipt
  (`designs/tc-hot-01-errc-receipt.md`, p99 209 ns, variance FAIL).
  The cpk −0.0018 / p99 375 pair is not in any of the six JSON files.
  The inject file is cpk −0.0011 and p99 333.
- `CHARTER-DFLSS.md` repeats cpk −0.0018 / −0.0025 and p99 375 ns for
  the smoke and the inject arm.
- `RECEIPTS.md` repeats cpk −0.0018 / −0.0025, p99 375 ns, for
  `spc-2026-10-22.json` and the inject file. The inject file on disk
  is −0.0011 and p99 333, which matches a later sentence in the same
  `RECEIPTS.md` (inject cpk −0.0011, mean 304.262, p99 333). The
  window index contains both generations.
- `DFLSS-DOSSIER.md` says Cpk is PENDING and that no Cpk number may be
  asserted. Six FAIL receipts and `cpk-correction.md` already hold
  numbers. The dossier's fold band of 184–216 ns is the min-of-batches
  instrument, which is a real measurement and a different question
  from SPC p99.

`designs/hot-path-baseline.md` keeps both generations on purpose. The
opening table's fold cell is the 10M p99 of 209 ns and the variance
FAIL. A section titled CORRECTION still records an earlier SPC reading
of p50 291 ns, p99 375 ns, Cpk about −0.002, max samples 5.5–27 ms.
That CORRECTION is history of the gate at the time it was written.
It is not the current `spc-2026-10-22.json`.

## p28 hot-path baseline and the capability study

`designs/hot-path-baseline.md`, release-mode, 1102-byte canon, 4 typed
params, min-of-batches under concurrent fan-out:

| Measurement | Result | Budget in that note | Reading |
|---|---|---|---|
| Full rehash `admitted_statement_hash` | 1673–1748 ns/op | ≤ 150 ns | about 11.6× over. A 0-param variant near 1630 ns is cited as evidence that the cost is absorbing the canon bytes |
| Incremental fold | the 10M row: p99 209 ns, batch p50s 125/166/166 | ≤ 150 ns with no gate, and ≤ 250 ns p99 | p99 inside 250; variance 26.91% FAIL |
| `compile_turtle` admission | 1.56–4.97 ms per compile | excluded from the hot path | once per compile |

The 184–216 ns band quoted in `RECEIPTS.md`, the charter, and the
dossier is this min-of-batches fold, from before the ERRC 10M run
replaced the p99 reading. `hot-path-baseline.md` says so in its
conclusions. `receipt_state` in that note is 32 bytes (`h_ast`), not
a 2 KB hasher clone. The CORRECTION in `cpk-correction.md` says the
same: post-M3 there is no 2 KB clone; `Hasher::new()` is the init
cost that was measured, and a `ChunkState` hazmat route was estimated
at a 40–80 ns saving and was not landed.

Capability study, `p28_hot_path_capability`, n = 1,000,000 per-sample
`Instant` pairs, release, USL 250 ns. Source:
`designs/cpk-correction.md`.

Least-contended run:

- μ = 278.0 ns, σ = 202.1 ns, Cpk = −0.05, P(X > 250 ns) = 42.9%.
- p50 250, p90 333, p99 375, p99.9 458, p99.99 2750, max 117,917 ns.
- Other runs under fan-out: μ = 326 / 652 / 340 / 370 / 347 ns,
  Cpk from −0.09 to 0.00, tails to 31 ms.
- Min-of-50 batches of 100 ops: 299.6 ns/op best, 397–400 ns typical.
- Disclosed clock: two `Instant::now` calls, about 40–50 ns inside
  every sample. The note's netted intrinsic band is 250–300 ns.
  That band is an argument, not a histogram field.

This p99 of 375 ns is the figure the dissertation attaches to
`spc-2026-10-22.json`. The capability study and the SPC JSON are
different programs. The capability study times one fold per sample
with a clock pair inside the sample. SPC times the bench histogram.
Both can print 375 ns. Only the capability note currently owns that
quantile. The charter Cpk of 1.44 is neither of them.

## TC-HOT-02 and PCE

`designs/tc-hot-full-receipt.md` is lane L99, two runs, both exit 1,
verdict written as REATTEMPT-QUEUED. Corpus asserted at 25,000,000
rows, 4 columns, 8.0 MB, two parquet files, loaded from
`target-lane92/tc-hot-25m`. DuckDB string in the receipt:
`v2.0.0-dev88595`, 16 threads, Apple M3 Max. Load average before run 1
was 13.85, before run 2 was 19.30. Query shape: `SELECT batch_id,
SUM(cost) FROM telemetry_events WHERE tenant_id = $1 AND latency_ms > $2
GROUP BY batch_id`, 32 rows emitted.

Run 1:

- compiled cold µs: p50 18,351, p95 24,223, p99 24,223, mean 18,668.8,
  15 reps. Batch p50s `[18607, 17295, 17295]`, spread 7.40% vs 5%, FAIL.
- compiled hot (plan-cache hit, excluded from PCE): p50 8, p95 11 µs.
- raw prepared: p50 12,415, p95 13,015, min 11,583 µs, 32 rows.
- PCE min-of-5: engine 11,583 µs, pipeline 16,432 µs, PCE 70.49%,
  substrate overhead 41.86%.
- scanned rows/sec 1,339,130,863.
- corpus assert PASS, timer sanity PASS, readback 32 = 32 PASS.

Run 2:

- compiled cold µs: p50 16,575, p95 18,575, p99 18,575, mean 16,428.3.
  Batch p50s `[17759, 15223, 17295]`, spread 15.13%, FAIL.
- compiled hot: p50 7, p95 10 µs.
- raw prepared: p50 11,111, p95 11,783, min 10,086 µs.
- PCE: engine 10,086, pipeline 12,384, PCE 81.44%, overhead 22.78%.
- scanned rows/sec 1,521,764,266.

`designs/pce-baseline.md` is a third 25M operating point, also
`TC_HOT_FULL=1`, min-of-5, warm cache, same machine class and the same
DuckDB version string:

- compiled `eval_ast_query`: p50 12,927 µs, p95 13,847 µs, min 12,384 µs.
- raw prepared floor: p50 8,823 µs, min 8,714 µs.
- PCE 70.36% (8,714 / 12,384). Substrate overhead 42.12%.
- Scanned throughput about 1.92 billion rows/s.

The charter "< 2.8%" overhead is refuted at all three of these points
(41.86%, 22.78%, 42.12%). The 12,000,000 rows/sec floor is met on the
scanned-rows figure in all three (1.34e9, 1.52e9, 1.92e9), and that
figure is a DuckDB scan rate. `ctq-traceability.md` withdraws CTQ 4
and CTQ 5 for that reason and says the bench file is deleted. The
file is 311 lines. The withdrawal describes the metric, and it
mis-describes the tree.

`pce-baseline.md` also says the 0.19–0.22 µs fold framing "holds for
the evidentiary fold". That sentence is the min-of-batches band.
The SPC receipts and the 10M variance gate are the fold's actual
verdicts, and they are FAIL.

The errc receipt says TC-HOT-02 was not run because
`target-lane92/tc-hot-25m` was absent at that later moment. L99 had
already run against that path. Absence later does not erase the L99
receipt.

## Ring push

Two witnessed numbers, two legs.

- `RECEIPTS.md` and `DFLSS-DOSSIER.md`: joint ring-to-ledger court,
  max push 292 ns against the 10 µs fence, drops persisted, verify
  passed. 6 tests, 5.14 s, in that receipts paragraph.
- `designs/cold-path-ledger.md`: release absolute leg, command
  `cargo test --release --features evidentiary-ring --test evidentiary_ring_court no_blocking_release_absolute_bound`.
  Output line: `stalled max push 1.584µs < 10µs`. 1 passed, 0.13 s,
  darwin 25.2.0. The note says this is a single shot and that no
  250 ns claim is made for the push.

292 ns and 1.584 µs are both under 10 µs. They are not the same run.
The dev-profile ratio fence is a property court (stalled ≤ 10× unstalled,
floor 100 ns). It does not publish a single absolute number in the
ledger note.

## WASM size

`tests/wasm_size_court.rs` fails if the artifact exceeds 67,108,864
bytes. The pinned module the window cites is 6,793,617 bytes, about
10.1% of the ceiling. This census did not stat the file again. Size
is included because it is the only budget court on the wasm artifact.
The hot-path benches do not run inside the module.

## CTQ matrix against this census

`designs/ctq-traceability.md`, six rows, as that file states them,
with the census correction on row 4:

1. Typed refusal ≤ 0.10 µs. 10M p50 41 ns / p99 42 ns, PASS, on the
   static `write_str` leg. The L20 smoke of the older heap-string leg
   is p99 416 ns, FAIL. The matrix cites the 10M leg.
2. H_query fold ≤ 0.15 µs. PARTIAL in the matrix because 184–216 ns
   misses 150 ns and the 150 ns row has no gate. The 10M p99 is 209 ns,
   which also misses 150 ns and meets 250 ns.
3. Evidentiary overhead ≤ 0.25 µs. FAIL. 10M p99 209 ns meets 250 ns
   and variance 26.91% fails the conjunction. SPC Cpk 0.001 fails
   1.33. The matrix cites T191548, which matches the JSON.
4. > 12M rows/sec. The matrix says REMOVED and the file deleted.
   The file is present and unregistered. L99 and the PCE note record
   scanned rates three orders of magnitude above 12M rows/sec, on a
   DuckDB scan, with variance FAIL.
5. p95 < 12 ms on `eval_ast_query`. The matrix says REMOVED. L99 run 1
   cold p95 is 24.2 ms. L99 run 2 cold p95 is 18.6 ms. The PCE note's
   p95 is 13.8 ms. All three are above 12 ms. All three time DuckDB
   plus the substrate. The matrix is right that this is not a graphlaw
   fold budget. It is wrong that no number exists.
6. Receipt coverage / anti-vacuity. PARTIAL. Mutation court is a
   separate gate (`scripts/mutation_court.sh`). It is not a latency
   bench and this census does not restate its verdict.

## What this census did not do

- No `cargo bench`, no `scripts/spc_court.sh`, no 10M run, no 25M run.
- No release ring court.
- The 150 ns fold row was not given a gate.
- USL 250 ns, refusal 100 ns, param 40 ns, variance 5%, and Cpk 1.33
  were not retuned.
- Historical sentences in `DISSERTATION.md`, `CHARTER-DFLSS.md`,
  `RECEIPTS.md`, and `DFLSS-DOSSIER.md` were not edited. They are
  cited here so the next copy does not treat them as the current JSON.
