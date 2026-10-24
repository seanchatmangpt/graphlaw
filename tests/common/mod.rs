//! Shared harness for tests that drive the *compiled* wasm module beside the
//! native `graphlaw::abi` (same requests, byte-identical responses expected).
//!
//! Extracted from `tests/wasm_abi.rs`. Set `GRAPHLAW_WASM` to test a prebuilt
//! module; otherwise it is built once into
//! `$CARGO_TARGET_DIR/wasm-abi` (or `target/wasm-abi` when unset), honoring
//! the caller's `CARGO_TARGET_DIR` so concurrent lane build roots never
//! collide on a hardcoded path.
//!
//! Public API (frozen for v26.9.29): `native`, `native_bytes`, `wasm`,
//! `wasm_bytes`, `ok`, `refused`, `read`.
#![cfg(all(feature = "abi", not(target_arch = "wasm32")))]
#![allow(dead_code)]

use std::path::PathBuf;
use std::process::Command;
use std::sync::{Mutex, OnceLock};

use serde_json::Value;
use wasmtime::{Config, Engine, Instance, Linker, Memory, Module, Store, TypedFunc};
use wasmtime_wasi::p1::{WasiP1Ctx, add_to_linker_sync};
use wasmtime_wasi::WasiCtxBuilder;

/// Path of the wasm artifact: `GRAPHLAW_WASM`, else a one-time build.
///
/// Concurrency: racing callers are serialized by cargo's own per-target-dir
/// file lock — the inner invocation shares this helper's `--target-dir`, so
/// two concurrent callers block on cargo's flock rather than corrupting the
/// artifact, and cargo places the final `.wasm` atomically (hardlink from
/// `deps/`) so a reader never observes a partial artifact. No additional lock
/// file is layered on top.
pub fn wasm_path() -> PathBuf {
    if let Some(p) = std::env::var_os("GRAPHLAW_WASM") {
        return p.into();
    }
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    // Honor CARGO_TARGET_DIR so concurrent lanes never share a hardcoded path.
    let target = match std::env::var_os("CARGO_TARGET_DIR") {
        Some(dir) if !dir.is_empty() => PathBuf::from(dir).join("wasm-abi"),
        _ => root.join("target/wasm-abi"),
    };
    let out = Command::new(std::env::var("CARGO").unwrap_or_else(|_| "cargo".into()))
        .current_dir(&root)
        .args([
            "build",
            "-p",
            "graphlaw-wasm",
            "--target",
            "wasm32-wasip1",
            "--profile",
            "wasm",
            "--target-dir",
        ])
        .arg(&target)
        .output()
        .unwrap_or_else(|e| panic!("GRAPHLAW-WASM-BUILD-FAILED: cargo did not run: {e}"));
    if !out.status.success() {
        let stderr = String::from_utf8_lossy(&out.stderr);
        let wasi_sdk_refusal = stderr
            .lines()
            .find(|l| l.contains("WASI_SDK_PATH"))
            .map(|l| l.to_string());
        let linker_line = wasi_sdk_refusal
            .clone()
            .or_else(|| {
                stderr
                    .lines()
                    .find(|l| l.contains("rust-lld") || l.contains("-lduckdb"))
                    .map(|l| l.to_string())
            })
            .or_else(|| {
                stderr
                    .lines()
                    .rev()
                    .find(|l| l.starts_with("error:"))
                    .map(|l| l.to_string())
            })
            .unwrap_or_else(|| "<linker line not present in stderr>".to_string());
        let label = if wasi_sdk_refusal.is_some() {
            "BLOCKED-WASI-SDK (frozen-duckdb-sys build.rs refusal: WASI_SDK_PATH required \
             — set WASI_SDK_PATH to a wasip1 SDK; the legacy -lduckdb linker mode is retired)"
        } else {
            "BLOCKED-LINK"
        };
        panic!(
            "GRAPHLAW-WASM-BUILD-FAILED ({}): `cargo build -p graphlaw-wasm \
             --target wasm32-wasip1 --profile wasm` exited with {status}\n\
             {}linker line: {linker_line}\n\
             --- cargo stderr ---\n{stderr}--- end stderr ---\n\
             See docs/WASM.md for the wasm link requirements (frozen-duckdb \
             native symbol `duckdb_*` must resolve for wasm32-wasip1).",
            label,
            if wasi_sdk_refusal.is_some() {
                "typed refusal (first match): "
            } else {
                ""
            },
            status = out.status,
        );
    }
    let artifact = target.join("wasm32-wasip1/wasm/graphlaw_wasm.wasm");
    assert!(
        artifact.is_file(),
        "GRAPHLAW-WASM-BUILD-FAILED: cargo reported success but artifact is missing: {}",
        artifact.display()
    );
    artifact
}

