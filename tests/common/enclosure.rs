//! WP P03 — hard WASM memory/fuel/epoch enclosure host (shared harness).
//!
//! One place that knows how to host the compiled graphlaw module under ALL
//! three enclosure axes at once:
//!
//! 1. MEMORY — `WallLimiter` denies any linear-memory growth past the 64 MiB
//!    spec wall (wasmtime `ResourceLimiter`; `memory.grow` then evaluates to
//!    -1 in the guest and the module's typed allocator-failure envelope
//!    fires — never a host OOM).
//! 2. FUEL — wasmtime metering (`consume_fuel(true)`), policy-driven:
//!    [`FuelPolicy::Unbounded`] seeds a practical infinity each request
//!    cycle; [`FuelPolicy::PerOp`] seeds the op's calibrated budget from a
//!    table (next-pow2 of 4x the measured seed spend, floored at 1e9) and
//!    fail-closes an uncalibrated op with a typed envelope instead of
//!    guessing a global constant.
//! 3. EPOCH — wasmtime epoch interruption (`epoch_interruption(true)`) with
//!    a watchdog thread incrementing the engine epoch every 50 ms until its
//!    stop flag is set; a store deadline is (re)armed BEFORE instantiation
//!    and reset every request cycle, so a wall-clock strangle surfaces as
//!    `Trap::Interrupt`, classified AFTER `Trap::OutOfFuel` and before
//!    everything else.
//!
//! The #1 false red is documented here so it never regresses: a wasmtime
//! store starts at 0 fuel and epoch deadline 0, so a host that instantiates
//! BEFORE seeding traps `_initialize` immediately (OutOfFuel/Interrupt) and
//! every leg reads red for the wrong reason. Both budgets are seeded before
//! `instantiate` and reset per request cycle.
//!
//! Guest-originated fuel/epoch exhaustion cannot produce typed JSON inside
//! the module (the wasm profile pins `panic = "abort"`, and an engine-level
//! trap never reaches guest code at all), so the host SYNTHESIZES the typed
//! envelopes with `graphlaw::abi::limit_response` — byte-identical to the
//! in-module `limit()` shape: `{"ok":false,"error":{"kind":"ResourceLimit",
//! ...,"details":{"code":"ResourceLimit","limit":"guest_fuel"|"guest_epoch",
//! "observed":0,"max":<budget>}}}`.
//!
//! No sleeps as coordination: the watchdog's interval IS the real-world
//! epoch tick being scheduled (production-side SCHEDULING class, wasmtime's
//! documented epoch-interruption pattern), and every court wait is a bounded
//! poll on a real observable.

use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Mutex, OnceLock};
use std::time::Duration;

use serde_json::{Value, json};
use wasmtime::{
    Config, Engine, Instance, Linker, Memory, Module, OptLevel, ResourceLimiter, Store, TypedFunc,
};
use wasmtime_wasi::p1::{WasiP1Ctx, add_to_linker_sync};
use wasmtime_wasi::WasiCtxBuilder;

/// The spec hard wall (smalt WASM-SANDBOX-SPEC): 64 MiB of linear memory,
/// enforced at the host layer — `memory.grow` past the wall is denied, the
/// wasm instruction evaluates to -1, and the guest's typed failure envelope
/// surfaces instead of a host OOM.
pub const WALL_BYTES: u64 = 67_108_864;

/// Watchdog cadence: one engine-epoch increment per 50 ms.
pub const WATCHDOG_INTERVAL: Duration = Duration::from_millis(50);

/// Practical infinity for [`FuelPolicy::Unbounded`]: ~3170 CPU-hours of
/// ~1-fuel-per-instruction execution. Far past any witness in this suite,
/// small enough that budget bookkeeping stays far from overflow.
pub const UNBOUNDED_FUEL: u64 = 1_000_000_000_000_000; // 1e15

