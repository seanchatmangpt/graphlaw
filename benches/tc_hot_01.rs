//! TC-HOT-01 — Receipt hot path benchmark: re-absorb baseline vs clone path
//! (SMOKE: 1M iterations each; full 10M behind `TC_HOT_FULL=1`).
//!
//! Times BOTH receipt legs per iteration:
//! 1. BASELINE: full re-absorb `admitted_statement_hash(canon, params)`
//!    (BLAKE3(Canon(G_AST) || P_params), sql-rdf Rec. section 1.4).
//! 2. ANCHOR PATH (P28-SIL §3.1, TC-HOT-01 two-node): fresh hasher over
//!    `CompiledQuery.receipt_state` (= h_ast = BLAKE3(Canon(G_AST)), a 32B
//!    Copy) + `hash_param_buffer` P_params fold + finalize — the canon half
//!    is compressed to one node at compile time, so a receipt costs one
//!    Hasher::new + two updates + finalize.
//!
//! Also courts the refusal invariant leg: `Refusal` construction + `Display`
//! (src/dialect.rs refusal surface), asserting the rendered refusal carries
//! its class and message. Plus p50/p95/p99/p99.9 HDR quantiles and a machine
//! disclosure block. Chicago: real hashes over real canonical bytes; no
//! mocks, no sleeps.

use std::hint::black_box;
use std::time::Instant;

use hdrhistogram::Histogram;

use graphlaw::duckdb_eval::{
    admitted_statement_hash, canonical_ast_graph_bytes, fold_params_anchored, hash_param_buffer,
    ParamBytes,
};
use graphlaw::sql_ast::{compile_turtle, SqlLiteral};

/// Per-fold latency budget (nanoseconds), p99, on the incremental clone path.
const BUDGET_NS: u64 = 250;

/// Charter A03 param-integration budget (nanoseconds), p99, per leg:
/// `ParamBytes::encode` + single `Hasher::update` fold, both the small
/// (<64B inline) and large (>64B digest/fallback) vectors.
const PARAM_BUDGET_NS: u64 = 40;

/// PRD refusal-construction budget: 0.10 µs (100 ns) per Refusal build+render.
/// The current Refusal carries a heap `String` message (src/dialect.rs), so
/// this leg is expected to FAIL honestly until the `RefusalMessage::Static`
/// fast path (designs/P28-SIL-specs.md section 3.1) is implemented
/// (post-integration item).
const REFUSAL_BUDGET_NS: u64 = 100;

/// Measurement-isolation configuration disclosure (charter invariant:
/// repeatable baselines, run-to-run variance < 5%).
///
/// Honest macOS constraints, stable Rust:
/// - Thread affinity: stable Rust has no portable thread-pinning API and the
///   `core_affinity` crate is third-party (not added to keep the dep tree
///   unchanged); isolation is via duckdb `SET threads` (TC-HOT-02) and
///   single-process serial legs here instead.
/// - Priority: `setpriority`/`nice` requires privileges on macOS; not
///   implemented. Run under `sudo nice -n -10` manually if a lower-variance
///   baseline is needed.
/// - CPU PM state: disclosed live via `pmset -g therm` below (thermal
///   pressure is reported, not forced).
fn isolation_disclosure() -> String {
    let nice = std::process::Command::new("nice")
        .args(["-n", "0", "true"])
        .status()
        .map(|s| s.success())
        .unwrap_or(false);
    format!(
        "thread_affinity=unavailable_stable_rust_macos (core_affinity crate not added; dev-dep would be required)\npriority_elevation=requires_privileges_macos (setpriority/nice not applied)\npmset_therm_disclosure=live_below\nnice_probe={nice}"
    )
}

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

/// Real mixed typed parameter buffer P_params.
fn params() -> Vec<SqlLiteral> {
    vec![
        SqlLiteral::String("batch-42".to_string()),
        SqlLiteral::Integer(-987_654_321),
        SqlLiteral::Float(std::f64::consts::PI),
        SqlLiteral::Boolean(true),
        SqlLiteral::Null,
    ]
}

