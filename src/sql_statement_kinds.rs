//! Statement kinds + extensions of the SQL dialect, re-exported verbatim.
//!
//! What this is: `include!` of the ggen-generated 35 PEG statement kinds
//! (`ggen/sql-dialect/sql_statement_kinds.rs`) plus the extensions surface
//! (`ggen/sql-dialect/sql_extensions.rs`).
//!
//! Regeneration: `ggen sync run` from `ggen/sql-dialect/` (never hand-edit the
//! included files). Provenance: the ontology in `ontology/statements.ttl` is
//! extracted from the `Statement` rule of frozen DuckDB PEG commit `b6013f7aa2`.
//!
//! Pinned by the court `generated_kinds_match_the_peg_statement_rule`
//! (tests/duckdb_statement_kinds.rs), which asserts the generated kinds equal
//! the names parsed from the frozen `.gram` grammar file; that court has an
//! environmental precondition — the checkout of the grammar must exist at
//! `GRAM` (frozen-duckdb path), else the court panics loudly, never silently
//! skips.

include!("../ggen/sql-dialect/sql_statement_kinds.rs");
include!("../ggen/sql-dialect/sql_extensions.rs");
