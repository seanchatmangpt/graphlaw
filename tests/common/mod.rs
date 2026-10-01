//! Shared harness for tests that drive the *compiled* wasm module beside the
//! native `graphlaw::abi` (same requests, byte-identical responses expected).
//!
//! Extracted from `tests/wasm_abi.rs`. Set `GRAPHLAW_WASM` to test a prebuilt
//! module; otherwise it is built once into `target/wasm-abi`.
//!
//! Public API (frozen for v26.9.29): `native`, `native_bytes`, `wasm`,
//! `wasm_bytes`, `ok`, `refused`, `read`.
#![cfg(all(feature = "abi", not(target_arch = "wasm32")))]
#![allow(dead_code)]

use std::path::PathBuf;
use std::process::Command;
use std::sync::{Mutex, OnceLock};

use serde_json::Value;
use wasmi::{Engine, Instance, Linker, Memory, Module, Store, TypedFunc};
use wasmi_wasi::{WasiCtx, WasiCtxBuilder};

/// Path of the wasm artifact: `GRAPHLAW_WASM`, else a one-time build.
pub fn wasm_path() -> PathBuf {
    if let Some(p) = std::env::var_os("GRAPHLAW_WASM") {
        return p.into();
    }
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let target = root.join("target/wasm-abi");
    let status = Command::new(std::env::var("CARGO").unwrap_or_else(|_| "cargo".into()))
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
        .status()
        .expect("cargo runs");
    assert!(
        status.success(),
        "wasm build failed (is the wasm32-wasip1 target installed?)"
    );
    target.join("wasm32-wasip1/wasm/graphlaw_wasm.wasm")
}

struct Host {
    store: Store<WasiCtx>,
    memory: Memory,
    alloc: TypedFunc<u32, u32>,
    free: TypedFunc<(u32, u32), ()>,
    call: TypedFunc<(u32, u32), u64>,
}

impl Host {
    fn new() -> Self {
        let bytes = std::fs::read(wasm_path()).expect("wasm artifact");
        let engine = Engine::default();
        let module = Module::new(&engine, &bytes[..]).expect("valid wasm");
        // The only imports allowed are WASI's (clock, random, stdio): no JavaScript glue.
        for i in module.imports() {
            assert_eq!(
                i.module(),
                "wasi_snapshot_preview1",
                "unexpected host import {}::{}",
                i.module(),
                i.name()
            );
        }
        let mut store = Store::new(&engine, WasiCtxBuilder::new().build());
        let mut linker = <Linker<WasiCtx>>::new(&engine);
        wasmi_wasi::add_to_linker(&mut linker, |ctx| ctx).expect("links WASI");
        let instance: Instance = linker
            .instantiate_and_start(&mut store, &module)
            .expect("instantiates");
        // Reactor-style modules expose `_initialize`; hosts must call it once.
        if let Ok(init) = instance.get_typed_func::<(), ()>(&store, "_initialize") {
            init.call(&mut store, ()).expect("_initialize");
        }
        let memory = instance
            .get_memory(&store, "memory")
            .expect("exports memory");
        Host {
            alloc: instance.get_typed_func(&store, "gl_alloc").unwrap(),
            free: instance.get_typed_func(&store, "gl_free").unwrap(),
            call: instance.get_typed_func(&store, "gl_call").unwrap(),
            store,
            memory,
        }
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
