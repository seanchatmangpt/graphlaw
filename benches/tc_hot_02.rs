//! TC-HOT-02 — duckdb-generated Parquet corpus benchmark (SMOKE: 500k rows;
//! full 25M corpus exists behind `TC_HOT_FULL=1` but is NOT run in this lane).
//!
//! Real frozen-duckdb `COPY (...) TO parquet` corpus, real
//! `register_parquet_view`, real compiled SELECT (WHERE + GROUP BY) timed
//! through `GraphLawDuckDB::eval_ast_query` over 5 repetitions, with a
//! raw-connection path delta and honesty lines (version, threads, warm
//! cache). Asserts on real table readbacks. No mocks, no sleeps.

use std::fs;
use std::path::PathBuf;
use std::time::Instant;

use hdrhistogram::Histogram;

use graphlaw::duckdb_eval::GraphLawDuckDB;
use graphlaw::sql_ast::compile_turtle;

const REPS: u64 = 5;

/// Variance-stability self-check (charter invariant: run-to-run variance
/// < 5%): 3 consecutive batches of REPS timed evals each, per-batch p50,
/// relative spread must stay under 5% or the verdict is VARIANCE-EXCEEDED.
const BATCHES: u64 = 3;
const VARIANCE_BUDGET_PCT: f64 = 5.0;

/// Measurement-isolation disclosure (honest macOS constraints, stable Rust):
/// thread affinity needs the third-party `core_affinity` crate (not added);
/// priority elevation (`nice -n -10`) requires privileges and is not applied;
/// CPU PM state is disclosed live via `pmset -g therm`. Isolation actually
/// applied here: duckdb `SET threads` pinning, per-rep unique scratch dirs
/// purged between reps, untimed warm-up pass, min-of-REPS reporting.
const ISOLATION_DISCLOSURE: &str = "thread_affinity=unavailable_stable_rust_macos (core_affinity crate not added)\npriority_elevation=requires_privileges_macos (not applied)\npmset_therm_disclosure=live_above\nscratch_purge=per-rep unique dirs (temp_directory rotated + removed between reps)";

fn machine_disclosure() -> String {
    let arch = std::env::consts::ARCH;
    let cores = std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(0);
    let brand = std::process::Command::new("sysctl")
        .args(["-n", "machdep.cpu.brand_string"])
        .output()
        .ok()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_else(|| "unknown".to_string());
    let therm = std::process::Command::new("pmset")
        .args(["-g", "therm"])
        .output()
        .ok()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_else(|| "unknown".to_string());
    format!("ARCH={arch}\ncore_count={cores}\ncpu={brand}\nthermal={therm}")
}

/// SELECT batch_id, SUM(cost) AS total_cost FROM <view>
/// WHERE (tenant_id = $1) AND (latency_ms > $2) GROUP BY batch_id
///
/// Graph-backed compile (SQLRDF-17): the statement is admitted from a real
/// RDF AST graph via compile_turtle, so eval_ast_query receives the pinned
/// h_ast receipt anchor + Canon(G_AST) bytes. A hand-built SelectStatement
/// (no graph) is a typed NoCanonicalGraph refusal by design.
fn compiled_select_turtle(view: &str) -> String {
    format!(
        r#"@prefix sql: <http://graphlaw.org/sql#> .
@prefix xsd: <http://www.w3.org/2001/XMLSchema#> .
<http://example.org/tc02> a sql:SelectStatement ;
    sql:statementIndex 1 ;
    sql:fromTable [ sql:tableName "{view}" ] ;
    sql:selectList (
        [ sql:columnName "batch_id" ]
        [ sql:aliasName "total_cost" ; sql:expression [
            sql:functionName "SUM" ; sql:arguments ( [ sql:columnName "cost" ] ) ] ]
    ) ;
    sql:where [
        sql:operator "AND" ;
        sql:leftOperand [
            sql:operator "=" ;
            sql:leftOperand [ sql:columnName "tenant_id" ] ;
            sql:rightOperand [ sql:literalValue "tenant-A" ]
        ] ;
        sql:rightOperand [
            sql:operator ">" ;
            sql:leftOperand [ sql:columnName "latency_ms" ] ;
            sql:rightOperand [ sql:literalValue "100"^^xsd:integer ]
        ]
    ] ;
    sql:groupBy ( [ sql:columnName "batch_id" ] ) ."#
    )
}

