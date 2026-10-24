# Refusal Catalog

What a caller sees when GraphLaw refuses work, and what to do about it. This is a
user-facing view; the normative register is [`docs/refusals.md`](../refusals.md).
Codes are stable and append-only — they are never renumbered or reused.

## Envelope shape

Over the JSON ABI (feature `abi`), every refusal is one envelope:

```json
{
  "ok": false,
  "error": {
    "kind": "DuckDb",
    "message": "unknown or expired session `metrics`"
  },
  "details": {
    "code": "REFUSAL-03",
    "shape_id": 5
  }
}
```

- `error.kind` is the coarse refusal class (`RefusalKind`,
  `src/dialect/refusal.rs`).
- `details` is optional. Engine-raised failures (bad syntax, unknown query
  form) carry no `details`; the typed codes below always do. Decoders must
  treat `details` as optional and unknown codes as open — see the
  [capability registry](capability-registry.md).
- Full envelope/ABI field reference: [abi-reference.md](../abi-reference.md).

## SQL-AST codes (`REFUSAL-01`..`REFUSAL-05`)

These fire during SQL-in-RDF compile/validation, before the query is
dispatched to frozen-duckdb. Each message embeds a `[REFUSAL-nn]` marker; the
trigger-to-code mapping is fixed.

### REFUSAL-01 — MalformedAstRefusal

The graph AST failed strict closed-world SHACL shape validation before
admission. Covers: missing or non-integer `sql:statementIndex`, empty or
missing `sql:from`, missing or empty projection list, malformed or unclosed
RDF list, unexpected terminal node, recursion-limit breach, more than one
expression discriminator per node, an operator token outside the section 4
enum, a non-`sql:Scope` individual, a `sql:joinType` IRI that is not
`sql:InnerJoin`/`sql:LeftJoin`, and integer/double literals whose lexical
forms do not parse (never silently coerced to 0).

Carried by `sql_ast::AstError` (`MalformedAst`, plus the non-shape malformed
cases `MissingProperty`, `MalformedNode`, `MalformedList`,
`RecursionLimitExceeded`, `InvalidIdentifier`).

```json
{
  "details": {
    "code": "REFUSAL-01",
    "message": "SHACL AST structural violation: [REFUSAL-01] \
MalformedAstRefusal: statement <q1> is missing required sql:statementIndex"
  }
}
```

Recovery: fix the AST to satisfy the shapes in `ontologies/sql-ast.ttl`, then
re-admit.

### REFUSAL-02 — UnimplementedStatementRefusal

A dialect statement kind was recognized by the grammar but has no evaluation
path in this DuckDB build. Currently: `ConnectStatement`,
`DisconnectStatement`.

```json
{
  "details": {
    "code": "REFUSAL-02",
    "message": "compile refusal: http://graphlaw.org/sql#ConnectStatement \
[REFUSAL-02] UnimplementedStatementRefusal: not implemented by this DuckDB build"
  }
}
```

Recovery: rewrite the query using admitted statement kinds.

### REFUSAL-03 — ScopeIntegrityRefusal

Scope discrimination boundary violated (rec §1.3). Covers: an aggregate in a
`WHERE`/`JOIN`/`GROUP BY` position; a `sql:ProcessScope` column in the
projection list; a `sql:ViewScope` column inside a predicate (including
`HAVING`); a scope annotation on an aggregate argument beneath a
predicate-rooted aggregate; a MERGE `ON` condition referencing no
target-qualified column.

```json
{
  "details": {
    "code": "REFUSAL-03",
    "message": "SHACL AST structural violation: [REFUSAL-03] \
ScopeIntegrityRefusal: column 'pwd' is sql:ProcessScope but sits in the \
projection list"
  }
}
```

Recovery: move aggregates into `SELECT`/`HAVING`, keep `ProcessScope` columns
out of projections and `ViewScope` columns out of predicates, drop scope
annotations on predicate-side aggregate arguments, and qualify MERGE `ON` to
at least one target column (e.g. `t.id = s.id`).

### REFUSAL-04 — CapabilityRefusal

Builtin SQL eval was handed a reader that is not document-backed. The wasm
module admits and compiles SQL-in-RDF through the same `compile_turtle` path
as every other dialect.

