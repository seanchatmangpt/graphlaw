#!/usr/bin/env bash
# Native mutation court over the ABI registry surface (src/registry.rs,
# src/registry/*.rs, src/abi.rs).
#
#   scripts/mutation_court.sh            run the court, print the summary
#   scripts/mutation_court.sh --list     list mutants only (no test run)
#   scripts/mutation_court.sh --check    run the court; exit 3 when any mutant in
#                                        src/registry.rs, in src/registry/*.rs,
#                                        or in the capabilities()/dispatch()
#                                        bodies of src/abi.rs is MISSED
#   scripts/mutation_court.sh --leg callsite-revert
#                                        run ONLY the callsite-revert leg: patch
#                                        law.rs authority_for_capability to return
#                                        Ok unconditionally (SHIM-COURT), assert
#                                        the law_state UNKNOWN-STEP-CAPABILITY
#                                        court FAILS under the shim, restore
#                                        byte-exact (sha256). Exit 7 on a vacuous
#                                        leg (test still passes under the shim)
#                                        or on shim compile failure.
#
# Streaming / resumability (L27):
#   --log <file>    tee all court output (stdout+stderr) to <file> so per-case
#                   progress survives a kill at any point.
#   --case <name>   run only mutants whose description matches <name> (regex,
#                   passed to cargo-mutants -F/--re). Repeatable (OR-ed).
#   --force         override the dirty-tree refusal.
#
# Dirty-tree refusal (typed): cargo-mutants runs --in-place, so uncommitted
# work under src/ would be mutated in place and false-red concurrent lanes.
# The court refuses to start when `git status --porcelain` shows any entry
# under src/ (tracked modifications or untracked files), naming the dirty
# files. Exit code 6.
#
# Restore is verified byte-exact by sha256 before/after each guarded file;
# a mismatch is a loud fail-closed exit 4.
#
# Env: CARGO_TARGET_DIR (default target-mutants), MUTANTS_OUTPUT (default: repo root,
# which yields ./mutants.out), MUTANTS_EXTRA (extra cargo-mutants args).
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

# ---- argument parsing (any order) -------------------------------------------
MODE=run
FORCE=0
LOG_FILE=""
CASES=()
LEGS=()
while [ $# -gt 0 ]; do
  case "$1" in
    "") shift ;;
    --list) MODE=list; shift ;;
    --check) MODE=check; shift ;;
    --leg)
      [ $# -ge 2 ] || { echo "mutation_court: --leg requires a name" >&2; exit 2; }
      LEGS+=("$2"); shift 2 ;;
    --force) FORCE=1; shift ;;
    --log)
      [ $# -ge 2 ] || { echo "mutation_court: --log requires a file argument" >&2; exit 2; }
      LOG_FILE="$2"; shift 2 ;;
    --case)
      [ $# -ge 2 ] || { echo "mutation_court: --case requires a mutant-name argument" >&2; exit 2; }
      CASES+=("$2"); shift 2 ;;
    -h|--help) sed -n '2,40p' "$0"; exit 0 ;;
    *) echo "mutation_court: unknown argument: $1 (use --list | --check | --log F | --case N | --force)" >&2; exit 2 ;;
  esac
done

