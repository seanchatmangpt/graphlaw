# SQL-in-RDF tickets — v26.10.22

> **As of 2026-11-09:** historical record of the v26.10.22 window; body kept as written. Superseded as the active window by [v26.10.28](../v26.10.28/INDEX.md). Sibling files archived out of this directory are recoverable from git history; links to them were removed.

Status of every ticket: as recorded in the dispatch-era tables below
(kept as-written; this index is append-only), with the **closeout standing
of all 32 rows recorded in the [WRAP section](#wrap--v261022-closeout-2026-10-24)
(2026-10-24, closeout lane)** — 15 Closed / 13 Deferred / 1 Withdrawn /
1 Absorbed / 2 program no-flip, each flip receipt-cited there. Prior
exceptions: SQLRDF-08 (withdrawn: DuckDB is inside the wasm unit), SQLRDF-27
(Closed 2026-10-23, receipt-gated on FIBO 4/0 + lifecycle 3/0 — see
RECEIPTS.md "2026-10-23 ledger truth-sync"), SQLRDF-26 (ALIVE — atomic
document execution landed, verification receipts inline in
SQLRDF-26 (ticket file archived)). The CanonArrow ADR
([ADR-canonarrow-dual-witness-egress.md](ADR-canonarrow-dual-witness-egress.md))
is absorbed into the v26.10.22 release boundary — the 26.10.23 moniker is
deferred.

The conformance target is the pasted sql-rdf Recommendation, the conformance text for v26.10.22. Section citations in the tickets point at that text. SQLRDF-12 files it at `docs/reference/sql-rdf-rec.md`.

These surfaces already exist, and the tickets do not ask for them again:

- [`src/sql_ast.rs`](../../../src/sql_ast.rs) compiles `sql:selectList`, `sql:fromTable`, one `sql:joinClause`, `sql:whereClause`, `sql:groupBy`, `sql:limit`, and `sql:offset` for `SELECT` (`SelectStatement::compile` at lines 222–290, `extract_select_statement` at lines 499–616).
- The other 32 runnable PEG classes compile as statement templates in `DatasetReader::compile_command` (lines 771–911) and run on in-memory frozen-duckdb.
- `CONNECT` and `DISCONNECT` are compile refusals inside `compile_turtle` (lines 667–671).
- Chicago receipts in [`tests/chicago_validation.rs`](../../../tests/chicago_validation.rs) hash SQL text plus a table readback (`state_receipt`, lines 127–151).
- `graphlaw-wasm` is the one unit. It links frozen-duckdb. `REFUSAL-04` is the non-document-backed builtin reader ([`docs/refusals.md`](../../refusals.md)).

Compiler files the tickets point at:

- [`src/sql_ast.rs`](../../../src/sql_ast.rs)
- [`ontologies/sql-ast.ttl`](../../../ontologies/sql-ast.ttl)
- [`src/duckdb_eval.rs`](../../../src/duckdb_eval.rs)
- [`docs/refusals.md`](../../refusals.md)
- [`tests/chicago_validation.rs`](../../../tests/chicago_validation.rs)
- [`wasm/Cargo.toml`](../../../wasm/Cargo.toml)

| Id | Title | Status | Gap | Ticket |
| --- | --- | --- | --- | --- |

## Lanes

Wave map: 8 build lanes + 13 per-ticket DoD courts over one canonical checkout
(`/Users/sac/graphlaw`). Ticket Status fields stay **Open**; the coordinator
flips them at integration. Latest observed verdicts per ticket live in
[RECEIPTS.md](RECEIPTS.md). [WRAP closeout, 2026-10-24]: the integration flip
pass ran as the WRAP section's receipt-cited flip table below — the table
cells above stay as-filed (append-only index); the WRAP table is the
authoritative end-state.

| Lane | Scope | Tickets / files |
| --- | --- | --- |
| L1 | Ontology | SQLRDF-01, SQLRDF-02 · [`ontologies/sql-ast.ttl`](../../../ontologies/sql-ast.ttl) |
| L2 | Compiler core | SQLRDF-02, SQLRDF-03, SQLRDF-04, SQLRDF-06, SQLRDF-07 · [`src/sql_ast.rs`](../../../src/sql_ast.rs), [`src/duckdb_eval.rs`](../../../src/duckdb_eval.rs) |
| L3 | Statement kinds | SQLRDF-05 · [`src/sql_statement_kinds.rs`](../../../src/sql_statement_kinds.rs) |
| L4 | Chicago courts | SQLRDF-04, SQLRDF-07, SQLRDF-09 · [`tests/chicago_validation.rs`](../../../tests/chicago_validation.rs) |
| L5 | Receipts | SQLRDF-09 · [`src/duckdb_eval.rs`](../../../src/duckdb_eval.rs) |
| L6 | Wasm | SQLRDF-08 · [`wasm/Cargo.toml`](../../../wasm/Cargo.toml), [`wasm/src/lib.rs`](../../../wasm/src/lib.rs) |
| L7 | Refusals register | SQLRDF-02, SQLRDF-06, SQLRDF-07, SQLRDF-08, SQLRDF-10 · [`docs/refusals.md`](../../refusals.md) |
| L8 | Milestone docs | SQLRDF-12 · this directory, [`docs/reference/sql-rdf-rec.md`](../../reference/sql-rdf-rec.md) |
| DoD-D1..D13 | Per-ticket courts | One DoD court per ticket SQLRDF-01..SQLRDF-12 plus the cross-ticket anti-vacuity run (`bash scripts/mutation_court.sh`) |

### Hardening wave 2 closures (post-integration, 2026-10-22)

Verified on disk; per-lane receipts in [RECEIPTS.md](RECEIPTS.md)
"Successor wave 2" section.

- **SPOF #3 (mutex poisoning = process-wide cache brick) closed** —
  `src/query_cache.rs:53-63` recovers the inner guard via
  `PoisonError::into_inner`; no `.expect` on the shared lock remains.
- **Silent-skip census item closed** — `tests/insa_law_order.rs:51-73`
  emits a loud typed skip naming the missing pack path
  (`INSA_LAW_PACK_ONTOLOGY` override) instead of a silent pass.
- **Stale-claims remediation landed** — `docs/wasm-integration.md` pin
  section rewritten to the then-live pin `332ea218…2c30c` (6,793,617 B; historical — superseded by final pin `472d5557…bf80ea`, RECEIPTS.md §doc-impl-reconciliation);
  REFUSAL-04 trigger corrected at `docs/refusals.md:45`; INDEX court
  counts 18/18 → 25/0 synced (rows SQLRDF-03/09). Audit residue: registry
  re-pin (C9) **closed in wave 3** (see below); `docs/WASM.md`
  pending-update marker still open (verified on disk, "re-pin is pending
  the reproducible-SDK rebuild").

### Wave 3 closures (skeptic + DX + drift, 2026-10-22 ~17:4x)

Verified on disk; per-lane receipts in [RECEIPTS.md](RECEIPTS.md)
"Successor wave 3" section.

- **Registry re-pin (C9) closed** — the real regen flow
  (`graphlaw-registry -- --write`) re-pinned `registry` → `4cd56fd5…` and
  `receipt-surface` → `0cc02ca4…`; `registry_artifacts` 22/0,
  `registry_dispatch` 14/0, both exit 0; wasm row untouched
  (`designs/registry-regen-receipt.md`) — the wasm pin row was untouched
  at wave-3 time and is superseded by the final pin `472d5557…bf80ea`
  (RECEIPTS.md §(a)/§doc-impl-reconciliation).
- **Refusal register reconciled** — 12 surfaces enumerated, 5 additions
  (kernel `NO-CANONICAL-GRAPH`, `ATTEST-REFUSED`; 5-row harness section);
  0 renumberings, 0 discriminant defects
  (`designs/refusals-register-reconciliation.md`).
- **Shapes ↔ Rec §4 reconciliation applied** — rows 16/19/12/§2.B in
  `ontologies/sql-ast.ttl` + `ontologies/shapes/ast_governance.ttl`;
  courts `sql_rdf_dialect` 25/0, `w3c_conformance` 8/0
  (`designs/shapes-reconciliation-applied.md`).
- **Silent-skip census (scripts) clean** — 5 scripts census: 0 ORPHAN,
  every script runnable-with-receipt or loudly LINK-BLOCKED (typed exit 1)
  (`designs/scripts-census.md`); pending-marker census: 126 hits →
  7 RESOLVED-SINCE / 10 LIVE-BLOCKED / 0 ABANDONED
  (`designs/pending-marker-census.md`).
- **Split pre-writes verified, now wired** — L43/L45/L46/L67 VERIFIED
  byte-exact (174/186 checks); L44 discrepancies repaired by L76 (missing
  RefusalCode/RefusalContext/AstError extracted, base pin corrected);
  L66 reformat-vs-verbatim finding **closed in the N2 reformat pass**
  (re-slice to byte-exact verbatim recorded in the former
  `src/dialect_split/SPLIT-NOTES.md` "Repair record", rustfmt parse gate
  exit 0, files re-sliced 18:34 — see wave 5 below); L65 unverified. All
  eight trees are `mod`-wired (`dialect`, `abi`, `sql_ast`, `query_ledger`,
  `receipt_chain`, `registry`, eyeron `parser`, eyeron `reasoner`) and the
  `*_split` staging folders, including their SPLIT-NOTES, are deleted
  (`designs/split-prewrite-verification.md`).
- **Wasm4pm fixtures 44/46 ALIVE** at graphlaw 615dbfa; 4 refreshed,
  6 alive-with-registered provenance drift; oracle gate DISK-BLOCKED
  (`designs/wasm4pm-fixtures-drift.md`).
- **Clippy-strict census (L120) closed** — pre-fix ~96 warning sites
  counted by lint class; after `cargo clippy --fix` + lane edits, 0 lint
  warnings remain except 41 residual `result_large_err` sites (39 lib +
  2 wasm_boundary_fuzz), which the `-D warnings` gate fails on exactly
  and loudly (`designs/clippy-strict-census.md`, "Gate results").
- **Doc-staleness gate (L35) landed** — `scripts/doc_staleness_gate.sh`
  (exit 0/1/2); anti-vacuity witnessed: 50 findings EXIT=1 honest-FAIL,
  injected stale file flips 50→52, removal restores 50/EXIT=1
  (`designs/doc-staleness-gate-design.md:92-115`); findings are the
  remediation backlog, not a silent pass.
- **Examples census (L70) closed** — `cargo build --examples` EXIT=0,
  **15 COMPILED+RAN / 0 COMPILE-FAIL / 0 SKIP-TIMEOUT** at 615dbfa;
  cross-repo fixture side effect disclosed, not lane-owned
  (`designs/examples-census.md`).
- Still open after wave 3: `docs/WASM.md` pending-update marker (re-pin
  still pending the reproducible-SDK rebuild, `docs/WASM.md:99-104`);
  TC_HOT_FULL receipt filed 2026-10-22
  (`designs/tc-hot-01-errc-receipt.md`: 10M p99 209 ns inside 250, refusal
  p99 42 ns PASS, variance 26.91% FAIL, SPC Cpk 0.001 FAIL).
  The full instrument-by-instrument gathering is
  designs/benchmark-census.md: six SPC
  JSON receipts, all FAIL, highest Cpk 0.555. Prose that still quotes
  cpk −0.0018 and p99 375 ns for `receipts/spc-2026-10-22.json` is
  describing bytes that file no longer holds (current object: Cpk
  0.2948, p99 250 ns).
  mutation-court FULL receipt still pending. [Wave 5 update, 2026-10-22]: the L66 dialect_split
  reformat row is **closed** — the re-slice to byte-exact verbatim landed
  and is recorded in `src/dialect_split/SPLIT-NOTES.md` "Repair record
  (lane L76)": 4 reflow sites re-sliced whole-body from the working tree
  (`refusal.rs` = dialect.rs 112–265 + 507–555; `sniff.rs` = 556–880),
  containment verified byte-identical, rustfmt parse gate exit 0, split
  files re-sliced on disk 18:34; "no reflow annotation remains — the
  slices are now verbatim". `designs/split-prewrite-verification.md`
  "Bottom line" (4 sites vs verbatim) is superseded by that repair
  record. Wave 5 left the trees unwired; the switchover has since landed
  (see the split bullet above).

### Wave 4 closures (link-green + truth pass, 2026-10-22 ~17:5x)

Verified on disk; per-lane receipts in [RECEIPTS.md](RECEIPTS.md)
"Successor wave 4" section.

- **Wasm link (L1) GREEN** — `WASI_SDK_PATH` is a wasi-sdk root
  (`bin/clang`, `share/cmake/wasi-sdk-p1.cmake`; CI installs wasi-sdk 34 at
  `/opt/wasi-sdk`; the engine cache is not that root). The link is witnessed
  green (2026-10-22, exit 0) at `docs/WASM.md` "Pin currency"
  (`designs/sweep-pre-integration-2.md`). The `332ea218…` pin was NOT
  replaced — re-pin is still pending the reproducible-SDK rebuild; the
  coordinator MUST NOT re-pin until then (`docs/WASM.md:99-104`). L1's
  four-hop root-cause receipt is UNFILED (loud).
- **Refusal register additions landed** — kernel `NO-CANONICAL-GRAPH`,
  `ATTEST-REFUSED`; harness section `WASI_PATCH_BUILD_REFUSED`,
  `MUTATION-COURT-DIRTY-TREE` (exit 6),
  `GRAPHLAW-WASM-BUILD-FAILED`/`BLOCKED-LINK`, design-stage
  MANIFEST/DISK-REFUSED; reconciled 0 renumberings
  (`designs/refusals-register-reconciliation.md`). RESOLVED-SINCE note:
  the wasm-link harness refusal's subject is now link-green, so
  `GRAPHLAW-WASM-BUILD-FAILED`/`BLOCKED-LINK` fire only on hosts whose
  `WASI_SDK_PATH` is not a wasi-sdk root (`bin/clang`,
  `share/cmake/wasi-sdk-p1.cmake`). The engine cache is not that root.
- **L89/L98 UNFILED markers closed** — reports landed:
  `designs/l89-marketplace-qualification.md`,
  `designs/l98-wasm4pm-vecgen-provenance.md`.
- **L106 cross-repo oracle 4/0** — wasm4pm `graphlaw_vector_oracle`
  re-executed exit 0 (RECEIPTS wave 4, embedded output); closes L90's
  disk-BLOCKED oracle leg and L98's routed schema-v1 leg.
- **Chain integrity restored** — spc_court.sh append-only (L111);
  coordinator re-baseline head `2c316b02…`, `--check` exit 0 (L108,
  `designs/chain-resync-receipt.md`).
- Still open after wave 4: L105/L112/L102 receipts UNFILED (fixes on
  disk, reports loud-missing); L113-L116 UNFILED; advisories residual 3
  REQUIREMENTS-CHANGE (L103); js browser court needs Playwright Chromium
  (L107); wasm artifact re-pin pending reproducible SDK.

### Architecture doc

- `docs/ARCHITECTURE.md` — admission-law flow
  with verified file:line anchors, grouped module map, invariants, wasm
  unit state. Status: **CURRENT** (landed 2026-10-22, lane L104).

### TC-MEGA-FIBO-01 — Operation Global Tear-Up (setup phase, 2026-10-22)

Status: SETUP-COMPLETE-IMPLEMENTATION-PENDING. Lane L75.

Deliverables under
[mega/tc-mega-fibo-01/](mega/tc-mega-fibo-01/):

- `corpus/` — 100k-leg derivative tear-up corpus (smoke 1k + full 100k,
  .ttl/.trig/.nq) plus the hand-authored anti-vacuity twin
  `antivacuity-leg-41209-malformed.ttl`; deterministic regeneration
  proven (sha256 digests, README "Determinism invariant").
- `ontology/` — 7 SHACL shapes, 2 N3 rules, benign/mutated decimal
  twins, `gen_shapes.py`.
  census at spec time: 1 WITNESSED (M2 count class) / 7 UNVERIFIED
  (M1, M3, M4, M5, M7, M8, M9) / 1 REFUTED (M6).
  audit; U5 records the wasm artifact not linking (`rust-lld
  -lduckdb`).
  — 17 work orders (WO-01..WO-17), 6 NEW-MECHANISM class.

Critical dependency: WO-10 (wasm client sim, serves M7) is blocked on
the wasm32-wasip1 artifact actually linking — until then a loud typed
skip, never a silent pass.

No-Rust constraint honored: setup is python3-stdlib-only
(`generators/gen_corpus.py`); no src/ or tests/ files touched.

### Session design records — EH runner / integration prep (2026-10-22, lane L165)

Rows verified present on disk via `ls` before filing. Loud skips:
`wasmi-eh-research` (not landed — no file under `designs/`),
`g2-repin-prestage`, `p7-final-regen-prestage` (no files found); the
EH memo amendment is in-file (`../../archive/designs-2026-10-23/eh-runner-decision-memo.md:76`, L126
stand-down), not a separate record; the M/N receipt set is covered by
the `m-wave-verdicts` and N-lane rows below.

| Record | What it is | Status |
|---|---|---|
| designs/eh-runner-decision-memo.md | EH runner decision memo (+ L126 amendment): DuckDB wasm artifact carries a Tag section wasmi cannot host; recommends a route, decision required | DECISION REQUIRED (A1 refuted by amendment) |

### Session design records — post-L165 (2026-10-22, lane L170)

Each row verified present on disk via `ls`/`wc` before filing. Loud skips:
L163 (courts live in `tests/sql_rdf_dialect.rs`, no design doc); L167/L168
(no doc under `designs/`, no RECEIPTS.md entry found). Already indexed by
L165 (dedupe skips): `commit-drafts-validation.md` (L148),
`pure-artifact-witness.md`, base `mutation-court-full-receipt.md`,
`wasm4pm-fixtures-drift.md` (wave-3 bullet). The `wasmi-eh-research.md`
file has since landed (L165 recorded it as not-landed) — indexed below.

| Record | What it is | Status |
|---|---|---|
| designs/p7-final-regen-prestage.md | P7 pre-stage: final registry regen procedure, queued on the GO/NO-GO board row 47 | PRE-STAGE (not yet executed) |
| [designs/staleness-76-triage.md](designs/staleness-76-triage.md) | Staleness-gate residual triage: live count 91 (not 76); rule split R2=48/R3=15/R4=28 | TRIAGED |
| designs/repin-staging-gap.md | Re-pin staging gap: wasi-sdk-34 vs emsdk-built staging; nothing re-pinned, every claim receipt-cited | GAP DOCUMENTED |

### Session design records — post-L170 (2026-10-22, lane L171)

Each row verified present on disk via `ls -lt` + `test -f` before filing
(exits 0). Loud skips: `s2-provenance-census-verify` and
`s4-spc-design-verify` NOT landed (no file under `designs/`).

| Record | What it is | Status |
|---|---|---|

### Session design records — post-L171 (2026-10-22, lane L174)

Each row verified present on disk via `test -f` before filing (exit 0) and
deduped against L165/L170/L171 rows via grep: `commit-manifest.md` (L165),
`s1-surface-analysis-verify.md`, `s7-chain-board-check.md` (L171) already
indexed — dedupe skips. L171's loud skips (`s2-provenance-census-verify`,
`s4-spc-design-verify`) have **since landed** and are indexed below. No
new loud skips: no expected candidate was absent on disk; no S8/N17
outputs found under `designs/` in this batch.

| Record | What it is | Status |
|---|---|---|
| designs/surface-six-entry-patch.md | L172 staged patch expanding `src/registry/emit_json.rs` FILES const from 2 to 6 entries (closes the S1-confirmed 35.8% pin gap); DO-NOT-APPLY until quiescent window, sequenced after green compile and BEFORE any `graphlaw-registry --write` | STAGED (deferred, not applied) |

### V-L20 integration verification (2026-10-22, lane V-L20)

Live note — compile gates + fmt census over the V-L1..V-L19 touched surface
(git status snapshot, 76 .rs files tracked-modified + 3 untracked). Commands
run with `CARGO_TARGET_DIR=/tmp/gl-target --offline` under contention.

| Check | Command | Exit |
|---|---|---|
| graphlaw tests compile | `cargo check -p graphlaw --tests --offline` | 0 (Finished dev, 16.52s) |
| eyeron compile | `cargo check -p graphlaw-eyeron --offline` | 0 (Finished dev, 1.54s) |

**fmt verdict** (`rustfmt --check`, no repo rustfmt.toml; graphlaw files at
edition 2024, eyeron at 2021):

- graphlaw-eyeron (`crates/graphlaw-eyeron/src/parser.rs`,
  `reasoner.rs`): **CLEAN** at edition 2021 (both).
- graphlaw (edition 2024): 55 of 69 files FAIL `rustfmt --check`.
  Classified against HEAD baselines (`git show HEAD:file` snapshots):
  53 were already fmt-dirty at HEAD (**pre-existing**, repo is not
  fmt-clean at HEAD), 16 HEAD-clean files are still clean now. Net
  regressions vs HEAD (session-introduced, 3):
  `src/bin/smon_broaden_topic.rs`, `src/bin/smon_transcript_to_turtle.rs`,
  `tests/common/mod.rs`. Net improvements (4): `src/dialect.rs`,
  `src/query_ledger.rs`, `src/rules/mod.rs`, `src/sql_ast.rs` (dirty at
  HEAD, clean now). New untracked files dirty:
  `examples/refusal_probe.rs`, `tests/smon_cli_court.rs`,
  `tests/wasm_duckdb_op_wired.rs`.

Verdict: compile gates PASS (exit 0 both); fmt is NOT clean repo-wide and
is pre-existing at HEAD, not a lane regression; 3 tracked files regressed
this session and 3 new files landed unformatted. No code edited by this
lane; nothing committed.

### Operator brief — file splits, agent admission, three partitions (2026-10-22)

The brief is filed, reconciled to `wc -l` on this tree, in
designs/core-team-decomposition.md and
designs/deterministic-agent-admission.md.
The moonshot catalog rows are in [designs/INDEX.md](designs/INDEX.md).

Eight trees are `mod`-wired. Parents stayed `foo.rs` with children in
`foo/`. `*_split` folders are deleted. Public paths are unchanged. Receipt
field order and the SQL receipt formula were not edited.

| Cut | On disk | Still open |
|---|---|---|
| Reasoner | `limits.rs`, `builtins/{list,log,math,string}.rs`, `solve_context.rs` (dead draft) | `unify.rs`, `tabled.rs`, `BuiltinPredicate`, replacing the long `solve_backward_goal` parameters. `reasoner.rs` is still 4248 lines and still allows `clippy::too_many_arguments`. `builtins/mod.rs` is 946. |
| Parser | `cursor.rs` (438), `grammar.rs` (1500), entry points in `parser.rs` (342) | Further cut of `grammar.rs` |
| Receipt chain | `frame.rs`, `record.rs`, `store.rs`; epoch and validator stayed in the parent | — |
| Registry | `definitions.rs`, `emit_json.rs`, `emit_ttl.rs`; parent is the 130-line façade | Surface digest not re-run for this filing. `tests/registry_hash_invariance.rs` does not exist. |
| ABI | `ops/{meta,parse_convert,query_engines,governance}.rs` | `lease.rs` is a module comment. `LeaseContext` is not compiled and no op admits it up front. |
| Dialect | `refusal.rs`, `sniff.rs` | Sniff was not rewritten to take `&[u8]` |
| Query ledger | 29-line façade, `batch.rs`, `storage.rs` | `LedgerBatch::fold_entry` was not added. [cold-path-ledger.md](designs/cold-path-ledger.md) |
| Not split | `law.rs` 1021, `evidence_ring.rs` 914, `duckdb_eval.rs` 1681, `w3c_rdf.rs` 1293 | Chicago fixtures not extracted (`tests/common/chicago_fixtures.rs` absent) |

Switchover checks, both exit 0: `cargo check -p graphlaw --tests --offline`
and `cargo check -p graphlaw-eyeron --offline` (`CARGO_TARGET_DIR=/tmp/gl-target`).
Workspace clippy, `cargo test --workspace`, and the partition court were not
re-run after the move.

The same brief's agent-admission half is design only. There is no Rust-review
ontology, no semantic-ticket graph, and no receipt issued for a source edit.
The measured partition court remains the N21 receipt: G3 FAIL, 44 unadmitted
projection leaks, handwritten/derived ratio 1.1425. Those leak paths still
name the deleted `*_split` folders; the path map is the switchover note in
code-partition-census.md. The ratio has
not inverted. G5 prints it and does not call it a pass.

### Session lithography (2026-10-22)

Filed at designs/session-lithography.md.
A restart loads the Lithography section of `CLAUDE.md` and
`.claude/hooks/lithography_card.py` (`SessionStart`:
`startup|resume|clear|compact`).

| Matcher | Scripts | Hit |
|---|---|---|
| `Agent` | `ban_generic_agents.py` | `GENERIC_AGENT_REFUSED`, exit 2 |
| `EnterWorktree` | `topology_guard.py` | `REFUSED(topology)` |
| `Bash` | `topology_guard.py`, then `disk_guard.py` | worktree/stash refusal, then `DISK-REFUSED` under 20 GiB |
| Edit / Write, PreToolUse | `manifest_guard.py`, then `lithography_guard.py`, then `graphlaw_admission_guard.py` | `MANIFEST-REFUSED` on root `Cargo.toml` / `deny.toml`; `LITHO-REFUSED` on generated docs, registry artifacts, partition pin files, and `_split/` paths; `shacl` admission of an `act:ToolCall` against `.claude/policy/graphlaw-seed-shapes.ttl` |
| Edit / Write, PostToolUse | `graphlaw_receipt_attest.py` | one `receipts/transition-*.json` and a `CHAIN.blake3` fold after a mutating edit with a readable file |

`lithography_guard.py` fails closed on unreadable stdin (exit 2). The other
Python guards stay fail-open, including admission when `graphlaw-run`,
`priv/graphlaw.wasm`, or the seed shapes are missing. Admission calls
`graphlaw-run` for the `shacl` op. It does not call the wasm `hooks` op.
Axiom `.rs` sources stay editable. A deny is not a receipt: the receipt
hook runs after the tool. `src/abi/lease.rs` is not turned on.
`.claude/policy/fs-boundary-shapes.ttl` is not the graph the hook sends.

### Window files (index re-sync, 2026-10-22, lane D3)

Re-synced against `ls docs/jira/v26.10.22/`: every ticket and file named above
exists on disk. Link-resolve claim, re-verified 2026-10-24 (closeout lane):
182 unique relative `.md` links in this index, 182 resolve, 0 missing after
the :420 stale-row repair (`RELEASE-NOTES-DRAFT-26.10.21.md` did not exist;
index now points at `RELEASE-NOTES-26.10.22.md`) — verified by `test -f` per
link. The 2026-10-22 "all 61 links resolve" figure was true for that
sync's link set and is superseded by this count. Top-level files present on
disk but not named anywhere
above:

- [`CHARTER-DFLSS.md`](CHARTER-DFLSS.md) — DFLSS charter

(`DISSERTATION.md`, `PRD-P28-SIL.md`, `RECEIPTS.md`, `THESIS.md`,
`SQLRDF-01..20.md`, `mega/`, and `designs/` are indexed above.)

### Designs appendix — records not named in the body (index re-sync, lane D3)

76 of the 133 files under `designs/` were not named anywhere above. Titles
below are each file's own H1 (verified via `head -1`); statuses are not
re-adjudicated here — see [designs/RECEIPTS.md](designs/RECEIPTS.md) and
[RECEIPTS.md](RECEIPTS.md) for verdicts.

| Record | H1 (verbatim) |
| --- | --- |
| claude-policy-commit-note.md | .claude/ Commit Policy — GO/NO-GO Operator Decision 3 |
| commit-receipt-provenance-gap.md | Commit receipt — staged provenance-gap section (lane L176) |
| [gate-rule-changes-receipt.md](designs/gate-rule-changes-receipt.md) | Receipt: doc_staleness_gate rule changes (L168) |
| R2-capability-routes.md | R2 — capability route table (design record) |
| [rules-pruner-design.md](designs/rules-pruner-design.md) | C23 Harness Pruner — Design (L34) |
| [stale-claims-audit.md](designs/stale-claims-audit.md) | Stale-claims audit — v26.10.22 (skeptic lane, read-only) |
| [t4-refusal-reaudit.md](designs/t4-refusal-reaudit.md) | T4 refusal-register re-audit (post-switchover drift check) |

### Index re-sync — lane R4 (2026-10-22)

Re-synced against disk (D3 discipline: `ls` + `head -1` per file before
filing). Artifacts landed since the D3 sync that were not named anywhere
above:

**Archive** (docs/archive/) — moved by the
coordinator (`git mv`), historical narrative and receipts retired out of the
live tree:

- `THESIS.md` — moonshot thesis (verbatim body), moved from `docs/THESIS.md`
- `UNIFIED_DISSERTATION.md` — earlier unified dissertation draft, moved from `docs/UNIFIED_DISSERTATION.md`
- `EXTRACTION.md` — GraphLaw extraction receipt
- `branch-disposition.md` — branch disposition record
- `receipt-2026-10-09-receipt-chain-and-refuse-hooks.md` — session receipt: receipt-chain port + refuse hooks (2026-10-09)
- `receipt-2026-10-10-publish-dod.md` — v26.10.10 publish Definition-of-Done receipt (skeleton)
- `README.md` — archive readme

**New gate scripts** (`scripts/`, census was 5 at wave 3, now 19):

| Script | Header-verbatim purpose |
| --- | --- |
| [chain_flag_court.sh](../../../scripts/chain_flag_court.sh) | non-vacuity court for `scripts/receipt_chain_sync.sh` (lane M14) |
| [count_gate.sh](../../../scripts/count_gate.sh) | stale "N passed; 0 failed" detector for cheap-allowlist courts |
| [exit_capture_check.sh](../../../scripts/exit_capture_check.sh) | M11 self-test for the Exit-capture law (`.claude/agents/court-runner.md`) |
| [fleet_cleanup.sh](../../../scripts/fleet_cleanup.sh) | advisory disk reclaim for lane build roots (target-* dirs by mtime) |
| [machine_shaped_gate.sh](../../../scripts/machine_shaped_gate.sh) | MACHINE-SHAPED gate (K1 side-arm, lane M09) |
| [partition_court.sh](../../../scripts/partition_court.sh) | Chicago court over the three-partition code census (`courts/code-partition/manifest.json`) |
| [receipt_chain_sync.sh](../../../scripts/receipt_chain_sync.sh) | BLAKE3 sidecar chain over `receipts/` (lane L52) |
| [receipt_replay_court.sh](../../../scripts/receipt_replay_court.sh) | K7-class court: fabricated receipt (never-executed output) must FAIL verification |
| [shape_mutation_court.sh](../../../scripts/shape_mutation_court.sh) | TC-MUT-01 shape-flip court (anti-vacuity over `ontologies/sql-ast.ttl`) |
| [wasm_audit.sh](../../../scripts/wasm_audit.sh) | W04 wasm artifact size ceiling + header court |
| [wasm_unknown_unknown_smoke.sh](../../../scripts/wasm_unknown_unknown_smoke.sh) | W03 falsifier smoke for wasm32-unknown-unknown (G13) |
| [witness_gate.sh](../../../scripts/witness_gate.sh) | M02: a factual build/run witness in `docs/**/*.md` must cite an existing file |
| [partition_court_selfcheck_g1.py](../../../scripts/partition_court_selfcheck_g1.py) / [g2.py](../../../scripts/partition_court_selfcheck_g2.py) | G1/G2 self-check courts for `partition_court.sh` |
| [gen_wasm4pm_graphlaw_vectors.py](../../../scripts/gen_wasm4pm_graphlaw_vectors.py) | wasm4pm vector generator (cross-repo fixtures) |

**New courts in [`tests/`](../../../tests/):**

- [`k5_hostile_sqlrdf.rs`](../../../tests/k5_hostile_sqlrdf.rs) — lane K5 adversarial court: literal/identifier smuggling on the sql-rdf path ("cannot corrupt or leak data")
- [`k6_hostile_lease.rs`](../../../tests/k6_hostile_lease.rs) — lane K6 adversarial court: authority-model forgery attempts flip to typed refusals
- [`dl_dualroute_vector_pin.rs`](../../../tests/dl_dualroute_vector_pin.rs) — vector-byte pin for the DL dual-route generator (K10 defect class: silent provenance-field drift)
- [`ocel_export_court.rs`](../../../tests/ocel_export_court.rs) — OCEL v2 export court (lane PM4): real law session → real SQLite database readback
- [`law_state.rs`](../../../tests/law_state.rs) — (extended) law-state transitions over the innovation reference pack: Turtle → N3 derivation → SHACL admission → SPARQL CONSTRUCT, every step receipted

**External black-box courts** ([`tests_py/`](../../../tests_py/test_blackbox_ops.py)):

- [`conftest.py`](../../../tests_py/conftest.py) + [`test_blackbox_ops.py`](../../../tests_py/test_blackbox_ops.py) — external black-box court for the graphlaw binaries

**CHANGELOG-era top-level files:**

- [`CHANGELOG.md`](../../../CHANGELOG.md) — Keep a Changelog 1.1.0 entries (per-crate versioning policy stated once)
- [`GOALS.md`](../../../GOALS.md) — GraphLaw production goals + invariants
- [`MIGRATION.md`](../../../MIGRATION.md) — migration guide 26.9.28 → 26.10.22
- [`SECURITY.md`](../../../SECURITY.md) — security policy (supported versions)
- [`CONTRIBUTING.md`](../../../CONTRIBUTING.md) — contributing guide (Rust 1.96 pinned, merge-only)
- `SPLIT-NOTES.md` — Lane L46 phase-4 extraction notes (UNWIRED by directive)
| SQLRDF-25-RG | SQLRDF-25 remaining deferred gaps ledger (star modifiers, COLUMNS, PIVOT/UNPIVOT, composite literals, JSON operators) | Open (rows 1-5 LANDED in `88d6665`) | Five actionable rows split from the parent's open rows; rows 1-5 court-witnessed in commit `88d6665`: pivot 3/3, star_modifier 4/4, columns_projection 4/4, json_operator 2/2, composite_literal 4/4 (17 passed / 0 failed, exit 0; arms 48-52 folded; shapes pinned `ontologies/sql-ast.ttl` blake3 `e4ee5661…39297b`) — receipt `receipts/2026-10-23-sqlrdf25-close.md`. Open rungs not witnessed there: wasm one-unit court, shape mutation court | [SQLRDF-25-remaining-gaps](SQLRDF-25-remaining-gaps.md) |
| ADR-CANONARROW | Canonical Arrow IPC dual-witness columnar egress (operator-approved 2026-10-23) | Absorbed into the v26.10.22 release boundary (26.10.23 moniker deferred) | Native in-WASM Arrow IPC streams + versioned `receipt_v2` dual-witness envelope (`receipt_v1` verifiers untouched; `R_receipt_v2 = BLAKE3("GLAW_RECEIPT_V2" ∥ H_query ∥ H_pre ∥ H_post ∥ H_triples ∥ H_columnar ∥ epoch)`); projection-order preservation; guest view-pinning. Landed leg: receipt_v2 court 6/6 (W60) with corrected goldens; CHAIN re-baseline documented (terminal receipt §3, `receipts/2026-10-23-chain-rebaseline-note.md`) | [ADR-canonarrow-dual-witness-egress](ADR-canonarrow-dual-witness-egress.md) |

### Status refresh 2026-10-23 (EH-closure wave)

Per-ticket standing as of the EH-closure wave. Receipts:
[RECEIPTS.md](RECEIPTS.md) lines 1169-1340 and the wave receipt
receipts/2026-10-23-eh-closure-wave.md.

- **SQLRDF-21** — implementation essentially complete: gates removed,
  wasmtime host, session surface, doc sequences wired. Op courts 16/0.
  Open: the artifact trap (SQLRDF-24) → re-pin → in-wasm courts A/B.
- **SQLRDF-22** — G1/G3/G4/G6 LANDED with courts; the G3 egress-digest
  receipt field landed. G5 designed and filed forward as SQLRDF-23.
- **SQLRDF-23** — compile + eval + wire LANDED. Court counts:
  `sql_ast` 32/0, `duckdb_eval` 18/0, `duckdb_op` 16/0. SHACL
  sequence vocabulary landed.
- **SQLRDF-24** — trap root-caused to the atexit writer/reader skew.
  Exoneration matrix: EH flavor, LTO, table encoding, gc-sections,
  compat archive. libc++abi candidate rebuild + disassembly in
  flight; re-pin still held.

### Status addendum 2026-10-23 (ledger-sync wave, W60-W64)

Receipts: [RECEIPTS.md](RECEIPTS.md) §"2026-10-23 ledger-sync wave".
- **Banking (DvP)** — 4/4 GREEN witnessed (`receipts/2026-10-23-banking-4of4-witness.md`,
  historical b3sum `f3570f15d3d98ad52af83187404d8cdf99b1ad7f36ddcdac4cb9f55d3f77920d`);
  BCBS/Rehypo/UBO 3/3 each as-of prior witness.
- **receipt_v2** — court 6/6 GREEN with corrected goldens (W60); trust_attest
  full file 12/0.
- **Pack gates** — cargo package dry-run 360 files (corrected from 363, W61);
  npm N/A typed at root (no `package.json`); amortization bench excluded from
  the published package (cargo warning embedded in W61).
- **Workload amortization** — bench + anti-vacuity court files landed; court
  RED at `tests/amortization_court.rs:155` (left: 1, right: 0; 64.71s, W62
  OPEN — owner assignment pending). Measurement pending.
- **Whitepaper** — claims audit recorded (W63); one count corrected (363→360).
- **SQLRDF-27** — **Closed** (supersedes the "verdict pending relink"
  status above): FIBO 4/0 + lifecycle 3/0 witnessed on the wrap-enabled
  artifact (`receipts/2026-10-23-sqlrdf27-verdict.md` lifecycle
  amendment); ticket flipped receipt-gated; final artifact identity is
  the settle-commit pin `472d5557…bf80ea` (58,121,500 B, commit
  `d9cab2f`). Owed non-gating rungs remain open.

### WRAP — v26.10.22 closeout (2026-10-24)

Closeout lane receipt: this section is the window's integration flip pass.
Every table cell above stays as-filed (this index is append-only; prior
sections were never rewritten and none is rewritten now) — the WRAP table
below is the authoritative per-ticket end-state, each row citing its
receipt path. Ground ledger (G1 DISCHARGED / G2-G3 REMAINING / G4 SPLIT):
GO-NO-GO-v26.10.22.md §A (v5).
OD-4 memo filed + OD-6 anomaly recorded:
designs/OPERATOR-DECISIONS.md — both
DECISION lines left BLANK for the operator (runbook S3/S4:
`docs/jira/v26.10.23/OPERATOR-RUNBOOK-v26.10.22-PUBLISH.md`). Residuals
hand off to the v26.10.23 window: mutation-court verdict (G2), FULL battery
on the shipped pin (G3), R-1 rungs (G4/R-1), SPC FULL + OD-4 letter, the two
operator credential acts (PUBLISH-STATUS rows 2/6), registry
receipt-surface resync, refusals arms 53-56 register rows,
docs/reference-generated refresh.

End-state: **15 Closed / 13 Deferred / 1 Withdrawn / 1 Absorbed / 2 program
no-flip** (32 rows). Count law note: the dispatch projected 18/10; rows
SQLRDF-22/23/25 fall to DEFAULT-DEFERRED under their own decision rules on
disk evidence (below), and a conditional row is never silently Closed —
the honest count governs.

| Row | Was | End state | Receipt / decision rule |
|---|---|---|---|
| SQLRDF-01 | Open | **Closed** | Compiler-side hop closed, dialect court 33/0 exit 0 — `receipts/sql-rdf-dialect-2026-10-22.md` (INDEX row) |
| SQLRDF-02 | Open | **Closed** | Shapes landed + conformance 7/0 — `receipts/sql-rdf-conformance-2026-10-22.md`; dialect 33/0 `receipts/sql-rdf-dialect-2026-10-22.md` |
| SQLRDF-03 | Open | **Closed** | HAVING/ORDER BY/from compiler, courted 33/0 final 2026-10-22 — same dialect receipt |
| SQLRDF-04 | Open | **Closed** | Parameterized MERGE, chicago_validation 9/9 + receipt courts — INDEX row |
| SQLRDF-05 | Open | **Closed** | Expression-level graphs, 6/0 + `.gram` env exception cleared — `receipts/duckdb-statement-kinds-2026-10-22.md` |
| SQLRDF-06 | Open | **Closed** | Register-aligned REFUSAL-02 carrier, courted branches — `tests/sql_rdf_dialect.rs` |
| SQLRDF-07 | Open | **Closed** | Select-list scope integrity, 9/9 + evasion/join courts — INDEX row |
| SQLRDF-08 | Withdrawn | **Withdrawn** (no change) | Contradicts the one-unit law; `REFUSAL-04` is the non-document-backed reader (preamble) |
| SQLRDF-09 | Open | **Closed** | Canonical-AST formula hash + dialect 33/0 — INDEX row |
| SQLRDF-10 | Open | **Closed** | §1.4 conformance note filed, attestation courts 10/0 (Σ_PQC achievement itself stays a program-level disclosure, refusals register) |
| SQLRDF-11 | Open | **Closed** | §6.2 courts on compiled AST graphs over real frozen-duckdb — INDEX row |
| SQLRDF-12 | Open | **Closed** | `docs/reference/sql-rdf-rec.md` filed + citation gates — INDEX row |
| SQLRDF-13 | Open (follow-up) | **Deferred** | Shapes withheld at `ontologies/sql-ast.ttl:315-330`; successor work, not v26.10.22 acceptance |
| SQLRDF-14 | Open (follow-up) | **Deferred** | PEG leaf re-parenting; follow-up |
| SQLRDF-15 | Open (follow-up) | **Deferred** | sf=0.01 divergence unadjudicated (frozen pin sources absent) |
| SQLRDF-16 | Open (follow-up) | **Deferred** | Moonshot backlog, BLOCKED-BY integration commit |
| SQLRDF-17 | Open (follow-up) | **Deferred** | C2 hole P1; `src/duckdb_op.rs:198-219` raw-SQL arm still open |
| SQLRDF-18 | Open (successor program) | **Deferred** | TC_HOT_FULL variance 26.91% + SPC Cpk legs open (OD-4 class) |
| SQLRDF-19 | Open (follow-up) | **Deferred** | Untested feature cells; release-only combos |
| SQLRDF-20 | Open (follow-up) | **Deferred** | BEAM hot-upgrade protocol |
| SQLRDF-21 | Open (courts landing this wave) | **Closed** | EH-closure wave: gates removed, wasmtime host, session surface, op courts 16/0, lifecycle 3/0 amendment, re-pin landed — `receipts/2026-10-23-eh-closure-wave.md` §8; W13→W40 supersession RECEIPTS.md:1539 |
| SQLRDF-22 | Open | **Deferred** (rule) | Rule: Closed iff all six audited gaps land typed + courted. Observed: G1/G3/G4/G6 landed with courts (INDEX status refresh 2026-10-23; G6 receipted `docs/jira/v26.10.22/RECEIPTS.md:1883-1891`), G5 filed forward as SQLRDF-23, **G2 COPY egress policy not witnessed landed** (no COPY policy arm in `docs/refusals.md`, no COPY-policy court) → DEFAULT DEFERRED |
| SQLRDF-23 | Open | **Deferred** (rule) | Rule: Closed iff court plan (a)-(f) closes. Observed: compile+eval+wire LANDED with courts (`sql_ast` 32/0, `duckdb_eval` 18/0, `duckdb_op` 16/0; RECEIPTS.md:1723-1726) but mutation-court inversions for (b)/(f) still pending and the last suite run was DISK-BLOCKED (SQLRDF-23.md "Implementation progress") → DEFAULT DEFERRED |
| SQLRDF-24 | Open | **Closed** (rule met) | Rule: trap root-caused + fixed + artifact links/executes/pins. Evidence: CONFIRMED + FIXED (atexit member swap + reactor-contract completion), re-pin superseded the hold, lifecycle 3/3 — `receipts/2026-10-23-eh-closure-wave.md` §8; RECEIPTS.md:1516-1521; shipped pin `58a44095` 4-surface |
| SQLRDF-25 | Open | **Deferred** | Rows 1-5 landed in `88d6665` (17/0, `receipts/2026-10-23-sqlrdf25-close.md`) but window OVER/frames, QUALIFY, CTEs not witnessed landed; remainder tracked in SQLRDF-25-RG |
| SQLRDF-27 | Closed 2026-10-23 | **Closed** (no change) | Close condition met (`receipts/2026-10-23-sqlrdf27-verdict.md` + lifecycle amendment); owed rungs = GO-NO-GO G4/R-1, not this ticket |
| SQLRDF-28 | Open | **Deferred** | Acceptance = mutation rerun moves all 11 survivors to caught; kill court landed 10/0 (`b499fb7`, `tests/abi_response_builders_court.rs`) but the sweep has no completed verdict (ENOSPC; GO-NO-GO G2) — closes with the G2 receipt |
| PRD-P28-SIL | Open | **program no-flip** | Filed verbatim as the operative PRD for the successor program; the register rows are the flip surface, not this filing row |
| DISSERTATION | Open (filed) | **program no-flip** | Filed verbatim; chapter→artifact map is the standing record |
| SQLRDF-25-RG | Open (rows 1-5 LANDED) | **Deferred** | Open rungs: wasm one-unit court, shape mutation court (INDEX row) |
| ADR-CANONARROW | Absorbed | **Absorbed** (no change) | Absorbed into the v26.10.22 release boundary (26.10.23 moniker deferred); receipt_v2 court 6/6 (W60) |
