#!/usr/bin/env bash
# SPC court — §5 statistical process control over the receipt hot path.
#
# Test-ladder position: run AFTER scripts/mutation_court.sh (the mutation
# court proves the gates are non-vacuous; this court proves the measured
# process is within control limits). See repo CLAUDE.md "Test ladder".
#
#   scripts/spc_court.sh                 SMOKE run (1M iterations default sample)
#   SPC_MODE=FULL scripts/spc_court.sh   10M-iteration full run (TC_HOT_FULL=1)
#   SPC_INJECT_OUTLIER=1 ...             anti-vacuity: inject a real 2µs
#                                        busy-spin outlier into every recorded
#                                        clone-path sample; the verdict MUST
#                                        flip to FAIL or the gate exits 1
#                                        (VACUOUS GATE).
#
# Computes μ/σ/Cpk from the real measured sample against USL=250ns (the
# TC-HOT-01 BUDGET_NS, sql-rdf Rec. section 1.4 receipt hash budget) and
# exits 1 when Cpk < 1.33 or p99 > USL. Writes the histogram/quantile
# summary to receipts/spc-<YYYY-MM-DD>.json.
#
# Env: CARGO_TARGET_DIR (default target-dflss12), SPC_MODE (SMOKE|FULL),
#      SPC_INJECT_OUTLIER, SPC_USL_NS (default 250), SPC_CPK_MIN (default 1.33).
# Chicago: real frozen bench binary, real hashes, real measurements; no mocks,
# no sleeps (the fault injection is a busy-spin, never a timer wait).
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-target-dflss12}"
USL_NS="${SPC_USL_NS:-250}"
CPK_MIN="${SPC_CPK_MIN:-1.33}"
MODE="${SPC_MODE:-SMOKE}"
INJECT="${SPC_INJECT_OUTLIER:-0}"
mkdir -p receipts

# L57 mktemp robustness (L50 finding): a polluted TMPDIR (stale spc-court.*
# files, huge dir) can make mktemp's mkstemp fail. Use a fresh mktemp -d
# scratch dir (atomically created, no name collision with stale entries),
# falling back to /tmp if TMPDIR is unusable. Measurement behavior unchanged.
SPC_SCRATCH=""
for d in "${TMPDIR:-/tmp}" /tmp; do
  SPC_SCRATCH="$(mktemp -d "${d%/}/spc-court.XXXXXXXX" 2>/dev/null)" && break
  SPC_SCRATCH=""
done
[ -n "$SPC_SCRATCH" ] || { echo "spc_court: cannot create temp scratch dir" >&2; exit 2; }
TMP_JSON="$SPC_SCRATCH/spc-court.json"
trap 'rm -rf "$SPC_SCRATCH"' EXIT

echo "spc_court: building tc_hot_01 (release) in $CARGO_TARGET_DIR" >&2
cargo build --release --bench tc_hot_01 >&2

BENCH="$CARGO_TARGET_DIR/release/deps/tc_hot_01"
[ -x "$BENCH" ] || BENCH="$(ls -t "$CARGO_TARGET_DIR/release/deps/"tc_hot_01-* 2>/dev/null | grep -v '\.d$' | head -1)"
[ -x "$BENCH" ] || { echo "spc_court: bench binary not found after build" >&2; exit 2; }

ENV_ARGS=()
[ "$MODE" = FULL ] && ENV_ARGS+=("TC_HOT_FULL=1")
[ "$INJECT" = 1 ] && ENV_ARGS+=("TC_HOT_INJECT_OUTLIER_NS=2000")

echo "spc_court: run mode=$MODE inject=$INJECT usl=${USL_NS}ns cpk_min=$CPK_MIN" >&2
BENCH_RC=0
env ${ENV_ARGS[@]+"${ENV_ARGS[@]}"} TC_HOT_SPC_EXPORT="$TMP_JSON" "$BENCH" >&2 || BENCH_RC=$?
# The bench's own internal budget gate is a separate court; the SPC court owns
# its verdict from the exported sample. Only a REFUSED-style abort (rc>=2) is
# an infrastructure failure here.
[ "$BENCH_RC" -le 1 ] || { echo "spc_court: bench aborted (rc=$BENCH_RC)" >&2; exit 2; }

[ -s "$TMP_JSON" ] || { echo "spc_court: no SPC export written by bench" >&2; exit 2; }

# μ/σ/Cpk + verdict from the real sample; emit the receipt JSON.
set +e
RECEIPT="$(python3 - "$TMP_JSON" "$USL_NS" "$CPK_MIN" "$MODE" "$INJECT" <<'PY'
import json, sys, datetime