# ---- legs (defined + dispatched before the dirty-tree refusal; the leg is
# ---- self-snapshotting and byte-exact-restoring, so --leg mode never gates on
# ---- uncommitted src/ work) -----------------------------------------------
# callsite-revert (lane M05, K10): the law_state capability court must fail when
# the law.rs authority helper is neutered (returns Ok unconditionally). This is
# the non-vacuity leg: a court that still passes under the shim is a vacuous
# gate. Shim-court protocol: sha256 baseline -> shim applied and marked
# SHIM-COURT -> court must FLIP -> byte-exact restore verified by sha256.
leg_callsite_revert() {
  local law=src/law.rs
  local snap_dir
  snap_dir="$(mktemp -d "${TMPDIR:-/tmp}/mutation-court-leg.XXXXXX")"
  local snap="$snap_dir/law.rs.shim-court"
  cp -p "$law" "$snap"
  local sha_before now
  sha_before="$(shasum -a 256 "$law" | awk '{print $1}')"

  # Shim: replace the body of authority_for_capability with an unconditional Ok
  # (first declared authority). Marked SHIM-COURT.
  python3 - "$law" <<'PYEOF'
import sys
path = sys.argv[1]
with open(path) as f:
    lines = f.readlines()
sig = "pub fn authority_for_capability(capability: &str) -> Result<BackendAuthority, LawError> {"
idx = next(i for i, l in enumerate(lines) if sig in l)
depth = 0
end = None
for i in range(idx, len(lines)):
    depth += lines[i].count("{") - lines[i].count("}")
    if depth == 0:
        end = i
        break
body = [
    sig + "\n",
    "    // SHIM-COURT callsite-revert: helper neutered to always-Ok (never commit this)\n",
    "    let _ = capability;\n",
    "    Ok(BACKEND_AUTHORITIES[0])\n",
    "}\n",
]
lines[idx:end + 1] = body
with open(path, "w") as f:
    f.writelines(lines)
PYEOF
  if ! grep -q "SHIM-COURT callsite-revert" "$law"; then
    echo "mutation_court: callsite-revert leg FAILED to apply shim" >&2
    cp "$snap" "$law"
    exit 7
  fi

  # The court under the shim: must FAIL (flip). A compile abort is not a flip.
  local out rc=0
  out="$(cargo test --test law_state transition_with_unknown_capability_refuses_typed_end_to_end 2>&1)" || rc=$?
  if printf '%s' "$out" | grep -qE 'error\[|could not compile'; then
    printf '%s\n' "$out" | tail -40 >&2
    echo "mutation_court: callsite-revert shim did not COMPILE (compile abort, not a flip)" >&2
    cp "$snap" "$law"
    exit 7
  fi
  if [ "$rc" -eq 0 ]; then
    echo "mutation_court: VACUOUS COURT: law_state capability court still PASSES with the" >&2
    echo "  authority helper neutered to always-Ok — the gate cannot exit 1. Exit 7." >&2
    cp "$snap" "$law"
    exit 7
  fi
  echo "mutation_court: callsite-revert leg: court FLIPPED under shim (exit $rc, expected)" >&2
  echo "mutation_court: shim output tail:" >&2
  printf '%s\n' "$out" | tail -6 >&2

  # Restore + byte-exact verification.
  cp "$snap" "$law"
  now="$(shasum -a 256 "$law" | awk '{print $1}')"
  if [ "$sha_before" != "$now" ]; then
    echo "mutation_court: RESTORE FAILED for $law (sha256 mismatch: $sha_before != $now)" >&2
    echo "mutation_court: snapshot kept at $snap — fix src/law.rs by hand" >&2
    exit 4
  fi
  echo "mutation_court: callsite-revert restore byte-exact (sha256 $now verified)" >&2
  rm -rf "$snap_dir"

  # Post-restore sanity: the court passes on the unmutated tree.
  if ! cargo test --test law_state transition_with_unknown_capability_refuses_typed_end_to_end >/dev/null 2>&1; then
    echo "mutation_court: post-restore sanity FAILED: court does not pass on restored tree" >&2
    exit 4
  fi
}

if [ "${#LEGS[@]}" -gt 0 ]; then
  for leg in "${LEGS[@]}"; do
    case "$leg" in
      callsite-revert) leg_callsite_revert ;;
      *) echo "mutation_court: unknown leg: $leg" >&2; exit 2 ;;
    esac
  done
  echo "mutation_court: all requested legs passed" >&2
  exit 0
fi

# ---- tool presence (before any state is touched) ----------------------------
MUTANTS_BIN="${CARGO_MUTANTS:-cargo-mutants}"
command -v "$MUTANTS_BIN" >/dev/null 2>&1 || MUTANTS_BIN="$HOME/.cargo/bin/cargo-mutants"
[ -x "$(command -v "$MUTANTS_BIN" 2>/dev/null || echo "$MUTANTS_BIN")" ] || {
  echo "mutation_court: cargo-mutants not installed (cargo install cargo-mutants)" >&2; exit 2; }

# ---- dirty-tree refusal (typed: MUTATION-COURT-DIRTY-TREE) ------------------
# In-place mutation over uncommitted work is the false-red channel.
DIRTY="$(git status --porcelain -- src/ || true)"
if [ -n "$DIRTY" ] && [ "$FORCE" -ne 1 ]; then
  echo "mutation_court: REFUSED [MUTATION-COURT-DIRTY-TREE]: uncommitted work under src/;" >&2
  echo "  cargo-mutants runs --in-place and would mutate those edits (false-red channel)." >&2
  echo "  Dirty files:" >&2
  while IFS= read -r line; do printf '    %s\n' "$line" >&2; done <<<"$DIRTY"
  echo "  Commit, stash-elsewhere, or pass --force to run anyway." >&2
  exit 6