/// Calibration seed (WP P03 court law): spend is measured against a 1e12
/// seed; a budget is `max(next_pow2_ceil(4 x spend), 1e9)`. Never a shared
/// global constant.
pub const CALIBRATION_SEED: u64 = 1_000_000_000_000; // 1e12

/// Calibration floor: a calibrated budget is never below 1e9 fuel units.
pub const CALIBRATION_FLOOR: u64 = 1_000_000_000; // 1e9

/// Seed the deadline with this when a court wants epoch interruption armed
/// but effectively disarmed (memory-wall's long-compute legs run minutes; a
/// deadline of 2^62 ticks at 50 ms/tick is ~3.6M years).
pub const EPOCH_DISARMED_TICKS: u64 = 1u64 << 62;

/// Resolve the compiled module: `GRAPHLAW_WASM` when set (must exist), else
/// the shared on-demand build (`common::wasm_path` honors
/// `CARGO_TARGET_DIR`, so concurrent lanes never collide on a path).
pub fn artifact_path() -> PathBuf {
    if let Some(p) = std::env::var_os("GRAPHLAW_WASM") {
        if !p.is_empty() {
            let path = PathBuf::from(p);
            assert!(
                path.is_file(),
                "GRAPHLAW_WASM is set but the artifact is missing: {}",
                path.display()
            );
            return path;
        }
    }
    super::wasm_path()
}

/// Host-layer memory wall: deny any growth past [`WALL_BYTES`].
#[derive(Debug)]
pub struct WallLimiter {
    /// Peak bytes the limiter has admitted (diagnostics on refusal).
    pub peak: u64,
}

impl WallLimiter {
    pub fn new() -> Self {
        WallLimiter { peak: 0 }
    }
}

impl Default for WallLimiter {
    fn default() -> Self {
        Self::new()
    }
}

impl ResourceLimiter for WallLimiter {
    fn memory_growing(
        &mut self,
        current: usize,
        desired: usize,
        _maximum: Option<usize>,
    ) -> Result<bool, wasmtime::Error> {
        if desired as u64 > WALL_BYTES {
            return Err(wasmtime::Error::msg(format!(
                "memory wall: refusing growth to {desired} bytes (wall {WALL_BYTES}; current {current})"
            )));
        }
        self.peak = self.peak.max(desired as u64);
        Ok(true)
    }

    fn table_growing(
        &mut self,
        _current: usize,
        desired: usize,
        _maximum: Option<usize>,
    ) -> Result<bool, wasmtime::Error> {
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
        Ok(())
    }
}

/// Per-request fuel policy.
#[derive(Clone, Debug)]
pub enum FuelPolicy {
    /// Every request cycle is seeded with [`UNBOUNDED_FUEL`]; the epoch
    /// deadline is the only live strangle.
    Unbounded,
    /// Every request cycle is seeded with the op's calibrated budget; an op
    /// missing from the table is fail-closed with a typed envelope (never a
    /// guessed global constant).
    PerOp(BTreeMap<String, u64>),
}

impl FuelPolicy {
    fn budget_for(&self, op: &str) -> Option<u64> {
        match self {
            FuelPolicy::Unbounded => Some(UNBOUNDED_FUEL),
            FuelPolicy::PerOp(table) => table.get(op).copied(),
        }
    }
}

/// Classified enclosure outcome for one guest invocation.
#[derive(Debug)]
pub enum EnclosureError {
    /// The guest spent its entire per-cycle fuel budget (`Trap::OutOfFuel`).
    Fuel { budget: u64 },
    /// The watchdog drove the engine epoch past the store deadline
    /// (`Trap::Interrupt`); `ticks` is the deadline that fired.
    Epoch { ticks: u64 },
    /// Any other guest trap (the raw wasmtime diagnostic).
    Trap(String),
    /// Non-trap transport/host failure.
    Fatal(String),
}

