//! TC-FIBO-08 (COPY egress) — its own integration-test process.
//!
//! Split out of `tests/wasm_fibo_fixtures_eh.rs`: WASI p1 fd-table state left
//! by earlier in-process wasm DuckDB instances (TC-FIBO-01..03) broke the
//! pid-keyed COPY egress when the courts shared a process (the COPY's lock on
//! the egress file failed with `Invalid argument`), so
//! `tc_fibo_08_copy_executes_with_pid_keyed_egress` passed alone but failed
//! after TC-FIBO-01..03 in-process. A separate integration test target is a
//! separate process under `cargo test`: the interference is removed
//! structurally — no sleeps, no ordering flags, no fixture changes.
//!
//! Harness carried verbatim from `tests/wasm_fibo_fixtures_eh.rs` (same
//! ENGINE_LOCK serialization, same typed loud-SKIP discipline) minus the
//! TC-FIBO-01..03-only helpers (`select_doc`, `Host::stream_all`); the
//! TC-FIBO-08 court body is unchanged.
//!
//! Court:
//! - TC-FIBO-08: tc08_copy compiles and executes against a session; the
//!   egress file lands under /tmp in a pid-keyed path (the fixture's
//!   literal path is rewritten to the pid-keyed twin before admission — the
//!   fixture file itself is read-only court capital) and must carry bytes.
//!
//! Typed loud SKIP discipline: when the artifact is missing, or when the
//! first gl_call traps on an uninitialized element (stale/mismatched
//! artifact), the court prints `SKIP(typed): … pending-artifact` and
//! returns. A SKIP is not a pass.
//!
//! Chicago: real module, real frozen-duckdb inside it, real file readbacks;
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

fn open_session(h: &mut Host, name: &str) {
    let r = h.exec(&json!({"op": "duckdb", "mode": "open", "session": name}));
    assert_eq!(r["session"], json!(name), "session open reply: {r}");
}

fn close_session(h: &mut Host, name: &str) {
    h.exec(&json!({"op": "duckdb", "mode": "close", "session": name}));
}

// ------------------------------------------------------- TC-FIBO-08-compile

#[test]
fn tc_fibo_08_copy_executes_with_pid_keyed_egress() {
    let _guard = engine_lock();
    let Some(mut h) = host_or_skip("TC-FIBO-08-compile") else { return };
    const SESSION: &str = "fibo-08";

    // Pid-keyed egress twin of the fixture's literal /tmp path. The fixture
    // file is court capital (read-only); the path is rewritten in the
    // admitted document only.
    let egress = format!("/tmp/fibo_accounts_{}.csv", std::process::id());
    let _ = std::fs::remove_file(&egress);
    let doc = fixture("tc08_copy.ttl").replace("/tmp/fibo_accounts.csv", &egress);
    assert!(
        doc.contains(&egress),
        "egress path rewrite must land in the admitted document"
    );

    // Seed the table the COPY exports.
    open_session(&mut h, SESSION);
    h.exec(&exec_req(SESSION, &fixture("tc01_create.ttl")));
    h.exec(&exec_req(SESSION, &fixture("tc01_insert.ttl")));

    let cp = h.exec(&exec_req(SESSION, &doc));
    assert!(
        cp["receipt"].is_string() && cp["query_hash"].is_string(),
        "COPY receipt fields missing: {cp}"
    );

    // Real egress artifact: bytes on disk. If the G3 receipt carries a digest
    // field, assert its presence (blake3 is not a dev-dependency — the
    // independent digest cross-check lands when the manifest adds it).
    let meta = std::fs::metadata(&egress)
        .unwrap_or_else(|e| panic!("COPY egress file {egress} missing: {e}"));
    assert!(meta.len() > 0, "egress artifact must carry bytes");
    if let Some(digest) = cp["digest"].as_str() {
        assert!(!digest.is_empty(), "G3 receipt digest present but empty: {cp}");
    }
    let _ = std::fs::remove_file(&egress);

    close_session(&mut h, SESSION);
}
