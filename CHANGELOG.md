# Changelog

All notable changes to this project are documented here, in the
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/) format.

Versioning policy (stated once, applies to every entry): `graphlaw-eyeron` keeps its own
independent, upstream-vendored version (currently 0.7.8) and is not coupled to the
workspace calendar version; "Published as crates.io" lines refer to the `graphlaw` crate.

## [26.10.22] - 2026-10-22

Wave hardening: guard/gate census (19 scripts), typed-refusal expansions, exhaustive
signature-tamper courts, OCEL exporter, and an external oracle CI job.

### Added

- Guard/gate census: 19 executable guard/gate/court scripts now live under `scripts/`
  (`chain_flag_court.sh`, `count_gate.sh`, `doc_staleness_gate.sh`, `exit_capture_check.sh`,
  `fleet_cleanup.sh`, `gen_w3c_manifest.sh`, `gen_wasm4pm_graphlaw_vectors.py`,
  `machine_shaped_gate.sh`, `mutation_court.sh`, `partition_court.sh`,
  `partition_court_selfcheck_g1.py`, `partition_court_selfcheck_g2.py`,
  `receipt_chain_sync.sh`, `receipt_replay_court.sh`, `shape_mutation_court.sh`,
  `spc_court.sh`, `wasm_audit.sh`, `wasm_unknown_unknown_smoke.sh`, `witness_gate.sh`).
  The 14 new in this wave, one line each:
  - `chain_flag_court.sh` — non-vacuity court for `receipt_chain_sync.sh`: real `--check`
    verifies + unknown-flag usage refusal (exit 2) pinned so a flag regression fails loudly.
  - `count_gate.sh` — re-runs allowlist courts cited by "N passed; 0 failed" rows in
    `docs/**/*.md` and flags COUNT-STALE / COUNT-UNPAIRED mismatches; fenced output skipped
    loudly, never counted as a claim.
  - `doc_staleness_gate.sh` — fails on stale file:line doc anchors (code moved, doc did not).
  - `exit_capture_check.sh` — self-test for the court-runner Exit-capture law: proves the
    documented zsh pattern detects the real `false | true` head exit (a gate that cannot
    exit 1 is the failure).
  - `fleet_cleanup.sh` — advisory reclaim of repo-local `target-*` lane build roots older
    than a grace window (default 45 min); never touches `./target`; always exits 0.
  - `machine_shaped_gate.sh` — fails closed on `/Users/` literals in `tests/`/`src/` unless
    commented or `MACHINE-PINNED:` annotated; carries a `--self-test` anti-vacuity court.
  - `partition_court.sh` — code-partition court over `courts/code-partition/` pin files
    (axioms blake3 + manifest), with `partition_court_selfcheck_g1.py`/`g2.py` as its
    anti-vacuity self-check pair.
  - `receipt_chain_sync.sh` — syncs `receipts/CHAIN.blake3` and verifies the receipt chain
    (`--check`, unknown flags refused with usage).
  - `receipt_replay_court.sh` — K7-class court: re-executes embedded `cargo test` commands
    from a rotating deterministic sample of receipts and compares claimed vs observed counts;
    selfcheck proves a fabricated receipt is flagged.
  - `shape_mutation_court.sh` — anti-vacuity sweep for SHACL shapes: gates must flip on
    shape mutation.
  - `spc_court.sh` — SPC gate: Cpk ≥ 1.33 on the receipt fold path (P28-SIL program).
  - `witness_gate.sh` — every factual build/run witness in `docs/**/*.md` must cite an
    existing file:line (or carry `witness-gate:allow`); generated docs excluded.
- Typed law-kernel refusal `UNKNOWN-STEP-CAPABILITY` (`REFUSAL_UNKNOWN_STEP_CAPABILITY`,
  `src/law.rs:764`): a step naming a capability with no `BACKEND_AUTHORITIES` entry is a
  typed refusal, not a panic. Courts in `tests/law_state.rs` (direct + K10 callsite path).
- OCEL exporter binary `graphlaw-ocel-export` (`src/bin/graphlaw_ocel_export.rs`) with
  court `tests/ocel_export_court.rs`.
- External oracle CI job `external-oracle` (`.github/workflows/ci.yml`): the
  ggen-marketplace pm4pytest/graphlaw pytest suite runs against the binaries this repo
  builds (real subprocesses, no mocks; `continue-on-error: true` until stable).
- EH wall closed (SQLRDF-21): the exception-handling wall that blocked in-wasm
  DuckDB execution is closed by the dual-artifact policy — the archive law
  `-mllvm -wasm-use-legacy-eh=0` produces a new-EH (`try_table`/`throw`)
  artifact, hosted by wasmtime 48 with `wasm_exceptions` (wasmi 2.0.0 is a
  hard NO: no exceptions-proposal support). A `runner-eh` cargo feature plus a
  `graphlaw-run-eh` binary wrap the wasmtime-48 host runner. Design:
  `docs/jira/v26.10.22/SQLRDF-21.md` (A1' recipe,
  `designs/a1prime-trytable-recipe.md`; ENGINE-tier host decision,
  `designs/wasmi-eh-research.md`). Artifact re-pinned 2026-10-23: blake3
  `fa1558306690493249b176ec312a4f01e97798937e05cd49f2946d61a7e6d107`,
  57,781,384 B, Tag(13)×1, wasm_audit PASS, registry `--check` exit 0
  (`receipts/2026-10-23-eh-closure-wave.md` §8, canonical re-pin; the
  interim new-EH pin `991b2bee…` 46,138,360 B and the legacy `f149433c…`
  pin are historical).
- Op `duckdb` (op-15) promoted in-module: the host-only cfg gates are removed;
  the wasm module dispatches the `duckdb` op with no
  `#[cfg(target_arch)]` strip and no `duckdb` feature, and the op authorities
  now include frozen-duckdb on wasm. Raw SQL text is still not admitted —
  SQLRDF-17 stands; the in-module op accepts a Turtle AST document via
  `compile_turtle` only.
