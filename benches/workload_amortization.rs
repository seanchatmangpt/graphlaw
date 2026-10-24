//! Workload-mix amortization benchmarks (lane: benches/workload_amortization.rs).
//!
//! Converts the measured cold/hot asymmetry (cold graph-backed eval ~238.5 ms
//! median vs C1 prepared-reuse ~2.4 us vs raw engine prepared ~123 us,
//! benches/sql_rdf_engine.rs) into a measured amortization curve over a
//! bounded Zipf workload mix.
//!
//! Design (Chicago: real frozen-duckdb, real receipts, no sleeps, no mocks):
//! - 16 real template shapes (8 seeded tables x {id,name} projection), Zipf
//!   rank skew (weights ~ 1/(rank+1.375)).
//! - "Instances" are distinct statement IRIs over the same SQL shape: the C1
//!   cache key is BLAKE3(Canon(G_AST) || P_params), so a distinct IRI forks
//!   the cache key while the lowered SQL (and engine plan) stays identical.
//! - A cold fraction f is realized as M = clamp(round(f*N), 1, N) first-touch
//!   queries (guaranteed cache misses) followed by replay of earlier
//!   instances (guaranteed hits). Achieved cold fraction = M/N, reported
//!   against target (schedule granularity tolerance 1/N).
//! - All CompiledQuery values are compiled and all miss entries warm-filled
//!   BEFORE the timing loop: the timed region contains eval_ast_query only
//!   (zero generator overhead in timing).
//! - In-loop non-vacuity: every receipt must show the real 2-row readback
//!   (rows_emitted == 2) and receipt.cached equal to the scheduled class.
//!
//! ENGINE_LOCK discipline: taken once at process start via a leaked guard,
//! per benches/sql_rdf_engine.rs (the frozen engine segfaults under
//! concurrent engine-heavy threads; tests/duckdb_statement_kinds.rs:11).

use std::hint::black_box;
use std::sync::Mutex;

use graphlaw::duckdb_eval::{ExecutionReceipt, GraphLawDuckDB};
use graphlaw::sql_ast::{CompiledQuery, compile_turtle};

static ENGINE_LOCK: Mutex<()> = Mutex::new(());

const N_QUERIES: usize = 10_000;
const N_TEMPLATES: usize = 16; // 8 tables x 2 projected columns

fn table_name(k: usize) -> String {
    format!("t{k}")
}

fn template_shape(tid: usize) -> (usize, &'static str) {
    (tid / 2, if tid % 2 == 0 { "id" } else { "name" })
}

/// Turtle source for one instance: (template, salt). The salt lives in the
/// statement IRI, so each salt is a distinct admitted graph (distinct C1
/// cache key) over identical lowered SQL.
fn turtle_for(tid: usize, salt: usize) -> String {
    let (table, column) = template_shape(tid);
    let iri = format!("http://example.org/bench/inst/{salt}");
    format!(
        concat!(
            "@prefix sql: <http://graphlaw.org/sql#> .\n",
            "@prefix xsd: <http://www.w3.org/2011/XMLSchema#> .\n",
            "<{iri}> a sql:SelectStatement ;\n",
            "    sql:statementIndex 1 ;\n",
            "    sql:select ( [ a sql:ColumnIdentifier ; sql:columnName \"{col}\" ; sql:scope sql:ViewScope ] ) ;\n",
            "    sql:from ( [ a sql:TableIdentifier ; sql:tableName \"{tbl}\" ] ) .\n",
        ),
        iri = iri,
        col = column,
        tbl = table_name(table),
    )
}

/// Fixed-seed xorshift64* — deterministic, no time or entropy in the
/// workload generator (the same schedule on every run).
struct Rng(u64);
impl Rng {
    fn unit(&mut self) -> f64 {
        let mut x = self.0;
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        self.0 = x;
        let v = x.wrapping_mul(0x2545_F491_4F6C_DD1D);
        (v >> 11) as f64 / (1u64 << 53) as f64
    }
}

