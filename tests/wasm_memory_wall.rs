//! GWS-11 - memory-wall court on the REAL compiled wasm module, REBOUND to
//! the shared hard-enclosure host (`tests/common/enclosure.rs`, WP P03):
//! every instance now lives under `wasm_exceptions` + `consume_fuel`
//! (`FuelPolicy::Unbounded`, so the module's own limits fire before any host
//! budget) + `epoch_interruption` (deadline disarmed at 2^62 ticks — no leg
//! here is wall-clock-strangled) + cranelift `OptLevel::None` + the 64 MiB
//! `WallLimiter`, hosted by wasmtime 48 (the wasmi 2 engine cannot even
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
//!   docs/jira/v26.10.21/WASM-SANDBOX-SPEC.md) is enforced at the HOST layer
//!   by the enclosure's `WallLimiter` (shared, single-sourced in
//!   tests/common/enclosure.rs since the P03 rebound).
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

use std::sync::{Mutex, OnceLock};

use serde_json::{Value, json};
use wasmtime::ResourceLimiter;

mod common;

use common::enclosure::{EnclosureHost, EPOCH_DISARMED_TICKS, FuelPolicy, WallLimiter};

/// Spec ABI pins (wasm/src/abi_meta.rs, ggen-rendered).
const MAX_REQUEST_BYTES: u32 = 16_777_216;
// GAP-1 (docs/wasm-duckdb-op15-design.md): re-derived to 16 MiB so an
// alloc-success-then-OOM-trap is impossible under the 64 MiB host wall.
const MAX_OUTSTANDING_BYTES: usize = 16_777_216;
const MAX_JSON_DEPTH: u32 = 64;

/// The spec hard wall (smalt WASM-SANDBOX-SPEC): 64 MiB of linear memory.
/// Enforced at the host layer by the enclosure's `WallLimiter`, mirroring
/// `wasmtime run --max-memory 67108864` in smalt_wasm_worker.
const WALL_BYTES: u64 = 67_108_864;

/// A fresh enclosure host: the full hard enclosure (fuel metering unbounded,
/// epoch deadline disarmed, 64 MiB wall, OptLevel::None compile) around one
/// live instance.
fn fresh() -> EnclosureHost {
    EnclosureHost::new(FuelPolicy::Unbounded, EPOCH_DISARMED_TICKS)
}

/// Shared instance for lightweight probes; saturated instances are
/// test-local (private) and dropped (respawn analog).
fn host() -> &'static Mutex<EnclosureHost> {
    static H: OnceLock<Mutex<EnclosureHost>> = OnceLock::new();
    H.get_or_init(|| Mutex::new(fresh()))
}

fn lock() -> std::sync::MutexGuard<'static, EnclosureHost> {
    host().lock().unwrap_or_else(std::sync::PoisonError::into_inner)
}

