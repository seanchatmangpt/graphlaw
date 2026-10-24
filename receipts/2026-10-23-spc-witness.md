# SPC Gate Witness — 2026-10-23 (audit finding #10)

## Subject

- Repo: /Users/sac/graphlaw
- Command: `CARGO_TARGET_DIR=target-lane-spc bash scripts/spc_court.sh`
- Mode: SMOKE (1M iterations), no outlier injection, USL=250ns, CPK_MIN=1.33
- Isolation: dedicated `target-lane-spc` build root; script only builds the
  `tc_hot_01` bench and writes an append-only `receipts/spc-<stamp>.json` —
  no mutation of `src/` or shared files; safe alongside concurrent lanes.
- Exit code: **1**

## Verdict: FAIL

Cpk **0.0008** < 1.33 threshold. p99 (209ns) is within the 250ns USL, but the
sample stddev is 16282.667ns (max sample 7,483,391ns — outlier tail), collapsing
the one-sided Cpk to ~0.

## Embedded real output (tail of run)

```
spc_court: run mode=SMOKE inject=0 usl=250ns cpk_min=1.33
=== TC-HOT-01 receipt hot path (re-absorb baseline vs clone path) ===
mode=SMOKE
ARCH=aarch64  core_count=16  cpu=Apple M3 Max
iterations=1000000 canon_bytes=649 params=5 bit_identity=VERIFIED
digest=[00, aa, 6b, bb, 32, 4f, dc, 50, de, 8d, 88, af, ac, df, c3, 9e, 6c, 81, d7, 3c, ac, d4, 43, f5, c5, 93, 85, 4d, ae, be, e8, ff]
VARIANCE_SELF_CHECK batches=3 batch_iters=333333 batch_p50s=[166, 166, 166] spread_pct=0.00% budget=5% : PASS
re_absorb_baseline: p50=1208ns p95=1375ns p99=1625ns p99.9=1750ns
clone_path: p50=166ns p95=167ns p99=209ns p99.9=291ns
refusal_invariant: p50=41ns p95=42ns p99=42ns p99.9=42ns
GATE clone_path p99(209ns) <= 250ns AND refusal invariant AND variance<5% : PASS
GATE refusal_string_path p50(41ns)/p99(42ns) <= 100ns (0.10us PRD) : PASS
SPC_EXPORT path=.../spc-court.json samples=999999
{
  "receipt": "spc",
  "date": "2026-10-23",
  "bench": "tc_hot_01",
  "leg": "clone_path",
  "mode": "SMOKE",
  "inject_outlier": false,
  "usl_ns": 250.0,
  "cpk_min": 1.33,
  "mean_ns": 210.397,
  "stddev_ns": 16282.667,
  "cpk": 0.0008,
  "p50_ns": 166,
  "p95_ns": 167,
  "p99_ns": 209,
  "p999_ns": 291,
  "max_ns": 7483391,
  "iterations": 1000000,
  "samples": 999999,
  "shewhart": {
    "rule1_beyond_3sigma": { "flagged": true, "count": 33, "lcl_ns": -48637.604, "ucl_ns": 49058.398 },
    "rule2_monotonic_run_8": { "flagged": false, "longest_run": 6, "direction": "down" },
    "assignable_cause_detected": true
  },
  "verdict": "FAIL",
  "reasons": [ "cpk 0.0008 < 1.33" ]
}
spc_court: FAIL — process out of control (see receipt above)
EXIT_CODE=1
```

## Machine receipt

- `receipts/spc-2026-10-23T152558.json`
  sha256 `538ad80fa453f977ecb264d98ac50cc67b9bf2b269b634b676926dd0d0c47b1e`

## Notes

- The gate is NOT vacuous: it correctly exits 1 on the Cpk criterion and
  Shewhart rule 1 flags 33 samples beyond 3-sigma (assignable cause detected).
- No tuning was applied — reported faithfully per instructions. Root cause of
  the wide tail (rare multi-ms samples under concurrent lane load) is UNKNOWN;
  a solo-machine rerun would be the next falsifier.