fi

export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-target-mutants}"
OUTPUT_DIR="${MUTANTS_OUTPUT:-$ROOT}"

# LOAD-BEARING: GUARDED must cover every file matched by examine_globs in
# .cargo/mutants.toml. cargo-mutants runs --in-place, so a widened inventory
# behind a narrower restore set LEAKS MUTATED FILES on SIGKILL (the EXIT trap
# is skipped; mc4 precedent). Any glob added to examine_globs gets a
# corresponding concrete entry here.
GUARDED=(src/registry.rs src/registry/definitions.rs src/registry/emit_json.rs src/registry/emit_ttl.rs src/abi.rs)
SNAP="$(mktemp -d "${TMPDIR:-/tmp}/mutation-court.XXXXXX")"
STATUS_BEFORE="$SNAP/status.before"
git status --porcelain -- src/ >"$STATUS_BEFORE" 2>/dev/null || : >"$STATUS_BEFORE"

# sha256 of each guarded file at baseline — restore is verified against these.
SHA_BEFORE="$SNAP/sha.before"
: >"$SHA_BEFORE"
for f in "${GUARDED[@]}"; do
  if [ -f "$f" ]; then
    # Guarded files may be nested (src/registry/*.rs): create the file's own
    # directory inside the snapshot, not a hardcoded SNAP/src.
    mkdir -p "$SNAP/$(dirname "$f")"
    cp -p "$f" "$SNAP/$f"
    printf '%s  %s\n' "$(shasum -a 256 "$f" | awk '{print $1}')" "$f" >>"$SHA_BEFORE"
  fi
done

MPID=
WPID=
restore() {
  local rc=$?
  trap - EXIT INT TERM
  # Stop cargo-mutants and the watchdog first: an in-place run that outlives this trap
  # would keep mutating the tree after the sources were restored.
  kill -TERM ${MPID:+"$MPID"} ${WPID:+"$WPID"} 2>/dev/null || true
  if [ -n "$MPID" ]; then wait "$MPID" 2>/dev/null || true; fi
  local restored=0 sha_fail=0 f now
  for f in "${GUARDED[@]}"; do
    if [ -f "$SNAP/$f" ] && ! cmp -s "$SNAP/$f" "$f"; then
      cp -p "$SNAP/$f" "$f"; restored=1
      echo "mutation_court: restored mutated $f" >&2
    fi
  done
  # Byte-exact restore verification: sha256 after must equal sha256 before.
  : >"$SNAP/sha.after"
  for f in "${GUARDED[@]}"; do
    [ -f "$f" ] && printf '%s  %s\n' "$(shasum -a 256 "$f" | awk '{print $1}')" "$f" >>"$SNAP/sha.after"
  done
  if ! cmp -s "$SHA_BEFORE" "$SNAP/sha.after"; then
    echo "mutation_court: RESTORE FAILED (sha256 mismatch)" >&2
    diff "$SHA_BEFORE" "$SNAP/sha.after" >&2 || true
    echo "mutation_court: snapshot kept at $SNAP — fix src/ by hand, do NOT trust this tree" >&2
    sha_fail=1
  fi
  local after="$SNAP/status.after"
  # Drift check is scoped to src/ — the court's own trust boundary. Untracked
  # receipts/transition-*.json churn from the session receipt hook during the
  # court's in-place mutations is not tree drift (witnessed 2026-10-23: a
  # repo-wide check forced exit 4 on every run while src/ stayed byte-exact).
  git status --porcelain -- src/ >"$after" 2>/dev/null || : >"$after"
  if ! cmp -s "$STATUS_BEFORE" "$after"; then
    echo "mutation_court: WARNING git status (src/) changed during the court:" >&2
    diff "$STATUS_BEFORE" "$after" >&2 || true
    [ "$rc" -eq 0 ] && rc=4
  elif [ "$restored" -eq 0 ] && [ "$sha_fail" -eq 0 ]; then
    echo "mutation_court: tree restored byte-exact (sha256 verified, git status unchanged)" >&2
  fi
  [ "$sha_fail" -eq 1 ] && [ "$rc" -eq 0 ] && rc=4
  if [ "$sha_fail" -eq 0 ]; then rm -rf "$SNAP"; fi
  exit "$rc"
}
trap restore EXIT INT TERM

