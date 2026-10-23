# Design: FIBO banking validation via pm4pytest — court matrix

Status: DESIGN (courts landing this wave under
[SQLRDF-21](../SQLRDF-21.md)). Nothing in this file is a landed receipt;
acceptance (f) of SQLRDF-21 is the matrix run green.

## Goal

Validate the sovereign in-WASM DuckDB execution surface against an
independent external oracle: pm4pytest (Petri-net conformance checking,
living in `~/ggen-marketplace`) over FIBO (Financial Industry Business
Ontology) banking scenarios. The engine under test is the wasm module; the
oracle is a process-mining conformance checker that never sees engine
internals — it sees only the emitted event streams and receipts.

## Four-axis combinatorial matrix

| Axis | Values |
| --- | --- |
| Dialect | sql-in-rdf (compiled AST graph), bare-SQL refusal cell (must refuse per SQLRDF-17) |
| SQL lifecycle | CREATE+INSERT, SELECT, UPDATE, MERGE, DELETE-guarded, DELETE-unbounded (refusal cell) |
| Safety guard | WHERE-bound mutation (pass), WHERE-less UPDATE/DELETE `REFUSAL-05` (refuse), raw-SQL text (refuse) |
| FIBO entity | Account, Party, Transaction, Loan, CounterpartyRelationship |

Full cross is not run cell-by-cell; the TC-FIBO table below selects the
adversarial cells. A cell whose expected state is a refusal must still
produce a pm4pytest-comparable event (the typed refusal is the event).

## Scenarios

- **Alpha** — account lifecycle: CREATE Account + Transaction tables,
  INSERT rows, SELECT balance aggregate, UPDATE one balance (WHERE-bound).
  Expected: conformance 1.0 on the happy-path process model.
- **Beta** — loan mutation with guard: UPDATE Loan status WHERE-bound;
  a WHERE-less UPDATE variant must refuse `REFUSAL-05` pre-engine with the
  DB byte-identical pre/post (SQLRDF-21 acceptance c).
- **Gamma** — MERGE epoch: MERGE into CounterpartyRelationship; MERGE is
  classified mutating and bumps the epoch counter (SQLRDF-21 acceptance d);
  the epoch delta is an event in the pm4pytest log.
- **Delta** — laundering/refusal: raw-SQL text cell must refuse (no
  admission, SQLRDF-17 stands); unbounded DELETE cell must refuse
  `REFUSAL-05`; both refusals are emitted as typed refusal events.

## TC-FIBO-01..10

| TC | Axis cells | Expected engine state | pm4pytest conformance criterion |
| --- | --- | --- | --- |
| TC-FIBO-01 | sql-in-rdf × CREATE+INSERT × pass × Account | statements execute in-module; rows read back | fitness = 1.0 on the Alpha happy-path Petri net |
| TC-FIBO-02 | sql-in-rdf × SELECT × pass × Transaction | aggregate SELECT returns real readback | fitness = 1.0; log replay fits the model |
| TC-FIBO-03 | sql-in-rdf × UPDATE × WHERE-bound × Account | update lands; table state changed | fitness = 1.0; no token shortages on the update transition |
| TC-FIBO-04 | sql-in-rdf × UPDATE × WHERE-less × Account | `REFUSAL-05` pre-engine; DB byte-identical pre/post | refusal event fits the refusal subprocess; precision = 1.0 |
| TC-FIBO-05 | sql-in-rdf × DELETE × WHERE-bound × Transaction | guarded delete lands | fitness = 1.0 |
| TC-FIBO-06 | sql-in-rdf × DELETE × WHERE-less × Transaction | `REFUSAL-05` pre-engine; DB byte-identical | refusal subprocess conformance, precision = 1.0 |
| TC-FIBO-07 | sql-in-rdf × MERGE × pass × CounterpartyRelationship | MERGE executes mutating; epoch bumps | epoch-delta event present; fitness = 1.0 |
| TC-FIBO-08 | sql-in-rdf × MERGE × pass × Loan | loan merge replayable (durable session, not per-call in-memory — F2 residue of SQLRDF-17) | fitness = 1.0 across replayed log |
| TC-FIBO-09 | bare-SQL × lifecycle × raw text × Party | typed refusal, never executed (SQLRDF-17) | refusal event; precision = 1.0 (no unmodeled behavior) |
| TC-FIBO-10 | sql-in-rdf × full lifecycle × pass × Loan | CREATE→INSERT→SELECT→UPDATE under wasmtime 48 + `wasm_exceptions` | end-to-end fitness = 1.0; pm4pytest log matches receipt chain |

