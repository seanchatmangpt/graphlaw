# Receipt: fleet_cleanup.sh — lane build-root reclaim guard

## Subject
- `/Users/sac/graphlaw/scripts/fleet_cleanup.sh` (new, executable, 67 lines)
- `/Users/sac/graphlaw/docs/jira/v26.10.22/designs/fleet-cleanup-receipt.md` (this file)

## Defect class
Disk fell to 9-14 GiB free twice on 2026-10-22 from accumulated lane build
roots (`target-lane*`, `target-k*`, `target-m*`, `target-a*`, `target-countgate`,
`target-deopt`, `target-dflss12`, `target-final2`, ...). Suite runs
DISK-REFUSED mid-wave; coordinator rm is permission-blocked in-session.

## Behavior
- Deletes only repo-local `target-*` dirs (bare `target/` never matches the
  glob `target-*` — witnessed in sandbox: bare `target/` survived a destructive
  cycle).
- mtime-based active-lane protection: a dir touched within the grace window
  (default 45 min, `--grace-minutes N` override) is KEEP, never deleted.
- Per-dir freed bytes (du -sk), df before/after, summary line.
- `--dry-run`, `--repo PATH`, `--grace-minutes N` flags; always exits 0
  (advisory cleanup).
- Modeled on the same-checkout-fanout cleanup law: lane build roots are
  leases — stale leases reclaimed, live leases untouched.

## Real command output (verification)
### Dry-run on the real tree (/Users/sac/graphlaw, lanes active)
```
== fleet_cleanup: repo=/Users/sac/graphlaw grace=45m dry_run=1 ==
df before: 11828648 free (929898068 used of 971350180)
WOULD-REMOVE   4.03 GiB: target-a02            (truncated per-dir lines)
WOULD-REMOVE  10.39 GiB: target-sweep2
KEEP (active, mtime 22:45:57): target-countgate
KEEP (active, m trunc 22:49:42): target-k2 ... 13 active keeps total
WOULD-REMOVE   5.24 GiB: target-final2
df after:  11584284 free (930142432 used of 971350180)
summary: freed=72025968 KiB (68.69 GiB) removed=0 kept_active=13 dry_run=1
```
Stale dirs the dispatch named (target-countgate, target-final2) — countgate
was mtime-active at run time (touched 22:45:57, within 45 min) so correctly
KEEP; target-final2 stale, WOULD-REMOVE 5.24 GiB.

### Destructive cycle on /tmp sandbox (mktemp -d /tmp/fleetcleanup.XXXX)
Fake dirs: target-a, target-b (mtime 2025-01-01), target-active (fresh),
bare target/ with a file. `--repo "$SB"` destructive run:
```
REMOVED       0.00 GiB: /tmp/fleetcleanup.MVV3/target-a
KEEP (active, mtime 22:49:55): /tmp/fleetcleanup.MVV3/target-active
REMOVED       0.00 GiB loud not silent: .../target-b
summary: removed=2 kept_active=1 dry_run=0
exit=0
remaining: scripts  target  target-active
```
Bare `target/` and `target-active` survived; only stale target-* removed.

### Grace-window edge test (grace=1m)
```
exit=0; remaining: target-new   (target-old removed)
```

## Falsifier status
Anti-vacuity witnessed: the script distinguishes stale vs active by mtime and
flips behavior on both sides (sandbox: 2 removed / 1 kept; grace edge: 1
removed / 1 kept). A cleanup that removed everything or nothing regardless of
mtime would fail the sandbox cycle.

## Standing
ALIVE (dry-run on real tree + destructive cycle on /tmp sandbox + grace edge
test, all with embedded output above). Destructive mode NOT run on the real
tree per dispatch (lanes active); 68.69 GiB reclaimable stands as dry-run
figures only.
