//! Lane L125 court: the `duckdb` op routed through the wasm unit (L115
//! finding).
//!
//! Findings this court pins down:
//!
//! 1. The routing is NOT missing from source: `src/abi.rs:479` dispatches
//!    `"duckdb" => crate::duckdb_op::dispatch` unconditionally on every
//!    target, and `wasm/src/lib.rs`'s shell routes every request through
//!    `graphlaw::abi::call`. The module built from current source contains the
//!    dispatch (data-section strings + 45 MB frozen-duckdb archive) — rung A.
//! 2. The L115 `unknown op \`duckdb\`` came from the SHIPPED artifact
//!    (priv/graphlaw.wasm, BLAKE3 `332ea218…2c30c`, 6,793,617 B, built
//!    26.10.21, before the op): driven through the same driver it returns the
//!    typed `Unsupported / unknown op \`duckdb\`` refusal — rung B (before
//!    state, executed witness).
//! 3. The freshly built module does not yet EXECUTE in any local runtime —
//!    typed runtime matrix in
//!    docs/jira/v26.10.22/designs/wasm-duckdb-op-wired.md — so the
//!    execution rung is `#[ignore]`-marked with the typed reason, never a
//!    silent pass. `cargo test -- --ignored` runs it and it fails loudly
//!    today (BLOCKED-MODULE-EXEC).
//!
//! Runtime seam: wasmi 2.0.0 has no EH support; wasmtime 48 removed legacy
//! EH; V8/Node 26 rejects the module-level legacy+exnref EH mix (and traps
//! `function signature mismatch` in a duckdb vtable `call_indirect` even on
//! the exnref-translated module); wabt 1.0.42 and Binaryen 133 cannot run
//! modules. Chicago: real modules, real CSV-injected corpus file, real
//! frozen-duckdb engine. No mocks.
#![cfg(all(feature = "abi", not(target_arch = "wasm32")))]

use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};

use serde_json::{Value, json};

// --------------------------------------------------------------- module location

/// The wasm artifact under test: `GRAPHLAW_WASM` overrides, else the lane
/// build output. No implicit build: the link needs `WASI_SDK_PATH`, so the
/// court names the command instead of hiding it.
fn wasm_path() -> PathBuf {
    if let Some(p) = std::env::var_os("GRAPHLAW_WASM") {
        return p.into();
    }
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let target = match std::env::var_os("CARGO_TARGET_DIR") {
        Some(dir) if !dir.is_empty() => PathBuf::from(dir),
        _ => root.join("target-lane125"),
    };
    let artifact = target.join("wasm32-wasip1/wasm/graphlaw_wasm.wasm");
    assert!(
        artifact.is_file(),
        "wasm artifact missing: {} — build it with `WASI_SDK_PATH=<wasi-sdk> \
         cargo build -p graphlaw-wasm --target wasm32-wasip1 --profile wasm`",
        artifact.display()
    );
    artifact
}

/// One Node driver process wrapping a compiled module (line-delimited JSON
/// request/response over stdio; single JSON documents, no binary framing, so
/// the latin1/packet-N framing law is not in play).
struct Driver {
    child: Child,
    stdin: ChildStdin,
    stdout: BufReader<ChildStdout>,
}

impl Driver {
    fn spawn(module: &PathBuf) -> Self {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let mut child = Command::new("node")
            // V8 otherwise refuses to COMPILE the module-level legacy+exnref
            // EH mix; with the flag it compiles and traps later (see rung C).
            .arg("--wasm-allow-mixed-eh-for-testing")
            .arg(root.join("tests/fixtures/l125_wasm_driver.mjs"))
            .arg(module)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .spawn()
            .expect("spawn node driver");
        let stdin = child.stdin.take().expect("driver stdin");
        let stdout = BufReader::new(child.stdout.take().expect("driver stdout"));
        Driver {
            child,
            stdin,
            stdout,
        }
    }

    fn request_bytes(&mut self, body: Vec<u8>) -> Vec<u8> {
        self.stdin
            .write_all(&body)
            .and_then(|()| self.stdin.write_all(b"\n"))
            .and_then(|()| self.stdin.flush())
            .expect("write request frame");
        let mut line = String::new();
        self.stdout
            .read_line(&mut line)
            .expect("read response frame");
        assert!(line.ends_with('\n'), "driver closed mid-frame: {line}");
        line.into_bytes()
    }

    fn request(&mut self, req: &Value) -> Value {
        let out = self.request_bytes(req.to_string().into_bytes());
        serde_json::from_slice(&out).expect("driver response is JSON")
    }
}

