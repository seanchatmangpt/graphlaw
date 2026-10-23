# SQLRDF-26 — Atomic document execution mode

Status: ALIVE (this implementation; verification receipts inline)
Lane: sql-rdf compiler surface (`src/sql_ast.rs` module, `src/duckdb_eval.rs`, `src/duckdb_op.rs`)

## 1. Atomic mode (this implementation)

`sql:atomic true` on the `sql:StatementSequence` root (or wire-level
`"atomic": true` on the document exec op) selects atomic execution:

- `BEGIN TRANSACTION` on the target connection before member 0.
- Members run in spine order via the unchanged per-member path
  (`eval_member`, seal verify → receipt hash → prepare/bind/execute).
- Full spine success → `COMMIT`.
- Any member failure → `ROLLBACK`, then the typed
  `BatchMemberFailed` envelope with `failed_at` and a partial receipt marked
  `status:"aborted"`, `state_delta:0`.
- Default (`sql:atomic` absent, wire `atomic` absent/false) is the SQLRDF-23
  best-effort sequential-autocommit (pipeline) path — byte-identical when
  atomic is absent.

New REFUSAL-01 arm (append-only): `ATOMIC_LITERAL` (46) — a `sql:atomic`
literal that is not an `xsd:boolean` `true`/`false` refuses typed at compile
(never a silent default).

Wire shape gains `status` and `state_delta` on the success envelope and on
the `partial_receipt` inside `BatchMemberFailed`; both `#[serde(default)]`
so SQLRDF-23 wire JSON deserializes unchanged.

## 2. Naming delta (documentation alias, not a code change)

The combinatorial-sequencing specification names the receipt fields
`steps_total` / `steps_executed` / `step_receipts` / `status`; the landed
SQLRDF-23/26 wire shape is `members` / `rows_emitted` / `failed_at` /
`status`. Mapping (alias only — no rename, no wire break):

| spec name | landed field |
|---|---|
| `steps_total` | `members.len()` (post-batch; the receipt carries executed members) |
| `steps_executed` | `members.len()` on success, `failed_at` on failure |
| `step_receipts` | `members` |
| `status` | `status` (identical) |

Recorded as a documentation alias; renaming would break the landed
SQLRDF-23 wire courts for zero behavioral gain.

## 3. doc_hash design note

The spec describes a chained accumulator
`H_i = BLAKE3(H_{i-1} || h_i)`; the landed form is the flat domain-separated
fold `BLAKE3(0x00 || h_1 || 0x00 || h_2 || ...)` (pinned in
`compile_turtle_document`, `src/sql_ast.rs`). Both are order-sensitive: any
permutation of the spine changes the hash, so the landed receipt-invariance
court (order-sensitivity twin, `src/duckdb_eval.rs` document courts) already
pins the property the spec asks for. The chained variant is ticketed here as
an alternative formulation, not a defect — switching it would fork the
landed `doc_hash` identity for the members it already commits.
