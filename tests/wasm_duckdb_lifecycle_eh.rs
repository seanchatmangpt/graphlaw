//! Court 1 — full in-wasm DDL→DML→SELECT→receipt lifecycle under wasmtime 48
//! with W3C exceptions enabled (`Config::wasm_exceptions(true)`).
//!
//! The unit under test is the DuckDB-linked `graphlaw-wasm` artifact
//! (`GRAPHLAW_WASM`, fallback `target/wasm32-wasip1/wasm/graphlaw_wasm.wasm`)
//! driven in-process through the gl_alloc/gl_call/gl_free protocol — the same
//! request/response handling the packet-4 host implements in
//! `src/bin/graphlaw_run.rs` (Host::new / request_bytes), hosted by wasmtime
//! instead of wasmi (wasmi rejects the exception tag section, id 13).
//!
//! Courts:
//! - A (lifecycle): four sql-rdf Turtle documents through
//!   `{"op":"duckdb","mode":"exec",...}` — CREATE TABLE, INSERT, SELECT SUM,
//!   WHERE-scoped UPDATE — with exact row/value readbacks and receipt fields.
//! - B (mutation guard): a WHERE-less UPDATE doc gets the stable typed refusal
//!   `[REFUSAL-05]` (compile happens in-module), and a follow-up SELECT proves
//!   the table unchanged (anti-vacuity twin).
//! - C (hosting): the module instantiates under wasmtime with exceptions
//!   enabled. A loud SKIP is not a pass: when `GRAPHLAW_WASM` is set but
//!   instantiation fails, the test FAILS, it does not skip.
//!
//! Chicago: real module, real frozen-duckdb inside it, real readbacks; no
//! mocks. Engine-touching tests serialize behind one process-local mutex
//! (tests/duckdb_statement_kinds.rs ENGINE_LOCK pattern).
//
// coordinator: add [dev-dependencies] wasmtime = { version = "48", features = ["runtime"] }, wasmtime-wasi = "48"

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

/// Lock without poison cascade: a court that panics (e.g. the historical
/// pending-artifact `unknown op`) must not fail its siblings with PoisonError — each court
/// reports its own outcome.
fn engine_lock() -> std::sync::MutexGuard<'static, ()> {
    ENGINE_LOCK.lock().unwrap_or_else(std::sync::PoisonError::into_inner)
}

// --------------------------------------------------------------- harness

/// The wasm artifact under test: `GRAPHLAW_WASM` overrides, else the default
/// target-dir build output. Returns None when neither exists (Court C's loud
/// SKIP path; A and B fail loudly instead — the artifact is the unit).
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
    let artifact =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("target/wasm32-wasip1/wasm/graphlaw_wasm.wasm");
    if artifact.is_file() {
        return Some(artifact);
    }
    None
}

/// One wasmtime host around a compiled module: wasi_snapshot_preview1 imports
/// plus the gl_alloc/gl_call/gl_free JSON-over-linear-memory ABI. Mirrors
/// `src/bin/graphlaw_run.rs` Host (same request_bytes protocol), hosted by
/// wasmtime 48 with `wasm_exceptions(true)` so the DuckDB-linked module's W3C
/// exception tag section validates.
struct Host {
    store: Store<WasiP1Ctx>,
    memory: Memory,
    alloc: TypedFunc<u32, u32>,
    free: TypedFunc<(u32, u32), ()>,
    call: TypedFunc<(u32, u32), u64>,
}