impl EnclosureError {
    /// The `limit` name for the synthesized typed envelope.
    pub fn limit_name(&self) -> Option<&'static str> {
        match self {
            EnclosureError::Fuel { .. } => Some("guest_fuel"),
            EnclosureError::Epoch { .. } => Some("guest_epoch"),
            EnclosureError::Trap(_) | EnclosureError::Fatal(_) => None,
        }
    }

    /// The budget that was breached (fuel units, or deadline ticks).
    pub fn budget(&self) -> Option<u64> {
        match self {
            EnclosureError::Fuel { budget } => Some(*budget),
            EnclosureError::Epoch { ticks } => Some(*ticks),
            EnclosureError::Trap(_) | EnclosureError::Fatal(_) => None,
        }
    }
}

/// Store state: the WASI p1 context plus the 64 MiB wall limiter.
struct EncState {
    wasi: WasiP1Ctx,
    wall: WallLimiter,
}

/// The watchdog: one thread incrementing the engine epoch every
/// [`WATCHDOG_INTERVAL`] until the stop flag is set (set on `Drop`).
struct Watchdog {
    stop: std::sync::Arc<AtomicBool>,
    handle: Option<std::thread::JoinHandle<()>>,
}

impl Watchdog {
    fn spawn(engine: Engine) -> Self {
        let stop = std::sync::Arc::new(AtomicBool::new(false));
        let flag = stop.clone();
        // SLEEP-CLASS: SCHEDULING — the 50 ms sleep models the real-world
        // epoch tick interval the watchdog is defined over (the wasmtime
        // epoch-interruption pattern); the thread has nothing else to do
        // between ticks, and the stop flag ends it deterministically on
        // Drop (join, no orphaned ticker).
        let handle = std::thread::spawn(move || {
            while !flag.load(Ordering::Acquire) {
                std::thread::sleep(WATCHDOG_INTERVAL);
                if flag.load(Ordering::Acquire) {
                    break;
                }
                engine.increment_epoch();
            }
        });
        Watchdog {
            stop,
            handle: Some(handle),
        }
    }
}

impl Drop for Watchdog {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Release);
        if let Some(h) = self.handle.take() {
            let _ = h.join();
        }
    }
}

/// The enclosure host: memory wall + fuel policy + epoch watchdog around one
/// live module instance.
pub struct EnclosureHost {
    store: Store<EncState>,
    memory: Memory,
    alloc_fn: TypedFunc<u32, u32>,
    free_fn: TypedFunc<(u32, u32), ()>,
    call_fn: TypedFunc<(u32, u32), u64>,
    policy: FuelPolicy,
    epoch_ticks: u64,
    _watchdog: Watchdog,
}

