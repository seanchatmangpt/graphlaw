//! `graphlaw-run-eh` (GWS-18): wasmtime 48 host for the DuckDB-linked wasm
//! artifact — the sole packet-4 stdio runner (4-byte BE length-prefixed
//! frames, smalt eval-envelope auto-detect, stub-compatible exit codes)
//! since the wasmi retirement (docs/jira/v26.10.23/ADR-wasmi-retirement.md).
//!
//! Contract (formerly stated as differences from the retired wasmi host,
//! `src/bin/graphlaw_run.rs`):
//! - runtime is wasmtime 48 with the W3C exception-handler proposal enabled
//!   (`wasm_exceptions(true)`), matching `-wasm-use-legacy-eh=0` upstream;
//! - WASI is linked via `wasmtime-wasi` (`add_to_linker_sync`, stdio
//!   inherited), and `--features runner-eh` gates compilation via the
//!   `[[bin]]` `required-features` (coordinator wires Cargo.toml);
//! - hardening: the WASI environment is sterile (explicitly empty — no host
//!   variable leaks into the guest), frame prefixes are bounded by
//!   `graphlaw::abi::MAX_REQUEST_BYTES` (exit 92, before any allocation),
//!   and guest execution runs under a per-request fuel budget
//!   (`FUEL_PER_REQUEST`, exhaustion = typed exit 98) with epoch
//!   interruption driven by a host watchdog that increments the engine epoch
//!   every 50 ms: a guest stalling past the armed deadline traps `Interrupt`
//!   and exits typed 99 (`--epoch-ticks` re-arms the deadline).
//!
//! Import policy is unchanged: only `wasi_snapshot_preview1` imports are
//! accepted; anything else is a typed instantiation refusal (exit 96).

use std::io::{Read, Write};
use std::path::PathBuf;
use std::process::ExitCode;

use serde_json::{Value, json};
use wasmtime::{
    Config, Engine, Linker, Module, Store,
    WasmBacktraceDetails,
};
use wasmtime_wasi::p1::{WasiP1Ctx, add_to_linker_sync};
use wasmtime_wasi::WasiCtxBuilder;

const WALL_BYTES: u32 = 67_108_864; // `--max-memory 67108864` parity

/// Coherence guard (design-read-frame-bound.md): a frame the cap admits must
/// be allocatable under the runner memory wall. The two constants serve
/// different layers (untrusted wire prefix vs total guest memory) and are
/// deliberately not equal — this only asserts cap <= wall.
const _: () = assert!(graphlaw::abi::MAX_REQUEST_BYTES <= WALL_BYTES as usize);

/// Guest fuel budget per request (wasmtime fuel units, ~1 per instruction).
/// A guest call that spends it all is a typed refusal (exit 98), never a raw
/// trap or an unbounded hang. Reset before every guest execution.
const FUEL_PER_REQUEST: u64 = 100_000_000_000;

/// Epoch-deadline ticks beyond the current epoch, re-armed every request
/// cycle. A host watchdog (`WATCHDOG_INTERVAL`) increments the engine epoch
/// every 50 ms, so the default `EPOCH_TICKS` (1e9 ≈ 1.6 years) is the
/// effectively-disarmed production posture: the wall-clock strangle path is
/// live — `--epoch-ticks N` arms a real deadline and a guest that stalls past
/// it traps `Interrupt` → typed exit 99.
const EPOCH_TICKS: u64 = 1_000_000_000;

/// Watchdog cadence: one engine-epoch increment per 50 ms
/// (SLEEP-CLASS: SCHEDULING — the interval IS the real-world epoch tick the
/// wasmtime epoch-interruption pattern is defined over; the thread has
/// nothing else to do between ticks).
const WATCHDOG_INTERVAL: std::time::Duration = std::time::Duration::from_millis(50);

/// The epoch watchdog: increments the engine epoch every
/// [`WATCHDOG_INTERVAL`] until the stop flag is set (set on `Drop`, joined —
/// no orphaned ticker outlives the host).
struct Watchdog {
    stop: std::sync::Arc<std::sync::atomic::AtomicBool>,
    handle: Option<std::thread::JoinHandle<()>>,
}

impl Watchdog {
    fn spawn(engine: Engine) -> Self {
        use std::sync::atomic::{AtomicBool, Ordering};
        let stop = std::sync::Arc::new(AtomicBool::new(false));
        let flag = stop.clone();
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
        use std::sync::atomic::Ordering;
        self.stop.store(true, Ordering::Release);
        if let Some(h) = self.handle.take() {
            let _ = h.join();
        }
    }
}

