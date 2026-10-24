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

/// Point query in the admitted bounded-SELECT shape (labeled blank nodes):
/// root `sql:SelectStatement`, `sql:statementIndex`, `sql:fromTable`,
/// `sql:selectList` column nodes, and a `sql:whereClause` comparison
/// (tests/sql_rdf_dialect.rs:14 shape; `sql:whereColumn`/`sql:whereValue`
/// are the bounded UPDATE/DELETE vocabulary, not SELECT).
const POINT_QUERY: &str = r#"@prefix ex: <http://example.org/fibo#> .
@prefix sql: <http://graphlaw.org/sql#> .
@prefix xsd: <http://www.w3.org/2001/XMLSchema#> .

ex:q a sql:SelectStatement ;
    sql:statementIndex 0 ;
    sql:fromTable _:ft ;
    sql:selectList ( _:c1 _:c2 ) ;
    sql:whereClause _:w .

_:ft sql:tableName "fibo_accounts" .
_:c1 sql:columnName "account_id" .
_:c2 sql:columnName "balance" .
_:w sql:operator sql:GreaterThan ;
    sql:leftOperand [ sql:columnName "balance" ] ;
    sql:rightOperand [ sql:literalValue "1250.50"^^xsd:decimal ] .
"#;

fn main() {
    // Fail fast, loudly, if the fixture ever stops admitting: a bench
    // silently skipping its subject is a vacuous pass.
    graphlaw::sql_ast::compile_turtle(POINT_QUERY).expect("POINT_QUERY must admit");
    divan::main();
}

// ---------------------------------------------------------------------------
// libtest-mode timing (runs TODAY, before the coordinator adds the
// `[[bench]] sql_rdf_admission / harness = false` manifest entry, which is
// coordinator-owned). These exercise the identical benchmark bodies and
// print real wall-clock numbers; once the manifest entry lands and the
// divan harness runs, they remain as admission smoke tests.
// ---------------------------------------------------------------------------

/// Iterations per timing sample; small N because compile_turtle is ~ms-class.
const LIBTEST_ITERS: u32 = 50;

fn time_ns<F: FnMut()>(mut f: F) -> f64 {
    let start = std::time::Instant::now();
    for _ in 0..LIBTEST_ITERS {
        f();
    }
    start.elapsed().as_nanos() as f64 / LIBTEST_ITERS as f64
}

#[test]
fn admission_hot_path_timings() {
    // 0. Fail-fast: the fixture must admit, else every number below is void.
    graphlaw::sql_ast::compile_turtle(POINT_QUERY).expect("POINT_QUERY must admit");

    let bytes = POINT_QUERY.len() as f64;

    // 1. Raw BLAKE3 over the Turtle bytes (floor).
    let blake3_ns = time_ns(|| {
        std::hint::black_box(blake3::hash(POINT_QUERY.as_bytes()));
    });

    // 2. Full compile_turtle admission (SHACL + parse + compile + seal).
    let compile_ns = time_ns(|| {
        let c = graphlaw::sql_ast::compile_turtle(POINT_QUERY).expect("must admit");
        std::hint::black_box(&c);
    });

    // 3. Canonicalization only (purrdf parse + RDFC-1.0 try_canonicalize).
    let canon_ns = time_ns(|| {
        let b = graphlaw::duckdb_eval::canonical_ast_graph_bytes(POINT_QUERY)
            .expect("canon must succeed");
        std::hint::black_box(b);
    });

    // 4. Receipt anchor fold on the REAL compiled params (steady state).
    let compiled = graphlaw::sql_ast::compile_turtle(POINT_QUERY).expect("must admit");
    let h_ast = compiled.receipt_state.expect("graph-backed compile carries receipt_state");
    let fold_ns = time_ns(|| {
        std::hint::black_box(graphlaw::duckdb_eval::fold_params_anchored(&h_ast, &compiled.params));
    });

    // Ordered-sanity guard: admission must dominate the raw-hash floor.
    assert!(
        compile_ns > blake3_ns,
        "compile_turtle admission must cost more than raw BLAKE3 over the same bytes"
    );

    println!("\nsql-rdf admission hot path ({LIBTEST_ITERS} iters, {bytes:.0} Turtle bytes):");
    println!("  raw_blake3_turtle_bytes        {:>12.0} ns", blake3_ns);
    println!("  compile_turtle_admission       {:>12.0} ns", compile_ns);
    println!("  canonical_ast_graph_bytes      {:>12.0} ns", canon_ns);
    println!("  fold_params_anchored_compiled  {:>12.0} ns", fold_ns);
    println!(
        "  canon share of admission:      {:>11.1} %",
        100.0 * canon_ns / compile_ns
    );
}
