//! Case 4 — UBO sanctions gate court (FinCEN CDD / 6AMLD): recursive
//! transitive-ownership resolution via the in-module `datalog` op (same
//! fixpoint engine the `wasm_abi.rs` datalog court exercises over gl_call),
//! feeding a host-side account-creation gate over an in-module DuckDB
//! session (the `tests/wasm_fibo_fixtures_eh.rs` harness pattern).
//!
//! Schema (per session):
//! - ubo_owns(child VARCHAR, parent VARCHAR, pct DECIMAL)
//! - ubo_sanctioned(party VARCHAR)
//! - ubo_accounts(entity VARCHAR, status VARCHAR)
//!
//! Data: C→B 60%, B→SDN 50% (transitive 30% ≥ 25% threshold, SDN
//! sanctioned) and the lawful chain D→E 20%, E→F 30% (6% < 25%).
//!
//! Courts:
//! 1. Sanctioned: the datalog op resolves C's transitive ownership to the
//!    sanctioned party at the same fixpoint; readback of the ownership table
//!    proves effective control 0.30 ≥ 0.25 → the host gate REFUSES the
//!    account-creation document (it is never sent to the module) →
//!    event vector [Request_Received, Ownership_Resolved,
//!    Sanctioned_Control_Detected, Refused, State_Preserved] and readback
//!    proves `ubo_accounts` has zero rows.
//! 2. Lawful: D's transitive control over F is 6% < 25% → the gate admits
//!    the CREATE account doc; it executes, the account row materializes,
//!    the receipt seals (response carries `receipt` + `query_hash`) →
//!    event vector [Request_Received, Ownership_Resolved, Cleared, Admitted,
//!    Account_Materialized, Receipt_Sealed].
//! 3. Anti-vacuity: a broken gate (skips the ownership traversal, admits C)
//!    → C's row lands in `ubo_accounts` → the conformance check flags the
//!    missing Refused/State_Preserved events and the illegal
//!    materialization → the detection must fire. The same check must NOT
//!    fire on courts 1 and 2 (non-vacuity of the checker itself).
//!
//! Chicago: real module, real frozen-duckdb inside it, real table readbacks;
//! no mocks. Typed loud SKIP discipline as in the FIBO fixtures court.

#![cfg(all(feature = "abi", not(target_arch = "wasm32")))]

use std::path::PathBuf;
use std::sync::Mutex;

use serde_json::{Value, json};
use wasmtime::{Config, Engine, Instance, Linker, Memory, Module, Store, TypedFunc};
use wasmtime_wasi::WasiCtxBuilder;
use wasmtime_wasi::p1::{WasiP1Ctx, add_to_linker_sync};

static ENGINE_LOCK: Mutex<()> = Mutex::new(());

fn engine_lock() -> std::sync::MutexGuard<'static, ()> {
    ENGINE_LOCK.lock().unwrap_or_else(std::sync::PoisonError::into_inner)
}

// --------------------------------------------------------------- harness
// (verbatim pattern from tests/wasm_fibo_fixtures_eh.rs)

fn wasm_path() -> Option<PathBuf> {
    if let Some(p) = std::env::var_os("GRAPHLAW_WASM") {
        let path = PathBuf::from(p);
        if !path.is_file() {
            panic!(
                "GRAPHLAW_WASM is set but the artifact is missing: {}",
                path.display()
            );
        }
        return Some(path);
    }
    let artifact = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("target/wasm32-wasip1/wasm/graphlaw_wasm.wasm");
    if artifact.is_file() {
        return Some(artifact);
    }
    None
}

#[derive(Debug)]
enum HostError {
    Trap(String),
    Fatal(String),
}

struct Host {
    store: Store<WasiP1Ctx>,
    memory: Memory,
    alloc: TypedFunc<u32, u32>,
    free: TypedFunc<(u32, u32), ()>,
    call: TypedFunc<(u32, u32), u64>,
}

