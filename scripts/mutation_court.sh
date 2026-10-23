#!/usr/bin/env bash
# Native mutation court over the ABI registry surface (src/registry.rs, src/abi.rs).
#
#   scripts/mutation_court.sh            run the court, print the summary
#   scripts/mutation_court.sh --list     list mutants only (no test run)
#   scripts/mutation_court.sh --check    run the court; exit 3 when any mutant in
#                                        src/registry.rs or in the capabilities()/
#                                        dispatch() bodies of src/abi.rs is MISSED
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
while [ $# -gt 0 ]; do
  case "$1" in
    "") shift ;;
    --list) MODE=list; shift ;;
    --check) MODE=check; shift ;;
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

GUARDED=(src/registry.rs src/abi.rs)
SNAP="$(mktemp -d "${TMPDIR:-/tmp}/mutation-court.XXXXXX")"
STATUS_BEFORE="$SNAP/status.before"
git status --porcelain >"$STATUS_BEFORE" 2>/dev/null || : >"$STATUS_BEFORE"

# sha256 of each guarded file at baseline — restore is verified against these.
SHA_BEFORE="$SNAP/sha.before"
: >"$SHA_BEFORE"
for f in "${GUARDED[@]}"; do
  if [ -f "$f" ]; then
    mkdir -p "$SNAP/src"
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
  git status --porcelain >"$after" 2>/dev/null || : >"$after"
  if ! cmp -s "$STATUS_BEFORE" "$after"; then
    echo "mutation_court: WARNING git status changed during the court:" >&2
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
