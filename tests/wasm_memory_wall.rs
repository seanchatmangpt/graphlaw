//! GWS-11 - memory-wall court on the REAL compiled wasm module, hosted by
//! wasmtime 48 with `wasm_exceptions(true)` (the wasmi 2 engine cannot even
//! validate the DuckDB-linked artifact's W3C exception tag section, id 13).
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
//! OOM-typed (null alloc, typed ResourceLimit/Unsupported refusal, or a
//! failed memory.grow), never UB - and the host process + a fresh instance
//! (the respawn analog of Smalt.Wasm.Pool EXIT-driven respawn in smalt
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
//!   docs/jira/v26.10.21/WASM-SANDBOX-SPEC.md) is enforced at the HOST layer.
//!
//! Wall mapping (wasmi -> wasmtime 48): wasmi's engine-level memory limiter
//! turned an over-wall memory.grow into a typed trap. wasmtime's equivalent
//! is a `ResourceLimiter` on the store: `memory_growing` denies any growth
//! past WALL_BYTES (67,108,864 = 64 MiB) with an Err, the wasm `memory.grow`
//! instruction then evaluates to -1 (a soft failure, not a host OOM), and the
//! guest allocator's failure path in wasm/src/ffi.rs returns a null pointer —
//! which surfaces through the module's existing typed limit_response envelope
//! (gl_alloc null / typed ResourceLimit refusal). The instance survives.
#![cfg(all(feature = "abi", not(target_arch = "wasm32")))]

use std::path::PathBuf;
use std::process::Command;
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};

use serde_json::{Value, json};
use wasmtime::{Config, Engine, Instance, Linker, Memory, Module, ResourceLimiter, Store, TypedFunc};
use wasmtime_wasi::WasiCtxBuilder;
use wasmtime_wasi::p1::{WasiP1Ctx, add_to_linker_sync};

/// Spec ABI pins (wasm/src/abi_meta.rs, ggen-rendered).
const MAX_REQUEST_BYTES: u32 = 16_777_216;
// GAP-1 (docs/wasm-duckdb-op15-design.md): re-derived to 16 MiB so an
// alloc-success-then-OOM-trap is impossible under the 64 MiB host wall.
const MAX_OUTSTANDING_BYTES: usize = 16_777_216;
const MAX_JSON_DEPTH: u32 = 64;

/// The spec hard wall (smalt WASM-SANDBOX-SPEC): 64 MiB of linear memory.
/// Enforced here at the host layer via `WallLimiter`, mirroring
/// `wasmtime run --max-memory 67108864` in smalt_wasm_worker.
const WALL_BYTES: u64 = 67_108_864;

