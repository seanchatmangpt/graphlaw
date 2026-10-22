//! P28 hot-path baseline (release-mode timing test; no criterion harness exists
//! in this crate). Measures the three PRD-budget surfaces against real
//! collaborators:
//!   1. H_query = BLAKE3(Canon(G_ast) || P_params) per-op cost (`admitted_statement_hash`)
//!   2. compile_turtle = the actual admission gate (SHACL validate + canonicalize),
//!      once per compile — reported as ADMISSION cost, explicitly NOT hot path
//!   3. eval_ast_query per-call overhead excluding engine execution
//!      (trivial SELECT round-trip minus raw prepared-statement baseline)
//! Run: CARGO_TARGET_DIR=target-bench cargo test --release --test p28_hot_path_bench -- --nocapture
use graphlaw::duckdb_eval::{GraphLawDuckDB, admitted_statement_hash, hash_param_buffer};
use graphlaw::sql_ast::{CompiledQuery, SqlLiteral, compile_turtle};
use std::hint::black_box;
use std::time::Instant;

const SELECT_T: &str = r#"<http://example.org/s> a sql:SelectStatement ;
    sql:select ( [ a sql:ColumnIdentifier ; sql:columnName "id" ; sql:scope sql:ViewScope ] ) ;
    sql:from ( [ a sql:TableIdentifier ; sql:tableName "t" ] ) ."#;

/// Min-of-batches: under fan-out machine contention a single mean is noise
/// (witnessed: eval measured FASTER than the raw baseline it strictly
/// contains). The minimum batch is the least-contended estimate.
fn timed_ns_m<F: FnMut()>(warmup: usize, batch: u64, batches: u64, mut f: F) -> f64 {
    for _ in 0..warmup {
        f();
    }
    let mut best = f64::INFINITY;
    for _ in 0..batches {
        let t = Instant::now();
        for _ in 0..batch {
            f();
        }
        let per = t.elapsed().as_secs_f64() / batch as f64 * 1e9;
        if per < best {
            best = per;
        }
    }
    best
}

