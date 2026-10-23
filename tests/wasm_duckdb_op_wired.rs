//! Lane L125 court: `duckdb` is an in-module capability. `graphlaw-wasm` is
//! one unit: it links frozen-duckdb (a normal dependency, never feature-gated)
//! and the `duckdb` dispatch arm compiles for `wasm32-wasip1`. A `duckdb`
//! frame on the module is executed against a real frozen-duckdb connection
//! INSIDE the module — the typed refusals (`DuckDb` kind, mode validation,
//! SQLRDF-17) come from the same code the host runs.
//!
//! Rungs:
//! 1. The built module carries the duckdb dispatch strings in its data
//!    section (`missing string field \`mode\``, `unknown mode`) — static.
//! 2. The module, driven through the Node gl_alloc/gl_call driver, DISPATCHES
//!    the op: mode validation refusals are duckdb-shaped (`DuckDb` kind),
//!    never `unknown op`; capabilities advertise `frozen-duckdb`.
//! 3. Legacy witness: the pre-duckdb shipped artifact (`priv/graphlaw.wasm`,
//!    digest `f149433c…`) still answers `unknown op \`duckdb\`` — pinned and
//!    digest-gated; a re-pinned artifact is a loud SKIP, never a faked pass.
//! 4. Native `graphlaw::abi::call` exec/stream/close unchanged.
//!
//! Runtime note: the module load happens in Node's V8 (wasm exceptions
//! supported natively). wasmi 2.0.0 cannot load the duckdb-linked artifact
//! ("exceptions proposal not enabled"); if the driver cannot instantiate the
//! module the court emits a typed SKIP with that reason rather than failing
// coordinator dev-dep: wasmtime 48 (runtime), wasmtime-wasi 48 — a wasmtime
//! harness should replace the Node driver once the coordinator wires it; the
//! instantiation helper below already returns Result for that switchover.
//!
//! Chicago: real module, real Node WASI driver, real frozen-duckdb in-module.
#![cfg(all(feature = "abi", not(target_arch = "wasm32")))]

use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};

use serde_json::{Value, json};

// --------------------------------------------------------------- module location

/// Digest of the pre-duckdb legacy artifact (`priv/graphlaw.wasm` at the
/// pre-wiring pin). When the shipped artifact's blake3 no longer matches,
/// the legacy witness is obsolete and skips loudly.
const LEGACY_DIGEST: &str = "f149433c6e1a84ec95e961ac92b16b65b28e15d6fd8546beb1779d3b54a02e7e";

/// Candidate fresh-module locations, first existing wins: `GRAPHLAW_WASM`,
/// then `CARGO_TARGET_DIR`, then the lane build root, then the shared root.
/// No implicit build: the link needs `WASI_SDK_PATH`, so the court names the
/// command instead of hiding it.
fn fresh_module_candidates() -> Vec<PathBuf> {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let mut out = Vec::new();
    if let Some(p) = std::env::var_os("GRAPHLAW_WASM") {
        out.push(PathBuf::from(p));
    }
    if let Some(dir) = std::env::var_os("CARGO_TARGET_DIR") {
        if !dir.is_empty() {
            out.push(
                PathBuf::from(dir).join("wasm32-wasip1/wasm/graphlaw_wasm.wasm"),
            );
        }
    }
    out.push(root.join("target-lane10/wasm32-wasip1/wasm/graphlaw_wasm.wasm"));
    out.push(root.join("target/wasm32-wasip1/wasm/graphlaw_wasm.wasm"));
    out
}

fn fresh_module() -> Option<PathBuf> {
    fresh_module_candidates()
        .into_iter()
        .find(|p| p.is_file())
}

fn blake3_hex(path: &Path) -> String {
    let bytes = std::fs::read(path).expect("read wasm artifact for digest");
    blake3::Hasher::new()
        .update(&bytes)
        .finalize()
        .to_string()
}

