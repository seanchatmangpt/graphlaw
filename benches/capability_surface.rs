//! Capability-surface benchmark — every registered ABI op plus the native
//! `duckdb` modes, timed through real `graphlaw::abi::call` (Chicago: no
//! mocks). Workloads are the positive-control `ok` requests from
//! `registry/op-examples.json` (one per registry op) and the documented
//! duckdb open/exec/stream/close arms.
//!
//! Method: min-of-batches (least-contended estimate). Warm-up is untimed.
//! This is a Measure-lane census, not a hard gate — verdicts are relative
//! (rank + disclosure), not USL courts.
//!
//! Run:
//!   CARGO_TARGET_DIR=target-cap-bench cargo bench --bench capability_surface -- --nocapture
//! Optional:
//!   CAP_BENCH_JSON=benches/reports/capability_surface.json  (write machine receipt)
//!   CAP_BENCH_BATCHES=20 CAP_BENCH_ITERS=50                 (override sample plan)

use std::collections::BTreeMap;
use std::hint::black_box;
use std::time::Instant;

use serde_json::{json, Value};

const OP_EXAMPLES: &str = include_str!("../registry/op-examples.json");

/// Registry ops in wire order (must match `capabilities` / registry).
const REGISTRY_OPS: &[&str] = &[
    "capabilities",
    "sniff",
    "parse",
    "convert",
    "canonical",
    "sparql",
    "shacl",
    "shex",
    "n3",
    "entail",
    "datalog",
    "hooks",
    "law",
    "policy",
];

#[derive(Clone, Debug)]
struct Leg {
    id: String,
    op: String,
    request_bytes: Vec<u8>,
    /// When set, rebuild the request each iteration (duckdb session names).
    rebuild: Option<RebuildKind>,
}

#[derive(Clone, Copy, Debug)]
enum RebuildKind {
    DuckdbOpen,
    DuckdbExecCreate,
    DuckdbStreamPage,
    DuckdbCloseSession,
}

fn env_u64(key: &str, default: u64) -> u64 {
    std::env::var(key)
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(default)
}

/// Min-of-batches over a pure body (no per-iter setup). Same method as
/// `p28_hot_path_bench`: the minimum batch is the least-contended estimate.
fn timed_ns_m<F: FnMut()>(warmup: u64, batch: u64, batches: u64, mut f: F) -> f64 {
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

/// Min-of-batches where each iteration may prepare `(bytes, session)` outside
/// the stopwatch, time only `abi::call`, then tear down with the session.
/// Used for duckdb modes so open/seed work is not attributed to the mode.
fn timed_ns_prepared<P, A>(
    warmup: u64,
    batch: u64,
    batches: u64,
    mut prepare: P,
    mut after: A,
) -> f64
where
    P: FnMut() -> (Vec<u8>, String),
    A: FnMut(&str, Value),
{
    for _ in 0..warmup {
        let (bytes, session) = prepare();
        let out = graphlaw::abi::call(&bytes);
        let resp: Value = serde_json::from_slice(&out).expect("json");
        after(&session, resp);
    }
    let mut best = f64::INFINITY;
    for _ in 0..batches {
        let mut total = std::time::Duration::ZERO;
        for _ in 0..batch {
            let (bytes, session) = prepare();
            let t = Instant::now();
            let out = graphlaw::abi::call(black_box(&bytes));
            total += t.elapsed();
            let resp: Value = serde_json::from_slice(&out).expect("json");
            after(&session, resp);
        }
        let per = total.as_secs_f64() / batch as f64 * 1e9;
        if per < best {
            best = per;
        }
    }
    best
}

fn load_positive_controls() -> BTreeMap<String, Leg> {
    let doc: Value = serde_json::from_str(OP_EXAMPLES).expect("op-examples.json");
    assert_eq!(doc["schema"], "graphlaw.op-examples/1");
    let examples = doc["examples"].as_array().expect("examples array");
    let mut by_op: BTreeMap<String, Vec<&Value>> = BTreeMap::new();
    for ex in examples {
        let op = ex["op"].as_str().expect("op").to_string();
        by_op.entry(op).or_default().push(ex);
    }
    let mut out = BTreeMap::new();
    for op in REGISTRY_OPS {
        let exs = by_op
            .get(*op)
            .unwrap_or_else(|| panic!("op-examples missing op `{op}`"));
        let pick = exs
            .iter()
            .find(|e| e["positive_control"] == true && e["outcome"] == "ok")
            .or_else(|| exs.iter().find(|e| e["outcome"] == "ok"))
            .unwrap_or_else(|| panic!("no ok example for `{op}`"));
        let id = pick["id"].as_str().expect("id").to_string();
        let request = &pick["request"];
        assert_eq!(request["op"], *op, "{id}: request.op");
        let request_bytes = serde_json::to_vec(request).expect("request serializes");
        // Smoke once: positive control must succeed before we time it.
        let resp: Value =
            serde_json::from_slice(&graphlaw::abi::call(&request_bytes)).expect("response JSON");
        assert_eq!(
            resp["ok"], true,
            "positive control `{id}` must be ok before timing: {resp}"
        );
        out.insert(
            (*op).to_string(),
            Leg {
                id,
                op: (*op).to_string(),
                request_bytes,
                rebuild: None,
            },
        );
    }
    out
}

fn duckdb_create_turtle() -> &'static str {
    r#"@prefix sql: <http://graphlaw.org/sql#> .
@prefix ex: <http://example.org/cap-bench#> .
ex:C a sql:CreateStatement ;
    sql:statementIndex 1 ;
    sql:targetTable "cap_bench_events" ;
    sql:assignColumn "batch_id" .
"#
}