impl Host {
    fn new(bytes: &[u8]) -> Result<Self, HostError> {
        let fail = |e: wasmtime::Error, stage: &'static str| {
            let s = format!("{stage}: {e}");
            if e.to_string().contains("uninitialized element") {
                HostError::Trap(s)
            } else {
                HostError::Fatal(s)
            }
        };
        let mut cfg = Config::new();
        cfg.wasm_exceptions(true);
        let engine = Engine::new(&cfg).map_err(|e| HostError::Fatal(format!("engine: {e}")))?;
        let module =
            Module::new(&engine, bytes).map_err(|e| HostError::Fatal(format!("module: {e}")))?;
        for i in module.imports() {
            if i.module() != "wasi_snapshot_preview1" {
                return Err(HostError::Fatal(format!(
                    "unexpected host import {}::{}",
                    i.module(),
                    i.name()
                )));
            }
        }
        let mut store = Store::new(
            &engine,
            WasiCtxBuilder::new()
                .inherit_stdio()
                .inherit_env()
                .build_p1(),
        );
        let mut linker = <Linker<WasiP1Ctx>>::new(&engine);
        add_to_linker_sync(&mut linker, |ctx| ctx)
            .map_err(|e| HostError::Fatal(format!("wasi linker: {e}")))?;
        let instance: Instance = linker
            .instantiate(&mut store, &module)
            .map_err(|e| fail(e, "instantiate"))?;
        if let Ok(init) = instance.get_typed_func::<(), ()>(&mut store, "_initialize") {
            init.call(&mut store, ()).map_err(|e| fail(e, "_initialize"))?;
        }
        let memory = instance
            .get_memory(&mut store, "memory")
            .ok_or_else(|| HostError::Fatal("module does not export memory".into()))?;
        let missing = |name: &'static str, e: wasmtime::Error| {
            HostError::Fatal(format!("missing or mistyped export {name}: {e}"))
        };
        Ok(Host {
            alloc: instance
                .get_typed_func::<u32, u32>(&mut store, "gl_alloc")
                .map_err(|e| missing("gl_alloc", e))?,
            free: instance
                .get_typed_func::<(u32, u32), ()>(&mut store, "gl_free")
                .map_err(|e| missing("gl_free", e))?,
            call: instance
                .get_typed_func::<(u32, u32), u64>(&mut store, "gl_call")
                .map_err(|e| missing("gl_call", e))?,
            store,
            memory,
        })
    }

    fn request_bytes(&mut self, body: &[u8]) -> Result<Vec<u8>, HostError> {
        let ptr = self
            .alloc
            .call(&mut self.store, body.len() as u32)
            .map_err(|e| fail_classify("gl_alloc", e))?;
        if ptr == 0 {
            return Err(HostError::Fatal("gl_alloc refused a pre-checked request".into()));
        }
        self.memory
            .write(&mut self.store, ptr as usize, body)
            .map_err(|e| HostError::Fatal(format!("memory write: {e}")))?;
        let packed = self
            .call
            .call(&mut self.store, (ptr, body.len() as u32))
            .map_err(|e| fail_classify("gl_call", e))?;
        let (out_ptr, out_len) = ((packed >> 32) as u32, (packed & 0xffff_ffff) as u32);
        let mut out = vec![0u8; out_len as usize];
        self.memory
            .read(&self.store, out_ptr as usize, &mut out)
            .map_err(|e| HostError::Fatal(format!("memory read: {e}")))?;
        self.free
            .call(&mut self.store, (out_ptr, out_len))
            .map_err(|e| fail_classify("gl_free", e))?;
        Ok(out)
    }

    fn request(&mut self, req: &Value) -> Result<Value, HostError> {
        let out = self.request_bytes(req.to_string().as_bytes())?;
        serde_json::from_slice(&out)
            .map_err(|e| HostError::Fatal(format!("response is not JSON: {e}")))
    }

    fn exec(&mut self, req: &Value) -> Value {
        let r = self.request(req).expect("request transport");
        assert_eq!(r["ok"], json!(true), "exec request failed: {req} -> {r}");
        r
    }

    fn stream_all(&mut self, session: &str, cursor: &str) -> String {
        let mut body = String::new();
        loop {
            let page = self.exec(&json!({
                "op": "duckdb", "mode": "stream", "session": session,
                "cursor": cursor, "max_rows": 100
            }));
            if let Some(b) = page["nquads"].as_str() {
                body.push_str(b);
            }
            if page["more"].as_bool() != Some(true) {
                return body;
            }
        }
    }
}