## Oracle contract

- **pm4pytest** lives in `~/ggen-marketplace`; it checks Petri-net
  conformance with **fitness = precision = 1.0** as the green bar.
- **Independent receipt/tamper verification**: the oracle verifies the
  receipt chain (BLAKE3 over `Canon(G_AST) || P_params`) without engine
  cooperation — tampering a receipt byte must flip the conformance verdict
  to red (anti-vacuity: a matrix that cannot flip on tamper is FAIL, same
  law as `bash scripts/mutation_court.sh`).
- The oracle runs out-of-process; a loud SKIP with a typed reason (e.g.
  oracle unavailable) is not a pass.

## Honest boundaries

- **COPY TO egress digest must be receipt-carried.** Any `COPY TO`
  egress cell emits the artifact digest inside the receipt, not as a
  side-channel stdout claim; the oracle consumes the receipt-carried
  digest.
- **"42 ns admission" is aspirational.** It is a design target, not a
  measured figure. The only measured gate latency claim this window may
  make comes from the SPC court (`bash scripts/spc_court.sh`, Cpk ≥ 1.33
  on the receipt fold path). No other latency number is asserted.
- This matrix validates execution and refusal behavior; it does not
  re-litigate admission (SQLRDF-17 stands) or the one-unit link law
  (SQLRDF-08 stays Withdrawn).

## Fixtures materialized (2026-10-23)

Six TTL fixtures under `tests/fixtures/fibo/` (tc01_create.ttl,
tc01_insert.ttl, tc02_update_bounded.ttl, tc03_delete_unbounded.ttl,
tc05_select_sum.ttl, tc08_copy.ttl), each with its expected SQL in the
header comment. Per-file status against the on-disk compiler
(`src/sql_ast.rs`):

| Fixture | Form | Status |
|---|---|---|
| tc01_create.ttl | expression (`sql:tableDefinition`) | ADMITS. Type allowlist has DECIMAL (G1)
  (G1, `sql_ast.rs` allowlist arm). Expected:
  `CREATE TABLE fibo_accounts (id INTEGER, balance DECIMAL)` |
| tc01_insert.ttl | scalar assign pair | ADMITS. `xsd:decimal` with lossless lexical renders
  unquoted via render_scalar_literal (SQLRDF-22 G1). Expected:
  `INSERT INTO fibo_accounts (balance) VALUES (1250.50)` |
| tc02_update_bounded.ttl | both predicates | ADMITS. Compiler law is stricter than the shape: BOTH
  whereColumn AND whereValue required
  (`compile_command` UpdateStatement guard). Expected:
  `UPDATE fibo_accounts SET balance = 99000.25 WHERE facility_id = 'FAC-CORP-9882'` |
| tc03_delete_unbounded.ttl | no WHERE | REFUSES. **Correction**: the refusal fires at the SHACL
  layer first — the DeleteStatementShape `sh:or` (minCount 1 on
  whereColumn OR whereValue) fails validation, producing
  MalformedAst/SHACL_VIOLATIONS whose first violation message carries
  `[REFUSAL-05] UnboundedMutationRefusal`. The compiler's own
  RefusalCode::UnboundedMutation arm is defense in depth behind the
  shape. **Beta courts must assert on the message string, not
  RefusalCode::UnboundedMutation.** |
| tc05_select_sum.ttl | sec 7 aggregate + legacy alias | ADMITS by inspection — UNVERIFIED-by-eye on
  finish_alias (`src/sql_ast.rs:533`); execution pending a consuming
  court. Expected: `SELECT SUM(balance) AS total_balance FROM
  fibo_accounts` |
| tc08_copy.ttl | expression (`sql:copyOperation`) | ADMITS. Format: CSV\|PARQUET\|JSON; uses
  uses CSV. Expected: `COPY fibo_accounts TO
  '/tmp/fibo_accounts.csv' (FORMAT CSV)` |

Consuming drivers: `tests/wasm_duckdb_lifecycle_eh.rs` courts (no tc
fixture references on disk yet — wiring pending) and the pm4py driver
`pm4pytest/test_packet4_eh_fibo.py` (directory not yet present in this
checkout — pending). Until those land, tc05 execution stays
UNVERIFIED-by-eye; the table above is compile-layer status, not
execution status.
