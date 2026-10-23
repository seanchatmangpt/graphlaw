#!/usr/bin/env bash
# fleet_cleanup.sh — advisory disk reclaim for lane build roots.
# Deletes repo-local target-* dirs (never ./target itself) whose mtime is
# older than a grace window (default 45 min), so active lanes are protected.
# Prints per-dir freed bytes and df before/after. Always exits 0.
# Usage: fleet_cleanup.sh [--dry-run] [--grace-minutes N] [--repo PATH]
set -u
DRY_RUN=0
GRACE_MIN=45
REPO="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
while [ $# -gt 0 ]; do
  case "$1" in
    --dry-run) DRY_RUN=1 ;;
    --grace-minutes) GRACE_MIN="$2"; shift ;;
    --repo) REPO="$2"; shift ;;
    *) printf 'unknown arg: %s\n' "$1" >&2 ;;
  esac
  shift
done

now=$(date +%s)
cutoff=$(( now - GRACE_MIN * 60 ))
df_line() { df -k "$REPO" | tail -1 | awk '{printf "%s free (%s used of %s)\n", $4, $3, $2}'; }

echo "== fleet_cleanup: repo=$REPO grace=${GRACE_MIN}m dry_run=$DRY_RUN =="
echo "df before: $(df_line)"

freed_total=0
removed=0
kept_active=0
for d in "$REPO"/target-*; do
  [ -d "$d" ] || continue
  mtime=$(stat -f %m "$d" 2>/dev/null) || continue
  if [ "$mtime" -gt "$cutoff" ]; then
    kept_active=$(( kept_active + 1 ))
    printf 'KEEP (active, mtime %s): %s\n' "$(date -r "$mtime" '+%H:%M:%S')" "$d"
    continue
  fi
  kb=$(du -sk "$d" 2>/dev/null | awk '{print $1}')
  kb=${kb:-0}
  if [ "$DRY_RUN" -eq 1 ]; then
    printf 'WOULD-REMOVE %6.2f GiB: %s\n' "$(echo "scale=4; $kb/1048576" | bc)" "$d"
    freed_total=$(( freed_total + kb ))
  else
    if rm -rf -- "$d"; then
      printf 'REMOVED     %6.2f GiB: %s\n' "$(echo "scale=4; $kb/1048576" | bc)" "$d"
      freed_total=$(( freed_total + kb ))
      removed=$(( removed + 1 ))
    else
      printf 'REMOVE-FAILED (left in place): %s\n' "$d"
    fi
  fi
done

echo "df after:  $(df_line)"
printf 'summary: freed=%d KiB (%.2f GiB) removed=%d kept_active=%d dry_run=%d\n' \
  "$freed_total" "$(echo "scale=4; $freed_total/1048576" | bc)" "$removed" "$kept_active" "$DRY_RUN"
exit 0