fn fail_classify(stage: &str, e: wasmtime::Error) -> HostError {
    let s = format!("{stage}: {e}");
    if e.to_string().contains("uninitialized element") {
        HostError::Trap(s)
    } else {
        HostError::Fatal(s)
    }
}

fn try_host() -> Result<Host, String> {
    let path = wasm_path().ok_or_else(|| "artifact-missing".to_string())?;
    let bytes = std::fs::read(&path).map_err(|e| format!("fatal: read {}: {e}", path.display()))?;
    match Host::new(&bytes) {
        Ok(h) => Ok(h),
        Err(HostError::Trap(e)) => Err(format!("trap: {e}")),
        Err(HostError::Fatal(e)) => Err(format!("fatal: {e}")),
    }
}

fn host_or_skip(test_name: &str) -> Option<Host> {
    match try_host() {
        Ok(h) => Some(h),
        Err(reason) => {
            let kind = if reason == "artifact-missing" {
                "artifact not built"
            } else if reason.starts_with("trap:") {
                "first gl_call trapped on uninitialized element (stale/mismatched artifact)"
            } else {
                "host construction failed"
            };
            eprintln!(
                "// SKIP(typed) [{test_name}]: {kind} — reason: {reason} — \
                 suite pending-artifact; build with `WASI_SDK_PATH=<wasi-sdk> \
                 cargo build -p graphlaw-wasm --target wasm32-wasip1 --profile wasm`"
            );
            None
        }
    }
}

// ---------------------------------------------------------------- schema

const NS: &str = "http://example.org/ubo#";
const SQL: &str = "http://graphlaw.org/sql#";
const XSD: &str = "http://www.w3.org/2001/XMLSchema#";

fn create_doc(table: &str, columns: &[(&str, &str)]) -> String {
    let cols: Vec<String> = columns
        .iter()
        .map(|(n, t)| format!("[ sql:columnName \"{n}\" ; sql:columnType \"{t}\" ]"))
        .collect();
    format!(
        r#"@prefix sql: <{SQL}> .
@prefix ex: <{NS}> .
ex:stmt a sql:CreateStatement ;
    sql:statementIndex 0 ;
    sql:tableDefinition [
        sql:tableName "{table}" ;
        sql:columns ( {} ) ; ] ."#,
        cols.join("\n            ")
    )
}

/// One INSERT doc: pairs are (column, value, datatype-opt) in reader order.
fn insert_doc(table: &str, pairs: &[(&str, &str, Option<&str>)]) -> String {
    let mut s = format!(
        r#"@prefix sql: <{SQL}> .
@prefix xsd: <{XSD}> .
@prefix ex: <{NS}> .
ex:stmt a sql:InsertStatement ;
    sql:statementIndex 1 ;
    sql:targetTable "{table}" ;
"#
    );
    let last = pairs.len() - 1;
    for (i, (col, val, dt)) in pairs.iter().enumerate() {
        let sep = if i == last { " ." } else { " ;" };
        match dt {
            Some(d) => s.push_str(&format!("    sql:assignColumn \"{col}\" ;\n    sql:assignValue \"{val}\"^^xsd:{d}{sep}\n")),
            None => s.push_str(&format!("    sql:assignColumn \"{col}\" ;\n    sql:assignValue \"{val}\"{sep}\n")),
        }
    }
    s
}

fn select_doc(table: &str, columns: &[&str]) -> String {
    let list: Vec<String> = columns
        .iter()
        .map(|c| format!("[ sql:columnName \"{c}\" ]"))
        .collect();
    format!(
        r#"@prefix sql: <{SQL}> .
@prefix ex: <{NS}> .
ex:stmt a sql:SelectStatement ;
    sql:statementIndex 1 ;
    sql:fromTable [ sql:tableName "{table}" ] ;
    sql:selectList ( {} ) ."#,
        list.join(" ")
    )
}