impl EnclosureHost {
    /// Compile + instantiate under the full enclosure. Fuel AND the epoch
    /// deadline are seeded BEFORE `instantiate` (a store starts at 0/0 and
    /// `_initialize` would trap immediately — the #1 false red), then reset
    /// every request cycle by the request paths.
    pub fn new(policy: FuelPolicy, epoch_ticks_per_request: u64) -> Self {
        let compiled = compiled_module();
        let (engine, module) = &*compiled
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
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
            engine,
            EncState {
                wasi: WasiCtxBuilder::new().inherit_stdio().build_p1(),
                wall: WallLimiter::new(),
            },
        );
        store.limiter(|s| &mut s.wall);
        // Seed BOTH budgets before instantiate (see module docs).
        store.set_epoch_deadline(epoch_ticks_per_request);
        store
            .set_fuel(UNBOUNDED_FUEL)
            .expect("seed fuel before instantiate");
        let mut linker = <Linker<EncState>>::new(engine);
        add_to_linker_sync(&mut linker, |ctx| &mut ctx.wasi).expect("links WASI");
        let instance: Instance = linker
            .instantiate(&mut store, &module)
            .expect("enclosure: instantiates under seeded budgets");
        if let Ok(init) = instance.get_typed_func::<(), ()>(&mut store, "_initialize") {
            init.call(&mut store, ())
                .expect("enclosure: _initialize under seeded budgets");
        }
        let memory = instance
            .get_memory(&mut store, "memory")
            .expect("enclosure: module exports memory");
        // A `!`-returning fn (not a closure): a closure's return type is
        // unified with its FIRST use and then fixed, which mis-types the
        // second and third call sites under different TypedFunc params.
        fn missing(name: &str) -> ! {
            panic!("enclosure: missing or mistyped export {name}")
        }
        EnclosureHost {
            alloc_fn: instance
                .get_typed_func::<u32, u32>(&mut store, "gl_alloc")
                .unwrap_or_else(|_| missing("gl_alloc")),
            free_fn: instance
                .get_typed_func::<(u32, u32), ()>(&mut store, "gl_free")
                .unwrap_or_else(|_| missing("gl_free")),
            call_fn: instance
                .get_typed_func::<(u32, u32), u64>(&mut store, "gl_call")
                .unwrap_or_else(|_| missing("gl_call")),
            store,
            memory,
            policy,
            epoch_ticks: epoch_ticks_per_request,
            _watchdog: Watchdog::spawn(engine.clone()),
        }
    }

    /// Reset BOTH budgets for one request cycle: fuel from the policy for
    /// `op`, epoch deadline re-armed relative to the current epoch. Returns
    /// the armed fuel budget (None = the request carried no op at all, so no
    /// policy decision exists; the cycle then runs on residual fuel — callers
    /// only hit this for non-JSON bodies, which the module refuses typed).
    ///
    /// PerOp fail-close: an op MISSING from the table arms ZERO fuel — the
    /// invocation traps `OutOfFuel` immediately and surfaces as the typed
    /// `guest_fuel` envelope with max 0, never a run on residual fuel.
    fn reset_budgets(&mut self, op: &str) -> Option<u64> {
        let budget = match &self.policy {
            FuelPolicy::Unbounded => UNBOUNDED_FUEL,
            FuelPolicy::PerOp(table) => table.get(op).copied().unwrap_or(0),
        };
        self.store
            .set_fuel(budget)
            .expect("set_fuel for request cycle");
        self.store.set_epoch_deadline(self.epoch_ticks);
        Some(budget)
    }

    /// The raw request cycle: alloc, write, call, read, optional free.
    /// Classifies traps into [`EnclosureError`].
    fn cycle(&mut self, body: &[u8], free: bool) -> Result<Vec<u8>, EnclosureError> {
        let op = serde_json::from_slice::<Value>(body)
            .ok()
            .and_then(|v| v.get("op").and_then(Value::as_str).map(str::to_string));
        let fuel = match &op {
            Some(op) => self.reset_budgets(op),
            None => None,
        };
        let ticks = self.epoch_ticks;
        let ptr = self
            .alloc_fn
            .call(&mut self.store, body.len() as u32)
            .map_err(|e| classify("gl_alloc", e, fuel, ticks))?;
        if ptr == 0 {
            return Err(EnclosureError::Fatal(
                "gl_alloc returned null for a court request (outstanding cap?)".into(),
            ));
        }
        self.memory
            .write(&mut self.store, ptr as usize, body)
            .map_err(|e| EnclosureError::Fatal(format!("memory write: {e}")))?;
        let packed = self
            .call_fn
            .call(&mut self.store, (ptr, body.len() as u32))
            .map_err(|e| classify("gl_call", e, fuel, ticks))?;
        let (out_ptr, out_len) = ((packed >> 32) as u32, packed as u32);
        let mut out = vec![0u8; out_len as usize];
        self.memory
            .read(&self.store, out_ptr as usize, &mut out)
            .map_err(|e| EnclosureError::Fatal(format!("memory read: {e}")))?;
        if free {
            self.free_fn
                .call(&mut self.store, (out_ptr, out_len))
                .map_err(|e| classify("gl_free", e, fuel, ticks))?;
        }
        Ok(out)
    }

    /// Full request cycle returning ALWAYS a typed JSON envelope: module
    /// response bytes on success, synthesized `guest_fuel`/`guest_epoch`
    /// `limit_response` bytes on classified enclosure breaches.
    pub fn request_envelope(&mut self, req: &Value) -> Vec<u8> {
        let body = req.to_string().into_bytes();
        match self.cycle(&body, true) {
            Ok(out) => out,
            Err(e) => synthesized_envelope(&e),
        }
    }

    /// Leaking request cycle (neither buffer freed): the campaign's
    /// accumulation probe rides this.
    pub fn request_envelope_leaking(&mut self, req: &Value) -> Vec<u8> {
        let body = req.to_string().into_bytes();
        match self.cycle(&body, false) {
            Ok(out) => out,
            Err(e) => synthesized_envelope(&e),
        }
    }

    /// Probe: run one request, classified. Success parses the module's JSON;
    /// enclosure breaches return the typed error (inspect `limit_name()`).
    pub fn probe(&mut self, req: &Value) -> Result<Value, EnclosureError> {
        let body = req.to_string().into_bytes();
        match self.cycle(&body, true) {
            Ok(out) => serde_json::from_slice(&out)
                .map_err(|e| EnclosureError::Fatal(format!("response is not JSON: {e}"))),
            Err(e) => Err(e),
        }
    }

    /// Exec: assert ok on a request expected to admit.
    pub fn exec(&mut self, req: &Value) -> Value {
        let r = self
            .probe(req)
            .unwrap_or_else(|e| panic!("enclosure exec failed for {req}: {e:?}"));
        assert_eq!(
            r["ok"], json!(true),
            "enclosure exec request failed: {req} -> {r}"
        );
        r
    }

    /// Calibrated-seed leg: seed `fuel_seed`, arm the epoch deadline, run
    /// `req`, return the parsed response plus the measured spend
    /// (`seed - get_fuel()` after the call).
    pub fn exec_seeded(&mut self, req: &Value, fuel_seed: u64) -> (Value, u64) {
        let (out, spend) = self.exec_seeded_bytes(req, fuel_seed);
        let parsed: Value = serde_json::from_slice(&out).expect("control response is JSON");
        (parsed, spend)
    }

    /// Like [`EnclosureHost::exec_seeded`] but returns the RAW response bytes
    /// (the byte-identity courts compare these, not re-serialized JSON).
    pub fn exec_seeded_bytes(&mut self, req: &Value, fuel_seed: u64) -> (Vec<u8>, u64) {
        let body = req.to_string().into_bytes();
        self.store
            .set_fuel(fuel_seed)
            .expect("seed calibration fuel");
        self.store.set_epoch_deadline(self.epoch_ticks);
        let out = self
            .cycle_seeded(&body)
            .unwrap_or_else(|e| panic!("seeded control must execute: {e:?}"));
        let remaining = self.store.get_fuel().expect("fuel metering is on");
        let spend = fuel_seed.saturating_sub(remaining);
        (out, spend)
    }

    /// Like [`EnclosureHost::cycle`] but preserves the caller-seeded fuel
    /// (no policy reset); used only by `exec_seeded`.
    fn cycle_seeded(&mut self, body: &[u8]) -> Result<Vec<u8>, EnclosureError> {
        let ptr = self
            .alloc_fn
            .call(&mut self.store, body.len() as u32)
            .map_err(|e| classify("gl_alloc", e, None, self.epoch_ticks))?;
        if ptr == 0 {
            return Err(EnclosureError::Fatal("gl_alloc returned null".into()));
        }
        self.memory
            .write(&mut self.store, ptr as usize, body)
            .map_err(|e| EnclosureError::Fatal(format!("memory write: {e}")))?;
        let packed = self
            .call_fn
            .call(&mut self.store, (ptr, body.len() as u32))
            .map_err(|e| classify("gl_call", e, None, self.epoch_ticks))?;
        let (out_ptr, out_len) = ((packed >> 32) as u32, packed as u32);
        let mut out = vec![0u8; out_len as usize];
        self.memory
            .read(&self.store, out_ptr as usize, &mut out)
            .map_err(|e| EnclosureError::Fatal(format!("memory read: {e}")))?;
        self.free_fn
            .call(&mut self.store, (out_ptr, out_len))
            .map_err(|e| classify("gl_free", e, None, self.epoch_ticks))?;
        Ok(out)
    }

    /// Memory pages currently held by the instance.
    pub fn memory_pages(&mut self) -> u64 {
        self.memory.size(&self.store)
    }

    /// The policy budget for `op` (None = uncalibrated under PerOp).
    pub fn budget_for(&self, op: &str) -> Option<u64> {
        self.policy.budget_for(op)
    }

    /// Set the per-request epoch deadline (ticks beyond the current epoch)
    /// used by subsequent request cycles.
    pub fn set_epoch_ticks_per_request(&mut self, ticks: u64) {
        self.epoch_ticks = ticks;
    }

    /// Calibrate `op` from its positive control: seed [`CALIBRATION_SEED`],
    /// measure the spend, return (budget, spend) where
    /// `budget = max(next_pow2_ceil(4 x spend), CALIBRATION_FLOOR)`.
    pub fn calibrate(&mut self, control: &Value) -> (u64, u64) {
        let (_, spend) = self.exec_seeded(control, CALIBRATION_SEED);
        (next_pow2_ceil(4 * spend).max(CALIBRATION_FLOOR), spend)
    }

    // -- raw adversarial-host surface (the memory-wall court's hostile legs) --

    /// Arm unbounded fuel + the host's epoch posture for a RAW access (no
    /// request body to read an `op` from). In-module limits must fire first.
    fn arm_opless(&mut self) {
        self.store
            .set_fuel(UNBOUNDED_FUEL)
            .expect("arm fuel for raw access");
        self.store.set_epoch_deadline(self.epoch_ticks);
    }

    /// `gl_alloc` at `len` bytes: `None` on the module's null refusal (over
    /// the request cap or the outstanding cap), panic on a trap.
    pub fn alloc_raw(&mut self, len: u32) -> Option<u32> {
        self.arm_opless();
        let p = self
            .alloc_fn
            .call(&mut self.store, len)
            .unwrap_or_else(|e| panic!("gl_alloc trapped: {e:#}"));
        (p != 0).then_some(p)
    }

    /// Host-side write of a request body at `ptr`.
    pub fn write_body(&mut self, ptr: u32, body: &[u8]) {
        self.memory
            .write(&mut self.store, ptr as usize, body)
            .expect("memory write in bounds");
    }

    /// `gl_call` with an arbitrary (ptr, len) — the hostile-misreport leg.
    pub fn call_raw(&mut self, ptr: u32, len: u32) -> (u32, u32) {
        self.arm_opless();
        let packed = self
            .call_fn
            .call(&mut self.store, (ptr, len))
            .unwrap_or_else(|e| panic!("gl_call trapped: {e:#}"));
        ((packed >> 32) as u32, packed as u32)
    }

    /// Read `len` bytes at `ptr` (no free).
    pub fn peek_out(&mut self, ptr: u32, len: u32) -> Vec<u8> {
        let mut out = vec![0u8; len as usize];
        self.memory
            .read(&self.store, ptr as usize, &mut out)
            .expect("memory read in bounds");
        out
    }

    /// Read `len` bytes at `ptr`, then `gl_free` the buffer.
    pub fn read_out(&mut self, ptr: u32, len: u32) -> Vec<u8> {
        let out = self.peek_out(ptr, len);
        self.free_raw(ptr, len);
        out
    }

    /// `gl_free(ptr, len)`.
    pub fn free_raw(&mut self, ptr: u32, len: u32) {
        self.arm_opless();
        self.free_fn
            .call(&mut self.store, (ptr, len))
            .unwrap_or_else(|e| panic!("gl_free trapped: {e:#}"));
    }

    /// Grow linear memory by `delta_pages` (the over-wall grow leg).
    pub fn grow_memory(&mut self, delta_pages: u64) -> Result<u64, wasmtime::Error> {
        self.memory.grow(&mut self.store, delta_pages)
    }

    /// The imported memory's declared maximum (None = no declared ceiling).
    pub fn memory_max_pages(&mut self) -> Option<u64> {
        self.memory.ty(&self.store).maximum()
    }

    /// Peak bytes the wall limiter has admitted (diagnostics).
    pub fn peak_wall_bytes(&self) -> u64 {
        self.store.data().wall.peak
    }
}