/// Precomputed Zipf CDF over N_TEMPLATES templates, weights ~ 1/(rank+1.375).
fn zipf_cdf() -> Vec<f64> {
    let weights: Vec<f64> = (0..N_TEMPLATES).map(|i| 1.0 / (i as f64 + 1.375)).collect();
    let total: f64 = weights.iter().sum();
    let mut cdf = Vec::with_capacity(N_TEMPLATES);
    let mut acc = 0.0;
    for w in &weights {
        acc += w / total;
        cdf.push(acc);
    }
    let last = cdf.len() - 1;
    cdf[last] = 1.0;
    cdf
}

fn zipf_template(u: f64, cdf: &[f64]) -> usize {
    cdf.partition_point(|&c| c < u).min(N_TEMPLATES - 1)
}

/// One scheduled query: pre-compiled before timing, with its expected cache
/// class for in-loop non-vacuity.
struct Scheduled {
    compiled: CompiledQuery,
    expect_hit: bool,
    salt: usize,
    tid: usize,
}

/// Builds the deterministic pre-compiled schedule for cold fraction `f`.
///
/// Fill-then-evict: warm-fill the COLD (miss) entries first so their compile
/// cost lands outside the timed region, then `QueryCache::bump()` evicts
/// exactly them (epoch invalidation), so every scheduled miss is a genuine
/// first touch inside `run_mix`. HOT entries are warm-filled AFTER the bump,
/// so their first replay is a genuine C1 hit.
fn build_schedule(db: &GraphLawDuckDB, f: f64, cdf: &[f64]) -> (Vec<Scheduled>, usize) {
    let m = ((f * N_QUERIES as f64).round() as usize).clamp(1, N_QUERIES);
    let mut rng = Rng(0x9E37_79B9_7F4A_7C15);
    let mut out: Vec<Scheduled> = Vec::with_capacity(N_QUERIES);
    let mut hot_pending: Vec<(usize, usize, usize)> = Vec::new(); // (q, tid, salt)
    // (tid, salt) of every COLD slot: hot replays sharing one of these keys
    // must NOT be warm-filled in pass 2, or the cold first-touch in run_mix
    // would arrive as a hit. Those keys are (re)populated by the cold miss
    // itself, which runs before any replay (q ascending).
    let mut cold_keys: std::collections::HashSet<(usize, usize)> = Default::default();
    // Pass 1: compile the whole schedule; warm-fill COLD entries only.
    for q in 0..N_QUERIES {
        let tid = zipf_template(rng.unit(), cdf);
        let salt = if q < m { q } else { q % m };
        let expect_hit = q >= m;
        let src = turtle_for(tid, salt);
        let compiled = compile_turtle(&src).expect("instance graph must compile");
        if !expect_hit {
            // Warm-fill the C1 entry ONCE, outside the timed region.
            let w = db
                .eval_ast_query("urn:bench:warm", &compiled, "urn:r:", "urn:p:")
                .expect("warm-up eval must succeed");
            assert!(!w.cached, "warm-up fill is a miss");
            assert_eq!(w.rows_emitted, 2, "warm-up readback");
            cold_keys.insert((tid, salt));
        } else {
            hot_pending.push((q, tid, salt));
        }
        out.push(Scheduled { compiled, expect_hit, salt, tid });
    }
    // Evict the cold entries so the timed first touch is a genuine miss.
    let epoch = db.shared_cache().bump();
    assert!(epoch >= 1, "bump must advance the epoch");
    // Pass 2: warm-fill HOT entries at the new epoch so their first replay
    // in run_mix is a genuine hit. Keys shared with a cold slot are skipped:
    // the cold miss (scheduled earlier) repopulates them before the replay.
    let mut hot_seen: std::collections::HashSet<(usize, usize)> = Default::default();
    for &(q, tid, salt) in &hot_pending {
        if cold_keys.contains(&(tid, salt)) {
            continue;
        }
        let w = db
            .eval_ast_query("urn:bench:warm", &out[q].compiled, "urn:r:", "urn:p:")
            .expect("hot warm-up eval must succeed");
        // Two hot slots can share a key (same tid, same salt): its first
        // touch after the bump is a miss; every later touch must hit.
        if hot_seen.insert((tid, salt)) {
            assert!(!w.cached, "first hot warm-up of a key is the fill (miss)");
        } else {
            assert!(w.cached, "repeat hot warm-up must be a hit");
        }
        assert_eq!(w.rows_emitted, 2, "hot warm-up readback");
    }
    (out, m)
}

