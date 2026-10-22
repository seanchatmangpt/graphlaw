# SQLRDF-06 — REFUSAL-02 for ConnectStatement and DisconnectStatement

- **Id:** SQLRDF-06
- **Title:** REFUSAL-02 for ConnectStatement and DisconnectStatement
- **Status:** Open

## Conformance target

Section 5 of the pasted sql-rdf Recommendation, the conformance text for v26.10.22. `REFUSAL-02` is `UnimplementedStatementRefusal`. Section 3.2 records `ConnectStatement` and `DisconnectStatement` as engine refusals.

## Current behavior

`compile_turtle` in [`src/sql_ast.rs`](../../../src/sql_ast.rs) (lines 667–671) returns `AstError::ShaclViolation` with the message `compile refusal: {kind} is not implemented by this DuckDB build` for `ConnectStatement` and `DisconnectStatement`. The match arm returns before `compile_command` and before any DuckDB call.

[`tests/chicago_validation.rs`](../../../tests/chicago_validation.rs) `every_peg_statement_compiles_and_runs_or_refuses` (lines 56–66) requires that message to contain `compile refusal`. `statement_object` (lines 203–213) records the same error as `DenialPolarity::CONFORMANCE_GATE_FAILED`.

[`docs/refusals.md`](../../refusals.md) has no `REFUSAL-02` row. The SQL row is `AstError` with reasons `InjectionAttempt`, `MissingWhereClause`, and `ShapeViolation`.

## Required behavior

`docs/refusals.md` records the stable code `REFUSAL-02` (`UnimplementedStatementRefusal`) for `ConnectStatement` and `DisconnectStatement`. `compile_turtle` returns that code. The statements are not executed.

## Acceptance

`cargo test --test chicago_validation -- every_peg_statement_compiles_and_runs_or_refuses`.

## Correction (2026-10-22)

The AstError reasons list in Current behavior (`InjectionAttempt`, `MissingWhereClause`, `ShapeViolation`) is stale; it was replaced by the real AstError variants plus stable `[REFUSAL-nn]` markers. See [`docs/refusals.md`](../../refusals.md) for the current register.
