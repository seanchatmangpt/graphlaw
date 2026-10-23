# Commit Drafts Validation — L148

Status: PASS (with 3 corrections applied to drafts in `/tmp/l124-commits/`)
Validated: 2026-10-22, lane L148, repo /Users/sac/graphlaw
Input: 8 drafts in `/tmp/l124-commits/` (L146 updated in place there;
`/tmp/l146-commits/` does not exist — loud check, not a skip).

## Method

Each draft validated against `git status --porcelain` read at use time:
subject length, body presence, per-bucket path existence + dirty/untracked
state, exclusion list, ordering constraints.

## Per-draft verdicts

| Bucket | Subject (chars) | Verdict | Notes |
|---|---|---|---|
| 01-pre-writes | 58 | PASS | src/{abi,dialect,query_ledger,receipt,registry,sql}_split/, crates/graphlaw-eyeron/src/{parser,reasoner}_split/, SPLIT-NOTES.md — all present as untracked; inert pre-writes claim matches status |
| 02-sql-surface | 55 | PASS | all named src/, ontologies/, ggen/, docs/, tests/, examples/ paths dirty/untracked as claimed; owns gws4_ipc_injection_court (deduped vs 05) |
| 03-ring-ledger | 58 | PASS | receipts/CHAIN.blake3 + spc receipts (incl. 3 timestamped) present; chain-resync-receipt.md and spc-append-only-fix.md are references only — the files land in bucket 7 |
| 04-mega-courts | 54 | PASS | docs/jira/v26.10.22/mega/, courts/code-partition/, partition court scripts+selfchecks, smon bins + smon_cli_court.rs all present |
| 05-wasm-linkgreen | 51 | PASS after FIX | see FIX-1, FIX-2 |
| 06-docs-truth | 61 | PASS | LICENSE deletion + LICENSE.md addition confirmed in status; commit-manifest.md and RELEASE-NOTES-DRAFT land here |
| 07-eh-truth-records | 61 (was 76) | PASS after FIX | see FIX-1, FIX-3 |
| 08-registry-repin | 72 | PASS | OPERATOR-DECISION-PENDING marker present in body; commit-last note present. NOT in initial sequence. |

## Corrections applied

- **FIX-1 (dup claims, single-owner rule)**: `tests/gws4_ipc_injection_court.rs`
  was claimed by both bucket 02 and bucket 05; `tests/authority_manifest.rs`
  by both 05 and 08. Bucket 05's court list corrected: gws4 → bucket 02,
  authority_manifest → bucket 08; disclosure line added to 05. File:
  `/tmp/l124-commits/05-wasm-linkgreen.txt` lines 16-18.
- **FIX-2 (subject over 72)**: bucket 07 subject was 76 chars
  ("...wave receipts, designs corpus"); shortened to
  "docs(jira): truth-pass + EH decision records (designs corpus)" (61).
- **FIX-3 (unclaimed paths)**: four untracked designs existed in the tree but
  were named by no bucket:
  `integration-gonogo.md`, `m-wave-verdicts.md`,
  `precommit-acceptance-receipt.md`, `pure-artifact-witness.md`.
  Assigned to bucket 7 (designs corpus) with disclosure line; all four
  confirmed on disk (existence check, exit 0). `commit-manifest.md` was
  already bucket 6's.

## Exclusion list

PASS. No draft claims any path under `.claude/` (hooks/__pycache__,
disk_guard.py, manifest_guard.py, settings.json remain uncommitted and
excluded), no `/tmp` paths, no scratch files. Bucket 05's mention of
/Users/sac paths is a disclosure about wasm/ggen.toml *content*, not a
claimed path — allowed, operator-flagged in commit-manifest.md.

## Ordering

PASS. 01 is the inert pre-writes bucket (first); 08 carries
OPERATOR-DECISION-PENDING and a commit-LAST note — excluded from the initial
sequence until the dual-artifact decision lands. Buckets 02-07 proceed
surface → ring/ledger → courts → wasm → docs → jira designs; no bucket's
gate consumes an artifact from a later bucket except doc_staleness_gate.sh
(bucket 4 script, run as gate in 06/07 — bucket 4 precedes both, holds).

## Receipt

- Draft location check: `ls /tmp/l146-commits/` → "No such file or
  directory"; `/tmp/l124-commits/` holds 8 .txt drafts (real ls output
  above in session log).
- Subject lengths measured: 58/55/58/54/51/61/61/72 (real wc -c output).
- Designs coverage: b7 references 42 names, 0 missing on disk;
  unclaimed-before-fix list of 4 produced by real comm/python diff.
- Post-fix re-measure: all 8 subjects ≤72 (max 72, bucket 08).
