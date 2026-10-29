# GraphLaw docs index — by reader intent

The intent map for `docs/`. The Diataxis hub ([index.md](index.md)) organizes by
doc type; this file organizes by what you came to do, and covers the half of the
tree the hub does not: conformance text, courts/gates/scripts, the wasm
boundary, and the milestone program records under `jira/`. One entry per doc
with a freshness verdict. All paths verified present on 2026-10-22.

Freshness vocabulary: **CURRENT** (maintained against the live tree) ·
**HISTORICAL-MARKED** (carries an explicit dated/historical banner) ·
**PENDING** (typed open items inside, or stale-but-unmarked — noted per entry).

## Getting started

- [quickstart.md](quickstart.md) — runnable quickstart: plan → admission →
  receipt chain → first refusal. CURRENT.
- [index.md](index.md) — Diataxis hub (tutorials/how-to/reference/explanation);
  the type-based map complementing this file. CURRENT.
- [how-to/run-the-wasm-module.md](how-to/run-the-wasm-module.md) — build the
  WASI module, drive the JSON ops from any host, limits. CURRENT.
- [how-to/authorize-with-signed-leases.md](how-to/authorize-with-signed-leases.md)
  — Ed25519 leases, ceilings, trusted clocks, refusals. CURRENT.
- [how-to/verify-without-trusting-the-operator.md](how-to/verify-without-trusting-the-operator.md)
  — `graphlaw-verify` offline replay: flags, exit codes, tamper refusals. CURRENT.
- [how-to/use-knowledge-hooks.md](how-to/use-knowledge-hooks.md) — `kh:` packs,
  fixpoint materialization, refuse effect. CURRENT.
- [how-to/how-to-edge-ingestion-and-compaction.md](how-to/how-to-edge-ingestion-and-compaction.md)
  — frozen-duckdb ingestion, SQL-in-RDF, compaction proofs. CURRENT.
- [how-to/use-from-javascript.md](how-to/use-from-javascript.md) — npm package
  in Node/browsers over the JSON ABI. CURRENT.
- [how-to/first-in-wasm-sql-lifecycle.md](how-to/first-in-wasm-sql-lifecycle.md)
  — end-to-end SQL-in-wasm walkthrough: build, session, CREATE/INSERT/
  SELECT, stream, close, typed DELETE refusal. CURRENT.
- [how-to/persistent-duckdb-sessions.md](how-to/persistent-duckdb-sessions.md)
  — named DuckDB sessions over the op-15 wire; reuse connections across
  exec frames. CURRENT.
- [how-to/document-sequences.md](how-to/document-sequences.md) — run several
  admitted SQL statements in one wire call, receipt per member. CURRENT.
- [api-stability.md](api-stability.md) — semver policy, `#[non_exhaustive]`
  list, MSRV, frozen ABI_VERSION 1. CURRENT.

## Architecture

- [abi-reference.md](abi-reference.md) — JSON envelope, all ops, error kinds;
  prose companion to the registry. CURRENT.
- [capability-registry.md](capability-registry.md) — machine-readable op
  surface and digest rule (`graphlaw-registry --check`). CURRENT.
- [DIALECTS.md](DIALECTS.md) — dialect routing table; source of truth
  `src/dialect.rs`. CURRENT.
- [ARD-26.10.25-DIALECTS.md](ARD-26.10.25-DIALECTS.md) — Architectural Review Document:
  RDF Dialects, Formal Semantics & Projection Mechanics (Sovereign Lattice, PurRDF 5-table Parquet,
  Loss Matrix, SHACL/Datalog relational pushdown). CURRENT.
- [refusals.md](refusals.md) — typed refusal register: codes, `details`
  fields, remediation. CURRENT.
- [reference/refusal-catalog.md](reference/refusal-catalog.md) — user-facing
  refusal view: envelope shape, per-code cause and fix; normative register is
  `refusals.md`. CURRENT.
- [reference/runner-exit-codes.md](reference/runner-exit-codes.md) — typed
  exit codes for `graphlaw-run` / `graphlaw-run-eh`, diagnostics, recovery.
  CURRENT.
