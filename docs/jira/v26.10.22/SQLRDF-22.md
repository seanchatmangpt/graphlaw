# SQLRDF-22 — FIBO-matrix compiler gaps: DECIMAL, COPY egress policy, multi-row INSERT, batch vocabulary, ATTACH path validation, egress digests

- **Id:** SQLRDF-22
- **Title:** FIBO-matrix compiler gaps: DECIMAL, COPY egress policy, multi-row INSERT, batch vocabulary, ATTACH path validation, egress digests
- **Status:** Open
- **Base:** HEAD `dab6a73` (chore(snapshot): fleet wave safety snapshot —
  2026-10-22 annealing waves W/Z/AA/AB/K)
- **Design:** [designs/fibo-pm4py-court-matrix.md](designs/fibo-pm4py-court-matrix.md)

## Why this ticket exists

The FIBO/pm4pytest court-matrix fixture design audited the compiler surface
the matrix needs and found six gaps. None blocks admission; all are typed
vocabulary or policy holes that force fixtures to contort (quoted-string
decimals, one-compile-per-statement lifecycle courts, hardcoded egress
destinations). Each gap lands as either typed vocabulary in the compiler or
a documented typed refusal, each with a court.

## Conformance target

Section 5 of [`docs/reference/sql-rdf-rec.md`](../../reference/sql-rdf-rec.md)
(admission + typed refusal conventions of
[`docs/refusals.md`](../../refusals.md), codes append-only, never renumbered).

## Current behavior (verified on disk, 2026-10-23)

- **G1 — no DECIMAL column type.** The `column_type` allowlist is
  `INTEGER|BIGINT|VARCHAR|DOUBLE|BOOLEAN|DATE` and refuses anything else
  (`UnsupportedTerm("column type ...")`)
  (`src/sql_ast.rs:1723-1735`). Typed `xsd:decimal` literals are not
  unquoted either: `lit()` treats only `xsd:integer`/`xsd:double` (or an
  all-digit lexeme) as numeric, so a decimal like `1.23` renders as a
  quoted string relying on engine coercion (`src/sql_ast.rs:1420-1437`).
- **G3 — COPY TO egress hardcoded in the scalar arm.** The scalar compile
  arm renders `COPY {t} TO '/tmp/graphlaw-rdf-{t}.csv' (FORMAT CSV)` with
  the destination and format fixed in source
  (`src/sql_ast.rs:1948`; EXPORT/IMPORT neighbors `:1952`, `:1957`). The
  expression-level COPY form exists (`copyOperation`/`tableName`/
  `fileFormat`, `src/sql_ast.rs:1524`) but no COMPRESSION/option surface
  is expressible and there is no egress path policy.
- **G4 — single-column single-row INSERT only.** INSERT shapes are
  `sh:maxCount 1` on the value/row properties
  (`ggen/sql-dialect/statement_classes.ttl:145-160`).
- **G5 — one statement per document.** `statement_kind` returns a single
  candidate and the reader compiles one root statement per document
  (`src/sql_ast.rs:1380-1407`, reader call at `src/sql_ast.rs:1198`); a
  lifecycle court must make N `compile_turtle` calls; batch vocabulary is
  absent.
- **G6 — scalar ATTACH hardcodes `:memory:`, no path validation.** The
  scalar arm renders `ATTACH ':memory:' AS {name}`
  (`src/sql_ast.rs:2008`, also `:2011`, `:2015`); the expression-level
  form reads `attachTarget`/`databasePath` (`src/sql_ast.rs:1673-1678`)
  but no lease/path validation exists on `databasePath`.
- **Egress digests not receipt-carried.** `ExecutionReceipt` has no
  artifact-digest field (`src/duckdb_eval.rs:134-156`) — a COPY/EXPORT
  execution leaves no blake3 witness in the receipt.

## Required behavior

1. **G1: DECIMAL typed.** `column_type` admits `DECIMAL` (with optional
   precision/scale vocabulary), and `lit()` unquotes `xsd:decimal`
   literals; a court compiles and executes a DECIMAL-column fixture on a
   real frozen-duckdb connection.
