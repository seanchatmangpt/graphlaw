//! Engine-frontier benchmarks: graph-backed SQL-RDF eval vs the engine's own
//! prepared-statement floor, on real frozen-duckdb connections (Chicago: no
//! mocks, no sleeps). The prepare-vs-eval delta is the P1 headline.
//!
//! Legs:
//! 1. `cold_graph_eval` — full graph-backed eval: `compile_turtle` (SHACL
//!    validation + canonicalization + SQL lowering) + `eval_ast_query` on a
//!    seeded in-memory database.
//! 2. `c1_hit_eval` — repeat `eval_ast_query` of the SAME compiled query on
//!    the same connection: the C1 prepared-statement cache HIT path (the P1
//!    floor the compiled-plan cache defends).
//! 3. `fresh_connection_eval` — fresh `GraphLawDuckDB::open_in_memory` +
//!    minimal seed + eval per iteration: fresh-connection cost.
//! 4. `raw_engine_prepared` — raw `frozen_duckdb::Connection::prepare_cached`
//!    + query of the equivalent SQL string: engine-internal prepared cost.
//!
//! ENGINE_LOCK: the frozen engine segfaults under concurrent engine-heavy
//! threads (witnessed; see tests/duckdb_statement_kinds.rs:11). Divan benches
//! in one process share the lock, so it is taken ONCE at setup via a
//! `LazyLock` that holds the guard for the process lifetime — never
//! per-iteration, or the benchmark would measure lock contention.

use std::hint::black_box;
use std::sync::Mutex;

use graphlaw::duckdb_eval::{GraphLawDuckDB, ExecutionReceipt};
use graphlaw::sql_ast::{CompiledQuery, compile_turtle};

static ENGINE_LOCK: Mutex<()> = Mutex::new(());

/// The seeded in-memory database: table `t(id INTEGER, name VARCHAR)` with
/// two rows, per tests/chicago_validation.rs conventions.
const SEED_SQL: &str = "CREATE TABLE t(id INTEGER, name VARCHAR); \
                        INSERT INTO t VALUES (1, 'a'), (2, 'b');";

/// Graph source for a SELECT over the seeded table (the exact shape of the
/// chicago_validation SelectStatement fixture).
const SELECT_TURTLE: &str = concat!(
    "@prefix sql: <http://graphlaw.org/sql#> .\n",
    "@prefix xsd: <http://www.w3.org/2001/XMLSchema#> .\n",
    "<http://example.org/s> a sql:SelectStatement ;\n",
    "    sql:statementIndex 1 ;\n",
    "    sql:select ( [ a sql:ColumnIdentifier ; sql:columnName \"id\" ; sql:scope sql:ViewScope ] ) ;\n",
    "    sql:from ( [ a sql:TableIdentifier ; sql:tableName \"t\" ] ) .\n",
);

fn compiled_select() -> CompiledQuery {
    compile_turtle(SELECT_TURTLE).expect("SELECT graph must compile")
}

fn open_seeded() -> GraphLawDuckDB {
    let db = GraphLawDuckDB::open_in_memory().expect("in-memory engine must open");
    db.raw_connection()
        .execute_batch(SEED_SQL)
        .expect("seed must succeed");
    db
}

fn main() {
    // Take the engine lock ONCE for the whole bench process before any bench
    // runs: leaking the guard keeps the mutex held for the process lifetime
    // (the guard is never dropped), so no per-iteration locking occurs.
    std::mem::forget(ENGINE_LOCK.lock().unwrap_or_else(|p| p.into_inner()));

    // Warm correctness court: every leg must produce the real receipt shape
    // before timing, or the bench is measuring nothing.
    {
        let compiled = compiled_select();
        let db = open_seeded();
        let r = db
            .eval_ast_query(
                "http://example.org/bench/warm",
                &compiled,
                "urn:r:",
                "urn:p:",
            )
            .expect("warm eval must succeed");
        assert_eq!(r.rows_emitted, 2, "warm SELECT must read back both rows");
        assert!(compiled.sql.contains("SELECT"), "lowered sql: {}", compiled.sql);
    }

    divan::Divan::from_args().main();
}

// Leg 1 — cold path: compile the Turtle graph to SQL (SHACL + canon + lower)
// AND evaluate it, per iteration.
#[divan::bench]
fn cold_graph_eval() {
    let db = open_seeded();
    let receipt: ExecutionReceipt = db
        .eval_ast_query(
            "http://example.org/bench/cold",
            black_box(&compiled_select()),
            "urn:r:",
            "urn:p:",
        )
        .expect("cold eval must succeed");
    black_box(receipt.rows_emitted);
}

// Leg 2 — C1 HIT path: the SAME connection, the SAME compiled query,
// evaluated repeatedly (prepared-statement reuse; the P1 floor).
#[divan::bench]
fn c1_hit_eval(bencher: divan::Bencher) {
    let db = open_seeded();
    let compiled = compiled_select();
    // First eval populates the C1 cache; all timed iterations are hits.
    db.eval_ast_query("http://example.org/bench/hit", &compiled, "urn:r:", "urn:p:")
        .expect("cache-fill eval must succeed");
    // The C1 compiled-plan cache serves this leg: the entry is stored by the
    // fill eval, and every timed eval consults it (short-circuiting before
    // any engine call). Witness the stored entry.
    assert_eq!(
        db.shared_cache().len(),
        1,
        "C1 cache must hold exactly the compiled-plan entry"
    );
    let reference_hash = {
        let r = db
            .eval_ast_query("http://example.org/bench/hit", &compiled, "urn:r:", "urn:p:")
            .expect("reference eval must succeed");
        assert!(r.cached, "reference eval must be served from the C1 cache");
        r.query_hash
    };
    bencher.bench_local(|| {
        let receipt = db
            .eval_ast_query(
                "http://example.org/bench/hit",
                black_box(&compiled),
                "urn:r:",
                "urn:p:",
            )
            .expect("hit eval must succeed");
        assert!(receipt.cached, "timed evals must be C1 cache hits");
        assert_eq!(
            receipt.query_hash, reference_hash,
            "C1 hits reproduce the receipt identity"
        );
        black_box(receipt.rows_emitted)
    });
}

// Leg 3 — fresh-connection cost: open + seed + eval per iteration.
#[divan::bench]
fn fresh_connection_eval() {
    let db = open_seeded();
    let compiled = compiled_select();
    let receipt = db
        .eval_ast_query(
            "http://example.org/bench/fresh",
            black_box(&compiled),
            "urn:r:",
            "urn:p:",
        )
        .expect("fresh eval must succeed");
    black_box(receipt.rows_emitted);
}

// Leg 4 — engine-internal floor: the raw frozen-duckdb connection with
// prepare_cached over the equivalent SQL string (no graph, no receipt).
#[divan::bench]
fn raw_engine_prepared(bencher: divan::Bencher) {
    let db = open_seeded();
    let sql = compiled_select().sql;
    // Prepare once outside timing, then time prepare_cached + query: the
    // engine's own prepared-statement floor for the same plan.
    db.raw_connection()
        .prepare_cached(&sql)
        .expect("raw prepare must succeed");
    bencher.bench_local(|| {
        let mut stmt = db
            .raw_connection()
            .prepare_cached(black_box(&sql))
            .expect("raw prepare_cached must succeed");
        let mut rows = stmt.query([]).expect("raw query must succeed");
        let mut n = 0usize;
        while rows.next().unwrap().is_some() {
            n += 1;
        }
        black_box(n)
    });
}