struct Host {
    store: Store<WasiP1Ctx>,
    memory: Memory,
    alloc: TypedFunc<u32, u32>,
    free: TypedFunc<(u32, u32), ()>,
    call: TypedFunc<(u32, u32), u64>,
}

impl Host {
    fn new() -> Self {
        let bytes = std::fs::read(wasm_path()).expect("wasm artifact");
        Self::from_bytes(&bytes).expect("hosts the wasm module")
    }

    fn from_bytes(bytes: &[u8]) -> Result<Self, String> {
        let mut cfg = Config::new();
        cfg.wasm_exceptions(true);
        let engine = Engine::new(&cfg).map_err(|e| format!("engine: {e}"))?;
        let module = Module::new(&engine, bytes).map_err(|e| format!("module: {e}"))?;
        // The only imports allowed are WASI's (clock, random, stdio): no JavaScript glue.
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
        // `Linker::instantiate` runs the start section, then the host calls
        // `_initialize` once for reactor-style modules.
        let instance: Instance = linker
            .instantiate(&mut store, &module)
            .map_err(|e| format!("instantiate: {e}"))?;
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

    fn request_bytes(&mut self, req: &Value) -> Vec<u8> {
        let body = req.to_string().into_bytes();
        let ptr = self.alloc.call(&mut self.store, body.len() as u32).unwrap();
        self.memory
            .write(&mut self.store, ptr as usize, &body)
            .unwrap();
        let packed = self
            .call
            .call(&mut self.store, (ptr, body.len() as u32))
            .unwrap();
        let (out_ptr, out_len) = ((packed >> 32) as u32, (packed & 0xffff_ffff) as u32);
        let mut out = vec![0u8; out_len as usize];
        self.memory
            .read(&self.store, out_ptr as usize, &mut out)
            .unwrap();
        self.free.call(&mut self.store, (out_ptr, out_len)).unwrap();
        out
    }
}

/// One shared instance: instantiation is the expensive part.
fn host() -> &'static Mutex<Host> {
    static H: OnceLock<Mutex<Host>> = OnceLock::new();
    H.get_or_init(|| Mutex::new(Host::new()))
}

/// Raw response bytes from the native `graphlaw::abi`.
pub fn native_bytes(req: &Value) -> Vec<u8> {
    graphlaw::abi::call(req.to_string().as_bytes())
}

/// Parsed response from the native `graphlaw::abi`.
pub fn native(req: &Value) -> Value {
    serde_json::from_slice(&native_bytes(req)).expect("native response is JSON")
}

/// Raw response bytes from the compiled wasm module.
pub fn wasm_bytes(req: &Value) -> Vec<u8> {
    host()
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .request_bytes(req)
}

/// Parsed response from the compiled wasm module.
pub fn wasm(req: &Value) -> Value {
    serde_json::from_slice(&wasm_bytes(req)).expect("wasm response is JSON")
}

fn both(req: &Value) -> (Vec<u8>, Vec<u8>) {
    let n = native_bytes(req);
    let w = wasm_bytes(req);
    assert!(
        n == w,
        "native and wasm responses diverge for {req}\nnative: {}\nwasm:   {}",
        String::from_utf8_lossy(&n),
        String::from_utf8_lossy(&w)
    );
    (n, w)
}

/// Runs `req` on native and wasm; asserts `ok == true` on both and byte identity.
pub fn ok(req: &Value) -> Value {
    let (n, _) = both(req);
    let v: Value = serde_json::from_slice(&n).expect("response is JSON");
    assert_eq!(v["ok"], Value::Bool(true), "request {req} failed: {v}");
    v
}

/// Runs `req` on native and wasm; asserts `ok == false` on both and byte identity.
pub fn refused(req: &Value) -> Value {
    let (n, _) = both(req);
    let v: Value = serde_json::from_slice(&n).expect("response is JSON");
    assert_eq!(
        v["ok"],
        Value::Bool(false),
        "request {req} was admitted: {v}"
    );
    v
}

/// Reads a repo-relative file.
pub fn read(path: &str) -> String {
    std::fs::read_to_string(format!("{}/{path}", env!("CARGO_MANIFEST_DIR")))
        .unwrap_or_else(|e| panic!("read {path}: {e}"))
}

// ---------------------------------------------------------------------------
// engine_court: the shared wasm-engine harness for the finance-domain courts
// (bcbs_aggregation, dvp_settlement, ubo_sanctions). Reduced verbatim from
// tests/bcbs_aggregation_court.rs — the trio's copies were identical modulo
// comment/format drift; ubo omits the /tmp WASI preopen, so `preopen_tmp`
// parameterizes it instead of widening every court's capability grant.
// ---------------------------------------------------------------------------

pub mod engine_court {
    use std::path::PathBuf;
    use std::sync::Mutex;

    use serde_json::{Value, json};
    use wasmtime::{Config, Engine, Instance, Linker, Memory, Module, Store, TypedFunc};
    use wasmtime_wasi::WasiCtxBuilder;
    use wasmtime_wasi::p1::{WasiP1Ctx, add_to_linker_sync};