# ---- optional output tee (--log) ---------------------------------------------
# shellcheck disable=SC2206
EXTRA=(${MUTANTS_EXTRA:-})
for c in "${CASES[@]+"${CASES[@]}"}"; do
  EXTRA+=(-F "$c")
done

# All court output goes through tee when --log is set: per-case progress lines
# from cargo-mutants land in the file as they are emitted, so a kill leaves a
# resumable record (pair with --case to resume the remaining mutants).
if [ -n "$LOG_FILE" ]; then
  : >"$LOG_FILE"
  exec > >(tee -a "$LOG_FILE") 2>&1
  echo "mutation_court: log file: $LOG_FILE"
fi

# run/check modes: the callsite-revert leg runs before cargo-mutants starts
# (restore is byte-verified first, so no in-place collision).
if [ "$MODE" != list ]; then
  leg_callsite_revert
fi

if [ "$MODE" = list ]; then
  "$MUTANTS_BIN" mutants --list --no-shuffle ${EXTRA[@]+"${EXTRA[@]}"}
  exit $?
fi

# A previous run killed mid-mutant (OOM/SIGKILL skips the trap) leaves edits behind.
MARK="changed by ""cargo-mutants"
if grep -rn --include='*.rs' "$MARK" src >&2; then
  echo "mutation_court: leaked mutant edits in src/ (above); fix them before running" >&2
  exit 5
fi

# A mutant can loop while allocating; nothing else bounds memory (macOS ignores
# ulimit -v). Watchdog: when any test process from this target
# dir exceeds MUTANTS_MAX_RSS_MB and KILL it: cargo-mutants records the mutant as failed
# and continues, and the EXIT trap above still restores the sources.
MAX_RSS_KB=$(( ${MUTANTS_MAX_RSS_MB:-4096} * 1024 ))
set +e
"$MUTANTS_BIN" mutants --in-place --no-shuffle --output "$OUTPUT_DIR" ${EXTRA[@]+"${EXTRA[@]}"} &
MPID=$!
(
  while kill -0 "$MPID" 2>/dev/null; do
    # argv0 of a test binary from this target dir only (not rustc/ld lines that merely mention it)
    OFFENDER=$(ps -axo pid=,rss=,command= | awk -v max="$MAX_RSS_KB" -v dir="$CARGO_TARGET_DIR/debug/deps/" \
      'index($3, dir) && $2 > max { print $1; exit }')
    if [ -n "$OFFENDER" ]; then
      echo "mutation_court: RSS cap exceeded by pid $OFFENDER; killing it" >&2
      kill -KILL "$OFFENDER" 2>/dev/null
      sleep 5
    fi
    sleep 2
  done
) &
WPID=$!
wait "$MPID"
RC=$?
kill "$WPID" 2>/dev/null
set -e

OUT="$OUTPUT_DIR/mutants.out"
echo "mutation_court: summary path: $OUT" >&2
# cargo-mutants exit: 0 all caught, 2 some missed, 3 timeouts, 4 baseline failed.
case "$RC" in 0|2|3) ;; *) echo "mutation_court: cargo-mutants failed (exit $RC)" >&2; exit "$RC" ;; esac
[ "$MODE" = run ] && exit "$RC"

MISSED="$OUT/missed.txt"
[ -f "$MISSED" ] || { echo "mutation_court: no missed.txt at $MISSED" >&2; exit 2; }
# Gated surface: everything in registry.rs, and mutants whose description names
# capabilities() or dispatch() in abi.rs.
GATED="$(grep -E '(^|/)src/registry\.rs:|(^|/)src/registry/[^:]*\.rs:|^src/abi\.rs:.*(\bcapabilities\b|\bdispatch\b)' "$MISSED" || true)"
if [ -n "$GATED" ]; then
  echo "mutation_court: MISSED gated mutants:" >&2
  printf '%s\n' "$GATED" >&2
  exit 3
fi
echo "mutation_court: no MISSED mutants in src/registry.rs, src/registry/*.rs, or capabilities()/dispatch()" >&2
