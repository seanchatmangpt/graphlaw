# Commit receipt — staged provenance-gap section (lane L176)

**Status: STAGED — coordinator includes verbatim in the switchover commits'
receipt.** Repo: /Users/sac/graphlaw @ 615dbfa (dirty working tree), 2026-10-22.
Every claim below is sourced from an on-disk receipt, verified present at
use time (`test -f` exit 0 for each cited path). Two of the dispatch's cited
sources are NOT on disk and are recorded as such, not absorbed (see §3 and §5).

Sources read at use time (all exit 0):

- `docs/jira/v26.10.22/designs/switchover-provenance-census.md` (N26)
- `designs/s2-provenance-census-verify.md` (S2)
- `designs/s8-unlogged-writer-watch.md` (S8, 20:14–20:20 local)
- `designs/surface-six-entry-patch.md` (L172, STAGED, DO-NOT-APPLY)
- `designs/receipt-surface-post-switchover.md` (N27, PROPOSED-NOT-APPLIED)
- `designs/eyeron-switchover-fixforward.md` (N25)
- `designs/precommit-acceptance-receipt.md` (L147 + L147b/N23 section)
- `designs/commit-manifest.md` (L124/L146)

## §1 The unattributed executor

Eight split trees — `src/sql_ast/`, `src/registry/`, `src/abi/`,
`src/dialect/`, `src/query_ledger/`, `src/receipt_chain/`, eyeron
`crates/graphlaw-eyeron/src/parser/`, and
`crates/graphlaw-eyeron/src/reasoner/` — are WIRED into the compile (each
facade carries `mod`/`pub use` decls; N26 §1 table, S2 §1 independently
re-derived) with per-tree coherent mtimes 19:32:38–19:57:34 local 2026-10-22
(abi 19:32 → dialect 19:33 → registry 19:38 → receipt_chain 19:39 → sql_ast
19:46 → parser 19:49 → reasoner 19:53–19:57; N26 §1/§3). A programmatic sweep
of the FULL session transcript corpus — main session + recursive
`subagents/agent-*.jsonl`, UTC-window-verified — found **ZERO logged
Write/Edit tool_use records touching any wired split path** in the wiring
window (N26 §3; S2 §2 confirmed by an independent python jsonl sweep with a
different tool). The only in-window Bash hits were 9 read-only greps/ls from
skeptic lanes (S2 §2).

The wiring was STILL LIVE after the census window: `src/sql_ast/ast.rs` mtime
20:06:43, `reasoner/builtins/*.rs` 20:05–20:06, facade re-modifications after
19:58, again with zero transcript records at those timestamps (S2 §4). The
writer is quiesced as of the canary: S8's bounded-poll watch (POLL-STEP,
5 rounds x 60 s, sha256-compared snapshots of 31 files across all 8 trees)
observed **zero changes over 20:14–20:20 local** (byte-exact before/after,
sha `e1c52602…f7854`); `lsof +D` on the watched dirs returned zero handles;
only the session's own claude process (PID 23692) and unrelated cargo/rustc
builds under `target*/` were alive (S8 §Method/§Attribution). S8's most recent
observed mtimes (20:05–20:06) match S2's finding, i.e. the 20:05–20:06 writer
terminated or went quiet before 20:14; **identity unknown** (S8 §Attribution).

