# SQLRDF-07 — REFUSAL-03 scope integrity

- **Id:** SQLRDF-07
- **Title:** REFUSAL-03 scope integrity
- **Status:** Open

## Conformance target

Sections 1.3 and 5 of the pasted sql-rdf Recommendation, the conformance text for v26.10.22. `REFUSAL-03` is `ScopeIntegrityRefusal`.

Section 1.3 distinguishes viewing from processing. A column in the select list is `sql:ViewScope`. A column inside `WHERE`, `JOIN`, or `HAVING` is `sql:ProcessScope`. Section 5 triggers `REFUSAL-03` when a term crosses that boundary, including an aggregate placed inside `WHERE`.

## Current behavior

`serialize_expr` in [`src/sql_ast.rs`](../../../src/sql_ast.rs) (lines 293–340) writes column references, literals, binary expressions, function calls, and aliases. An aggregate under `sql:whereClause` is serialized like any other expression. No reader in `extract_select_statement` (lines 499–616) loads `sql:scope`, `sql:ViewScope`, or `sql:ProcessScope`.

[`docs/refusals.md`](../../refusals.md) has no `REFUSAL-03` row.

## Required behavior

An aggregate under `sql:where`, or a `sql:ProcessScope` term in the select list, is `REFUSAL-03`. The database is unchanged. A `sql:ViewScope` column in the select list still executes.

The code is stable in [`docs/refusals.md`](../../refusals.md).

## Acceptance

`cargo test --test chicago_validation`.
