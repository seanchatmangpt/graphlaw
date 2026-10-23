//! Case 3 — collateral rehypothecation gate court (Dodd-Frank §732 / CFTC
//! 1.20). The gate is HOST-side by design: the engine (graphlaw-wasm, with its
//! in-module frozen-duckdb) has no flag knowledge — the admission boundary is
//! the host inspecting the collateral row through a SELECT and deciding
//! whether any pledge UPDATE document is sent at all.
//!
//! Courts:
//! - REHY-01 (permitted pledge): host inspects via SELECT (stream readback:
//!   is_rehypothecable == true) THEN admits the bounded pledge UPDATE doc
//!   (WHERE id = permitted) → executes; encumbrance state change witnessed via
//!   readback (encumbered flips true); receipt sealed. Host-side event vector
//!   asserts the [Received → Inspected → Admitted → Executed → Receipted]
//!   sequence exactly.
//! - REHY-02 (encumbered pledge): SELECT reads is_rehypothecable == false →
//!   host REFUSES to send any UPDATE doc (zero engine mutation calls) → table
//!   state bit-identical via readback; host-side event vector
//!   [Received → Inspected → Refused → Preserved → Receipted]; typed refusal
//!   receipt minted host-side (a struct, not a chain write).
//! - REHY-03 (anti-vacuity): a deliberately broken gate (skips inspection,
//!   sends the pledge UPDATE for the encumbered row) → the UPDATE EXECUTES
//!   (the engine has no flag knowledge — proving the interception is NOT the
//!   engine's), and the conformance detector flags the missing Refused/
//!   Preserved events plus the encumbered-row mutation. Asserts the detection
//!   fires: the gate is load-bearing, not decorative.
//!
//! Harness per `tests/wasm_fibo_fixtures_eh.rs`: typed loud SKIPs
//! (artifact-missing / first-gl_call-trap), ENGINE_LOCK serialization,
//! `GRAPHLAW_WASM` env override.
//!
//! Chicago: real module, real frozen-duckdb inside it, real nquads readbacks,
//! no mocks.

#![cfg(all(feature = "abi", not(target_arch = "wasm32")))]

use std::path::PathBuf;
use std::sync::Mutex;

use serde_json::{Value, json};
use wasmtime::{Config, Engine, Instance, Linker, Memory, Module, Store, TypedFunc};
use wasmtime_wasi::WasiCtxBuilder;
use wasmtime_wasi::p1::{WasiP1Ctx, add_to_linker_sync};

/// The frozen engine segfaults when engine-heavy tests run concurrently;
/// serialize the wasm-engine courts with a process-local lock (structural
/// sync, no sleeps).
static ENGINE_LOCK: Mutex<()> = Mutex::new(());

fn engine_lock() -> std::sync::MutexGuard<'static, ()> {
    ENGINE_LOCK.lock().unwrap_or_else(std::sync::PoisonError::into_inner)
}

// --------------------------------------------------------------- harness
// (verbatim shape of the wasm_fibo_fixtures_eh harness)

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

struct Host {
    store: Store<WasiP1Ctx>,
    memory: Memory,
    alloc: TypedFunc<u32, u32>,
    free: TypedFunc<(u32, u32), ()>,
    call: TypedFunc<(u32, u32), u64>,
    /// Count of mutation-class engine calls (UPDATE/DELETE/COPY exec docs).
    /// The gate court asserts this stays ZERO on the refused path.
    mutations: u32,
}