#[derive(Debug)]
enum RunError {
    /// Usage / artifact I/O fault. Exit 2.
    Usage(String),
    /// Artifact digest mismatch refusal. Exit 84.
    DigestMismatch {
        path: PathBuf,
        expected: String,
        got: String,
    },
    /// EOF in the middle of a packet-4 frame. Exit 90.
    FrameEof,
    /// Frame length prefix exceeds the request cap
    /// (`graphlaw::abi::MAX_REQUEST_BYTES`). Fired on the prefix alone,
    /// before any body allocation. Exit 92.
    FrameOversize { len: usize, cap: usize },
    /// Stdin read error. Exit 91.
    StdinRead(std::io::Error),
    /// Malformed eval envelope (undecodable base64 payload). Exit 93.
    MalformedEnvelope,
    /// Module returned a non-JSON ABI response. Exit 94.
    NonJsonResponse(String),
    /// Guest-panic / memory-wall trap. Exit 86.
    Trap,
    /// Bytes are not a valid wasm module. Exit 95.
    WasmInvalid(String),
    /// Module failed to instantiate/link/export the graphlaw ABI. Exit 96.
    WasmInstantiation(String),
    /// Guest ABI call or linear-memory access faulted. Exit 97.
    AbiFault(String),
    /// Guest spent its entire per-request fuel budget (`FUEL_PER_REQUEST`).
    /// Exit 98 — a budget refusal, not a trap.
    FuelExhausted,
    /// The watchdog drove the engine epoch past the store deadline while the
    /// guest ran (`Trap::Interrupt`). Exit 99 — the wall-clock strangle path,
    /// classified AFTER fuel (98).
    EpochExpired { ticks: u64 },
}

impl RunError {
    fn code(&self) -> u8 {
        match self {
            RunError::Usage(_) => 2,
            RunError::DigestMismatch { .. } => 84,
            RunError::Trap => 86,
            RunError::FrameEof => 90,
            RunError::FrameOversize { .. } => 92,
            RunError::StdinRead(_) => 91,
            RunError::MalformedEnvelope => 93,
            RunError::NonJsonResponse(_) => 94,
            RunError::WasmInvalid(_) => 95,
            RunError::WasmInstantiation(_) => 96,
            RunError::AbiFault(_) => 97,
            RunError::FuelExhausted => 98,
            RunError::EpochExpired { .. } => 99,
        }
    }

    fn diagnostic(&self) -> String {
        match self {
            RunError::Usage(m) => format!("graphlaw-run-eh: {m}"),
            RunError::DigestMismatch {
                path,
                expected,
                got,
            } => format!(
                "graphlaw-run-eh: ARTIFACT DIGEST MISMATCH for {}: expected {expected}, got {got}",
                path.display()
            ),
            RunError::Trap => {
                "graphlaw-run-eh: guest trap (crash mode or memory wall)".into()
            }
            RunError::FrameEof => "graphlaw-run-eh: EOF mid-frame".into(),
            RunError::FrameOversize { len, cap } => {
                format!("graphlaw-run-eh: frame length {len} exceeds cap {cap}")
            }
            RunError::StdinRead(e) => format!("graphlaw-run-eh: stdin read error: {e}"),
            RunError::MalformedEnvelope => {
                "graphlaw-run-eh: malformed envelope (bad base64 payload)".into()
            }
            RunError::NonJsonResponse(e) => {
                format!("graphlaw-run-eh: module returned non-JSON response: {e}")
            }
            RunError::WasmInvalid(e) => {
                format!("graphlaw-run-eh: invalid wasm artifact: {e}")
            }
            RunError::WasmInstantiation(e) => {
                format!("graphlaw-run-eh: wasm instantiation failed: {e}")
            }
            // `{:#}` renders the full anyhow chain so the root cause (e.g.
            // "uninitialized element 0") reaches stderr, not just the wrapper.
            RunError::AbiFault(e) => format!("graphlaw-run-eh: guest ABI fault: {e:#}"),
            RunError::FuelExhausted => {
                format!("graphlaw-run-eh: guest exhausted fuel budget {FUEL_PER_REQUEST}")
            }
            RunError::EpochExpired { ticks } => format!(
                "graphlaw-run-eh: guest epoch deadline expired after {ticks} ticks \
                 (wall-clock strangle)"
            ),
        }
    }
}

