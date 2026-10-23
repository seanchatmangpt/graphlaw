# SQLRDF-23 Design: Statement Sequences (G5)

Status: verified design (anchors read from the tree 2026-10-23)
Ticket: SQLRDF-23 (this window)
Owner lane: docs lane (this file + INDEX row only)

## 1. Problem

The compiler admits exactly one root `sql:*Statement` per document.
`GraphLawAst::statement_kind` (`src/sql_ast.rs:1376-1409`) collects every
`rdf:type sql:*Statement` subject that is nobody's object, sorts, and takes
`.next()` — the lexicographically smallest. A document with two top-level
statements therefore silently compiles the smaller-named one and drops the
other. That is the silent multi-root hazard this design closes.

Eval has no batch surface either: `eval_ast_query`
(`src/duckdb_eval.rs:787`) executes one `CompiledQuery` and returns one
`ExecutionReceipt`.

## 2. Vocabulary

- `sql:StatementSequence` — class of the document root container.
- `sql:statements` — predicate from the document root to the head of an
  `rdf:List` of `sql:*Statement` nodes.
- `sql:statementIndex` — per-member integer literal, 0-based, required on
  every member. The walk order is the `unroll_rdf_list` order
  (`src/sql_ast.rs:89-126`); the explicit index is a cross-check, not the
  ordering source.

## 3. Compile path

`compile_turtle_document(turtle) -> CompiledDocument { statements:
Vec<CompiledQuery>, doc_hash: [u8; 32] }` (new function; new struct).

- `compile_turtle` (`src/sql_ast.rs:1190`) stays untouched: single-root
  documents compile bit-for-bit identically. The single-root path is the
  back-compat guarantee, pinned by a golden hash court (§7a).
- Sequence detection: a document root is a `sql:StatementSequence` whose
  `sql:statements` points at an rdf:List head. `unroll_rdf_list` walks the
  spine; the strictly-ascending `sql:statementIndex` check runs on the
  result (`idx[i] == i` after a 0-based strictness pass — refuse unless
  ascending with no gaps, since members may be heterogeneous kinds).
- New REFUSAL-01 arm `INDEX_SEQUENCE_DISORDER` (MalformedAst family,
  append-only per `docs/refusals.md` conventions, never renumbered): fired
  when the cross-check fails — non-integer, duplicate, or non-ascending
  `sql:statementIndex`. Recovery: renumber the members.

## 4. Document hash

`doc_hash` is domain-separated and used only for the sequence form:

```
doc_hash = BLAKE3(0x00 || h1 || 0x00 || h2 || ... )
```

where `h_i` is the existing per-query hash already carried by
`CompiledQuery` (the `purrdf::try_canonicalize`-anchored hash pinned by
`compile_turtle`). The 0x00 separator bytes are the domain separation from
any bare-form hash. Consequence, intended: a sequence of exactly one member
hashes differently from the bare single-root form of the same statement —
the envelope is part of the identity. This is what makes a receipt for the
document form non-forgeable as the single form and vice versa.

## 5. Eval path

`eval_ast_document(graphlaw, turtle, params) -> BatchReceipt`:

```
BatchReceipt {
  query_iri:   String,
  doc_hash:    [u8; 32],
  members:     Vec<ExecutionReceipt>,  // one per member, compile order
  rows_emitted: Vec<usize>,            // per-member row counts
  failed_at:   Option<usize>,          // 0-based index of first failure
}
```

Semantics: best-effort sequential autocommit. Members execute in spine
order on one real connection; each member commits on success and leaves
its effects committed on failure of a later member. `failed_at` names the
first failing member; prior members' receipts and row counts stay in
`BatchReceipt` (partial-commit semantics documented here, not hidden).
`failed_at = Some(i)` implies members[0..i] succeeded and committed;
members[i..] did not execute. members[i]'s failure detail is the last
element of `members` (execution stops at the failure).