fn duckdb_insert_turtle() -> &'static str {
    r#"@prefix sql: <http://graphlaw.org/sql#> .
@prefix xsd: <http://www.w3.org/2001/XMLSchema#> .
@prefix ex: <http://example.org/cap-bench#> .
ex:I a sql:InsertStatement ;
    sql:statementIndex 1 ;
    sql:targetTable "cap_bench_events" ;
    sql:assignColumn "batch_id" ;
    sql:assignValue "7"^^xsd:integer .
"#
}

fn duckdb_select_turtle() -> &'static str {
    r#"@prefix sql: <http://graphlaw.org/sql#> .
@prefix ex: <http://example.org/cap-bench#> .
ex:S a sql:SelectStatement ;
    sql:statementIndex 1 ;
    sql:fromTable [ sql:tableName "cap_bench_events" ] ;
    sql:selectList ( [ sql:columnName "batch_id" ] ) .
"#
}

fn duckdb_exec_req(session: &str, turtle: &str, query_ast: &str) -> Value {
    json!({
        "op": "duckdb",
        "mode": "exec",
        "session": session,
        "query_ast": query_ast,
        "ast": {"turtle": turtle},
        "subject_prefix": "urn:row/",
        "vocab_prefix": "urn:p/",
    })
}

fn call_ok(req: &Value) -> Value {
    let bytes = serde_json::to_vec(req).expect("req");
    let resp: Value = serde_json::from_slice(&graphlaw::abi::call(&bytes)).expect("response JSON");
    assert_eq!(resp["ok"], true, "request must succeed: {req} → {resp}");
    resp
}

/// DuckDB legs need fresh session names each call so open/close stay idempotent
/// under batch timing. Stream re-seeds a cursor outside the timed call when
/// the prior page exhausted it — for timing we measure one stream page after
/// a fresh SELECT exec (setup outside the stopwatch for the stream leg).
fn duckdb_legs() -> Vec<Leg> {
    // Smoke the lifecycle once so a broken DuckDB surface fails loud before
    // the timing loop, not as a silent NaN.
    let smoke = format!("cap-bench-smoke-{}", std::process::id());
    call_ok(&json!({"op": "duckdb", "mode": "open", "session": smoke}));
    call_ok(&duckdb_exec_req(
        &smoke,
        duckdb_create_turtle(),
        "urn:create",
    ));
    call_ok(&duckdb_exec_req(
        &smoke,
        duckdb_insert_turtle(),
        "urn:insert",
    ));
    let sel = call_ok(&duckdb_exec_req(
        &smoke,
        duckdb_select_turtle(),
        "urn:select",
    ));
    let cur = sel["cursor"].as_str().expect("cursor").to_string();
    call_ok(&json!({"op": "duckdb", "mode": "stream", "cursor": cur, "max_rows": 10}));
    call_ok(&json!({"op": "duckdb", "mode": "close", "session": smoke}));

    vec![
        Leg {
            id: "duckdb.open".into(),
            op: "duckdb".into(),
            request_bytes: Vec::new(),
            rebuild: Some(RebuildKind::DuckdbOpen),
        },
        Leg {
            id: "duckdb.exec.create".into(),
            op: "duckdb".into(),
            request_bytes: Vec::new(),
            rebuild: Some(RebuildKind::DuckdbExecCreate),
        },
        Leg {
            id: "duckdb.stream.page".into(),
            op: "duckdb".into(),
            request_bytes: Vec::new(),
            rebuild: Some(RebuildKind::DuckdbStreamPage),
        },
        Leg {
            id: "duckdb.close.session".into(),
            op: "duckdb".into(),
            request_bytes: Vec::new(),
            rebuild: Some(RebuildKind::DuckdbCloseSession),
        },
    ]
}