enum Mode {
    Eval,
    Crash,
    Hang,
}

fn run() -> Result<ExitCode, RunError> {
    let mut wasm: Option<PathBuf> = None;
    let mut expect_blake3: Option<String> = None;
    let mut mode = Mode::Eval;
    let mut epoch_ticks: Option<u64> = None;
    let mut args = std::env::args().skip(1);
    while let Some(a) = args.next() {
        match a.as_str() {
            "--crash" => mode = Mode::Crash,
            "--hang" => mode = Mode::Hang,
            "--wasm" => match args.next() {
                Some(p) => wasm = Some(PathBuf::from(p)),
                None => return Err(RunError::Usage("--wasm needs a path".into())),
            },
            "--expect-blake3" => match args.next() {
                Some(h) => expect_blake3 = Some(h),
                None => {
                    return Err(RunError::Usage("--expect-blake3 needs a hex digest".into()))
                }
            },
            "--epoch-ticks" => match args.next().and_then(|v| v.parse::<u64>().ok()) {
                Some(t) if t > 0 => epoch_ticks = Some(t),
                _ => return Err(RunError::Usage("--epoch-ticks needs a tick count > 0".into())),
            },
            other => return Err(RunError::Usage(format!("unknown argument `{other}`"))),
        }
    }

    // Same resolution ladder as graphlaw_run.rs / scripts/wasm_audit.sh:
    // --wasm, then GRAPHLAW_WASM, then the working-tree build artifact.
    let path = wasm
        .or_else(|| std::env::var_os("GRAPHLAW_WASM").map(PathBuf::from))
        .unwrap_or_else(|| {
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("target/wasm32-wasip1/wasm/graphlaw_wasm.wasm")
        });

    let bytes = std::fs::read(&path).map_err(|e| {
        RunError::Usage(format!("cannot read wasm artifact {}: {e}", path.display()))
    })?;

    if let Some(hex) = &expect_blake3 {
        let got = blake3_hex(&bytes);
        if &got != hex {
            return Err(RunError::DigestMismatch {
                path: path.clone(),
                expected: hex.clone(),
                got,
            });
        }
    }

    let mut host = Host::new(&bytes, epoch_ticks.unwrap_or(EPOCH_TICKS))?;

    let mut stdin = std::io::stdin().lock();
    let mut stdout = std::io::stdout().lock();

    loop {
        let Some(body) = read_frame(&mut stdin)? else {
            return Ok(ExitCode::from(0));
        };
        match mode {
            Mode::Hang => {
                // Simulated guest infinite loop: never reply, never exit.
                // (SLEEP-CLASS: SCHEDULING — the fail-closed timeout fixture.)
                loop {
                    std::thread::sleep(std::time::Duration::from_secs(3600));
                }
            }
            Mode::Crash => {
                return Err(RunError::Trap);
            }
            Mode::Eval => {
                let reply = eval_frame(&mut host, &body)?;
                write_frame(&mut stdout, &reply)?;
                stdout
                    .flush()
                    .map_err(|e| RunError::Usage(format!("stdout flush failed: {e}")))?;
            }
        }
    }
}

fn main() -> ExitCode {
    match run() {
        Ok(code) => code,
        Err(e) => {
            eprintln!("{}", e.diagnostic());
            ExitCode::from(e.code())
        }
    }
}

struct Host {
    store: Store<WasiP1Ctx>,
    memory: wasmtime::Memory,
    alloc: wasmtime::TypedFunc<u32, u32>,
    free: wasmtime::TypedFunc<(u32, u32), ()>,
    call: wasmtime::TypedFunc<(u32, u32), u64>,
    epoch_ticks: u64,
    _watchdog: Watchdog,
}

