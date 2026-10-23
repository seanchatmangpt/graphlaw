#!/usr/bin/env bash
# receipt_replay_court.sh — K7-class court: the blake3 chain proves byte-immutability,
# not truth. A fabricated receipt (never-executed output) verifies clean against the
# chain. This court re-executes the embedded `cargo test --test X` command from a
# sample of receipts and compares embedded counts vs observed counts.
#
# Usage:
#   scripts/receipt_replay_court.sh              # check + selfcheck (default: all)
#   scripts/receipt_replay_court.sh check        # sample N=5, replay, compare
#   scripts/receipt_replay_court.sh selfcheck    # anti-vacuity: fabricated receipt must be flagged
#
# Exit codes: 0 = court admits; 1 = divergence or VACUOUS GATE.
# Sampled receipts rotate deterministically by (date, index): same day → same sample,
# next day → different sample, full pool coverage over time.
set -euo pipefail

REPO="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
RECEIPTS_DIR="$REPO/receipts"
N=5
DATE="$(date -u +%Y-%m-%d)"
WORK="$(mktemp -d /tmp/receipt-replay-court.XXXXXX)"
trap 'rm -rf "$WORK"' EXIT
LANE_TARGET="$WORK/lane-target"
OBS_STATE="$WORK/observed.tsv"   # receipt<TAB>passed<TAB>failed, written by check

fail() { echo "RECEIPT-REPLAY-COURT FAIL: $*" >&2; exit 1; }

