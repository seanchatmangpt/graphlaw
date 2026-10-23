//! GWS-11 - memory-wall court on the REAL compiled wasm module.
//!
//! Adversarial memory exhaustion through the JSON ABI, exactly as a host
//! (Elixir/Wasmex, or the smalt port driver with wasmtime --max-memory)
//! would. Three attack families, per the spec gates:
//!
//! (a) bulk focus-node injection past the linear-memory budget: gl_alloc
//!     refuses any length over MAX_REQUEST_BYTES (16 MiB) with a null pointer
//!     (never a trap); the typed request_bytes ResourceLimit behind the cap is
//!     reachable only through a hostile host misreporting len to gl_call, and
//!     is asserted here; gl_call(0, 0) yields the typed missing-buffer
//!     refusal. A host that allocates and never frees cannot fill linear
//!     memory: gl_alloc returns null once MAX_OUTSTANDING_BYTES (16 MiB, GAP-1) is
//!     outstanding, and freeing restores the budget exactly.
//! (b) a compute-explosive workload: N3 string-doubling (nested expansion
//!     inside the rules evaluator) plus a depth-explosive JSON body - both
//!     refuse with typed ResourceLimit before memory or the wall clock die.
//! (c) accumulation probe: hundreds of repeated evals without host-side
//!     cleanup leave linear memory flat (bounded allocator retention).
//!
//! Survival law (spec Gates 2/3): the instance dies bounded - every breach is
//! OOM-typed (null alloc, typed ResourceLimit/Unsupported refusal, or a wasmi
//! trap), never UB - and the host process + a fresh instance (the respawn
//! analog of Smalt.Wasm.Pool EXIT-driven respawn in smalt
//! lib/smalt/wasm/pool.ex) keep serving.
//!
//! Enforced-boundary map (asserted, not narrated):
//! - MAX_REQUEST_BYTES (16,777,216 B) - enforced INSIDE the module at
//!   gl_alloc (null return, wasm/src/ffi.rs alloc_buf) for honest hosts, and
//!   at the typed request_bytes ResourceLimit (graphlaw/src/abi.rs call)
//!   behind a hostile len misreport to gl_call.
//! - MAX_OUTSTANDING_BYTES (16,777,216 B, GAP-1 re-baseline) - enforced INSIDE the module
//!   (ffi.rs reserve): bounds a host's outstanding, unfreed buffers.
//! - Linear-memory ceiling - NOT declared by the imported memory's type
//!   (maximum is None); in raw WASI the guest grows memory per its own
//!   allocation discipline, so the spec's hard 64 MiB wall (smalt
//!   docs/jira/v26.10.21/WASM-SANDBOX-SPEC.md) is enforced at the HOST layer
//!   (wasmtime run --max-memory 67108864 in smalt_wasm_worker), and here at
//!   the module's own request/outstanding caps plus wasmi's typed trap on a
//!   failed memory.grow.
#![cfg(not(target_arch = "wasm32"))]

use std::path::PathBuf;
use std::process::Command;
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};

use serde_json::{Value, json};
use wasmi::{Engine, Instance, Linker, Memory, Module, Store, TypedFunc};
use wasmi_wasi::{WasiCtx, WasiCtxBuilder};

/// Spec ABI pins (wasm/src/abi_meta.rs, ggen-rendered).
const MAX_REQUEST_BYTES: u32 = 16_777_216;
// GAP-1 (docs/wasm-duckdb-op15-design.md): re-derived to 16 MiB so an
// alloc-success-then-OOM-trap is impossible under the 64 MiB host wall.
const MAX_OUTSTANDING_BYTES: usize = 16_777_216;
const MAX_JSON_DEPTH: u32 = 64;

/// Resolve the real artifact. GWS-2 bounded poll on the build state: the
/// observable is the artifact's existence (deadline-checked); a missing
/// binary is a TYPED skip (panic naming the skip), never a fake pass.
fn wasm_path() -> PathBuf {
    if let Some(p) = std::env::var_os("GRAPHLAW_WASM")
        .filter(|p| !p.is_empty())
    {
        return p.into();
    }
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let target = root.join("target/wasm-abi");
    let artifact = target.join("wasm32-wasip1/wasm/graphlaw_wasm.wasm");
    if !artifact.exists() {
        let cargo = std::env::var("CARGO").unwrap_or_else(|_| "cargo".into());
        let status = Command::new(cargo)
            .current_dir(&root)
            .args([
                "build", "-p", "graphlaw-wasm",
                "--target", "wasm32-wasip1",
                "--profile", "wasm",
                "--target-dir",
            ])
            .arg(&target)
            .status()
            .expect("cargo runs");
        assert!(status.success(), "wasm build failed");
    }
    // Bounded deadline poll on the observable (artifact existence); no fixed
    // sleep — yield to the scheduler while the deadline holds (no-sleep law).
    let deadline = Instant::now() + Duration::from_secs(30);
    while !artifact.exists() && Instant::now() < deadline {
        std::thread::yield_now();
    }
    artifact
}