```json
{
  "details": {
    "code": "REFUSAL-04",
    "message": "REFUSAL-04 CapabilityRefusal: document-backed reader required"
  }
}
```

Recovery: supply the admitted Turtle document and compile with
`compile_turtle` (`query_document_turtle`).

### REFUSAL-05 — UnboundedMutationRefusal

An UPDATE or DELETE statement node carries neither `sql:whereColumn` nor
`sql:whereValue` — the rendered SQL would mutate every row of the target
table. Refused at compile, zero engine contact. (Appended 2026-10-23;
append-only, never renumbered.)

```json
{
  "details": {
    "code": "REFUSAL-05",
    "message": "compile refusal: statement <u1> [REFUSAL-05] \
UnboundedMutationRefusal: unbounded mutation refused: UPDATE/DELETE requires \
sql:whereColumn or sql:whereValue"
  }
}
```

Recovery: add a WHERE clause bounding the mutation.

## Document-sequence codes (`SQLRDF-23`, appended)

Typed arms of the `RefusalCode` namespace for `document:true` execution.
All are `MalformedAst` family refusals: `sql_ast::AstError` with
`shape_id` naming the arm. Each is a compile-time refusal — no engine
contact.

### `MIXED_DOCUMENT_ROOTS`

The document carries more than one bare statement root, or both a bare root
and a sequence root. One root or one sequence, never both — the previous
silent lexicographic pick is closed. Recovery: wrap multiple statements in
one `rdf:List` sequence, or emit a single bare root.

### `INDEX_SEQUENCE_DISORDER`

The sequence member `sql:statementIndex` values are not strictly ascending
along the `rdf:List` spine (non-integer, duplicate, gapped, or out of
order). Spine order is the ordering source; the explicit index must agree.
Recovery: renumber `sql:statementIndex` so members ascend strictly from 0.

### `SEQUENCE_EMPTY`

A sequence whose `sql:statements` list has zero members (defense in depth
below the shape refusal). Recovery: give the sequence at least one statement
member.

Example message:

```text
SHACL AST structural violation: [REFUSAL-01] MalformedAstRefusal: \
sql:StatementSequence <seq1> has an empty sql:statements list; a document \
sequence carries at least one member
```

## Set-operation codes (`SQLRDF-27`, appended)

Typed refusal arms for `sql:SetOperationStatement` documents (`UNION` /
`INTERSECT` / `EXCEPT`), appended with the v26.10.23 set-operation surface.
All are MalformedAst (`REFUSAL-01`) family refusals carried by
`sql_ast::AstError` with `shape_id` naming the arm, the `[REFUSAL-01]`
marker in the message — refused at compile, zero engine contact. The
normative rows (with numeric arm discriminants) live in
[`docs/refusals.md`](../refusals.md).

| code | what it means | recovery |
|---|---|---|
| `SET_OP_NO_RIGHT` | The set-operation statement carries no `sql:setRight` — the right operand is missing, so the rendered SQL would be `( SELECT … ) UNION ` with an empty right operand. A set operation is binary by construction. | Attach a nested `sql:SelectStatement` via `sql:setRight`. |
| `SET_OP_KIND_TOKEN` | The `sql:setOperator` token is not `UNION`, `INTERSECT` or `EXCEPT` (case-insensitive). | Use one of the three admitted operator tokens. |
| `SET_OP_FLAG_LEX` | A `sql:setAll` / `sql:byName` flag literal is not an `xsd:boolean` `true`/`false` lexical. | Write `sql:setAll` / `sql:byName` with an `xsd:boolean` `true`/`false` literal. |
| `SET_OP_BY_NAME_KIND` | `sql:byName true` beside a non-UNION `sql:setOperator` — only `UNION` admits `BY NAME` (the engine rejects `INTERSECT BY NAME` / `EXCEPT BY NAME` as parser errors). | Drop the `sql:byName` flag, or use `UNION`. |

Example message:

```text
SHACL AST structural violation: [REFUSAL-01] MalformedAstRefusal: \
sql:SetOperationStatement <s1> carries no sql:setRight; a set operation \
is binary by construction
```

## Runtime surfaces

Refusals raised while executing an admitted query against the DuckDB
substrate (ABI op `duckdb`, `src/duckdb_op.rs`). All are typed; the
substrate never traps.

### `DuckDb` (`RefusalKind::DuckDb`)