2. **G3: egress policy.** COPY TO destination/format/COMPRESSION become
   typed graph properties with a default-deny path policy (paths outside
   the permitted root refuse with a stable refusal code); a court
   witnesses both the allow and the typed refusal.
3. **G4: multi-row INSERT vocabulary.** Shapes and compiler accept
   N-column N-row inserts; a court executes one and reads back real rows.
4. **G5: batch vocabulary.** A batch/statement-list root admits N
   statements per document with per-statement receipts; a lifecycle court
   compiles once and executes CREATE/INSERT/SELECT in sequence.
5. **G6: ATTACH path policy.** `databasePath` validates against the same
   path policy (or refuses with a stable code when a file path is
   out-of-policy); court covers allow + typed refusal.
6. **Egress digests receipt-carried.** `ExecutionReceipt` gains an
   artifact-digest field populated on egress statements; the receipt
   validates and the digest matches the real file blake3.

## Acceptance

- (a) DECIMAL column + `xsd:decimal` literal court green on a real
  frozen-duckdb connection (`cargo test --test <target>`).
- (b) COPY egress allow/refuse court green; refusal code registered in
  `docs/refusals.md` (append-only).
- (c) multi-row INSERT court green with real readback.
- (d) batch-compile lifecycle court green (one `compile_turtle` call,
  N statements).
- (e) ATTACH path allow/refuse court green.
- (f) egress digest receipt court green (receipt field == file blake3).

## Constraints

- Refusal codes append-only (`docs/refusals.md`) — never renumber.
- DuckDB is not a feature; no cfg-gating the SQL surface out of any target.
- Courts use real frozen-duckdb connections (Chicago); a loud SKIP is not
  a pass; a gate that cannot exit 1 is the failure.

## Already-implemented boundary

Admission itself, the expression-level COPY form (`copyOperation`/
`tableName`/`fileFormat`, `src/sql_ast.rs:1524`), the expression-level
ATTACH form (`attachTarget`/`databasePath`, `src/sql_ast.rs:1673-1678`),
the single-statement reader (`statement_kind`), and the `ExecutionReceipt`
struct are landed and receipted. This ticket does not re-ask for them; it
asks for the six typed gaps above plus their courts.

## Implementation progress (2026-10-23 wave)

Per-gap status. Anchors read on disk at `src/duckdb_op.rs` this session.

- **G1 DECIMAL — lane in flight.** `column_type` widening + `lit()` typed
  `xsd:decimal` unquoting; courts (a)(b)(c) in the lane's diff. Not yet on
  main; no anchors cited.
- **G3 egress digest — lane in flight.** `artifact_digest_hex`/
  `artifact_bytes` fields on `ExecutionReceipt` plus COPY-path hashing.
  Not yet on main. The receipt-field court (f) stays pending until this
  lane lands; do not file (f) green before its struct change exists.
- **G4 multi-column INSERT — lane in flight.** Multi assign-pair
  rendering with back-compat single-pair path; court (c) readback in the
  lane's diff. Not yet on main.
- **G6 ATTACH policy — LANDED.** `enforce_attach_policy` env wrapper at
  `src/duckdb_op.rs:217-220` (`GRAPHLAW_ATTACH_ROOTS`, unset/empty =
  fail-closed) over the pure seam `enforce_attach_policy_with` at
  `src/duckdb_op.rs:224`; wired into the execute path at
  `src/duckdb_op.rs:382`. `':memory:'` never canonicalizes, so it is
  refused (doc comment `src/duckdb_op.rs:203-216`). Courts:
  `attach_inside_root_executes_and_reads_back` (`:909`, real file DB +
  readback through the alias),
  `attach_outside_root_refused_connection_unpoisoned` (`:944`),
  `attach_env_unset_refused` (`:989`). Receipt (real run this session):
  `cargo test --lib attach` → `3 passed; 0 failed; 0 ignored; ... 62
  filtered out; finished in 0.10s`. Acceptance (e) is answered by this
  court set.
- **G5 batch vocabulary — design filed, not implemented.** See
  `docs/jira/v26.10.22/designs/statement-sequence-design.md`; successor
  ticket SQLRDF-23 filed. Acceptance (d) moves to SQLRDF-23.
