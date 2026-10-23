# SQL-in-RDF tickets — v26.10.22

Status of every ticket: **Open**, except SQLRDF-08 (withdrawn: DuckDB is inside the wasm unit).

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
| SQLRDF-01 | Normative vocabulary | Open | was: Section 3.1 and 3.3 names absent → now present in [`ontologies/sql-ast.ttl`](../../../ontologies/sql-ast.ttl) (legacy aliases retained in the alias window); compiler-side hop closed — re-executed 33/0 exit 0 (receipts/[sql-rdf-dialect-2026-10-22.md](../../../receipts/sql-rdf-dialect-2026-10-22.md); grew from the legacy 25/0 as the court file grew) | [SQLRDF-01](SQLRDF-01.md) |
| SQLRDF-02 | Closed-world SHACL from section 4 | Open | was: no `statementIndex`, projection-list, operator-token, or `rdf:List` shapes → shapes now landed in [`ontologies/sql-ast.ttl`](../../../ontologies/sql-ast.ttl) (`sql:StatementBaseShape`, `sql:SelectStatementShape`, `sql:BinaryExpressionShape`, `sql:statementIndex`); compiler-side hop (`REFUSAL-01` on missing index) closed — dialect re-executed 33/0 exit 0 (receipts/[sql-rdf-dialect-2026-10-22.md](../../../receipts/sql-rdf-dialect-2026-10-22.md)) + conformance 7/0 (receipts/[sql-rdf-conformance-2026-10-22.md](../../../receipts/sql-rdf-conformance-2026-10-22.md)) | [SQLRDF-02](SQLRDF-02.md) |
| SQLRDF-03 | Clause compiler for having, orderBy, and sql:from | Open | was: `HAVING`, `ORDER BY`, schema-qualified `sql:from`, and scope IRIs unread → closed by compiler support in [`src/sql_ast.rs`](../../../src/sql_ast.rs), courted by `cargo test --test sql_rdf_dialect` (33/0 final, 2026-10-22, exit 0) plus the HAVING court | [SQLRDF-03](SQLRDF-03.md) |
| SQLRDF-04 | Section 7 analytical MERGE | Open | was: merge was a one-row template, not the section 7.1 parameterized statement → closed by the parameterized MERGE compiler in [`src/sql_ast.rs`](../../../src/sql_ast.rs) / [`src/duckdb_eval.rs`](../../../src/duckdb_eval.rs), courted by `cargo test --test chicago_validation` (9/9) plus the receipt courts | [SQLRDF-04](SQLRDF-04.md) |
| SQLRDF-05 | Expression-level graphs for the other admitted classes | Open | was: the remaining admitted classes compiled as fixed templates → closed by expression-level graphs in [`src/sql_statement_kinds.rs`](../../../src/sql_statement_kinds.rs) (env-grammar classes), courted 6/0 exit 0 in [`tests/duckdb_statement_kinds.rs`](../../../tests/duckdb_statement_kinds.rs) — the `.gram` environmental exception cleared (receipts/[duckdb-statement-kinds-2026-10-22.md](../../../receipts/duckdb-statement-kinds-2026-10-22.md)) | [SQLRDF-05](SQLRDF-05.md) |
| SQLRDF-06 | REFUSAL-02 for ConnectStatement and DisconnectStatement | Open | was: compile refused both classes without the stable `REFUSAL-02` code → closed by the register-aligned typed refusal carrier in [`src/sql_ast.rs`](../../../src/sql_ast.rs) / [`docs/refusals.md`](../../refusals.md), courted by the REFUSAL-02 branches in [`tests/sql_rdf_dialect.rs`](../../../tests/sql_rdf_dialect.rs) | [SQLRDF-06](SQLRDF-06.md) |
| SQLRDF-07 | REFUSAL-03 scope integrity | Open | was: aggregates and `sql:ProcessScope` in the select list were not refused → closed by select-list scope integrity in [`src/sql_ast.rs`](../../../src/sql_ast.rs), courted by `cargo test --test sql_rdf_dialect` (9/9) plus the evasion set and join courts | [SQLRDF-07](SQLRDF-07.md) |
| SQLRDF-08 | REFUSAL-04 on the wasm build | Withdrawn | The required behavior (refuse every `sql` document on wasm, `default-features = false`) contradicts the one-unit law. `REFUSAL-04` is the non-document-backed builtin reader. SQL-in-RDF admits through `compile_turtle` inside the module. | [SQLRDF-08](SQLRDF-08.md) |
| SQLRDF-09 | Admitted-statement receipt | Open | was: the execution hash covered SQL text and parameters, not the canonical AST graph → closed by the formula hash over the canonical AST graph in [`src/duckdb_eval.rs`](../../../src/duckdb_eval.rs), witnessed by the formula receipt and courted by `cargo test --test sql_rdf_dialect` (33/0 final, 2026-10-22, exit 0) | [SQLRDF-09](SQLRDF-09.md) |
| SQLRDF-10 | Section 1.4 post-quantum identity is unmet | Open | was: `Σ_PQC` had no conformance note → closed by the conformance note filed in [`docs/refusals.md`](../../refusals.md) (§1.4 identity), courted 10/0 in the attestation courts | [SQLRDF-10](SQLRDF-10.md) |
| SQLRDF-11 | Chicago extension court through compiled AST graphs | Open | was: section 6.2 checks ran as raw SQL and could skip when unsigned extensions were off → closed by §6.2 courts running compiled AST graphs over real frozen-duckdb, courted by [`tests/chicago_validation.rs`](../../../tests/chicago_validation.rs) | [SQLRDF-11](SQLRDF-11.md) |
| SQLRDF-12 | File the conformance text | Open | was: the pasted Recommendation had no stable path in the tree → closed by [`docs/reference/sql-rdf-rec.md`](../../reference/sql-rdf-rec.md) plus the citation gates in this window's tickets | [SQLRDF-12](SQLRDF-12.md) |
| SQLRDF-13 | Validation-scoped RDFS closure for §4 sh:class constraints | Open (follow-up, not in v26.10.22 acceptance) | `sh:class sql:Expression`/`sql:ASTNode` withheld at `ontologies/sql-ast.ttl:315-330` (asserted-data-only validation) | [SQLRDF-13](SQLRDF-13.md) |
| SQLRDF-14 | §3.1 letter conformance — re-parent the 35 PEG leaf classes under the six family classes | Open (follow-up, not in v26.10.22 acceptance) | The 35 PEG leaves generate flat under `sql:Statement` with no family ancestor | [SQLRDF-14](SQLRDF-14.md) |
| SQLRDF-15 | §6.2 tpch setup divergence — retire the sf=0.01 hand-patch into the generator + register | Open (follow-up, not in v26.10.22 acceptance) | Hand-patch in `ggen/sql-dialect/sql_extensions.rs:22-26` disagrees with `extract_statements.py:31`; the 0-row sf=0 witness is contradicted by upstream dbgen.cpp, so the divergence is unadjudicated (frozen pin sources absent) | [SQLRDF-15](SQLRDF-15.md) |
| SQLRDF-16 | Moonshot backlog — the 1000× ERRC horizon (thesis Part II) | Open (follow-up, BLOCKED-BY: v26.10.22 integration commit) | Eight moonshots (E1/E2/E3/R1/R2/A1/C1/C2) registered with falsifiers + dependency order E1→A1→E2→E3, R1→C1, R2→C2; thesis at [THESIS.md](THESIS.md) (verbatim body pending — dispatch did not carry the prose) | [SQLRDF-16](SQLRDF-16.md) |
| SQLRDF-17 | ABI duckdb exec accepts raw SQL text (C2 hole P1) | Open (follow-up, BLOCKED-BY: v26.10.22 integration commit) | `src/duckdb_op.rs:198-219` accepts `ast.text` as raw SQL (legacy `(None,None)` hash arm at `src/duckdb_eval.rs:183-188`); dialect sniff misroutes bare SQL to Sparql (`src/dialect/sniff.rs:128` `sniff`, no `BareSql` class); `raw_connection()` is `pub` (`src/duckdb_eval.rs:155`) — spin-out of SQLRDF-16 row C2 | [SQLRDF-17](SQLRDF-17.md) |
| SQLRDF-18 | P28-SIL program — substrate + zero-overhead execution architecture | Open (successor program, BLOCKED-BY: v26.10.22 integration commit except LICENSE.md + TC-LEGAL-01 + baseline measurement, which proceed now) | PRD "GraphLaw Substrate & P28-SIL Zero-Overhead Execution Architecture" v1.0.0 filed verbatim at [PRD-P28-SIL.md](PRD-P28-SIL.md); TC-LEGAL-01/02 courts landed (3/0); TC-MOCK-00 gate landed; Phase 3 residue closed (corpus 14+/0 inline flips + shape_mutation_court.sh); Phase 4 license packaged (LICENSE-P28-SIL.md); Phase 2 skeleton implemented (ring+ledger+join courts green); Phase 1 designed (designs/PHASE1-generated-gate.md), BLOCKED-BY E1→A1→E2; TC-HOT benches landed, budget adjudication pending full-corpus run; §3.1 builds on landed R1 canon-once (`src/sql_ast.rs:2141`) | [SQLRDF-18](SQLRDF-18.md) |
| SQLRDF-19 | CI feature-matrix gaps — untested build cells | Open (follow-up, BLOCKED-BY: v26.10.22 integration commit) | PR CI runs one native combo (`pack-tools,abi,wasi-patched-deps`, ci.yml:29/37); `--all-features` (ring, runner) is release-only (release.yml:85); untested cells: `runner` bin zero coverage, ring-without-duckdb, ring+duckdb seam, `--no-default-features`, pairwise off-default cells | [SQLRDF-19](SQLRDF-19.md) |
| SQLRDF-20 | BEAM hot-upgrade protocol — artifact swap under the supervisor, with its own court | Open (follow-up, BLOCKED-BY: v26.10.22 integration commit + SQLRDF-19) | Upgrade is a versioned release event (digest re-pin + ARTIFACTS row + changelog before first admission); evidentiary continuity via host-side ring/ledger; required court: `--expect-blake3` pin + admission-table version in first post-upgrade ledger batch + receipt chain across the boundary ([SQLRDF-20.md](SQLRDF-20.md)) | [SQLRDF-20](SQLRDF-20.md) |
| SQLRDF-21 | Sovereign in-WASM DuckDB execution (EH wall closed) | Open (courts landing this wave) | `duckdb` ABI op is host-side (`src/duckdb_op.rs:74-82` per-call in-memory connections, raw-SQL exec arm `src/duckdb_op.rs:198-219`); required: in-module dispatch with no cfg gate, full-lifecycle court under wasmtime 48 + `wasm_exceptions`, WHERE-less UPDATE/DELETE `REFUSAL-05` pre-engine with DB byte-identical, MERGE mutating + epoch-bump court, artifact re-pin with witnessed receipt, pm4pytest FIBO matrix green ([designs/fibo-pm4py-court-matrix.md](designs/fibo-pm4py-court-matrix.md)) | [SQLRDF-21](SQLRDF-21.md) |
| SQLRDF-22 | FIBO-matrix compiler gaps — DECIMAL, COPY egress policy, multi-row INSERT, batch vocabulary, ATTACH path validation, egress digests | Open | Six audited gaps: no DECIMAL column type (`src/sql_ast.rs:1723-1735` allowlist; `xsd:decimal` quoted by `lit()` `src/sql_ast.rs:1420-1437`), COPY TO destination hardcoded (`src/sql_ast.rs:1948`, expression form `:1524` but no options/policy), single-row INSERT shapes (`statement_classes.ttl:145-160`), one statement per document (`src/sql_ast.rs:1380-1407`), scalar ATTACH hardcodes `:memory:` (`:2008`, expression form `:1673-1678` no path validation), `ExecutionReceipt` has no artifact-digest field (`src/duckdb_eval.rs:134-156`); each lands typed vocabulary or a documented typed refusal with a court; egress digest receipt-carried ([SQLRDF-22.md](SQLRDF-22.md)) | [SQLRDF-22](SQLRDF-22.md) |
| SQLRDF-23 | Statement sequences (G5) — multi-statement documents with ordered, receipted batch execution | Open | Verified design at [designs/statement-sequence-design.md](designs/statement-sequence-design.md): closes the silent multi-root pick (`src/sql_ast.rs:1376-1409`) with `sql:StatementSequence`/`sql:statements` vocabulary, `compile_turtle_document` (single-root back-compat bit-for-bit), domain-separated `doc_hash`, `BatchReceipt` with `failed_at` partial-commit semantics, per-member cache law, new `INDEX_SEQUENCE_DISORDER`/`SEQUENCE_EMPTY`/`MIXED_DOCUMENT_ROOTS` MalformedAst arms, court plan (a)-(f) | [SQLRDF-23](SQLRDF-23.md) |
| PRD-P28-SIL | Operator PRD — GraphLaw Substrate & P28-SIL Zero-Overhead Execution Architecture v1.0.0 (operative PRD for the successor program) | Open | Filed 2026-10-22 verbatim (126 lines, v1.0.0, supersedes the earlier skeleton) at [PRD-P28-SIL.md](PRD-P28-SIL.md); no re-dispatch needed | [PRD-P28-SIL](PRD-P28-SIL.md) |
| DISSERTATION | Operator dissertation — "Admitted Computation: The Canonical Graph as the Sole Substrate for Relational Execution" (verbatim, chapters 1–10 + abstract) | Open (filed; artifact mapping below) | Filed verbatim at [DISSERTATION.md](DISSERTATION.md). Chapter→program-artifact map: Ch1 introduction → this window's program frame; Ch2 admission calculus → [`ontologies/sql-ast.ttl`](../../../ontologies/sql-ast.ttl) + [`src/sql_ast.rs`](../../../src/sql_ast.rs); Ch3 receipt algebra → SQLRDF-09 (landed: canonical-AST-graph formula hash, [`src/duckdb_eval.rs`](../../../src/duckdb_eval.rs)); Ch4 one-description → designs E1/A1/E2; Ch5 executable conformance → design [E3-conformance.md](designs/E3-conformance.md) (rung skeleton); Ch6 content-addressed + evidentiary → designs [C1-cache.md](designs/C1-cache.md) + [P28-SIL-specs.md](designs/P28-SIL-specs.md); Ch7 single door → SQLRDF-17 / design [C2 hole P1](SQLRDF-17.md); Ch8 courts methodology → this window's court practice (RECEIPTS.md + mutation_court); Ch9 limitations → Σ_PQC / epoch / federation disclosures (SQLRDF-10 conformance note, SQLRDF-13..15 follow-ups); Ch10 conclusion → v26.10.22 acceptance boundary | [DISSERTATION](DISSERTATION.md) |

