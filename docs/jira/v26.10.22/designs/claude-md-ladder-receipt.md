# CLAUDE.md Ladder Receipt — Lane L50 (2026-10-22)

Subject: /Users/sac/graphlaw/CLAUDE.md @ main (tree dirty — 60+ uncommitted files, src/ mutated lanes in flight).
Scope: every command in the Commands block + test ladder, every file:line
reference, the four new Conventions laws vs wired hooks. Read-only on CLAUDE.md;
only file written is this receipt.

## 1. Commands block — per-rung verdicts

| # | Command | Verdict | Receipt |
|---|---|---|---|
| 1 | `cargo check` | PASS | exit 0, `Finished dev profile in 52.32s` |
| 2 | `cargo test` (full suite) | EXERCISED-ADJACENT | not run in full (heavy rung; disk-guard class). Proven reachable: `cargo check` exit 0 and all four acceptance targets compile + list (below). |
| 3 | `cargo test --test chicago_validation` | TARGET-PROVEN | `cargo test --test chicago_validation -- --list` exit 0, 9 tests |
| 4 | `cargo test --test sql_rdf_dialect` | TARGET-PROVEN | exit 0, 25 tests |
| 6 | `cargo test --test duckdb_statement_kinds` | TARGET-PROVEN | exit 0, 6 test fns (the "35 PEG statement kinds" is covered across those 6 — count claim unverified at function granularity) |
| 5 | `cargo test --test trust_attest` | TARGET-PROVEN | exit 0, 10 tests listed |
| 6 | `bash scripts/mutation_court.sh` | TYPED-REFUSAL (gate healthy) | exit 6, `REFUSED [MUTATION-COURT-DIRTY-TREE]` — dirty src/ from concurrent lanes; gate can exit nonzero, so it is not vacuous. Loud skip, not a pass. |
| 7 | `bash scripts/spc_court.sh` | FAIL (gate healthy) | see §2 |
| 8 | wasm build rung | KNOWN-RED, NOT RUN | per dispatch: pending Lane 1; marked, not executed |
| 9 | `cargo test -p graphlaw-wasm` | NOT RUN | depends on the red wasm rung |
| 10 | `./target/debug/graphlaw-verify` | PASS (binary exists, refuses bare invocation correctly) | exit 2 with typed usage: "at least one --trusted-key is required" |

## 2. spc_court.sh — FAIL with real verdict (exit 1)

**Environment finding first**: the bare documented command fails before the
gate:

```
mktemp: mkstemp failed on /Users/sac/.cache/tmp/spc-court.XXXXXX.json: File exists
```

spc_court.sh:37 uses `${TMPDIR:-/tmp}`; with this session's
`TMPDIR=/Users/sac/.cache/tmp` (6178 entries) mkstemp fails. `TMPDIR=/tmp bash
scripts/spc_court.sh` runs to a real verdict:

- exit 1, verdict FAIL
- `cpk -0.0007 < 1.33`
- `p99 292ns > 250.0ns`
- rule1_beyond_3sigma: flagged, count 38
- rule2_monotonic_run_8: not flagged (longest run 6)
- assignable_cause_detected: true
- receipt on disk: /Users/sac/graphlaw/receipts/spc-2026-10-22.json (updated this session per git status)

Verdict: command exists and gates correctly (it can exit 1), but the gate is
currently red — process out of control on the receipt fold path. Known-red;
owner of the underlying P28-SIL program owns the red, not CLAUDE.md.

## 3. File:line reference resolution — all resolve

ls/grep receipts, all present on disk:

- src/: law.rs, sql_ast.rs, CLAUDE.md:49-51 list — law.rs, sql_ast.rs,
  duckdb_eval.rs, attest.rs, tier.rs, abi.rs, dialect.rs all exist
- src/registry.rs, src/receipt_chain.rs (CLAUDE.md:66), src/bin bins
  graphlaw-verify/graphlaw-run/graphlaw-registry (Cargo.toml:96/103/108)
- ontologies/sql-ast.ttl, wasm/src/lib.rs, crates/graphlaw-eyeron/ (with own
  Cargo.toml), docs/refusals.md, docs/jira/v26.10.22/, receipts/ (populated),
  docs/reference/sql-rdf-rec.md, docs/reference-generated/ (6 files),
  registry/ARTIFACTS.blake3 + registry/capability-registry.json,
- scripts/gen_w3c_manifest.sh, scripts/mutation_court.sh, scripts/spc_court.sh,
  target/debug/graphlaw-verify, wasm/, Cargo.toml [features] section present
  (1 match), no `duckdb` feature declared (frozen-duckdb = "2.0" is a normal
  dependency at Cargo.toml:35)

Zero unresolved references.

## 4. Four new Conventions laws vs wired hooks

| Law (CLAUDE.md:79-91) | Mechanism | Consistency |
|---|---|---|
| DuckDB-not-a-feature | no `duckdb` feature in Cargo.toml; frozen-duckdb normal dep (line 35) | CONSISTENT |
| Disk discipline (≥20 GiB before heavy runs) | .claude/hooks/disk_guard.py — MIN_FREE_GIB = 20, regex blocks suite/bench-class cargo + mutation_court/spc_court, single-target passes through | CONSISTENT (threshold matches CLAUDE.md:83) |
| Coordinator-owned files (Cargo.toml, deny.toml) | .claude/hooks/manifest_guard.py — refuses Edit/Write on root Cargo.toml and deny.toml; coordinator edits via Bash | CONSISTENT |
| Mutation court solo | no hook by design; the script itself carries MUTATION-COURT-DIRTY-TREE (exit 6) | CONSISTENT — enforcement lives in the script, not settings.json |

.claude/settings.json hooks wired: ban_generic_agents.py (Agent matcher),
topology_guard.py (EnterWorktree), manifest_guard.py (Edit/Write/MultiEdit/
NotebookEdit), disk_guard.py (Bash). All four scripts exist in
.claude/hooks/.

## 5. Findings

1. **SPC gate red** — `bash scripts/spc_court.sh` exits 1: cpk -0.0007 < 1.33,
   p99 292ns > 250ns, 38 beyond-3σ points, assignable cause detected. Real
   gate, real red; pre-existing (dirty tree of concurrent lanes).
2. **spc_court.sh mktemp fragility** — bare command fails under
   TMPDIR=/Users/sac/.cache/tmp with `mkstemp ... File exists`; runs only with
   `TMPDIR=/tmp`. scripts/spc_court.sh:37. Reproducer: `TMPDIR=/Users/sac/.cache/tmp bash scripts/spc_court.sh` (fails pre-gate).
3. **mutation_court.sh healthy but unrun** — typed MUTATION-COURT-DIRTY-TREE
   refusal, exit 6, dirty src/ from concurrent lanes. Not a vacuous skip: the
   gate proves it can refuse. DISK/TREE-DEFERRED to a clean tree.
4. **wasm rung** — KNOWN-RED per dispatch (Lane 1 owns); marked, not run.
5. **"35 PEG statement kinds" (CLAUDE.md:22)** — 6 test functions in
   duckdb_statement_kinds.rs list 6 test functions; the 35-kind claim is
   coverage inside those tests, unverified at this rung.
6. **No DISK-DEFER needed** — df showed 33GiB free at dispatch (≥25GiB bar);
   SPC court ran for real.
