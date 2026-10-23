# Switchover provenance census (lane N26)

Date: 2026-10-22, census window 19:30–20:00 local · Lane: N26 (read-only) ·
Repo: /Users/sac/graphlaw @ 615dbfa (dirty working tree).

## 1. Wiring census: wired vs unwired

Verified per tree by grepping the facade for `mod`/`pub use` referencing the
directory and by `git status` (all split dirs are UNTRACKED = new in tree).

| Tree | Facade | Wired? | Facade mod decls (grep evidence) | Split file mtimes | Lines (split files) |
|---|---|---|---| sequence |---|---|
| src/sql_ast/ | src/sql_ast.rs (2,338 lines, was ~2,918; diff −2,018/+723... see §4) | WIRED | `mod ast; mod refusals; mod serializer;` + `pub use ast::*` etc. | 19:46:39 | 1,355 (520+603+232) |
| src/registry/ | src/registry.rs (130 lines) | WIRED | `mod definitions; emit_json; emit_ttl;` | 19:38:00–19:38:15 | 1,954 (884+737+333) |
| src/abi/ | src/abi.rs (341 lines) | WIRED | `mod lease; mod ops;` | 19:32:38–19:33:58 | 928 |
| src/dialect/ | src/dialect.rs (583 lines) | WIRED | `mod refusal; mod sniff;` | 19:33:58 | 584 (265+319) |
| src/query_ledger/ | src/query_ledger.rs (29 lines) | WIRED | `mod batch; mod storage;` | 19:46:39 | ~1,700 both files |
| src/receipt_chain/ | src/receipt_chain.rs (1,033 lines, retains `pub mod epoch`+`pub mod validator` inline) | WIRED | `pub mod frame; mod record; pub mod store;` | 19:39:49–19:40:19 | 1,329 (373+848+108) |
| crates/graphlaw-eyeron/src/parser/ | parser.rs (348 lines) | WIRED | `mod cursor; mod grammar;` + `pub(crate) use cursor::MAX_TERM_NESTING_DEPTH`, `use cursor::{number_literal, Parser}` | 19:49:48–19:50:12 | 1,905 (1,476+429) |
| crates/graphlaw-eyeron at src/reasoner/ | reasoner.rs (4,238 lines) | WIRED (during census, 19:57) | `mod builtins; mod limits; mod solve_context;` + `pub use limits::`pub use limits::ReasonerLimit` | 19:53:35 → 19:57:32–34 | 4,172 across 8 files |

Every planned prewrite tree (src/*_split, eyeron *_split) as named in
SPLIT-NOTES.md used different directory names (src/receipt_split/,
src/abi_split/, etc.). Those directories DO NOT EXIST. The on-disk wired dirs
are named after the module (src/abi/, src/dialect/, src/registry/, ...) — i.e.
the executed switchover did not use the L44–L67 prewritten staging trees as
laid out in SPLIT-NOTES.md; either the staging trees were renamed on switch or
fresh extractions were done. parser_split/ and reasoner_split/ still exist as
untracked staging (parser_split/cursor.rs, grammar.rs, SPLIT-NOTES.md;
reasoner_split/ incl. extract_split.py + provenance.json).

## 2. Compile state (this lane's own runs, CARGO_TARGET_DIR=target-lane-n26)

- 19:54 local: `cargo check -p graphlaw-eyeron` → **33 errors** (E0277 class).
- 19:55 local: rerun → **4 errors** (E0432 `crate::ast::Bindings`,
  E0425 `REGEX_CACHE`, E0603 private `ReasonerLimit` /
  `verify_builtin_triple`) — N25 fix-forward live during census.
- 19:58 local: rerun → **exit 0, clean** for `cargo check -p graphlaw-eyeron`
  AND `cargo check --lib -p graphlaw` (root crate also blocked earlier by the
  same eyeron errors; now Finished dev profile, exit 0).

So the freeze was real during the census window and was fixed forward
in-flight; both crates are green as of 19:58 local 2026-10-22.

## 3. Executor candidates (candidates, not accusations)

Searched ALL session transcripts for Write/Edit/Bash tool_use touching the
wired paths in the 02:30–02:56Z window (19:30–19:56 local):

- Subagent transcripts: ZERO Write/Edit records to any wired split file
  (src/sql_ast/, src/registry/, src/abi/, src/dialect/, src/query_ledger/,
  src/receipt_chain/, eyeron parser/ + reasoner/). Only reads.
- Main session transcript: zero writes; only a coordinator `git add` at
  23:09:06Z (19:09 local) of src/{abi,dialect,...}.rs + queue-operation
  entries at 23:58Z referencing the wired paths (post-wiring).
- Timing correlation: wiring mtimes 19:32:38–19:57:34 local fall inside the
  N-wave dispatch window (N13 reform 18:49, N25 dispatched ~19:49, coordinator
  acceptance probes at 02:47–02:49Z = 19:47–19:49 local already grepping the
  wired facades — i.e. the coordinator observed the wiring as pre-existing
  fact within ~1 min of its mtime, then N25 confirmed it compiles).

Candidates (candidates, not accusations):

1. The coordinator (main-session) process executing the switchover through a
   channel not logged as Write/Edit tool_use (e.g. a bulk script). Not
   evidenced in the logs searched.
2. An out-of-band actor on the same checkout. No other session transcript
   shows activity today after 11:11.
3. The L44–L67 prewrite lanes + a switchover pass whose transcript records
   were not persisted (harness loss) — the wiring itself matches the
   prewrites' item maps (receipt_chain range map in SPLIT-NOTES.md matches
   frame/record/store contents).

The honest finding: the wiring is real on disk with coherent per-tree mtimes
(abi 19:32 → dialect 19:33 → registry 19:38 → receipt_chain 19:39 →
sql_ast 19:46 → parser 19:49 → reasoner 19:53–19:57), but NO logged Write/Edit
exists in this session's transcript corpus. Executor UNDETERMINED from
available evidence; every transcript in the session corpus was searched
programmatically, not sampled.

## 4. Coordinator-facing verdict: commit-readiness and manifest delta

**Manifest delta (L124/L146 commit-manifest.md is stale in three ways):**

1. Bucket 1 "pre-writes (unwired splits)" lists `src/{abi,dialect,query_ledger,receipt,registry,sql}_split/` — those dirs do not exist. The real surface is the six wired dirs (src/{abi,dialect,query_ledger,receipt_chain,registry,sql_ast}/) plus eyeron parser/+reasoner/, all WIRED and now compiling. Bucket 1 must be re-derived: the wired trees are not "inert pre-writes", they change compile behavior and belong with their facade files in one commit per tree (facade + split dir together, never split across buckets).
2. src/sql_ast.rs, src/registry.rs, src/abi.rs, src/dialect.rs, src/query_ledger.rs, src/receipt_chain.rs, eyeron parser.rs/reasoner.rs are MODIFIED (facades), their split dirs UNTRACKED — `git add` of facades without the untracked dirs would create a broken commit. Manifest bucket 2 lists facades; the split dirs must be added to the same bucket rows.
3. Untracked staging dirs `parser_split/` and `reasoner_split/` (incl. extract_split.py, provenance.json, SPLIT-NOTES.md) plus root SPLIT-NOTES.md must be explicitly commit-or-exclude decided; manifest's SPLIT-NOTES.md reference assumes src/*_split names that don't exist.

**Remaining verification before switchover commit (per manifest + N25):**
- [x] `cargo check -p graphlaw-eyeron` exit 0 (this lane, 19:58 local)
- [x] `cargo check --lib -p graphlaw` exit 0 (this lane, 19:59 local)
- [ ] full host suite `cargo test` (not run by N26)
- [ ] `cargo test -p graphlaw-eyeron` (N25 owns)
- [ ] registry regen via real flow (L73/L122; registry/ artifacts already show M
  in git status — regen receipt p7-final-regen-prestage.md is the gate)
- [ ] mutation/SPC courts rerun post-switchover (line ranges moved; mutation
  sites may have drifted)
- unchecked: whether untracked staging dirs (parser_split/, reasoner_split/,
  SPLIT-NOTES.md) are committed or excluded — coordinator decision.

## 5. Receipt

Commands run (real output embedded above in census sections): ls -lT, wc -l,
grep, git status --porcelain, transcript JSONL sweep (python, full corpus, 0
Write/Edit hits to wired paths), CARGO_TARGET_DIR=target-lane-n26 cargo check
(p graphlaw-eyeron → 33 → 4 → exit 0; --lib -p graphlaw → exit 0).
Executor: UNDETERMINED. Working tree NOT commit-ready as-manifested; manifest
buckets 1/2/8 require the deltas in §4.