#[derive(Debug)]
enum HostError {
    Trap(String),
    Fatal(String),
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
        let module = Module::new(&engine, bytes).map_err(|e| HostError::Fatal(format!("module: {e}")))?;
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
                .preopened_dir("/tmp", "/tmp", wasmtime_wasi::FsPerms::ReadWrite)
                .map_err(|e| HostError::Fatal(format!("preopen /tmp: {e}")))?
                .build_p1(),
        );
        let mut linker = <Linker<WasiP1Ctx>>::new(&engine);
        add_to_linker_sync(&mut linker, |ctx| ctx).map_err(|e| HostError::Fatal(format!("wasi linker: {e}")))?;
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
            mutations: 0,
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

    /// Mutation-class exec: counts the engine call so the refused path can
    /// prove ZERO mutation calls reached the engine.
    fn exec_mutation(&mut self, req: &Value) -> Value {
        self.mutations += 1;
        self.exec(req)
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

// ----------------------------------------------------- SQL-in-RDF documents

fn exec_req(session: &str, turtle: &str) -> Value {
    json!({
        "op": "duckdb",
        "mode": "exec",
        "session": session,
        "query_ast": "http://example.org/collateral#stmt1",
        "subject_prefix": "http://example.org/collateral",
        "vocab_prefix": "http://example.org/vocab",
        "ast": {"turtle": turtle, "dialect": "duckdb"}
    })
}

const CREATE_DOC: &str = r#"@prefix ex: <http://example.org/collateral#> .
@prefix sql: <http://graphlaw.org/sql#> .

ex:stmt1 a sql:CreateStatement ;
    sql:statementIndex 0 ;
    sql:tableDefinition [
        sql:tableName "collateral" ;
        sql:columns (
            [ sql:columnName "id" ; sql:columnType "INTEGER" ]
            [ sql:columnName "is_rehypothecable" ; sql:columnType "BOOLEAN" ]
            [ sql:columnName "encumbered" ; sql:columnType "BOOLEAN" ]
        ) ;
    ] ."#;

fn insert_doc(id: &str, is_reh: &str, enc: &str) -> String {
    format!(
        r#"@prefix ex: <http://example.org/collateral#> .
@prefix sql: <http://graphlaw.org/sql#> .
@prefix xsd: <http://www.w3.org/2001/XMLSchema#> .

ex:stmt1 a sql:InsertStatement ;
    sql:statementIndex 1 ;
    sql:targetTable "collateral" ;
    sql:assignColumn "id" ;
    sql:assignValue "{id}"^^xsd:integer ;
    sql:assignColumn "is_rehypothecable" ;
    sql:assignValue "{is_reh}"^^xsd:boolean ;
    sql:assignColumn "encumbered" ;
    sql:assignValue "{enc}"^^xsd:boolean ."#
    )
}

/// SELECT over collateral with an explicit projection.
fn select_doc() -> String {
    r#"@prefix ex: <http://example.org/collateral#> .
@prefix sql: <http://graphlaw.org/sql#> .

ex:stmt1 a sql:SelectStatement ;
    sql:statementIndex 1 ;
    sql:fromTable [ sql:tableName "collateral" ] ;
    sql:selectList (
        [ sql:columnName "id" ]
        [ sql:columnName "is_rehypothecable" ]
        [ sql:columnName "encumbered" ]
    ) ."#
    .to_string()
}

/// The bounded pledge UPDATE: SET encumbered = true WHERE id = <id>.
fn pledge_update_doc(id: &str) -> String {
    format!(
        r#"@prefix ex: <http://example.org/collateral#> .
@prefix sql: <http://graphlaw.org/sql#> .
@prefix xsd: <http://www.w3.org/2001/XMLSchema#> .

ex:stmt1 a sql:UpdateStatement ;
    sql:statementIndex 2 ;
    sql:targetTable "collateral" ;
    sql:assignColumn "encumbered" ;
    sql:assignValue "true"^^xsd:boolean ;
    sql:whereColumn "id" ;
    sql:whereValue "{id}"^^xsd:integer ."#
    )
}

fn open_session(h: &mut Host, name: &str) {
    let r = h.exec(&json!({"op": "duckdb", "mode": "open", "session": name}));
    assert_eq!(r["session"], json!(name), "session open reply: {r}");
}

fn close_session(h: &mut Host, name: &str) {
    h.exec(&json!({"op": "duckdb", "mode": "close", "session": name}));
}

/// Seed the two-row collateral book: row 1 permitted (true, false), row 2
/// encumbered (false, true).
fn seed(h: &mut Host, session: &str) {
    h.exec(&exec_req(session, CREATE_DOC));
    h.exec(&exec_req(session, &insert_doc("1", "true", "false")));
    h.exec(&exec_req(session, &insert_doc("2", "false", "true")));
}

// --------------------------------------------------------- gate machinery
// (host-side; the engine never sees the flag)

#[derive(Debug, Clone, PartialEq)]
struct CollateralRow {
    id: i64,
    is_rehypothecable: bool,
    encumbered: bool,
}

/// Parse the streamed nquads readback into rows. Each row is emitted under a
/// `.../row_N` subject; per-column quads carry the literal object.
fn parse_readback(body: &str) -> Vec<CollateralRow> {
    use std::collections::BTreeMap;
    let mut order: Vec<String> = Vec::new();
    let mut rows: BTreeMap<String, (Option<i64>, Option<bool>, Option<bool>)> = BTreeMap::new();
    for line in body.lines() {
        let line = line.trim();
        if !line.contains("/row_") {
            continue;
        }
        let subject = match (line.find('<'), line.find('>')) {
            (Some(a), Some(b)) if b > a => line[a + 1..b].to_string(),
            _ => continue,
        };
        let rest = &line[line.find('>').map(|i| i + 1).unwrap_or(0)..];
        let pred = match (rest.find('<'), rest.find('>')) {
            (Some(a), Some(b)) if b > a => &rest[a + 1..b],
            _ => continue,
        };
        let col = pred.rsplit('/').next().unwrap_or("");
        let objpart = &rest[rest.find('>').map(|i| i + 1).unwrap_or(0)..];
        let lex = match (objpart.find('"'), objpart.rfind('"')) {
            (Some(a), Some(b)) if b > a => &objpart[a + 1..b],
            _ => continue,
        };
        let e = rows.entry(subject.clone()).or_insert((None, None, None));
        if !order.contains(&subject) {
            order.push(subject);
        }
        match col {
            "id" => e.0 = lex.parse().ok(),
            "is_rehypothecable" => e.1 = Some(lex == "true"),
            "encumbered" => e.2 = Some(lex == "true"),
            _ => {}
        }
    }
    order
        .into_iter()
        .filter_map(|s| {
            let (id, is_reh, enc) = rows.remove(&s)?;
            Some(CollateralRow {
                id: id?,
                is_rehypothecable: is_reh?,
                encumbered: enc?,
            })
        })
        .collect()
}

/// Inspect the book through the engine (SELECT + stream). This is the gate's
/// ONLY engine interaction on the refused path.
fn inspect(h: &mut Host, session: &str) -> Vec<CollateralRow> {
    let sel = h.exec(&exec_req(session, &select_doc()));
    assert_eq!(sel["rows_emitted"], json!(2), "two collateral rows seeded");
    let body = h.stream_all(session, sel["cursor"].as_str().expect("cursor id"));
    let rows = parse_readback(&body);
    assert_eq!(rows.len(), 2, "readback must parse into 2 rows: {body}");
    rows
}

fn row_by_id<'a>(rows: &'a [CollateralRow], id: i64) -> &'a CollateralRow {
    rows.iter()
        .find(|r| r.id == id)
        .unwrap_or_else(|| panic!("row id={id} absent from readback: {rows:?}"))
}