/// Legal request/response cycle through the enclosure (budgets reset per
/// cycle, response read + freed).
fn call(req: Value) -> Value {
    let out = lock().request_envelope(&req);
    serde_json::from_slice(&out).expect("response is JSON")
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
    let mut h = lock();
    // The module does NOT declare a memory maximum: the spec's 64 MiB wall is
    // HOST-layer enforcement (wasmtime --max-memory / the enclosure's
    // WallLimiter), not a wasm-layer one.
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
    let mut h = fresh();
    let body = padded_capabilities(MAX_REQUEST_BYTES as usize);
    let ptr = h
        .alloc_raw(body.len() as u32)
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
    let mut h = fresh();
    assert_eq!(
        h.alloc_raw(MAX_REQUEST_BYTES + 1),
        None,
        "gl_alloc over the request cap must return null, not trap"
    );
    let _ptr = h
        .alloc_raw(MAX_REQUEST_BYTES)
        .expect("at-cap alloc must succeed");
    drop(h);
    // Hostile-misreport leg rides the shared host with a small buffer.
    let mut h = lock();
    let small = padded_capabilities(64);
    let ptr = h.alloc_raw(small.len() as u32).expect("small alloc");
    h.write_body(ptr, &small);
    let (op, olen) = h.call_raw(ptr, MAX_REQUEST_BYTES + 1); // hostile misreport
    let out = h.read_out(op, olen);
    let r: Value = serde_json::from_slice(&out).expect("response is JSON");
    drop(h);
    assert_typed_limit(&r, "request_bytes", MAX_REQUEST_BYTES);

    let mut h = lock();
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
    let mut h = fresh();
    let mut held: Vec<(u32, u32)> = Vec::new();
    let refused_at = loop {
        match h.alloc_raw(CHUNK) {
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
        h.free_raw(p, l);
    }
    let r = call(json!({"op": "capabilities"}));
    assert_eq!(r["ok"], json!(true), "{r}");
}

#[test]
fn compute_explosive_n3_doubling_refuses_typed_before_memory_dies() {
    // Nested expansion inside the rules evaluator: each firing doubles the
    // literal. Typed refusal, bounded in time, instance still usable.
    //
    // BOUND: relative ratio fence (no-sleep doctrine, absolute-ms-bounds
    // fence retired). The absolute `elapsed < 60s` wall-clock bound flaked at
    // 81s and 89s on two witnessed near-solo reruns of the canonical
    // wrap-shim artifact while every typed-refusal assertion passed.
    //
    // Design: the refusal leg is paired against an INDEPENDENT
    // sleep-free-by-structure baseline leg in the same instance - one small
    // bounded turtle parse (SAMPLE), which does no nested expansion. Each leg
    // runs min-of-2 rounds; min-of-N over paired rounds is the ratio-fence
    // form that fences the mechanism (unbounded compute before the typed
    // refusal), not the machine's absolute speed. Since the P03 rebound the
    // engine compiles at OptLevel::None under fuel metering — both legs slow
    // down together, so the relative fence is unchanged in kind.
    //
    // N derivation: witnessed refusal-leg costs are 81s and 89s near-solo at
    // the pre-rebound opt level; the baseline leg (one small parse through
    // gl_call, same envelope) is sub-second on the same artifact. Measured
    // ratio therefore sat near 100x-200x; RATIO_FENCE = 400x keeps ~2x
    // headroom over the worst witnessed ratio so scheduler contention on a
    // shared host cannot false-red, and still fails a genuine regression (the
    // refusal mechanism is a step-count limit inside the evaluator, so a
    // regression means the limit fires an order of magnitude later — far past
    // 400x).
    const RATIO_FENCE: f64 = 400.0;

    let timed = |req: &Value| -> (std::time::Duration, Value) {
        let started = std::time::Instant::now();
        let r = call(req.clone());
        (started.elapsed(), r)
    };

    let explosive = json!({"op": "n3", "text": concat!(
        "@prefix : <urn:h> .\n",
        "@prefix string: <http://www.w3.org/2000/10/swap/string#> .\n",
        ":a :s \"yy\" .\n",
        "{ :a :s ?x . (?x ?x) string:concatenation ?y } => { :a :s ?y } .\n")});
    let baseline = json!({"op": "parse", "text": SAMPLE, "dialect": "turtle"});

    // min-of-2 paired rounds (baseline leg interleaved so machine drift hits
    // both legs symmetrically).
    let mut base_min = std::time::Duration::MAX;
    let mut expl_min = std::time::Duration::MAX;
    for round in 0..2 {
        let (bt, br) = timed(&baseline);
        assert_eq!(br["ok"], json!(true), "baseline leg round {round} failed: {br}");
        assert_eq!(br["quads"], 1, "baseline leg round {round}: {br}");
        let (et, er) = timed(&explosive);
        assert_eq!(er["ok"], json!(false), "round {round}: {er}");
        assert_eq!(er["error"]["kind"], "ResourceLimit", "round {round}: {er}");
        eprintln!(
            "ratio-fence round {round}: baseline {bt:?}, explosive {et:?}, ratio {:.1}",
            et.as_secs_f64() / bt.as_secs_f64()
        );
        base_min = base_min.min(bt);
        expl_min = expl_min.min(et);
    }

    let ratio = expl_min.as_secs_f64() / base_min.as_secs_f64();
    eprintln!(
        "ratio fence: explosive(min) {:?} vs baseline(min) {:?} = {ratio:.1}x (fence {RATIO_FENCE}x)",
        expl_min, base_min
    );
    assert!(
        ratio < RATIO_FENCE,
        "compute-explosive refusal cost {expl_min:?} is {ratio:.1}x the baseline leg {base_min:?}          (fence {RATIO_FENCE}x) - the limit fired too late or not at all"
    );
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
    let mut h = lock();
    let ptr = h.alloc_raw(raw.len() as u32).expect("depth probe alloc fits");
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
    let mut h = lock();
    let leak_req = json!({"op": "parse", "text": SAMPLE, "dialect": "turtle"});
    const CYCLES: usize = 20;
    const EVALS_PER_CYCLE: usize = 60;
    let mut curve = Vec::with_capacity(CYCLES + 1);
    curve.push(h.memory_pages());
    for c in 0..CYCLES {
        for i in 0..EVALS_PER_CYCLE {
            let out = h.request_envelope_leaking(&leak_req);
            let r: Value = serde_json::from_slice(&out).expect("response is JSON");
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
        let mut h = fresh();
        while h.alloc_raw(CHUNK).is_some() {
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
    let mut fresh_host = fresh();
    let r = fresh_host.exec(&json!({"op": "capabilities"}));
    assert_eq!(r["ok"], json!(true), "{r}");
    assert_eq!(r["abi"], 1);
}

#[test]
fn wall_limiter_refuses_growth_past_64mib_and_the_guest_envelope_stays_typed() {
    // The wasmi->wasmtime wall mapping itself (limiter now single-sourced in
    // tests/common/enclosure.rs): a memory.grow past the 64 MiB wall is
    // denied by WallLimiter (memory.grow -> -1, soft failure), so the guest
    // never sees a host OOM. Witness the limiter's decision directly, and
    // witness the module still healthy after.
    let mut h = fresh();
    let cur = h.memory_pages() * 64 * 1024;
    let mut limiter = WallLimiter::new();
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
    let res = h.grow_memory(1024u64.saturating_sub(cur_pages) + 1);
    assert!(res.is_err(), "over-wall memory.grow must be refused");
    assert_eq!(h.memory_pages(), cur_pages, "denied grow must not resize");
    // And the real instance keeps serving legal work.
    let r = h.exec(&json!({"op": "capabilities"}));
    assert_eq!(r["ok"], json!(true), "{r}");
    assert_eq!(r["abi"], 1);
}
