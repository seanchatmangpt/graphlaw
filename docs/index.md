# GraphLaw docs index — by reader intent

The intent map for `docs/`: organized by what you came to do. It is the only index file in
`docs/` (README.md (doc table) link here). Every link below was resolved against
the tree on 2026-11-09; entries for files removed in the docs consolidation were dropped and
the three relocations are reflected (see Relocated). Freshness is stated per entry, not
recalled: counts and statuses live in the linked registers, not here.

Freshness vocabulary: **CURRENT** (maintained against the live tree) ·
**HISTORICAL-MARKED** (carries an explicit dated/historical banner) ·
**PENDING** (typed open items inside).

## Getting started

- [quickstart.md](quickstart.md) — runnable quickstart: plan → admission → receipt chain →
  first refusal. CURRENT.
- [how-to/run-the-wasm-module.md](how-to/run-the-wasm-module.md) — build the WASI module,
  drive the JSON ops from any host, limits. CURRENT.
- [how-to/authorize-with-signed-leases.md](how-to/authorize-with-signed-leases.md) — Ed25519
  leases, ceilings, trusted clocks, refusals. CURRENT.
- [how-to/verify-without-trusting-the-operator.md](how-to/verify-without-trusting-the-operator.md)
  — `graphlaw-verify` offline replay: flags, exit codes, tamper refusals. CURRENT.
- [how-to/use-knowledge-hooks.md](how-to/use-knowledge-hooks.md) — `kh:` packs, fixpoint
  materialization, refuse effect. CURRENT.
- [how-to/how-to-edge-ingestion-and-compaction.md](how-to/how-to-edge-ingestion-and-compaction.md)
  — frozen-duckdb ingestion, SQL-in-RDF, compaction proofs. CURRENT.
- [how-to/use-from-javascript.md](how-to/use-from-javascript.md) — npm package in
  Node/browsers over the JSON ABI. CURRENT.
- [how-to/first-in-wasm-sql-lifecycle.md](how-to/first-in-wasm-sql-lifecycle.md) — SQL-in-wasm
  walkthrough: build, session, CREATE/INSERT/SELECT, stream, close, typed DELETE refusal.
  CURRENT.
- [how-to/persistent-duckdb-sessions.md](how-to/persistent-duckdb-sessions.md) — named DuckDB
  sessions over the op-15 wire. CURRENT.
- [how-to/document-sequences.md](how-to/document-sequences.md) — several admitted SQL
  statements in one wire call, receipt per member. CURRENT.
- [how-to/egress-and-attach-policy.md](how-to/egress-and-attach-policy.md) — export tables
  and attach databases under policy. CURRENT.
- [how-to/release-publish.md](how-to/release-publish.md) — operator runbook for releasing and
  publishing. CURRENT.
- [api-stability.md](api-stability.md) — semver policy, `#[non_exhaustive]` list, MSRV, frozen
  ABI_VERSION. CURRENT.

## Architecture and reference

- [abi-reference.md](abi-reference.md) — JSON envelope, all ops, error kinds; prose companion
  to the registry. CURRENT.
- [capability-registry.md](capability-registry.md) — machine-readable op surface and digest
  rule (`graphlaw-registry --check`). CURRENT.
- [DIALECTS.md](DIALECTS.md) — dialect routing table; source of truth `src/dialect.rs`.
  CURRENT.
- [refusals.md](refusals.md) — typed refusal register: codes, `details` fields, remediation;
  append-only, never renumbered. CURRENT.
- [refusal-playbook.md](refusal-playbook.md) — symptom → refusal → cause → fix, messages
  captured from `examples/refusal_probe.rs`. CURRENT (re-run 2026-11-09).
- [reference/refusal-catalog.md](reference/refusal-catalog.md) — user-facing refusal view;
  normative register is `refusals.md`. CURRENT.
- [reference/runner-exit-codes.md](reference/runner-exit-codes.md) — exit codes for
  `graphlaw-run` / `graphlaw-run-eh`. CURRENT.
