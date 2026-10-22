# SQL-in-RDF tickets — v26.10.22

Status of every ticket: **Open**.

The conformance target is the pasted sql-rdf Recommendation, the conformance text for v26.10.22. Section citations in the tickets point at that text. SQLRDF-12 files it at `docs/reference/sql-rdf-rec.md`.

These surfaces already exist, and the tickets do not ask for them again:

- [`src/sql_ast.rs`](../../../src/sql_ast.rs) compiles `sql:selectList`, `sql:fromTable`, one `sql:joinClause`, `sql:whereClause`, `sql:groupBy`, `sql:limit`, and `sql:offset` for `SELECT` (`SelectStatement::compile` at lines 222–290, `extract_select_statement` at lines 499–616).
- The other 32 runnable PEG classes compile as statement templates in `DatasetReader::compile_command` (lines 771–911) and run on in-memory frozen-duckdb.
- `CONNECT` and `DISCONNECT` are compile refusals inside `compile_turtle` (lines 667–671).
- Chicago receipts in [`tests/chicago_validation.rs`](../../../tests/chicago_validation.rs) hash SQL text plus a table readback (`state_receipt`, lines 127–151).
- The wasm crate does not enable `duckdb` ([`wasm/Cargo.toml`](../../../wasm/Cargo.toml) line 19).

Compiler files the tickets point at:

- [`src/sql_ast.rs`](../../../src/sql_ast.rs)
- [`ontologies/sql-ast.ttl`](../../../ontologies/sql-ast.ttl)
- [`src/duckdb_eval.rs`](../../../src/duckdb_eval.rs)
- [`docs/refusals.md`](../../refusals.md)
- [`tests/chicago_validation.rs`](../../../tests/chicago_validation.rs)
- [`wasm/Cargo.toml`](../../../wasm/Cargo.toml)