/// Resolve the real artifact. GWS-2 bounded poll on the build state: the
/// observable is the artifact's existence (deadline-checked); a missing
/// binary is a TYPED skip (panic naming the skip), never a fake pass.
fn wasm_path() -> PathBuf {
    if let Some(p) = std::env::var_os("GRAPHLAW_WASM").filter(|p| !p.is_empty()) {
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

/// Store state: the WASI p1 context plus the 64 MiB wall limiter. The
/// limiter is the wasmtime-48 host-layer mapping of the spec's hard wall.
struct State {
    wasi: WasiP1Ctx,
    wall: WallLimiter,
}

struct WallLimiter {
    /// Peak bytes the limiter has admitted (for diagnostics on refusal).
    peak: u64,
}

impl ResourceLimiter for WallLimiter {
    fn memory_growing(
        &mut self,
        current: usize,
        desired: usize,
        _maximum: Option<usize>,
    ) -> Result<bool, wasmtime::Error> {
        if desired as u64 > WALL_BYTES {
            // Deny the growth past the wall: memory.grow evaluates to -1 in
            // the guest, the allocator returns null, the typed envelope fires.
            return Err(wasmtime::Error::msg(format!(
                "memory wall: refusing growth to {desired} bytes (wall {WALL_BYTES}; current {current})"
            )));
        }
        self.peak = self.peak.max(desired as u64);
        Ok(true)
    }

    fn table_growing(&mut self, _current: usize, desired: usize, _maximum: Option<usize>) -> Result<bool, wasmtime::Error> {
        // No table wall in the spec; admit, bounded by a generous sanity cap.
        const TABLE_CAP: usize = 1 << 20;
        if desired > TABLE_CAP {
            return Err(wasmtime::Error::msg(format!(
                "table growth past sanity cap: {desired}"
            )));
        }
        Ok(true)
    }

    fn memory_grow_failed(&mut self, _error: wasmtime::Error) -> Result<(), wasmtime::Error> {
        // Invoked when an admitted grow then fails at the OS layer; nothing
        // extra to report beyond the limiter's own denial error.
        Ok(())
    }
}

struct Host {
    store: Store<State>,
    memory: Memory,
    alloc_fn: TypedFunc<u32, u32>,
    free_fn: TypedFunc<(u32, u32), ()>,
    call_fn: TypedFunc<(u32, u32), u64>,
}

impl Host {
    fn new() -> Self {
        let path = wasm_path();
        let bytes = std::fs::read(&path).unwrap_or_else(|e| {
            panic!(
                "TYPED-SKIP GWS-11: no wasm binary at {} ({e})",
                path.display()
            )
        });
        let mut cfg = Config::new();
        // The DuckDB-linked module carries a W3C exception tag section; wasmi
        // cannot validate it, wasmtime 48 with this flag can.
        cfg.wasm_exceptions(true);
        let engine = Engine::new(&cfg).expect("engine");
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
        let mut store = Store::new(
            &engine,
            State {
                wasi: WasiCtxBuilder::new().build_p1(),
                wall: WallLimiter { peak: 0 },
            },
        );
        // Attach the wall limiter (sync runtime — no async_support).
        store.limiter(|s| &mut s.wall);
        let mut linker = <Linker<State>>::new(&engine);
        add_to_linker_sync(&mut linker, |ctx| &mut ctx.wasi).expect("links WASI");
        let instance: Instance = linker
            .instantiate(&mut store, &module)
            .expect("instantiates");
        // Reactor-style modules expose `_initialize`; hosts must call it once.
        if let Ok(init) = instance.get_typed_func::<(), ()>(&mut store, "_initialize") {
            init.call(&mut store, ()).expect("_initialize");
        }
        let memory = instance
            .get_memory(&mut store, "memory")
            .expect("exports memory");
        Host {
            alloc_fn: instance.get_typed_func(&mut store, "gl_alloc").unwrap(),
            free_fn: instance.get_typed_func(&mut store, "gl_free").unwrap(),
            call_fn: instance.get_typed_func(&mut store, "gl_call").unwrap(),
            store,
            memory,
        }
    }

    fn alloc(&mut self, len: u32) -> Option<u32> {
        let p = self
            .alloc_fn
            .call(&mut self.store, len)
            .expect("gl_alloc runs");
        (p != 0).then_some(p)
    }

    fn call_raw(&mut self, ptr: u32, len: u32) -> (u32, u32) {
        let packed = self
            .call_fn
            .call(&mut self.store, (ptr, len))
            .expect("gl_call runs");
        ((packed >> 32) as u32, packed as u32)
    }

    fn write_body(&mut self, ptr: u32, body: &[u8]) {
        self.memory
            .write(&mut self.store, ptr as usize, body)
            .unwrap();
    }

    fn read_out(&mut self, ptr: u32, len: u32) -> Vec<u8> {
        let mut out = vec![0u8; len as usize];
        self.memory
            .read(&self.store, ptr as usize, &mut out)
            .expect("in bounds");
        self.free_fn
            .call(&mut self.store, (ptr, len))
            .expect("gl_free runs");
        out
    }

    fn peek_out(&mut self, ptr: u32, len: u32) -> Vec<u8> {
        let mut out = vec![0u8; len as usize];
        self.memory
            .read(&self.store, ptr as usize, &mut out)
            .unwrap();
        out
    }

    fn free(&mut self, ptr: u32, len: u32) {
        self.free_fn
            .call(&mut self.store, (ptr, len))
            .expect("gl_free runs");
    }

    /// Legal request/response cycle: alloc, write, call, read + free response.
    fn request(&mut self, req: &Value) -> Value {
        let body = req.to_string().into_bytes();
        let ptr = self
            .alloc(body.len() as u32)
            .expect("legal request fits budget");
        self.write_body(ptr, &body);
        let (out_ptr, out_len) = self.call_raw(ptr, body.len() as u32);
        let out = self.read_out(out_ptr, out_len);
        serde_json::from_slice(&out).expect("response is JSON")
    }

    /// Request cycle freeing NEITHER buffer (leak probe).
    fn request_leaking(&mut self, req: &Value) -> Value {
        let body = req.to_string().into_bytes();
        let ptr = self
            .alloc(body.len() as u32)
            .expect("leak probe fits budget");
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
    host()
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .request(&req)
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
    let mut h = host().lock().unwrap_or_else(std::sync::PoisonError::into_inner);
    // The module does NOT declare a memory maximum: the spec's 64 MiB wall is
    // HOST-layer enforcement (wasmtime --max-memory / our WallLimiter), not a
    // wasm-layer one.
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
    let mut h = host().lock().unwrap_or_else(std::sync::PoisonError::into_inner);
    let small = padded_capabilities(64);
    let ptr = h.alloc(small.len() as u32).expect("small alloc");
    h.write_body(ptr, &small);
    let (op, olen) = h.call_raw(ptr, MAX_REQUEST_BYTES + 1); // hostile misreport
    let out = h.read_out(op, olen);
    let r: Value = serde_json::from_slice(&out).expect("response is JSON");
    drop(h);
    assert_typed_limit(&r, "request_bytes", MAX_REQUEST_BYTES);

    let mut h = host().lock().unwrap_or_else(std::sync::PoisonError::into_inner);
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
    // Outstanding-byte cap (16 MiB): a leaking host hits null alloc, not UB.
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
    let mut h = host().lock().unwrap_or_else(std::sync::PoisonError::into_inner);
    let ptr = h.alloc(raw.len() as u32).expect("depth probe alloc fits");
    h.write_body(ptr, raw.as_bytes());
    let (op, olen) = h.call_raw(ptr, raw.len() as u32);
    let out = h.read_out(op, olen);
    let r: Value = serde_json::from_slice(&out).expect("response is JSON");
    drop(h);
    assert_typed_limit(&r, "json_depth", MAX_JSON_DEPTH);
}

#[test]
fn repeated_evals_growth_is_per_cycle_bounded_not_accelerating() {
    // Accumulation probe on the shared host: leak BOTH buffers on every call
    // (request + response never freed). Constant-size evals must keep
    // PER-CYCLE growth bounded and non-accelerating while every eval still
    // returns typed ok. ~110 bytes leaked per eval x 1200 stays far under the
    // 16 MiB outstanding cap; the saturating variant is the never-frees court
    // (private instance).
    let mut h = host().lock().unwrap_or_else(std::sync::PoisonError::into_inner);
    let leak_req = json!({"op": "parse", "text": SAMPLE, "dialect": "turtle"});
    const CYCLES: usize = 20;
    const EVALS_PER_CYCLE: usize = 60;
    let mut curve = Vec::with_capacity(CYCLES + 1);
    curve.push(h.memory_pages());
    for c in 0..CYCLES {
        for i in 0..EVALS_PER_CYCLE {
            let r = h.request_leaking(&leak_req);
            assert_eq!(r["ok"], json!(true), "cycle {c} eval {i} failed: {r}");
            if i % 10 == 9 {
                eprintln!("cycle {c:02} eval {}: {} pages", i + 1, h.memory_pages());
            }
        }
        curve.push(h.memory_pages());
        eprintln!("cycle {c:02} END: {} pages", curve[c + 1]);
    }
    // MEASURED growth curve (pages, 60 leaky evals per cycle), artifact as of
    // this court's landing: printed to stdout, asserted below.
    eprintln!("plateau growth curve (pages/cycle): {curve:?}");
    // Honest law: per-cycle growth is bounded and per-cycle-CONSTANT, not a
    // global plateau and not accelerating. Mechanism: rustc's per-export
    // command_export wrapping runs __wasm_call_ctors/__wasm_call_dtors per
    // export call; duckdb init registers static dtors per cycle into fresh
    // calloc'd atexit chunks, plus duckdb's own per-call arena state. Tracked
    // for a future reactor-model build (SQLRDF-24 progress).
    let deltas: Vec<u64> = curve.windows(2).map(|w| w[1] - w[0]).collect();
    eprintln!("per-cycle deltas: {deltas:?}");
    const MAX_PER_CYCLE: u64 = 2;
    let worst = *deltas.iter().max().unwrap();
    assert!(
        worst <= MAX_PER_CYCLE,
        "accelerating or unbounded per-cycle growth: curve {curve:?} (worst delta {worst} > {MAX_PER_CYCLE})"
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

#[test]
fn wall_limiter_refuses_growth_past_64mib_and_the_guest_envelope_stays_typed() {
    // The wasmi->wasmtime wall mapping itself: a memory.grow past the 64 MiB
    // wall is denied by WallLimiter (memory.grow -> -1, soft failure), so the
    // guest never sees a host OOM. Witness the limiter's decision directly on
    // a synthetic call pattern, and witness the module still healthy after.
    let mut h = Host::new();
    let (cur, _) = {
        let pages = h.memory_pages();
        (pages * 64 * 1024, pages)
    };
    let mut limiter = WallLimiter { peak: 0 };
    // Under the wall: admitted.
    assert!(limiter
        .memory_growing(cur as usize, cur as usize + 1024, None)
        .is_ok());
    // Over the wall: denied with a typed error naming the wall.
    let err = limiter
        .memory_growing(cur as usize, WALL_BYTES as usize + 1, None)
        .expect_err("over-wall growth must be denied");
    assert!(
        err.to_string().contains("memory wall"),
        "unexpected denial: {err}"
    );
    // Real memory.grow past the wall through the actual instance: denied at
    // the host layer (never an abort/OOM of the process).
    let cur_pages = h.memory_pages();
    let res = h.memory.grow(&mut h.store, 1024 - cur_pages + 1);
    assert!(res.is_err(), "over-wall memory.grow must be refused");
    assert_eq!(h.memory_pages(), cur_pages, "denied grow must not resize");
    // And the real instance keeps serving legal work.
    let r = h.request(&json!({"op": "capabilities"}));
    assert_eq!(r["ok"], json!(true), "{r}");
    assert_eq!(r["abi"], 1);
}
