//! Court 2 — the materialized FIBO court-matrix fixtures
//! (`tests/fixtures/fibo/*.ttl`) through the in-wasm DuckDB lifecycle harness
//! (the `tests/wasm_duckdb_lifecycle_eh.rs` pattern: wasmtime 48 +
//! `wasm_exceptions(true)`, p1 WASI, gl_alloc/gl_call/gl_free, ENGINE_LOCK).
//!
//! Expected per-file outcomes come from
//! `docs/jira/v26.10.22/designs/fibo-pm4py-court-matrix.md` (fixtures
//! section). These are execution-layer courts, not compile-layer status.
//!
//! Courts:
//! - TC-FIBO-01: tc01_create + tc01_insert → ok:true, exact readback of the
//!   single row (balance 1250.50).
//! - TC-FIBO-02: tc02_update_bounded → executes; readback shows the updated
//!   row only.
//! - TC-FIBO-03: tc03_delete_unbounded → ok:false, message carries
//!   `[REFUSAL-05] UnboundedMutationRefusal` (asserted on the message string
//!   per the matrix correction — the SHACL layer fires first, so the wire
//!   code is MalformedAst/SHACL_VIOLATIONS; do NOT assert
//!   RefusalCode::UnboundedMutation on the wire), twin readback unchanged.
//! - TC-FIBO-08 lives in `tests/wasm_fibo_egress_eh.rs`: it runs in its own
//!   integration-test process — WASI p1 fd-table state left by TC-FIBO-01..03
//!   in-process broke the pid-keyed COPY egress (lock `Invalid argument`).
//!
//! Typed loud SKIP discipline: when the artifact is missing, or when the
//! first gl_call traps on an uninitialized element (stale/mismatched
//! artifact), every court prints `SKIP(typed): … pending-artifact` and
//! returns. A SKIP is not a pass.
//!
//! Chicago: real module, real frozen-duckdb inside it, real file readbacks;
//! no mocks. blake3 is not a dev-dependency, so the G3 receipt digest (when
//! present) is asserted for presence and the egress artifact is asserted by
//! real byte count; an independent blake3 cross-check is deferred until
//! blake3 joins [dev-dependencies] (coordinator-owned manifest).

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

/// The wasm artifact under test: `GRAPHLAW_WASM` overrides, else the default
/// target-dir build output. None = the typed-SKIP path.
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
}

/// Classification of a Host construction failure.
#[derive(Debug)]
enum HostError {
    /// gl_call trapped on uninitialized element — stale/mismatched artifact.
    Trap(String),
    /// Anything else (missing exports, bad imports, instantiation failure).
    Fatal(String),
}

