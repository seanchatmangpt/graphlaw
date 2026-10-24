//! sql-rdf compiler cost-split benchmarks (Divan).
//!
//! Isolates the internal cost split of `graphlaw::sql_ast::compile_turtle`
//! (the P1 target the ring cache circumvents). Legs:
//!
//! 1. `compile_full_cache_miss` — full `compile_turtle` on a canonically
//!    fresh doc per call (memo never engages; the true miss path).
//! 2. `compile_repeat_same` — full `compile_turtle` on the SAME doc per call
//!    (Charter-A04 memo hit path, sql_ast.rs AST_COMPILE_MEMO_CAPACITY=256).
//! 3. `shacl_validate_only` — `validate_ast_shacl` over the same doc.
//! 4. `canon_blake3_only` — purrdf parse + `try_canonicalize` (RDFC-1.0) +
//!    BLAKE3, i.e. `Canon(G_AST)` + `h_ast` without SHACL/extract/serialize.
//! 5. `serialize_only` — hand-built `SelectStatement` (tc05 shape) through the
//!    public `SelectStatement::compile` serializer entrypoint.
//!
//! Parse-only leg: `DatasetReader` is `struct DatasetReader` (private,
//! sql_ast.rs:1700) and `from_turtle` is not publicly reachable, so that leg
//! is not measurable from outside the crate — its cost is the residual of
//! (miss) − (shacl + canon + serialize).
//!
//! Chicago: real compile paths over real fixture bytes; no mocks, no sleeps.
//! Run: `cargo bench --bench sql_rdf_compiler`

use std::sync::atomic::{AtomicUsize, Ordering};

use divan::{Bencher, counter::ItemsCount};
use graphlaw::sql_ast::{compile_turtle, validate_ast_shacl};

/// tc05_select_sum fixture shape (tests/fixtures/fibo/tc05_select_sum.ttl):
/// bare-aggregate SUM select, admitted shape, compiles to
/// `SELECT SUM(balance) FROM fibo_accounts`.
const TC05_TTL: &str = r#"
@prefix ex: <http://example.org/fibo#> .
@prefix sql: <http://graphlaw.org/sql#> .

ex:stmt1 a sql:SelectStatement ;
    sql:statementIndex 5 ;
    sql:fromTable [ sql:tableName "fibo_accounts" ] ;
    sql:selectList ( [ sql:expression [
        sql:function <http://graphlaw.org/functions#SUM> ;
        sql:argument [ sql:columnName "balance" ] ] ] ) .
"#;

/// SHACL shapes exactly as compile_turtle builds them (sql_ast.rs:1420).
fn shapes() -> String {
    format!(
        "{}\n{}",
        include_str!("../ontologies/sql-ast.ttl"),
        include_str!("../ggen/sql-dialect/statement_classes.ttl")
    )
}

/// Monotonic doc counter: every `compile_turtle` miss-path call gets a
/// canonically distinct doc (unique tableName), so the Charter-A04 memo
/// (capacity 256, flush-on-overflow) can never serve it from cache.
static FRESH_DOC: AtomicUsize = AtomicUsize::new(0);

fn fresh_turtle() -> String {
    let n = FRESH_DOC.fetch_add(1, Ordering::Relaxed);
    TC05_TTL.replace(
        "fibo_accounts",
        &format!("fibo_accounts_{n:06}")[..],
    )
}

fn main() {
    // Warm-up + sanity: the fixture compiles and seals on a real path.
    let compiled = compile_turtle(TC05_TTL).expect("tc05 compiles");
    assert!(compiled.sql.starts_with("SELECT SUM(balance)"));
    assert!(compiled.verify_seal(), "admission seal verifies");
    let repeated = compile_turtle(TC05_TTL).expect("repeat compile");
    assert_eq!(compiled.sql, repeated.sql);

    divan::main();
}

/// Leg 1 — full compile_turtle, guaranteed cache MISS (fresh doc per call).
#[divan::bench]
fn compile_full_cache_miss(b: Bencher) {
    b.counter(ItemsCount::new(1u64)).bench(|| {
        let turtle = fresh_turtle();
        compile_turtle(&turtle).expect("fresh doc compiles")
    });
}

/// Leg 2 — full compile_turtle, guaranteed memo HIT (same doc every call).
#[divan::bench]
fn compile_repeat_same(b: Bencher) {
    b.bench(|| {
        compile_turtle(TC05_TTL).expect("tc05 compiles")
    });
}

/// Leg 3 — SHACL validation only (the first phase of compile_turtle).
#[divan::bench]
fn shacl_validate_only(b: Bencher) {
    let shapes = shapes();
    b.bench(|| {
        validate_ast_shacl(TC05_TTL, &shapes).expect("tc05 validates")
    });
}

/// Leg 4 — parse + RDFC-1.0 canonicalize + BLAKE3 only (Canon(G_AST), h_ast).
#[divan::bench]
fn canon_blake3_only(b: Bencher) {
    b.bench(|| {
        let dataset = purrdf::parse_dataset(TC05_TTL.as_bytes(), "text/turtle", None)
            .expect("parses");
        let canon = purrdf::try_canonicalize(&dataset).expect("canonicalizes");
        blake3::hash(canon.nquads.as_bytes())
    });
}

/// Leg 5 — serializer only: hand-built tc05-shape SelectStatement through the
/// public `SelectStatement::compile` entrypoint (no SHACL, no parse, no
/// canon, no extract).
#[divan::bench]
fn serialize_only(b: Bencher) {
    b.bench(|| {
        use graphlaw::sql_ast::{SelectStatement, SqlExpr, TableRef};
        let stmt = SelectStatement {
            ctes: Vec::new(),
            qualify: None,
            from: TableRef {
                name: "fibo_accounts".into(),
                alias: None,
                schema: None,
            },
            joins: Vec::new(),
            select_list: vec![SqlExpr::FunctionCall {
                function: graphlaw::sql_ast::AggregateFunction::Sum,
                argument: Box::new(SqlExpr::ColumnRef {
                    column: "balance".into(),
                    table_alias: None,
                }),
            }
            .into()],
            where_clause: None,
            group_by: Vec::new(),
            having: None,
            order_by: Vec::new(),
            limit: None,
            offset: None,
        };
        stmt.compile().expect("serializer compiles")
    });
}