- Op-15 exec admits all graph-backed statements: the execution arm accepts
  every statement class backed by the admitted graph, not a hand-picked
  subset. The legacy raw-SQL hole stays closed under SQLRDF-17.
- `REFUSAL-05 UnboundedMutationRefusal` enforced: a WHERE-less UPDATE/DELETE
  requires BOTH `sql:wherePredicate` slots to be present — stricter than the
  shape's minimum — and refuses typed (`docs/refusals.md`, REFUSAL-05 row)
  before any engine contact, leaving the database byte-identical.
- MERGE is classified as a mutating statement class: its execution bumps the
  epoch counter (receipt-witnessed), so MERGE effects participate in
  epoch-based resync like other mutations.
- Session surface note: the `open` / `exec-with-session` in-module session
  surface is marked "landing" — status is declared in the ticket, not yet
  receipted.

### Changed

- MERGE `ON` scope refusal widened: `REFUSAL-03 ScopeIntegrityRefusal` gains arm
  `MERGE_ON_NO_TARGET_COLUMN = 7` (`src/sql_ast.rs:987-993`) — a MERGE `ON` condition
  referencing no target-qualified column (a constant-only ON matches every target row and
  makes WHEN MATCHED fan-out unbounded) is refused; register row in `docs/refusals.md`.
- Exhaustive 64-byte signature-tamper courts (`tests/trust_attest.rs:316-364`): flipping
  any single one of the 64 Ed25519 signature bytes fails verification, byte by byte.
- Binaries return typed exit codes via `ExitCode`/`RunError` instead of ad hoc
  `std::process::exit`/stringly errors (`src/bin/graphlaw_run.rs`, `dl_dualroute.rs`,
  `sa2a_portability_verify.rs`, `sa2a_run14_verify.rs` — see `git diff` for the four bins).
- Machine-shaped path fixes: `/Users/` absolute-path literals removed from tests per
  `machine_shaped_gate.sh` (comments and `MACHINE-PINNED:` annotations excepted).
- `docs/THESIS.md` and `docs/UNIFIED_DISSERTATION.md` moved to `docs/archive/` with a new
  `docs/archive/README.md` marking them as superseded material, not current conformance text.
- Op-15 `duckdb` no longer host-only (supersedes the 26.10.22 MIGRATION section
  "Op-15 `duckdb` is host-only in the wasm artifact"): the
  `#[cfg(not(target_arch = "wasm32"))]` dispatch gate is removed and the op
  runs in-module; the typed `unknown op \`duckdb\`` refusal on wasm no longer
  fires. See MIGRATION.md for what consumers must change.
- Docs reconciled to the post-switchover reality: `docs/WASM.md`,
  `docs/wasm-integration.md`, `docs/ARCHITECTURE.md`, `docs/DIALECTS.md`,
  `docs/how-to/run-the-wasm-module.md`, `docs/index.md`; `docs/refusals.md`
  rows 119-121 updated; operator decision OD-1 marked decided in
  `docs/jira/v26.10.22/designs/OPERATOR-DECISIONS.md`.

### Added (SQLRDF-21..24 wave, second batch)

- Op-15 persistent session surface (`src/duckdb_op.rs`): `open` /
  `exec`+`session` / `close` modes over a bounded registry — 16-session cap
  (`src/duckdb_op.rs:77`), typed unknown-session and duplicate-open refusals,
  session dies with the module instance.
- ATTACH path admission policy: `GRAPHLAW_ATTACH_ROOTS` allowlist, fail-closed
  when unset; `:memory:` is refused as an ATTACH target (`src/duckdb_op.rs`).
- `REFUSAL-05` shape-layer alignment: the ggen shape's `xone` relaxed to `or`
  to match the kernel's two-slot WHERE rule, and refusal messages carry the
  `[REFUSAL-05]` prefix; register row at `docs/refusals.md:80`.
- Multi-column INSERT/UPDATE extraction (SQLRDF-22 gap G4) with courts.
- DECIMAL column type plus typed decimal literals (SQLRDF-22 gap G1,
  `src/sql_ast.rs:1799`, courts at `src/sql_ast.rs:2650` block).
- COPY egress artifact digest recorded in the ExecutionReceipt (SQLRDF-22 gap
  G3), so egress is receipt-witnessed.
- Session-ABI court around the op-15 session surface.
- Wasmtime ports: common `Host` abstraction, boundary fuzz corpus (1172 cases,
  0 failures), memory-wall court (8/8), and a gws4 surface court.
- `graphlaw-run` EH-class pre-flight: the runner probes exception-support
  before dispatch and exits 96 on the EH-blocked class
  (`src/bin/graphlaw_run.rs`).
- pm4pytest precision API: `min_precision` parameter with a non-vacuity court
  (external oracle suite, CI `external-oracle` job).
- FIBO fixtures for conformance courts: `tests/fixtures/fibo/` (delegation,
  dvp_securities and failure-path variants).
- Statement-sequence design ticket SQLRDF-23
  (`docs/jira/v26.10.22/SQLRDF-23.md`).

### Changed (SQLRDF-21..24 wave, second batch)

- Canonical libunwind is now the uniform new-EH rebuild: candidate B
  (sha256 `6a5f79f7…`) is the single rebuild path for the new-EH artifact; the
  legacy variant is named only in build-script FAIL messages. Witness:
  `receipts/2026-10-23-staging-wipe-incident.md`.

### Fixed

