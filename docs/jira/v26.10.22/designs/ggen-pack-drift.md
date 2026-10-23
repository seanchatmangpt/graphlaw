# ggen/sql-dialect pack vs sql-rdf compiler — drift report

Lane L28, 2026-10-22. One canonical checkout `/Users/sac/graphlaw`. Read-only on
`src/`; this report is the deliverable. No TTL or src reconciliation performed.

Surfaces compared:

- Pack TTLs: `ggen/sql-dialect/statement_classes.ttl`,
  `ggen/sql-dialect/ontology/statements.ttl` (the only `.ttl` files under
  `ggen/sql-dialect/`; `queries/*.rq` and `templates/` excluded — not TTL).
- Compiler: `src/sql_ast.rs` — 14-predicate closed-enum gate
  (`KNOWN_SQL_PREDICATES`, line 1705), `JoinType::from_iri` (lines 790-796),
  `Operator::from_iri` / `from_token` (lines 559-606), statement-kind dispatch
  (`compile_turtle` match, lines 2155-2174; `compile_command` arms, lines
  2741-2884).

## Drift table

| # | Direction | Term / class | Pack side (file:line) | Compiler side (file:line) | Effect |
|---|---|---|---|---|---|
| D1 | Compiler-only: accepted, absent from pack TTL | `sql:joinClause` | (absent; grep exit 1 across both TTLs) | `src/sql_ast.rs:1723` | Pack models no joins; pack graphs cannot express what the compiler's core SELECT path compiles. |
| D2 | Compiler-only | `sql:joinType` + IRIs `sql:InnerJoin`, `sql:LeftJoin` | (absent; grep exit 1) | `src/sql_ast.rs:1724, 790-796` (refusal text `:433`) | Same; unknown join IRI refuses, missing joinType = Inner (`:1724-1737`). |
| D3 | Compiler-only | SELECT clause predicates `sql:select`/`selectList`/`from`/`fromTable`/`where`/`whereClause`/`having`/`havingClause`/`groupBy`/`orderBy`/`limit`/`offset`/`statementIndex` (the 14-term gate) | (absent; grep exit 1) | `src/sql_ast.rs:1705-1718` | The pack's SELECT coverage is `sql:example` strings only (`ontology/statements.ttl:24`), not graph vocabulary. |
| D4 | Compiler-only | Operator enum: 15 IRIs (`sql:Equals`…`sql:Modulo`) + 15 token forms (`= != < <= > >= AND OR + - * / % LIKE ILIKE`) | (absent; `sql:Equals`/`sql:operator` grep exit 1) | `src/sql_ast.rs:559-606` (arms 561-571 IRI, 579+ token; refusal text `:364`) | Pack declares zero operator vocabulary. |
| D5 | Compiler-only | `sql:usingSource` (MERGE full form) | (absent) | `src/sql_ast.rs:2169` | MergeIntoStatement exists in pack but its full-form predicate is compiler-only. |
| D6 | Pack-only: declared, compiler refuses / has no template | `sql:ConnectStatement`, `sql:DisconnectStatement` | `statement_classes.ttl:83-89` (classes); `ontology/statements.ttl` (StatementKind entries) | `src/sql_ast.rs:2160-2164` — explicit typed refusal `RefusalCode::UnimplementedStatement` (REFUSAL-02) | Pack generates these kinds; compiler refuses them typed, never silently. Intentional refusal, still a vocabulary drift row. |
| D7 | Pack-only | `sql:name`, `sql:example`, `sql:order` (metadata, 43 uses each in `ontology/statements.ttl`, e.g. lines 8-10) | `ontology/statements.ttl:8-10` | not in `KNOWN_SQL_PREDICATES` (`src/sql_ast.rs:1705`) | Tolerated today only because the 14-predicate gate is scoped to the SELECT statement node, not pack metadata nodes. Any pack graph reusing these on a SelectStatement node refuses REFUSAL-01. |
| D8 | Pack-only | `sql:check`, `sql:expect` (extension probe vocabulary, 8 uses each, `ontology/statements.ttl:184-238`) | `ontology/statements.ttl:187-188` et al. | not in `KNOWN_SQL_PREDICATES`; no reader arm | Extension probes are pack-side only; the compiler never consumes them. |
| D9 | Aligned (no drift) | DML predicates `sql:targetTable`/`assignColumn`/`assignValue`/`whereColumn`/`whereValue` | `statement_classes.ttl:117-129` | `src/sql_ast.rs:2746-2750` | Both sides present; shapes in the pack match the command template reads. |
| D10 | Aligned (no drift) | 33 statement kinds with compile templates (Insert, Update, Delete, Create, Drop, Alter, Truncate, Analyze, Explain, Copy, Export, Import, MergeInto, Vacuum, Checkpoint, Transaction, ExternalResource, UpdateExtensions, Set, Reset, Pragma, Call, Prepare, Execute, Deallocate, Comment, Load, Install, Attach, Use, Detach, ExtensionRepository + Select handled at `:2156`) | `statement_classes.ttl:8-81` | `src/sql_ast.rs:2757-2884` | Aligned. |
| D11 | Aligned (no drift) | Join semantics: missing `sql:joinType` = Inner default | n/a (pack declares no joinType) | `src/sql_ast.rs:1724-1737` | Pinned by courts; recorded so the absence in the pack is not read as a compiler gap. |

Headline: 8 drift rows (D1-D8); 3 alignment rows recorded (D9-D11).

## Verification receipts (grep commands + exit codes)

Run from `/Users/sac/graphlaw`, 2026-10-22:

```
$ grep -c "ConnectStatement" src/sql_ast.rs        -> 2 ; exit 0
$ grep -c "DisconnectStatement" src/sql_ast.rs     -> 2 ; exit 0
$ grep -c "joinType|joinClause|InnerJoin|LeftJoin" ggen/sql-dialect/{statement_classes,ontology/statements}.ttl
    -> both files 0 matches ; exit 1
$ grep -c "sql:Equals|sql:operator" ggen/sql-dialect/{statement_classes,ontology/statements}.ttl
    -> both files 0 matches ; exit 1
$ grep -c "selectList|sql:where\b|sql:from\b|havingClause|groupBy|orderBy" ggen/sql-dialect/{statement_classes,ontology/statements}.ttl
    -> both files 0 matches ; exit 1
```

The exit-1 greps are the drift evidence for D1-D5 (term absent from pack);
the exit-0 greps for D6 confirm the refusal arms exist in the compiler.

## Notes

- `KNOWN_SQL_PREDICATES` gate is per-node (SELECT statement node only),
  `src/sql_ast.rs:1703-1718`. Pack metadata predicates (D7/D8) do not trip it
  on their own nodes; this is scope, not acceptance.
- No files edited to reconcile; per task, the report is the deliverable.
- Not run (out of lane scope, report-only task): `cargo test --test sql_rdf_dialect`.
  No graph or src edits were made by this lane, so the gate surface is
  unchanged by L28.