struct Host {
    store: Store<WasiCtx>,
    memory: Memory,
    alloc_fn: TypedFunc<u32, u32>,
    free_fn: TypedFunc<(u32, u32), ()>,
    call_fn: TypedFunc<(u32, u32), u64>,
}

impl Host {
    fn new() -> Self {
        let path = wasm_path();
        let bytes = std::fs::read(&path)
            .unwrap_or_else(|e| panic!("TYPED-SKIP GWS-11: no wasm binary at {} ({e})", path.display()));
        let engine = Engine::default();
        let module = Module::new(&engine, &bytes[..]).expect("valid wasm");
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
        if let Ok(init) = instance.get_typed_func::<(), ()>(&store, "_initialize") {
            init.call(&mut store, ()).expect("_initialize");
        }
        let memory = instance.get_memory(&store, "memory").expect("exports memory");
        Host {
            alloc_fn: instance.get_typed_func(&store, "gl_alloc").unwrap(),
            free_fn: instance.get_typed_func(&store, "gl_free").unwrap(),
            call_fn: instance.get_typed_func(&store, "gl_call").unwrap(),
            store,
            memory,
        }
    }

    fn alloc(&mut self, len: u32) -> Option<u32> {
        let p = self.alloc_fn.call(&mut self.store, len).expect("gl_alloc runs");
        (p != 0).then_some(p)
    }

    fn call_raw(&mut self, ptr: u32, len: u32) -> (u32, u32) {
        let packed = self.call_fn.call(&mut self.store, (ptr, len)).expect("gl_call runs");
        ((packed >> 32) as u32, packed as u32)
    }

    fn write_body(&mut self, ptr: u32, body: &[u8]) {
        self.memory.write(&mut self.store, ptr as usize, body).unwrap();
    }

    fn read_out(&mut self, ptr: u32, len: u32) -> Vec<u8> {
        let mut out = vec![0u8; len as usize];
        self.memory.read(&self.store, ptr as usize, &mut out).expect("in bounds");
        self.free_fn.call(&mut self.store, (ptr, len)).expect("gl_free runs");
        out
    }

    fn peek_out(&mut self, ptr: u32, len: u32) -> Vec<u8> {
        let mut out = vec![0u8; len as usize];
        self.memory.read(&self.store, ptr as usize, &mut out).unwrap();
        out
    }

    fn free(&mut self, ptr: u32, len: u32) {
        self.free_fn.call(&mut self.store, (ptr, len)).expect("gl_free runs");
    }

    /// Legal request/response cycle: alloc, write, call, read + free response.
    fn request(&mut self, req: &Value) -> Value {
        let body = req.to_string().into_bytes();
        let ptr = self.alloc(body.len() as u32).expect("legal request fits budget");
        self.write_body(ptr, &body);
        let (out_ptr, out_len) = self.call_raw(ptr, body.len() as u32);
        let out = self.read_out(out_ptr, out_len);
        serde_json::from_slice(&out).expect("response is JSON")
    }

    /// Request cycle freeing NEITHER buffer (leak probe).
    fn request_leaking(&mut self, req: &Value) -> Value {
        let body = req.to_string().into_bytes();
        let ptr = self.alloc(body.len() as u32).expect("leak probe fits budget");
        self.write_body(ptr, &body);
        let (out_ptr, out_len) = self.call_raw(ptr, body.len() as u32);
        let out = self.peek_out(out_ptr, out_len);
        serde_json::from_slice(&out).expect("response is JSON")
    }

    fn memory_pages(&mut self) -> u64 {
        self.memory.size(&self.store)
    }

    fn memory_max_pages(&mut self) -> Option<u64> {
        self.memory.ty(&self.store).maximum()
    }
}

/// Shared instance for lightweight probes; saturated instances are
/// test-local (private) and dropped (respawn analog).
fn host() -> &'static Mutex<Host> {
    static H: OnceLock<Mutex<Host>> = OnceLock::new();
    H.get_or_init(|| Mutex::new(Host::new()))
}

fn call(req: Value) -> Value {
    host().lock().unwrap_or_else(|e| e.into_inner()).request(&req)
}

fn ok(req: Value) -> Value {
    let r = call(req.clone());
    assert_eq!(r["ok"], json!(true), "request {req} failed: {r}");
    r
}

fn assert_typed_limit(r: &Value, limit: &str, max: u32) {
    assert_eq!(r["ok"], json!(false), "{r}");
    assert_eq!(r["error"]["kind"], "ResourceLimit", "{r}");
    assert_eq!(r["error"]["details"]["code"], "ResourceLimit", "{r}");
    assert_eq!(r["error"]["details"]["limit"], limit, "{r}");
    assert_eq!(r["error"]["details"]["max"], json!(max), "{r}");
}