## Lanes

Wave map: 8 build lanes + 13 per-ticket DoD courts over one canonical checkout
(`/Users/sac/graphlaw`). Ticket Status fields stay **Open**; the coordinator
flips them at integration. Latest observed verdicts per ticket live in
[RECEIPTS.md](RECEIPTS.md).

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
  section rewritten to the live pin `332ea218…2c30c` (6,793,617 B);
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
  (`designs/registry-regen-receipt.md`).
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
  [designs/benchmark-census.md](designs/benchmark-census.md): six SPC
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

- [`docs/ARCHITECTURE.md`](../../../docs/ARCHITECTURE.md) — admission-law flow
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
- [COURT-SPEC.md](mega/tc-mega-fibo-01/COURT-SPEC.md) — CTQ status
  census at spec time: 1 WITNESSED (M2 count class) / 7 UNVERIFIED
  (M1, M3, M4, M5, M7, M8, M9) / 1 REFUTED (M6).
- [PILLARS-AUDIT.md](mega/tc-mega-fibo-01/PILLARS-AUDIT.md) — pillar
  audit; U5 records the wasm artifact not linking (`rust-lld
  -lduckdb`).
- [IMPLEMENTATION-PLAN.md](mega/tc-mega-fibo-01/IMPLEMENTATION-PLAN.md)
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
EH memo amendment is in-file (`designs/eh-runner-decision-memo.md:76`, L126
stand-down), not a separate record; the M/N receipt set is covered by
the `m-wave-verdicts` and N-lane rows below.