impl Host {
    fn new(bytes: &[u8], epoch_ticks: u64) -> Result<Self, RunError> {
        let mut config = Config::new();
        config.wasm_exceptions(true);
        config.wasm_backtrace_details(WasmBacktraceDetails::Enable);
        // Deterministic budget: every guest execution runs under fuel; a
        // spent budget is a typed refusal (exit 98), not a trap or a hang.
        config.consume_fuel(true);
        // Epoch interruption armed for the wall-clock strangle path; see
        // EPOCH_TICKS for why the deadline is finite but cannot fire today.
        config.epoch_interruption(true);
        // Compile-time parity with the court deadline: the 46 MB
        // DuckDB-linked artifact is JIT-compiled on every spawn, and a
        // debug-built cranelift at `SpeedAndSize` cannot compile it within
        // ARTIFACT_DEADLINE (observed >570 s). `OptLevel::None` moves the
        // compile under the deadline; guest execution is interpreter-paced
        // DuckDB work where the opt level is not load-bearing.
        config.cranelift_opt_level(wasmtime::OptLevel::None);
        // Persist the compiled module so repeated spawns (the exit-code
        // courts spawn once per test) reuse the first compile instead of
        // re-JITing the artifact every time.
        let cache = wasmtime::Cache::new(wasmtime::CacheConfig::default())
            .map_err(|e| RunError::WasmInvalid(format!("wasmtime cache init: {e}")))?;
        config.cache(Some(cache));
        let engine = Engine::new(&config)
            .map_err(|e| RunError::WasmInvalid(format!("engine config: {e}")))?;
        let module = match Module::new(&engine, bytes) {
            Ok(module) => module,
            Err(e) => return Err(RunError::WasmInvalid(e.to_string())),
        };
        // The only imports allowed are WASI's (clock, random, stdio).
        for i in module.imports() {
            if i.module() != "wasi_snapshot_preview1" {
                return Err(RunError::WasmInstantiation(format!(
                    "unexpected host import {}::{}",
                    i.module(),
                    i.name()
                )));
            }
        }
        let mut store = Store::new(
            &engine,
            // Sterile WASI context: stdio is inherited (the packet-4 protocol
            // is the module's I/O surface) but the environment is explicitly
            // empty — the builder default, no `.inherit_env()` / `.env()` /
            // `.envs()` call at all — so no host variable (keys, tokens,
            // paths) reaches the guest.
            WasiCtxBuilder::new().inherit_stdio().build_p1(),
        );
        // Fuel and the epoch deadline must be seeded BEFORE `instantiate`:
        // a store starts at 0 fuel (any guest execution traps OutOfFuel
        // immediately) and at epoch deadline 0 (traps immediately with
        // epochs enabled). The start section and `_initialize` are guest
        // code and run under the same budget as requests.
        store.set_epoch_deadline(EPOCH_TICKS);
        store
            .set_fuel(FUEL_PER_REQUEST)
            .map_err(|e| RunError::AbiFault(format!("fuel budget init: {e:#}")))?;
        let mut linker = <Linker<WasiP1Ctx>>::new(&engine);
        add_to_linker_sync(&mut linker, |ctx| ctx)
            .map_err(|e| RunError::WasmInstantiation(e.to_string()))?;
        // `Linker::instantiate` runs the start section and returns the
        // ready instance (no separate InstancePre::start in this API).
        let instance = linker
            .instantiate(&mut store, &module)
            .map_err(|e| RunError::WasmInstantiation(e.to_string()))?;
        // Reactor-style modules expose `_initialize`; hosts must call it once.
        if let Ok(init) = instance.get_typed_func::<(), ()>(&mut store, "_initialize") {
            init.call(&mut store, ())
                .map_err(|e| RunError::WasmInstantiation(format!("_initialize: {e}")))?;
        }
        let memory = instance
            .get_memory(&mut store, "memory")
            .ok_or_else(|| RunError::WasmInstantiation("module does not export memory".into()))?;
        let export = |name: &'static str| {
            RunError::WasmInstantiation(format!("missing or mistyped export {name}"))
        };
        Ok(Host {
            alloc: instance
                .get_typed_func::<u32, u32>(&mut store, "gl_alloc")
                .map_err(|_| export("gl_alloc"))?,
            free: instance
                .get_typed_func::<(u32, u32), ()>(&mut store, "gl_free")
                .map_err(|_| export("gl_free"))?,
            call: instance
                .get_typed_func::<(u32, u32), u64>(&mut store, "gl_call")
                .map_err(|_| export("gl_call"))?,
            epoch_ticks,
            _watchdog: Watchdog::spawn(engine.clone()),
            store,
            memory,
        })
    }

    /// `gl_alloc` + memory write + `gl_call` + read + `gl_free`, mirroring the
    /// request_bytes pattern of graphlaw_run.rs and tests/wasm_abi.rs.
    fn request_bytes(&mut self, body: &[u8]) -> Result<Vec<u8>, RunError> {
        // Fresh deterministic budget per request: one guest call cannot
        // spend more than FUEL_PER_REQUEST regardless of driver behavior;
        // exhaustion is classified from Trap::OutOfFuel in `abi_fault`. The
        // epoch deadline is re-armed for the cycle too (reset BOTH budgets
        // per request cycle — the enclosure law).
        self.store
            .set_fuel(FUEL_PER_REQUEST)
            .map_err(|e| RunError::AbiFault(format!("set_fuel: {e:#}")))?;
        self.store.set_epoch_deadline(self.epoch_ticks);
        if body.len() > WALL_BYTES as usize {
            // Memory wall: enforced before any reply can be framed.
            return Err(RunError::Trap);
        }
        let ptr = self
            .alloc
            .call(&mut self.store, body.len() as u32)
            .map_err(|e| abi_fault("gl_alloc", e, self.epoch_ticks))?;
        if ptr == 0 {
            return Err(RunError::AbiFault(
                "gl_alloc refused a request the caller pre-checked".into(),
            ));
        }
        self.memory
            .write(&mut self.store, ptr as usize, body)
            .map_err(|e| RunError::AbiFault(format!("memory write: {e}")))?;
        let packed = self
            .call
            .call(&mut self.store, (ptr, body.len() as u32))
            .map_err(|e| abi_fault("gl_call", e, self.epoch_ticks))?;
        let (out_ptr, out_len) = ((packed >> 32) as u32, (packed & 0xffff_ffff) as u32);
        let mut out = vec![0u8; out_len as usize];
        self.memory
            .read(&self.store, out_ptr as usize, &mut out)
            .map_err(|e| RunError::AbiFault(format!("memory read: {e}")))?;
        self.free
            .call(&mut self.store, (out_ptr, out_len))
            .map_err(|e| abi_fault("gl_free", e, self.epoch_ticks))?;
        Ok(out)
    }
}

