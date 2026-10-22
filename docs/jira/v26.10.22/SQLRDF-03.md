# SQLRDF-03 — Clause compiler for having, orderBy, and sql:from

- **Id:** SQLRDF-03
- **Title:** Clause compiler for having, orderBy, and sql:from
- **Status:** Open

## Conformance target

Section 3.3 of the pasted sql-rdf Recommendation, the conformance text for v26.10.22.

`sql:having` and `sql:orderBy` are subproperties of `sql:hasClause`. `sql:from` is an `rdf:List` of data sources. `sql:scope` stores `sql:ViewScope` or `sql:ProcessScope` on an `sql:ASTNode`.

## Current behavior

`SelectStatement` in [`src/sql_ast.rs`](../../../src/sql_ast.rs) (lines 202–210) holds `from`, `joins`, `select_list`, `where_clause`, `group_by`, `limit`, and `offset`. It has no `having`, no `order_by`, no schema, and no scope.

`SelectStatement::compile` (lines 222–290) emits `SELECT`, `FROM`, one join, `WHERE`, `GROUP BY`, `LIMIT`, and `OFFSET`. `extract_select_statement` (lines 499–616) reads `sql:fromTable` as a single `TableRef { name, alias }`. It never reads `sql:havingClause` or `sql:orderBy`.

Those two properties are already declared in [`ontologies/sql-ast.ttl`](../../../ontologies/sql-ast.ttl) (lines 100–106). `sql:groupBy`, `sql:limit`, and `sql:offset` already compile. This ticket does not ask for them again.

## Required behavior

Compile `sql:having` and `sql:orderBy`. Accept `sql:from` as an `rdf:List` of table identifiers with optional schema. Store scope IRIs on nodes.

The `SELECT` path that already compiles `sql:selectList`, `sql:fromTable`, one join, `sql:whereClause`, `sql:groupBy`, `sql:limit`, and `sql:offset` stays in place. Alias documents from SQLRDF-01 still compile.

## Acceptance

`cargo test --test sql_rdf_dialect` gains a `SELECT` that groups, filters with `HAVING`, and orders, executed on `GraphLawDuckDB::open_in_memory`, with the result checked by a table readback.
