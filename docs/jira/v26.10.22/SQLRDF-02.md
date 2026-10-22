# SQLRDF-02 — Closed-world SHACL from section 4

- **Id:** SQLRDF-02
- **Title:** Closed-world SHACL from section 4
- **Status:** Open

## Conformance target

Section 4 of the pasted sql-rdf Recommendation, the conformance text for v26.10.22, and `REFUSAL-01` (`MalformedAstRefusal`) from section 5. Shapes live in [`ontologies/sql-ast.ttl`](../../../ontologies/sql-ast.ttl).

Section 4 requires `sql:statementIndex` on every statement, exactly one projection list, an operator token drawn from a fixed string enum, and ordered collections as `rdf:List` (`sql:OrderedListShape`).

## Current behavior

`sql:SelectStatementShape` in [`ontologies/sql-ast.ttl`](../../../ontologies/sql-ast.ttl) (lines 185–197) requires one `sql:fromTable` and one `sql:selectList`. It does not require `sql:statementIndex`, one `sql:select` list, the section 4 operator token enum, or `sql:OrderedListShape`.

Operators today are IRIs. `Operator::from_iri` in [`src/sql_ast.rs`](../../../src/sql_ast.rs) (lines 86–98) accepts `sql:Equals`, `sql:NotEquals`, `sql:GreaterThan`, `sql:GreaterThanOrEq`, `sql:LessThan`, `sql:LessThanOrEq`, `sql:And`, `sql:Or`, and `sql:Like`. Section 4’s `sql:BinaryExpressionShape` instead requires an `xsd:string` in `("=" "!=" "<" "<=" ">" ">=" "AND" "OR" "+" "-" "*" "/" "%" "LIKE" "ILIKE")`.

A hostile table name already refuses before DuckDB and leaves the row count at 1:

- `hostile_identifier_is_refused_and_does_not_change_rows` in [`tests/chicago_validation.rs`](../../../tests/chicago_validation.rs) (lines 105–118)
- `shape_violation_refuses_before_duckdb` in [`tests/sql_rdf_dialect.rs`](../../../tests/sql_rdf_dialect.rs) (lines 41–54)

[`docs/refusals.md`](../../refusals.md) records SQL AST failure as `AstError` with reasons `InjectionAttempt`, `MissingWhereClause`, and `ShapeViolation`. It has no `REFUSAL-01` row.

SELECT fixtures in [`tests/sql_rdf_dialect.rs`](../../../tests/sql_rdf_dialect.rs) and [`tests/chicago_validation.rs`](../../../tests/chicago_validation.rs) omit `sql:statementIndex`.

## Required behavior

Install section 4’s shapes in [`ontologies/sql-ast.ttl`](../../../ontologies/sql-ast.ttl):

- `sql:StatementBaseShape` (`sql:statementIndex`, datatype `xsd:integer`, minCount 1, maxCount 1)
- `sql:SelectStatementShape` (exactly one projection list, at most one `sql:from`, at most one `sql:where`)
- `sql:BinaryExpressionShape` (left, operator token enum, right)
- `sql:OrderedListShape` (`rdf:first` and `rdf:rest`, each minCount 1 and maxCount 1)

A missing index or a disallowed operator is `REFUSAL-01` and does not call DuckDB. The hostile table name still refuses and leaves the row count unchanged.

Alias properties from SQLRDF-01 stay legal. `sql:statementIndex` is an additional required property. Today’s fixtures that omit it are the missing-index case, and that is not a reason to drop the aliases.

## Acceptance

`cargo test --test chicago_validation -- hostile_identifier_is_refused_and_does_not_change_rows` and `cargo test --test sql_rdf_dialect -- shape_violation_refuses_before_duckdb`.

## Correction (2026-10-22)

The AstError reasons list in Current behavior (`InjectionAttempt`, `MissingWhereClause`, `ShapeViolation`) is stale; it was replaced by the real AstError variants plus stable `[REFUSAL-nn]` markers. See [`docs/refusals.md`](../../refusals.md) for the current register.