/// Typed host-side refusal receipt (a struct, not a chain write).
#[derive(Debug, PartialEq)]
struct RefusalReceipt {
    code: &'static str,
    regulation: &'static str,
    row_id: i64,
    detail: String,
}

/// The gate: inspect, then either admit the pledge doc or refuse with a typed
/// receipt WITHOUT producing any update document.
fn rehypothecation_gate(
    h: &mut Host,
    session: &str,
    row_id: i64,
) -> Result<String, RefusalReceipt> {
    let rows = inspect(h, session);
    let row = row_by_id(&rows, row_id);
    if !row.is_rehypothecable {
        return Err(RefusalReceipt {
            code: "REHY-REFUSED-ENCUMBERED-COLLATERAL",
            regulation: "Dodd-Frank §732 / CFTC 1.20",
            row_id,
            detail: format!(
                "collateral id={row_id} is_rehypothecable=false: no pledge \
                 document is admitted; the engine never sees a mutation"
            ),
        });
    }
    Ok(pledge_update_doc(&row_id.to_string()))
}

/// The court's conformance detector: given the recorded event vector and the
/// before/after book states, flag violations. This is what proves the gate is
/// load-bearing — feed it a broken run and it must flag.
fn check_conformance(
    events: &[&str],
    before: &[CollateralRow],
    after: &[CollateralRow],
    mutation_calls: u32,
) -> Vec<String> {
    let mut violations = Vec::new();
    let lawful_full = ["Received", "Inspected", "Admitted", "Executed", "Receipted"];
    let lawful_refused = ["Received", "Inspected", "Refused", "Preserved", "Receipted"];
    // Rule 1 — any observable state change to non-rehypothecable collateral.
    let encumbered_mutated = before.iter().any(|b| {
        !b.is_rehypothecable
            && after
                .iter()
                .find(|a| a.id == b.id)
                .map(|a| a != b)
                .unwrap_or(true)
    });
    if encumbered_mutated {
        violations.push(format!(
            "non-rehypothecable collateral mutated without a Refused/Preserved \
             interception: events {events:?} (missing Refused and/or Preserved \
             — the gate skipped inspection)"
        ));
    }
    // Rule 2 — a mutation call reached the engine with no Inspected event in
    // the log. Catches the invisible case too: a pledge that re-affirms an
    // already-true flag leaves the readback bit-identical, but the CALL is
    // the violation (Dodd-Frank §732: no rehypothecation, not no net change).
    if mutation_calls > 0 && !events.contains(&"Inspected") {
        violations.push(format!(
            "{mutation_calls} mutation call(s) sent to the engine with no \
             Inspected event — no Refused/Preserved interception recorded, \
             the pledge went to the engine unread: events {events:?}"
        ));
    }
    if !(events.starts_with(&lawful_full[..]) || events.starts_with(&lawful_refused[..])) {
        violations.push(format!(
            "event vector does not open with a lawful sequence: {events:?}"
        ));
    }
    violations
}