- [reference/knowledge-hook-projection.md](reference/knowledge-hook-projection.md) — powerless
  state → hook projection contract (authority NONE). CURRENT.
- [reference/typescript-api.md](reference/typescript-api.md) — npm package exported surface,
  types, error mapping. CURRENT.
- [reference/design-read-frame-bound.md](reference/design-read-frame-bound.md) — read_frame
  length-prefix bound design (G4, both hosts). Relocated; content unchanged.
- [reference/design-sniff-fixes.md](reference/design-sniff-fixes.md) — dialect sniff fixes
  (SNIFF-01, the `BareSql` refusal). Relocated; content unchanged.
- [reference/n3-audit-100-report.md](reference/n3-audit-100-report.md) — N3 audit report.
  Relocated; content unchanged.
- [adr/ADR-wasmi-retirement.md](adr/ADR-wasmi-retirement.md) — decision record retiring the
  wasmi host. Relocated.
- [explanation/receipt-fields.md](explanation/receipt-fields.md) — receipt fields, informal
  mapping to the portfolio receipt concept. CURRENT.
- [explanation/eh-closure-story.md](explanation/eh-closure-story.md) — why the wasm module was
  unhostable (legacy vs try_table EH) and how it closed. CURRENT.
- [explanation/edge-sovereignty-and-zero-trust-compaction.md](explanation/edge-sovereignty-and-zero-trust-compaction.md)
  — why local analytical compaction for edge/DDIL systems. CURRENT.
- [QUALIFICATION_ALGEBRA.md](QUALIFICATION_ALGEBRA.md) — standing as executable falsification;
  executable binding section maps it to `src/qualification.rs`. CURRENT (re-read 2026-11-09).
- [qme-1-reference.md](qme-1-reference.md) — QME-1 participation, pinned to
  chatman-ecosystem@88b276f. CURRENT.

## Conformance

- [reference/sql-rdf-rec.md](reference/sql-rdf-rec.md) — the sql-rdf Recommendation
  conformance text (pasted text; its W3C status claims are document text). Cite by section.
  CURRENT.
- [reference-generated/sql-rdf-conformance.md](reference-generated/sql-rdf-conformance.md) —
  GENERATED clause-by-clause report (regenerated by the `sql_rdf_conformance` court); never
  hand-edit. CURRENT.
- [reference/conformance_claim.md](reference/conformance_claim.md) — court-to-EU-AI-Act
  article matrix. CURRENT.
- [CHICAGO_N3_DATALOG.md](CHICAGO_N3_DATALOG.md) — bounded N3 + Datalog compliance court
  (`tests/chicago_n3_datalog.rs`). CURRENT.
- [CHICAGO_FIBO.md](CHICAGO_FIBO.md) — FIBO court: authority NONE, consequence EVIDENCE_ONLY,
  generated from RDF (`tests/chicago_fibo.rs`). CURRENT.
- [CHICAGO_CROSS_AUTHORITY.md](CHICAGO_CROSS_AUTHORITY.md) — cross-authority court, same
  standing vocabulary (`tests/chicago_cross_authority.rs`). CURRENT.
- [reference/wasm-gap-register.md](reference/wasm-gap-register.md) — consolidated wasm gap
  register. PENDING (typed open gaps inside).

## Operations (courts, gates, scripts)

Entry points under `scripts/` (listed from `ls scripts/` on 2026-11-09):
`mutation_court.sh`, `mutation_families.sh`, `shape_mutation_court.sh`, `spc_court.sh`,
`partition_court.sh`, `receipt_chain_sync.sh`, `receipt_replay_court.sh`, `count_gate.sh`,
`doc_staleness_gate.sh`, `witness_gate.sh`, `machine_shaped_gate.sh`, `wasm_audit.sh`,
`wasm_unknown_unknown_smoke.sh`, `gen_w3c_manifest.sh`, `gen_wasm4pm_graphlaw_vectors.py`.
The test ladder and exit discipline live in the root `CLAUDE.md` Commands section.