fn padded_capabilities(total: usize) -> Vec<u8> {
    let head = br#"{"op":"capabilities","pad":""#;
    let tail = br#""}"#;
    let mut b = Vec::with_capacity(total);
    b.extend_from_slice(head);
    b.resize(total - tail.len(), b'x');
    b.extend_from_slice(tail);
    assert_eq!(b.len(), total);
    b
}

const SAMPLE: &str = "@prefix ex: <https://e/> . ex:s ex:p ex:o .";

#[test]
fn boundary_map_memory_has_no_declared_maximum_module_caps_hold_the_line() {
    let mut h = host().lock().unwrap_or_else(|e| e.into_inner());
    // The module does NOT declare a memory maximum: the spec's 64 MiB wall is
    // HOST-layer enforcement (wasmtime --max-memory), not a wasm-layer one.
    assert_eq!(
        h.memory_max_pages(),
        None,
        "binary now declares a memory maximum; update this court's boundary map"
    );
    drop(h);

    // At the cap exactly, the request still works (16 MiB honest buffer).
    // Private instance: an at-cap alloc consumes the whole outstanding budget
    // (16 MiB = cap), so this probe must not share the module with the other
    // courts.
    let mut h = Host::new();
    let body = padded_capabilities(MAX_REQUEST_BYTES as usize);
    let ptr = h
        .alloc(body.len() as u32)
        .expect("at-cap alloc must succeed");
    h.write_body(ptr, &body);
    let (op, olen) = h.call_raw(ptr, body.len() as u32);
    let out = h.read_out(op, olen);
    let r: Value = serde_json::from_slice(&out).expect("response is JSON");
    drop(h);
    assert_eq!(r["ok"], json!(true), "at-cap request must still work: {r}");
}

#[test]
fn bulk_focus_node_injection_over_request_cap_is_typed_never_a_trap() {
    // gl_alloc over the request cap returns null (never traps); a hostile
    // host that misreports len to gl_call gets the typed request_bytes
    // ResourceLimit; gl_call(0, 0) yields the typed missing-buffer refusal.
    // Private instance: the over-cap null check and the at-cap alloc (16 MiB
    // fills the outstanding budget) must not starve the shared instance.
    let mut h = Host::new();
    assert_eq!(
        h.alloc(MAX_REQUEST_BYTES + 1),
        None,
        "gl_alloc over the request cap must return null, not trap"
    );
    let _ptr = h
        .alloc(MAX_REQUEST_BYTES)
        .expect("at-cap alloc must succeed");
    drop(h);
    // Hostile-misreport leg rides the shared host with a small buffer.
    let mut h = host().lock().unwrap_or_else(|e| e.into_inner());
    let small = padded_capabilities(64);
    let ptr = h.alloc(small.len() as u32).expect("small alloc");
    h.write_body(ptr, &small);
    let (op, olen) = h.call_raw(ptr, MAX_REQUEST_BYTES + 1); // hostile misreport
    let out = h.read_out(op, olen);
    let r: Value = serde_json::from_slice(&out).expect("response is JSON");
    drop(h);
    assert_typed_limit(&r, "request_bytes", MAX_REQUEST_BYTES);

    let mut h = host().lock().unwrap_or_else(|e| e.into_inner());
    let (op, olen) = h.call_raw(0, 0);
    let out = h.read_out(op, olen);
    let r: Value = serde_json::from_slice(&out).expect("response is JSON");
    drop(h);
    assert_eq!(r["ok"], json!(false), "{r}");
    assert_eq!(r["error"]["kind"], "Unsupported", "{r}");
    let msg = r["error"]["message"].as_str().unwrap_or("");
    assert!(
        msg.contains("request buffer missing"),
        "unexpected refusal text: {msg}"
    );
}

#[test]
fn host_that_never_frees_cannot_fill_linear_memory_and_free_restores_the_budget() {
    // Outstanding-byte cap (256 MiB): a leaking host hits null alloc, not UB.
    const CHUNK: u32 = 1 << 20; // 1 MiB
    let max_chunks = MAX_OUTSTANDING_BYTES / CHUNK as usize;
    // Private instance: this probe deliberately saturates the outstanding
    // budget, which would starve the other courts sharing host().
    let mut h = Host::new();
    let mut held: Vec<(u32, u32)> = Vec::new();
    let refused_at = loop {
        match h.alloc(CHUNK) {
            Some(p) => {
                held.push((p, CHUNK));
                assert!(
                    held.len() <= max_chunks,
                    "outstanding cap never fired after {} MiB - the module leaked",
                    held.len()
                );
            }
            None => break held.len(),
        }
    };
    assert!(
        refused_at >= max_chunks - 1 && refused_at <= max_chunks,
        "cap fired at {refused_at} MiB outstanding; expected ~{max_chunks}"
    );
    while let Some((p, l)) = held.pop() {
        h.free(p, l);
    }
    let r = h.request(&json!({"op": "capabilities"}));
    assert_eq!(r["ok"], json!(true), "{r}");
}

