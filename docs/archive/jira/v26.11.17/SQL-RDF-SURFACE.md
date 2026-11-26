# SQL-RDF Surface (Lane 07), v26.11.17

Static inspection 2026-11-17; no tests run.

## 1. Statement-kind coverage
- 35/35 PEG kinds generated (`ggen/sql-dialect/sql_statement_kinds.rs`, 35 `(Kind, SQL)` rows) from the frozen DuckDB PEG `Statement` rule (commit b6013f7aa2) via extract_statements.py + `ggen sync run`. `src/sql_statement_kinds.rs` only `include!`s them.
- Rec §3.2: 33 execute, 2 typed refusals (Connect/Disconnect, REFUSAL-02).
- Extension kinds beyond the 35: SetOperationStatement, PivotStatement, UnpivotStatement (dispatched, absent from Rec §3.2).
- Pin court: tests/duckdb_statement_kinds.rs::generated_kinds_match_the_peg_statement_rule needs the frozen .gram and panics if absent. Not run.

## 2. Pipeline
1. `compile_turtle` (src/sql_ast.rs:1951) sole admission path; `compile_turtle_document` (:2250) multi-statement.
2. SHACL validation (`validate_ast_shacl` :1812) against sql-ast.ttl + statement_classes.ttl; failure = REFUSAL-01.
3. statement kind, require_statement_index (also nested usingSource), canonicalize, `h_ast = BLAKE3(Canon(G_AST))`.
4. memo (SelectStatement only); `compile_statement_arm` (:2068); `compile_command` (:3508).
5. UPDATE/DELETE without where-predicates: REFUSAL-05 UnboundedMutation.
6. stamp receipt_state, canon_graph_bytes, cacheable, admission_seal (RUST-5).
7. Eval (duckdb_eval.rs: eval_ast_query[_bounded] :1262/:1285, eval_ast_document[_bounded] :1321/:1347, eval_admitted_ast :1896): `verify_seal` first (:1456); receipt `H_query = BLAKE3(h_ast || P_params)` (:829, Rec §1.4); missing canon graph = NoCanonicalGraph; cache consult; prepare/execute/receipt.
8. duckdb_op.rs: builtin dispatch with typed `DuckDb` refusals.

## 3. Refusals
Rec §5 documents REFUSAL-01..04; code adds REFUSAL-05 and internal codes 16-21, 36, 37 (src/sql_ast/refusals.rs) not in the Rec.

## 4. Gaps vs Recommendation
- G1 §5: REFUSAL-05 and internal codes undocumented.
- G2 §3.2/§6.1(3): "executes to completion" is shallow for several kinds: render arms emit fixed SQL (Transaction always BEGIN, Vacuum always VACUUM, Attach `ATTACH ':memory:' AS x`, Call `CALL name()`); graph cannot express COMMIT/ROLLBACK/CALL args.
- G3: Set-op/Pivot/Unpivot not in Rec inventory.
- G4: StructLiteral, ListLiteral, IndexAccess, SliceAccess -> UnsupportedTerm on inline path (sql_ast.rs:3482-3503; SQLRDF-25 Row 4).
- G5: recursive CTE body "not graph-expressible" notes (:4812, :4893) may be stale now that set-op arm exists.
- G6: comment "no cache store exists yet" (~:1997) likely stale; query_cache seam exists.
- G7: Connect/Disconnect messages illustrative; courts pin loose substrings (tests/duckdb_statement_kinds.rs:123-124).

## 5. Markers
No TODO/FIXME/todo!/unimplemented! in lane files. One `PENDING-LANE` at sql_ast.rs:3482. UnsupportedTerm sites: ast.rs:450,472,542; sql_ast.rs:3032,3057,3171,3254,3703.

## 6. Follow-ups
Run duckdb_statement_kinds and sql_rdf_conformance with the frozen grammar; amend the Rec; resolve G2; refresh stale comments.