Refusal attribution: `REFUSAL-02` (UnimplementedStatementRefusal,
`docs/refusals.md`) and `REFUSAL-05` (UnboundedMutationRefusal) fire per
member at compile time inside `compile_turtle_document`; the
`BatchReceipt` layer wraps them with position attribution (which member
index) instead of changing the refusal payloads.

## 6. Cache law

The batch never consults the statement cache for the sequence envelope.
Per-member:

- If ALL members are cacheable (select arm), their entries store as
  individual single-statement cache entries under each member's own hash
  — no batch-keyed cache entry exists.
- If ANY member is non-cacheable or mutating, no member stores.
- A mutating member bumps the epoch once per member at its own execution
  point (existing epoch mechanism, `src/duckdb_eval.rs` stmt-cache flush
  path).

Rationale: no batch-level cache key means no batch invalidation problem;
the only new invalidation obligation is the per-member epoch bump on
mutating members, which reuses the existing mechanism.

## 7. SHACL

One `sql:statements` head per `sql:StatementSequence`:
`sh:minCount 1`, `sh:maxCount 1` (ontologies/sql-ast.ttl shape addition;
generated projections regenerated, never patched).

In Rust, beyond the shape: member-count floor and spine integrity walk —
an empty list (head = `rdf:nil` immediately) refuses
`SEQUENCE_EMPTY`; a broken spine (missing `rdf:first`/`rdf:rest` —
malformed list) surfaces through `unroll_rdf_list`'s existing
`MalformedList`/`MissingProperty` errors. New MalformedAst arms:
`SEQUENCE_EMPTY`, `MIXED_DOCUMENT_ROOTS` — a document containing both a
`sql:StatementSequence` root and a bare `sql:*Statement` root refuses
rather than the current silent lexicographic pick.

## 8. Court plan

- (a) Golden single-root hash vector: an existing single-root document
  hashes identically before/after this change (back-compat, bit-for-bit).
- (b) Sequence-of-1 hash difference: the same statement wrapped in a
  1-member sequence produces a different `doc_hash` than the bare form
  (domain separation is load-bearing, not cosmetic).
- (c) Connect-in-sequence refusal: a sequence containing a
  `sql:ConnectStatement` refuses with `REFUSAL-02` plus position
  attribution.
- (d) REFUSAL-05 position attribution: a sequence whose UPDATE/DELETE
  member lacks a WHERE bound refuses `REFUSAL-05` naming the member index.
- (e) Epoch bump after mutating member: a sequence [SELECT, UPDATE,
  SELECT] executes the second SELECT uncached (epoch bumped by the
  UPDATE), witnessed on a real frozen-duckdb connection.
- (f) Mid-sequence failure names index: a member that fails at execution
  yields `failed_at = Some(i)`, members[0..i] receipts present, and the
  error names index i.

Anti-vacuity: (b) and (f) must flip under operator inversion (same hash,
dropped `failed_at`) per the mutation-court discipline.

## 9. Risks

- Silent multi-root hazard is the motivating defect
  (`src/sql_ast.rs:1376-1409`); `MIXED_DOCUMENT_ROOTS` must land with the
  sequence path, not after.
- Per-member stmt-cache bookkeeping: "all cacheable or none store" keeps
  the invariant checkable; a partial-store rule would need per-entry
  invalidation proofs.
- `purrdf` list-order dependence: spine order is graph-serialization
  order in `unroll_rdf_list`; the index cross-check is the guard against
  any reordering, which is why it is strictly-ascending, not
  advisory.
- `failed_at` partial-commit semantics: documented best-effort autocommit;
  a caller needing transactionality must not use the batch form (typed
  boundary, not a hidden rollback).
- Unroll cap bound: `unroll_rdf_list` enforces `max_items`
  (`RecursionLimitExceeded`, `src/sql_ast.rs:103-105`); the sequence path
  passes the same bound, so a pathological document refuses typed, never
  hangs.

## 10. Non-goals

- No transactional batch semantics (see §5).
- No batch-keyed cache entries.
- No change to `compile_turtle`, `eval_ast_query`, or the
  `ExecutionReceipt` struct.
- No renumbering of refusal codes.