- [jira/v26.10.22/RECEIPTS.md](jira/v26.10.22/RECEIPTS.md) — one row per ticket: gate
  command, latest verdict, as-of timestamp. HISTORICAL-MARKED (v26.10.22 ledger).
- [jira/v26.10.22/designs/INDEX.md](jira/v26.10.22/designs/INDEX.md) — condensed index of the
  v26.10.22 design records kept under `designs/`. HISTORICAL-MARKED.
- [jira/v26.10.22/designs/RECEIPTS.md](jira/v26.10.22/designs/RECEIPTS.md) — receipts for the
  design lanes. HISTORICAL-MARKED.
- [jira/v26.10.22/designs/staleness-76-triage.md](jira/v26.10.22/designs/staleness-76-triage.md)
  — live staleness-gate triage. HISTORICAL-MARKED.
- [jira/v26.10.22/designs/t4-refusal-reaudit.md](jira/v26.10.22/designs/t4-refusal-reaudit.md)
  — refusal-register anchor re-audit (cited by `refusals.md`). HISTORICAL-MARKED.

## Wasm boundary

- [WASM.md](WASM.md) — wasm runbook: build, verify, embed, upgrade `graphlaw-wasm`. CURRENT.
- [wasm-integration.md](wasm-integration.md) — WASI shell vs hand-written safe core, re-read
  from `wasm/src/`. CURRENT.
- [how-to/run-the-wasm-module.md](how-to/run-the-wasm-module.md) — host-side run guide (also
  under Getting started). CURRENT.

## Legal

- [../LICENSE.md](../LICENSE.md) — P28 Sovereign Invariant License (P28-SIL); the sole license
  file at repo root. CURRENT.

## Program records (jira)

Active window: `jira/v26.10.28/` (newest directory under `docs/jira/`). The v26.10.22 window
is a historical record. Statuses are in the linked registers; read them at use time.

- [jira/v26.10.28/INDEX.md](jira/v26.10.28/INDEX.md) — window index and file table.
  CURRENT.
- [jira/v26.10.28/GL28-BACKLOG.md](jira/v26.10.28/GL28-BACKLOG.md) — single authoritative
  GL28 register. CURRENT.
- [jira/v26.10.28/WORK-ORDERS.md](jira/v26.10.28/WORK-ORDERS.md) — per-CTQ work orders with
  session-receipt standing. CURRENT.
- [jira/v26.10.28/GO-NO-GO.md](jira/v26.10.28/GO-NO-GO.md) — evidence table + verdict.
  CURRENT.
- [jira/v26.10.28/RECEIPTS.md](jira/v26.10.28/RECEIPTS.md) — append-only manufacturing
  receipts. CURRENT.
- [jira/v26.10.28/VOC-CTQ-TURNSTILE.md](jira/v26.10.28/VOC-CTQ-TURNSTILE.md) ·
  [jira/v26.10.28/QFD-HOUSE-OF-QUALITY.md](jira/v26.10.28/QFD-HOUSE-OF-QUALITY.md) ·
  [jira/v26.10.28/VISION-2028-C4.md](jira/v26.10.28/VISION-2028-C4.md) — CTQ matrix, house
  of quality, C4 vision. CURRENT.
- [jira/v26.10.28/GL28-1-ENVELOPE-SPEC.md](jira/v26.10.28/GL28-1-ENVELOPE-SPEC.md) ·
  [jira/v26.10.28/GL28-2-SMALT-WIRE-SPEC.md](jira/v26.10.28/GL28-2-SMALT-WIRE-SPEC.md) ·
  [jira/v26.10.28/GL28-4-EXPORTER-SPEC.md](jira/v26.10.28/GL28-4-EXPORTER-SPEC.md) ·
  [jira/v26.10.28/GL28-13-SLIMMING-CENSUS.md](jira/v26.10.28/GL28-13-SLIMMING-CENSUS.md) —
  GL28 specs and census. CURRENT.
- [jira/v26.10.28/provenance/README.md](jira/v26.10.28/provenance/README.md) — fork-swap
  provenance (lockfile pre-swap snapshot). CURRENT.
