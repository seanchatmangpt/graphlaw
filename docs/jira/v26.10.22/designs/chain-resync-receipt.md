# Chain Resync Receipt — L108 (2026-10-22)

## Lane

L108, canonical checkout `/Users/sac/graphlaw`. Ownership:
`receipts/CHAIN.blake3` (via `scripts/receipt_chain_sync.sh` only) and this file.

## Interface note (pinned 2026-10-22, lane M14)

`scripts/receipt_chain_sync.sh` has exactly two modes: `sync` (default) and
`--check`, plus an optional second argument naming an alternate repo root.
There is no `--force` mode; re-baseline is always the explicit coordinator
procedure `rm receipts/CHAIN.blake3 && bash scripts/receipt_chain_sync.sh sync`.
Any unknown first argument exits 2 (usage). Guard:
`scripts/chain_flag_court.sh`.

## Commands + real output

```
$ bash scripts/receipt_chain_sync.sh
TAMPER-DETECTED: receipts/spc-2026-10-22.json (chain=1da77e6b743577d627aa588f1e1264fea4ff9bcf86b8dd3f30ecca0ba4cbbc3a actual=50896fa3253cf162946b5c7eb6eb325fe909cda9305cdb3359619fa52ba11708)
REFUSED: chain drift above; receipts are immutable — no CHAIN.blake3 write (exit 1)
EXIT=1
```

```
$ bash scripts/receipt_chain_sync.sh --check
TAMPER-DETECTED: receipts/spc-2026-10-22.json (chain=1da77e6b...ba4cbbc3a actual=50896fa3...ba11708)
EXIT=1
```

## Census (6 receipt files, 1 flagged)

| file | embeds real output | flag |
|---|---|---|
| receipts/gws-10-e2e-coldchain-wasm.md | yes (12 evidence matches) | — |
| receipts/gws-16-wasm-duckdb-op15.md | yes (18) | — |
| receipts/gws-9-wasm-boundary-fuzz.md | yes (7) | — |
| receipts/sa2a/factory-b-v26.9.29-executable-gates.md | NO (0 matches) | NO-OUTPUT (known, persists) |
| receipts/spc-2026-10-22.json | yes (4: cpk/verdict counts) | — |
| receipts/spc-2026-10-22-inject.json | yes (4) | — |

## FINDING (drift, not silent overwrite)

`spc_court.sh` (lines 155/157) **overwrites in place**: `> receipts/spc-<date>.json`.
Every court re-run rewrites the day's SPC receipt with fresh timing numbers
(this drift: `cpk -0.0016 → 0.2948`, `p99 292 → 250`, max_ns `61145087 → 29759`), while
`CHAIN.blake3` treats receipts/ as immutable. Chain mtime 16:51, receipt mtime 16:55 —
the court re-ran after the chain was last synced. Two files are also `M` vs git HEAD
(`spc-2026-10-22.json`, `spc-2026-10-22-inject.json`), so the drift exists in the working
tree, not just vs chain.

**Structural defect**: an in-place-mutating receipt writer feeding an immutability
chain cannot ever re-sync without either (a) re-baselining the chain over a mutated
receipt (banned — no overwrite) or (b) fixing the writer. Fix belongs in
`scripts/spc_court.sh` (not this lane's file): make it write
`receipts/spc-<date>T<HHMMSS>.json` per-run timestamps, append-only. Until then,
the chain stays TAMPER-DETECTED and refuses to write — the gate works as designed;
the writer is the defect.

## Chain head

- before: `abb89cec5337dbc109f1a5159e81e395c216c7bba9cd35f462bbf780f96dcdc5` (unchanged — no write occurred)
- after: same (sync refused, exit 1, no CHAIN.blake3 write)

## Coordinator re-baseline decision (2026-10-22)

The drift on receipts/spc-2026-10-22.json was a HARNESS DEFECT, not
tampering: scripts/spc_court.sh overwrote the same-day receipt in place
(fixed by L111 — per-run timestamped append-only filenames). The current
bytes are the court's real output. Decision: re-baseline the chain over
the post-fix receipt set. Executed: rm CHAIN.blake3 && sync → 8 receipts
chained, head 2c316b0261c992cfb0463d135e4af4f6975daf9318368bfaf531098b57749d05;
--check VERIFIED exit 0. The overwritten file's pre-overwrite content
survives in git history (HEAD version, cpk -0.0016 numbers) — provenance
preserved there, not in the chain.

## Lane N14 re-sync (2026-10-22, later)

Ownership: `receipts/CHAIN.blake3` (via `scripts/receipt_chain_sync.sh` only).

```
$ bash scripts/receipt_chain_sync.sh
NO-OP: CHAIN.blake3 already current; head=2c316b0261c992cfb0463d135e4af4f6975daf9318368bfaf531098b57749d05
EXIT=0
$ bash scripts/receipt_chain_sync.sh --check
VERIFIED: 8 receipts match CHAIN.blake3; head=2c316b0261c992cfb0463d135e4af4f6975daf9318368bfaf531098b57749d05
EXIT=0
```

### Census (8 receipt files chained, 1 NO-OUTPUT flag)

| file | embeds real output | flag |
|---|---|---|
| receipts/gws-10-e2e-coldchain-wasm.md | yes | — |
| receipts/gws-16-wasm-duckdb-op15.md | yes | — |
| receipts/gws-9-wasm-boundary-fuzz.md | yes | — |
| receipts/sa2a/factory-b-v26.9.29-executable-gates.md | NO (0 matches) | NO-OUTPUT (known, persists) |
| receipts/spc-2026-10-22.json | yes | — |
| receipts/spc-2026-10-22-inject.json | yes | — |
| receipts/spc-2026-10-22T173758.json | yes (post-fix timestamped writer) | — |
| receipts/spc-2026-10-22T173809.json | yes (post-fix timestamped writer) | — |

### FINDING: dispatcher's "new receipts" are not under receipts/

The dispatch named m3-shapes-receipt, m12-staleness-receipt, merge-shapes-receipt,
merge-fixture-fix-receipt, clippy-strict-census, eh-runner-decision-memo,
a1prime-recipe, dual-artifact-design, stale-wasi-sweep, wasm-link-root-cause,
gws4-postlink, graphlaw-run-live, precommit-acceptance — all live under
`docs/jira/v26.10.22/designs/` (and `docs/jira/v26.10.22/RECEIPTS.md`), NOT under
`receipts/`. `scripts/receipt_chain_sync.sh` enumerates `receipts/` only, so the
chain is correctly current (NO-OP, 8 receipts); those files are outside chain
scope by construction. Promoting them into `receipts/` is a coordinator
transition, not a lane N14 action.

### Chain head

- before: `2c316b0261c992cfb0463d135e4af4f6975daf9318368bfaf531098b57749d05` (NO-OP — no write)
- after:  same (byte-identical, --check VERIFIED)

## Second canary window (S12, 2026-10-22 ~21:0x) — investigation CLOSED QUIESCED

10 rounds × 60s over 37 wired split files: sha256 `c0a96f0b…` stable all
10 rounds; `*_split/` staging dirs never reappeared; `lsof +D` → zero
handles; fswatch/watchman absent; no write fd on any watched file.
Newest mtime remains src/sql_ast/ast.rs 20:06:43. Combined with S8
(5 min): ~15 contiguous minutes hash-stable. Verdict: SAFE-TO-COMMIT
from writer-liveness; the 20:05–20:06 unlogged mutation remains
UNATTRIBUTED — carried in designs/commit-receipt-provenance-gap.md,
which the switchover commit receipt must include verbatim.