#[test]
fn p28_hot_path_baseline() {
    let doc = format!("@prefix sql: <http://graphlaw.org/sql#> .\n@prefix xsd: <http://www.w3.org/2001/XMLSchema#> .\n{}\n<http://example.org/s> sql:statementIndex 1 .", SELECT_T);
    let compiled = compile_turtle(&doc).expect("select compiles");
    let canon = compiled
        .canon_graph_bytes
        .clone()
        .expect("graph-backed compile carries canon bytes");
    let params: Vec<SqlLiteral> = vec![
        SqlLiteral::Integer(1),
        SqlLiteral::String("x".into()),
        SqlLiteral::Float(1.0),
        SqlLiteral::Boolean(true),
    ];

    // (1) H_query per-op cost
    let h_ns = timed_ns_m(1_000, 20_000, 10, || {
        black_box(admitted_statement_hash(black_box(&canon), black_box(&params)));
    });
    let h_ns_empty = timed_ns_m(1_000, 20_000, 10, || {
        black_box(admitted_statement_hash(black_box(&canon), black_box(&[])));
    });
    println!("[1] H_query ({} canon bytes + 4 params): {:.1} ns/op (budget <= 150 ns)", canon.len(), h_ns);
    println!("[1] H_query ({} canon bytes, 0 params): {:.1} ns/op", canon.len(), h_ns_empty);

    // (2) compile_turtle = admission (once per compile, NOT hot path)
    let c_ns = timed_ns_m(20, 100, 5, || {
        black_box(compile_turtle(black_box(&doc)).unwrap());
    });
    println!("[2] compile_turtle (admission, once per compile): {:.0} ns/compile", c_ns);

    // (3) eval_ast_query overhead vs raw prepared-statement baseline
    let db = GraphLawDuckDB::open_in_memory().unwrap();
    db.raw_connection()
        .execute_batch("CREATE TABLE t(id INTEGER, name VARCHAR); INSERT INTO t VALUES (1,'a'),(2,'b');")
        .unwrap();

    // SQLRDF-17: hand-built ASTs no longer admit (legacy raw-SQL arm is a
    // typed NoCanonicalGraph refusal), so the eval legs are graph-backed:
    // the trivial query carries the real compile's canon anchor.
    let trivial = CompiledQuery {
        sql: "SELECT id, name FROM t WHERE id = $1".into(),
        params: vec![SqlLiteral::Integer(1)],
        ast_graph_turtle: None,
        canon_graph_bytes: Some(canon.clone()),
        receipt_state: Some(compiled.receipt_state.expect("compile pins h_ast")),
        // Lane minimal unblock (SLA, disclosed): new CompiledQuery field from
        // the C1-a lane; this hand-built bench query is not cacheable.
        cacheable: false,
    };
    let trivial_canon = trivial.clone();

    let raw_ns = timed_ns_m(200, 2_000, 10, || {
        let mut stmt = db
            .raw_connection()
            .prepare("SELECT id, name FROM t WHERE id = $1")
            .unwrap();
        let mut rows = stmt.query([1i64]).unwrap();
        while let Some(row) = rows.next().unwrap() {
            black_box(row.get::<_, i64>(0).unwrap());
        }
    });

    let eval_canon_ns = timed_ns_m(200, 2_000, 10, || {
        black_box(
            db.eval_ast_query("http://ex/s", &trivial_canon, "http://ex/row", "http://ex/pred")
                .unwrap(),
        );
    });

    println!("[3] raw prepared-stmt round-trip (engine baseline): {:.0} ns/call", raw_ns);
    println!("[3] eval_ast_query (graph-backed H_query path): {:.0} ns/call", eval_canon_ns);
    // Anchor path: fresh hasher over h_ast || P_params + finalize (P28-SIL
    // two-node path; hash_param_buffer is private, so the param bytes are
    // the test's own equivalent typed encoding — surrogate, labeled as such).
    let mut pb = Vec::new();
    for p in &params {
        match p {
            SqlLiteral::Integer(i) => { pb.push(1u8); pb.extend_from_slice(&i.to_le_bytes()); }
            SqlLiteral::String(s) => { pb.push(2u8); pb.extend_from_slice(&(s.len() as u64).to_le_bytes()); pb.extend_from_slice(s.as_bytes()); }
            SqlLiteral::Float(f) => { pb.push(3u8); pb.extend_from_slice(&f.to_le_bytes()); }
            SqlLiteral::Boolean(b) => { pb.push(4u8); pb.push(*b as u8); }
            SqlLiteral::Null => pb.push(5u8),
        }
    }
    let state = compiled.receipt_state.unwrap();
    let fold_ns = timed_ns_m(1_000, 20_000, 10, || {
        let mut h = blake3::Hasher::new();
        h.update(state.as_slice());
        h.update(black_box(&pb));
        black_box(*h.finalize().as_bytes());
    });
    println!("[1b] two-node fold (h_ast anchor + {} param bytes + finalize): {:.1} ns/op", pb.len(), fold_ns);

    println!("[3] eval overhead (eval - raw, H_query fold path): {:.0} ns", eval_canon_ns - raw_ns);
    // Verdict, not a hard gate: this is a Measure-lane baseline. The budget
    // miss is the finding; enforcement is a later court's job.
    if h_ns <= 150.0 {
        println!("[VERDICT] H_query within 150 ns budget");
    } else {
        println!("[VERDICT] BUDGET-MISS: full H_query {:.0} ns vs 150 ns budget ({:.1}x over); incremental fold {:.0} ns vs 150 ns budget", h_ns, h_ns / 150.0, fold_ns);
    }
}