/// Real graph-backed compile: gives BOTH the Canon(G_AST) bytes (re-absorb
/// baseline) and the pinned `h_ast` anchor in `receipt_state` (anchor path).
fn graph_backed_compile() -> (Vec<u8>, [u8; 32]) {
    // Real admitted AST graph (the same shape the P28-SIL §3.1 property
    // court compiles), canonicalized for real via RDFC-1.0.
    let turtle = "@prefix : <http://example.org/> .\n@prefix sql: <http://graphlaw.org/sql#> .\n:g a sql:SelectStatement ;\n    sql:statementIndex 1 ;\n    sql:fromTable [ sql:tableName \"t\" ] ;\n    sql:selectList ( [ sql:columnName \"v\" ] ) .";
    let compiled = match compile_turtle(turtle) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("TC-HOT-01 REFUSED: compile_turtle failure: {e:?}");
            std::process::exit(2);
        }
    };
    let canon = match &compiled.canon_graph_bytes {
        Some(bytes) => bytes.clone(),
        None => match canonical_ast_graph_bytes(turtle) {
            Ok(bytes) => bytes,
            Err(e) => {
                eprintln!("TC-HOT-01 REFUSED: canonicalization failure: {e:?}");
                std::process::exit(2);
            }
        },
    };
    let state = match compiled.receipt_state {
        Some(h_ast) => h_ast,
        None => {
            // Refuse loudly: without the pinned h_ast anchor there is no
            // anchor path to witness.
            eprintln!("TC-HOT-01 REFUSED: compile_turtle did not pin the h_ast receipt_state anchor");
            std::process::exit(2);
        }
    };
    (canon, state)
}