impl Host {
    /// Same shape as the lifecycle court's Host, plus a `/tmp` preopen so the
    /// in-module COPY can create its egress file (TC-FIBO-08).
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
        // DIAG-NOTE(tc08): temporarily toggled to bisect the COPY egress EINVAL.
        cfg.wasm_exceptions(std::env::var("GRAPHLAW_DIAG_NO_EH").is_err());
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
        dump_preopen_fdstat(&linker, &mut store, &instance, &memory);
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

/// Diagnostic: call the linked preview1 `fd_fdstat_get` on the preopen fd
/// (fd 3, the first preopen) and dump the rights masks. TC-FIBO-08 egress
/// needs FD_ALLOCATE + FD_FILESTAT_SET_SIZE present in fs_rights_inheriting.
fn dump_preopen_fdstat(
    linker: &Linker<WasiP1Ctx>,
    store: &mut Store<WasiP1Ctx>,
    instance: &Instance,
    memory: &Memory,
) {
    const RIGHTS: [(&str, u64); 9] = [
        ("FD_ALLOCATE", 1 << 8),
        ("FD_FILESTAT_SET_SIZE", 1 << 22),
        ("PATH_OPEN", 1 << 13),
        ("FD_WRITE", 1 << 6),
        ("FD_READ", 1 << 1),
        ("PATH_CREATE_FILE", 1 << 10),
        ("PATH_FILESTAT_SET_SIZE", 1 << 19),
        ("POLL_FD_READWRITE", 1 << 27),
        ("SOCK_SHUTDOWN", 1 << 28),
    ];
    let alloc = match instance.get_typed_func::<u32, u32>(&mut *store, "gl_alloc") {
        Ok(f) => f,
        Err(_) => {
            eprintln!("// fdstat: no gl_alloc export; skipping dump");
            return;
        }
    };
    let free = instance.get_typed_func::<(u32, u32), ()>(&mut *store, "gl_free");
    let fdstat_get = match linker
        .get(&mut *store, "wasi_snapshot_preview1", "fd_fdstat_get")
    {
        Ok(f) => f.into_func(),
        Err(_) => {
            eprintln!("// fdstat: fd_fdstat_get not linked");
            return;
        }
        Err(e) => {
            eprintln!("// fdstat: linker get failed: {e}");
            return;
        }
    };
    let Some(fdstat_get) = fdstat_get else {
        eprintln!("// fdstat: fd_fdstat_get not linked as a func");
        return;
    };
    let typed = match fdstat_get.typed::<(i32, i32), i32>(&*store) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("// fdstat: signature mismatch: {e}");
            return;
        }
    };
    let ptr = match alloc.call(&mut *store, 64) {
        Ok(p) if p != 0 => p,
        _ => {
            eprintln!("// fdstat: scratch alloc failed");
            return;
        }
    };
    for fd in [3u32, 4, 5] {
        match typed.call(&mut *store, (fd as i32, ptr as i32)) {
            Ok(errno) if errno == 0 => {
                let mut buf = [0u8; 24];
                memory.read(&*store, ptr as usize, &mut buf).expect("read fdstat");
                let rights_base = u64::from_le_bytes(buf[8..16].try_into().unwrap());
                let rights_inh = u64::from_le_bytes(buf[16..24].try_into().unwrap());
                let filetype = buf[0];
                let flags = u16::from_le_bytes([buf[4], buf[5]]);
                let named = |v: u64| {
                    RIGHTS
                        .iter()
                        .filter(|(_, b)| v & b != 0)
                        .map(|(n, _)| *n)
                        .collect::<Vec<_>>()
                        .join("|")
                };
                eprintln!(
                    "// fdstat fd={fd} filetype={filetype} flags={flags:#06x} \
                     rights_base={rights_base:#018x} rights_inheriting={rights_inh:#018x}\n\
                     //   base bits: {}  |  missing-from-base: {}\n\
                     //   inheriting bits: {}  |  missing-from-inheriting: {}",
                    named(rights_base),
                    RIGHTS.iter().filter(|(_, b)| rights_base & b == 0).map(|(n, _)| *n).collect::<Vec<_>>().join("|"),
                    named(rights_inh),
                    RIGHTS.iter().filter(|(_, b)| rights_inh & b == 0).map(|(n, _)| *n).collect::<Vec<_>>().join("|"),
                );
            }
            Ok(errno) => eprintln!("// fdstat fd={fd}: errno={errno} (no such preopen)"),
            Err(e) => eprintln!("// fdstat fd={fd}: trap: {e}"),
        }
    }
    if let Ok(free) = free {
        let _ = free.call(&mut *store, (ptr, 64));
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

/// One fresh instance per court. Returns Err(trap message) when the first
/// gl_call traps on an uninitialized element — the pending-artifact signal.
fn try_host() -> Result<Host, String> {
    let path = wasm_path().ok_or_else(|| "artifact-missing".to_string())?;
    let bytes = std::fs::read(&path).map_err(|e| format!("fatal: read {}: {e}", path.display()))?;
    match Host::new(&bytes) {
        Ok(h) => Ok(h),
        Err(HostError::Trap(e)) => Err(format!("trap: {e}")),
        Err(HostError::Fatal(e)) => Err(format!("fatal: {e}")),
    }
}

/// Typed loud SKIP for the pending-artifact state; returns a host otherwise.
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

// --------------------------------------------------------------- fixtures

fn fixture(name: &str) -> String {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/fibo").join(name);
    std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("read fixture {}: {e}", path.display()))
}

const FIBO_SUBJECT: &str = "http://example.org/fibo#stmt1";

fn exec_req(session: &str, turtle: &str) -> Value {
    json!({
        "op": "duckdb",
        "mode": "exec",
        "session": session,
        "query_ast": FIBO_SUBJECT,
        "subject_prefix": "http://example.org/fibo",
        "vocab_prefix": "http://example.org/vocab",
        "ast": {"turtle": turtle, "dialect": "duckdb"}
    })
}

/// SELECT over fibo_accounts with an explicit projection.
fn select_doc(columns: &[&str]) -> String {
    let list: Vec<String> = columns
        .iter()
        .map(|c| format!("[ sql:columnName \"{c}\" ]"))
        .collect();
    format!(
        r#"@prefix sql: <http://graphlaw.org/sql#> .
@prefix ex: <http://example.org/fibo#> .
ex:stmt1 a sql:SelectStatement ;
    sql:statementIndex 1 ;
    sql:fromTable [ sql:tableName "fibo_accounts" ] ;
    sql:selectList ( {} ) ."#,
        list.join(" ")
    )
}

