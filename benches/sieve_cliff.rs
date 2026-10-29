//! P08 — sieve cliff bench (the result-row envelope vs the raw substrate).
//!
//! Five timed legs per source size N in {10_000, 100_000, 1_000_000}, all
//! over a real in-engine-built Parquet file mounted as a view:
//!
//! - **reduce cold**      — first `eval_ast_query` of `SELECT g, COUNT(v) ..
//!   GROUP BY g` on a fresh connection (C1 miss, cold statement): full
//!   engine reduction to 64 rows.
//! - **reduce warm-stmt** — `cache.bump()` invalidates the C1 entry, so the
//!   eval re-executes on the WARM prepared statement (C1 miss by epoch).
//! - **reduce warm-cache** — plain re-eval: the C1 consult serves the
//!   receipt without touching the engine (`cached == true`).
//! - **refuse**           — `eval_ast_query` of unbounded `SELECT g`:
//!   typed `RowEnvelopeExceeded` at `observed == 65, max == 64`, fired
//!   mid-materialization. Cost is N-independent by construction.
//! - **raw control**      — `raw_connection` pulls ALL N rows of the same
//!   unbounded statement. THE differential: raw scales with N where the
//!   sieved eval refuses at a constant 65.
//!
//! Every cell asserts its contract (rows == 64, refusal exactness, raw
//! count == N) — the numbers only mean something if the legs do what they
//! claim. `--quick` shrinks iteration counts. Chicago: real engine, real
//! Parquet in `std::env::temp_dir()` (removed at scope exit), no mocks, no
//! sleeps.

use std::time::Instant;

use graphlaw::duckdb_eval::{
    DEFAULT_ROW_ENVELOPE, EvaluatorError, GraphLawDuckDB, ROW_ENVELOPE_LIMIT,
};
use graphlaw::sql_ast::{CompiledQuery, compile_turtle};

const SUBJECT: &str = "urn:s/";
const VOCAB: &str = "urn:v/";

/// Drop guard: the temp Parquet artifact is a lease, removed on scope exit.
struct TempParquet(std::path::PathBuf);

impl Drop for TempParquet {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.0);
    }
}

/// Nearest-rank quantile over a sorted slice.
fn quantile(sorted: &[f64], q: f64) -> f64 {
    if sorted.is_empty() {
        return 0.0;
    }
    let idx = ((q * (sorted.len() as f64 - 1.0)) + 0.5).floor() as usize;
    sorted[idx.min(sorted.len() - 1)]
}

/// Samples in milliseconds with p50/p95 nearest-rank.
struct Leg {
    samples_ms: Vec<f64>,
}

impl Leg {
    fn new() -> Self {
        Self {
            samples_ms: Vec::new(),
        }
    }

    fn time<F: FnMut()>(&mut self, iters: usize, mut f: F) {
        for _ in 0..iters {
            let t0 = Instant::now();
            f();
            self.samples_ms.push(t0.elapsed().as_secs_f64() * 1e3);
        }
        self.samples_ms
            .sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    }

    fn stats(&self) -> (f64, f64) {
        (
            quantile(&self.samples_ms, 0.50),
            quantile(&self.samples_ms, 0.95),
        )
    }
}

fn fmt_ms(ms: f64) -> String {
    if ms >= 1.0 {
        format!("{ms:.2} ms")
    } else if ms >= 0.001 {
        format!("{:.1} us", ms * 1e3)
    } else {
        format!("{:.1} ns", ms * 1e6)
    }
}

/// Builds a real Parquet file with `rows` rows (`v` = i, `g` = i % groups)
/// IN-ENGINE (CREATE TABLE AS SELECT over `range`, then COPY TO Parquet).
/// Setup cost — never inside a timed leg.
fn build_parquet(rows: usize, groups: usize, tag: &str) -> (TempParquet, String) {
    let path = std::env::temp_dir().join(format!(
        "graphlaw-sieve-cliff-{}-{}.parquet",
        std::process::id(),
        tag
    ));
    let path_str = path.to_str().expect("utf8 temp path").to_string();
    let db = GraphLawDuckDB::open_in_memory().expect("in-memory engine");
    db.raw_connection()
        .execute_batch(&format!(
            "CREATE TABLE raw AS SELECT i AS v, i % {groups} AS g FROM range({rows}) t(i); \
             COPY raw TO '{}' (FORMAT PARQUET);",
            path_str.replace('\'', "''")
        ))
        .expect("in-engine range seed + COPY TO parquet");
    (TempParquet(path), path_str)
}