- Wasm dtor-runner trap root-caused and fixed (SQLRDF-24): two defects
  witnessed — (1) musl atexit walker fn-read offsets + `slot` re-attach
  (`atexit.c.obj` replaced with the current wasi-libc member,
  sha256 prefix `a1874498…`), and (2) reactor-contract completion in
  `wasm/src/lib.rs`: static extension registration via `.init_array`
  constructors + atexit-family no-op overrides (destructors never run
  mid-module-life). Size delta explained by statically-linked
  core_functions/icu extension code + wave source delta under LTO.
  Qualification: lifecycle courts 3/3 GREEN; trap table clean
  (`receipts/2026-10-23-eh-closure-wave.md` §8).
- Document-sequence wire surface verified end to end (SQLRDF-23):
  duckdb_op 16/0, duckdb_eval 18/0, sql_ast 32/0 with the G1 DECIMAL 3
  and G4 multi-column 6 courts folded in
  (`receipts/2026-10-23-eh-closure-wave.md` §4).

### Added (SQLRDF-25..27 + CanonArrow wave, third batch)

- SQLRDF-25 rows 1-5 of the dialect-expansion gaps landed with courts (17/0:
  pivot 3, star 4, columns 4, json 2, composite 4 incl. a struct egress arm +
  SHACL depth route): PIVOT/UNPIVOT compiler dispatch (ON-IN-USING order
  load-bearing), `* EXCLUDE/REPLACE` star modifiers and `COLUMNS('regex')`
  closed shapes, JSON `->`/`->>` operators, composite struct/map/list
  literals + slicing (`tests/pivot_court.rs`, `tests/star_modifier_court.rs`,
  `tests/columns_projection_court.rs`, `tests/json_operator_court.rs`,
  `tests/composite_literal_court.rs`; remaining-gaps ledger
  `docs/jira/v26.10.22/SQLRDF-25-remaining-gaps.md`).
- Atomic document execution mode (SQLRDF-26): `sql:atomic true` selects
  BEGIN/COMMIT around the statement-sequence spine; any member failure
  ROLLBACKs and returns the typed `BatchMemberFailed` envelope with
  `failed_at` and a partial receipt marked `status:"aborted"`;
  `REFUSAL-01` arm `ATOMIC_LITERAL` (46) refuses a non-boolean atomic
  literal at compile.
- COPY-egress fix closed (SQLRDF-27): musl→wasi-libc fcntl ABI bridge via the
  standalone wrap shim `tests/fixtures/wasm-shim/glaw_wrap_shim.c` linked
  through `--wrap=open/--wrap=openat` RUSTFLAGS (canonical relink recipe,
  `docs/WASM.md`); close conditions FIBO 4/0 incl. tc08 COPY egress +
  lifecycle 3/0 witnessed (`receipts/2026-10-23-sqlrdf27-verdict.md`).
- ADR-CANONARROW dual-witness columnar egress
  (`docs/jira/v26.10.22/ADR-canonarrow-dual-witness-egress.md`):
  in-guest Arrow IPC stream egress plus versioned `receipt_v2`
  (`R_receipt_v2 = BLAKE3("GLAW_RECEIPT_V2" ∥ H_query ∥ H_pre ∥ H_post ∥
  H_triples ∥ H_columnar ∥ epoch)`, domain-separated from v1);
  `src/egress/` (`arrow.rs`, `mod.rs`), court `tests/receipt_v2_court.rs`;
  `receipt_v1` verifiers untouched; CHAIN re-baselined
  (`receipts/CHAIN.blake3`, `receipts/2026-10-23-chain-rebaseline-note.md`).
- Banking (DvP) battery 4/4 GREEN, including the INSERT pairing law fix
  (`tests/dvp_settlement_court.rs`;
  `receipts/2026-10-23-banking-4of4-witness.md`).
- Benchmark suite: `benches/` (`sql_rdf_admission`, `sql_rdf_compiler`,
  `sql_rdf_egress`, `sql_rdf_engine`, `tc_hot_01`, `tc_hot_02`,
  `workload_amortization`) with the amortization anti-vacuity court
  `tests/amortization_court.rs`.

Continued — sql-rdf dialect surface (35 statement classes, SHACL admission, expression
compiler, parameterized MERGE), typed refusal register, and receipt-hash pinning.

### Added

- 35 statement classes extracted from the DuckDB 2.0 PEG (`Statement` rule, commit
  `b6013f7aa2`) into `ggen/sql-dialect/sql_statement_kinds.rs`, generated via
  `ggen sync run` from `ontology/statements.ttl` and included by `src/sql_statement_kinds.rs`.
- Section 4 SHACL admission of the SQL AST: strict closed-world shape validation including
  required integer `sql:statementIndex` per statement (SHACL violation table anchored at the
  `sql:statementIndex` messages in `src/sql_ast.rs`, `validate_ast_shacl` at `src/sql_ast.rs:1031`).
- Expression-level compiler covering 29 command classes (`src/sql_ast.rs`).
- Section 7.1 parameterized `MERGE INTO` extraction (`extract_merge_statement`,
  `src/sql_ast.rs:952`).
- Scope lattice (`ProcessScope`/`ViewScope`) with `REFUSAL-03 ScopeIntegrityRefusal`
  (aggregate in WHERE/JOIN/GROUP BY, scope column in wrong position).
- Typed refusal register `REFUSAL-01..04` (`docs/refusals.md`): malformed-AST,
  unimplemented-statement, scope-integrity, and capability (wasm fail-closed) refusals.
- `H_query = BLAKE3(Canon(G_AST) || P_params)` admitted-statement receipts with
  canon-once carry (`src/duckdb_eval.rs`).
- Courts: SQL-in-RDF laundering corpus (`tests/sql_rdf_laundering_corpus.rs`), conformance
  rung (`tests/sql_rdf_conformance.rs`), determinism court (`tests/p28_determinism_court.rs`),
  WASI vendored-patch guard (`tests/wasi_patch_guard.rs` — a live test under feature
  `wasi-patched-deps` verifying the vendored `purrdf-sparql-eval` gates its clock on
  `target_os = "unknown"`, typed flunk on absence), and a `graphlaw-verify --help`
  surface court (`tests/verify_cli.rs` — exit 0 + documents `--trusted-key`, negative
  twin: unknown flag exits 2).