fn exec_ast(h: &mut Host, session: &str, turtle: &str) -> Value {
    h.exec(&json!({
        "op": "duckdb", "mode": "exec", "session": session,
        "query_ast": "urn:ubo:stmt",
        "subject_prefix": "urn:row/",
        "vocab_prefix": "urn:v/",
        "ast": {"turtle": turtle, "dialect": "duckdb"}
    }))
}

/// Opens a session and materializes the three-table UBO schema plus the
/// ownership + sanctioned-parties rows. Returns () — readbacks are per-court.
fn seed_session(h: &mut Host, session: &str) {
    h.exec(&json!({"op": "duckdb", "mode": "open", "session": session}));
    exec_ast(h, session, &create_doc("ubo_owns", &[("child", "VARCHAR"), ("parent", "VARCHAR"), ("pct", "DECIMAL")]));
    exec_ast(h, session, &create_doc("ubo_sanctioned", &[("party", "VARCHAR")]));
    exec_ast(h, session, &create_doc("ubo_accounts", &[("entity", "VARCHAR"), ("status", "VARCHAR")]));
    let edges: Vec<(&str, &str, &str, &str, &str)> = vec![
        ("C", "B", "0.60", "decimal", "ubo_owns"),
        ("B", "SDN", "0.50", "decimal", "ubo_owns"),
        ("D", "E", "0.20", "decimal", "ubo_owns"),
        ("E", "F", "0.30", "decimal", "ubo_owns"),
    ];
    for (child, parent, pct, dt, table) in edges {
        let doc = insert_doc(table, &[("child", child, None), ("parent", parent, None), ("pct", pct, Some(dt))]);
        exec_ast(h, session, &doc);
    }
    let sdn = insert_doc("ubo_sanctioned", &[("party", "SDN", None)]);
    exec_ast(h, session, &sdn);
}

/// A full-table readback: rows_emitted + the streamed ntriples body.
fn read_table(h: &mut Host, session: &str, table: &str, columns: &[&str]) -> (usize, String) {
    let sel = exec_ast(h, session, &select_doc(table, columns));
    let n = sel["rows_emitted"].as_u64().unwrap_or(0) as usize;
    let cur = sel["cursor"].as_str().expect("select cursor").to_string();
    (n, h.stream_all(session, &cur))
}

/// Extracts rows of (child, parent, f64 pct) from the streamed ntriples body
/// of a `ubo_owns` select. Triples are `<row-subj> <urn:v/<col>> <obj> .`
/// lines; the column is the predicate's last path segment.
fn parse_edges(body: &str) -> Vec<(String, String, f64)> {
    use std::collections::HashMap;
    let mut rows: HashMap<String, (Option<String>, Option<String>, Option<f64>)> = HashMap::new();
    for line in body.lines() {
        let line = line.trim();
        if !line.starts_with('<') {
            continue;
        }
        let parts: Vec<&str> = line.splitn(3, ' ').collect();
        if parts.len() < 3 {
            continue;
        }
        let subj = parts[0];
        let pred = parts[1].trim_matches(|c| c == '<' || c == '>');
        let col = pred.rsplit(['/', '#']).next().unwrap_or(pred);
        let obj = parts[2].trim_end_matches(" .").trim();
        let value = if let Some(rest) = obj.strip_prefix('"') {
            rest.split("\"^^").next().unwrap_or(rest).trim_end_matches('"').to_string()
        } else {
            obj.to_string()
        };
        let e = rows.entry(subj.to_string()).or_default();
        match col {
            "child" => e.0 = Some(value.clone()),
            "parent" => e.1 = Some(value.clone()),
            "pct" => e.2 = value.parse::<f64>().ok(),
            _ => {}
        }
    }
    rows.into_values()
        .filter_map(|(c, p, pct)| Some((c?, p?, pct?)))
        .collect()
}