impl Drop for Driver {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

// --------------------------------------------------------------- fixture

/// A temp CSV corpus; deleted on drop.
struct CsvFixture {
    path: std::path::PathBuf,
}

impl CsvFixture {
    fn write(rows: &[String]) -> Self {
        static SEQ: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
        let n = SEQ.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let mut path = std::env::temp_dir();
        path.push(format!("l125_duckdb_wired_{pid}_{n}.csv", pid = std::process::id()));
        std::fs::write(&path, "batch_id,cost,latency\n").expect("write csv header");
        for r in rows {
            use std::io::Write as _;
            let mut f = std::fs::OpenOptions::new()
                .append(true)
                .open(&path)
                .expect("open csv");
            writeln!(f, "{r}").expect("append csv row");
        }
        CsvFixture { path }
    }

    fn glob(&self) -> String {
        self.path.display().to_string()
    }
}

impl Drop for CsvFixture {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.path);
    }
}

fn rows() -> Vec<String> {
    vec![
        "batch-001,120,450".into(),
        "batch-001,80,600".into(),
        "batch-002,250,720".into(),
    ]
}

/// The aggregation AST document (same shape as tests/wasm_duckdb_op.rs's).
fn agg_turtle(threshold: i64) -> String {
    format!(
        r#"@prefix sql: <http://graphlaw.org/sql#> .
@prefix xsd: <http://www.w3.org/2001/XMLSchema#> .
@prefix ex: <https://example.org/queries/> .
ex:q1 a sql:SelectStatement ;
    sql:statementIndex 1 ;
    sql:fromTable [ sql:tableName "telemetry" ] ;
    sql:selectList (
        [ sql:columnName "batch_id" ]
        [ sql:aliasName "total_cost" ; sql:expression [
            sql:function <http://graphlaw.org/functions#SUM> ;
            sql:argument [ sql:columnName "cost" ] ] ]
    ) ;
    sql:whereClause [
        sql:operator sql:GreaterThan ;
        sql:leftOperand [ sql:columnName "latency" ] ;
        sql:rightOperand [ sql:literalValue "{threshold}"^^xsd:integer ]
    ] ;
    sql:groupBy ( [ sql:columnName "batch_id" ] ) .
"#
    )
}

const Q1: &str = "https://example.org/queries/q1";
const SUBJECT: &str = "https://example.org/data";
const VOCAB: &str = "https://example.org/vocab";

fn exec_request(csv: &CsvFixture, threshold: i64) -> Value {
    json!({
        "op": "duckdb",
        "mode": "exec",
        "views": [{"name": "telemetry", "glob": csv.glob()}],
        "query_ast": Q1,
        "ast": {"turtle": agg_turtle(threshold)},
        "subject_prefix": SUBJECT,
        "vocab_prefix": VOCAB,
        "params": [threshold],
    })
}

fn call_native(req: &Value) -> Value {
    serde_json::from_slice(&graphlaw::abi::call(req.to_string().as_bytes()))
        .expect("native response is JSON")
}

// --------------------------------------------------------------- rung A: static

/// The freshly built module carries the duckdb dispatch: the frozen-duckdb
/// engine is linked (archive-scale module) and the op's compiled-in refusal
/// strings are in the data section.
#[test]
fn built_module_carries_the_duckdb_dispatch_statically() {
    let module = wasm_path();
    let bytes = std::fs::read(&module).expect("module bytes");
    assert!(
        bytes.len() > 40_000_000,
        "module is not the duckdb-linked build ({} B): {}",
        bytes.len(),
        module.display()
    );
    for needle in [
        &b"missing string field `mode`"[..],
        b"unknown mode",
        b"SELECT batch_id",
    ] {
        assert!(
            bytes.windows(needle.len()).any(|w| w == needle),
            "dispatch string {needle:?} absent from {}",
            module.display()
        );
    }
}

// ------------------------------------------------------- rung B: executed before/after