- Refusal message ergonomics (lane L85/T4): every register code triggered FOR REAL with
  captured user-visible messages (`examples/refusal_probe.rs`, 13/13 MATCH), symptom →
  message → cause → fix playbook at `docs/refusal-playbook.md`, and a re-audit of all
  register file anchors (receipt: `docs/jira/v26.10.22/designs/t4-refusal-reaudit.md`).

### Changed

- Pure-data typed refusals (P29-RFL / SQLRDF-18 switchover): `AstError` is
  `{ code: RefusalCode, ctx: RefusalContext }` (`#[repr(C)] Copy`, 64 B, no heap;
  Display renders from a const code-to-template table), living in the decomposed
  `src/sql_ast/` module (`ast.rs`, `refusals.rs`, `serializer.rs`;
  `src/sql_ast.rs` re-exports via `pub use refusals::*`). Legacy stringly
  compiler errors and the SQLRDF-17 legacy receipt-hash/raw-SQL arms are deleted —
  receipts key on `H_query` only, and the ABI duckdb op refuses legacy `ast.text`
  with a typed refusal pointing at `compile_turtle`/`ast.turtle`
  (`src/duckdb_op.rs`). Per P29-RFL (`docs/jira/v26.10.22/RECEIPTS.md` §6-7).
- Statement receipt hash is the TC-HOT-01 two-node form
  `H_query = BLAKE3(h_ast || P_params)` (equivalently
  `BLAKE3(Canon(G_AST) || P_params)` via `purrdf::try_canonicalize`) with
  canon-once carry (`CompiledQuery.canon_graph_bytes`), witnessed green in
  `sql_rdf_dialect` + `chicago_validation` including a 5-variant
  cosmetic-invariance set (`docs/jira/v26.10.22/RECEIPTS.md` SQLRDF-09 row).
- SPSC evidentiary ring landed: `src/evidence_ring.rs`, `rtrb` behind the optional
  `evidentiary-ring` feature (default-off), never-block property fence (dev ratio leg
  + release absolute leg, 1.584 µs witnessed < 10 µs) and ring-to-ledger joint court
  (max push 292 ns; drop gaps persisted exactly); restart watermark persists crash
  loss as gap markers, never silent renumbering (RECEIPTS.md §8-9).
- Query ledger (`src/query_ledger.rs`), epoch-bounded cache store
  (`src/query_cache.rs`), LRU-256 prepared-statement seam with `$N`-bound
  parameters (RECEIPTS.md §8).
- `frozen-duckdb` 2.0.2 stays on the host
  (`[target.'cfg(not(target_arch = "wasm32"))'.dependencies]`). The wasm
  staticlib emits a Tag section that wasmi 2.0.0 rejects, so columnar
  execution is not linked into `graphlaw-wasm`. SQL-in-RDF admission stays
  in the module through `compile_turtle`. The vendored `purrdf-sparql-eval`
  WASI clock patch is guarded by `tests/wasi_patch_guard.rs` (feature
  `wasi-patched-deps`).
- Licensing consolidated: `LICENSE` (MIT) and `LICENSE-P28-SIL.md` are replaced by
  a single `LICENSE.md` carrying the P28 Sovereign Invariant License
  (`urn:graphlaw:license:p28-sil:v2.1.0-sovereign`).
- WASM artifact re-pinned with this version:
  `f149433c6e1a84ec95e961ac92b16b65b28e15d6fd8546beb1779d3b54a02e7e`
  (7,104,147 B, tag section count 0, `pin-source=26.10.22`). The previous
  workspace pin `74ec3c8d…` carried the 26.10.21 version string. `332ea218…`
  (6,793,617 B) is historical.

### Terminal integration (2026-10-23)

- Full-interception wrap shim (`__wrap_open`/`__wrap_openat` in
  `frozen-duckdb-sys/wasi_posix.c` + raw-lld `--wrap` link args via
  RUSTFLAGS): closes the musl-vs-wasi-libc fcntl ABI mismatch behind
  SQLRDF-27. Closed receipt-gated: FIBO 4/0 + lifecycle 3/0 on the
  wrap-enabled artifact, ×3 court passes
  (`receipts/2026-10-23-sqlrdf27-verdict.md`).
- Final artifact pin `472d5557…bf80ea` (58,121,500 B) at commit `d9cab2f`;
  registry + npm re-pinned (npm 198/198, W51; `b3sum priv/graphlaw.wasm`
  re-verified on disk this entry). Prior identities (`4d9083af…`,
  `332ea218…`, `33083b92…`, `259cf5b0…`) marked historical.
- Publish pipeline: `publish-npm.yml` scoped-tarball fix (npm pack of a
  scoped package emits `seanchatmangpt-graphlaw-*.tgz`; the old
  `graphlaw-$VERSION.tgz` assertion failed validate every run) plus the
  operator runbook `docs/how-to/release-publish.md`
  (commit `7267c47`).
- DvP fixture alignment: banking battery 11/2 pre-fix → 4/0 after the DvP
  pairing fix; the multi-root refusal law (`MIXED_DOCUMENT_ROOTS`,
  `src/sql_ast/refusals.rs:75`) is courted with an anti-vacuity twin
  (`tests/dvp_settlement_court.rs:629-644`).
- Memory-wall ratio fence: never-block property as dev ratio leg vs
  independent baseline (floored at 100ns) + release absolute leg
  (`tests/evidentiary_ring_court.rs:227,:415`; 1.584 µs < 10 µs);
  wasmtime port 8/0.
