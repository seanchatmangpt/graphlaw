#!/usr/bin/env bash
# Bounded wrapper for cargo-mutants: wall-clock timeout, memory cap, watchdog,
# and cleanup of leaked mutant processes and work dirs on exit.
#
# Usage: scripts/mutants.sh [cargo-mutants args...]
# Env:   MUTANTS_TIMEOUT   overall wall-clock bound (integer seconds, default 3600)
#        MUTANTS_MEM_KB    virtual-memory cap in KiB (default 8388608 = 8 GiB)
#        MUTANTS_JOBS      parallel mutants (default 2)
set -u

cd "$(dirname "$0")/.." || exit 2
ROOT=$(pwd)

BOUND=${MUTANTS_TIMEOUT:-3600}
MEM_KB=${MUTANTS_MEM_KB:-8388608}
JOBS=${MUTANTS_JOBS:-2}
case $BOUND in
    "" | *[!0-9]*) echo "mutants.sh: MUTANTS_TIMEOUT must be integer seconds" >&2; exit 2 ;;
esac
TMPBASE=${TMPDIR:-/tmp}

if ! cargo mutants --version >/dev/null 2>&1; then
    echo "mutants.sh: cargo-mutants not installed (cargo install cargo-mutants)" >&2
    exit 2
fi

# Snapshot pre-existing mutants work dirs so we only remove ones we leaked.
BEFORE=$(mktemp)
ls -d "$TMPBASE"/cargo-mutants-* 2>/dev/null | sort >"$BEFORE" || true

CHILD=0
WATCHDOG=0

cleanup() {
    trap - EXIT INT TERM HUP
    if [ "$WATCHDOG" -gt 0 ]; then
        pkill -P "$WATCHDOG" 2>/dev/null
        kill "$WATCHDOG" 2>/dev/null
    fi
    if [ "$CHILD" -gt 0 ]; then
        # Kill the whole process group we started (cargo mutants, cargo, rustc, tests).
        kill -TERM -- "-$CHILD" 2>/dev/null
        sleep 2
        kill -KILL -- "-$CHILD" 2>/dev/null
    fi
    # Leaked test binaries from mutant work dirs.
    pkill -KILL -f "$TMPBASE/cargo-mutants-" 2>/dev/null
    # Remove only work dirs created during this run.
    ls -d "$TMPBASE"/cargo-mutants-* 2>/dev/null | sort | comm -13 "$BEFORE" - |
        while IFS= read -r d; do rm -rf -- "$d"; done
    rm -f -- "$BEFORE"
}
trap cleanup EXIT
trap 'exit 130' INT TERM HUP

# Memory cap: prefer a systemd scope (covers children, cgroup-enforced), else ulimit -v.
RUNNER=()
if command -v systemd-run >/dev/null 2>&1 &&
    systemd-run --user --scope --quiet true >/dev/null 2>&1; then
    RUNNER=(systemd-run --user --scope --quiet -p "MemoryMax=${MEM_KB}K" -p MemorySwapMax=0)
else
    ulimit -v "$MEM_KB" 2>/dev/null ||
        echo "mutants.sh: warning: could not set memory cap" >&2
fi

# Run in its own process group (setsid) so the trap can kill everything.
# timeout sends TERM at BOUND, then KILL 30s later.
setsid timeout --kill-after=30 "$BOUND" \
    ${RUNNER[@]+"${RUNNER[@]}"} cargo mutants -j "$JOBS" "$@" &
CHILD=$!

# Watchdog: independent hard stop slightly past the bound in case timeout is wedged.
(
    sleep $((BOUND + 120))
    kill -KILL -- "-$CHILD" 2>/dev/null
) &
WATCHDOG=$!

wait "$CHILD"
rc=$?
CHILD_DONE=$rc
[ "$rc" -eq 124 ] && echo "mutants.sh: hit wall-clock bound (${BOUND}s)" >&2
cd "$ROOT" || true
exit "$CHILD_DONE"