| Record | What it is | Status |
|---|---|---|
| [designs/eh-runner-decision-memo.md](designs/eh-runner-decision-memo.md) | EH runner decision memo (+ L126 amendment): DuckDB wasm artifact carries a Tag section wasmi cannot host; recommends a route, decision required | DECISION REQUIRED (A1 refuted by amendment) |
| [designs/a1prime-trytable-recipe.md](designs/a1prime-trytable-recipe.md) | A1' rebuild recipe: emsdk try_table/new-style wasm EH flags for a uniformly new-EH artifact hostable by V8 + modern wasmtime | DESIGN (read-only recipe, nothing landed) |
| [designs/dual-artifact-policy-design.md](designs/dual-artifact-policy-design.md) | Route C dual-artifact policy: legacy EH engine artifact + PURE-tier pre-duckdb module for the wasmi court tier; supersedes the memo route pick | DESIGN (nothing landed) |
| [designs/wasm-link-root-cause.md](designs/wasm-link-root-cause.md) | WASM-LINK-RC consolidated receipt: four-hop root cause of the `-lduckdb` link failure and the EH consequence chain | DOCUMENTED |
| [designs/integration-gonogo.md](designs/integration-gonogo.md) | Integration GO/NO-GO board re-executing unreceipted gates; companion to the commit manifest | CURRENT (board as-of 2026-10-22) |
| [designs/commit-manifest.md](designs/commit-manifest.md) | Commit manifest: 218-entry working set split into 8 dependency-ordered buckets | CURRENT (pre-commit prep) |
| [designs/commit-drafts-validation.md](designs/commit-drafts-validation.md) | Validation of the 8 commit drafts against `git status` at use time; 3 corrections applied | PASS (with corrections) |
| [designs/post-commit-runbook.md](designs/post-commit-runbook.md) | Post-commit runbook: coordinator-executed gate sequence after the wave commit, every step receipt-cited | CONSOLIDATION (not yet executed) |
| [designs/mutation-court-full-receipt.md](designs/mutation-court-full-receipt.md) | Full mutation-court run on the dirty wave tree: typed refusal exit 6 MUTATION-COURT-DIRTY-TREE — the gate fired as designed | TYPED REFUSAL (gate non-vacuous) |
| [designs/precommit-acceptance-receipt.md](designs/precommit-acceptance-receipt.md) | Pre-commit acceptance run of the 8-court sql-rdf set in an isolated target dir; per-target counts embedded | PASS (composite re-run per target) |
| [designs/pure-artifact-witness.md](designs/pure-artifact-witness.md) | PURE-tier packet-4 frame witness (N15): beam4pm artifact BLAKE3-matches `8b4f4bc8…`; no rebuilds or pin changes | WITNESSED |
| [designs/m-wave-verdicts.md](designs/m-wave-verdicts.md) | M-lane verdict receipts (M2/M6/M9/M10/M11 OUT; M3/M12 FILED; M4 OUT->FILED by L149: `designs/m4-wasm-boundary-receipt.md`) transcribed from lane command output | CURRENT (coordinator-filed) |
| [designs/stale-wasi-sweep.md](designs/stale-wasi-sweep.md) | Read-only stale WASI/frozen-duckdb reference sweep with exclusions honored | CURRENT (sweep report) |

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
| [designs/m4-wasm-boundary-receipt.md](designs/m4-wasm-boundary-receipt.md) | M4 wasm-boundary verdict receipt (closes the M4 OUT->FILED flip noted in m-wave-verdicts) | FILED (L149) |
| [designs/p7-final-regen-prestage.md](designs/p7-final-regen-prestage.md) | P7 pre-stage: final registry regen procedure, queued on the GO/NO-GO board row 47 | PRE-STAGE (not yet executed) |
| [designs/mutation-court-full-receipt.md](designs/mutation-court-full-receipt.md) | In-file amendments: N1 verification + blocked-until-commit; L151 mutant-scope freshness re-verification (corrects N1's "124" count) | AMENDED (L151) |
| [designs/OPERATOR-DECISIONS.md](designs/OPERATOR-DECISIONS.md) | Consolidated operator decision register: question, evidence, options, recommendation per entry; nothing decided by the lane | DECISION REGISTER (open) |
| [designs/laundering-count-reconcile.md](designs/laundering-count-reconcile.md) | Reconciles GO/NO-GO row 1 "laundering 11/11" against the M2/L147 witness at 23/0 | RECONCILED |
| [designs/staleness-76-triage.md](designs/staleness-76-triage.md) | Staleness-gate residual triage: live count 91 (not 76); rule split R2=48/R3=15/R4=28 | TRIAGED |
| [designs/g2-repin-prep.md](designs/g2-repin-prep.md) | G2 AXIOM-IMMUTABILITY re-pin prep: read-only prep, writes only `courts/code-partition/axioms.blake3.next` | PREP (runbook step 3) |
| [designs/tc-hot-full-receipt.md](designs/tc-hot-full-receipt.md) | "Durable corpus relocation (L159)" section: TC-HOT corpus moved to a durable path with reattempt leg L118 | AMENDED (L159) |
| [designs/registry-drift-scope.md](designs/registry-drift-scope.md) | Registry drift scope receipt: court run with embedded exits/digests bounding registry drift | RECEIPTED |
| [designs/repin-staging-gap.md](designs/repin-staging-gap.md) | Re-pin staging gap: wasi-sdk-34 vs emsdk-built staging; nothing re-pinned, every claim receipt-cited | GAP DOCUMENTED |
| [designs/spc-red-remediation-design.md](designs/spc-red-remediation-design.md) | SPC red remediation design for `spc_court.sh`/`tc_hot_01.rs`; design only, gate code untouched by the lane | DESIGN (nothing landed) |
| [designs/wasmi-eh-research.md](designs/wasmi-eh-research.md) | wasmi exception-handling research feeding the ENGINE-tier host decision (wasmi EH support, wasmtime 48 new-EH) | RESEARCH (landed since L165) |

### Session design records — post-L170 (2026-10-22, lane L171)

Each row verified present on disk via `ls -lt` + `test -f` before filing
(exits 0). Loud skips: `s2-provenance-census-verify` and
`s4-spc-design-verify` NOT landed (no file under `designs/`).

| Record | What it is | Status |
|---|---|---|
| [designs/eyeron-switchover-fixforward.md](designs/eyeron-switchover-fixforward.md) | N25 receipt: `cargo check -p graphlaw-eyeron` exit 0, 0 errors — the 31-error parser split freeze was already fixed forward by the switchover lane; remaining `*_split/` staging dirs intentionally unwired | FIX-FORWARD RECEIPT (0-error census) |
| [designs/receipt-surface-post-switchover.md](designs/receipt-surface-post-switchover.md) | N27 analysis: receipt-surface FILES pins hard-code only 2 of 5 compiler files (`src/registry/emit_json.rs:73-92`) → ~35.8% of the compiler surface unpinned; 5-entry FILES expansion PROPOSED-NOT-APPLIED (src not quiescent during analysis) | PROPOSAL (code change not applied) |
| [designs/switchover-provenance-census.md](designs/switchover-provenance-census.md) | N26 census: all 8 split trees WIRED (facade `mod`/`pub use` evidence per tree); executor of the wiring UNDETERMINED — no logged actor in the session transcript corpus; working tree NOT commit-ready as-manifested | CENSUS (executor UNDETERMINED — unlogged-actor question open) |
| [designs/s1-surface-analysis-verify.md](designs/s1-surface-analysis-verify.md) | S1 skeptic verification of N27: 35.8% pin-gap arithmetic re-derived from `wc -c` (exit 0); FILES pins + registry JSON rows + `tests/registry_artifacts.rs:890/907` confirms `files.len() == 2` — all N27 cites verified | CONFIRMED (skeptic pass) |
| [designs/s7-chain-board-check.md](designs/s7-chain-board-check.md) | S7 witness: `receipt_chain_sync.sh --check` 22/22 VERIFIED, exit 0, head `5931264f…` (chain grew 14 receipts from N14's baseline without breaking the fold); GO/NO-GO board re-read at ~20:05 — 17 READY items | VERIFIED (22/22, exit 0) |

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
| [designs/s2-provenance-census-verify.md](designs/s2-provenance-census-verify.md) | S2 skeptic verification of N26: independently re-derived all-8-trees-wired (facade `mod`/`pub use` evidence), zero writes to split dirs in the wiring window via its own jsonl sweep; abi/ops and reasoner/builtins are nested dirs (12 files, not 8 flat); out-of-band actor not identified — advises do NOT commit until the unlogged writer is found | CONFIRMED with corrections (actor question open) |
| [designs/s4-spc-design-verify.md](designs/s4-spc-design-verify.md) | S4 skeptic verification of `spc-red-remediation-design.md` (L169): 5 receipts re-parsed, p999=292 shelf CONFIRMED in 4 runs (stronger than claimed), 3 arithmetic corrections, composite tail-magnitude loophole found; measurement model stands, gate needs one more term before implementation | PARTIAL (design confirmed; composite gate loophole) |
| [designs/surface-six-entry-patch.md](designs/surface-six-entry-patch.md) | L172 staged patch expanding `src/registry/emit_json.rs` FILES const from 2 to 6 entries (closes the S1-confirmed 35.8% pin gap); DO-NOT-APPLY until quiescent window, sequenced after green compile and BEFORE any `graphlaw-registry --write` | STAGED (deferred, not applied) |

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
[designs/core-team-decomposition.md](designs/core-team-decomposition.md) and
[designs/deterministic-agent-admission.md](designs/deterministic-agent-admission.md).
The moonshot catalog rows are in [designs/INDEX.md](designs/INDEX.md).

Eight trees are `mod`-wired. Parents stayed `foo.rs` with children in
`foo/`. `*_split` folders are deleted. Public paths are unchanged. Receipt
field order and the SQL receipt formula were not edited.

| Cut | On disk | Still open |
|---|---|---|
| Reasoner | `limits.rs`, `builtins/{list,log,math,string}.rs`, `solve_context.rs` (dead draft) | `unify.rs`, `tabled.rs`, `BuiltinPredicate`, replacing the long `solve_backward_goal` parameters. `reasoner.rs` is still 4248 lines and still allows `clippy::too_many_arguments`. `builtins/mod.rs` is 946. |
| Parser | `cursor.rs` (438), `grammar.rs` (1500), entry points in `parser.rs` (342) | Further cut of `grammar.rs` |
| SQL AST | `sql_ast/{ast,refusals,serializer}.rs` | Prep-lane reader/dataset/compile cut, `src/sql/` rename, `builder.rs`. [sql-ast-decomposition.md](designs/sql-ast-decomposition.md) |
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
[code-partition-census.md](designs/code-partition-census.md). The ratio has
not inverted. G5 prints it and does not call it a pass.

### Session lithography (2026-10-22)

Filed at [designs/session-lithography.md](designs/session-lithography.md).
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
exists on disk; all 61 relative `.md` links in this index resolve (verified by
`test -f` per link). Top-level files present on disk but not named anywhere
above:

- [`_LANES.md`](_LANES.md) — lane lease map for this window
- [`CHARTER-DFLSS.md`](CHARTER-DFLSS.md) — DFLSS charter
- [`DFLSS-DOSSIER.md`](DFLSS-DOSSIER.md) — DFLSS dossier (Define/Measure/Explore/Develop/Implement records)
- [`RELEASE-NOTES-DRAFT-26.10.21.md`](RELEASE-NOTES-DRAFT-26.10.21.md) — release-notes draft
- [`RUST-4-PHASE2-HARDENING.md`](RUST-4-PHASE2-HARDENING.md) — Rust phase-2 hardening ticket

(`DISSERTATION.md`, `PRD-P28-SIL.md`, `RECEIPTS.md`, `THESIS.md`,
`SQLRDF-01..20.md`, `mega/`, and `designs/` are indexed above.)

### Designs appendix — records not named in the body (index re-sync, lane D3)

76 of the 133 files under `designs/` were not named anywhere above. Titles
below are each file's own H1 (verified via `head -1`); statuses are not
re-adjudicated here — see [designs/RECEIPTS.md](designs/RECEIPTS.md) and
[RECEIPTS.md](RECEIPTS.md) for verdicts.

| Record | H1 (verbatim) |
| --- | --- |
| [A1-scope-as-type.md](designs/A1-scope-as-type.md) | A1 — scope-as-type via entailment closure (= SQLRDF-13 implementation plan) |
| [advisory-update-receipt.md](designs/advisory-update-receipt.md) | Advisory Update Receipt — Lane L103 (L63 residual) |
| [c4-architecture.md](designs/c4-architecture.md) | C4 Architecture Documentation — Deterministic Agent Governance Platform |
| [c4-reconciliation.md](designs/c4-reconciliation.md) | C4 reconciliation: c4-architecture.md vs. repo state |
| [cargo-deny-receipt.md](designs/cargo-deny-receipt.md) | cargo-deny Receipt (RUST-8, lane L63) |
| [claude-md-ladder-receipt.md](designs/claude-md-ladder-receipt.md) | CLAUDE.md Ladder Receipt — Lane L50 |
| [claude-policy-commit-note.md](designs/claude-policy-commit-note.md) | .claude/ Commit Policy — GO/NO-GO Operator Decision 3 |
| [clippy-zero-receipt.md](designs/clippy-zero-receipt.md) | Clippy-zero receipt — lane L130 |
| [code-first-refusals.md](designs/code-first-refusals.md) | Code-first refusals & binary-first serialization — P29-RFL (design record) |
| [commit-receipt-provenance-gap.md](designs/commit-receipt-provenance-gap.md) | Commit receipt — staged provenance-gap section (lane L176) |
| [cpk-correction.md](designs/cpk-correction.md) | Cpk capability analysis — corrected (DFLSS Measure-lane A record) |
| [ctq-traceability.md](designs/ctq-traceability.md) | CTQ → Court Traceability Matrix (DFLSS Define-lane A record) |
| [defect-pareto-verified.md](designs/defect-pareto-verified.md) | Defect Pareto — witnessed vs charter (DFLSS Define-lane B record) |
| [E1-alias-elimination.md](designs/E1-alias-elimination.md) | E1 — alias-window elimination (design record) |
| [E2-shapes-reader.md](designs/E2-shapes-reader.md) | E2 Design — Generate the Property-Path Reader from the SHACL Shapes |
| [eyeron-pin-reconciliation.md](designs/eyeron-pin-reconciliation.md) | Eyeron Pin Reconciliation (L59) |
| [gate-rule-changes-receipt.md](designs/gate-rule-changes-receipt.md) | Receipt: doc_staleness_gate rule changes (L168) |
| [ggen-pack-drift.md](designs/ggen-pack-drift.md) | ggen/sql-dialect pack vs sql-rdf compiler — drift report |
| [ggen-pack-reconciliation.md](designs/ggen-pack-reconciliation.md) | ggen/sql-dialect pack drift — reconciliation plan (L61) |
| [ggen-toml-tempath-proposal.md](designs/ggen-toml-tempath-proposal.md) | Proposal: normalize wasm/ggen.toml paths (L160) |
| [graphlaw-admission-blueprint.md](designs/graphlaw-admission-blueprint.md) | graphlaw Admission Blueprint — CONSTRUCT-OPERATIONAL-BLUEPRINT |
| [graphlaw-run-live-receipt.md](designs/graphlaw-run-live-receipt.md) | graphlaw-run Live Execution Receipt (lane L115) |
| [gws4-postlink-receipt.md](designs/gws4-postlink-receipt.md) | gws4 post-link receipt — L116 |
| [hot-path-baseline.md](designs/hot-path-baseline.md) | Hot-path measured baseline (DMEDI Measure) |
| [js-test-receipt.md](designs/js-test-receipt.md) | JS Test Receipt — L107 |
| [link-green-receipts.md](designs/link-green-receipts.md) | Link-green receipt set (coordinator-filed from lane reports) |
| [m12-staleness-receipt.md](designs/m12-staleness-receipt.md) | M12 — doc_staleness_gate Receipt |
| [m3-shapes-receipt.md](designs/m3-shapes-receipt.md) | M3 Shapes No-Drift Receipt |
| [merge-fixture-fix-receipt.md](designs/merge-fixture-fix-receipt.md) | Merge Fixture Fix Receipt (L127) |
| [merge-shapes-receipt.md](designs/merge-shapes-receipt.md) | MERGE Shape Coverage Receipt (Lane L123) |
| [n28-testhost-eh-fix.md](designs/n28-testhost-eh-fix.md) | N28 — test-host exceptions gate: diagnosis receipt |
| [p28-sil-capability-audit.md](designs/p28-sil-capability-audit.md) | P28-SIL v2.1.0 capability audit — consolidated record (12 lanes) |
| [pce-baseline.md](designs/pce-baseline.md) | PCE baseline — charter "<2.8%" REFUTED at the measured operating point |
| [peg-kinds-claim-receipt.md](designs/peg-kinds-claim-receipt.md) | Receipt: L60 — "35 PEG statement kinds" claim audit |
| [R2-capability-routes.md](designs/R2-capability-routes.md) | R2 — capability route table (design record) |
| [receipt-surface-pin.md](designs/receipt-surface-pin.md) | Receipt-surface digest pin (P28-SIL §4 circumvention witness) |
| [receipts-chain-audit.md](designs/receipts-chain-audit.md) | Receipts chain audit — lane L31 |
| [rules-pruner-design.md](designs/rules-pruner-design.md) | C23 Harness Pruner — Design (L34) |
| [rust-core-audit.md](designs/rust-core-audit.md) | Rust core-team audit — consolidated record (8 lanes, post-v26.10.22) |
| [s10-index-rows-verify.md](designs/s10-index-rows-verify.md) | S10 skeptic — adversarial verification of L174's 3 INDEX rows |
| [s11-bench-restore-verify.md](designs/s11-bench-restore-verify.md) | S11 skeptic verification — N22 restore claim, benches/tc_hot_02.rs |
| [s13-spc-amendment-verify.md](designs/s13-spc-amendment-verify.md) | S13 skeptic — verification of Amendment 1 vs s4-spc-design-verify.md |
| [s14-provenance-note-verify.md](designs/s14-provenance-note-verify.md) | s14-provenance-note-verify — skeptic verification of L176's staged provenance-gap note |
| [s15-manifest-refresh-verify.md](designs/s15-manifest-refresh-verify.md) | S15 — Skeptic verification of commit-manifest refresh #3 (L173) |
| [s16-census-receipt-verify.md](designs/s16-census-receipt-verify.md) | S16 — skeptic census of L110's sweep-post-link receipt |
| [s17-new-designs-verify.md](designs/s17-new-designs-verify.md) | S17 — provenance verification of two unassigned design records |
| [s8-unlogged-writer-watch.md](designs/s8-unlogged-writer-watch.md) | S8 — Unlogged Writer Watch (observation lane) |
| [s9-patch-verify.md](designs/s9-patch-verify.md) | S9 patch verify — L172 staged 6-entry surface patch (skeptic, read-only) |
| [shapes-rec-drift.md](designs/shapes-rec-drift.md) | Shapes ↔ Rec §4 Drift Report (Lane L33) |
| [shapes-rec-reconciliation.md](designs/shapes-rec-reconciliation.md) | Shapes ↔ Rec §4 Reconciliation Plan (Lane L55) |
| [SKEPTIC-thesis-review.md](designs/SKEPTIC-thesis-review.md) | Skeptic review — moonshot thesis (design record) |
| [smalt-pairing-receipt.md](designs/smalt-pairing-receipt.md) | Smalt-Graphlaw Pairing Receipt (Lane L121) |
| [spc-append-only-fix.md](designs/spc-append-only-fix.md) | L111: spc_court.sh append-only receipts (L108 structural finding, fixed) |
| [spc-mktemp-fix.md](designs/spc-mktemp-fix.md) | SPC Court mktemp Robustness Fix — L57 Receipt |
| [spof-scan.md](designs/spof-scan.md) | SPOF scan — consolidated record (8 lanes, v26.10.22) |
| [stale-claims-audit.md](designs/stale-claims-audit.md) | Stale-claims audit — v26.10.22 (skeptic lane, read-only) |
| [sweep-post-link.md](designs/sweep-post-link.md) | Sweep post-link: L110 witness of the wasm tier flip |
| [t1-mutation-full-receipt.md](designs/t1-mutation-full-receipt.md) | T1 Full Mutation-Court Receipt - PARTIAL (DISK-BLOCKED) |
| [t4-refusal-reaudit.md](designs/t4-refusal-reaudit.md) | T4 refusal-register re-audit (post-switchover drift check) |
| [tc-hot-baseline.md](designs/tc-hot-baseline.md) | TC-HOT-01 baseline receipt (lane L20) |
| [TC-HOT-benchmarks.md](designs/TC-HOT-benchmarks.md) | TC-HOT-01/02 benchmark harness design (design record) |
| [TC-MUT-01-mapping.md](designs/TC-MUT-01-mapping.md) | TC-MUT-01 / TC-MOCK-00 mapping (design record) |
| [twenty-agent-scaling-thesis.md](designs/twenty-agent-scaling-thesis.md) | The Leverage Gap — 20 Parallel Instances Under a Deterministic Substrate |
| [v2030-fs-boundary-shapes.md](designs/v2030-fs-boundary-shapes.md) | V2030-FS-BOUNDARY-SHAPES receipt (Lane V2030-2, Vision-2030 step 2) |
| [v2030-receipt-pipeline.md](designs/v2030-receipt-pipeline.md) | Vision-2030 Step 3 — Receipt Pipeline Deployment (V2030-3) |
| [v2030-zero-copy-protocol.md](designs/v2030-zero-copy-protocol.md) | Vision-2030 Step 1 — Zero-Copy Shared-Memory Protocol Design |
| [vecgen-v1-backmigration.md](designs/vecgen-v1-backmigration.md) | vecgen-v1-backmigration — lane N17 receipt |
| [vl18-registry-regen.md](designs/vl18-registry-regen.md) | V-L18 — independent registry regen post-switchover |
| [vl4-mutation-receipt.md](designs/vl4-mutation-receipt.md) | V-L4 Mutation Court Receipt |
| [wasi-sdk-reproducibility.md](designs/wasi-sdk-reproducibility.md) | wasi-sdk reproducibility: scripts/build_wasi_duckdb.sh (L109) |
| [wasm-duckdb-op-wired.md](designs/wasm-duckdb-op-wired.md) | wasm duckdb op wired — receipt (L125) |
| [wasm-repin-receipt.md](designs/wasm-repin-receipt.md) | Wasm re-pin attempt — receipt (L122) |
| [wo01-admission-run.md](designs/wo01-admission-run.md) | WO-01 — full-shape-corpus admission run (lane WO-01) |
| [wo03-rss-probe.md](designs/wo03-rss-probe.md) | WO-03 RSS Expansion Probe |

### Index re-sync — lane R4 (2026-10-22)

Re-synced against disk (D3 discipline: `ls` + `head -1` per file before
filing). Artifacts landed since the D3 sync that were not named anywhere
above:

**Archive** ([docs/archive/](../../../docs/archive/README.md)) — moved by the
coordinator (`git mv`), historical narrative and receipts retired out of the
live tree:

- [`THESIS.md`](../../../docs/archive/THESIS.md) — moonshot thesis (verbatim body), moved from `docs/THESIS.md`
- [`UNIFIED_DISSERTATION.md`](../../../docs/archive/UNIFIED_DISSERTATION.md) — earlier unified dissertation draft, moved from `docs/UNIFIED_DISSERTATION.md`
- [`EXTRACTION.md`](../../../docs/archive/EXTRACTION.md) — GraphLaw extraction receipt
- [`branch-disposition.md`](../../../docs/archive/branch-disposition.md) — branch disposition record
- [`receipt-2026-10-09-receipt-chain-and-refuse-hooks.md`](../../../docs/archive/receipt-2026-10-09-receipt-chain-and-refuse-hooks.md) — session receipt: receipt-chain port + refuse hooks (2026-10-09)
- [`receipt-2026-10-10-publish-dod.md`](../../../docs/archive/receipt-2026-10-10-publish-dod.md) — v26.10.10 publish Definition-of-Done receipt (skeleton)
- [`README.md`](../../../docs/archive/README.md) — archive readme

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
- [`SPLIT-NOTES.md`](../../../SPLIT-NOTES.md) — Lane L46 phase-4 extraction notes (UNWIRED by directive)
| SQLRDF-25 | Dialect expansion — window functions, QUALIFY, star modifiers, CTEs, PIVOT/UNPIVOT, composite literals, JSON operators | Open | Seven verified gaps, no AST terms/compile targets in [`src/sql_ast.rs`](../../../src/sql_ast.rs) (engine natively supports all; wasm artifact already executes them): window OVER/frames, QUALIFY, `* EXCLUDE/REPLACE` + `COLUMNS('regex')`, WITH/RECURSIVE CTEs, PIVOT/UNPIVOT, struct/map/list literals + slicing, JSON `->`/`->>`. REFUTED (already implemented, not re-asked): EXPLAIN/PRAGMA/SET/INSTALL/LOAD (`src/sql_ast.rs:1854-1932`, renderers :2250-2312, `LOAD icu` witnessed in-wasm), MERGE INTO (§7 shapes, extract_merge_statement, epoch + upsert courts), statement sequences (SQLRDF-23 landed). Per gap: ggen dialect shapes + sql_ast compile targets + SHACL admission + real-readback courts with anti-vacuity twins; engine and admission law untouched ([SQLRDF-25.md](SQLRDF-25.md)) | [SQLRDF-25](SQLRDF-25.md) |
| SQLRDF-24 | Artifact trap: `uninitialized element` on first `gl_call` (indirect-call vtable) | Open | DuckDB-linked artifact (46,015,267 B; 46,069,809 B new-EH, Tag×1) traps on FIRST `gl_call` with `wasm trap: uninitialized element` — `call_indirect` through a vtable slot read from linear memory inside the `__cxa_thread_atexit` destructor-list runner (old-artifact func 68925, new ~68950), reached from the wasi-libc `gl_*` export epilogue (dtors + stdio flush); `Table[0]` permanently null, op-agnostic; V8 and wasmtime both trap → in-module defect; exonerated: EH-flavor mixing, LTO (false/cgu16/thin), stack size, V8 hosting; in flight: dtor-list memory dump host + gc-sections probe (`-C link-dead-code`, `--no-gc-sections`; gc-sections dropping table entries while the data-segment vtable survives = leading mechanism hypothesis); acceptance (a)-(d) in [SQLRDF-24.md](SQLRDF-24.md); no artifact re-pin until (b) | [SQLRDF-24](SQLRDF-24.md) |

### Status refresh 2026-10-23 (EH-closure wave)

Per-ticket standing as of the EH-closure wave. Receipts:
[RECEIPTS.md](RECEIPTS.md) lines 1169-1340 and the wave receipt
[receipts/2026-10-23-eh-closure-wave.md](../../../receipts/2026-10-23-eh-closure-wave.md).

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