/// GROUP BY reduction AST: one row per group (64 groups at every N).
fn reduce_turtle(view: &str) -> String {
    format!(
        r#"
@prefix sql: <http://graphlaw.org/sql#> .
@prefix ex: <http://example.org/sieve#> .
ex:reduce a sql:SelectStatement ;
    sql:statementIndex 1 ;
    sql:fromTable [ sql:tableName "{view}" ] ;
    sql:selectList (
        [ sql:columnName "g" ]
        [ sql:aliasName "n" ; sql:expression [
            sql:function <http://graphlaw.org/functions#COUNT> ;
            sql:argument [ sql:columnName "v" ] ] ]
    ) ;
    sql:groupBy ( [ sql:columnName "g" ] ) .
"#
    )
}

/// Unbounded projection AST: one output row per source row.
fn unbounded_turtle(view: &str) -> String {
    format!(
        r#"
@prefix sql: <http://graphlaw.org/sql#> .
@prefix ex: <http://example.org/sieve#> .
ex:unbounded a sql:SelectStatement ;
    sql:statementIndex 1 ;
    sql:fromTable [ sql:tableName "{view}" ] ;
    sql:selectList ( [ sql:columnName "g" ] ) .
"#
    )
}

fn compile(turtle: &str) -> CompiledQuery {
    compile_turtle(turtle).expect("SELECT AST compiles")
}

/// Raw control: pulls EVERY row of `SELECT g FROM {view}` through the raw
/// connection — the unsieved substrate the eval path is measured against.
fn raw_pull_all(db: &GraphLawDuckDB, view: &str) -> usize {
    let mut stmt = db
        .raw_connection()
        .prepare(&format!("SELECT g FROM {view}"))
        .expect("raw prepare");
    stmt.query_map([], |r| r.get::<_, i64>(0))
        .expect("raw query")
        .map(|r| r.expect("raw row"))
        .count()
}