/// Prepare the request bytes for one duckdb-mode iteration. Side effects
/// that are not the mode under test (session open, table seed) happen here
/// and are excluded from the stopwatch by `timed_ns_prepared`.
fn prepare_duckdb(kind: RebuildKind, counter: &mut u64) -> (Vec<u8>, String) {
    *counter += 1;
    match kind {
        RebuildKind::DuckdbOpen => {
            let session = format!("cap-bench-open-{counter}");
            let bytes = serde_json::to_vec(&json!({
                "op": "duckdb", "mode": "open", "session": &session
            }))
            .unwrap();
            (bytes, session)
        }
        RebuildKind::DuckdbExecCreate => {
            let session = format!("cap-bench-exec-{counter}");
            call_ok(&json!({"op": "duckdb", "mode": "open", "session": &session}));
            let bytes = serde_json::to_vec(&duckdb_exec_req(
                &session,
                duckdb_create_turtle(),
                "urn:create",
            ))
            .unwrap();
            (bytes, session)
        }
        RebuildKind::DuckdbStreamPage => {
            let session = format!("cap-bench-stream-{counter}");
            call_ok(&json!({"op": "duckdb", "mode": "open", "session": &session}));
            call_ok(&duckdb_exec_req(
                &session,
                duckdb_create_turtle(),
                "urn:create",
            ));
            call_ok(&duckdb_exec_req(
                &session,
                duckdb_insert_turtle(),
                "urn:insert",
            ));
            let sel = call_ok(&duckdb_exec_req(
                &session,
                duckdb_select_turtle(),
                "urn:select",
            ));
            let cur = sel["cursor"].as_str().expect("cursor");
            let bytes = serde_json::to_vec(&json!({
                "op": "duckdb", "mode": "stream", "cursor": cur, "max_rows": 10
            }))
            .unwrap();
            (bytes, session)
        }
        RebuildKind::DuckdbCloseSession => {
            let session = format!("cap-bench-close-{counter}");
            call_ok(&json!({"op": "duckdb", "mode": "open", "session": &session}));
            let bytes = serde_json::to_vec(&json!({
                "op": "duckdb", "mode": "close", "session": &session
            }))
            .unwrap();
            (bytes, session)
        }
    }
}

fn teardown_duckdb(kind: RebuildKind, session: &str, resp: &Value) {
    assert_eq!(resp["ok"], true, "duckdb {kind:?} failed: {resp}");
    match kind {
        RebuildKind::DuckdbOpen | RebuildKind::DuckdbExecCreate | RebuildKind::DuckdbStreamPage => {
            // Best-effort close so sessions do not accumulate across batches.
            let _ = graphlaw::abi::call(
                serde_json::to_vec(&json!({
                    "op": "duckdb", "mode": "close", "session": session
                }))
                .unwrap()
                .as_slice(),
            );
        }
        RebuildKind::DuckdbCloseSession => {}
    }
}

#[derive(Clone, Debug)]
struct ResultRow {
    id: String,
    op: String,
    ns_per_op: f64,
    request_bytes: usize,
    ok: bool,
}

fn machine_line() -> String {
    let cores = std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(0);
    let rustc = std::process::Command::new("rustc")
        .arg("--version")
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_else(|_| "?".into());
    format!(
        "{} / {} / {} cores / {rustc}",
        std::env::consts::ARCH,
        std::env::consts::OS,
        cores
    )
}