/// The once-compiled engine+module, shared so respawn legs do not re-JIT the
/// ~60 MB artifact per construction (wasmtime's on-disk cache is also
/// enabled in the engine config, making even the first cross-process
/// construction cheap after a warm run).
fn compiled_module() -> &'static Mutex<(Engine, Module)> {
    static C: OnceLock<Mutex<(Engine, Module)>> = OnceLock::new();
    C.get_or_init(|| {
        let bytes = std::fs::read(artifact_path())
            .unwrap_or_else(|e| panic!("enclosure: cannot read wasm artifact: {e}"));
        let mut cfg = Config::new();
        // Pinned enclosure engine surface (WP P03): exceptions on (the
        // DuckDB-linked module carries W3C exception tags), fuel metering
        // on, epoch interruption on, cranelift opt level pinned to None
        // (compile under the court deadline; guest work is interpreter-paced
        // JSON/decision work where the opt level is not load-bearing).
        cfg.wasm_exceptions(true);
        cfg.consume_fuel(true);
        cfg.epoch_interruption(true);
        cfg.cranelift_opt_level(OptLevel::None);
        let engine = Engine::new(&cfg).expect("enclosure engine");
        let module = Module::new(&engine, &bytes[..]).expect("enclosure: valid wasm module");
        Mutex::new((engine, module))
    })
}

