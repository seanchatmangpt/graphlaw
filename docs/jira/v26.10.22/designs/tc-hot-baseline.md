# TC-HOT-01 baseline receipt (lane L20, 2026-10-22)

## Command and exit

```
CARGO_TARGET_DIR=target-lane20 cargo bench --bench tc_hot_01
```

- Exit code: **1**. Both FAIL gates are honest-by-design: the 250ns
  clone-path p99 budget and the 0.10µs PRD refusal budget are not met on
  the measured paths (heap-String refusal message; clone-path p99=292ns).
- The task's stated invocation `cargo run --release --bench tc_hot_01` is
  not a valid cargo form (`cargo run` rejects `--bench` → exit 1,
  "error: unexpected argument '--bench' found"); the harness=false bench
  target runs via `cargo bench --bench tc_hot_01`.
- First background attempt was killed at the 30-min background limit
  mid-compile (host contention with other lanes' builds); the restart
  resumed from cached artifacts. Compile 5m44s; the bench itself runs in
  seconds. Lane isolation honored (`target-lane20`); free disk stayed well
  above the 20GiB floor throughout (min observed ~35GiB avail).

## Machine disclosure (verbatim from run output)

```
=== TC-HOT-01 receipt hot path (re-absorb baseline vs clone path) ===
mode=SMOKE
ARCH=aarch64
core_count=16
cpu=Apple M3 Max
thermal=Note: No thermal warning level has been recorded
Note: No performance warning level has been recorded
Note: No CPU power status has been recorded
iterations=1000000 canon_bytes=649 params=5 bit_identity=VERIFIED
budget: clone-path p99 <= 250 ns
budget: refusal build+render p50/p99 <= 100 ns (0.10us PRD)
digest=[00, aa, 6b, bb, 32, 4f, dc, 50, de, 8d, 88, af, ac, df, c3, 9e, 6c, 81, d7, 3c, ac, d4, 43, f5, c5, 93, 85, 4d, ae, be, e8, ff]
thread_affinity=unavailable_stable_rust_macos (core_affinity crate not added; dev-dep would be required)
priority_elevation=requires_privileges_macos (setpriority/nice not applied)
nice_probe=true
warmup_iterations=50000 (untimed)
```

## Per-leg numbers (SMOKE, 1M iterations, 3 x 333,333 batches; verbatim)

```
VARIANCE_SELF_CHECK batches=3 batch_iters=333333 batch_p50s=[208, 208, 208] spread_pct=0.00% budget=5% : PASS
digest_tail=[00, aa, 6b, bb, 32, 4f, dc, 50, de, 8d, 88, af, ac, df, c3, 9e, 6c, 81, d7, 3c, ac, d4, 43, f5, c5, 93, 85, 4d, ae, be, e8, ff]
re_absorb_baseline: p50=1209ns p95=1583ns p99=1625ns p99.9=1917ns
clone_path: p50=208ns p95=291ns p99=292ns p99.9=375ns
refusal_invariant: p50=250ns p95=334ns p99=416ns p99.9=541ns
throughput: 518235.2 full-legs/sec (all 3 legs per iteration)
REFUSAL_INVARIANT display_render="EngineRejected refusal by Eyeron (N3): tc-hot-01 invariant probe" carries_class_and_message=true
```

## Gates and param legs (verbatim)

```
GATE clone_path p99(292ns) <= 250ns AND refusal invariant AND variance<5% : FAIL
GATE refusal_string_path p50(250ns)/p99(416ns) <= 100ns (0.10us PRD) : FAIL
NOTE closure=RefusalMessage::Static fast path (designs/P28-SIL-specs.md section 3.1), status=post-integration item; measured cost is the heap-String message path (src/dialect.rs Refusal { message: String })
RESULT param_integration_small net_ns=23.00 (gross=158.39, scaffold=135.38) budget=40ns
RESULT param_integration_large recompute net_ns=1362.54 (gross=1497.92, scaffold=135.38)
RESULT param_integration_large amortized net_ns=-28.10 (gross=107.28, scaffold=135.38) budget=40ns
GATE param_integration small net(23.00ns) / large amortized net(-28.10ns) <= 40ns : PASS
TIMESTAMP_SOURCE coarse_now_ns per-call=0.7ns (overhead-subtracted) instant_baseline=25.6ns monotonic=true
GATE timestamp_source 0.7ns < 10ns AND monotonic : PASS
```

Cargo tail (verbatim):

```
error: bench failed, to rerun pass `--bench tc_hot_01`
Caused by:
  process didn't exit successfully: /Users/sac/graphlaw/target-lane20/release/deps/tc_hot_01-00be4e77caf66c53 --bench (exit status: 1)
```

## Charter mapping (legs → TC-HOT charter items)

Charter source: `docs/jira/v26.10.22/designs/TC-HOT-benchmarks.md`; the
same mapping ships as the comment block at the top of `benches/tc_hot_01.rs`
(lines 19-37).

| Leg / output line | Charter item |
|---|---|
| `re_absorb_baseline` + `clone_path` legs | TC-HOT-01 latency court — H_query + invariant ≤ 0.25µs p99 (250ns), 10M consecutive under `TC_HOT_FULL=1`; evidentiary delta: full re-absorb `admitted_statement_hash` vs anchored `receipt_state` fold |
| `refusal_invariant` leg | TC-HOT-01 refusal-invariant leg — Refusal construction + Display (src/dialect.rs refusal surface) vs the 0.10µs PRD budget; honest FAIL expected until `RefusalMessage::Static` (P28-SIL-specs.md §3.1) lands |
| `param_integration` small/large legs | Charter A03 — `ParamBytes::encode` + single `Hasher` fold ≤ 40ns p99 per leg (inline <64B and spill >64B vectors) |
| `timestamp_source` leg | Lane A02 — CoarseClock `coarse_now_ns` < 10ns median acquisition + monotonicity, bare `Instant` baseline |
| `VARIANCE_SELF_CHECK` / `TC_HOT_SPC_EXPORT` / `TC_HOT_INJECT_OUTLIER_NS` hooks | measurement-isolation charter invariant (run-to-run variance < 5%); feeds `scripts/spc_court.sh` (Cpk ≥ 1.33 receipt-fold gate) |

## SUSPECTED-TIMER-DEFECT check (PCE ~2000x inversion precedent)

**No tc_hot_01 number is timer-suspect.** The baseline-vs-compiled ratio is
5.8x (1209ns re-absorb vs 208ns anchored clone path), directionally and
quantitatively consistent with the expected mechanism (the canon half
precomputed into `receipt_state` at compile time); nothing is
microsecond-class where milliseconds are expected. The nearest-to-suspect
value is the **negative amortized large-leg net (-28.10ns: gross 107.28ns
below scaffold 135.38ns)** — an artifact of the min-of-N batch statistic
against a separately-minimized scaffold, not an impossible magnitude; it is
disclosed verbatim in the output and the gate passes on it. Flagged as a
measurement caveat, not a timer defect.

## Concurrent-edit disclosure

While this lane owned the annotation block only, the measurement-isolation
charter lane's uncommitted edits to the same file (min-of-N `PARAM_BATCHES`
batch statistic, `param_integration_large recompute`/`amortized` split,
`unused variable: batch` warning) were present in the working tree and are
what the compiled bench above actually ran. This lane changed nothing in
the measurement logic.