fn skip(msg: &str) {
    eprintln!("[SKIP:typed] wasm_duckdb_op_wired: {msg}");
}

// --------------------------------------------------------------- driver

/// One Node driver process wrapping a compiled module (line-delimited JSON
/// request/response over stdio; single JSON documents, no binary framing, so
/// the latin1/packet-N framing law is not in play).
struct Driver {
    child: Child,
    stdin: ChildStdin,
    stdout: BufReader<ChildStdout>,
}

impl Driver {
    /// Spawn + probe. `Err(reason)` means the module could not be
    /// instantiated (e.g. an EH/exceptions-proposal import rejection in a
    /// runtime without wasm-exception support) — callers SKIP loudly.
    fn try_spawn(module: &Path) -> Result<Self, String> {
        Self::try_spawn_impl(module, true)
    }

    /// Spawn without the capabilities probe: the legacy pre-duckdb artifact
    /// predates the probe's expectations and stalls on it; the before-state
    /// witness only needs the duckdb request to answer.
    fn try_spawn_unprobed(module: &Path) -> Result<Self, String> {
        Self::try_spawn_impl(module, false)
    }

    fn try_spawn_impl(module: &Path, probe: bool) -> Result<Self, String> {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let mut child = Command::new("node")
            .arg(root.join("tests/fixtures/l125_wasm_driver.mjs"))
            .arg(module)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| format!("spawn node driver failed: {e}"))?;
        let stdin = child.stdin.take().expect("driver stdin");
        let stdout = BufReader::new(child.stdout.take().expect("driver stdout"));
        let stderr = child.stderr.take();
        let mut d = Driver {
            child,
            stdin,
            stdout,
        };
        // Probe: capabilities must answer, else the load failed (skipped
        // for the legacy artifact, which stalls on it).
        let probed = if probe {
            d.request_soft(json!({"op": "capabilities"}))
        } else {
            Some(json!({}))
        };
        match probed {
            Some(_) => {
                // stderr stays open and unread (banner fits the pipe
                // buffer); reading it here would block until node exits.
                std::mem::forget(stderr);
                Ok(d)
            }
            None => {
                let reason = stderr
                    .and_then(|mut e| {
                        use std::io::Read as _;
                        let mut buf = String::new();
                        e.read_to_string(&mut buf).ok()?;
                        Some(buf)
                    })
                    .unwrap_or_default();
                let _ = d.child.kill();
                let _ = d.child.wait();
                Err(format!(
                    "module failed to instantiate under the Node driver \
                     (wasm exceptions/EH import or WASI trap); driver stderr: \
                     {}",
                     reason.trim()
                ))
            }
        }
    }

    fn request_bytes(&mut self, body: Vec<u8>) -> Vec<u8> {
        self.stdin
            .write_all(&body)
            .and_then(|()| self.stdin.write_all(b"\n"))
            .and_then(|()| self.stdin.flush())
            .expect("write request frame");
        let mut line = String::new();
        let n = self.stdout.read_line(&mut line).expect("read response frame");
        assert!(n > 0, "driver closed mid-frame: {line}");
        assert!(line.ends_with('\n'), "driver closed mid-frame: {line}");
        line.into_bytes()
    }

    fn request(&mut self, req: &Value) -> Value {
        let out = self.request_bytes(req.to_string().into_bytes());
        serde_json::from_slice(&out).expect("driver response is JSON")
    }

    /// Non-panicking request for the spawn probe: `None` when the driver
    /// produced no frame (module load failure).
    fn request_soft(&mut self, req: Value) -> Option<Value> {
        self.stdin
            .write_all(req.to_string().as_bytes())
            .and_then(|()| self.stdin.write_all(b"\n"))
            .and_then(|()| self.stdin.flush())
            .ok()?;
        let mut line = String::new();
        let n = self.stdout.read_line(&mut line).ok()?;
        if n == 0 || !line.ends_with('\n') {
            return None;
        }
        serde_json::from_slice(line.as_bytes()).ok()
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
        path.push(format!(
            "l125_duckdb_wired_{pid}_{n}.csv",
            pid = std::process::id()
        ));
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

/// The built module is the duckdb-linked artifact: the duckdb dispatch
/// strings are IN its data section. Skips loudly when no artifact is built
/// or when the only artifact present is the legacy pre-duckdb pin (that
/// witness is covered by `legacy_pin_shipped_artifact_refuses_duckdb_op`).
#[test]
fn built_module_links_the_duckdb_dispatch() {
    let module = match fresh_module() {
        Some(m) => m,
        None => {
            skip(
                "no fresh wasm artifact built — run `WASI_SDK_PATH=<wasi-sdk> cargo \
                 build -p graphlaw-wasm --target wasm32-wasip1 --profile wasm` (or set \
                 GRAPHLAW_WASM)",
            );
            return;
        }
    };
    let bytes = std::fs::read(&module).expect("module bytes");
    if blake3_hex(&module) == LEGACY_DIGEST {
        skip(&format!(
            "{} is still the legacy pre-duckdb pin ({LEGACY_DIGEST}); the fresh \
             duckdb-linked module has not been built — run the wasm build command",
            module.display()
        ));
        return;
    }
    for needle in [&b"missing string field `mode`"[..], b"unknown mode"] {
        assert!(
            bytes.windows(needle.len()).any(|w| w == needle),
            "duckdb dispatch string {needle:?} absent from {}",
            module.display()
        );
    }
}

// ------------------------------------------------------- rung B: native contract

/// Native `graphlaw::abi::call` executes duckdb: exec/stream/close round trip,
/// plus the typed no-mode refusal naming `mode`.
#[test]
fn native_dispatch_exec_stream_close() {
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

// --------------------------------------------------- rung C: module execution

/// The fresh module DISPATCHES `duckdb`: mode validation is duckdb-shaped
/// (`DuckDb` kind naming the offending mode / missing field / SQLRDF-17
/// rule), never `unknown op \`duckdb\``; capabilities advertise
/// `frozen-duckdb`. Typed SKIP when the module cannot instantiate (EH).
#[test]
fn wasm_module_dispatches_duckdb_op() {
    let module = match fresh_module() {
        Some(m) => m,
        None => {
            skip(
                "no fresh wasm artifact built — run `WASI_SDK_PATH=<wasi-sdk> cargo \
                 build -p graphlaw-wasm --target wasm32-wasip1 --profile wasm` (or set \
                 GRAPHLAW_WASM)",
            );
            return;
        }
    };
    if blake3_hex(&module) == LEGACY_DIGEST {
        skip(&format!(
            "{} is the legacy pre-duckdb pin; a duckdb-dispatching module is \
             required for this court — build the fresh module",
            module.display()
        ));
        return;
    }
    let mut d = match Driver::try_spawn(&module) {
        Ok(d) => d,
        Err(reason) => {
            skip(&format!("cannot drive {}: {reason}", module.display()));
            return;
        }
    };

    // 1. No mode at all: duckdb-shaped refusal naming `mode`.
    let nomode = d.request(&json!({"op": "duckdb"}));
    assert_eq!(nomode["ok"], Value::Bool(false), "{nomode}");
    assert_eq!(nomode["error"]["kind"], "DuckDb", "{nomode}");
    let msg = nomode["error"]["message"].as_str().unwrap_or_default();
    assert!(msg.contains("mode"), "refusal must name `mode`: {nomode}");
    assert!(
        !msg.contains("unknown op"),
        "no-mode refusal leaked the unknown-op envelope: {nomode}"
    );

    // 2. Unknown mode: typed refusal echoing the mode, never `unknown op`.
    let badmode = d.request(&json!({"op": "duckdb", "mode": "teleport"}));
    assert_eq!(badmode["ok"], Value::Bool(false), "{badmode}");
    assert_eq!(badmode["error"]["kind"], "DuckDb", "{badmode}");
    let msg = badmode["error"]["message"].as_str().unwrap_or_default();
    assert!(
        msg.contains("teleport") && msg.contains("unknown mode"),
        "refusal must name the offending mode: {badmode}"
    );
    assert!(
        !msg.contains("unknown op"),
        "unknown-mode refusal leaked the unknown-op envelope: {badmode}"
    );

    // 3. Exec without required fields: typed refusal naming the missing
    //    field or the SQLRDF-17 raw-SQL rule.
    let bare_exec = d.request(&json!({"op": "duckdb", "mode": "exec"}));
    assert_eq!(bare_exec["ok"], Value::Bool(false), "{bare_exec}");
    assert_eq!(bare_exec["error"]["kind"], "DuckDb", "{bare_exec}");
    let msg = bare_exec["error"]["message"].as_str().unwrap_or_default();
    assert!(
        msg.contains("ast") || msg.contains("SQLRDF-17") || msg.contains("missing"),
        "exec refusal must name the missing field or SQLRDF-17: {bare_exec}"
    );
    assert!(
        !msg.contains("unknown op"),
        "bare-exec refusal leaked the unknown-op envelope: {bare_exec}"
    );

    // 4. Capabilities: the module now advertises frozen-duckdb, matching host.
    let caps = d.request(&json!({"op": "capabilities"}));
    let wasm_auth: Vec<&str> = caps["authorities"]
        .as_array()
        .expect("wasm authorities")
        .iter()
        .filter_map(|a| a["authority"].as_str())
        .collect();
    assert!(
        wasm_auth.contains(&"frozen-duckdb"),
        "wasm capabilities do not advertise frozen-duckdb: {caps}"
    );
    let host = call_native(&json!({"op": "capabilities"}));
    let host_auth: Vec<&str> = host["authorities"]
        .as_array()
        .expect("host authorities")
        .iter()
        .filter_map(|a| a["authority"].as_str())
        .collect();
    assert!(
        host_auth.contains(&"frozen-duckdb"),
        "host capabilities dropped frozen-duckdb: {host}"
    );
}

// --------------------------------------------------- rung D: legacy witness

/// The L115/L125 before-state, pinned: the pre-duckdb shipped artifact
/// answers the duckdb request with the typed unknown-op refusal. Digest-
/// gated — when `priv/graphlaw.wasm` is re-pinned past the pre-wiring
/// build, this skips loudly instead of faking a pass or failing falsely.
#[test]
fn legacy_pin_shipped_artifact_refuses_duckdb_op() {
    let shipped = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("priv/graphlaw.wasm");
    let bytes = match std::fs::read(&shipped) {
        Ok(b) => b,
        Err(_) => {
            skip(&format!(
                "legacy witness artifact {} missing — before-state refusal lives \
                 in the receipt now",
                shipped.display()
            ));
            return;
        }
    };
    let digest = blake3::Hasher::new().update(&bytes).finalize().to_string();
    if digest != LEGACY_DIGEST {
        skip(&format!(
            "{} digest {digest} no longer matches the legacy pre-duckdb pin \
             ({LEGACY_DIGEST}); artifact re-pinned past the before-state",
            shipped.display()
        ));
        return;
    }
    let mut d = match Driver::try_spawn_unprobed(&shipped) {
        Ok(d) => d,
        Err(reason) => {
            skip(&format!("cannot drive legacy artifact: {reason}"));
            return;
        }
    };
    let out = d.request(&json!({"op": "duckdb", "mode": "close", "cursor": "x"}));
    assert_eq!(out["ok"], Value::Bool(false), "{out}");
    assert_eq!(out["error"]["kind"], "Unsupported", "{out}");
    let msg = out["error"]["message"].as_str().unwrap_or_default();
    assert!(msg.contains("unknown op"), "{out}");
}
