# S7 vibration lane — chain + board witness (2026-10-22, post-L167)

Lane S7, read-only verification. Repo: /Users/sac/graphlaw @ 615dbfa (dirty
working tree, coordinator owns git). Every state re-read from disk at
~20:05 local; nothing recalled from session memory.

## 1. Receipt chain

```
$ bash scripts/receipt_chain_sync.sh --check
VERIFIED: 22 receipts match CHAIN.blake3; head=5931264fb2f93ede6f12c0f5e446da0ff27b266c896851839dd8f54d7705726f # historical
EXIT=0
```

Matches the dispatch expectation: 22/22 VERIFIED, exit 0. Head advanced from
N14's `2c316b02…d05` (8 receipts, chain-resync-receipt.md) — the chain grew
by 14 receipts without breaking the fold.

## 2. GO/NO-GO board (`designs/integration-gonogo.md`, mtime 19:48)

Read from disk; the board is the L152 state. Counts:

- READY: **17 items** (rows 1–5, 1'–2', 6–16), each carrying an on-disk
  receipt pointer. The board's row 1 now cites the 23/23 laundering
  correction (laundering-count-reconcile.md), superseding the stale 11/11.
- PENDING: **7 items open** (P1, P2, P4, P7, P8, P9 — P5/P6 CLOSED). The
  board's own heading says "7 items open; P5/P6 CLOSED by L149".
- FAILED: **0** rows in a FAILED state; the only failure-class row is P2
  (BLOCKED — wasmi EH wall, typed blocker in wasm-repin-receipt.md).
  Row 14 (precommit acceptance) is "READY modulo the 4 registry-drift rows"
  (113 passed / 4 failed, all one deterministic stale-pin defect).

## 3. OPERATOR-DECISIONS.md

Exists at `designs/OPERATOR-DECISIONS.md`, mtime 19:4x, header
"consolidated sign-off surface (L153)". Sections OD-1 through OD-6 present
(`grep -c '^## '` = 8 total H2s = 6 decisions + Summary table + See Also).
The six: OD-1 EH route for the wasm artifact; OD-2 dual-artifact sub-decisions
D1–D6; OD-3 registry re-pin timing; OD-4 Cpk/charter CTQ production bar;
OD-5 absolute paths in wasm/ggen.toml; OD-6 commit .claude/ as repo law.
Confirmed: 6 decisions, register marked open (nothing decided by the lane).

## 4. Cron-loop effect: designs vs INDEX gaps

- `designs/INDEX.md` covers 21 entries and ends at `cpk-correction.md` —
  it has NOT been updated for the L150+ batch; the live coverage surface is
  the window-level `INDEX.md` "Session design records — post-L165" table
  (L170), which indexes 12 post-L165 records including
  OPERATOR-DECISIONS.md, staleness-76-triage.md, g2-repin-prep.md,
  spc-red-remediation-design.md, wasmi-eh-research.md.
- **Gap found: three files landed AFTER L170's INDEX batch with zero index
  coverage anywhere** (grep across window INDEX.md, designs/INDEX.md, both
  RECEIPTS.md files → exit 1, no matches):
  - `designs/eyeron-switchover-fixforward.md` (N25, mtime 19:54)
  - `designs/receipt-surface-post-switchover.md` (mtime 20:00)
  - `designs/switchover-provenance-census.md` (N26, mtime 20:01)
- design INDEX staleness class: 99 .md files in `designs/` vs 21 in
  designs/INDEX.md — many pre-L150 receipts (e.g. chain-resync-receipt.md,
  m4-wasm-boundary-receipt.md, wasm-duckdb-op-wired.md) are indexed only in
  the window INDEX / board, not designs/INDEX.md. This is the established
  pattern (designs/INDEX.md = spec-lane records only), not new drift.

## Verdict

Chain VERIFIED 22/22 exit 0. Board: 17 READY / 7 PENDING open (P5, P6
CLOSED) / 0 FAILED (P2 typed BLOCKED). OPERATOR-DECISIONS.md present with
exactly OD-1..OD-6. One concrete gap: the three post-L170 files (N25/N26
switchover class) are unindexed anywhere — needs a follow-up INDEX batch.
