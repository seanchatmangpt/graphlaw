# Count Gate Receipt — Lane M03 (GUARD BUILDER), 2026-10-22

Subject: `scripts/count_gate.sh` (new, lane lease) — stale "N passed; 0 failed"
detector for the cheap-allowlist courts.

## Defect class

Receipt rows claim "N passed; 0 failed" for a named court; courts grow (new
tests land) and the claims rot: witnessed 33/0 claimed vs 34 actual
(sql_rdf_dialect), 7/0 vs 8/0 (query_cache_court), and mid-session growth
10→12 (trust_attest), 9→11 (chicago_validation) while this lane ran.

## Gate design

- Scans `docs/**/*.md`; pairs each `cargo test --test <court>` command line
  naming exactly one allowlist court with a count on the same line or within
  the next 2 lines. Multi-court commands and aggregates are skipped (count
  not attributable to one court).
- Cheap-allowlist: sql_rdf_dialect, chicago_validation, trust_attest,
  duckdb_statement_kinds, query_cache_court, query_ledger_court,
  evidentiary_ring_court (`--features evidentiary-ring`).
- Runs each referenced court once in a lane target dir
  (`CARGO_TARGET_DIR=target-countgate`), compares claimed vs actual.
- Outputs: COUNT-OK / COUNT-STALE (file:line, claimed, actual) /
  COUNT-UNKNOWN (court not runnable — concurrent lane edit; never silent).
  Exit 0 = no stale rows; exit 1 = stale rows exist.

## Verification run (real output, 2026-10-22, /Users/sac/graphlaw)

```
$ time bash scripts/count_gate.sh > /tmp/cg.out 2>/tmp/cg.err; echo "exit=$?"
exit=1
stale=29 ok=9 unk=3
```

Court measurements embedded in the run (from stderr):

```
COUNT-GATE ran sql_rdf_dialect -> 34 passed
COUNT-GATE ran evidentiary_ring_court -> 10 passed
COUNT-GATE ran chicago_validation -> 11 passed
COUNT-GATE ran duckdb_statement_kinds -> 6 passed
COUNT-GATE ran trust_attest -> 12 passed
COUNT-GATE ran query_cache_court -> UNKNOWN (COUNT-GATE COURT-BUILD-FAIL: query_cache_court — concurrent lane edit mid-gate)
COUNT-GATE ran query_ledger_court -> 9 passed
```

Timing: cold run 4:15 (builds 7 court binaries); warm run 51.9s wall —
within the <3min budget.

## Non-vacuity

1. Gate CAN exit 1: witnessed `exit=1` with 29 COUNT-STALE rows.
2. It flags the exact defect-class rows:
   - `docs/jira/v26.10.22/RECEIPTS.md:823` — sql_rdf_dialect claimed 33,
     actual 34 (the stale 33/0 row).
   - `docs/jira/v26.10.22/designs/eyeron-switchover-fixforward.md:44` —
     33 claimed vs 34 actual.
   - `docs/jira/v26.10.22/DFLSS-DOSSIER.md:81` — query_cache_court claimed 7,
     actual 8 (witnessed in an earlier same-session run:
     `COUNT-STALE docs/jira/v26.10.22/DFLSS-DOSSIER.md:81 court=query_cache_court claimed=7 actual=8`).
3. It passes matching rows (9 COUNT-OK witnessed), so it is not
   flag-everything: e.g. `COUNT-OK docs/jira/v26.10.22/RECEIPTS.md:164
   court=query_ledger_court 9/0`.

## Sample of the 29 COUNT-STALE findings (claimed vs actual)

| row | claimed | actual |
|---|---|---|
| docs/jira/v26.10.22/RECEIPTS.md:823 | 33 | 34 |
| docs/jira/v26.10.22/designs/eyeron-switchover-fixforward.md:44 | 33 | 34 |
| docs/jira/v26.10.22/designs/m-wave-verdicts.md:23 | 10 | 12 (trust_attest) |
| docs/jira/v26.10.22/RECEIPTS.md:825 | 9 | 11 (chicago_validation) |
| docs/jira/v26.10.22/designs/cold-path-ledger.md:84 | 7 | 10 (evidentiary_ring_court) |

## Known limitations (typed)

- A row whose count appears in a dated historical receipt block (e.g.
  RECEIPTS.md:823 embedded lane output) is flagged even though annotated
  as-of-date nearby; the gate is a detector, annotation triage is human/coordinator.
- A court that fails to build mid-gate (another lane's in-flight edit) yields
  COUNT-UNKNOWN rows, not a silent pass — witnessed on query_cache_court.
- `merge-shapes-receipt.md:71` is flagged claimed=12 for a 3-court aggregate
  command whose count sits 2 lines below the command line, attributed to the
  nearest preceding single-court line; a known attribution edge case.

## Environment receipts

- df at run start: `/dev/disk3s1s1 926Gi 11Gi 58Gi 17% /` (later dipped to
  15.8–16.4 GiB: two Bash invocations were DISK-REFUSED by the fleet hook;
  disk recovered to 33 GiB before the final full run).
- Lane build root `target-countgate/` (5.3 GiB) left as lane lease for
  coordinator deletion at integration.
