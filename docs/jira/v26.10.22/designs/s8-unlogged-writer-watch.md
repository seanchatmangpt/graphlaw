# S8 — Unlogged Writer Watch (observation lane)

Lane S8, /Users/sac/graphlaw, one canonical checkout. 2026-10-22 20:14–20:20 local (UTC-7).

## Verdict

**QUIESCED over the observed window — no write observed in 5 rounds × 60 s.**
Recommendation: **SAFE-TO-COMMIT from the writer-liveness standpoint only.**
(The underlying S2 finding — source files mutated with zero transcript Write/Edit
records — remains unexplained by this lane and stands.)

## Method (executed, with exits)

1. Snapshot (exit 0): stat(mtime,size) + sha256 of 31 files — src/sql_ast{,.rs,/*},
   src/registry{,.rs,/*}, src/abi{,.rs,/*}, src/dialect{,.rs,/*},
   src/query_ledger{,.rs,/*}, src/receipt_chain{,.rs,/*}, and eyeron parser/
   + reasoner/ (incl. mod files) — into `/tmp/s8-canary/before.jsonl`
   (sha256 e1c5260259f18ddf5ae4bb3d23a7878fe813b90dfe4d35f6e5e347ce579f7854). (historical)
2. Bounded poll — SLEEP-CLASS: POLL-STEP (single audited poll implementation,
   bounded 60 s steps, deadline-checked): 5 rounds, snapshots round1..round5.
3. `diff` round r vs r+1 for all pairs: **IDENTICAL** (byte-exact; sha256 of
   before.jsonl == round5.jsonl). Zero mtime/size/hash changes across ~5 min.
4. Attribution probes on baseline (no trip occurred, so no lsof-on-change fired):
   - Processes touching the tree: only `cargo`/`rustc` build processes writing
     under `target/` and `target-lane-s6/` (PIDs 29059/29328, 30200/30330-30359,
     started 20:13–20:14). None open handles on src/ or crates/*/src (confirmed
     via `lsof +D src crates/graphlaw-eyeron/src` → empty).
   - `lsof +D` on the watched source dirs: zero handles.
   - fswatch/watchman: not installed, not running. launchd user agents: no
     graphlaw entries. crontab -l: empty.
   - Other editor processes: no Code/vim/nvim/emacs/subl with handles on the tree.
5. Defect in my own first poll comparator (join/awk field indices wrong) produced
   5 false TRIPPED entries and 1.7 MB of noise log; detected by direct pairwise
   `diff` of the raw snapshots, which is the evidence of record. /tmp/s8-canary/
   contains round0..round5.jsonl, env-baseline.txt, poll.log (noise), TRIPPED (false).

## Attribution

**Absent.** No writer was caught in the act; nothing to attribute. The most recent
file mtimes in the watched set are 20:05–20:06 local (e.g. eyeron parser.rs
mtime 03:06:19 UTC), i.e. ~8 minutes before this lane started watching, and
nothing has touched them since. The 20:05–20:06 writer (S2's finding) terminated
or went quiet before 20:14; identity unknown. Only this session's claude process
(PID 23692) and unrelated cargo/rustc builds were alive during the window.

## Limitations

- 60 s snapshot granularity: a single write+restore inside one 60 s window would
  be missed. Hash-compare (not mtime-compare) was used, so a write-restore would
  still need to restore byte-exact content AND mtimes to evade — unlikely.
- Window is 5 minutes. A writer on a longer cadence is not excluded.
- fs_usage (per-syscall trace) requires root; not run.

## Commit-safety recommendation

Safe to commit from liveness standpoint (tree byte-stable, no writer, no watcher
infrastructure present). However the unlogged 20:05–20:06 mutation itself is
unattributed — commit if and only if the coordinator also accepts the content
(courts/gates) independent of provenance.

## Second canary window (S12)

Lane S12, /Users/sac/graphlaw, one canonical checkout. 2026-10-22 20:26–20:37 local (UTC-7).

### Verdict

**STILL QUIESCED — zero writes across 10 rounds × 60 s (~10 min), on top of S8's 5 min.**
Combined observed-quiescence: ~15 minutes of contiguous hash-level stillness, cutting
S8's "longer-cadence writer not excluded" limitation down to cadences > 15 min.
Staging-deletion check: **the `*_split/` staging dirs have NOT reappeared** (ls exit 1,
no matches, at window start; unchanged through the window — any reappearance inside a
round would have altered snapshot content and tripped the comparator).

### Method (executed, with exits)

1. Fresh snapshot (exit 0): stat(mtime,size)+sha256 of **37 files** — src/sql_ast{,.rs,/*},
   src/registry{,.rs,/*}, src/abi{,.rs,/*}, src/abi_manifest.rs, src/dialect{,.rs,/*},
   src/query_ledger{,.rs,/*}, src/receipt_chain{,.rs,/*}, eyeron parser{,.rs,/*},
   reasoner{,.rs,/*} — via `/tmp/s12-canary/snap.sh` into `before.jsonl`
   (sha256 c0a96f0bcb9a7d0f4c5837729d356ccd209922260666b4500e03b862c55015d8). (historical)
2. Bounded poll — SLEEP-CLASS: POLL-STEP (single audited poll implementation, 60 s
   bounded steps): 10 rounds, snapshot per round into round1..round10.jsonl.
3. Byte-exact `cmp` of each round vs baseline: **exit 0 all 10 rounds**; final
   round10.jsonl sha256 == baseline (c0a96f0bcb9a7d0f4c5837729d356ccd209922260666b4500e03b862c55015d8). (historical)
   No TRIPPED file exists; no lsof/ps attribution fired because no trip occurred.
4. Baseline attribution probes (no trip, so probes are corroboration not capture):
   - `lsof +D src crates/graphlaw-eyeron/src`: **zero handles** on any watched source file (exit 0, empty).
   - fswatch/watchman: not running (pgrep exit 1). No fs-watcher infrastructure present.
   - `lsof +D .` non-target handles: only this session's harness (claude PID 23692 cwd,
     lumen/oclnr-mcp/agy MCP servers cwd-only) and read-only fds on `.git/HEAD`,
     `.git/index`, `COMMIT_EDITMSG`, `FETCH_HEAD`, `ORIG_HEAD` from the coordinator's
     `agy` (PID 24999) — cwd or read-only; **no write fd on any watched source file**.
   - Newest mtime in the watched set: src/sql_ast/ast.rs at 1792724803
     (20:06:43 local) — S2's 20:05–20:06 mutation window, untouched since.
5. Script + 11 snapshots preserved at /tmp/s12-canary/ for replay.

### Commit-safety call

**SAFE-TO-COMMIT** (writer-liveness standpoint), now corroborated across two independent
windows (~15 min combined hash-level quiescence, no writer, no watcher infra, no write
fd on the watched set). The 20:05–20:06 unlogged mutation remains unattributed — the
provenance gap stands. Per S8's gate: commit if and only if the coordinator accepts the
content via courts/gates independent of provenance.

### Recommendation

Close the investigation as QUIESCED with the provenance-gap receipt note: the tree is
stable across two windows; the 20:05–20:06 writer was transient, terminated or went
quiet before 20:14, identity unknown. If a recurrence is observed, re-open with
`fs_usage` (root) for per-syscall attribution, which neither window could run.
