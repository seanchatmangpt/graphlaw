# SQLRDF-01 — Normative vocabulary

- **Id:** SQLRDF-01
- **Title:** Normative vocabulary
- **Status:** Open

## Conformance target

Sections 3.1 and 3.3 of the pasted sql-rdf Recommendation, the conformance text for v26.10.22.

Section 3.1 roots every statement at `sql:ASTNode` and groups the PEG classes under six family classes. Section 3.3 attaches clauses through `sql:hasClause` and names `sql:select`, `sql:from`, `sql:where`, `sql:orderBy`, and `sql:having`, plus `sql:statementIndex`, `sql:ViewScope`, `sql:ProcessScope`, and `sql:scope`.

## Current behavior

[`ontologies/sql-ast.ttl`](../../../ontologies/sql-ast.ttl) declares `sql:Statement` as a subclass of `owl:Thing` (lines 19–20). The clause properties in current use are the aliases `sql:selectList`, `sql:fromTable`, and `sql:whereClause` (lines 59–94). `sql:groupBy`, `sql:limit`, and `sql:offset` are already declared (lines 96–113) and are compiled for `SELECT`.

[`ggen/sql-dialect/statement_classes.ttl`](../../../ggen/sql-dialect/statement_classes.ttl) lists the 35 PEG classes as direct subclasses of `sql:Statement`. It has no family classes.

The ontology has no `sql:ASTNode`, `sql:hasClause`, `sql:statementIndex`, `sql:ViewScope`, `sql:ProcessScope`, or `sql:scope`, and it does not declare the section 3.3 names `sql:select`, `sql:from`, `sql:where`, `sql:orderBy`, or `sql:having`. `sql:havingClause` and `sql:orderBy` exist as properties (lines 100–106) and are outside this ticket’s compiler work.

[`src/sql_ast.rs`](../../../src/sql_ast.rs) already compiles `sql:selectList`, `sql:fromTable`, one `sql:joinClause`, `sql:whereClause`, `sql:groupBy`, `sql:limit`, and `sql:offset` for `SELECT` (`SelectStatement::compile` at lines 222–290, `extract_select_statement` at lines 499–616). This ticket does not ask for that compiler again.

## Required behavior

Add the section 3.1 and 3.3 vocabulary:

- `sql:ASTNode`, with `sql:Statement` under it
- the six family classes: `sql:DataQueryStatement`, `sql:DataManipulationStatement`, `sql:DataDefinitionStatement`, `sql:TransactionStatement`, `sql:SessionStatement`, `sql:ExtensionStatement`
- `sql:hasClause` and `sql:statementIndex`
- `sql:ViewScope`, `sql:ProcessScope`, and `sql:scope`
- the section 3.3 clause names `sql:select`, `sql:from`, `sql:where`, `sql:orderBy`, and `sql:having`

Keep `sql:selectList`, `sql:fromTable`, and `sql:whereClause` as accepted aliases so documents that already use them still compile. Leave `sql:groupBy`, `sql:limit`, and `sql:offset` as they are.

## Acceptance

`cargo test --test sql_rdf_dialect` and `cargo test --test chicago_validation -- every_peg_statement_compiles_and_runs_or_refuses` stay green on the alias documents.