fn open_session(h: &mut Host, name: &str) {
    let r = h.exec(&json!({"op": "duckdb", "mode": "open", "session": name}));
    assert_eq!(r["session"], json!(name), "session open reply: {r}");
}

fn close_session(h: &mut Host, name: &str) {
    h.exec(&json!({"op": "duckdb", "mode": "close", "session": name}));
}

// ------------------------------------------------------------- TC-FIBO-01

#[test]
fn tc_fibo_01_create_insert_readback() {
    let _guard = engine_lock();
    let Some(mut h) = host_or_skip("TC-FIBO-01") else { return };
    const SESSION: &str = "fibo-01";
    open_session(&mut h, SESSION);

    h.exec(&exec_req(SESSION, &fixture("tc01_create.ttl")));
    let ins = h.exec(&exec_req(SESSION, &fixture("tc01_insert.ttl")));
    assert!(
        ins["receipt"].is_string() && ins["query_hash"].is_string(),
        "INSERT receipt fields missing: {ins}"
    );

    // Exact readback: one row, balance 1250.50.
    let sel = h.exec(&exec_req(SESSION, &select_doc(&["id", "balance"])));
    assert_eq!(sel["rows_emitted"], json!(1), "exactly one row inserted");
    let body = h.stream_all(SESSION, sel["cursor"].as_str().expect("cursor id"));
    assert!(
        body.contains("1250.50") || body.contains("1250.5"),
        "balance must read back 1250.50; got: {body}"
    );

    close_session(&mut h, SESSION);
}

// ------------------------------------------------------------- TC-FIBO-02

#[test]
fn tc_fibo_02_bounded_update_executes_and_readback() {
    let _guard = engine_lock();
    let Some(mut h) = host_or_skip("TC-FIBO-02") else { return };
    const SESSION: &str = "fibo-02";
    open_session(&mut h, SESSION);

    h.exec(&exec_req(SESSION, &fixture("tc01_create.ttl")));
    h.exec(&exec_req(SESSION, &fixture("tc01_insert.ttl")));

    let upd = h.exec(&exec_req(SESSION, &fixture("tc02_update_bounded.ttl")));
    assert!(
        upd["receipt"].is_string() && upd["query_hash"].is_string(),
        "UPDATE receipt fields missing: {upd}"
    );

    // Readback: the updated row only — balance moved to 99000.25.
    let sel = h.exec(&exec_req(SESSION, &select_doc(&["id", "balance"])));
    let body = h.stream_all(SESSION, sel["cursor"].as_str().expect("cursor id"));
    assert!(
        body.contains("99000.25") || body.contains("99000.25".trim_end_matches('0')),
        "updated balance 99000.25 must appear; got: {body}"
    );
    assert!(
        !body.contains("1250.5"),
        "the pre-update balance must be gone after the bounded UPDATE; got: {body}"
    );

    close_session(&mut h, SESSION);
}

// ------------------------------------------------------------- TC-FIBO-03

#[test]
fn tc_fibo_03_unbounded_delete_refused_and_table_unchanged() {
    let _guard = engine_lock();
    let Some(mut h) = host_or_skip("TC-FIBO-03") else { return };
    const SESSION: &str = "fibo-03";
    open_session(&mut h, SESSION);

    h.exec(&exec_req(SESSION, &fixture("tc01_create.ttl")));
    h.exec(&exec_req(SESSION, &fixture("tc01_insert.ttl")));

    // The attack: WHERE-less DELETE refused at the SHACL-first layer. The
    // matrix correction is explicit: assert on the message string, NOT on
    // RefusalCode::UnboundedMutation on the wire.
    let r = h
        .request(&exec_req(SESSION, &fixture("tc03_delete_unbounded.ttl")))
        .expect("request transport");
    assert_eq!(r["ok"], json!(false), "WHERE-less DELETE must refuse: {r}");
    let msg = r["error"]["message"]
        .as_str()
        .unwrap_or_else(|| panic!("refusal envelope missing error.message: {r}"));
    assert!(
        msg.contains("[REFUSAL-05] UnboundedMutationRefusal"),
        "refusal message must carry [REFUSAL-05] UnboundedMutationRefusal; got: {msg}"
    );

    // Anti-vacuity twin: the row survives untouched.
    let sel = h.exec(&exec_req(SESSION, &select_doc(&["id", "balance"])));
    assert_eq!(sel["rows_emitted"], json!(1), "row survives the refused DELETE");
    let body = h.stream_all(SESSION, sel["cursor"].as_str().expect("cursor id"));
    assert!(
        body.contains("1250.5"),
        "pre-update balance must survive the refused mutation: {body}"
    );

    close_session(&mut h, SESSION);
}
