#!/usr/bin/env bash
# Native mutation court over the ABI registry surface (src/registry.rs, src/abi.rs).
#
#   scripts/mutation_court.sh            run the court, print the summary
#   scripts/mutation_court.sh --list     list mutants only (no test run)
#   scripts/mutation_court.sh --check    run the court; exit 3 when any mutant in
#                                        src/registry.rs or in the capabilities()/
#                                        dispatch() bodies of src/abi.rs is MISSED
#
# Runs `cargo mutants --in-place` (no copies, no worktrees). The examined sources
# are snapshotted first and restored by an EXIT trap if anything left them mutated.
# Only read-only git (`git status --porcelain`) is used.
#
# Env: CARGO_TARGET_DIR (default target-mutants), MUTANTS_OUTPUT (default: repo root,
# which yields ./mutants.out), MUTANTS_EXTRA (extra cargo-mutants args).
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

MODE=run
case "${1:-}" in
  "") MODE=run ;;
  --list) MODE=list ;;
  --check) MODE=check ;;
  -h|--help) sed -n '2,15p' "$0"; exit 0 ;;
  *) echo "mutation_court: unknown argument: $1 (use --list | --check)" >&2; exit 2 ;;
esac

export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-target-mutants}"
OUTPUT_DIR="${MUTANTS_OUTPUT:-$ROOT}"
MUTANTS_BIN="${CARGO_MUTANTS:-cargo-mutants}"
command -v "$MUTANTS_BIN" >/dev/null 2>&1 || MUTANTS_BIN="$HOME/.cargo/bin/cargo-mutants"
[ -x "$(command -v "$MUTANTS_BIN" 2>/dev/null || echo "$MUTANTS_BIN")" ] || {
  echo "mutation_court: cargo-mutants not installed (cargo install cargo-mutants)" >&2; exit 2; }

GUARDED=(src/registry.rs src/abi.rs)
SNAP="$(mktemp -d "${TMPDIR:-/tmp}/mutation-court.XXXXXX")"
STATUS_BEFORE="$SNAP/status.before"
git status --porcelain >"$STATUS_BEFORE" 2>/dev/null || : >"$STATUS_BEFORE"

for f in "${GUARDED[@]}"; do
  if [ -f "$f" ]; then mkdir -p "$SNAP/src"; cp -p "$f" "$SNAP/$f"; fi
done

MPID=
WPID=
restore() {
  local rc=$?
  trap - EXIT INT TERM
  # Stop cargo-mutants and the watchdog first: an in-place run that outlives this trap
  # would keep mutating the tree after the sources were restored.
  kill -TERM ${MPID:+"$MPID"} ${WPID:+"$WPID"} 2>/dev/null || true
  [ -n "$MPID" ] && wait "$MPID" 2>/dev/null || true
  local restored=0
  for f in "${GUARDED[@]}"; do
    if [ -f "$SNAP/$f" ] && ! cmp -s "$SNAP/$f" "$f"; then
      cp -p "$SNAP/$f" "$f"; restored=1
      echo "mutation_court: restored mutated $f" >&2
    fi
  done
  local after="$SNAP/status.after"
  git status --porcelain >"$after" 2>/dev/null || : >"$after"
  if ! cmp -s "$STATUS_BEFORE" "$after"; then
    echo "mutation_court: WARNING git status changed during the court:" >&2
    diff "$STATUS_BEFORE" "$after" >&2 || true
    [ "$rc" -eq 0 ] && rc=4
  elif [ "$restored" -eq 0 ]; then
    echo "mutation_court: tree clean (git status --porcelain unchanged)" >&2
  fi
  rm -rf "$SNAP"
  exit "$rc"
}
trap restore EXIT INT TERM

# shellcheck disable=SC2206
EXTRA=(${MUTANTS_EXTRA:-})

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
GATED="$(grep -E '(^|/)src/registry\.rs:|^src/abi\.rs:.*(\bcapabilities\b|\bdispatch\b)' "$MISSED" || true)"
if [ -n "$GATED" ]; then
  echo "mutation_court: MISSED gated mutants:" >&2
  printf '%s\n' "$GATED" >&2
  exit 3
fi
echo "mutation_court: no MISSED mutants in src/registry.rs or capabilities()/dispatch()" >&2