/// P28 PRD §5 TC-HOT-01 capability analysis of the M3 fold path
/// (fresh hasher over `h_ast` anchor + `hash_param_buffer` + `finalize`),
/// the path eval_ast_query actually executes when `receipt_state` is pinned.
/// USL = 0.25 µs = 250 ns (the TC-HOT-01 per-op budget). No LSL (lower is
/// better), so Cpk = Cpu = (USL - mu) / (3 sigma).
///
/// Run: CARGO_TARGET_DIR=target-dflss3 cargo test --release --test p28_hot_path_bench -- p28_hot_path_capability --nocapture
#[test]
fn p28_hot_path_capability() {
    const N: usize = 1_000_000; // samples (>= 1e6 per charter)
    const USL_NS: f64 = 250.0;

    let doc = format!("@prefix sql: <http://graphlaw.org/sql#> .\n@prefix xsd: <http://www.w3.org/2001/XMLSchema#> .\n{}\n<http://example.org/s> sql:statementIndex 1 .", SELECT_T);
    let compiled = compile_turtle(&doc).expect("select compiles");
    let state = compiled
        .receipt_state
        .expect("compile pins h_ast into receipt_state");
    let params: Vec<SqlLiteral> = vec![
        SqlLiteral::Integer(1),
        SqlLiteral::String("x".into()),
        SqlLiteral::Float(1.0),
        SqlLiteral::Boolean(true),
    ];

    // Warm-up before sampling (caches, page-in); not part of the study.
    for _ in 0..10_000 {
        let mut h = blake3::Hasher::new();
        h.update(state.as_slice());
        hash_param_buffer(&mut h, &params);
        black_box(*h.finalize().as_bytes());
    }

    // Per-sample timing: capability analysis needs the distribution, not a
    // batch mean. Instant::now overhead (~20-25 ns on this class of hardware)
    // is disclosed below and included in every sample symmetrically.
    let mut samples = Vec::with_capacity(N);
    for _ in 0..N {
        let t = Instant::now();
        let mut h = blake3::Hasher::new();
        h.update(state.as_slice());
        hash_param_buffer(&mut h, black_box(&params));
        black_box(*h.finalize().as_bytes());
        samples.push(t.elapsed().as_nanos() as f64);
    }

    let n = samples.len() as f64;
    let mean = samples.iter().sum::<f64>() / n;
    let var = samples.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / (n - 1.0);
    let sigma = var.sqrt();
    let cpk = (USL_NS - mean) / (3.0 * sigma);

    samples.sort_unstable_by(|a, b| a.partial_cmp(b).unwrap());
    let q = |p: f64| samples[((p * (n - 1.0)) as usize).min(samples.len() - 1)];
    let pct_below = samples.iter().filter(|x| **x <= USL_NS).count() as f64 / n * 100.0;

    println!("\n=== P28 hot-path capability analysis (M3 fold path) ===");
    println!("[MACHINE] {} / macOS {} / {} cores / rustc {}",
        std::env::consts::ARCH,
        { #[cfg(target_os = "macos")] { std::process::Command::new("sw_vers").args(["-productVersion"]).output().map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string()).unwrap_or_else(|_| "?".into()) } #[cfg(not(target_os = "macos"))] { "?".to_string() } },
        std::thread::available_parallelism().map(|p| p.get()).unwrap_or(0),
        { let v = rustc_version(); v });
    println!("[MACHINE] release profile, per-sample Instant::now overhead (~20-25 ns) included symmetrically");
    println!("[CMD] CARGO_TARGET_DIR=target-dflss3 cargo test --release --test p28_hot_path_bench -- p28_hot_path_capability --nocapture");
    println!("[PATH] h_ast anchor (32B Copy) + Hasher::new + hash_param_buffer(4 params) + finalize");
    println!("[N] {} samples, USL = {:.0} ns", N, USL_NS);
    println!("mu = {:.3} ns | sigma = {:.3} ns | Cpk(Cpu) = {:.2} | P(X > USL) = {:.4}%",
        mean, sigma, cpk, 100.0 - pct_below);
    println!("quantile table (ns):");
    println!("  p50={:.1} p90={:.1} p99={:.1} p99.9={:.1} p99.99={:.1} max={:.1}",
        q(0.50), q(0.90), q(0.99), q(0.999), q(0.9999), q(1.0));
    if mean <= USL_NS && cpk >= 1.33 {
        println!("[VERDICT] CAPABLE: mu <= USL and Cpk >= 1.33");
    } else {
        println!("[VERDICT] NOT-CAPABLE: charter mu=0.198us/sigma=0.012us/Cpk=1.44 NOT witnessed; measured Cpk={:.2}, mu={:.3}ns, sigma={:.3}ns", cpk, mean, sigma);
    }

    // Contention-clean reference: min-of-batches over 100-op batches (the
    // least-contended estimate, matching the baseline lane's methodology).
    // Separates the operation's intrinsic cost from scheduler noise in the
    // per-sample distribution above.
    let mut best = f64::INFINITY;
    for _ in 0..50 {
        let t = Instant::now();
        for _ in 0..100 {
            let mut h = blake3::Hasher::new();
            h.update(state.as_slice());
            hash_param_buffer(&mut h, black_box(&params));
            black_box(*h.finalize().as_bytes());
        }
        let per = t.elapsed().as_secs_f64() / 100.0 * 1e9;
        if per < best {
            best = per;
        }
    }
    println!("[REF] min-of-50x100-op batches: {:.1} ns/op (intrinsic cost, no per-op clock overhead, least-contended)", best);
}

fn rustc_version() -> String {
    std::process::Command::new("rustc")
        .arg("--version")
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_else(|_| "?".into())
}
