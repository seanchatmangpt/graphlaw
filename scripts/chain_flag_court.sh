#!/usr/bin/env bash
# chain_flag_court.sh — non-vacuity court for scripts/receipt_chain_sync.sh (lane M14).
#
# Asserts on the LIVE tree (no copies, real collaborators):
#   1. --check verifies the real chain: exit 0 + VERIFIED line.
#   2. Any unknown first argument (--force, -f, garbage) hits the usage
#      path: exit 2 + USAGE line. (Script behavior verified 2026-10-22;
#      this court pins it so a future flag regression fails loudly.)
#
# A court that cannot flip is FAIL: if both checks pass trivially the court
# would also pass against a script that accepted every flag — that is why
# check 2 asserts the exact refusal, not merely a nonzero exit.
set -u

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
SYNC="$SCRIPT_DIR/receipt_chain_sync.sh"
FAIL=0

# 1. --check must verify on the live chain.
OUT="$(bash "$SYNC" --check 2>&1)"; RC=$?
if [ "$RC" -eq 0 ] && printf '%s' "$OUT" | grep -q '^VERIFIED:'; then
  echo "PASS: --check exit 0: $OUT"
else
  echo "FAIL: --check rc=$RC out=$OUT"; FAIL=1
fi

# 2. Unknown flags must be refused with exit 2 + USAGE.
for FLAG in --force -f --dry-run garbage; do
  OUT="$(bash "$SYNC" "$FLAG" 2>&1)"; RC=$?
  if [ "$RC" -eq 2 ] && printf '%s' "$OUT" | grep -q '^USAGE:'; then
    echo "PASS: unknown flag '$FLAG' refused: rc=2 USAGE"
  else
    echo "FAIL: unknown flag '$FLAG' rc=$RC out=$OUT"; FAIL=1
  fi
done

if [ "$FAIL" -ne 0 ]; then
  echo "COURT-FAIL: receipt_chain_sync.sh flag surface drifted"; exit 1
fi
echo "COURT-PASS: flag surface pinned (--check ok, unknown flags exit 2)"