- [explanation/receipt-fields.md](explanation/receipt-fields.md) — receipt
  fields, informal mapping to the portfolio receipt concept. CURRENT.
- [explanation/eh-closure-story.md](explanation/eh-closure-story.md) — why
  the wasm module was unhostable (legacy vs try_table EH) and how it closed.
  CURRENT.
- [Edge sovereignty & compaction](explanation/edge-sovereignty-and-zero-trust-compaction.md)
  — why local analytical compaction for edge/DDIL systems. CURRENT.
- [QUALIFICATION_ALGEBRA.md](QUALIFICATION_ALGEBRA.md) — standing as
  executable falsification, not assertion. CURRENT.
- EH/a1prime/dual-artifact design records, under
  [jira/v26.10.22/designs/](jira/v26.10.22/designs/):
  `eh-runner-decision-memo.md` (+ in-file L126 amendment),
  `a1prime-trytable-recipe.md`, `dual-artifact-policy-design.md`,
  `wasmi-eh-research.md` — the DuckDB wasm artifact's Tag section blocks
  wasmi hosting; decision memo, try_table rebuild recipe, Route C
  dual-artifact policy, and the wasmi EH feasibility research feeding the
  engine-tier decision. Design records, nothing landed. CURRENT
  (dated 2026-10-22).
- [reference/knowledge-hook-projection.md](reference/knowledge-hook-projection.md)
  — powerless state → hook projection contract (authority NONE). CURRENT.
- [qme-1-reference.md](qme-1-reference.md) — QME-1 participation, pinned to
  chatman-ecosystem@88b276f. CURRENT.
- [archive/THESIS.md](archive/THESIS.md) — mathematical-foundations essay
  (sheaves, variational law calculus), 2026-10 provenance snapshot. ARCHIVED:
  superseded by [jira/v26.10.22/THESIS.md](jira/v26.10.22/THESIS.md).
- [archive/UNIFIED_DISSERTATION.md](archive/UNIFIED_DISSERTATION.md) —
  doctoral-essay predecessor of THESIS.md. ARCHIVED: aspirational PQC claims
  unreceipted (Z19 finding); superseded by
  [jira/v26.10.22/DISSERTATION.md](jira/v26.10.22/DISSERTATION.md).

## Conformance

- [reference/sql-rdf-rec.md](reference/sql-rdf-rec.md) — the sql-rdf
  Recommendation conformance text for v26.10.22 (pasted text; its W3C status
  claims are document text). Cite by section. CURRENT.
- [reference-generated/sql-rdf-conformance.md](reference-generated/sql-rdf-conformance.md)
  — GENERATED clause-by-clause conformance report (regen via the
  `sql_rdf_conformance` court); never hand-edit. CURRENT (regenerated).
- [CHICAGO_N3_DATALOG.md](CHICAGO_N3_DATALOG.md) — bounded N3 + Datalog
  compliance court (v26.10.1 smoke set). CURRENT.
- [CHICAGO_FIBO.md](CHICAGO_FIBO.md) — FIBO court: authority NONE,
  consequence EVIDENCE_ONLY, generated from RDF. CURRENT.
- [CHICAGO_CROSS_AUTHORITY.md](CHICAGO_CROSS_AUTHORITY.md) — cross-authority
  court, same standing vocabulary. CURRENT.
- [reference-generated/reference.md](reference-generated/reference.md) —
  GENERATED module/symbol reference (regen:
  [reference-generated/README.md](reference-generated/README.md)). CURRENT.
- [reference-generated/explanation.md](reference-generated/explanation.md),
  [reference-generated/how_to.md](reference-generated/how_to.md) — GENERATED
  doc projections. CURRENT (regenerated).
- [reference/typescript-api.md](reference/typescript-api.md) — npm package
  exported surface, types, error mapping. CURRENT.

## Operations (courts, gates, scripts)

Entry points under `scripts/`, all verified present: `mutation_court.sh`,
`shape_mutation_court.sh`, `spc_court.sh`, `doc_staleness_gate.sh`,
`receipt_chain_sync.sh`, `wasm_audit.sh`, `wasm_unknown_unknown_smoke.sh`,
`gen_w3c_manifest.sh`, `gen_wasm4pm_graphlaw_vectors.py`. The test ladder and
exit discipline live in the root `CLAUDE.md` Commands section.