fn main() {
    let full = std::env::var("TC_HOT_FULL").is_ok_and(|v| v == "1");
    let iters: u64 = if full { 10_000_000 } else { 1_000_000 };
    // SPC gate hooks (scripts/spc_court.sh):
    // - TC_HOT_SPC_EXPORT=<path>: write a machine-readable summary of the
    //   clone-path sample (count/mean/stddev/quantiles) as JSON.
    // - TC_HOT_INJECT_OUTLIER_NS=<ns>: ANTI-VACUITY fault injection — a real
    //   busy-spin (no sleeps) of this many ns precedes each recorded
    //   clone-path sample, so the gate must observe the injected latency and
    //   flip its verdict to FAIL.
    let spc_export = std::env::var("TC_HOT_SPC_EXPORT").ok();
    let inject_ns: u64 = std::env::var("TC_HOT_INJECT_OUTLIER_NS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(0);

    let params = params();
    let (canon, state) = graph_backed_compile();

    // Large parameter vector: encoded size well above the 64B inline budget
    // (16 x (1 + 33) = 544B), exercising the spill branch.
    let large_params: Vec<SqlLiteral> = (0..16)
        .map(|i| SqlLiteral::String(format!("param-value-{i:02}-aaaaaaaaaaaaaaaa")))
        .collect();

    // Charter A03 bit-identity preconditions before any timing:
    // 1. small (<64B): fold_params_anchored == admitted_statement_hash ref.
    // 2. large (>64B): the receipt path folds the reference encoding
    //    (law-stable fallback); inline/digest partition must classify
    //    exactly as the encoded size dictates.
    let small_ref = admitted_statement_hash(&canon, &params);
    let small_fast = fold_params_anchored(&state, &params);
    if small_ref != small_fast {
        eprintln!(
            "TC-HOT-01 REFUSED: ParamBytes small fold {:02x?} != reference {:02x?}",
            small_fast, small_ref
        );
        std::process::exit(2);
    }
    let large_encoded = ParamBytes::encode(&large_params);
    if large_encoded.is_inline() {
        eprintln!(
            "TC-HOT-01 REFUSED: large param vector ({}B) classified inline; spill branch unreachable",
            16 * (1 + 33)
        );
        std::process::exit(2);
    }
    let large_fast = fold_params_anchored(&state, &large_params);
    let mut h_large_ref = blake3::Hasher::new();
    h_large_ref.update(&state);
    hash_param_buffer(&mut h_large_ref, &large_params);
    if large_fast != *h_large_ref.finalize().as_bytes() {
        eprintln!("TC-HOT-01 REFUSED: large-fold fallback is not reference-identical");
        std::process::exit(2);
    }

    // Bit-identity precondition (P28-SIL §3.1 law): anchor path digest MUST
    // equal the admitted_statement_hash reference before any timing is
    // admitted.
    let baseline_digest = admitted_statement_hash(&canon, &params);
    let clone_digest = {
        let mut h = blake3::Hasher::new();
        h.update(state.as_slice());
        hash_param_buffer(&mut h, &params);
        *h.finalize().as_bytes()
    };
    if baseline_digest != clone_digest {
        eprintln!(
            "TC-HOT-01 REFUSED: clone-path digest {:02x?} != re-absorb reference {:02x?}",
            clone_digest, baseline_digest
        );
        std::process::exit(2);
    }

    println!("=== TC-HOT-01 receipt hot path (re-absorb baseline vs clone path) ===");
    println!("mode={}", if full { "FULL" } else { "SMOKE" });
    println!("{}", machine_disclosure());
    println!(
        "iterations={iters} canon_bytes={} params={} bit_identity=VERIFIED",
        canon.len(),
        params.len()
    );
    println!("budget: clone-path p99 <= {BUDGET_NS} ns");
    println!("budget: refusal build+render p50/p99 <= {REFUSAL_BUDGET_NS} ns (0.10us PRD)");
    println!("digest={:02x?}", baseline_digest);
    println!("{}", isolation_disclosure());

    // Warm-up (not timed): JIT-free but heats icache/branch predictors and
    // the allocator so batch 1 is not systematically slower than batch 3
    // (variance-stability precondition).
    const WARMUP_ITERS: u64 = 50_000;
    for _ in 0..WARMUP_ITERS {
        let d = admitted_statement_hash(black_box(&canon), black_box(&params));
        black_box(d);
        let mut h = blake3::Hasher::new();
        h.update(state.as_slice());
        hash_param_buffer(&mut h, black_box(&params));
        black_box(*h.finalize().as_bytes());
    }
    println!("warmup_iterations={WARMUP_ITERS} (untimed)");

    let mut baseline_hist = Histogram::<u64>::new(3).expect("hdrhistogram init");
    let mut clone_hist = Histogram::<u64>::new(3).expect("hdrhistogram init");
    let mut refusal_hist = Histogram::<u64>::new(3).expect("hdrhistogram init");
    let mut small_param_hist = Histogram::<u64>::new(3).expect("hdrhistogram init");
    let mut large_param_hist = Histogram::<u64>::new(3).expect("hdrhistogram init");
    // Per-sample clone-path series for the Shewhart control-chart rules
    // (scripts/spc_court.sh): rule 1 needs raw samples beyond mean±3σ, rule 2
    // needs run-length structure over consecutive samples. Raw u64 ns values,
    // in measurement order. Recorded unconditionally (allocation is per-leg
    // trivial vs the hash cost); exported only under TC_HOT_SPC_EXPORT.
    let mut clone_series: Vec<u64> = Vec::with_capacity(1 << 12);
    let mut tail: [u8; 32] = [0u8; 32];
    // Variance-stability self-check (charter invariant: run-to-run variance
    // < 5%): 3 consecutive batches of equal size, per-batch clone-path p50,
    // relative spread = (max - min) / mean of the 3 p50s must stay under 5%.
    const BATCHES: u64 = 3;
    const VARIANCE_BUDGET_PCT: f64 = 5.0;
    let batch_iters = iters / BATCHES;
    let mut batch_p50s: Vec<u64> = Vec::with_capacity(BATCHES as usize);
    let start = Instant::now();
    for batch in 0..BATCHES {
        let mut batch_clone_hist = Histogram::<u64>::new(3).expect("hdrhistogram init");
        for _ in 0..batch_iters {
        // Leg 1: full re-absorb baseline.
        let t0 = Instant::now();
        let d = admitted_statement_hash(black_box(&canon), black_box(&params));
        let el = t0.elapsed();
        black_box(d);
        tail = d;
        baseline_hist.record(el.as_nanos() as u64).expect("record");

        // Leg 2: incremental anchor path (h_ast update + P_params fold).
        if inject_ns > 0 {
            // Anti-vacuity busy-spin (structural fault injection, no sleeps).
            let spin_start = Instant::now();
            while spin_start.elapsed().as_nanos() < inject_ns as u128 {
                std::hint::black_box(&spin_start);
            }
        }
        let t1 = Instant::now();
        let mut h = blake3::Hasher::new();
        h.update(state.as_slice());
        hash_param_buffer(&mut h, black_box(&params));
        let d2 = *h.finalize().as_bytes();
        let el2 = t1.elapsed();
        black_box(d2);
        let el2_ns = el2.as_nanos() as u64;
        clone_hist.record(el2_ns).expect("record");
        batch_clone_hist.record(el2_ns).expect("record");
        clone_series.push(el2_ns);

        // Leg 3: refusal invariant (construction + Display, dialect.rs surface).
        let t2 = Instant::now();
        let refusal = graphlaw::dialect::Refusal {
            kind: graphlaw::dialect::RefusalKind::EngineRejected,
            dialect: Some(graphlaw::dialect::Dialect::N3),
            engine: Some(graphlaw::dialect::Dialect::N3.engine()),
            message: "tc-hot-01 invariant probe".to_string(),
        };
        let rendered = format!("{refusal}");
        let el3 = t2.elapsed();
        black_box(rendered);
        refusal_hist.record(el3.as_nanos() as u64).expect("record");

        // Leg 4 (charter A03): SMALL-params integration (<64B inline).
        // Timed unit = ParamBytes::encode + the single contiguous fold
        // update (param-integration cost; hasher new/finalize excluded, as
        // those are shared receipt costs, not param ingestion).
        let t3 = Instant::now();
        let enc_small = ParamBytes::encode(black_box(&params));
        let mut h4 = blake3::Hasher::new();
        h4.update(&state);
        enc_small.fold(&mut h4);
        let el4 = t3.elapsed();
        black_box(*h4.finalize().as_bytes());
        small_param_hist.record(el4.as_nanos() as u64).expect("record");

        // Leg 5 (charter A03): LARGE-params integration (>64B spill branch).
        let t4 = Instant::now();
        let enc_large = ParamBytes::encode(black_box(&large_params));
        let mut h5 = blake3::Hasher::new();
        h5.update(&state);
        enc_large.fold(&mut h5);
        let el5 = t4.elapsed();
        black_box(*h5.finalize().as_bytes());
        large_param_hist.record(el5.as_nanos() as u64).expect("record");
        }
        batch_p50s.push(batch_clone_hist.value_at_quantile(0.50));
    }
    let timed_iters = batch_iters * BATCHES;
    let total = start.elapsed();

    // Variance self-check verdict (charter invariant < 5%): PASS /
    // VARIANCE-EXCEEDED, emitted honestly either way.
    let batch_mean =
        batch_p50s.iter().map(|v| *v as f64).sum::<f64>() / BATCHES as f64;
    let spread = (batch_p50s.iter().max().unwrap() - batch_p50s.iter().min().unwrap()) as f64;
    let spread_pct = if batch_mean > 0.0 { spread / batch_mean * 100.0 } else { f64::INFINITY };
    let variance_ok = spread_pct < VARIANCE_BUDGET_PCT;
    println!(
        "VARIANCE_SELF_CHECK batches={BATCHES} batch_iters={batch_iters} batch_p50s={:?} spread_pct={spread_pct:.2}% budget={VARIANCE_BUDGET_PCT}% : {}",
        batch_p50s,
        if variance_ok { "PASS" } else { "VARIANCE-EXCEEDED" }
    );

    // Refusal invariant: the Display rendering must carry class + message.
    let refusal = graphlaw::dialect::Refusal {
        kind: graphlaw::dialect::RefusalKind::EngineRejected,
        dialect: Some(graphlaw::dialect::Dialect::N3),
        engine: Some(graphlaw::dialect::Dialect::N3.engine()),
        message: "tc-hot-01 invariant probe".to_string(),
    };
    let rendered = format!("{refusal}");
    let invariant_ok =
        rendered.contains("EngineRejected") && rendered.contains("tc-hot-01 invariant probe");

    println!("digest_tail={:02x?}", tail);
    for (name, h) in [("re_absorb_baseline", &baseline_hist), ("clone_path", &clone_hist), ("refusal_invariant", &refusal_hist), ("param_integration_small", &small_param_hist), ("param_integration_large", &large_param_hist)] {
        let (p50, p95, p99, p999) = (
            h.value_at_quantile(0.50),
            h.value_at_quantile(0.95),
            h.value_at_quantile(0.99),
            h.value_at_quantile(0.999),
        );
        println!("{name}: p50={p50}ns p95={p95}ns p99={p99}ns p99.9={p999}ns");
    }
    println!(
        "throughput: {:.1} full-legs/sec (all 3 legs per iteration)",
        timed_iters as f64 / total.as_secs_f64()
    );
    println!("REFUSAL_INVARIANT display_render={rendered:?} carries_class_and_message={invariant_ok}");

    let p99 = clone_hist.value_at_quantile(0.99);
    let pass = p99 <= BUDGET_NS && invariant_ok && variance_ok;
    println!(
        "GATE clone_path p99({p99}ns) <= {BUDGET_NS}ns AND refusal invariant AND variance<{VARIANCE_BUDGET_PCT}% : {}",
        if pass { "PASS" } else { "FAIL" }
    );

    // OCAP row-1 court: measured String-path refusal cost vs the 0.10us PRD
    // budget. Honest FAIL expected — RefusalMessage::Static fast path
    // (designs/P28-SIL-specs.md section 3.1) is the designed closure
    // (post-integration item), never implemented.
    let r_p50 = refusal_hist.value_at_quantile(0.50);
    let r_p99 = refusal_hist.value_at_quantile(0.99);
    let r_pass = r_p50 <= REFUSAL_BUDGET_NS && r_p99 <= REFUSAL_BUDGET_NS;
    println!(
        "GATE refusal_string_path p50({r_p50}ns)/p99({r_p99}ns) <= {REFUSAL_BUDGET_NS}ns (0.10us PRD) : {}",
        if r_pass { "PASS" } else { "FAIL" }
    );
    println!(
        "NOTE closure=RefusalMessage::Static fast path (designs/P28-SIL-specs.md section 3.1), status=post-integration item; measured cost is the heap-String message path (src/dialect.rs Refusal {{ message: String }})"
    );

    // Charter A03 param-integration gates: encode + fold, p99 <= 40ns per
    // leg (small <64B inline, large >64B spill), bit-identity already
    // verified as a precondition above.
    let sp = small_param_hist.value_at_quantile(0.99);
    let lp = large_param_hist.value_at_quantile(0.99);
    let param_pass = sp <= PARAM_BUDGET_NS && lp <= PARAM_BUDGET_NS;
    println!(
        "GATE param_integration small p99({sp}ns) / large p99({lp}ns) <= {PARAM_BUDGET_NS}ns : {}",
        if param_pass { "PASS" } else { "FAIL" }
    );
    // Machine-readable SPC export (scripts/spc_court.sh): clone-path sample
    // summary for μ/σ/Cpk computation against the USL. Written BEFORE the
    // internal gate exits so an out-of-budget run still exports its sample —
    // the SPC court owns its verdict from the data, not the bench gate.
    if let Some(path) = spc_export {
        use std::io::Write as _;
        let mean = clone_hist.mean();
        let stddev = clone_hist.stdev();
        let q = |p: f64| clone_hist.value_at_quantile(p);
        let write_json = |series: &[u64]| -> std::io::Result<()> {
            let mut f = std::io::BufWriter::new(std::fs::File::create(&path)?);
            write!(
                f,
                concat!(
                    "{{\"leg\":\"clone_path\",\"iterations\":{},\"inject_outlier_ns\":{},",
                    "\"mean_ns\":{:.3},\"stddev_ns\":{:.3},",
                    "\"p50_ns\":{},\"p95_ns\":{},\"p99_ns\":{},\"p999_ns\":{},\"max_ns\":{},"
                ),
                iters,
                inject_ns,
                mean,
                stddev,
                q(0.50),
                q(0.95),
                q(0.99),
                q(0.999),
                clone_hist.max(),
            )?;
            // Per-sample series (Shewhart rules 1+2 input): raw clone-path
            // ns values in measurement order.
            f.write_all(b"\"samples_ns\":[")?;
            for (i, s) in series.iter().enumerate() {
                if i > 0 {
                    f.write_all(b",")?;
                }
                write!(f, "{s}")?;
            }
            f.write_all(b"]}\n")?;
            f.flush()
        };
        if let Err(e) = write_json(&clone_series) {
            eprintln!("TC-HOT-01 REFUSED: SPC export write failure: {e}");
            std::process::exit(2);
        }
        println!(
            "SPC_EXPORT path={path} samples={}",
            clone_series.len()
        );
    }

    // Lane A02 timestamp-source court: the coarse hot-path clock
    // (src/duckdb_eval.rs CoarseClock, recalibrated from Instant every 64th
    // read, fetch_max-clamped) must cost <10 ns median acquisition AND stay
    // monotonic non-decreasing across the full read sequence. Baseline leg
    // times bare Instant::now() for honest comparison.
    {
        use graphlaw::duckdb_eval::coarse_now_ns;
        const TS_ITERS: u32 = 1_000_000;
        // warmup
        for _ in 0..10_000 {
            black_box(coarse_now_ns());
        }
        let mut mono_ok = true;
        let mut prev = 0u64;
        let t = Instant::now();
        for _ in 0..TS_ITERS {
            let v = coarse_now_ns();
            if v < prev {
                mono_ok = false;
            }
            prev = v;
            black_box(v);
        }
        let total = t.elapsed();
        // loop-overhead baseline (same loop, no clock read)
        let t2 = Instant::now();
        for _ in 0..TS_ITERS {
            black_box(0u8);
        }
        let ovh = t2.elapsed();
        let per_ns =
            (total.as_nanos() as f64 - ovh.as_nanos() as f64) / TS_ITERS as f64;
        let inst_t = Instant::now();
        for _ in 0..TS_ITERS {
            black_box(Instant::now());
        }
        let inst_per_ns = inst_t.elapsed().as_nanos() as f64 / TS_ITERS as f64;
        println!(
            "TIMESTAMP_SOURCE coarse_now_ns per-call={per_ns:.1}ns (overhead-subtracted) instant_baseline={inst_per_ns:.1}ns monotonic={mono_ok}"
        );
        let ts_pass = per_ns < 10.0 && mono_ok;
        println!(
            "GATE timestamp_source {per_ns:.1}ns < 10ns AND monotonic : {}",
            if ts_pass { "PASS" } else { "FAIL" }
        );
        if !ts_pass {
            std::process::exit(1);
        }
    }

    if !pass || !r_pass || !param_pass {
        std::process::exit(1);
    }
}