- Whitepaper receipted-actuals corrections: final-artifact pin, npm
  198/198 marked as-of W51, "Packaged 363 files" → 360, SPC honest-FAIL
  state, CHAIN immutability wording softened (W63 audit; commit
  `7267c47`).
- Ledger truth-sync: 18 coherence-lane findings applied append-only to
  `docs/jira/v26.10.22/RECEIPTS.md` (wasm re-pin supersessions, W9-W14
  PENDING rows closed with landing evidence, artifact lineage marked
  historical). Exit-codes court 7/0 ×3 passes stands as
  witnessed-transcript, terminal-receipt pending
  (RECEIPTS.md §e). 9 drafts archived to
  `docs/archive/designs-2026-10-23/`.

## [26.10.21] - 2026-10-21

Wave release: Unconditional `frozen-duckdb 2.0` integration, zero-trust semantic compaction, and
  life-sciences cold-chain OCEL verification.

### Added

- Embedded `frozen-duckdb 2.0` unconditionally into GraphLaw as the third admitted upstream engine
-   (no optional feature gate).
- Declarative SQL-in-RDF compiler (`src/sql_ast.rs`) supporting projection, aggregations (`COUNT`,
-   `SUM`, `AVG`, `MIN`, `MAX`), joins (`sql:joinClause`), and parameters.
- Fail-closed SHACL pre-flight gate (`validate_ast_shacl`) preventing SQL injection, verifying token
-   safety, and refusing queries missing bounded `WHERE` clauses.
- Vectorized query evaluator (`src/duckdb_eval.rs`) with direct `ValueRef` variant inspection into
-   typed RDF literals, Hive partition scanning, and Parquet view registration.
- Deductive rule built-in `builtin_eval_sql` bridging columnar analytical executions into PurRDF /
-   N3 deduction state.
- Regulated life-sciences cold-chain biologics verification case study
-   (`ontologies/biopharma-ocel.ttl`, `examples/cold_chain_biologics_verification.rs`).
- Zero-trust edge compaction test suite (`tests/test_edge_sovereign_ingestion.rs`): 10,000:1
-   semantic reduction and offline fail-closed actuator lockout.
- Diataxis documentation: `docs/how-to/how-to-edge-ingestion-and-compaction.md` and
-   `docs/explanation/edge-sovereignty-and-zero-trust-compaction.md`.

## [26.10.20] - 2026-10-20

Wave release (G-lanes): dialect layer, W3C conformance, and WASM bounds/ABI hardening.
Workspace version remains 26.10.13 in `Cargo.toml` at this tree; the entry is named for
the wave that landed it.

### Added

- Two-tier document routing (`src/tier.rs`): RDF datasets parse into a dataset,
  N3 logic rule sets validate as logic; a document offered to the wrong tier is a
  typed R04 tier-mismatch refusal instead of a parse error. Court:
  `tests/tier_court.rs` (7 tests).
- Typed parse-refusal taxonomy (`src/dialect.rs`, `ParseRefusal`/`ReasonClass`),
  including `FormulaInRdfLayer` (N3 formula `{ ... }`/`=>` offered to an RDF
  dialect) and `RemoteContext` (JSON-LD `@context` referencing a remote IRI;
  fetching is forbidden). Court: `tests/parse_refusal_court.rs` (12 tests).
- Cross-dialect RDFC-1.0 canonicalization court: `tests/canonical_cross_dialect.rs`
  (6 tests) pins the RDFC-1.0 + BLAKE3 canonical id through `LawState::from_dataset`
  across dialects.
- W3C RDF 1.1 conformance corpus vendored under `tests/fixtures/w3c/`
  (Turtle / N-Triples / N-Quads, with provenance and license in that README) and a
  325-entry inventory (`tests/w3c_manifest.rs`, generated by
  `scripts/gen_w3c_manifest.sh`). Court `tests/w3c_conformance.rs` buckets every
  entry as passed / refused / known-diverged and asserts per-dialect literal counts.
- WASM bounds: `MAX_LAW_STEPS` (1000, `src/abi.rs`), datalog caps
  (`MAX_DATALOG_RULES` 10,000 / `MAX_DATALOG_FACTS` 100,000), and a JSON request
  depth guard (`MAX_JSON_DEPTH` 64 via `json_depth`, applied to every op including
  `op_shacl`). Court: `tests/abi_bounds_court.rs` (8 tests).
- ABI manifest court `tests/abi_manifest_court.rs` (10 tests) over the committed
  `contracts/abi-manifest.json` (`src/abi_manifest.rs`): regeneration is
  byte-identical, exports/limits pinned.
- Native-vs-wasm differential court `tests/wasm_native_differential.rs` — 35 cases
  (23 fixed requests, 6 canonical cases, 6 pack-driven cases) comparing native
  `graphlaw` execution against the built `priv/graphlaw.wasm` artifact: zero
  divergence, byte-identical outputs and refusals.
- Artifact size court `scripts/wasm_audit.sh` (W04): real artifact size vs a
  25 MiB stripped-copy ceiling, plus header checks.
- Docs: `docs/WASM.md` and `docs/DIALECTS.md`.
- CI: `wasm-court` job in `.github/workflows/ci.yml` — compiles the facade for
  wasm32-unknown-unknown, builds the WASI module, and runs the WASI + ABI courts.

### Changed

- WASM crate version skew closed: `wasm/Cargo.toml` now at 26.10.13, matching the
  workspace, with the artifact digest committed in the same commit per its header
  comment. `priv/graphlaw.wasm.blake3` pins
  `cbc80f07346f025d4e1cd001f21d15474928cb055d4b1a9c0ac7c31c5938dc6a` — valid for
  the 26.10.13 source state only, and must be re-pinned with any further artifact
  change.

### Changed (known divergences)