/// One full five-leg cell at source size `n`.
fn cell(n: usize, reduce_iters: usize, raw_iters: usize, refuse_iters: usize) -> Vec<String> {
    let (parquet, path_str) = build_parquet(n, 64, &format!("n{n}"));
    let db = GraphLawDuckDB::open_in_memory().expect("in-memory engine");
    db.register_parquet_view("sieve_n", &path_str, false)
        .expect("parquet view mounts");

    let reduce = compile(&reduce_turtle("sieve_n"));
    let unbounded = compile(&unbounded_turtle("sieve_n"));

    let mut rows = Vec::new();

    // Leg 1 — reduce cold: first eval, fresh connection, C1 miss.
    let cold_iri = format!("urn:sieve:reduce-cold-{n}");
    let t0 = Instant::now();
    let receipt = db
        .eval_ast_query(&cold_iri, &reduce, SUBJECT, VOCAB)
        .expect("GROUP BY reduction lands inside the default envelope");
    let cold_ms = t0.elapsed().as_secs_f64() * 1e3;
    assert_eq!(receipt.rows_emitted, 64, "64 groups at every N");
    assert!(!receipt.cached, "cold eval is a miss, not a consult hit");
    rows.push(format!(
        "| {n} | reduce cold (1 shot) | 64 | {} | {} |",
        fmt_ms(cold_ms),
        fmt_ms(cold_ms)
    ));

    // Leg 2 — reduce warm-stmt: bump the C1 epoch (entry stale) so the eval
    // re-executes on the warm prepared statement.
    let warm_iri = format!("urn:sieve:reduce-warm-{n}");
    let mut lg = Leg::new();
    lg.time(reduce_iters, || {
        db.shared_cache().bump();
        let r = db
            .eval_ast_query(&warm_iri, &reduce, SUBJECT, VOCAB)
            .expect("warm-stmt reduction inside envelope");
        std::hint::black_box(r.rows_emitted);
    });
    let (p50, p95) = lg.stats();
    rows.push(format!(
        "| {n} | reduce warm-stmt (C1 miss, stmt warm) | 64 | {} | {} |",
        fmt_ms(p50),
        fmt_ms(p95)
    ));

    // Leg 3 — reduce warm-cache: the last warm-stmt iteration stored a fresh
    // entry; a plain re-eval consults without the engine.
    let cache_iri = format!("urn:sieve:reduce-cache-{n}");
    let mut lg = Leg::new();
    lg.time(reduce_iters, || {
        let r = db
            .eval_ast_query(&cache_iri, &reduce, SUBJECT, VOCAB)
            .expect("warm-cache consult");
        std::hint::black_box(&r);
    });
    let hit = db
        .eval_ast_query(&cache_iri, &reduce, SUBJECT, VOCAB)
        .expect("warm-cache consult");
    assert!(hit.cached, "warm-cache leg must be a consult hit");
    let (p50, p95) = lg.stats();
    rows.push(format!(
        "| {n} | reduce warm-cache (consult hit) | 64 | {} | {} |",
        fmt_ms(p50),
        fmt_ms(p95)
    ));

    // Leg 4 — refuse: unbounded eval refuses at 65/64, N-independent.
    let refuse_iri = format!("urn:sieve:refuse-{n}");
    let mut lg = Leg::new();
    lg.time(refuse_iters, || {
        let err = db
            .eval_ast_query(&refuse_iri, &unbounded, SUBJECT, VOCAB)
            .expect_err("unbounded result set must refuse");
        match err {
            EvaluatorError::RowEnvelopeExceeded {
                limit,
                observed,
                max,
            } => {
                assert_eq!(limit, ROW_ENVELOPE_LIMIT);
                assert_eq!(observed, DEFAULT_ROW_ENVELOPE + 1);
                assert_eq!(max, DEFAULT_ROW_ENVELOPE);
            }
            other => panic!("expected RowEnvelopeExceeded, got {other:?}"),
        }
    });
    let (p50, p95) = lg.stats();
    rows.push(format!(
        "| {n} | refuse (sieve at 65/64) | 65 | {} | {} |",
        fmt_ms(p50),
        fmt_ms(p95)
    ));

    // Leg 5 — raw control: the same unbounded statement, no sieve, all N rows.
    let total = raw_pull_all(&db, "sieve_n");
    assert_eq!(total, n, "raw control pulls every source row");
    let mut lg = Leg::new();
    lg.time(raw_iters, || {
        std::hint::black_box(raw_pull_all(&db, "sieve_n"));
    });
    let (p50, p95) = lg.stats();
    rows.push(format!(
        "| {n} | raw control (all rows, unsieved) | {n} | {} | {} |",
        fmt_ms(p50),
        fmt_ms(p95)
    ));

    drop(parquet);
    rows
}

fn main() {
    let quick = std::env::args().any(|a| a == "--quick");
    // (reduce iters, raw iters, refuse iters) per N, and the quick mode.
    let config: Vec<(usize, (usize, usize, usize))> = if quick {
        vec![
            (10_000, (10, 5, 50)),
            (100_000, (5, 3, 50)),
            (1_000_000, (2, 2, 50)),
        ]
    } else {
        vec![
            (10_000, (50, 20, 200)),
            (100_000, (20, 10, 200)),
            (1_000_000, (5, 5, 200)),
        ]
    };

    println!("# sieve_cliff run (WP P08: the result-row envelope vs raw)");
    println!(
        "# os={} arch={} quick={}",
        std::env::consts::OS,
        std::env::consts::ARCH,
        quick
    );
    println!("# reduce = in-engine GROUP BY to 64 rows; refuse = unbounded eval");
    println!("# refusing at observed 65 / max 64; raw control = the SAME unbounded");
    println!("# statement pulled through raw_connection with no sieve.");

    println!("\n== sieve cliff: N x leg (ms, nearest-rank p50/p95) ==");
    println!("| N | leg | rows | p50 | p95 |");
    println!("|---|---|---|---|---|");
    for (n, (reduce_iters, raw_iters, refuse_iters)) in config {
        for row in cell(n, reduce_iters, raw_iters, refuse_iters) {
            println!("{row}");
        }
    }

    println!("\n# The cliff: refuse and reduce rows stay flat at the envelope while");
    println!("# raw control scales with N — the sieve caps materialization, not the engine.");
}