- [jira/v26.10.22/INDEX.md](jira/v26.10.22/INDEX.md) — v26.10.22 ticket register with the
  closeout WRAP section. HISTORICAL-MARKED.
- [jira/v26.10.22/CHARTER-DFLSS.md](jira/v26.10.22/CHARTER-DFLSS.md) — operator charter;
  header admits dispatch-by-summary. PENDING (self-declared UNVERIFIED charter text).
- [jira/v26.10.22/ADR-canonarrow-dual-witness-egress.md](jira/v26.10.22/ADR-canonarrow-dual-witness-egress.md)
  — CanonArrow dual-witness egress ADR. HISTORICAL-MARKED.
- [jira/v26.10.22/SQLRDF-25-remaining-gaps.md](jira/v26.10.22/SQLRDF-25-remaining-gaps.md) —
  remaining-gaps ticket. HISTORICAL-MARKED.

## Relocated (2026-11-09)

| Old path | New path |
|---|---|
| `jira/v26.10.23/ADR-wasmi-retirement.md` | [adr/ADR-wasmi-retirement.md](adr/ADR-wasmi-retirement.md) |
| `jira/v26.10.23/design-read-frame-bound.md` | [reference/design-read-frame-bound.md](reference/design-read-frame-bound.md) |
| `jira/v26.10.23/design-sniff-fixes.md` | [reference/design-sniff-fixes.md](reference/design-sniff-fixes.md) |
| `jira/v26.10.23/n3-audit-100-report.md` | [reference/n3-audit-100-report.md](reference/n3-audit-100-report.md) |

Removed from this index (archived out of the tree, recoverable from git history): the
`archive/` essays and receipts, the v26.10.22 THESIS/DISSERTATION/PRD/DFLSS-DOSSIER, most of
`jira/v26.10.22/designs/`, the v26.10.23/24/26 windows, `sjira/`, `ARD-26.10.25-DIALECTS.md`,
`wasm-duckdb-*.md`, and the generated `reference-generated/` files other than
`sql-rdf-conformance.md`.

## Cross-repo references

Sync direction: produced-by emits, consumed-by reads; commands run from the producing repo
unless noted. Rows below were last verified on disk 2026-10-22 and carry that date; the
spec-dossier row points at files in the sibling `smalt` repo, which this lane did not re-read.

| Artifact | Produced-by | Consumed-by | Sync command | Current state |
|---|---|---|---|---|
| Spec dossier: `docs/jira/v26.10.21/{WASM-SANDBOX-SPEC,GRAPHLAW-SPEC,COLDCHAIN-CASE-STUDY}.md` + `receipts/` | `/Users/sac/smalt` | graphlaw (downstream consumer evidence) | none (read-only; re-run smalt courts against a new graphlaw wasm sha) | UNVERIFIED as of 2026-11-09 (verified present 2026-10-22) |
| Frozen vector fixtures: `crates/wasm4pm-cognition/tests/fixtures/graphlaw/*.json` | graphlaw (`scripts/gen_wasm4pm_graphlaw_vectors.py`) | `/Users/sac/wasm4pm` (`tests/graphlaw_vector_oracle.rs`; ancestry court requires the fixture rev to remain an ancestor of graphlaw main) | `python3 scripts/gen_wasm4pm_graphlaw_vectors.py`, then commit in wasm4pm | generator script present in `scripts/` 2026-11-09; consumer side not re-read |
| WASI FFI shell: `wasm/src/ffi.rs` (header `wasm/src/ffi.rs:7`: rendered by ggen from the wja: graph) | `/Users/sac/ggen-marketplace` pack `rust-wasi-wasmex-pack` | graphlaw `wasm/src/ffi.rs` (generated — regenerate, never hand-edit) | `cd wasm && ggen sync run` (`wasm/ggen.toml`) | `wasm/ggen.toml` present 2026-11-09 |
| frozen-duckdb crate | crates.io `frozen-duckdb` | graphlaw `Cargo.toml` `[dependencies]` | none — a normal dependency, never a path `[patch]` | see `Cargo.toml` for the pinned version (re-read at use time) |