- [jira/v26.10.22/RECEIPTS.md](jira/v26.10.22/RECEIPTS.md) — one row per
  ticket: gate command, latest verdict, as-of timestamp. CURRENT (ledger).
- [jira/v26.10.22/RUST-4-PHASE2-HARDENING.md](jira/v26.10.22/RUST-4-PHASE2-HARDENING.md)
  — evidence_ring.rs concurrency-audit backlog, Status: OPEN. CURRENT
  (open backlog).
- [jira/v26.10.22/designs/INDEX.md](jira/v26.10.22/designs/INDEX.md) — design
  and receipt index for the 40+ files under `designs/`. CURRENT.
- [jira/v26.10.22/INDEX.md](jira/v26.10.22/INDEX.md) "post-L165" section —
  gate/triage receipts: `staleness-76-triage.md` (live staleness findings 91),
  `registry-drift-scope.md`, `repin-staging-gap.md` (wasi-sdk-34 vs emsdk
  staging), `spc-red-remediation-design.md`, amended
  `mutation-court-full-receipt.md` / `tc-hot-full-receipt.md`. CURRENT.

## Wasm boundary

- [WASM.md](WASM.md) — wasm runbook: build, verify, embed, upgrade
  `graphlaw-wasm`. CURRENT.
- [wasm-integration.md](wasm-integration.md) — integration reference: WASI
  shell vs hand-written safe core, re-read from `wasm/src/`. CURRENT.
- [wasm-duckdb-integration.md](wasm-duckdb-integration.md) — GWS-1..12 wave
  ledger; GWS-5/6/9/10/11/12 landed as of 2026-10-21, others still typed
  `PENDING-GWS-n`. PENDING (typed pending waves remain).
- [wasm-duckdb-op15-design.md](wasm-duckdb-op15-design.md) — op-15 `duckdb`
  design; dispatch landed (`src/duckdb_op.rs`: exec/stream/close, `DuckDb`/
  `ResourceLimit` refusals); registry-table + manifest digest bump pending.
  PARTIAL.
- [how-to/run-the-wasm-module.md](how-to/run-the-wasm-module.md) — host-side
  run guide (also under Getting started). CURRENT.

## Legal

- [../LICENSE.md](../LICENSE.md) — P28 Sovereign Invariant License
  (P28-SIL) v2.1.0; the sole license file at repo root. CURRENT.

## Program records (jira)

Active window: `jira/v26.10.22/`. Older windows: `sjira/v26.10.8/`.

- [jira/v26.10.22/INDEX.md](jira/v26.10.22/INDEX.md) — ticket register:
  SQLRDF-01..22, all Open except SQLRDF-08 (withdrawn); SQLRDF-21
  (EH closure / in-module duckdb) and SQLRDF-22 (compiler gaps) open.
  CURRENT.
- [jira/v26.10.22/CHARTER-DFLSS.md](jira/v26.10.22/CHARTER-DFLSS.md) —
  operator charter; header admits dispatch-by-summary, verbatim text not on
  disk. PENDING (self-declared UNVERIFIED charter text).
- [jira/v26.10.22/PRD-P28-SIL.md](jira/v26.10.22/PRD-P28-SIL.md) — P28-SIL
  PRD, verbatim operator text, Approved for Engineering & Integration. CURRENT.
- [jira/v26.10.22/DFLSS-DOSSIER.md](jira/v26.10.22/DFLSS-DOSSIER.md) —
  industrial handover dossier (subject `ed3be81` + working set). CURRENT.
- [jira/v26.10.22/THESIS.md](jira/v26.10.22/THESIS.md) — state-of-SQL-RDF
  thesis, written against `ed3be81`. CURRENT.
- [jira/v26.10.22/DISSERTATION.md](jira/v26.10.22/DISSERTATION.md) —
  admitted-computation dissertation (milestone program record). CURRENT.