| Id | Title | Status | Gap | Ticket |
| --- | --- | --- | --- | --- |
| SQLRDF-01 | Normative vocabulary | Open | was: Section 3.1 and 3.3 names absent → now present in [`ontologies/sql-ast.ttl`](../../../ontologies/sql-ast.ttl) (legacy aliases retained in the alias window); remaining hop is compiler-side per [RECEIPTS.md](RECEIPTS.md) | [SQLRDF-01](SQLRDF-01.md) |
| SQLRDF-02 | Closed-world SHACL from section 4 | Open | was: no `statementIndex`, projection-list, operator-token, or `rdf:List` shapes → shapes now landed in [`ontologies/sql-ast.ttl`](../../../ontologies/sql-ast.ttl) (`sql:StatementBaseShape`, `sql:SelectStatementShape`, `sql:BinaryExpressionShape`, `sql:statementIndex`); remaining hop is compiler-side (`REFUSAL-01` on missing index) per [RECEIPTS.md](RECEIPTS.md) | [SQLRDF-02](SQLRDF-02.md) |
| SQLRDF-03 | Clause compiler for having, orderBy, and sql:from | Open | was: `HAVING`, `ORDER BY`, schema-qualified `sql:from`, and scope IRIs unread → closed by compiler support in [`src/sql_ast.rs`](../../../src/sql_ast.rs), courted by `cargo test --test sql_rdf_dialect` (18/18) plus the HAVING court | [SQLRDF-03](SQLRDF-03.md) |
| SQLRDF-04 | Section 7 analytical MERGE | Open | was: merge was a one-row template, not the section 7.1 parameterized statement → closed by the parameterized MERGE compiler in [`src/sql_ast.rs`](../../../src/sql_ast.rs) / [`src/duckdb_eval.rs`](../../../src/duckdb_eval.rs), courted by `cargo test --test chicago_validation` (9/9) plus the receipt courts | [SQLRDF-04](SQLRDF-04.md) |
| SQLRDF-05 | Expression-level graphs for the other admitted classes | Open | was: the remaining admitted classes compiled as fixed templates → closed by expression-level graphs in [`src/sql_statement_kinds.rs`](../../../src/sql_statement_kinds.rs) (env-grammar classes), courted at 5/6 in [`tests/duckdb_statement_kinds.rs`](../../../tests/duckdb_statement_kinds.rs) | [SQLRDF-05](SQLRDF-05.md) |
| SQLRDF-06 | REFUSAL-02 for ConnectStatement and DisconnectStatement | Open | was: compile refused both classes without the stable `REFUSAL-02` code → closed by the register-aligned typed refusal carrier in [`src/sql_ast.rs`](../../../src/sql_ast.rs) / [`docs/refusals.md`](../../refusals.md), courted by the REFUSAL-02 branches in [`tests/sql_rdf_dialect.rs`](../../../tests/sql_rdf_dialect.rs) | [SQLRDF-06](SQLRDF-06.md) |
| SQLRDF-07 | REFUSAL-03 scope integrity | Open | was: aggregates and `sql:ProcessScope` in the select list were not refused → closed by select-list scope integrity in [`src/sql_ast.rs`](../../../src/sql_ast.rs), courted by `cargo test --test sql_rdf_dialect` (9/9) plus the evasion set and join courts | [SQLRDF-07](SQLRDF-07.md) |
| SQLRDF-08 | REFUSAL-04 on the wasm build | Open | was: a `sql` document could be admitted on `graphlaw-wasm` with no capability refusal → closed by the capability refusal in [`wasm/src/lib.rs`](../../../wasm/src/lib.rs), courted by the wasm refusal courts (7/7) in `cargo test -p graphlaw-wasm` | [SQLRDF-08](SQLRDF-08.md) |
| SQLRDF-09 | Admitted-statement receipt | Open | was: the execution hash covered SQL text and parameters, not the canonical AST graph → closed by the formula hash over the canonical AST graph in [`src/duckdb_eval.rs`](../../../src/duckdb_eval.rs), witnessed by the formula receipt and courted by `cargo test --test sql_rdf_dialect` (18/18) | [SQLRDF-09](SQLRDF-09.md) |
| SQLRDF-10 | Section 1.4 post-quantum identity is unmet | Open | was: `Σ_PQC` had no conformance note → closed by the conformance note filed in [`docs/refusals.md`](../../refusals.md) (§1.4 identity), courted 10/0 in the attestation courts | [SQLRDF-10](SQLRDF-10.md) |
| SQLRDF-11 | Chicago extension court through compiled AST graphs | Open | was: section 6.2 checks ran as raw SQL and could skip when unsigned extensions were off → closed by §6.2 courts running compiled AST graphs over real frozen-duckdb, courted by [`tests/chicago_validation.rs`](../../../tests/chicago_validation.rs) | [SQLRDF-11](SQLRDF-11.md) |
| SQLRDF-12 | File the conformance text | Open | was: the pasted Recommendation had no stable path in the tree → closed by [`docs/reference/sql-rdf-rec.md`](../../reference/sql-rdf-rec.md) plus the citation gates in this window's tickets | [SQLRDF-12](SQLRDF-12.md) |
| SQLRDF-13 | Validation-scoped RDFS closure for §4 sh:class constraints | Open (follow-up, not in v26.10.22 acceptance) | `sh:class sql:Expression`/`sql:ASTNode` withheld at `ontologies/sql-ast.ttl:315-330` (asserted-data-only validation) | [SQLRDF-13](SQLRDF-13.md) |
| SQLRDF-14 | §3.1 letter conformance — re-parent the 35 PEG leaf classes under the six family classes | Open (follow-up, not in v26.10.22 acceptance) | The 35 PEG leaves generate flat under `sql:Statement` with no family ancestor | [SQLRDF-14](SQLRDF-14.md) |
| SQLRDF-15 | §6.2 tpch setup divergence — retire the sf=0.01 hand-patch into the generator + register | Open (follow-up, not in v26.10.22 acceptance) | Hand-patch in `ggen/sql-dialect/sql_extensions.rs:22-26` disagrees with `extract_statements.py:31`; the 0-row sf=0 witness is contradicted by upstream dbgen.cpp, so the divergence is unadjudicated (frozen pin sources absent) | [SQLRDF-15](SQLRDF-15.md) |
| SQLRDF-16 | Moonshot backlog — the 1000× ERRC horizon (thesis Part II) | Open (follow-up, BLOCKED-BY: v26.10.22 integration commit) | Eight moonshots (E1/E2/E3/R1/R2/A1/C1/C2) registered with falsifiers + dependency order E1→A1→E2→E3, R1→C1, R2→C2; thesis at [THESIS.md](THESIS.md) (verbatim body pending — dispatch did not carry the prose) | [SQLRDF-16](SQLRDF-16.md) |
| SQLRDF-17 | ABI duckdb exec accepts raw SQL text (C2 hole P1) | Open (follow-up, BLOCKED-BY: v26.10.22 integration commit) | `src/duckdb_op.rs:198-219` accepts `ast.text` as raw SQL (legacy `(None,None)` hash arm at `src/duckdb_eval.rs:183-188`); dialect sniff misroutes bare SQL to Sparql (`src/dialect.rs:740-758`, no `BareSql` class); `raw_connection()` is `pub` (`src/duckdb_eval.rs:155`) — spin-out of SQLRDF-16 row C2 | [SQLRDF-17](SQLRDF-17.md) |
| SQLRDF-18 | P28-SIL program — substrate + zero-overhead execution architecture | Open (successor program, BLOCKED-BY: v26.10.22 integration commit except LICENSE.md + TC-LEGAL-01 + baseline measurement, which proceed now) | PRD "GraphLaw Substrate & P28-SIL Zero-Overhead Execution Architecture" v1.0.0 filed verbatim at [PRD-P28-SIL.md](PRD-P28-SIL.md); TC-LEGAL-01/02 courts landed (3/0); TC-MOCK-00 gate landed; Phase 3 residue closed (corpus 14+/0 inline flips + shape_mutation_court.sh); Phase 4 license packaged (LICENSE-P28-SIL.md); Phase 2 skeleton implemented (ring+ledger+join courts green); Phase 1 designed (designs/PHASE1-generated-gate.md), BLOCKED-BY E1→A1→E2; TC-HOT benches landed, budget adjudication pending full-corpus run; §3.1 builds on landed R1 canon-once (`src/sql_ast.rs:1447-1449`) | [SQLRDF-18](SQLRDF-18.md) |
| SQLRDF-19 | CI feature-matrix gaps — untested build cells | Open (follow-up, BLOCKED-BY: v26.10.22 integration commit) | PR CI runs one native combo (`pack-tools,abi,wasi-patched-deps`, ci.yml:29/37); `--all-features` (ring, runner) is release-only (release.yml:85); untested cells: `runner` bin zero coverage, ring-without-duckdb, ring+duckdb seam, `--no-default-features`, pairwise off-default cells | [SQLRDF-19](SQLRDF-19.md) |
| SQLRDF-20 | BEAM hot-upgrade protocol — artifact swap under the supervisor, with its own court | Open (follow-up, BLOCKED-BY: v26.10.22 integration commit + SQLRDF-19) | Upgrade is a versioned release event (digest re-pin + ARTIFACTS row + changelog before first admission); evidentiary continuity via host-side ring/ledger; required court: `--expect-blake3` pin + admission-table version in first post-upgrade ledger batch + receipt chain across the boundary ([SQLRDF-20.md](SQLRDF-20.md)) | [SQLRDF-20](SQLRDF-20.md) |
| PRD-P28-SIL | Operator PRD — GraphLaw Substrate & P28-SIL Zero-Overhead Execution Architecture v1.0.0 (operative PRD for the successor program) | Open (verbatim body pending — dispatch carried structure only) | Filed at [PRD-P28-SIL.md](PRD-P28-SIL.md); re-dispatch the prose to complete | [PRD-P28-SIL](PRD-P28-SIL.md) |
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
