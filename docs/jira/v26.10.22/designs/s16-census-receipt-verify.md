# S16 — skeptic census of L110's sweep-post-link receipt (v26.10.22)

Lane S16, read-only, /Users/sac/graphlaw, 2026-10-22. Every verdict re-derived from
logs/disk, not from the receipt's own claims.

## Verdicts

| check | verdict | evidence |
|---|---|---|
| 1. Single-cause classification (exceptions gate at tests/common/mod.rs:115) | **CONFIRMED (live defect)** | `Engine::default()` at tests/common/mod.rs:114, `Module::new` at :115; `grep -rn wasm_exceptions tests/ src/ wasm/src` → 0 matches — NOT yet fixed; git diff vs HEAD touches only wasm_path()/CARGO_TARGET_DIR, not the engine config, so the receipt's "before" state is the current state |
| 2a. sweep-1 counts 634p/107f | **CONFIRMED** | `grep -E "^test result:" /tmp/sweep3.log` → passed: 634, failed: 107 (99 result lines; 98 binaries +1 anomalous — see note) |
| 2b. sweep-2 counts 128p/104f | **CONFIRMED** | /tmp/sweep3b.log → 128p/104f, 17 result lines; per-target p/f and per-target exceptions counts (3/0/1/24/1/3/3/7/3/2 = 47) all match the receipt's tables exactly |
| 2c. classification table | **CONFIRMED with one arithmetic nit** | 47+50(build-failed)+4(registry)+1(skip)+1(size court)+... internal inconsistency: table says "46 REAL (8 targets)" + "remainder 3" but 44 (8 targets) + 3 (cc_a) = 47 — the 46 is a typo-class error; the per-target table and grand totals are exact |
| 3. fibo/op_differential prediction | **CONSISTENT but not yet green** | sweep3c: 0 BLOCKED-LINK, 30 failures, all 30 match "exceptions proposal not enabled" at offset 80781, chicago_fibo 57p/4f, op_differential 0p/16f — EH was indeed the sole *sweep-2* cause (0 exceptions mentions for those targets in sweep3b); they flip green only after the exceptions gate fix, exactly as the receipt's standing section says |
| 4. stale-lane-root purge list | **CONFIRMED, and URGENT** | target-lane167 9.4G, target-lane122 8.5G, target-lane-s5 8.0G — sizes match receipt; 33 roots matching target-lane*/target-a*/target-gate*/target-fulltest total **~118 GiB**; disk now **5.0 GiB free** (`df -h /System/Volumes/Data`) — below the 20 GiB DISK-REFUSED gate; the receipt's "recovered to 37 GiB" is stale |

## Notes

- sweep3.log has 99 `test result:` lines for "98 test binaries" in the receipt — one
  extra result line (doctest or re-run), does not change totals.
- Loud-skip receipted: `SKIP [evidentiary-ring-courts]` appears once in sweep3.log and
  once in sweep3b.log — loud, typed, correctly counted FAILED by the harness.
- Purge is the highest-yield open action: ~118 GiB reclaimable, disk at 5 GiB is one
  mid-session compile away from a DISK-REFUSED abort.

## Standing

Receipt verified PARTIAL_ALIVE: counts and single-cause classification are exact;
one arithmetic nit (46 vs 44+3); disk claim is stale and the situation is worse
than receipted.
