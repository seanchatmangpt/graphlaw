# SPC Gate Solo Rerun — 2026-10-24 (falsifier for the 2026-10-23 fleet-load FAIL)

## Subject

- Repo: /Users/sac/graphlaw
- Command: `CARGO_TARGET_DIR=target-lane-spc2 bash scripts/spc_court.sh`
- Mode: SMOKE (1M iterations), no outlier injection, USL=250ns, CPK_MIN=1.33
- Isolation: dedicated `target-lane-spc2` build root (cold build, release,
  6m18s); fleet drained — machine otherwise idle (solo falsifier condition).
- Exit code: **1** (captured bare, no pipe: `EXIT_CODE=1` appended from the
  court's own process, not a pipeline tail)

## Verdict: FAIL — Cpk 0.632 < 1.33

The gate lawfully exits 1 again, but the failure signature changed materially.
The fleet-load hypothesis is CONFIRMED for the old fat tail; it is NOT a full
explanation for the Cpk failure:

| metric | 2026-10-23 (fleet load) | 2026-10-24 (solo) | delta |
|---|---|---|---|
| mean_ns | 210.397 | 128.202 | -39% |
| stddev_ns | 16282.667 | 64.24 | -253x |
| cpk | 0.0008 | 0.632 | +790x |
| p50_ns | 166 | 125 | -25% |
| p95/p99_ns | 167 / 209 | 167 / 167 | p99 now at p95 |
| p999_ns | 291 | 250 | at USL, in spec |
| max_ns | 7,483,391 | 24,591 | -304x |
| shewhart rule1 count | 33 | 283 | tail still present |

Solo, the multi-ms outliers are gone (max 24.6µs vs 7.48ms): the prior
tail was load-induced. The remaining FAIL is the body+shoulder of the
distribution itself: mean 128.2ns with sd 64.24ns against USL 250ns gives
Cpk = (250-128.202)/(3*64.24) = 0.632. p99 (167ns) is well inside the USL —
the internal budget gate PASSes — but the Cpk criterion demands
mean + 4*sd <= ~250ns at this skew, i.e. sd <= ~30ns at this mean, or a
lower mean at this sd.

## Embedded real output (tail of run)

```
spc_court: run mode=SMOKE inject=0 usl=250ns cpk_min=1.33
=== TC-HOT-01 receipt hot path (re-absorb baseline vs clone path) ===
mode=SMOKE
ARCH=aarch64
core_count=16
cpu=Apple M3 Max
thermal=Note: No thermal warning level has been recorded
iterations=1000000 canon_bytes=649 params=5 bit_identity=VERIFIED
thread_affinity=unavailable_stable_rust_macos (core_affinity crate not added; dev-dep would be required)
priority_elevation=requires_privileges_macos (setpriority/nice not applied)
warmup_iterations=50000 (untimed)
VARIANCE_SELF_CHECK batches=3 batch_iters=333333 batch_p50s=[125, 125, 125] spread_pct=0.00% budget=5% : PASS
re_absorb_baseline: p50=1041ns p95=1084ns p99=1250ns p99.9=1333ns
clone_path: p50=125ns p95=167ns p99=167ns p99.9=250ns
refusal_invariant: p50=0ns p95=42ns p99=42ns p99.9=42ns
GATE clone_path p99(167ns) <= 250ns AND refusal invariant AND variance<5% : PASS
GATE refusal_string_path p50(0ns)/p99(42ns) <= 100ns (0.10us PRD) : PASS
SPC_EXPORT path=/Users/sac/.cache/tmp/spc-court.IIqB9lal/spc-court.json samples=999999
GATE timestamp_source 0.6ns < 10ns AND monotonic : PASS
{
  "receipt": "spc",
  "date": "2026-10-24",
  "bench": "tc_hot_01",
  "leg": "clone_path",
  "mode": "SMOKE",
  "inject_outlier": false,
  "usl_ns": 250.0,
  "cpk_min": 1.33,
  "mean_ns": 128.202,
  "stddev_ns": 64.24,
  "cpk": 0.632,
  "p50_ns": 125,
  "p95_ns": 167,
  "p99_ns": 167,
  "p999_ns": 250,
  "max_ns": 24591,
  "iterations": 1000000,
  "samples": 999999,
  "shewhart": {
    "rule1_beyond_3sigma": {
      "flagged": true,
      "count": 283,
      "lcl_ns": -64.518,
      "ucl_ns": 320.922
    },
    "rule2_monotonic_run_8": {
      "flagged": false,
      "longest_run": 4,
      "direction": "up"
    },
    "assignable_cause_detected": true
  },
  "verdict": "FAIL",
  "reasons": [
    "cpk 0.6320 < 1.33"
  ]
}
spc_court: FAIL — process out of control (see receipt above)
EXIT_CODE=1
```

## Machine receipt

- `receipts/spc-2026-10-24T032511.json`
  sha256 `b346c2ae873243cd8dd66025c9828b7c33f642b0761cb635fe84176b8061d650`

## Notes

- The gate is NOT vacuous: it exits 1 on the Cpk criterion with p99 in spec,
  and rule 1 flags the residual right tail (283 samples beyond 3-sigma).
- The bench itself reported no thermal/power warnings and 0.00% batch spread;
  the residual 24.6µs max and the 64ns sd are process properties, not
  measurement artifacts.
- FULL mode (10M iterations) was not run; SMOKE is the standing rung per the
  script's default. Escalation to SPC_MODE=FULL is the next rung if the body
  sd is suspected to be a sampling artifact.
- Comparison baseline: `receipts/spc-2026-10-23T152558.json`
  (sha256 `538ad80fa453f977ecb264d98ac50cc67b9bf2b269b634b676926dd0d0c47b1e`,
  from receipts/2026-10-23-spc-witness.md).
