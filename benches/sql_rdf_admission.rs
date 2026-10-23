//! Divan benchmarks for the sql-rdf admission hot path (v26.10.22).
//!
//! Paths measured, all REAL APIs (no mocks — Chicago law):
//! 1. `blake3::hash` over the raw Turtle bytes — the floor any receipt
//!    hash can beat against.
//! 2. `graphlaw::sql_ast::compile_turtle` — SHACL admission + compile,
//!    one call (`src/sql_ast.rs`). This is the hot path under benchmark.
//! 3. `graphlaw::duckdb_eval::canonical_ast_graph_bytes` — RDFC-1.0
//!    canonicalization cost, measured separately from full admission so
//!    the canon share of the compile is visible.
//! 4. `graphlaw::duckdb_eval::fold_params_anchored` — receipt anchor fold
//!    `H_query = BLAKE3(h_ast || P_params)` on the REAL compiled params
//!    (`CompiledQuery::params`), the per-receipt steady state.
//!
//! Point-query Turtle uses the ADMITTED shape: root `sql:SelectStatement`
//! with `sql:statementIndex`, `sql:fromTable`, `sql:selectList` of column
//! nodes, and the bounded-WHERE pair `sql:whereColumn` + `sql:whereValue`
//! (tests/sql_rdf_dialect.rs:1391 shape). Labeled blank nodes (`_:b`),
//! per the purrdf anonymous-collection refusal.

use divan::Bencher;

#[divan::bench]
fn raw_blake3_turtle_bytes(bencher: Bencher) {
    bencher
        .counter(divan::counter::BytesCount::new(POINT_QUERY.len()))
        .bench(|| {
            let h = blake3::hash(POINT_QUERY.as_bytes());
            std::hint::black_box(h);
        });
}

/// Full admission: SHACL validation + parse + compile + seal in one call.
#[divan::bench]
fn compile_turtle_admission(bencher: Bencher) {
    bencher.bench(|| {
        let compiled = graphlaw::sql_ast::compile_turtle(POINT_QUERY)
            .expect("point query must admit (courts admit this shape)");
        std::hint::black_box(&compiled);
    });
}

/// Canonicalization only: purrdf parse_dataset + try_canonicalize.
#[divan::bench]
fn canonical_ast_graph_bytes(bencher: Bencher) {
    bencher.bench(|| {
        let bytes = graphlaw::duckdb_eval::canonical_ast_graph_bytes(POINT_QUERY)
            .expect("canonicalization must succeed on an admitted graph");
        std::hint::black_box(bytes);
    });
}

/// Per-receipt steady state: fold the REAL compiled params against the
/// REAL compiled `receipt_state` anchor (`fold_params_anchored`).
#[divan::bench]
fn fold_params_anchored_compiled(bencher: Bencher) {
    let compiled = graphlaw::sql_ast::compile_turtle(POINT_QUERY)
        .expect("point query must admit");
    let h_ast = compiled
        .receipt_state
        .expect("graph-backed compile carries receipt_state");
    bencher.bench(|| {
        let h = graphlaw::duckdb_eval::fold_params_anchored(&h_ast, &compiled.params);
        std::hint::black_box(h);
    });
}

/// Point query in the admitted bounded-SELECT shape (labeled blank nodes).
const POINT_QUERY: &str = r#"@prefix ex: <http://example.org/fibo#> .
@prefix sql: <http://graphlaw.org/sql#> .

ex:q a sql:SelectStatement ;
    sql:statementIndex 0 ;
    sql:fromTable _:ft ;
    sql:whereColumn "balance" ;
    sql:whereValue "1250.50"^^<http://www.w3.org/2001/XMLSchema#decimal> ;
    sql:selectList ( _:c1 _:c2 ) .

_:ft sql:tableName "fibo_accounts" .
_:c1 sql:columnName "account_id" .
_:c2 sql:columnName "balance" .
"#;

fn main() {
    // Fail fast, loudly, if the fixture ever stops admitting: a bench
    // silently skipping its subject is a vacuous pass.
    graphlaw::sql_ast::compile_turtle(POINT_QUERY).expect("POINT_QUERY must admit");
    divan::main();
}