Verdict of record: **executor UNDETERMINED** (N26 §3/§5; S2 §5 "UNDETERMINED
CONFIRMED, with escalation"). Candidate channels S2 ruled out: no second agent
runtime (one claude process), empty `~/.zsh_history` hits for split paths/
extract_split, empty Trash for `*_split`, no fswatch/watchman/launchd/cron
infrastructure, no editor handles (S2 §3; S8 §Attribution).

## §2 Staging-dir deletion — provenance evidence destroyed

The only provenance-bearing artifacts of the switchover — the prewrite staging
dirs (`src/receipt_split/`, `src/registry_split/`, `src/sql_split/`,
`src/abi_split/`, `src/query_ledger_split/`, eyeron `parser_split/` and
`reasoner_split/`, including `extract_split.py` and **the only
`provenance.json`**) — **no longer exist**. S2 observed them gone at 20:08
local, after N26 saw `parser_split/` + `reasoner_split/` present at ~19:58
(N26 §1; S2 §3), with **zero logged Write/Edit/Bash** for the deletion (S2 §3).
Re-verified live by this lane at write time: `ls -d src/*_split
crates/graphlaw-eyeron/src/*_split` → no matches (zsh exit 1, both globs
empty). The surviving `SPLIT-NOTES.md` documents byte-identity receipts for
staging paths that no longer exist (S2 §3) — the remaining provenance record
describes destroyed artifacts. The deletion traveled through the **same
unlogged channel as the wiring** (S2 §3/§4), so the wired content is committed
with its provenance evidence destroyed. This is a provenance gap of record;
it does not by itself impugn the content (§3).

## §3 Content accepted on merits, independent of provenance

The wired content is verified on its own terms by on-disk test receipts,
independent of who wrote it:

| Surface | Receipt | Result |
|---|---|---|
| eyeron lib + sql_rdf_dialect + chicago_validation | N25, `designs/eyeron-switchover-fixforward.md` gates table | `cargo test -p graphlaw-eyeron` 21 passed / 0 failed; `sql_rdf_dialect` 33/0; `chicago_validation` 9/0; `cargo check -p graphlaw-eyeron` exit 0 (all exit 0, `target-lane-n25`) |
| Acceptance set | L147b/N23, `designs/precommit-acceptance-receipt.md` §L147b | 128 tests, **124 passed / 4 failed — all 4 the expected-known registry-drift rows** (stale `registry/ARTIFACTS.blake3` pins; deterministic across two runs; drift court itself ALIVE) |
| wasm host contract | `designs/link-green-receipts.md` (L1): `cargo test -p graphlaw-wasm` 9 passed / 0 failed; L125 court `tests/wasm_duckdb_op_wired` → `3 passed; 0 failed; 1 ignored` (typed BLOCKED-MODULE-EXEC) per `designs/integration-gonogo.md` P3 row + `designs/wasm-duckdb-op-wired.md` | wasm boundary green on the wired tree |

Note on labels: the dispatch phrased these as "V1 (110/0), V2 (21/0+33/0),
S5 (wasm 9/0+3/0)". The **counts** are receipted on disk under lane labels
N25 (21/0 + 33/0 + 9/0), N23/L147b (124/4-known), and L1 (wasm 9/0). A
"110/0" run has **no receipt on disk** (grep over the whole milestone dir,
exit 1) and is recorded as UNRECEIPTED — if a V1 receipt lands, it amends this
section; until then 110/0 is not claimed. S5 has no receipt file on disk
either; the wasm counts cited stand on L1/L125 receipts instead.

Content stands on: real frozen-duckdb courts green at expected counts, the
known-drift rows typed and deterministic, and S8's own commit-safety
recommendation — "commit if and only if the coordinator also accepts the
content (courts/gates) independent of provenance" (S8 §Commit-safety).

## §4 Open follow-ups recorded against the commit

1. **Attribution investigation remains OPEN.** Executor UNDETERMINED (§1).
   A "110/0" V1-class run receipt, when it lands, amends §3. S12's canary
   result appends here when it lands; until then S8's 20:14–20:20 QUIESCED
   window is the witness of record.
2. **L172 six-entry receipt-surface patch is STAGED, DO-NOT-APPLY** until the
   quiescent window, sequenced after green compile and BEFORE any
   `graphlaw-registry -- --write` (`designs/surface-six-entry-patch.md`).
   N27's FILES-extension proposal with `tests/registry_artifacts.rs:907`
   len assert 2→5 must land in the same regen window (`designs/receipt-surface-post-switchover.md` §4–§5).
3. **P7 final regen + 117/0 re-run** of the L147 acceptance set before the
   switchover commits (`designs/integration-gonogo.md` P7 row; emitter hint
   `cargo run --features abi --bin graphlaw-registry -- --write`).
4. **Manifest bucket deltas from N26 §4** apply: wired trees commit as
   facade + split dir in one commit per tree, never split across buckets.

## §5 Receipt

- File written: this one (the only lane write).
- Files read: the ten receipt paths listed in the header — all `test -f`
  exit 0 at use time.
- Live re-verification by this lane: `ls -d src/*_split
  crates/graphlaw-eyeron/src/*_split` → "no matches found" (zsh exit 1) —
  staging dirs confirmed deleted as of write time, matching S2 §3.
- Commands with no on-disk receipt: `grep -rn "110 passed|110 ok|= 110|110
  tests"` over `docs/jira/v26.10.22/` → 0 hits, exit 1 (basis for the
  UNRECEIPTED marking in §3).
- Lane target dir: none (doc-only lane).