#[test]
fn compute_explosive_n3_doubling_refuses_typed_before_memory_dies() {
    // Nested expansion inside the rules evaluator: each firing doubles the
    // literal. Typed refusal, bounded in time, instance still usable.
    let req = json!({"op": "n3", "text": concat!(
        "@prefix : <urn:h> .\n",
        "@prefix string: <http://www.w3.org/2000/10/swap/string#> .\n",
        ":a :s \"yy\" .\n",
        "{ :a :s ?x . (?x ?x) string:concatenation ?y } => { :a :s ?y } .\n")});
    let started = Instant::now();
    let r = call(req);
    assert_eq!(r["ok"], json!(false), "{r}");
    assert_eq!(r["error"]["kind"], "ResourceLimit", "{r}");
    assert!(
        started.elapsed() < Duration::from_secs(60),
        "compute-explosive workload took {:?} - not bounded",
        started.elapsed()
    );
    let r = ok(json!({"op": "parse", "text": SAMPLE, "dialect": "turtle"}));
    assert_eq!(r["quads"], 1);
}

#[test]
fn depth_explosive_json_refuses_with_typed_json_depth_limit() {
    let mut body = String::new();
    for _ in 0..200 {
        body.push('[');
    }
    body.push_str("\"x\"");
    for _ in 0..200 {
        body.push(']');
    }
    let raw = format!(r#"{{"op":"capabilities","pad":{body}}}"#);
    let mut h = host().lock().unwrap_or_else(|e| e.into_inner());
    let ptr = h.alloc(raw.len() as u32).expect("depth probe alloc fits");
    h.write_body(ptr, raw.as_bytes());
    let (op, olen) = h.call_raw(ptr, raw.len() as u32);
    let out = h.read_out(op, olen);
    let r: Value = serde_json::from_slice(&out).expect("response is JSON");
    drop(h);
    assert_typed_limit(&r, "json_depth", MAX_JSON_DEPTH);
}

#[test]
fn repeated_evals_without_host_cleanup_hit_a_bounded_memory_plateau() {
    // Accumulation probe on the shared host: leak BOTH buffers on every call
    // (request + response never freed). Constant-size evals must keep linear
    // memory flat while every eval still returns typed ok. ~110 bytes leaked
    // per eval x 1200 stays far under the 256 MiB outstanding cap; the
    // saturating variant is the never-frees court (private instance).
    let mut h = host().lock().unwrap_or_else(|e| e.into_inner());
    let leak_req = json!({"op": "parse", "text": SAMPLE, "dialect": "turtle"});
    let s0 = h.memory_pages();
    for i in 0..600 {
        let r = h.request_leaking(&leak_req);
        assert_eq!(r["ok"], json!(true), "eval {i} failed: {r}");
    }
    let s1 = h.memory_pages();
    for i in 0..600 {
        let r = h.request_leaking(&leak_req);
        assert_eq!(r["ok"], json!(true), "eval {i} failed: {r}");
    }
    let s2 = h.memory_pages();
    assert!(
        s1.saturating_sub(s0) <= 8 && s2.saturating_sub(s1) <= 8,
        "linear memory walked upward under repeated evals: {s0} -> {s1} -> {s2} pages"
    );
}

#[test]
fn exhausted_instance_dies_bounded_and_a_fresh_respawn_serves() {
    // Spec Gate 3 analog (Smalt.Wasm.Pool EXIT-driven respawn in smalt
    // lib/smalt/wasm/pool.ex): saturate an instance, type its last words,
    // drop it, then bring up a FRESH instance that admits valid work.
    const CHUNK: u32 = 4 << 20; // 4 MiB
    {
        let mut h = Host::new();
        while h.alloc(CHUNK).is_some() {
            // leak until the module refuses
        }
        // The module never UBs under saturation: a null-buffer call is typed.
        let (op, olen) = h.call_raw(0, 0);
        let out = h.read_out(op, olen);
        let r: Value = serde_json::from_slice(&out).expect("response is JSON");
        assert_eq!(r["ok"], json!(false), "{r}");
        drop(h); // the saturated instance is discarded
    }
    // Respawn: a fresh instance admits valid work immediately.
    let mut fresh = Host::new();
    let r = fresh.request(&json!({"op": "capabilities"}));
    assert_eq!(r["ok"], json!(true), "{r}");
    assert_eq!(r["abi"], 1);
}