/// Effective transitive control pct from `root` to `target` via DFS product
/// over the readback edges (ties the gate to the table, not to the seed
/// literals).
fn effective_pct(edges: &[(String, String, f64)], root: &str, target: &str) -> f64 {
    fn dfs(edges: &[(String, String, f64)], cur: &str, target: &str, acc: f64, seen: &mut Vec<String>) -> f64 {
        if cur == target {
            return acc;
        }
        if seen.iter().any(|s| s == cur) {
            return 0.0;
        }
        seen.push(cur.to_string());
        let mut best = 0.0;
        for (c, p, pct) in edges {
            if c == cur {
                let v = dfs(edges, p, target, acc * pct, seen);
                if v > best {
                    best = v;
                }
            }
        }
        best
    }
    dfs(edges, root, target, 1.0, &mut Vec::new())
}

/// In-module datalog op: recursive transitive-ownership resolution at the
/// same least fixpoint (direct + recursive rule, exactly the
/// `datalog_computes_a_least_fixpoint` wire shape).
fn transitive_control(h: &mut Host, edges: &[(String, String, f64)]) -> Vec<(String, String)> {
    let facts: Vec<Value> = edges
        .iter()
        .map(|(c, p, _)| json!(["urn:ubo:e/".to_string() + c, "urn:ubo:owns", "urn:ubo:e/".to_string() + p]))
        .collect();
    let r = h
        .request(&json!({"op": "datalog",
            "rules": [
                {"head": ["?x", "urn:ubo:control", "?y"], "body": [["?x", "urn:ubo:owns", "?y"]]},
                {"head": ["?x", "urn:ubo:control", "?z"], "body": [["?x", "urn:ubo:control", "?y"], ["?y", "urn:ubo:owns", "?z"]]}
            ],
            "facts": facts}))
        .expect("datalog transport");
    assert_eq!(r["ok"], json!(true), "datalog refused: {r}");
    r["facts"]
        .as_array()
        .expect("datalog facts array")
        .iter()
        .filter(|f| f[1] == json!("urn:ubo:control"))
        .map(|f| {
            (
                f[0].as_str().unwrap().trim_start_matches("urn:ubo:e/").to_string(),
                f[2].as_str().unwrap().trim_start_matches("urn:ubo:e/").to_string(),
            )
        })
        .collect()
}

// ----------------------------------------------------------- gate model

/// The host gate's event log for one account-creation decision.
type Events = Vec<&'static str>;

struct Decision {
    admitted: bool,
    events: Events,
}

/// The lawful gate: readback → datalog traversal → threshold + sanctions
/// check → admit or refuse (never sends the doc when refusing).
fn run_gate(h: &mut Host, session: &str, applicant: &str) -> Decision {
    let mut events: Events = vec!["Request_Received"];
    let (_, owns_body) = read_table(h, session, "ubo_owns", &["child", "parent", "pct"]);
    let (_, sdn_body) = read_table(h, session, "ubo_sanctioned", &["party"]);
    let edges = parse_edges(&owns_body);
    let sanctioned = sdn_body.contains("\"SDN\"");
    events.push("Ownership_Resolved");

    let control = transitive_control(h, &edges);
    let hits: Vec<(String, String)> = control
        .iter()
        .filter(|(_, target)| sanctioned && sdn_body.contains(&format!("\"{target}\"")))
        .cloned()
        .collect();

    let mut blocked = false;
    for (who, sdn_party) in &hits {
        if who != applicant {
            continue; // the gate scopes the traversal to the applicant
        }
        let pct = effective_pct(&edges, who, sdn_party);
        if pct >= 0.25 {
            blocked = true;
        }
    }

    if blocked {
        events.extend(["Sanctioned_Control_Detected", "Refused", "State_Preserved"]);
        return Decision { admitted: false, events };
    }
    events.push("Cleared");
    events.push("Admitted");
    Decision { admitted: true, events }
}