The substrate failed or the request misused it. `details.message` names the
specific condition:

- **Unknown/expired session** — `unknown or expired session \`<name>\``.
  Use a live session name; sessions are process-local and crash-only.
- **Duplicate session name** — `session \`<name>\` already exists
  (close it before reopening)`. Close the session first, or open under a
  fresh name.
- **Attach policy violation** — `attach path outside
  GRAPHLAW_ATTACH_ROOTS allowlist: <path>`. Attach only under the configured
  allowlist roots (colon-separated absolute paths); file-backed attaches
  outside the allowlist are refused fail-closed.
- **Unknown/expired cursor** — `unknown or expired cursor \`<cursor>\``.
  Re-run the query; do not reuse dropped cursors.
- **Bad view mount / unsupported param / engine error** — free-form engine
  text in `details.message`. Fix the request per the message.
- **Batch member failure** — `BatchMemberFailed` (below). Fix the failing
  member; replay resumes from it.

Absence of an op from the hashed op list is a different surface (`Unsupported`
envelope), not this refusal.

### `BatchMemberFailed` (document execution)

`mode: "exec"` with `document: true` runs a sequence of statements and answers
the `BatchReceipt` response shape (per-member `rows_emitted`). A failing
member mid-sequence returns the typed envelope:

```json
{
  "ok": false,
  "error": { "kind": "DuckDb" },
  "details": {
    "code": "BatchMemberFailed",
    "member": 2,
    "partial_receipt": { "...": "receipt for the prefix that did run" }
  }
}
```

`member` names the failing statement index; `partial_receipt` covers the
prefix that executed. Fix that member's statement and replay resumes from the
named member.

### `ResourceLimit` (`RefusalKind::ResourceLimit`)

Bounded process-local resources. `details` carries `limit`, `observed`,
`max`. On the DuckDB surface the limits are:

| `limit` value | bound | what to do |
|---|---|---|
| `duckdb_sessions` | 16 live sessions (`MAX_SESSIONS`) | Close a session to free a slot. |
| `duckdb_chunk_rows` | 10,000 rows per chunk | Reduce the chunk `n` or paginate. |
| `duckdb_chunk_bytes` | 16 MiB per chunk | Reduce chunk size. |
| `duckdb_views` | 64 mounted views per request | Mount fewer views. |

Other limits on other surfaces (`request_bytes`, `json_depth`, N3 reasoner
limits, and so on) are listed in the register.

### `Unsupported` envelope

Valid-but-unusable requests: a malformed ABI request, blank nodes in plan
atoms, an unsigned `lease` without explicit opt-in, or an op outside the
hashed op list. `error.kind` is `Unsupported`; there is no `details.code` —
the `message` names the condition. Fix the request shape; the capability
registry is the open set of known codes.

## Guarantees

- **Refuse before contact.** SQL-AST refusals (`REFUSAL-01`..`05`) and the
  document-sequence codes fire at compile/validation, before any
  frozen-duckdb connection is opened. Attach-policy and chunk-limit checks
  run before the compiled SQL reaches the engine.
- **Typed, never trapped.** Every runtime failure on the DuckDB surface is a
  typed refusal envelope, not a panic or generic DB error. Refusal codes are
  append-only: existing codes never change meaning or number.
- **No silent degradation.** Unknown join-type IRIs never downgrade to Inner;
  malformed integer/double literals are never coerced to 0; extra document
  roots are never silently picked lexicographically.
- **Envelope discipline.** `details` is optional and open-ended; decoders
  treat unknown codes as open per the capability registry.

## Where the register lives

[`docs/refusals.md`](../refusals.md) is the normative register: every code,
its carrier, and its recovery, plus the harness refusal surfaces
(`MUTATION-COURT-DIRTY-TREE`, `GRAPHLAW-RUN-EH`, and so on) and the typed
refusal codes for attest/eval surfaces (`NO-CANONICAL-GRAPH`,
`ATTEST-REFUSED`). Symptom-to-fix mapping: the
[refusal playbook](../refusal-playbook.md). Envelope fields:
[abi-reference.md](../abi-reference.md).

## See Also

- [Refusals register](../refusals.md)
- [Refusal playbook](../refusal-playbook.md)
- [ABI reference](../abi-reference.md)
- [Capability registry](../capability-registry.md)