- Registered 9 W3C divergences, all negative-suite entries the owning engine
  (PurRdf) currently accepts: 5 Turtle, 2 N-Triples, 2 N-Quads
  (`KNOWN_DIVERGENCES` / `KNOWN_POSITIVE_DIVERGENCES` in
  `tests/w3c_conformance.rs`). Each is a tracked conformance gap, never a silent
  skip; the court fails on any unregistered divergence, and closing one in the
  engine breaks the literal counts and forces register removal.

## [26.10.13] - 2026-10-10

Published as crates.io `graphlaw` 26.10.13.

ERRC release: Eliminate/Reduce/Raise pass over the repo; no semantic changes to src/.

### Added
- tests/readme_first_ten_minutes.rs pins the README's example outputs (hashes,
  refusal text); docs_release_claims.rs pins the doc version lines to
  CARGO_PKG_VERSION.
### Reduced
- ci.yml: dropped the duplicated chicago/qualification steps and the clippy-subsumed
  check step.
### Eliminated
- extract-from-graphlaw.yml (destructive tree-replacing footgun; EXTRACTION.md updated);
  malformed generated docs/reference-generated/{how_to,explanation}.md; orphaned
  ggen.toml; dead .gitignore rules.

## [Unreleased]

### Added

- Initial JavaScript/TypeScript distribution: npm package `graphlaw` wrapping the
  pinned wasm32-wasip1 artifact (js/ tree) — typed client for all 14 ops,
  graphlaw/raw passthrough, types generated from registry/capability-registry.json,
  vitest golden-vector suite over registry/op-examples.json, and a Trusted-Publishing
  publish workflow. Unreleased; npm publish happens later.