fn main() {
    let full = std::env::var("TC_HOT_FULL").is_ok_and(|v| v == "1");
    let rows: i64 = if full { 25_000_000 } else { 500_000 };

    let dir: PathBuf = std::env::temp_dir().join(format!("tc_hot_02_{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).expect("temp corpus dir");

    let db = GraphLawDuckDB::open_in_memory().expect("duckdb open");

    // 1. duckdb-generated Parquet corpus, hive-partitioned by tenant_id.
    //    Every row is a REAL materialized parquet row, not a fixture stub.
    let setup_sql = format!(
        "COPY (
            SELECT
                CASE WHEN i % 2 = 0 THEN 'tenant-A' ELSE 'tenant-B' END AS tenant_id,
                'batch-' || (i % 64) AS batch_id,
                (i * 37) % 1000 AS cost,
                (i * 53) % 1200 AS latency_ms
            FROM range({rows}) AS t(i)
        ) TO '{d}/telemetry' (FORMAT PARQUET, PARTITION_BY (tenant_id));",
        d = dir.display()
    );
    let t_setup = Instant::now();
    db.raw_connection()
        .execute_batch(&setup_sql)
        .expect("corpus generation");
    let setup_elapsed = t_setup.elapsed();

    // 2. Register the hive-partitioned view.
    let glob = format!("{}/telemetry/**/*.parquet", dir.display());
    db.register_parquet_view("telemetry_events", &glob, true)
        .expect("register_parquet_view");

    // 3. Honesty lines.
    let version: String = db
        .raw_connection()
        .query_row("SELECT version()", [], |r| r.get(0))
        .unwrap_or_else(|_| "unknown".to_string());
    let threads: i64 = db
        .raw_connection()
        .query_row("SELECT current_setting('threads')", [], |r| r.get(0))
        .unwrap_or(0);

    let turtle = compiled_select_turtle("telemetry_events");
    let compiled = compile_turtle(&turtle)
        .map_err(|e| format!("compile_turtle refused: {e:?}"))
        .expect("graph-backed compile");
    println!("=== TC-HOT-02 parquet corpus compiled-query benchmark ===");
    println!("mode={}", if full { "FULL" } else { "SMOKE" });
    println!("{}", machine_disclosure());
    println!("duckdb={version}");
    println!("threads={threads}");
    println!("rows={rows} corpus_ms={:.1}", setup_elapsed.as_secs_f64() * 1000.0);
    println!("warm_cache=true (corpus generated and COPY-flushed before timing)");
    println!("sql={}", compiled.sql);
    println!("{ISOLATION_DISCLOSURE}");

    // Warm-up pass (not timed): page cache + plan cache.
    {
        let warm = db
            .eval_ast_query("urn:tc-hot-02:warm", &compiled, "urn:gl:s:", "urn:gl:p:")
            .expect("warm-up eval");
        println!("warmup rows_emitted={}", warm.rows_emitted);
    }

    // 4. Timed eval_ast_query reps -> HDR histogram. Measurement isolation:
    //    each rep gets a UNIQUE per-rep scratch dir as duckdb's
    //    temp_directory, purged immediately after the rep (scratch-dir purge
    //    between iterations; no cross-rep scratch reuse).
    let total_evals = BATCHES * REPS;
    let mut hist = Histogram::<u64>::new(3).expect("hdrhistogram init");
    let mut batch_stats: Vec<u64> = Vec::with_capacity(BATCHES as usize);
    let mut last_rows = 0u64;
    let mut last_hash = [0u8; 32];
    let mut eval_total = std::time::Duration::ZERO;
    let mut eval_index = 0u64;
    for batch in 0..BATCHES {
        let mut batch_hist = Histogram::<u64>::new(3).expect("hdrhistogram init");
        for _ in 0..REPS {
            let scratch = dir.join(format!("scratch_b{batch}_e{eval_index}"));
            let _ = fs::remove_dir_all(&scratch);
            fs::create_dir_all(&scratch).expect("per-rep scratch dir");
            db.raw_connection()
                .execute_batch(&format!(
                    "SET temp_directory='{}';",
                    scratch.display()
                ))
                .expect("set per-rep temp_directory");
            let t0 = Instant::now();
            let receipt = db
                .eval_ast_query(
                    &format!("urn:tc-hot-02:rep{eval_index}"),
                    &compiled,
                    "urn:gl:s:",
                    "urn:gl:p:",
                )
                .expect("eval_ast_query");
            let el = t0.elapsed();
            eval_total += el;
            last_rows = receipt.rows_emitted as u64;
            last_hash = receipt.query_hash;
            let us = el.as_micros() as u64;
            hist.record(us).expect("record");
            batch_hist.record(us).expect("record");
            eval_index += 1;
            // Scratch purge between iterations.
            let _ = fs::remove_dir_all(&scratch);
        }
        batch_stats.push(batch_hist.value_at_quantile(0.50));
    }

    let p50 = hist.value_at_quantile(0.50);
    let p95 = hist.value_at_quantile(0.95);
    let p99 = hist.value_at_quantile(0.99);
    let mean_us = eval_total.as_micros() as f64 / total_evals as f64;
    let group_rows_per_sec = (last_rows as f64 * total_evals as f64) / eval_total.as_secs_f64();
    let scanned_rows_per_sec = (rows as f64 * total_evals as f64) / eval_total.as_secs_f64();
    println!(
        "RESULT eval_ast_query_us: p50={p50} p95={p95} p99={p99} mean={mean_us:.1} reps={total_evals} (batches={BATCHES} x REPS={REPS})"
    );
    println!(
        "group_rows_emitted={last_rows} group_rows_per_sec={group_rows_per_sec:.0}",
    );
    println!("scanned_rows_per_sec={scanned_rows_per_sec:.0} (scan input = {rows} rows/query)");
    println!(
        "query_hash={:02x?}",
        last_hash
    );

    // Variance-stability self-check (charter invariant < 5%): PASS /
    // VARIANCE-EXCEEDED, emitted honestly either way.
    let batch_mean =
        batch_stats.iter().map(|v| *v as f64).sum::<f64>() / BATCHES as f64;
    let spread = (batch_stats.iter().max().unwrap() - batch_stats.iter().min().unwrap()) as f64;
    let spread_pct = if batch_mean > 0.0 { spread / batch_mean * 100.0 } else { f64::INFINITY };
    let variance_ok = spread_pct < VARIANCE_BUDGET_PCT;
    println!(
        "VARIANCE_SELF_CHECK batches={BATCHES} reps_per_batch={REPS} batch_p50s_us={:?} spread_pct={spread_pct:.2}% budget={VARIANCE_BUDGET_PCT}% : {}",
        batch_stats,
        if variance_ok { "PASS" } else { "VARIANCE-EXCEEDED" }
    );

    // 5. Raw prepared-path delta: the same statement shape straight through
    //    the raw frozen-duckdb connection (no triple projection, no receipt),
    //    PREPARED ONCE and re-executed with typed binds — the raw floor the
    //    compiled path must be compared against, not a per-rep re-parse.
    let raw_sql = format!(
        "SELECT COUNT(*) FROM ({}) AS raw_delta_subq",
        compiled.sql
    );
    let mut raw_stmt = db
        .raw_connection()
        .prepare(&raw_sql)
        .expect("prepare raw floor statement");
    let mut raw_hist = Histogram::<u64>::new(3).expect("hdrhistogram init");
    let mut raw_rows = 0u64;
    let mut raw_min_us = u64::MAX;
    for _ in 0..REPS {
        let t0 = Instant::now();
        let n: i64 = raw_stmt
            .query_row(frozen_duckdb::params!["tenant-A", 100i32], |r| r.get(0))
            .expect("raw count");
        let us = t0.elapsed().as_micros() as u64;
        raw_min_us = raw_min_us.min(us);
        raw_rows = n as u64;
        raw_hist.record(us).expect("record");
    }
    let raw_p95 = raw_hist.value_at_quantile(0.95);
    let raw_p50 = raw_hist.value_at_quantile(0.50);

    // 6. PCE per the Measure-lane charter: PCE = value-add engine time ÷
    //    total pipeline time, taken MIN-of-5 on both paths (clean-repetition
    //    discipline: earlier smoke measured -2.2%..+4.3% under contention;
    //    min is the closest observable to uncontended machine time).
    //      value-add (engine) time = raw prepared-path min
    //      total pipeline time     = compiled eval_ast_query min
    //      substrate overhead      = (total - value-add) / value-add.
    let compiled_min_us = hist.min();
    let overhead_pct =
        ((compiled_min_us as f64 - raw_min_us as f64) / raw_min_us as f64) * 100.0;
    let pce_pct = (raw_min_us as f64 / compiled_min_us as f64) * 100.0;
    println!(
        "RAW_PREPARED_PATH us: p50={raw_p50} p95={raw_p95} min={raw_min_us} rows={raw_rows}"
    );
    println!(
        "PCE min_of_{REPS}: value_add_engine_us={raw_min_us} total_pipeline_us={compiled_min_us} pce={pce_pct:.2}% substrate_overhead={overhead_pct:.2}%"
    );
    println!(
        "HONESTY: warm_cache=true (corpus COPY-flushed + warm-up eval before timing); min-of-{REPS} filtering, mean-under-contention not reported as PCE; same-process serial paths (no cross-lane load assumed absent); compiled path includes receipt-hash + triple projection + row materialization, raw prepared path includes none of these; single query shape (WHERE + GROUP BY), not a portfolio average"
    );

    // 7. Readback gate: the compiled path must actually read back rows and
    //    both paths must agree on the row count (same SQL).
    if last_rows == 0 || last_rows != raw_rows {
        eprintln!(
            "TC-HOT-02 GATE FAIL: readback mismatch last_rows={last_rows} raw_rows={raw_rows}"
        );
        let _ = fs::remove_dir_all(&dir);
        std::process::exit(1);
    }
    println!(
        "GATE readback rows agree ({} = {}) : PASS",
        last_rows, raw_rows
    );
    println!(
        "GATE variance < {VARIANCE_BUDGET_PCT}% (spread_pct={spread_pct:.2}%) : {}",
        if variance_ok { "PASS" } else { "FAIL" }
    );
    if !variance_ok {
        let _ = fs::remove_dir_all(&dir);
        std::process::exit(1);
    }

    let _ = fs::remove_dir_all(&dir);
    println!("cleanup corpus dir: removed");
}