/// Wraps a wasmtime trap into `RunError::AbiFault`, printing the guest
/// backtrace (one line per frame) to stderr so trap diagnosis does not need
/// external disassembly. Requires `WasmBacktraceDetails::Enable` for names.
fn abi_fault(op: &str, e: wasmtime::Error, epoch_ticks: u64) -> RunError {
    if let Some(bt) = e.downcast_ref::<wasmtime::WasmBacktrace>() {
        for (i, frame) in bt.frames().iter().enumerate() {
            let name = frame.func_name().unwrap_or("<unknown>");
            eprintln!("graphlaw-run-eh: guest frame #{i} in {op}: {name}");
        }
    }
    // Fuel exhaustion is a budget refusal (exit 98), not a crash; the epoch
    // deadline firing is the wall-clock strangle (exit 99), classified after
    // fuel.
    if let Some(wasmtime::Trap::OutOfFuel) = e.downcast_ref::<wasmtime::Trap>() {
        return RunError::FuelExhausted;
    }
    if let Some(wasmtime::Trap::Interrupt) = e.downcast_ref::<wasmtime::Trap>() {
        return RunError::EpochExpired { ticks: epoch_ticks };
    }
    RunError::AbiFault(format!("{op}: {e:#}"))
}

/// Reads one packet-4 frame: 4-byte BE length prefix + body. Returns None on a
/// clean EOF at a frame boundary.
fn read_frame(stdin: &mut impl Read) -> Result<Option<Vec<u8>>, RunError> {
    let mut len_buf = [0u8; 4];
    match stdin.read_exact(&mut len_buf) {
        Ok(()) => {}
        Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => return Ok(None),
        Err(e) => return Err(RunError::StdinRead(e)),
    }
    let len = u32::from_be_bytes(len_buf) as usize;
    // Bound the untrusted prefix BEFORE any body allocation: a 0xFFFFFFFF
    // prefix would otherwise allocate ~4 GiB of zeros and only fail with
    // FrameEof (90) after stdin is exhausted. The cap is the single-source
    // guest request cap (src/abi.rs), never a local duplicate.
    if len > graphlaw::abi::MAX_REQUEST_BYTES {
        return Err(RunError::FrameOversize {
            len,
            cap: graphlaw::abi::MAX_REQUEST_BYTES,
        });
    }
    let mut body = vec![0u8; len];
    match stdin.read_exact(&mut body) {
        Ok(()) => Ok(Some(body)),
        Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => Err(RunError::FrameEof),
        Err(e) => Err(RunError::StdinRead(e)),
    }
}