- Diataxis documentation tree linked from the README: docs/index.md hub,
  four hand-written how-tos (run-the-wasm-module, authorize-with-signed-leases,
  verify-without-trusting-the-operator, use-knowledge-hooks), and
  docs/explanation/receipt-fields.md. All commands in them executed against this
  tree; all ship in the crate via the existing /docs/** include.
- README rewritten from first principles (~120 lines): install, verified
  first-10-minutes flows, Diataxis index, authorities, production rule; wire
  contracts (limits, refusals, leases) moved out to the docs tree.

### Fixed

- README Eyeron pin wording (0.7.7 -> "0.7.8 vendored from upstream 0.7.7@d6568f65");
  docs/api-stability.md and SECURITY.md stale version lines -> 26.10.12.


### Changed

- Artifact digest migration sha256 -> BLAKE3: `priv/graphlaw.wasm.blake3` and
  release-asset sidecars now carry b3sum-format BLAKE3 digests (`b3sum`).
  CI (`js.yml`, `release.yml`, `publish-npm.yml`) installs and uses `b3sum`
  (`cargo install b3sum --locked`); the publish job's cross-job digest check
  env var renamed `EXPECTED_SHA256` -> `EXPECTED_BLAKE3`. Consumers pinning
  the old sha256 values must repin (see MIGRATION.md). The historical
  praxis-era content was purged earlier in this cycle; this entry records the
  digest swap itself.

## [26.10.11] - 2026-10-10

Published as crates.io `graphlaw` 26.10.11.

Docs-accuracy release: 16-lane verification fan-out over every `*.md`/`*.txt` in the repo; all
checkable claims re-verified against live code, tests actually re-run.

### Fixed

- README: receipt-chain and refuse-hook surfaces marked landed (`705dd6f`, `e732385`) instead of
  uncommitted working-tree state.
- MIGRATION: WASM module target corrected to `wasm32-wasip1` (not `wasm32-unknown-unknown`).
- EXTRACTION: false claim that the `extract-from-graphlaw` workflow was deleted corrected with a
  dated note (workflow still present; engine retirement is enforced by Cargo.toml pins).
- capability-registry: doc now reflects the emitted registry (15 limit keys, not 6; commands
  observed exiting 0, not UNKNOWN).
- api-stability: version header current (workspace 26.10.10); stable module list completed with
  the 5 missing `pub mod`s; semver-checks described as running in CI.
- refusals: `ResourceLimit` row lists all 5 N3 limit names; aggregate-boundary section rewritten
  after ggen deleted `crates/graphlaw` (`8e92df31f`).
- wasm-integration: ARTIFACTS digest pin corrected to `2802cede…` and drifted line citations
  re-anchored.
- graphlaw-eyeron README: dead relative doc links pointed at upstream URLs (docs/examples are not
  carried in the vendored copy).
- innovation: dead test references (removed by revert `0a38b68`) replaced with live test coverage.

### Added

- doc-hdit scaffolded reference documentation
-   (`docs/reference-generated/{reference,how_to,explanation}.md`,
  regen command in its `README.md`), generated from the code surface (`src/` + `crates/`; 94
  modules, 2,097 symbol rows) with `rust-doc-hdit-pack`; re-rendered with backticked identifier
  columns (`10b13bc`, `e1059f1`). Linked from the README API-stability section.

## [26.10.10] - 2026-10-10

Published as crates.io `graphlaw` 26.10.10.

Correction: the 26.10.5 entry below claimed a crates.io publish that never happened — that release
was `BLOCKED:crates_io_credentials`; the registry only carries 26.9.28/26.9.29. The line is removed
and the claim corrected here.

### Added

- Regenerated capability registry for 26.10.10: `graphlaw_version` bumped and
  `registry_blake3` recomputed to
  `blake3:21d2f8e04ccebd9c66564635274f6e8798b5a2eb552f01ab683609030e65f6d4`
  (`registry/capability-registry.{json,ttl}`, `registry/ARTIFACTS.blake3`). Consumers pinning
  `registry_blake3` must update. `priv/graphlaw.wasm` and its `.blake3`/`ARTIFACTS.blake3` pin
  were regenerated for the new lib (`wasm 2802cede…`, 6,659,724 B); the wasm crate version
  remains the documented 26.10.5 pin.
- Receipt chain + refuse hooks (`src/receipt_chain.rs`, hook integration in `src/hooks.rs`):
  chained BLAKE3 receipts with refuse/refusal hooks (typed `Verdict::{Fired, Refuse}`,
  `Materialized.verdicts`), per
  `docs/receipt-2026-10-09-receipt-chain-and-refuse-hooks.md`; 3 new integration test files
  (workspace suite: 436 passed, 0 failed, 1 ignored).

## [26.10.5] - 2026-10-05

### Added

- `graphlaw::capability_intake` (CASTLE donor intake): a descriptive, side-effect-free registry of
  the donor subjects GraphLaw may wrap or absorb while remaining the sole semantic-law owner.
  `DONORS` carries three projected donors (unrdf `WRAP`, graphlaw `ABSORB`, mfw `CANDIDATE_WRAP`);
  `donor(repository)` looks one up by exact repository identity; the consts `PROJECTION_SOURCE`
  (the admitting ggen-ecosystem projection, `@50fdfa2`), `OWNER_CAPABILITY`
  (`SEMANTIC_LAW_DERIVATION`) and `AUTHORITY_CEILING` (`CONSTRUCT`) bound the intake, and
  `consequence_authority` is `false` for every donor — projected donor capability never carries
  GraphLaw or CASTLE DO authority.

### Changed

- Capability registry (additive, schema id unchanged): `limits` now lists all 15 engine limits
  (adds `n3_max_derived_facts`, `n3_max_total_bytes`, `n3_max_term_bytes`, `n3_max_match_steps`,
  `max_plan_total_atoms`, `hooks_max_rounds`, `hooks_max_firings`, `hooks_max_state_quads`,
  `max_outstanding_alloc_bytes`), each read from its source constant; new `limit_meta` (scope,
  unit, source, refusal name), `models` (Lease, SignedLease, Receipt, Attestation, Plan, Action,
  PolicyEntry, PolicyOutcome) and `model_enums`; the Turtle projection gains `gac:Model*` and
  limit scope/unit/source. The wasm 256 MiB outstanding-alloc cap is now
  `graphlaw::abi::MAX_OUTSTANDING_ALLOC_BYTES`. Registry and wasm pins in `ARTIFACTS.blake3`
  were regenerated.
- Resource safety: the registry Turtle projection can no longer loop (a leaked cargo-mutants edit
  had made it spin and exhaust memory); Eyeron gains run-wide caps and `ReasonerLimit::ClosureSize`
  / `ClosureFacts` (`graphlaw-eyeron` 0.7.8); N3 refusals report their real limit and ceiling
  (`n3_iterations`, `n3_derived_facts`, `n3_total_bytes`, `n3_term_bytes`, `n3_match_steps`);
  `plan_total_atoms` caps a plan's cumulative atoms; hooks cap the state at 1,000,000 quads; the
  wasm module caps outstanding `gl_alloc` bytes at 256 MiB; `N3Error::Limit` gains `limit` and
    `max`.
- `plan::Action` gains `pre_not`, `plan::Plan` gains `goal_not`, and `LawError::PlanRefused`
  gains `violated_absent`; every in-repo struct literal (tests, examples) was updated with
  `..Default::default()`. Digests of plans without negation are unchanged.
  Present since v26.9.29 (misfiled as breaking in this section; verified via
  cargo-semver-checks 196/196 against the v26.9.29 baseline).
  Migration notes (introduced in 797056b, "Negative preconditions and negative goals in plan
  admission"):
  - `plan::Action` — exhaustive struct literals no longer compile; add `pre_not` or spread
    `..Default::default()`:

    ```rust
    // before (26.9.28)
    let a = graphlaw::plan::Action {
        name: "open".into(),
        pre: "<urn:d> <urn:p:is> <urn:v:closed> .".into(),
        add: "<urn:d> <urn:p:is> <urn:v:open> .".into(),
        del: "<urn:d> <urn:p:is> <urn:v:closed> .".into(),
    };
    // after (26.10.5)
    let a = graphlaw::plan::Action {
        name: "open".into(),
        pre: "<urn:d> <urn:p:is> <urn:v:closed> .".into(),
        pre_not: "<urn:d> <urn:p:is> <urn:v:locked> .".into(),
        add: "<urn:d> <urn:p:is> <urn:v:open> .".into(),
        del: "<urn:d> <urn:p:is> <urn:v:closed> .".into(),
    };
    // or: ..Default::default() after `del` to keep the old shape.
    ```

  - `plan::Plan` — same shape change for `goal_not` (or prefer `Plan::builder()`, which is
    unaffected):

    ```rust
    // before (26.9.28)
    let p = graphlaw::plan::Plan { actions, goal: goal_nt.into() };
    // after (26.10.5)
    let p = graphlaw::plan::Plan {
        actions,
        goal: goal_nt.into(),
        goal_not: String::new(),
    };
    ```

  - `LawError::PlanRefused` — the variant is `#[non_exhaustive]`-exempt (it is a struct
    variant), so exhaustive matches gain a new binding; add the field or a wildcard:

    ```rust
    // before (26.9.28)
    LawError::PlanRefused { index, action, missing } => { ... }
    // after (26.10.5)
    LawError::PlanRefused { index, action, missing, violated_absent } => { ... }
    // or: LawError::PlanRefused { index, action, missing, .. } => { ... }
    ```
- API stability: growable public enums (`Step`, `Dialect`, `Engine`, `RefusalKind`,
  `PolicyRefusalKind`, `LeaseReason`, `ReceiptReason`, `Ceiling`, `N3Error`, `LawError`,
  `StoreError`, `AttestError`) are `#[non_exhaustive]`; downstream matches need a wildcard arm.
- `#![deny(missing_docs)]` on `graphlaw` and `graphlaw-wasm`; every public item is documented and
  the core types have runnable doctests.

- Documentation fixes accompanying the registry (op reference, refusal codes and limits now agree
  with the emitted registry).
- CI and release workflows: every action reference is pinned to a commit SHA and cargo build/test
  steps use `--locked`.

- Unsigned lease transitions are now `transition_leased_unverified`; the ABI `law` op with a
  `lease` requires `signed_lease` and `trusted_keys` unless `unverified_lease: true` is set.

### Added

- Capability registry (`graphlaw.capability-registry/1`): the Rust table in `src/registry.rs`
  (feature `abi`) is the single source of truth for the 14 ABI ops, their request and response
  fields, dialects, regimes, law steps, refusal codes, limits and authorities. It is emitted to
  `registry/capability-registry.json` (canonical, sorted keys), `registry/capability-registry.ttl`
  (deterministic Turtle projection), with `registry/capability-registry.schema.json` and
  `registry/capability-registry.shapes.ttl` (SHACL, validated by GraphLaw's own `shacl` op in
  tests). The `graphlaw-registry` binary supports `--write`, `--check`, `--print-json`,
  `--print-ttl` and `--print-digests`; CI runs `--check`.
- `capabilities` op additive response fields `registry_schema`, `registry_blake3` and
  `surface_blake3`. Existing fields keep their names and values; `ops`, `rdf_dialects` and
  `other_dialects` in `abi.rs` are now built from the registry rather than a second literal.
  `ABI_VERSION` stays `1`; the change is additive only.
- Op-examples corpus `registry/op-examples.json` (`graphlaw.op-examples/1`, with schema): every op
  has an ok example listed before its refused examples, and every op except `capabilities` has at
  least one refused example. Examples are not part of the registry digest.
- Registry-vs-dispatch court: tests parse the `"<op>" =>` arms of `abi::dispatch` and require them
  to equal the registry op list and order, with the `unknown op` fallthrough intact.
- Native mutation court over the registry and dispatch surface.
- Per-op differential tests (each op against the registry and op-examples) and forward-compat
  tests (unknown request and response fields are tolerated, unknown refusal codes remain
  refusals).
- Release assets: `capability-registry.json`, `capability-registry.ttl` and `op-examples.json`,
  each with a `.blake3`, are attached next to `graphlaw.wasm` (release remains held by
  `RELEASE_HOLD`).
- `docs/api-stability.md`: registry additive-field rule and `surface_blake3` compatibility rule.
- Plan negation (negative preconditions and goals) — see Changed above for the full treatment,
  migration blocks and provenance.
- `plan::Triple` (validated IRIs, escaped literals), `plan::PlanBuilder` / `Plan::builder()`,
  `plan::ActionBuilder` / `Action::builder()` and `plan::TripleError`; `examples/quickstart_plan.rs`
  uses them. Struct-literal construction is unchanged.
- `docs/api-stability.md` (stability, deprecation, MSRV and `ABI_VERSION` policy),
  `tests/api_stability.rs`, `tests/plan_builder.rs`.
- CI jobs `semver` (cargo-semver-checks, informational), `deny` (`deny.toml`) and
  `audit` (cargo-audit).

- Independent plan admission: `plan::Plan::admit` replays actions and refuses on the first unmet
  precondition or goal, with a receipt per action and a canonical plan digest.
- Receipts and receipt feedback: `receipt::record` / `receipt::require` and
  `Step::RequireReceipt` gate a step on a recorded receipt; native/wasm determinism checks.
- Authority leases (`law::Lease`, `Ceiling`) and an out-of-subject receipt store.
- FOND strong-cyclic policy admission (`policy`, feature `abi`).
- Resource limits on the JSON ABI (`request_bytes`, `json_depth`, `atoms_per_field`,
  `plan_actions`, `policy_entries`, `n3_iterations`).
- Structured refusal `details` objects with a stable `code` on ABI refusals.
- Signed receipts and leases: Ed25519 `attest`, `receipt::record_signed` / `require_signed`,
  `Step::RequireSignedReceipt`, `SignedLease` judged against an injected `Clock` with a skew bound.
- `graphlaw-verify` binary: replays a plan and checks the receipt chain and attestations offline.
- Wasm lease-boundary test and an upstream-ready vendored clock patch.
- `examples/`, `docs/quickstart.md`, `docs/refusals.md`, `SECURITY.md`, `CONTRIBUTING.md`.

## [26.9.28] - 2026-09-28

### Changed

- Semantic reset: GraphLaw delegates RDF/SPARQL/SHACL/ShEx/Datalog/entailment to PurRDF and N3 to
  Eyeron; the bespoke parser, triplestore, reasoner and validators were removed.
- Crate renamed `graphlaw` to `graphlaw`.

### Added

- Law-state layer, dialect router, differential oracle, knowledge hooks, JSON ABI and a
  self-contained WASI module; vendored Eyeron as `graphlaw-eyeron`.
## [26.10.12] - 2026-10-10

Published as crates.io `graphlaw` 26.10.12.

### Changed

- `cicd.toml`: added `[docs] verify_cmd` gate config consumed by cargo-cicd's
  doc-drift gate (`94dfc0c`); regenerated with section preservation (`917dbcc`).

### Fixed

- README "Releasing" section corrected: CI tags and ships the GitHub release only;
  crates.io publishing is local-only (`cargo cicd publish run`). The 26.10.11 README
  shipped a stale CI-publish claim (docs verified before a workflow change, publish
  after — no gate re-checked in between). crates.io pages are immutable per version,
  so the correction ships in this release.
- `docs/receipt-2026-10-10-publish-dod.md` carries a dated supersession footer; body
  preserved.

### Added

- `tests/docs_release_claims.rs`: the README's Releasing section must match
  `release.yml`'s actual publish behavior in either direction (runs in the CI native
  job's existing `--all-targets` sweep).