/// BEFORE/AFTER witness pair, both EXECUTED through the same driver:
/// - the shipped pre-wiring artifact (`GRAPHLAW_WASM=priv/graphlaw.wasm`)
///   answers `unknown op \`duckdb\`` (L115's witness), while
/// - the native `graphlaw::abi::call` — the exact dispatch the freshly built
///   module compiles — executes the op end to end.
///   The module-internal execution itself is rung C (currently BLOCKED).
#[test]
fn before_state_refusal_and_after_state_native_dispatch() {
    // AFTER: native dispatch (the function compiled into the module).
    let csv = CsvFixture::write(&rows());
    let ex = call_native(&exec_request(&csv, 500));
    assert_eq!(ex["ok"], Value::Bool(true), "exec refused: {ex}");
    assert_eq!(ex["rows_emitted"], 2, "{ex}");
    let receipt = ex["receipt"].as_str().expect("receipt IRI").to_string();
    assert!(receipt.starts_with(SUBJECT), "receipt IRI: {receipt}");
    assert!(ex["query_hash"].as_str().is_some(), "{ex}");
    let cursor = ex["cursor"].as_str().expect("cursor id").to_string();
    let s1 = call_native(&json!({
        "op": "duckdb", "mode": "stream", "cursor": cursor, "max_rows": 1
    }));
    assert_eq!(s1["ok"], Value::Bool(true), "{s1}");
    assert_eq!(s1["rows"], 1);
    let nt = s1["nquads"].as_str().expect("nquads");
    assert!(nt.contains("<https://example.org/data/row_1>"), "{nt}");
    let c = call_native(&json!({"op": "duckdb", "mode": "close", "cursor": cursor}));
    assert_eq!(c["ok"], Value::Bool(true));

    // No-mode twin: typed refusal naming `mode`, never a trap.
    let nomode = call_native(&json!({"op": "duckdb"}));
    assert_eq!(nomode["ok"], Value::Bool(false), "{nomode}");
    assert_eq!(nomode["error"]["kind"], "DuckDb", "{nomode}");
    let msg = nomode["error"]["message"].as_str().unwrap_or_default();
    assert!(msg.contains("mode"), "refusal must name `mode`: {nomode}");
}

/// The L115 before-state, executed: the shipped pre-wiring artifact answers
/// the duckdb request with the typed unknown-op refusal. Skipped (loud) when
/// the artifact is absent or has been re-pinned past the pre-wiring build.
#[test]
fn shipped_pre_wiring_artifact_refuses_duckdb_op() {
    let shipped = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("priv/graphlaw.wasm");
    let Ok(bytes) = std::fs::read(&shipped) else {
        panic!("BLOCKED-BEFORE-WITNESS: {} missing", shipped.display());
    };
    if bytes.len() > 40_000_000 {
        panic!(
            "OBSOLETE-WITNESS: {} has been re-pinned to a duckdb-linked build; \
             the before-state witness lives in the receipt now",
            shipped.display()
        );
    }
    let mut d = Driver::spawn(&shipped);
    let out = d.request(&json!({"op": "duckdb", "mode": "close", "cursor": "x"}));
    assert_eq!(out["ok"], Value::Bool(false), "{out}");
    assert_eq!(out["error"]["kind"], "Unsupported", "{out}");
    let msg = out["error"]["message"].as_str().unwrap_or_default();
    assert!(msg.contains("unknown op"), "{out}");
}

// --------------------------------------------------- rung C: module execution

/// The execution rung: the duckdb op driven through the compiled module's
/// gl_alloc/gl_call/gl_free ABI. Currently BLOCKED (typed) at the runtime
/// seam — see the module header and the receipt for the runtime matrix. Run
/// with `cargo test -- --ignored` to surface the live blocker; flip this to
/// un-ignored the day a runtime executes the module.
#[test]
#[ignore = "BLOCKED-MODULE-EXEC: no local runtime executes the duckdb-linked \
           module (wasmi 2.0.0: no EH; wasmtime 48: legacy EH removed; V8 26: \
           EH-dialect mix rejected, and `function signature mismatch` in a \
           duckdb vtable call_indirect even on the exnref-translated module)"]
fn wasm_module_executes_duckdb_op() {
    let csv = CsvFixture::write(&rows());
    let req = exec_request(&csv, 500);
    let mut d = Driver::spawn(&wasm_path());
    let ex = d.request(&req);
    assert_eq!(ex["ok"], Value::Bool(true), "module exec refused: {ex}");
    assert_eq!(ex["rows_emitted"], 2, "{ex}");
    let receipt = ex["receipt"].as_str().expect("receipt IRI").to_string();
    assert!(receipt.starts_with(SUBJECT), "{receipt}");
    let native = graphlaw::abi::call(req.to_string().as_bytes());
    let wasm_bytes = d.request_bytes(req.to_string().into_bytes());
    assert_eq!(
        native, wasm_bytes,
        "native and module responses diverge:\nnative: {}\nwasm:   {}",
        String::from_utf8_lossy(&native),
        String::from_utf8_lossy(&wasm_bytes)
    );
}