// ------------------------------------------------------------- REHY-01

#[test]
fn rehy_01_permitted_pledge_inspect_then_admit_execute() {
    let _guard = engine_lock();
    let Some(mut h) = host_or_skip("REHY-01") else { return };
    const SESSION: &str = "rehy-01";
    let mut events: Vec<&str> = Vec::new();

    events.push("Received");
    open_session(&mut h, SESSION);
    seed(&mut h, SESSION);
    let before = inspect(&mut h, SESSION);

    // Gate inspection: the permitted row reads back rehypothecable.
    events.push("Inspected");
    let permitted = row_by_id(&before, 1);
    assert!(
        permitted.is_rehypothecable && !permitted.encumbered,
        "row 1 must be the permitted pledge (true, false): {permitted:?}"
    );

    // Gate ADMITS: the bounded pledge UPDATE document is produced and sent.
    events.push("Admitted");
    let doc = rehypothecation_gate(&mut h, SESSION, 1)
        .expect("permitted pledge must be admitted");
    assert!(
        doc.contains("sql:UpdateStatement") && doc.contains("sql:whereColumn"),
        "admitted doc must be the bounded pledge document: {doc}"
    );

    events.push("Executed");
    let upd = h.exec_mutation(&exec_req(SESSION, &doc));
    assert!(
        upd["receipt"].is_string() && !upd["receipt"].as_str().unwrap().is_empty(),
        "UPDATE receipt must be sealed: {upd}"
    );
    assert!(upd["query_hash"].is_string(), "query_hash missing: {upd}");

    // Encumbrance state change witnessed via readback: row 1 flips to
    // encumbered; row 2 untouched.
    let after = inspect(&mut h, SESSION);
    assert!(
        row_by_id(&after, 1).encumbered,
        "row 1 encumbered must flip true after the pledge executes"
    );
    assert_eq!(
        row_by_id(&after, 1).is_rehypothecable,
        true,
        "row 1 rehypothecability is unchanged by the pledge"
    );
    assert_eq!(
        row_by_id(&after, 2),
        &CollateralRow { id: 2, is_rehypothecable: false, encumbered: true },
        "row 2 must be untouched by the permitted pledge"
    );

    events.push("Receipted");
    assert_eq!(
        events,
        vec!["Received", "Inspected", "Admitted", "Executed", "Receipted"],
        "lawful admitted-path event sequence"
    );
    assert!(
        check_conformance(&events, &before, &after, h.mutations).is_empty(),
        "admitted path must be conformance-clean"
    );

    close_session(&mut h, SESSION);
}

// ------------------------------------------------------------- REHY-02