/// The BROKEN gate (anti-vacuity operator): skips the ownership traversal
/// entirely and admits unconditionally. Its event log carries
/// Request_Received → (claimed) Cleared/Admitted with no Ownership_Resolved,
/// no Refused, no State_Preserved.
fn run_broken_gate(h: &mut Host, session: &str, applicant: &str) -> Decision {
    let _ = (h, session, applicant);
    Decision {
        admitted: true,
        events: vec!["Request_Received", "Cleared", "Admitted"],
    }
}

/// Conformance check over a decision + real accounts readback: a sanctioned
/// applicant MUST end in Refused/State_Preserved with zero account rows; an
/// admitted applicant MUST have produced the full lawful event vector with
/// the account materialized. Returns the list of violations (empty = pass).
fn conformance_check(d: &Decision, events: &[&str], accounts_rows: usize, applicant_was_sanctioned: bool) -> Vec<String> {
    let mut v = Vec::new();
    let log = &d.events;
    if applicant_was_sanctioned {
        if !log.contains(&"Refused") || !log.contains(&"State_Preserved") {
            v.push(format!(
                "sanctioned applicant {applicant_was_sanctioned}: event log missing Refused/State_Preserved; got {log:?}"
            ));
        }
        if accounts_rows != 0 {
            v.push(format!(
                "illegal materialization: sanctioned applicant admitted, accounts table has {accounts_rows} row(s)"
            ));
        }
        if log.contains(&"Admitted") {
            v.push("sanctioned applicant carried an Admitted event".into());
        }
    } else if d.admitted {
        for req in ["Ownership_Resolved", "Cleared", "Admitted", "Account_Materialized", "Receipt_Sealed"] {
            if !events.contains(&req) && !log.contains(&req) {
                v.push(format!("admitted account missing required event {req}"));
            }
        }
    }
    v
}

// ------------------------------------------------------------- courts

const THRESHOLD: f64 = 0.25;

#[test]
fn tc_ubo_01_sanctioned_transitive_control_refused_state_preserved() {
    let _guard = engine_lock();
    let Some(mut h) = host_or_skip("TC-UBO-01") else { return };
    const SESSION: &str = "ubo-01";
    seed_session(&mut h, SESSION);

    // Datalog fixpoint proves the transitive edge C→SDN exists in-module.
    let (_, owns_body) = read_table(&mut h, SESSION, "ubo_owns", &["child", "parent", "pct"]);
    let edges = parse_edges(&owns_body);
    assert_eq!(edges.len(), 4, "four ownership edges read back: {edges:?}");
    let control = transitive_control(&mut h, &edges);
    assert!(
        control.iter().any(|(a, b)| a == "C" && b == "SDN"),
        "datalog fixpoint must resolve C→SDN transitively: {control:?}"
    );
    assert!(
        !control.iter().any(|(a, b)| a == "D" && b == "SDN"),
        "the lawful chain must not reach SDN: {control:?}"
    );
    let pct = effective_pct(&edges, "C", "SDN");
    assert!(
        (pct - 0.30).abs() < 1e-9 && pct >= THRESHOLD,
        "C's transitive SDN control must be 0.30 ≥ 0.25; got {pct}"
    );

    // The gate refuses; the account-creation doc is never sent.
    let d = run_gate(&mut h, SESSION, "C");
    assert!(!d.admitted, "C must be refused");
    assert_eq!(
        d.events,
        vec!["Request_Received", "Ownership_Resolved", "Sanctioned_Control_Detected", "Refused", "State_Preserved"],
        "sanctioned event vector: {:?}",
        d.events
    );

    // Readback: state preserved — zero account rows.
    let (n, _) = read_table(&mut h, SESSION, "ubo_accounts", &["entity", "status"]);
    assert_eq!(n, 0, "accounts table must have zero rows after the refusal");
    let violations = conformance_check(&d, &d.events, n, true);
    assert!(violations.is_empty(), "conformance violations: {violations:?}");

    h.exec(&json!({"op": "duckdb", "mode": "close", "session": SESSION}));
}