# --- pool: receipts carrying a `cargo test --test X` command -------------------
pool() {
  # Replayable = at least one cleanly-paired (command, ok-result) pair.
  local f
  for f in "$RECEIPTS_DIR"/*.md; do
    if cmd_result_pairs "$f" | grep -q .; then echo "$f"; fi
  done
}

# --- deterministic rotation by (date, index) -----------------------------------
# seed = sha256(DATE) as integer; sample indices seed+i mod poolsize, deduped.
sample_pool() {
  local pool_file="$1" seed i idx
  seed=$((0x$(printf '%s' "$DATE" | shasum -a 256 | cut -c1-12) ))
  local size
  size=$(wc -l < "$pool_file" | tr -d ' ')
  if [ "$size" -lt "$N" ]; then
    fail "pool of $size replayable receipts < N=$N — enlarge the receipt pool"
  fi
  : > "$WORK/sample.txt"
  i=0
  # Emit up to N*3 candidates in rotation order; the replay loop counts
  # successful replays and stops at N. Candidates that loud-SKIP (target
  # mid-edit in the working tree) are replaced by the next rotation slot.
  while [ "$i" -lt $((N * 3)) ] && [ "$i" -lt "$size" ]; do
    idx=$(( (seed + i) % size + 1 ))
    sed -n "${idx}p" "$pool_file" >> "$WORK/sample.txt"
    sort -u "$WORK/sample.txt" -o "$WORK/sample.txt"
    i=$((i + 1))
  done
}

# --- extract embedded command+counts PAIRS from one receipt --------------------
# Pairing matters: a receipt can carry multiple commands (e.g. one FAILED run and
# one ok run). Pairing command 1 with command 2's counts would itself fabricate.
# cmd_result_pairs <receipt> -> lines of "target passed failed" for each
# (cargo test --test X command, subsequent `test result: ok.` line) pair.
cmd_result_pairs() {
  awk '
    /cargo test --test/ { if (match($0, /--test[ \t]+[a-z0-9_]+/)) target=substr($0, RSTART+6, RLENGTH-6); gsub(/[ \t]/, "", target) }
    /test result: ok\. [0-9]+ passed; [0-9]+ failed/ {
      if (target != "") {
        line=$0; gsub(/[^0-9 ]/, "", line)
        split(line, a, " ")
        print target, a[1], a[2]
        target=""
      }
    }
  ' "$1"
}

# embedded_cmd <receipt> -> echoes "target passed failed" of the FIRST ok pair
embedded_first_pair() {
  local receipt="$1"
  cmd_result_pairs "$receipt" | head -1
}

# --- replay: re-execute the embedded command in a lane dir ----------------------
replay_one() {
  local receipt="$1" target passed failed obs
  read -r target passed failed <<< "$(embedded_first_pair "$receipt")"
  [[ "$passed" =~ ^[0-9]+$ && "$failed" =~ ^[0-9]+$ ]] \
    || fail "$receipt: embedded counts unparseable (got '$passed'/'$failed') — refusing vacuous compare"

  if [ ! -f "$REPO/tests/$target.rs" ]; then
    fail "$receipt: embedded command targets tests/$target.rs which does not exist"
  fi

  # A receipt replayed against a tree whose target is mid-edit (another lane's
  # in-flight diff) diverges by staleness, not by fabrication. Skip loudly;
  # the receipt stays in the pool for a clean-tree wave.
  if ! git -C "$REPO" diff --quiet -- "tests/$target.rs"; then
    echo "[SKIP] $receipt: tests/$target.rs is mid-edit in the working tree — replay would conflate staleness with fabrication; re-sampling"
    return 1
  fi

  echo "-- replay $receipt : cargo test --test $target (embedded ${passed}p/${failed}f)"
  local out
  if ! out=$(CARGO_TARGET_DIR="$LANE_TARGET" cargo test --test "$target" --quiet 2>&1); then
    echo "$out" | tail -20 >&2
    fail "$receipt: replayed command exited nonzero — receipt output not reproducible"
  fi
  obs=$(echo "$out" | grep -m1 -o 'test result: ok\. [0-9]* passed; [0-9]* failed' || true)
  [ -n "$obs" ] || fail "$receipt: replay produced no parseable test result line"
  obs="${obs//[!0-9 ]/}"
  local opass ofail
  read -r opass ofail <<< "$(echo "$obs" | awk '{print $1, $2}')"

  printf '%s\t%s\t%s\n' "$receipt" "$opass" "$ofail" >> "$OBS_STATE"

  if [ "$opass" -ne "$passed" ] || [ "$ofail" -ne "$failed" ]; then
    fail "$receipt: COUNT DIVERGENCE — embedded ${passed}p/${failed}f vs observed ${opass}p/${ofail}f (fabricated or stale receipt)"
  fi
  echo "   match: ${opass}p/${ofail}f"
}

# --- check mode ----------------------------------------------------------------
cmd_check() {
  pool > "$WORK/pool.txt"
  [ -s "$WORK/pool.txt" ] || fail "no replayable receipts in $RECEIPTS_DIR"
  sample_pool "$WORK/pool.txt"
  echo "== sampled (rotation seed $DATE), replaying until N=$N match:"
  : > "$OBS_STATE"
  local ok=0
  while IFS= read -r r; do
    [ "$ok" -ge "$N" ] && break
    if replay_one "$r"; then ok=$((ok + 1)); else
      [ -n "${SKIP_SEEN:-}" ] || true
    fi
  done < "$WORK/sample.txt"
  if [ "$ok" -lt "$N" ]; then
    fail "only $ok/$N sampled receipts replayed cleanly — too many mid-edit targets; rerun on a clean tree"
  fi
  echo "== PASS: $N sampled receipts reproduce their embedded output byte-for-count"
}

# --- selfcheck: fabricated receipt must be flagged (anti-vacuity) ---------------
cmd_selfcheck() {
  [ -s "$OBS_STATE" ] || fail "selfcheck requires a prior check run (no observed state)"
  # Fabricate: take the first sampled receipt, inflate its counts to impossible
  # values (42 passed on whatever the real suite size is — K7 protocol), keep the
  # embedded command. The court MUST flag it against the same observed replay.
  local real fabrication
  real=$(awk 'NR==1{print $1}' "$OBS_STATE")
  fabrication="$WORK/fabricated-receipt.md"
  sed -E 's/test result: ok\. [0-9]+ passed; [0-9]+ failed/test result: ok. 42 passed; 0 failed/' \
    "$real" > "$fabrication"
  read -r _ opass ofail <<< "$(awk -v r="$real" '$1==r{print; exit}' "$OBS_STATE")"
  echo "== selfcheck: planted fabricated receipt (42 passed) for $real"
  # Run the same comparison the check path uses, against the observed replay.
  local target passed failed obs rc=0
  read -r target passed failed <<< "$(embedded_first_pair "$fabrication")"
  if [ "$passed" -ne 42 ]; then
    fail "selfcheck fabrication did not plant 42 — sed no-op, selfcheck is vacuous"
  fi
  # Compare fabricated-embedded counts against the real observed counts (the
  # replay itself is shared with check mode; the court's job is the comparison).
  set +e
  if [ "$opass" -ne "$passed" ] || [ "$ofail" -ne "$failed" ]; then
    echo "   flagged: fabricated ${passed}p/${failed}f vs observed ${opass}p/${ofail}f"
  else
    rc=1
  fi
  set -e
  if [ "$rc" -ne 0 ]; then
    fail "VACUOUS GATE: fabricated receipt with 42 passed was NOT flagged — comparison is dead"
  fi
  echo "== selfcheck PASS: court flags fabricated counts (exit 1 class exercised)"
}

case "${1:-all}" in
  check)     cmd_check ;;
  selfcheck) cmd_selfcheck ;;
  all)       cmd_check; cmd_selfcheck ;;
  *) fail "usage: receipt_replay_court.sh [check|selfcheck|all]" ;;
esac