/// Runs the mix against a real engine, asserting non-vacuity per query.
/// Returns (hits, misses).
fn run_mix(db: &GraphLawDuckDB, schedule: &[Scheduled]) -> (usize, usize) {
    let mut hits = 0usize;
    let mut misses = 0usize;
    for s in schedule {
        let receipt: ExecutionReceipt = db
            .eval_ast_query(
                "urn:bench:mix",
                black_box(&s.compiled),
                "urn:r:",
                "urn:p:",
            )
            .expect("mix eval must succeed");
        // In-loop non-vacuity: real readback + scheduled cache class.
        assert_eq!(receipt.rows_emitted, 2, "real 2-row readback must hold");
        assert_eq!(
            receipt.cached, s.expect_hit,
            "cache class must match the schedule (salt {} tid {})",
            s.salt, s.tid
        );
        if receipt.cached {
            hits += 1;
        } else {
            misses += 1;
        }
    }
    (hits, misses)
}

/// Timed-loop variant for the divan leg: bench_local runs the closure many
/// iterations, and after the first, cold entries are legitimately populated
/// (the previous iteration's misses). The per-schedule cache-class court
/// therefore runs exactly once, in payload_sweep's single pass via run_mix;
/// this variant keeps the real-readback non-vacuity on every iteration.
fn run_mix_timed(db: &GraphLawDuckDB, schedule: &[Scheduled]) -> (usize, usize) {
    let mut hits = 0usize;
    let mut misses = 0usize;
    for s in schedule {
        let receipt: ExecutionReceipt = db
            .eval_ast_query(
                "urn:bench:mix",
                black_box(&s.compiled),
                "urn:r:",
                "urn:p:",
            )
            .expect("mix eval must succeed");
        assert_eq!(receipt.rows_emitted, 2, "real 2-row readback must hold");
        if receipt.cached {
            hits += 1;
        } else {
            misses += 1;
        }
    }
    (hits, misses)
}

fn open_seeded_8() -> GraphLawDuckDB {
    let db = GraphLawDuckDB::open_in_memory().expect("in-memory engine must open");
    for k in 0..8 {
        db.raw_connection()
            .execute_batch(&format!(
                "CREATE TABLE t{k}(id INTEGER, name VARCHAR); \
                 INSERT INTO t{k} VALUES ({}, 'a'), ({}, 'b');",
                k * 10 + 1,
                k * 10 + 2
            ))
            .expect("seed must succeed");
    }
    db
}

/// Payload sweep: one wall-clock pass per fraction on a fresh engine each
/// time; writes target/amortization-payload.json (never receipts/ — that dir
/// is chain-gated; a stray unchained write there is a tamper event).
fn payload_sweep() {
    let cdf = zipf_cdf();
    let mut rows: Vec<String> = Vec::new();
    for &f in &[1.0f64, 1e-2, 1e-3, 1e-4, 1e-5] {
        let db = open_seeded_8();
        let init0 = std::time::Instant::now();
        let (schedule, m) = build_schedule(&db, f, &cdf);
        let init_ms = init0.elapsed().as_secs_f64() * 1000.0;
        let t0 = std::time::Instant::now();
        let (hits, misses) = run_mix(&db, &schedule);
        let ms = t0.elapsed().as_secs_f64() * 1000.0;
        let per_q_us = ms * 1000.0 / N_QUERIES as f64;
        assert_eq!(hits + misses, N_QUERIES);
        assert_eq!(misses, m, "achieved miss count must equal the schedule");
        rows.push(format!(
            "    {{\"cold_fraction_target\": {:.6}, \"cold_fraction_achieved\": {:.6}, \"n_queries\": {}, \"hits\": {}, \"misses\": {}, \"total_ms\": {:.3}, \"amortized_us_per_query\": {:.3}, \"init_ms\": {:.1}}}",
            f,
            m as f64 / N_QUERIES as f64,
            N_QUERIES,
            hits,
            misses,
            ms,
            per_q_us,
            init_ms
        ));
    }
    let json = format!(
        "{{\n  \"workload\": \"zipf16\", \"n_queries\": {}, \"n_templates\": {},\n  \"fractions\": [\n{}\n  ]\n}}\n",
        N_QUERIES,
        N_TEMPLATES,
        rows.join(",\n")
    );
    let path = std::path::Path::new(env!("CARGO_TARGET_TMPDIR")).join("amortization-payload.json");
    std::fs::write(&path, json).expect("payload write must succeed");
    println!("PAYLOAD: {}", path.display());
}