    /// The frozen engine segfaults when engine-heavy tests run concurrently;
    /// serialize the wasm-engine courts with a process-local lock (structural
    /// sync, no sleeps).
    pub static ENGINE_LOCK: Mutex<()> = Mutex::new(());

    /// Lock without poison cascade: a court that panics (e.g. a pending-
    /// artifact `unknown op`) must not fail its siblings with PoisonError —
    /// each court reports its own outcome.
    pub fn engine_lock() -> std::sync::MutexGuard<'static, ()> {
        ENGINE_LOCK
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }

    /// The wasm artifact under test: `GRAPHLAW_WASM` overrides (and must exist
    /// when set), else the default target-dir build output. `None` = not built.
    pub fn wasm_path() -> Option<PathBuf> {
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

    /// Classified host failure: `Trap` ("uninitialized element" — stale or
    /// mismatched artifact) vs `Fatal` (everything else).
    #[derive(Debug)]
    pub enum HostError {
        /// gl_call/gl_free trapped on an uninitialized element.
        Trap(String),
        /// Deterministically fatal construction/transport failure.
        Fatal(String),
    }

    /// One wasmtime host around a compiled module: wasi_snapshot_preview1
    /// imports plus the gl_alloc/gl_call/gl_free JSON-over-linear-memory ABI.
    pub struct Host {
        store: Store<WasiP1Ctx>,
        memory: Memory,
        alloc: TypedFunc<u32, u32>,
        free: TypedFunc<(u32, u32), ()>,
        call: TypedFunc<(u32, u32), u64>,
    }

    impl Host {
        /// Compile + instantiate. `preopen_tmp` grants the module `/tmp`
        /// ReadWrite (bcbs/dvp grant it; ubo does not).
        pub fn new(bytes: &[u8], preopen_tmp: bool) -> Result<Self, HostError> {
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
            let wasi = if preopen_tmp {
                WasiCtxBuilder::new()
                    .inherit_stdio()
                    .inherit_env()
                    .preopened_dir("/tmp", "/tmp", wasmtime_wasi::FsPerms::ReadWrite)
                    .map_err(|e| HostError::Fatal(format!("preopen /tmp: {e}")))?
                    .build_p1()
            } else {
                WasiCtxBuilder::new().inherit_stdio().inherit_env().build_p1()
            };
            let mut store = Store::new(&engine, wasi);
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

        pub fn request_bytes(&mut self, body: &[u8]) -> Result<Vec<u8>, HostError> {
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

        pub fn request(&mut self, req: &Value) -> Result<Value, HostError> {
            let out = self.request_bytes(req.to_string().as_bytes())?;
            serde_json::from_slice(&out)
                .map_err(|e| HostError::Fatal(format!("response is not JSON: {e}")))
        }

        pub fn exec(&mut self, req: &Value) -> Value {
            let r = self.request(req).expect("request transport");
            assert_eq!(r["ok"], json!(true), "exec request failed: {req} -> {r}");
            r
        }

        /// Stream a cursor fully and return the N-Triples body.
        pub fn stream_all(&mut self, session: &str, cursor: &str) -> String {
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

    /// Classify a wasmtime invocation error by its trap shape.
    pub fn fail_classify(stage: &str, e: wasmtime::Error) -> HostError {
        let s = format!("{stage}: {e}");
        if e.to_string().contains("uninitialized element") {
            HostError::Trap(s)
        } else {
            HostError::Fatal(s)
        }
    }

    /// Build the host or report the typed failure reason.
    pub fn try_host(preopen_tmp: bool) -> Result<Host, String> {
        let path = wasm_path().ok_or_else(|| "artifact-missing".to_string())?;
        let bytes =
            std::fs::read(&path).map_err(|e| format!("fatal: read {}: {e}", path.display()))?;
        match Host::new(&bytes, preopen_tmp) {
            Ok(h) => Ok(h),
            Err(HostError::Trap(e)) => Err(format!("trap: {e}")),
            Err(HostError::Fatal(e)) => Err(format!("fatal: {e}")),
        }
    }

    /// The court's harness or a loud typed SKIP (a SKIP is not a pass).
    pub fn host_or_skip(test_name: &str, preopen_tmp: bool) -> Option<Host> {
        match try_host(preopen_tmp) {
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

    /// Open a named duckdb session (duplicate names are refused).
    pub fn open_session(h: &mut Host, name: &str) {
        let r = h.exec(&json!({"op": "duckdb", "mode": "open", "session": name}));
        assert_eq!(r["session"], json!(name), "session open reply: {r}");
    }

    /// Close a named duckdb session.
    pub fn close_session(h: &mut Host, name: &str) {
        h.exec(&json!({"op": "duckdb", "mode": "close", "session": name}));
    }
}
