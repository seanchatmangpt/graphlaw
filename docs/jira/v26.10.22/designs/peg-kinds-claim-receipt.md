# Receipt: L60 — "35 PEG statement kinds" claim audit (L50 finding)

Date: 2026-10-22. Lane: L60, /Users/sac/graphlaw, read-only on tests/ and CLAUDE.md.

## Question
CLAUDE.md Commands block claims `cargo test --test duckdb_statement_kinds  # 35 PEG
statement kinds on real frozen-duckdb`, but the target contains 6 `#[test]`
functions. Is 35 witnessed coverage inside those fns, or stale?

## Counting commands + output
    $ grep -c "a sql:StatementKind" ggen/sql-dialect/ontology/statements.ttl
    35
    $ grep -c '("' ggen/sql-dialect/sql_statement_kinds.rs
    35
    $ grep -n "for (kind" tests/duckdb_statement_kinds.rs
    358:    for (kind, _) in SQL_STATEMENTS {

tests/duckdb_statement_kinds.rs line 9 imports
`graphlaw::sql_statement_kinds::{SQL_EXTENSIONS, SQL_STATEMENTS}` (the generated
table in ggen/sql-dialect/sql_statement_kinds.rs). `every_statement_kind_runs`
(line 103) iterates all of `SQL_STATEMENTS` (line 358), exercising each kind on a
real `GraphLawDuckDB` connection with readbacks. `generated_kinds_match_the_committed_grammar`
(line 47) asserts the generated table equals `ontology/statements.ttl` — so the two
35-counts are coupled by a passing test, not coincidence.

## Test run (real output)
    $ CARGO_TARGET_DIR=target-lane60 cargo test --test duckdb_statement_kinds
    exit=0
    running 6 tests
    test generated_kinds_match_the_committed_grammar ... ok
    test expression_courts_flip_under_operator_inversion ... ok
    test every_statement_kind_runs ... ok
    test every_command_class_compiles_from_expressions_and_runs_on_its_own_connection ... ok
    test every_source_extension_answers_or_is_absent ... ok
    test section6_2_certified_checks_run_through_compiled_ast_graphs ... ok

    test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.19s

## Verdict
CLAIM-ACCURATE. The vocabulary (ggen/sql-dialect/ontology/statements.ttl) declares
exactly 35 `sql:StatementKind` individuals; the generated SQL_STATEMENTS table has
exactly 35 tuples; and the 6 test functions — chiefly `every_statement_kind_runs` —
execute all 35 against real frozen-duckdb connections. The "35" in CLAUDE.md is
witnessed coverage, not stale. No CLAUDE.md edit required.