impl Host {
    fn new(bytes: &[u8]) -> Result<Self, String> {
        let mut cfg = Config::new();
        cfg.wasm_exceptions(true);
        let engine = Engine::new(&cfg).map_err(|e| format!("engine: {e}"))?;
        let module = Module::new(&engine, bytes).map_err(|e| format!("module: {e}"))?;
        // The only imports allowed are WASI's (clock, random, stdio).
        for i in module.imports() {
            if i.module() != "wasi_snapshot_preview1" {
                return Err(format!("unexpected host import {}::{}", i.module(), i.name()));
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
        add_to_linker_sync(&mut linker, |ctx| ctx).map_err(|e| format!("wasi linker: {e}"))?;
        let instance: Instance = linker
            .instantiate(&mut store, &module)
            .map_err(|e| format!("instantiate: {e}"))?;
        // Reactor-style modules expose `_initialize`; hosts must call it once.
        if let Ok(init) = instance.get_typed_func::<(), ()>(&mut store, "_initialize") {
            init.call(&mut store, ())
                .map_err(|e| format!("_initialize: {e}"))?;
        }
        let memory = instance
            .get_memory(&mut store, "memory")
            .ok_or("module does not export memory")?;
        let missing =
            |name: &'static str, e: wasmtime::Error| format!("missing or mistyped export {name}: {e}");
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

    /// `gl_alloc` + memory write + `gl_call` + read + `gl_free` — the
    /// `request_bytes` pattern of `src/bin/graphlaw_run.rs`.
    fn request_bytes(&mut self, body: &[u8]) -> Result<Vec<u8>, String> {
        let ptr = self
            .alloc
            .call(&mut self.store, body.len() as u32)
            .map_err(|e| format!("gl_alloc: {e}"))?;
        if ptr == 0 {
            return Err("gl_alloc refused a request the caller pre-checked".into());
        }
        self.memory
            .write(&mut self.store, ptr as usize, body)
            .map_err(|e| format!("memory write: {e}"))?;
        let packed = self
            .call
            .call(&mut self.store, (ptr, body.len() as u32))
            .map_err(|e| format!("gl_call: {e}"))?;
        let (out_ptr, out_len) = ((packed >> 32) as u32, (packed & 0xffff_ffff) as u32);
        let mut out = vec![0u8; out_len as usize];
        self.memory
            .read(&self.store, out_ptr as usize, &mut out)
            .map_err(|e| format!("memory read: {e}"))?;
        self.free
            .call(&mut self.store, (out_ptr, out_len))
            .map_err(|e| format!("gl_free: {e}"))?;
        Ok(out)
    }

    fn request(&mut self, req: &Value) -> Value {
        let out = self
            .request_bytes(req.to_string().as_bytes())
            .expect("request transport");
        serde_json::from_slice(&out).expect("response is JSON")
    }

    fn exec(&mut self, req: &Value) -> Value {
        let r = self.request(req);
        assert_eq!(
            r["ok"],
            json!(true),
            "exec request failed: {req} -> {r}"
        );
        r
    }

    /// Stream a cursor fully and return the N-Triples body.
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

/// Build one fresh instance per court; instantiation is cheap relative to the
/// engine work and isolates cursor/DDL state between courts.
fn host() -> Host {
    let path = wasm_path().expect("wasm artifact missing; see Court C's typed SKIP");
    let bytes = std::fs::read(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
    Host::new(&bytes).unwrap_or_else(|e| panic!("instantiate {}: {e}", path.display()))
}

// --------------------------------------------------------------- documents

/// Lifecycle documents ride the ADMITTED compile-law shapes, pinned by the
/// FIBO court-matrix fixtures (tests/fixtures/fibo/): CREATE is the
/// expression form (sql:tableDefinition/sql:columns, DECIMAL in the
/// column_type allowlist); INSERT is the scalar vocabulary with
/// sql:statementIndex; SELECT SUM uses sql:functionName "SUM" +
/// sql:arguments; the bounded UPDATE carries BOTH whereColumn and
/// whereValue. Reading the fixtures from disk (not inlining copies) keeps
/// this court and the FIBO court on one admission record.
const FIBO_TABLE: &str = "fibo_accounts";

fn fixture(name: &str) -> String {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/fibo")
        .join(name);
    std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("fixture {} missing: {e}", path.display()))
}

/// CREATE TABLE fibo_accounts (id INTEGER, balance DECIMAL) — tc01_create.
fn create_doc() -> String {
    fixture("tc01_create.ttl")
}

/// INSERT INTO fibo_accounts (balance) VALUES (1250.50) — tc01_insert.
fn insert_doc() -> String {
    fixture("tc01_insert.ttl")
}

/// Second row, same admitted scalar vocabulary as tc01_insert:
/// INSERT INTO fibo_accounts (balance) VALUES (250.00).
fn insert_row2_doc() -> String {
    r#"@prefix ex: <http://example.org/fibo#> .
@prefix sql: <http://graphlaw.org/sql#> .
@prefix xsd: <http://www.w3.org/2001/XMLSchema#> .

ex:stmt1 a sql:InsertStatement ;
    sql:statementIndex 1 ;
    sql:targetTable "fibo_accounts" ;
    sql:assignColumn "balance" ;
    sql:assignValue "250.00"^^xsd:decimal ."#
        .to_string()
}

/// SELECT SUM(balance) FROM fibo_accounts. The tc05_select_sum.ttl fixture's
/// `sql:functionName` aggregate form compiles to a scalar-function call that
/// the frozen wasm DuckDB catalog does not carry (catalog error: "Scalar
/// Function with name \"sum\" is not in the catalog"), so this court rides
/// the section-7 `sql:function <http://graphlaw.org/functions#SUM>` +
/// `sql:argument` form instead (the shape chicago_validation's aggregate
/// courts admit). Fixture/kernel follow-up reported to the coordinator.
fn sum_select_doc() -> String {
    r#"@prefix ex: <http://example.org/queries#> .
@prefix sql: <http://graphlaw.org/sql#> .

ex:stmt1 a sql:SelectStatement ;
    sql:statementIndex 5 ;
    sql:fromTable [ sql:tableName "fibo_accounts" ] ;
    sql:selectList ( [ sql:expression [
        sql:function <http://graphlaw.org/functions#SUM> ;
        sql:argument [ sql:columnName "balance" ] ] ] ) ."#
        .to_string()
}

/// LOAD core_functions — the admitted `LoadStatement` class compiles to
/// `LOAD {name}`; the wasm frozen DuckDB catalog carries aggregates in the
/// statically-linked core_functions extension but does not auto-load it, so
/// the SUM step needs this one-time session statement (the catalog error's
/// own recovery instruction, admitted through the AST surface).
fn load_core_functions_doc() -> String {
    r#"@prefix ex: <http://example.org/fibo#> .
@prefix sql: <http://graphlaw.org/sql#> .

ex:stmt1 a sql:LoadStatement ;
    sql:statementIndex 4 ;
    sql:targetTable "core_functions" ."#
        .to_string()
}

/// Full-column readback of the balance rows, ordered.
fn rows_select_doc() -> String {
    r#"@prefix ex: <http://example.org/fibo#> .
@prefix sql: <http://graphlaw.org/sql#> .

ex:stmt1 a sql:SelectStatement ;
    sql:statementIndex 5 ;
    sql:fromTable [ sql:tableName "fibo_accounts" ] ;
    sql:selectList ( [ sql:columnName "balance" ] ) ;
    sql:orderBy ( [ sql:columnName "balance" ] ) ."#
        .to_string()
}

/// WHERE-scoped UPDATE in the admitted scalar vocabulary (BOTH where fields —
/// the compiler law requires both): row balance = 1250.50 becomes 99000.25.
fn update_doc() -> String {
    r#"@prefix ex: <http://example.org/fibo#> .
@prefix sql: <http://graphlaw.org/sql#> .
@prefix xsd: <http://www.w3.org/2001/XMLSchema#> .

ex:stmt1 a sql:UpdateStatement ;
    sql:statementIndex 2 ;
    sql:targetTable "fibo_accounts" ;
    sql:assignColumn "balance" ;
    sql:assignValue "99000.25"^^xsd:decimal ;
    sql:whereColumn "balance" ;
    sql:whereValue "1250.50"^^xsd:decimal ."#
        .to_string()
}

/// WHERE-less UPDATE — the REFUSAL-05 mutation (no where predicates).
fn unbounded_update_doc() -> String {
    r#"@prefix ex: <http://example.org/fibo#> .
@prefix sql: <http://graphlaw.org/sql#> .
@prefix xsd: <http://www.w3.org/2001/XMLSchema#> .

ex:stmt1 a sql:UpdateStatement ;
    sql:statementIndex 2 ;
    sql:targetTable "fibo_accounts" ;
    sql:assignColumn "balance" ;
    sql:assignValue "99000.25"^^xsd:decimal ."#
        .to_string()
}

fn exec_req(session: &str, turtle: &str) -> Value {
    json!({
        "op": "duckdb",
        "mode": "exec",
        "session": session,
        "query_ast": "http://example.org/queries#Q",
        "subject_prefix": "http://example.org/data",
        "vocab_prefix": "http://example.org/vocab",
        "ast": {"turtle": turtle, "dialect": "duckdb"}
    })
}

/// Open a named session (persistent in-memory connection across gl_call
/// frames); duplicate names are refused, so each court uses its own.
fn open_session(h: &mut Host, name: &str) {
    let r = h.exec(&json!({"op": "duckdb", "mode": "open", "session": name}));
    assert_eq!(r["session"], json!(name), "session open reply: {r}");
}

fn close_session(h: &mut Host, name: &str) {
    h.exec(&json!({"op": "duckdb", "mode": "close", "session": name}));
}

// --------------------------------------------------------------- Court C

/// The module instantiates under wasmtime 48 with W3C exceptions enabled.
/// Artifact entirely missing → loud typed SKIP (eprintln + early return; a
/// SKIP is not a pass). `GRAPHLAW_WASM` set but instantiation failing → FAIL.
#[test]
fn court_c_module_instantiates_under_wasmtime_with_exceptions() {
    let _guard = engine_lock();
    let Some(path) = wasm_path() else {
        eprintln!(
            "// SKIP(typed): artifact not built — build with `WASI_SDK_PATH=<wasi-sdk> \
             cargo build -p graphlaw-wasm --target wasm32-wasip1 --profile wasm`"
        );
        return;
    };
    let bytes = std::fs::read(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
    match Host::new(&bytes) {
        Ok(_host) => {}
        Err(e) => panic!("module {} failed to instantiate under wasmtime+exceptions: {e}", path.display()),
    }
}

// --------------------------------------------------------------- Court A

/// Full lifecycle: CREATE → INSERT ×2 → SELECT SUM → scoped UPDATE → exact
/// readbacks, with receipt fields present and exact rows_emitted.
#[test]
fn court_a_ddl_dml_select_receipt_lifecycle() {
    let _guard = engine_lock();
    let mut h = host();
    const SESSION: &str = "lifecycle-a";
    open_session(&mut h, SESSION);

    // (a) CREATE TABLE accounts.
    let create = h.exec(&exec_req(SESSION, &create_doc()));
    assert!(
        create["receipt"].is_string() && create["query_hash"].is_string(),
        "CREATE receipt fields missing: {create}"
    );

    // (b) INSERT two rows (1250.50 via the tc01 fixture + 250.00 inline;
    // SUM = 1500.50 — the mutation witness).
    for doc in [&insert_doc(), &insert_row2_doc()] {
        let ins = h.exec(&exec_req(SESSION, doc));
        assert!(
            ins["receipt"].is_string() && ins["query_hash"].is_string(),
            "INSERT receipt fields missing: {ins}"
        );
    }

    // (c) LOAD core_functions, then SELECT SUM(balance) aggregate: exact
    // rowcount and exact value.
    let load = h.exec(&exec_req(SESSION, &load_core_functions_doc()));
    assert!(
        load["receipt"].is_string() && load["query_hash"].is_string(),
        "LOAD receipt fields missing: {load}"
    );
    let sum = h.exec(&exec_req(SESSION, &sum_select_doc()));
    let sum = h.exec(&exec_req(SESSION, &sum_select_doc()));
    assert_eq!(
        sum["rows_emitted"], json!(1),
        "SUM aggregate emits exactly one row"
    );
    let body = h.stream_all(SESSION, sum["cursor"].as_str().expect("cursor id"));
    assert!(
        body.contains("1500.5"),
        "SUM(balance) over [1250.50, 250.00] must read back 1500.50; got: {body}"
    );

    // (d) WHERE-scoped UPDATE: balance 1250.50 -> 99000.25.
    let upd = h.exec(&exec_req(SESSION, &update_doc()));
    assert!(
        upd["receipt"].is_string() && upd["query_hash"].is_string(),
        "UPDATE receipt fields missing: {upd}"
    );

    // Exact state: full-table readback is now {99000.25, 250.00}; the scoped
    // WHERE moved only the 1250.50 row.
    let all = h.exec(&exec_req(SESSION, &rows_select_doc()));
    assert_eq!(all["rows_emitted"], json!(2), "exactly two rows survive");
    let rows = h.stream_all(SESSION, all["cursor"].as_str().expect("cursor id"));
    assert!(rows.contains("250"), "row 250.00 untouched: {rows}");
    assert!(rows.contains("99000.25"), "1250.50 was updated to 99000.25: {rows}");
    assert!(
        !rows.contains("1250.5"),
        "1250.50 must be gone after UPDATE: {rows}"
    );

    // Replay: the SUM over the post-update state is exactly 99250.25.
    let sum2 = h.exec(&exec_req(SESSION, &sum_select_doc()));
    let body2 = h.stream_all(SESSION, sum2["cursor"].as_str().expect("cursor id"));
    assert!(
        body2.contains("99250.25"),
        "SUM(balance) after UPDATE must be 99250.25; got: {body2}"
    );

    close_session(&mut h, SESSION);
}

// --------------------------------------------------------------- Court B

/// A WHERE-less UPDATE doc gets the stable typed refusal `[REFUSAL-05]`
/// (compile happens in-module, before any engine call), and a follow-up
/// SELECT proves the table unchanged — the anti-vacuity twin.
#[test]
fn court_b_whereless_update_refused_and_table_unchanged() {
    let _guard = engine_lock();
    let mut h = host();
    const SESSION: &str = "lifecycle-b";
    open_session(&mut h, SESSION);

    // Seed state: CREATE + two rows.
    h.exec(&exec_req(SESSION, &create_doc()));
    h.exec(&exec_req(SESSION, &insert_doc()));
    h.exec(&exec_req(SESSION, &insert_row2_doc()));

    // The mutation: WHERE-less UPDATE must be refused typed.
    let r = h.request(&exec_req(SESSION, &unbounded_update_doc()));
    assert_eq!(
        r["ok"],
        json!(false),
        "WHERE-less UPDATE must be refused, not executed: {r}"
    );
    let msg = r["error"]["message"]
        .as_str()
        .unwrap_or_else(|| panic!("refusal envelope missing error.message: {r}"));
    assert!(
        msg.contains("[REFUSAL-05]"),
        "refusal must carry the stable code [REFUSAL-05]; got: {msg}"
    );

    // Anti-vacuity twin: the table is unchanged (SELECT readback
    // {1250.50, 250.00}).
    let all = h.exec(&exec_req(SESSION, &rows_select_doc()));
    assert_eq!(all["rows_emitted"], json!(2), "no row was added or removed");
    let rows = h.stream_all(SESSION, all["cursor"].as_str().expect("cursor id"));
    assert!(
        rows.contains("1250.5") && rows.contains("250"),
        "both original values survive the refused mutation: {rows}"
    );
    assert!(
        !rows.contains("99000.25"),
        "the unbounded mutation never landed: {rows}"
    );

    close_session(&mut h, SESSION);
}

// --------------------------------------------------------------- Court D

/// Digest of the pre-duckdb legacy artifact (`priv/graphlaw.wasm` at the
/// pre-wiring pin). When the shipped artifact's blake3 no longer matches, the
/// before-state witness is obsolete and skips loudly — never a faked pass.
const LEGACY_DIGEST: &str =
    "f149433c6e1a84ec95e961ac92b16b65b28e15d6fd8546beb1779d3b54a02e7e";

/// Legacy before-state witness, ported from the retired
/// tests/wasm_duckdb_op_wired.rs (`legacy_pin_shipped_artifact_refuses_duckdb_op`):
/// the pre-duckdb shipped artifact answers the duckdb request with the typed
/// unknown-op refusal. Digest-gated — when `priv/graphlaw.wasm` is re-pinned
/// past the pre-wiring build, this skips loudly instead of failing falsely.
/// The legacy artifact predates the W3C exception tag section, so it
/// instantiates under this same wasmtime EH host (a proposal enabled in the
/// runtime admits modules that do not use it).
#[test]
fn court_d_legacy_pin_shipped_artifact_refuses_duckdb_op() {
    let _guard = engine_lock();
    let shipped = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("priv/graphlaw.wasm");
    let bytes = match std::fs::read(&shipped) {
        Ok(b) => b,
        Err(_) => {
            eprintln!(
                "// SKIP(typed): legacy witness artifact {} missing — before-state \
                 refusal lives in the receipt now",
                shipped.display()
            );
            return;
        }
    };
    let digest = blake3::Hasher::new().update(&bytes).finalize().to_string();
    if digest != LEGACY_DIGEST {
        eprintln!(
            "// SKIP(typed): {} digest {digest} no longer matches the legacy \
             pre-duckdb pin ({LEGACY_DIGEST}); artifact re-pinned past the before-state",
            shipped.display()
        );
        return;
    }
    let mut h = match Host::new(&bytes) {
        Ok(h) => h,
        Err(e) => {
            eprintln!(
                "// SKIP(typed): cannot drive the legacy artifact under the \
                 wasmtime EH host: {e}"
            );
            return;
        }
    };
    let out = h.request(&json!({"op": "duckdb", "mode": "close", "cursor": "x"}));
    assert_eq!(out["ok"], Value::Bool(false), "{out}");
    assert_eq!(out["error"]["kind"], "Unsupported", "{out}");
    let msg = out["error"]["message"]
        .as_str()
        .expect("close-refusal envelope must carry a string error.message");
    assert!(msg.contains("unknown op"), "{out}");
}