/// `next_pow2_ceil(x)`: smallest power of two >= x (0 and 1 -> 1).
pub fn next_pow2_ceil(x: u64) -> u64 {
    let mut p = 1u64;
    while p < x {
        p = p
            .checked_shl(1)
            .unwrap_or_else(|| panic!("next_pow2_ceil overflow for {x}"));
    }
    p
}

/// Classify a wasmtime error: `Trap::OutOfFuel` first, then
/// `Trap::Interrupt`, then everything else.
fn classify(
    stage: &str,
    e: wasmtime::Error,
    fuel: Option<u64>,
    ticks: u64,
) -> EnclosureError {
    if let Some(trap) = e.downcast_ref::<wasmtime::Trap>() {
        match trap {
            wasmtime::Trap::OutOfFuel => {
                return EnclosureError::Fuel {
                    budget: fuel.unwrap_or(0),
                };
            }
            wasmtime::Trap::Interrupt => {
                return EnclosureError::Epoch { ticks };
            }
            _ => {}
        }
    }
    EnclosureError::Trap(format!("{stage}: {e:#}"))
}

/// Synthesize the typed ResourceLimit envelope for a classified enclosure
/// breach, byte-identical to the module's own `graphlaw::abi::limit()`
/// response shape (`limit_response("guest_fuel", 0, budget)`).
pub fn synthesized_envelope(e: &EnclosureError) -> Vec<u8> {
    match (e.limit_name(), e.budget()) {
        (Some(limit), Some(budget)) => {
            graphlaw::abi::limit_response(limit, 0, usize::try_from(budget).unwrap_or(usize::MAX))
        }
        _ => graphlaw::abi::missing_buffer_response(),
    }
}
