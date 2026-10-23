# SQLRDF-23 — Statement sequences (G5)

- **Id**: SQLRDF-23
- **Title**: Statement sequences — ordered multi-statement documents with
  receipted batch execution
- **Status**: Open
- **Conformance target**: design doc is authoritative —
  [`designs/statement-sequence-design.md`](designs/statement-sequence-design.md)

## Acceptance

Real court targets (court plan (a)-(f) in the design doc):

- `cargo test --test sql_rdf_dialect` — extended with:
  - (a) golden single-root hash vector unchanged (back-compat)
  - (b) sequence-of-1 `doc_hash` differs from bare form
  - (c) `ConnectStatement` member → `REFUSAL-02` + position attribution
  - (d) WHERE-less mutating member → `REFUSAL-05` + member index
  - (e) mutating member bumps epoch; later SELECT uncached (real
    frozen-duckdb connection)
  - (f) mid-sequence failure sets `failed_at = Some(i)`, names index i
- `bash scripts/mutation_court.sh` — (b) and (f) inversions flip.

## Implementation progress (compile half LANDED 2026-10-23)

Compile half landed in `src/sql_ast.rs`; eval half in flight.

- `compile_turtle_document` (`src/sql_ast.rs:1343`) parses the document,
  refuses more than one top-level root (or bare-root-plus-sequence) with
  `MIXED_DOCUMENT_ROOTS` (`src/sql_ast.rs:1445`) — the silent
  lexicographic multi-root hazard is closed; sequence members are
  spine-ordered via `unroll_rdf_list` capped at 1024
  (`src/sql_ast.rs:1385`), cross-checked against `sql:statementIndex`
  (`INDEX_SEQUENCE_DISORDER`, `src/sql_ast.rs:1402`, `:1411`), empty list
  refuses `SEQUENCE_EMPTY` (`src/sql_ast.rs:1384`, `:1389`).
- Receipt fold is domain-separated in `CompiledDocument`
  (`src/sql_ast.rs:1376`, `:1434`): the bare identity form keeps
  `receipt_state = h_doc` shared and is bit-identical to the pre-feature
  single-statement path; sequence-of-1 differs from bare by design.
  Sequence members compile conservative `cacheable = false`
  (`src/sql_ast.rs:1360`); failure attribution stamps the 0-based member
  position into `ctx.predicate_idx` (`src/sql_ast.rs:1422`).
- Refusal arms are append-only: `MIXED_DOCUMENT_ROOTS = 39`,
  `INDEX_SEQUENCE_DISORDER = 40`, `SEQUENCE_EMPTY = 41`.
- SHACL vocabulary landed `ontologies/sql-ast.ttl:552-570`
  (`sql:StatementSequence`, `sql:statements`, `sql:StatementSequenceShape`
  with `sql:OrderedListShape` refusing the empty list before the
  `SEQUENCE_EMPTY` arm — both refusals typed).
- Courts: 7 new `compile_turtle_document` courts
  (`src/sql_ast.rs:3131`+ Chicago tests over real Turtle graphs).
  `cargo test --test sql_rdf_dialect` → `36 passed; 0 failed` (witnessed
  2026-10-23, exit 0). sql_ast unit-suite count not re-witnessed this
  session: the lib-test cfg is currently blocked by an in-flight lane edit
  (`src/duckdb_op.rs:1122`, E0425 `Value` not found) outside this lane's
  lease — pre-existing, not session-introduced.

## Implementation progress (eval + wire LANDED 2026-10-23)

Eval half and wire half landed; the whole feature is on disk.

- Wire: `{"op":"duckdb","mode":"exec","document":true,
  "query_ast":...,"ast":{"turtle":<sequence doc>},...}` routes at
  `src/duckdb_op.rs:372-374` to `exec_document_on`
  (`src/duckdb_op.rs:463-530`). Non-empty wire `params` on the document
  path are refused typed per the SQLRDF-17 one-for-one law
  (`src/duckdb_op.rs:474-477`); G6 ATTACH policy enforced per member
  (`src/duckdb_op.rs:479-481`); works on named sessions and fresh
  connections (same routing as the single-statement path).
- Success answers BatchReceipt `{query_iri, doc_hash(64-hex), members,
  rows_emitted, failed_at:null}` (`src/duckdb_op.rs:492-503`).
- Eval: `eval_ast_document` (`src/duckdb_eval.rs:1052`) folds members
  sequentially (best-effort autocommit); a mid-sequence failure returns
  `EvaluatorError::BatchMemberFailed {index, source, partial}`
  (`src/duckdb_eval.rs:209-212`) carrying the partial `BatchReceipt`
  (`src/duckdb_eval.rs:408-421`, members `[0..failed_at]` committed).
  The op converts this to the typed `BatchMemberFailed` details envelope
  (`code`/`member`/`message`/`partial_receipt`), never a generic DB
  error (`src/duckdb_op.rs:507-524`).
- Courts witnessed 2026-10-23 (earlier lanes): duckdb_op lib 16 passed /
  0 failed (3 document courts: three-member CREATE→INSERT→SELECT with
  real readback `src/duckdb_op.rs:1139`; typed failing-member envelope
  `src/duckdb_op.rs:1187`; legacy no-flag path unchanged
  `src/duckdb_op.rs:1225`), duckdb_eval lib 18/0, sql_rdf_dialect 32/0.
  NOT re-run this session: suite runs DISK-BLOCKED —
  `df -h /`: 17.7 GiB free < 20 GiB gate (DISK-REFUSED by hook); no
  purgeable `target-lane*` roots on disk. Acceptance (a)-(f) test targets
  live at the anchors above; mutation-court inversions for (b)/(f)
  still pending.