#[test]
fn tc_ubo_02_lawful_applicant_admitted_receipt_sealed() {
    let _guard = engine_lock();
    let Some(mut h) = host_or_skip("TC-UBO-02") else { return };
    const SESSION: &str = "ubo-02";
    seed_session(&mut h, SESSION);

    let (_, owns_body) = read_table(&mut h, SESSION, "ubo_owns", &["child", "parent", "pct"]);
    let edges = parse_edges(&owns_body);
    let control = transitive_control(&mut h, &edges);
    assert!(
        control.iter().any(|(a, b)| a == "D" && b == "F"),
        "datalog fixpoint must resolve D→F transitively: {control:?}"
    );
    let pct = effective_pct(&edges, "D", "F");
    assert!(
        (pct - 0.06).abs() < 1e-9 && pct < THRESHOLD,
        "D's transitive control must be 0.06 < 0.25; got {pct}"
    );

    // Gate admits → the CREATE + INSERT docs execute in-module.
    let d = run_gate(&mut h, SESSION, "D");
    assert!(d.admitted, "D must be admitted");

    let ins = exec_ast(
        &mut h,
        SESSION,
        &insert_doc("ubo_accounts", &[("entity", "D", None), ("status", "OPEN", None)]),
    );
    assert!(
        ins["receipt"].is_string() && ins["query_hash"].is_string(),
        "receipt must seal on the admitted mutation: {ins}"
    );

    let (n, body) = read_table(&mut h, SESSION, "ubo_accounts", &["entity", "status"]);
    assert_eq!(n, 1, "exactly the lawful account row materialized");
    assert!(body.contains("\"D\"") && body.contains("\"OPEN\""), "account row readback: {body}");

    let mut events = d.events.clone();
    events.extend(["Account_Materialized", "Receipt_Sealed"]);
    assert_eq!(
        events,
        vec!["Request_Received", "Ownership_Resolved", "Cleared", "Admitted", "Account_Materialized", "Receipt_Sealed"],
        "lawful event vector: {events:?}"
    );
    let violations = conformance_check(&d, &events, n, false);
    assert!(violations.is_empty(), "conformance violations: {violations:?}");

    h.exec(&json!({"op": "duckdb", "mode": "close", "session": SESSION}));
}

#[test]
fn tc_ubo_03_broken_gate_flags_illegal_materialization() {
    let _guard = engine_lock();
    let Some(mut h) = host_or_skip("TC-UBO-03") else { return };
    const SESSION: &str = "ubo-03";
    seed_session(&mut h, SESSION);

    // The broken gate skips traversal and admits the sanctioned applicant C.
    let d = run_broken_gate(&mut h, SESSION, "C");
    assert!(d.admitted, "the broken gate admits unconditionally by construction");
    assert!(
        !d.events.contains(&"Refused") && !d.events.contains(&"State_Preserved"),
        "broken gate log must lack the refusal events: {:?}",
        d.events
    );

    // Its illegal mutation lands: C's row is materialized.
    exec_ast(
        &mut h,
        SESSION,
        &insert_doc("ubo_accounts", &[("entity", "C", None), ("status", "OPEN", None)]),
    );
    let (n, body) = read_table(&mut h, SESSION, "ubo_accounts", &["entity", "status"]);
    assert_eq!(n, 1, "the broken gate's illegal row landed");
    assert!(body.contains("\"C\""), "C's illegal account row readback: {body}");

    // The conformance check MUST flag it: missing Refused/State_Preserved +
    // illegal materialization. Non-vacuity twin: the same check passes
    // courts 1 and 2 (asserted there).
    let violations = conformance_check(&d, &d.events, n, true);
    assert!(
        violations.iter().any(|v| v.contains("Refused/State_Preserved")),
        "detection must flag the missing refusal events: {violations:?}"
    );
    assert!(
        violations.iter().any(|v| v.contains("illegal materialization")),
        "detection must flag the illegal materialization: {violations:?}"
    );

    h.exec(&json!({"op": "duckdb", "mode": "close", "session": SESSION}));
}
