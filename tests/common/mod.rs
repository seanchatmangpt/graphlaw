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