fn main() {
    let warmup = env_u64("CAP_BENCH_WARMUP", 5);
    let batch = env_u64("CAP_BENCH_ITERS", 25);
    let batches = env_u64("CAP_BENCH_BATCHES", 10);
    // Heavier ops (law, hooks, duckdb) use the same plan; override via env.

    println!("=== GraphLaw capability-surface benchmark ===");
    println!("[MACHINE] {}", machine_line());
    println!("[PLAN] warmup={warmup} iters/batch={batch} batches={batches} (min-of-batches)");
    println!("[CMD] CARGO_TARGET_DIR=target-cap-bench cargo bench --bench capability_surface");
    println!(
        "[SOURCE] registry/op-examples.json positive_control ok + duckdb open/exec/stream/close"
    );

    let controls = load_positive_controls();
    let mut legs: Vec<Leg> = REGISTRY_OPS
        .iter()
        .map(|op| controls.get(*op).expect("loaded").clone())
        .collect();
    legs.extend(duckdb_legs());

    // Coverage gate: every registry op + four duckdb modes present.
    assert_eq!(
        legs.len(),
        REGISTRY_OPS.len() + 4,
        "expected 14 registry ops + 4 duckdb modes"
    );

    let mut rows = Vec::with_capacity(legs.len());
    let mut counter = 0u64;

    for leg in &legs {
        let (ns, req_len, note) = match leg.rebuild {
            None => {
                let bytes = leg.request_bytes.clone();
                let req_len = bytes.len();
                let ns = timed_ns_m(warmup, batch, batches, || {
                    let out = graphlaw::abi::call(black_box(&bytes));
                    black_box(out);
                });
                (ns, req_len, "positive_control")
            }
            Some(kind) => {
                let (sample, session) = prepare_duckdb(kind, &mut counter);
                let req_len = sample.len();
                // Drop the sample without invoking the mode: for Open the
                // session was never opened; for others, close the seed session.
                match kind {
                    RebuildKind::DuckdbOpen => {}
                    _ => {
                        let _ = graphlaw::abi::call(
                            serde_json::to_vec(&json!({
                                "op": "duckdb", "mode": "close", "session": session
                            }))
                            .unwrap()
                            .as_slice(),
                        );
                    }
                }

                let ns = timed_ns_prepared(
                    warmup,
                    batch,
                    batches,
                    || prepare_duckdb(kind, &mut counter),
                    |session, resp| teardown_duckdb(kind, session, &resp),
                );
                (ns, req_len, "mode call only; seed outside stopwatch")
            }
        };

        println!(
            "[{:<22}] {:>10.0} ns/op  ({}; req {req_len} B)",
            leg.id, ns, note
        );
        rows.push(ResultRow {
            id: leg.id.clone(),
            op: leg.op.clone(),
            ns_per_op: ns,
            request_bytes: req_len,
            ok: true,
        });
    }

    let mut ranked = rows.clone();
    ranked.sort_by(|a, b| a.ns_per_op.partial_cmp(&b.ns_per_op).unwrap());
    println!("\n=== rank (fastest → slowest, min-of-batches ns/op) ===");
    for (i, r) in ranked.iter().enumerate() {
        println!(
            "{:>2}. {:<22} {:>10.0} ns/op  ({} B req)",
            i + 1,
            r.id,
            r.ns_per_op,
            r.request_bytes
        );
    }

    let fastest = &ranked[0];
    let slowest = ranked.last().unwrap();
    println!(
        "\n[VERDICT] COVERED {} legs (14 registry ops + 4 duckdb modes); all ok",
        rows.len()
    );
    println!(
        "[VERDICT] fastest={} ({:.0} ns)  slowest={} ({:.0} ns)  span={:.1}x",
        fastest.id,
        fastest.ns_per_op,
        slowest.id,
        slowest.ns_per_op,
        slowest.ns_per_op / fastest.ns_per_op.max(1.0)
    );
    println!(
        "[NOTE] duckdb mode legs time only the mode call; session open/seed/close sit outside the stopwatch."
    );

    if let Ok(path) = std::env::var("CAP_BENCH_JSON") {
        let payload = json!({
            "schema": "graphlaw.capability-surface-bench/1",
            "graphlaw_version": env!("CARGO_PKG_VERSION"),
            "machine": machine_line(),
            "plan": {"warmup": warmup, "iters_per_batch": batch, "batches": batches, "method": "min-of-batches"},
            "legs": rows.iter().map(|r| json!({
                "id": r.id,
                "op": r.op,
                "ns_per_op": r.ns_per_op,
                "request_bytes": r.request_bytes,
                "ok": r.ok,
            })).collect::<Vec<_>>(),
            "rank": ranked.iter().map(|r| r.id.clone()).collect::<Vec<_>>(),
            "fastest": fastest.id,
            "slowest": slowest.id,
        });
        if let Some(parent) = std::path::Path::new(&path).parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        std::fs::write(&path, serde_json::to_vec_pretty(&payload).unwrap())
            .unwrap_or_else(|e| panic!("write {path}: {e}"));
        println!("[WROTE] {path}");
    }
}