path, usl, cpk_min, mode, inject = sys.argv[1], float(sys.argv[2]), float(sys.argv[3]), sys.argv[4], sys.argv[5] == "1"
s = json.load(open(path))
mean, sd, p99 = s["mean_ns"], s["stddev_ns"], s["p99_ns"]
samples = s.get("samples_ns", [])
# Shewhart control-chart rules (Western Electric 1+2 minimum) over the real
# per-sample clone-path series. Detection only: rule flags are assignable-cause
# evidence; the overall verdict stays owned by Cpk + p99 against the USL.
rule1 = rule2 = False
r1_count = 0
longest_run = 0
longest_run_dir = ""
if samples and sd > 0:
    lcl, ucl = mean - 3 * sd, mean + 3 * sd  # 3-sigma action limits
    r1_count = sum(1 for x in samples if x > ucl or x < lcl)
    rule1 = r1_count > 0
    # Rule 2: longest monotonic (strictly) consecutive run, either direction.
    run = 1
    run_dir = 0
    for i in range(1, len(samples)):
        d = (samples[i] > samples[i - 1]) - (samples[i] < samples[i - 1])
        if d != 0 and d == run_dir:
            run += 1
        else:
            run = 2 if d != 0 else 1
            run_dir = d
        if run > longest_run:
            longest_run, longest_run_dir = run, ("up" if run_dir > 0 else "down")
    rule2 = longest_run >= 8
rules_flagged = rule1 or rule2
# One-sided Cpk against the upper spec limit (latency budget).
cpk = (usl - mean) / (3 * sd) if sd > 0 else float("-inf")
passed = cpk >= cpk_min and p99 <= usl
out = {
    "receipt": "spc",
    "date": datetime.date.today().isoformat(),
    "bench": "tc_hot_01",
    "leg": "clone_path",
    "mode": mode,
    "inject_outlier": inject,
    "usl_ns": usl,
    "cpk_min": cpk_min,
    "mean_ns": round(mean, 3),
    "stddev_ns": round(sd, 3),
    "cpk": round(cpk, 4) if sd > 0 else None,
    "p50_ns": s["p50_ns"], "p95_ns": s["p95_ns"], "p99_ns": p99,
    "p999_ns": s["p999_ns"], "max_ns": s["max_ns"],
    "iterations": s["iterations"],
    "samples": len(samples),
    "shewhart": {
        "rule1_beyond_3sigma": {
            "flagged": rule1,
            "count": r1_count,
            "lcl_ns": round(mean - 3 * sd, 3) if sd > 0 else None,
            "ucl_ns": round(mean + 3 * sd, 3) if sd > 0 else None,
        },
        "rule2_monotonic_run_8": {
            "flagged": rule2,
            "longest_run": longest_run,
            "direction": longest_run_dir or None,
        },
        "assignable_cause_detected": rules_flagged,
    },
    "verdict": "PASS" if passed else "FAIL",
    "reasons": [r for r, bad in [
        (f"cpk {cpk:.4f} < {cpk_min}", cpk < cpk_min),
        (f"p99 {p99}ns > {usl}ns", p99 > usl),
    ] if bad] or None,
}
print(json.dumps(out, indent=2))
sys.exit(0 if passed else 1)
PY
)"
RC=$?
set -e

# Append-only receipts (L111 fix): every run writes a NEW timestamped file
# (spc-<date>T<HHMMSS>.json); no same-day file is ever overwritten in place.
# Immutability chains consume append-only inputs — an overwrite breaks re-sync
# forever (witnessed: spc-2026-10-22.json drift is permanent history).
STAMP="$(date +%Y-%m-%dT%H%M%S)"
# SPC_INJECT_OUTLIER runs are anti-vacuity probes: they document the flip, they
# do not overwrite the standing receipt (which must reflect the real process).
# A4 fix: hoist the receipt path so writer, readback and PASS message all use
# one variable (previously the rule-1 readback opened a stale file, not the
# receipt just written).
RECEIPT_PATH="receipts/spc-${STAMP}.json"
if [ "$INJECT" = 1 ]; then
  RECEIPT_PATH="receipts/spc-${STAMP}-inject.json"
fi
echo "$RECEIPT" > "$RECEIPT_PATH"
echo "$RECEIPT" >&2

if [ "$INJECT" = 1 ]; then
  if ! python3 -c '
import json,sys
try:
    r=json.load(open(sys.argv[1]))
    flagged=r["shewhart"]["rule1_beyond_3sigma"]["flagged"]
except Exception as e:
    print(f"spc_court: rule-1 check error: {e!r}", file=sys.stderr); sys.exit(2)
sys.exit(0 if flagged else 1)
' "$RECEIPT_PATH" ; then
    echo "spc_court: VACUOUS GATE — injected outliers did NOT trip Shewhart rule 1 (beyond mean±3σ)" >&2
    exit 1
  fi
  if [ "$RC" -eq 0 ]; then
    echo "spc_court: VACUOUS GATE — injected 2µs outliers did NOT flip the verdict to FAIL" >&2
    exit 1
  fi
  echo "spc_court: anti-vacuity OK — injected outliers flipped verdict to FAIL and tripped Shewhart rule 1 (as required)" >&2
  exit 0
fi
if [ "$RC" -ne 0 ]; then
  echo "spc_court: FAIL — process out of control (see receipt above)" >&2
  exit 1
fi
echo "spc_court: PASS — receipt: $RECEIPT_PATH" >&2
