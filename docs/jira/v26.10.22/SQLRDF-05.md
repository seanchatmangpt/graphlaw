# SQLRDF-05 — Expression-level graphs for the other admitted classes

- **Id:** SQLRDF-05
- **Title:** Expression-level graphs for the other admitted classes
- **Status:** Open

## Conformance target

The section 3.2 inventory of the pasted sql-rdf Recommendation, the conformance text for v26.10.22, and section 6.1 (a Turtle document for each admitted class executes on its own real frozen-duckdb connection).

This ticket covers the admitted classes other than `SELECT` and other than insert, update, delete, and the section 7 merge: create, alter, drop, truncate, comment, copy, export, import, prepare, execute, deallocate, explain, vacuum, analyze, checkpoint, call, pragma, set, reset, transaction begin/commit/rollback, load, install, update extensions, extension repository, external resource, attach, detach, use.

## Current behavior

Each of those classes is one fixed string in `DatasetReader::compile_command` ([`src/sql_ast.rs`](../../../src/sql_ast.rs) lines 771–911). `TransactionStatement` emits only `BEGIN` (line 852). The other arms interpolate `sql:targetTable`, `sql:assignColumn`, and `sql:assignValue` into a single template and return `params: Vec::new()`.

[`ggen/sql-dialect/statement_classes.ttl`](../../../ggen/sql-dialect/statement_classes.ttl) constrains those classes to identifier slots (`sql:targetTable` and, where present, `sql:assignColumn`).

[`tests/duckdb_statement_kinds.rs`](../../../tests/duckdb_statement_kinds.rs) `every_statement_kind_runs` executes the raw SQL strings in [`ggen/sql-dialect/sql_statement_kinds.rs`](../../../ggen/sql-dialect/sql_statement_kinds.rs). [`tests/chicago_validation.rs`](../../../tests/chicago_validation.rs) `every_peg_statement_compiles_and_runs_or_refuses` compiles the same template vocabulary and runs it on `GraphLawDuckDB::open_in_memory_allowing_unsigned_extensions`.

`SELECT` expression compilation (`extract_select_statement`, lines 499–616) and the insert, update, and delete templates are already in the tree. This ticket does not ask for them again. The section 7 merge is SQLRDF-04.

## Required behavior

Each class in the list above has a SHACL shape and a Turtle document whose expression graph compiles and executes on its own connection. Transaction begin, commit, and rollback are three executions of `sql:TransactionStatement`, each on its own connection. Template-only compilation does not close this ticket.

## Acceptance

`cargo test --test duckdb_statement_kinds` and `cargo test --test chicago_validation -- every_peg_statement_compiles_and_runs_or_refuses`.