fn write_frame(stdout: &mut impl Write, body: &[u8]) -> Result<(), RunError> {
    let mut buf = Vec::with_capacity(4 + body.len());
    buf.extend_from_slice(&(body.len() as u32).to_be_bytes());
    buf.extend_from_slice(body);
    stdout
        .write_all(&buf)
        .map_err(|e| RunError::Usage(format!("frame write failed: {e}")))
}

fn eval_frame(host: &mut Host, body: &[u8]) -> Result<Vec<u8>, RunError> {
    let parsed: Value = match serde_json::from_slice(body) {
        Ok(v) => v,
        // Not the eval envelope: forward the raw bytes to the module as an
        // ABI request and reply under "result" (abi-mode passthrough).
        Err(_) => {
            let response = call_and_decode(host, body)?;
            return serde_json::to_vec(&json!({ "ok": true, "result": response }))
                .map_err(reply_serialize);
        }
    };

    match parsed.get("op").and_then(Value::as_str) {
        Some("eval") => {
            let payload_b64 = parsed
                .get("payload_b64")
                .and_then(Value::as_str)
                .unwrap_or("");
            let payload = base64_decode(payload_b64).ok_or(RunError::MalformedEnvelope)?;
            if payload.len() > WALL_BYTES as usize {
                return Err(RunError::Trap);
            }
            let frames_count = parsed
                .get("predicate_frames")
                .and_then(Value::as_array)
                .map(|a| a.len())
                .unwrap_or(0);
            let response = call_and_decode(host, &payload)?;
            let reply = json!({
                "ok": true,
                "result": response,
                "predicate_frames_count": frames_count,
            });
            serde_json::to_vec(&reply).map_err(reply_serialize)
        }
        // Any other op: the frame IS an ABI request; same envelope, raw
        // response as "result".
        _ => {
            let response = call_and_decode(host, body)?;
            serde_json::to_vec(&json!({ "ok": true, "result": response })).map_err(reply_serialize)
        }
    }
}

fn reply_serialize(e: serde_json::Error) -> RunError {
    RunError::Usage(format!("reply serialization failed: {e}"))
}

fn call_and_decode(host: &mut Host, request_bytes: &[u8]) -> Result<Value, RunError> {
    let abi_response = host.request_bytes(request_bytes)?;
    let response: Value = serde_json::from_slice(&abi_response)
        .map_err(|e| RunError::NonJsonResponse(e.to_string()))?;
    Ok(response)
}

/// Base64 decode without an extra dependency (standard alphabet, padding).
fn base64_decode(s: &str) -> Option<Vec<u8>> {
    const INVALID: u8 = 0xFF;
    let mut table = [INVALID; 256];
    let alphabet = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut i = 0;
    for &c in alphabet {
        table[c as usize] = i;
        i += 1;
    }
    let bytes = s.as_bytes();
    let mut out = Vec::with_capacity(bytes.len() / 4 * 3);
    let mut chunk = [0u8; 4];
    let mut n = 0;
    for &c in bytes {
        match c {
            b'=' => break,
            b'\n' | b'\r' | b' ' | b'\t' => continue,
            _ => {}
        }
        let v = table[c as usize];
        if v == INVALID {
            return None;
        }
        chunk[n] = v;
        n += 1;
        if n == 4 {
            out.push((chunk[0] << 2) | (chunk[1] >> 4));
            out.push((chunk[1] << 4) | (chunk[2] >> 2));
            out.push((chunk[2] << 6) | chunk[3]);
            n = 0;
        }
    }
    match n {
        0 => Some(out),
        2 => {
            out.push((chunk[0] << 2) | (chunk[1] >> 4));
            Some(out)
        }
        3 => {
            out.push((chunk[0] << 2) | (chunk[1] >> 4));
            out.push((chunk[1] << 4) | (chunk[2] >> 2));
            Some(out)
        }
        _ => None,
    }
}

/// BLAKE3 digest of `bytes` as lowercase hex (graphlaw already depends on blake3).
fn blake3_hex(bytes: &[u8]) -> String {
    let mut hasher = blake3::Hasher::new();
    hasher.update(bytes);
    let hash = hasher.finalize();
    let mut hex = String::with_capacity(64);
    for b in hash.as_bytes() {
        hex.push_str(&format!("{b:02x}"));
    }
    hex
}