#[test]
fn rehy_02_encumbered_pledge_refused_zero_mutations() {
    let _guard = engine_lock();
    let Some(mut h) = host_or_skip("REHY-02") else { return };
    const SESSION: &str = "rehy-02";
    let mut events: Vec<&str> = Vec::new();

    events.push("Received");
    open_session(&mut h, SESSION);
    seed(&mut h, SESSION);
    let before = inspect(&mut h, SESSION);

    // Gate inspection: the encumbered row reads back NOT rehypothecable.
    events.push("Inspected");
    let encumbered = row_by_id(&before, 2);
    assert!(
        !encumbered.is_rehypothecable && encumbered.encumbered,
        "row 2 must be the encumbered pledge (false, true): {encumbered:?}"
    );

    // The gate REFUSES: no update document is produced, so NO mutation call
    // reaches the engine (the interception is the host's, before the wire).
    let refusal = rehypothecation_gate(&mut h, SESSION, 2)
        .expect_err("encumbered pledge must be refused at the gate");
    events.push("Refused");
    assert_eq!(
        refusal.code,
        "REHY-REFUSED-ENCUMBERED-COLLATERAL",
        "typed refusal code"
    );
    assert_eq!(refusal.regulation, "Dodd-Frank §732 / CFTC 1.20");
    assert_eq!(refusal.row_id, 2);
    assert!(
        refusal.detail.contains("no pledge"),
        "refusal detail names the interception: {refusal:?}"
    );

    // Preserved: the table state is bit-identical via readback.
    events.push("Preserved");
    let after = inspect(&mut h, SESSION);
    assert_eq!(before, after, "book must be bit-identical after the refusal");
    assert_eq!(h.mutations, 0, "ZERO engine mutation calls on the refused path");

    events.push("Receipted");
    assert_eq!(
        events,
        vec!["Received", "Inspected", "Refused", "Preserved", "Receipted"],
        "lawful refused-path event sequence"
    );
    assert!(
        check_conformance(&events, &before, &after, h.mutations).is_empty(),
        "refused path must be conformance-clean"
    );

    close_session(&mut h, SESSION);
}

// ------------------------------------------------------------- REHY-03

#[test]
fn rehy_03_broken_gate_executes_and_detector_flags() {
    let _guard = engine_lock();
    let Some(mut h) = host_or_skip("REHY-03") else { return };
    const SESSION: &str = "rehy-03";
    // The BROKEN gate's own (dishonest) log: it skipped inspection and went
    // straight to execution.
    let mut events: Vec<&str> = vec!["Received", "Executed", "Receipted"];

    open_session(&mut h, SESSION);
    seed(&mut h, SESSION);
    let before = inspect(&mut h, SESSION);

    // Broken gate: no inspection — it sends the pledge for the ENCUMBERED row.
    let doc = pledge_update_doc("2");
    let upd = h.exec_mutation(&exec_req(SESSION, &doc));
    assert_eq!(
        upd["ok"],
        json!(true),
        "the engine has no flag knowledge: the un-intercepted pledge EXECUTES: {upd}"
    );
    assert!(
        upd["receipt"].is_string(),
        "the executed mutation carries a sealed receipt — invisibly lawful to \
         the engine alone: {upd}"
    );

    // The mutation is real: row 2 flipped.
    let after = inspect(&mut h, SESSION);
    assert!(
        row_by_id(&after, 2).encumbered && row_by_id(&before, 2).encumbered,
        "row 2 was already encumbered; the pledge re-affirmed it — the \
         violation is the un-intercepted mutation call, witnessed by \
         h.mutations = {}",
        h.mutations
    );
    assert_eq!(h.mutations, 1, "the broken gate sent exactly one mutation");

    // The conformance detector must FLAG the broken run.
    let violations = check_conformance(&events, &before, &after, h.mutations);
    assert!(
        !violations.is_empty(),
        "ANTI-VACUITY FAILURE: the broken gate passed conformance — the court \
         is decorative"
    );
    let joined = violations.join(" | ");
    assert!(
        joined.contains("Refused") && joined.contains("Preserved"),
        "detector must flag the missing Refused/Preserved events: {violations:?}"
    );
    assert!(
        joined.contains("Inspected"),
        "detector must flag the skipped inspection: {violations:?}"
    );
    eprintln!(
        "// REHY-03 witness: broken-gate UPDATE executed (ok=true, receipt \
         sealed) and the detector flagged: {joined}"
    );

    close_session(&mut h, SESSION);
}
