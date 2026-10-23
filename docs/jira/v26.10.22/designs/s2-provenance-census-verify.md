# S2 skeptic verification of N26 switchover provenance census

Date 2026-10-22 20:00–20:12 PDT · Lane S2 (read-only) · repo /Users/sac/graphlaw @ 615dbfa (dirty).
Method: independent re-derivation + own sweep tooling (python over jsonl), not a re-read of N26.

## 1. All-8-wired state: CONFIRMED independently

Grepped each facade for mod decls, listed each dir, stat'ed birth+mtime:

| Tree | Wired facade evidence (my own grep) | Split files (birth/mod, Oct 22 local) |
|---|---|---|
| src/sql_ast/ | sql_ast.rs:18-25 `mod ast; mod refusals; mod serializer;` + pub use | ast.rs 19:44/20:06; refusals, serializer 19:46 |
| src/registry/ | registry.rs:17-23 `mod definitions; emit_json; emit_ttl;` | 19:36/19:38 |
| src/abi/ | abi.rs:39-42 `mod lease; mod ops;` | lease.rs 19:32/19:32; ops/ = dir, 5 files 19:33 |
| src/dialect/ | dialect.rs:112-116 `mod refusal; mod sniff;` | 19:33 |
| src/query_ledger/ | query_ledger.rs:25-29 `mod batch; mod storage;` | 19:46 |
| src/receipt_chain/ | receipt_chain.rs:44-48 `pub mod frame; mod record; pub mod store;` | 19:39-19:40 |
| eyeron parser/ | parser.rs:6-9 `mod cursor; mod grammar;` | cursor 19:49/20:06; grammar post-19:58 |
| eyeron reasoner/ | reasoner.rs:14-25 `mod builtins; mod limits; mod solve_context;` | builtins/ dir (5 files, mod 20:05-20:06); limits, solve_context 19:53/20:05 |

Note: abi/ops and reasoner/builtins are DIRECTORIES (N26's line counts treated them flatly;
actual split surface is 12 files + 2 nested dirs, not 8 flat trees).

## 2. Transcript sweep (independent method, different tool than N26)

python3 over ALL jsonl (main 223f3838 + recursive glob incl. subagents/agent-*.jsonl;
Claude Code timestamps verified UTC by comparing last-record vs `date -u`):
window 2026-10-23T02:30–03:00Z (= 19:30–20:00 local):

- Write/Edit/MultiEdit to ANY wired split dir: **0 hits** — CONFIRMS N26's ZERO.
- Bash touching split paths in window: only 9 commands, all read-only greps/ls
  (02:52–02:57Z) from other skeptic lanes. No mv/cp/rsync/heredoc-write/python-writer.
- Subagent corpus: prewrite lanes (L44/L46 etc.) wrote `src/*_split/` staging +
  SPLIT-NOTES.md at 23:46–00:20Z (16:46–17:20 local) — BEFORE the wiring window, coherent
  with the prewrite->switchover model. Facade Edits in-window are to .rs facades, not splits.

## 3. Out-of-band actor evidence

- Processes: exactly ONE claude process (PID 23692, started 11:11AM) at 20:07 local.
  No second agent runtime. lumen LSP/config-lsp irrelevant.
- Shell history: grep of ~/.zsh_history for split paths/extract_split: ZERO entries.
- fsevents: not retroactively queryable; no dtrace/FSEvent stream was running in window.
- Trash: no *_split dirs in ~/.Trash.
- Staging dirs: `src/receipt_split/`, `src/registry_split/`, `src/sql_split/`,
  `src/abi_split/`, `src/query_ledger_split/`, eyeron `parser_split/`, `reasoner_split/`
  (the exact paths written by the logged prewrite lanes, incl. extract_split.py +
  provenance.json) **ALL NO LONGER EXIST** at 20:08 local. N26 observed
  parser_split/ + reasoner_split/ present at ~19:58. Their deletion happened
  in-window or immediately post-window, with ZERO logged Write/Edit/Bash.
- SPLIT-NOTES.md at repo root survives and documents byte-identity receipts for the
  staging content; the wired files carry the matching shapes (registry definitions.rs
  spot-check matches the prewrite provenance table).

## 4. NEW FINDING (post-census, hardens rather than refutes N26)

The wiring is STILL ONGOING through the unlogged channel: src/sql_ast/ast.rs mtime
20:06:43, reasoner/builtins/*.rs 20:05–20:06, reasoner.rs/parser.rs re-modified after
19:58 — with ZERO Write/Edit records at those timestamps in ANY transcript (sweep
03:00–03:10Z: 0 hits). An unattributed writer is actuating on this checkout right now,
outside every logged tool_use channel.

## 5. Verdict

**UNDETERMINED CONFIRMED, with escalation.** N26's zero-write finding replicates under an
independent method (recursive python jsonl sweep, UTC-verified window). No attribution to
any logged actor is possible. But the census understates the situation:

1. The unlogged writer was (and at 20:05–20:06 still was) ACTIVE after the census window.
2. The prewrite staging dirs (incl. the only provenance-bearing artifacts
   extract_split.py / provenance.json) were deleted during/after the window — the
   remaining provenance record (SPLIT-NOTES.md) documents staging paths that no longer
   exist. Committing now would commit unattributed content with its provenance evidence
   destroyed.
3. Coordinator action: do NOT commit until the unlogged writer is identified/stopped;
   consider lsof/fs_usage on the wired paths or a canary file to trap the writer alive;
   treat deleted staging dirs as a provenance gap to record in the commit receipt.

Own exits: all sweeps python3 exit 0; grep/ls/stat exit 0 (one harmless zsh glob error on
`echo ===`, no effect); Trash/history/other-project checks exit 0/2 (empty results).