- [sjira/v26.10.8/CAMPAIGN-RECEIPT.md](sjira/v26.10.8/CAMPAIGN-RECEIPT.md) —
  v26.10.8 campaign receipt (older window). HISTORICAL-MARKED (dated record).
- `archive/*` — branch-disposition.md, EXTRACTION.md,
  receipt-2026-10-09-receipt-chain-and-refuse-hooks.md,
  receipt-2026-10-10-publish-dod.md; all carry explicit historical banners.
  HISTORICAL-MARKED.

## Verification receipts

- All 40 doc paths above confirmed present via `ls`/`find` on 2026-10-22
  (full listings in session transcript); scripts verified via `ls scripts/`.
- Every markdown line in this file is ≤100 chars (checked with
  `awk 'length>100'` returning empty), with one exception: the Cross-repo
  references table rows (long single-line table syntax) exceed 100 by
  necessity — re-checked with `awk 'length>100'` on 2026-10-22, exactly 4
  lines, all table rows in that section.
- README.md pointer: LANDED — the README docs table (README.md:49-52) now
  carries `**Index** | [docs/INDEX.md](docs/INDEX.md)` at README.md:49,
  alongside the Diataxis hub link at line 45.

## Cross-repo references

The two-repo (and neighbor-repo) surface graphlaw participates in. Every row
verified on disk 2026-10-22 via `ls`/`grep` (receipts listed per row). Sync
direction: produced-by emits, consumed-by reads; commands run from the
producing repo unless noted.

| Artifact | Produced-by | Consumed-by | Sync command | Current state |
|---|---|---|---|---|
| Spec dossier: `docs/jira/v26.10.21/{WASM-SANDBOX-SPEC,GRAPHLAW-SPEC,COLDCHAIN-CASE-STUDY}.md` + `receipts/` (gl4/gl5 graphlaw courts, gws2/gws7/gws11 wasm courts, wasm1 otp29 port, coldchain receipts) | `/Users/sac/smalt` | graphlaw (downstream consumer evidence: wasm sandbox + graphlaw engine courts run against the graphlaw artifact) | none (read-only spec receipts; re-run smalt courts against a new graphlaw wasm sha) | VERIFIED — all three spec docs + receipts dir present 2026-10-22; receipts confirmed by filename grep |
| Frozen vector fixtures: `crates/wasm4pm-cognition/tests/fixtures/graphlaw/*.json` (50 vectors, ENCODERS.md, SCHEMA.md, REV-RELEASE-MAP.md) | graphlaw (`scripts/gen_wasm4pm_graphlaw_vectors.py`) | `/Users/sac/wasm4pm` (test court `tests/graphlaw_vector_oracle.rs`; `graphlaw_rev_ancestry_court` requires fixture rev to remain an ancestor of graphlaw main) | `python3 scripts/gen_wasm4pm_graphlaw_vectors.py` (regenerate at new graphlaw rev, then commit in wasm4pm) | VERIFIED — fixtures dir + generator script present; ancestry court named in REV-RELEASE-MAP.md |
| WASI FFI shell: `wasm/src/ffi.rs` ("Rendered by ggen (rust-wasi-wasmex-pack) from the wja: graph", header `wasm/src/ffi.rs:7`) | `/Users/sac/ggen-marketplace` pack `rust-wasi-wasmex-pack` (`templates/abi/ffi.rs.tmpl` + `queries/ffi.rq`) | graphlaw `wasm/src/ffi.rs` (generated artifact — regenerate, never hand-edit) | `ggen sync run` from the pack graph (no `ggen.toml` pin in graphlaw root yet — render is manual today) | VERIFIED — template, `ffi.rq` query, and rendered artifact all present; template header matches artifact header |
| frozen-duckdb crate | crates.io `frozen-duckdb` 2.0.2 | graphlaw `Cargo.toml` `[dependencies]` (`frozen-duckdb = { version = "2.0.2" }`) | none — not a path dep; `[patch.crates-io]` no longer exists (the purrdf family is git-sourced from the pinned [`seanchatmangpt/purrdf`](https://github.com/seanchatmangpt/purrdf) fork) | CLOSED — version-pinned to 2.0.2. Do not add a path `[patch]`. |