/// Boot-tax leg: compile+admission of the full 16-template set on a fresh
/// engine; hot floor measured on the same engine; break-even derived.
fn boot_tax() {
    let db = open_seeded_8();
    let mut srcs = Vec::with_capacity(N_TEMPLATES);
    for i in 0..N_TEMPLATES {
        srcs.push(turtle_for(i, 50_000 + i));
    }
    let t0 = std::time::Instant::now();
    let mut compiled_templates = Vec::with_capacity(N_TEMPLATES);
    for src in &srcs {
        compiled_templates.push(compile_turtle(src).expect("template must compile"));
    }
    let boot_ms = t0.elapsed().as_secs_f64() * 1000.0;
    assert_eq!(compiled_templates.len(), N_TEMPLATES);
    // Admit them for real: one eval each (misses by construction).
    for c in compiled_templates.iter() {
        let r = db
            .eval_ast_query("urn:bench:boot", c, "urn:r:", "urn:p:")
            .expect("boot admission eval must succeed");
        assert_eq!(r.rows_emitted, 2);
    }

    // Hot floor: mean per-query us over 1000 real hit evals of template 0.
    let hot = &compiled_templates[0];
    db.eval_ast_query("urn:bench:hotwarm", hot, "urn:r:", "urn:p:")
        .expect("hot warm-up must succeed");
    let n_hot = 1000usize;
    let mut hot_hits = 0usize;
    let t1 = std::time::Instant::now();
    for _ in 0..n_hot {
        let r = db
            .eval_ast_query("urn:bench:hot", hot, "urn:r:", "urn:p:")
            .expect("hot eval must succeed");
        assert!(r.cached, "hot floor must be C1 hits");
        assert_eq!(r.rows_emitted, 2);
        hot_hits += 1;
    }
    let hot_us = t1.elapsed().as_secs_f64() * 1e6 / n_hot as f64;
    let break_even = boot_ms * 1000.0 / hot_us;
    println!(
        "BOOT-TAX: total_ms={:.3} ms_per_shape={:.4} hot_floor_us={:.2} break_even_queries={:.0} hot_hits={}",
        boot_ms,
        boot_ms / N_TEMPLATES as f64,
        hot_us,
        break_even,
        hot_hits
    );
}

fn main() {
    // Take the engine lock ONCE for the whole bench process (leaked guard),
    // per benches/sql_rdf_engine.rs.
    std::mem::forget(ENGINE_LOCK.lock().unwrap_or_else(|p| p.into_inner()));

    payload_sweep();
    boot_tax();

    divan::Divan::from_args().main();
}

// Parametric sweep: one bench per cold fraction; setup (compile + warm-fill)
// happens per invocation outside bencher.bench_local.
#[divan::bench(args = [1.0, 1e-2, 1e-3, 1e-4, 1e-5])]
fn zipf_mix_amortization(bencher: divan::Bencher, f: f64) {
    let db = open_seeded_8();
    let cdf = zipf_cdf();
    let (schedule, _m) = build_schedule(&db, f, &cdf);
    bencher.bench_local(|| {
        let (hits, misses) = run_mix_timed(&db, &schedule);
        black_box((hits, misses))
    });
}

// Boot-tax leg as a bench arg too: compile+admission of the template set.
#[divan::bench]
fn boot_tax_admission() {
    let _db = open_seeded_8();
    let mut srcs = Vec::with_capacity(N_TEMPLATES);
    for i in 0..N_TEMPLATES {
        srcs.push(turtle_for(i, 70_000 + i));
    }
    let mut compiled = Vec::with_capacity(N_TEMPLATES);
    for src in &srcs {
        compiled.push(compile_turtle(src).expect("template must compile"));
    }
    let _ = black_box(compiled.len());
}
