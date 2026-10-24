# Document Sequences: Multi-Statement Exec

Run several admitted SQL statements in one wire call against a real
frozen-duckdb connection, with a typed receipt per member.

Design record:
`docs/jira/v26.10.22/designs/statement-sequence-design.md` (SQLRDF-23,
sections 2, 4, 5, 6). Op half: `src/duckdb_op.rs:379-385` and
`exec_document_on` (`src/duckdb_op.rs:463-532`). Eval half:
`BatchReceipt` (`src/duckdb_eval.rs:397-422`) and
`eval_ast_document` (`src/duckdb_eval.rs:1052`).

## When to use

Use `document: true` when one logical job needs several statements that
share a connection and must be receipted as a unit:

- CREATE + INSERT + SELECT ladders (the worked example below).
- Any multi-step load-then-read where the intermediate state only
  exists between members on the same connection.

Use N separate single-statement execs when you need per-call wire
`params`, per-call cursors, or atomic mode with a single statement.
The document form:

- refuses wire `params` (each member's literals live in the Turtle AST
  document itself — `src/duckdb_op.rs:479-487`);
- has no cursor (results ride each member's `result_triples`);
- defaults to best-effort autocommit (pipeline mode, below); pass
  `"atomic": true` for BEGIN/COMMIT/ROLLBACK semantics.

## Atomic vs pipeline (SQLRDF-26)

Two execution modes share one wire surface, selected per request:

- **Pipeline** (default, `atomic` absent or false): the SQLRDF-23
  best-effort sequential autocommit path, byte-identical when the flag
  is absent. Committed members survive a later member's failure — see
  Partial-commit semantics.
- **Atomic**: `"atomic": true` on the wire, or `sql:atomic true` on the
  Turtle root; the two OR together (`src/duckdb_op.rs:493-508`). The
  whole sequence runs inside BEGIN/COMMIT/ROLLBACK in
  `eval_ast_document`: on a member failure the partial receipt is
  marked `status: "aborted"` with `state_delta: 0` (rollback), and on
  success the envelope carries `status` and `state_delta`
  (`state_delta` = committed member count).

Refusal surfaces: unbounded UPDATE/DELETE members refuse `REFUSAL-05`
(`UnboundedMutationRefusal`, `docs/refusals.md`) at compile, before
any engine call, in both modes.

Single-root documents without the flag are untouched (court at
`src/duckdb_op.rs:1236`,
`legacy_exec_without_document_flag_unchanged`).

## Turtle sequence vocabulary

The document root is a `sql:StatementSequence` whose `sql:statements`
predicate points at an `rdf:List` of member statement nodes:

```turtle
sql:doc a sql:StatementSequence ;
    sql:statements ( ex:c ex:i ex:s ) .
```

Every member carries `sql:statementIndex`, a 0-based integer literal.
Walk order is the `unroll_rdf_list` spine order; the index is a strict
cross-check, not the ordering source — it must ascend with no gaps
(`idx[i] == i`), otherwise the compile refuses
`INDEX_SEQUENCE_DISORDER` (MalformedAst family, append-only per
`docs/refusals.md`). Compile-side refusals
(`src/sql_ast.rs:1335-1445`):

- `SEQUENCE_EMPTY` — the member list has no members.
- `MIXED_DOCUMENT_ROOTS` — a sequence root and a bare `sql:*Statement`
  root in one document (closes the silent lexicographic-pick hazard).
- `INDEX_SEQUENCE_DISORDER` — non-integer, duplicate, or non-ascending
  `sql:statementIndex`.

Each member compiles by the same law as a single statement
(`compile_turtle_document`, `src/sql_ast.rs:1343`), so CONNECT members
refuse `REFUSAL-02` and unbounded mutations refuse `REFUSAL-05`, with
position attribution added by the batch layer (design section 5).

## Wire shape

```json
{
  "op": "duckdb",
  "mode": "exec",
  "document": true,
  "query_ast": "urn:doc-howto",
  "ast": { "turtle": "<Turtle document>" },
  "subject_prefix": "urn:row/",
  "vocab_prefix": "urn:p/"
}
```

`document: true` routes the whole Turtle document through
`compile_turtle_document` + `eval_ast_document`
(`src/duckdb_op.rs:383-385`). Each member passes the ATTACH allowlist
before the engine sees it (`src/duckdb_op.rs:490-492`). Success
answers the BatchReceipt shape; a member failure answers the typed
`BatchMemberFailed` envelope.

## BatchReceipt fields

On success (`ok: true`):

| Field | Meaning |
|---|---|
| `query_iri` | caller-supplied envelope IRI |
| `doc_hash` | hex BLAKE3 over the member hashes with `0x00` separators — domain-separated from the bare single-statement hash; a 1-member sequence hashes differently from the bare form (design section 4) |
| `members` | one `ExecutionReceipt` per executed member, spine order |
| `rows_emitted` | per-member row counts, same indexing as `members` |
| `failed_at` | `null` on full success; else 0-based first-failure index |

Each member is a full single-statement receipt: `query_iri` is the
envelope IRI plus `#member-N`, with `query_hash`, `sql_statement`,
`rows_emitted`, `result_triples`, `cached`.

## Partial-commit semantics

Pipeline mode only; atomic mode rolls back instead (see above).

Best-effort sequential autocommit on one real connection
(`src/duckdb_eval.rs:402-406`). Members execute in spine order; each
commits on success. `failed_at = Some(i)` means:

- `members[0..i]` executed and committed — their effects stay
  visible. There is no rollback; this is documented partial-commit,
  not a hidden one.
- `members[i..]` did not execute.
- The failing member's own detail is the typed
  `EvaluatorError::BatchMemberFailed { index, source, partial }`
  (`src/duckdb_eval.rs:209-213`).

The op layer maps that to the wire envelope
`error.details = {code: "BatchMemberFailed", member: i, message,
partial_receipt}` (`src/duckdb_op.rs:505-529`) — never a raw DB error
surfaced untyped, never a silent swallow. `partial_receipt` carries
the receipts and row counts of exactly the members that executed and
committed.

## Cache law

The batch envelope never consults the statement cache. Per member
(design section 6):

- All members cacheable (select-arm) -> each stores individually under
  its own hash. No batch-keyed cache entry exists.
- Any member non-cacheable or mutating -> no member stores.
- A mutating member bumps the epoch once at its own execution point
  (existing stmt-cache flush mechanism), so later members recompile.

## Worked example (real run)

Document (3 members):

```turtle
@prefix sql: <http://graphlaw.org/sql#> .
@prefix xsd: <http://www.w3.org/2001/XMLSchema#> .
@prefix ex: <http://example.org/doc#> .
sql:doc a sql:StatementSequence ;
    sql:statements ( ex:c ex:i ex:s ) .
ex:c a sql:CreateStatement ;
    sql:statementIndex 0 ;
    sql:targetTable "doc_events" ;
    sql:assignColumn "batch_id" .
ex:i a sql:InsertStatement ;
    sql:statementIndex 1 ;
    sql:targetTable "doc_events" ;
    sql:assignColumn "batch_id" ;
    sql:assignValue "42"^^xsd:integer .
ex:s a sql:SelectStatement ;
    sql:statementIndex 2 ;
    sql:fromTable [ sql:tableName "doc_events" ] ;
    sql:selectList ( [ sql:columnName "batch_id" ] ) .
```

Request sent as one packet-4 frame to
`target-lane-doc/debug/graphlaw-run-eh --wasm
target-auto/wasm32-wasip1/wasm/graphlaw_wasm.wasm` (2026-10-23).
Real success reply (trimmed to the load-bearing fields):

```json
{
 "doc_hash": "0c36f3151666bbe0495893b14569836e8d001718cdc30edb1cadc34677a2cf36",  # historical example-run digest
 "failed_at": null,
 "ok": true,
 "query_iri": "urn:doc-howto",
 "rows_emitted": [0, 1, 1],
 "members": [
  {"query_iri": "urn:doc-howto#member-0",
   "sql_statement": "CREATE TABLE doc_events (batch_id INTEGER)",
   "rows_emitted": 0, "cached": false},
  {"query_iri": "urn:doc-howto#member-1",
   "sql_statement": "INSERT INTO doc_events (batch_id) VALUES (42)",
   "rows_emitted": 1, "cached": false},
  {"query_iri": "urn:doc-howto#member-2",
   "sql_statement": "SELECT batch_id FROM doc_events",
   "rows_emitted": 1, "cached": false}
 ]
}
```

Member 2's `result_triples` carried the inserted row back out:
`<urn:row//row_1> <urn:p//batch_id> "42"^^xsd:integer` — the SELECT
read what the INSERT committed on the same connection.

### Failing member

Same document with member 2 reading `no_such_table`. Real reply
(same rung, 2026-10-23):

```
error.kind: DuckDb
details.code: BatchMemberFailed
details.member: 2
details.message: member 2 failed: DuckDB execution failure:
  Catalog Error: Table with name no_such_table does not exist!
partial_receipt.failed_at: 2
partial_receipt.rows_emitted: [0, 1]
partial_receipt.members: 2 entries
partial_receipt.members[0].sql_statement:
  CREATE TABLE doc_events (batch_id INTEGER)
```

Reading it: members 0 and 1 executed and committed (`CREATE TABLE
doc_events` exists; the INSERT's count row is receipted), the SELECT
failed, and the envelope names index 2 with the partial receipt. The
caller sees exactly which state survived. Replays against the same
document are deterministic on a fresh per-call connection.

## See Also

- `docs/jira/v26.10.22/designs/statement-sequence-design.md`
- `docs/how-to/run-the-wasm-module.md`
- `docs/refusals.md`
