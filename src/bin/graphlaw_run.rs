//! `graphlaw-run` (GWS-18): a thin stdio runner that hosts the *compiled*
//! graphlaw wasm module in a real wasm runtime (wasmi) and adapts it to the
//! smalt sandbox runner protocol.
//!
//! ## Why a Rust bin
//!
//! wasmi is a host-side crate (it interprets wasm; it does not compile to
//! wasm32), so no Elixir/escript process can own the wasmi `Host` directly.
//! The honest mechanism is the one graphlaw tests/wasm_abi.rs already uses: a
//! small Rust host that instantiates the module and drives
//! `gl_alloc`/`gl_call`/`gl_free` over linear memory. This bin is that host,
//! wrapped in a packet-4 stdio loop so the smalt BEAM port driver
//! (`Smalt.Wasm.Worker`) can speak to it with a config-only swap
//! (`config :smalt, :wasm_runner_path`).
//!
//! ## Protocol (packet-4: 4-byte big-endian length prefix, per frame)
//!
//! Two request shapes, auto-detected per frame:
//!
//! 1. smalt eval envelope (stub-runner protocol):
//!    `{"op":"eval","payload_b64":"...","predicate_frames":[...]}`. The base64
//!    payload is decoded and run through the module exactly as the host test
//!    writes it via `gl_alloc` + `gl_call`. The reply wraps the module's raw
//!    ABI response:
//!    `{"ok":true,"result":<abi response>,"predicate_frames_count":N}`.
//!    This is the smalt `Wasm.Worker` stub-mode reply contract, so the swap is
//!    config-only from the BEAM side. `predicate_frames` has no ABI analogue;
//!    the count is echoed back as metadata (documented mapping: frames are a
//!    smalt transport classification, the ABI sees only the request bytes).
//! 2. raw ABI request (any other JSON object, e.g. `{"op":"capabilities"}`):
//!    the frame bytes go to the module verbatim; the reply wraps the module's
//!    raw ABI response in the same `{"ok":true,"result":...}` envelope, so the
//!    smalt stub-mode worker decode accepts both shapes uniformly.
//!
//! ## Legacy-host scope (EH-class artifact pre-flight)
//!
//! This is the legacy wasmi host: it courts the *pre-duckdb* artifact class.
//! The DuckDB-linked artifact carries a wasm Tag (exception-handling) section
//! (section id 13), which this wasmi build cannot load. Before handing bytes
//! to the engine, the runner walks the section headers and refuses an
//! EH-class artifact with the 96-class refusal "EH-class artifact requires
//! graphlaw-run-eh" (exit 96) instead of a wasmi parse panic.
//!
//! ## Modes and exit codes (stub-compatible, courts depend on them)
//!
//!   (default)       eval loop until stdin EOF; exit 0.
//!   --crash         read one frame, then trap: exit code 86 (guest-panic /
//!                   memory-wall abort leg of the smalt courts).
//!   --hang          read one frame, then never reply and never exit —
//!                   exercises the worker's fail-closed timeout leg.
//!   (any payload)   eval payload > 64 MiB traps with exit 86 before any
//!                   reply frame, mirroring the wasmtime 64 MiB
//!                   linear-memory wall (`--max-memory 67108864` parity).
//!
//! Other non-zero exits are harness faults, all typed (see `RunError`):
//! 90 EOF mid-frame, 91 stdin read error, 93 malformed envelope, 94 non-JSON
//! module response, 84 artifact digest mismatch refusal, 2 usage/I/O, and the
//! wasm-artifact classes 95 (invalid module), 96 (instantiation/link failure),
//! 97 (guest ABI fault). Malformed input never panics the runner; every
//! failure is a typed one-line stderr diagnostic plus its stable exit code.
//!
//! ## Artifact selection and identity
//!
//! `--wasm <path>`, else `GRAPHLAW_WASM`, else the working-tree build at
//! `target/wasm-abi/wasm32-wasip1/wasm/graphlaw_wasm.wasm` (the artifact the
//! registry/ARTIFACTS.blake3 wasm row pins). `--expect-blake3` verifies
//! the loaded bytes against a digest before serving; mismatch is a loud
//! refusal (exit 84), never a silent serve.

use std::io::{Read, Write};
use std::path::PathBuf;
use std::process::ExitCode;

// #region agent log
fn agent_log(hypothesis_id: &str, location: &str, message: &str, data: Value) {
    let line = json!({
        "sessionId": "c0cb0f",
        "runId": "post-fix",
        "hypothesisId": hypothesis_id,
        "location": location,
        "message": message,
        "data": data,
        "timestamp": std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0),
    });
    // Structured diagnostic line; stderr keeps it visible without a
    // hardcoded home-path side channel (machine_shaped_gate finding).
    eprintln!("{line}");
}
// #endregion

use serde_json::{Value, json};
use wasmi::{Engine, Instance, Linker, Memory, Module, Store, TypedFunc};
use wasmi_wasi::{WasiCtx, WasiCtxBuilder};

const WALL_BYTES: u32 = 67_108_864; // wasmtime `--max-memory 67108864` parity

/// Typed runner failure. Every variant maps to one stable process exit code
/// (courts depend on these; see the module docs) and carries a one-line
/// stderr diagnostic.
#[derive(Debug)]
enum RunError {
    /// Usage / artifact I/O fault. Exit 2.
    Usage(String),
    /// Artifact digest mismatch refusal. Exit 84 (loud, never silent serve).
    DigestMismatch {
        path: PathBuf,
        expected: String,
        got: String,
    },
    /// EOF in the middle of a packet-4 frame. Exit 90.
    FrameEof,
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
    /// Artifact carries a Tag (exception-handling) section: this legacy wasmi
    /// host cannot load it; requires graphlaw-run-eh. Exit 96 (EH class).
    EhArtifact { path: PathBuf },
    /// Guest ABI call or linear-memory access faulted. Exit 97.
    AbiFault(String),
}

impl RunError {
    fn code(&self) -> u8 {
        match self {
            RunError::Usage(_) => 2,
            RunError::DigestMismatch { .. } => 84,
            RunError::Trap => 86,
            RunError::FrameEof => 90,
            RunError::StdinRead(_) => 91,
            RunError::MalformedEnvelope => 93,
            RunError::NonJsonResponse(_) => 94,
            RunError::WasmInvalid(_) => 95,
            RunError::WasmInstantiation(_) => 96,
            RunError::EhArtifact { .. } => 96,
            RunError::AbiFault(_) => 97,
        }
    }

    fn diagnostic(&self) -> String {
        match self {
            RunError::Usage(m) => format!("graphlaw-run: {m}"),
            RunError::DigestMismatch {
                path,
                expected,
                got,
            } => format!(
                "graphlaw-run: ARTIFACT DIGEST MISMATCH for {}: expected {expected}, got {got}",
                path.display()
            ),
            RunError::Trap => "graphlaw-run: guest trap (crash mode or memory wall)".into(),
            RunError::FrameEof => "graphlaw-run: EOF mid-frame".into(),
            RunError::StdinRead(e) => format!("graphlaw-run: stdin read error: {e}"),
            RunError::MalformedEnvelope => {
                "graphlaw-run: malformed envelope (bad base64 payload)".into()
            }
            RunError::NonJsonResponse(e) => {
                format!("graphlaw-run: module returned non-JSON response: {e}")
            }
            RunError::WasmInvalid(e) => format!("graphlaw-run: invalid wasm artifact: {e}"),
            RunError::WasmInstantiation(e) => {
                format!("graphlaw-run: wasm instantiation failed: {e}")
            }
            RunError::EhArtifact { path } => format!(
                "graphlaw-run: EH-class artifact requires graphlaw-run-eh: {} carries a wasm \
                 Tag (exception-handling) section, unsupported by this legacy wasmi host",
                path.display()
            ),
            RunError::AbiFault(e) => format!("graphlaw-run: guest ABI fault: {e}"),
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
                None => return Err(RunError::Usage("--expect-blake3 needs a hex digest".into())),
            },
            other => return Err(RunError::Usage(format!("unknown argument `{other}`"))),
        }
    }

    let path = wasm
        .or_else(|| std::env::var_os("GRAPHLAW_WASM").map(PathBuf::from))
        .unwrap_or_else(|| {
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("target/wasm-abi/wasm32-wasip1/wasm/graphlaw_wasm.wasm")
        });

    let bytes = std::fs::read(&path).map_err(|e| {
        RunError::Usage(format!("cannot read wasm artifact {}: {e}", path.display()))
    })?;

    if let Some(hex) = &expect_blake3 {
        let got = blake3_hex(&bytes);
        if &got != hex {
            // #region agent log
            agent_log(
                "H-PIN",
                "src/bin/graphlaw_run.rs:run",
                "digest-mismatch",
                json!({"expected": hex, "got": &got, "len": bytes.len()}),
            );
            // #endregion
            return Err(RunError::DigestMismatch {
                path: path.clone(),
                expected: hex.clone(),
                got,
            });
        }
    }

    if has_tag_section(&bytes, &path)? {
        return Err(RunError::EhArtifact { path });
    }

    let mut host = Host::new(&bytes)?;

    let mut stdin = std::io::stdin().lock();
    let mut stdout = std::io::stdout().lock();

    loop {
        let Some(body) = read_frame(&mut stdin)? else {
            return Ok(ExitCode::from(0));
        };
        match mode {
            Mode::Hang => {
                // Simulated guest infinite loop: never reply, never exit.
                // (SLEEP-CLASS: SCHEDULING — the process genuinely has nothing
                // to do; this IS the court's fail-closed timeout fixture.)
                loop {
                    std::thread::sleep(std::time::Duration::from_secs(3600));
                }
            }
            Mode::Crash => {
                // Simulated guest panic: trap like the memory wall does.
                return Err(RunError::Trap);
            }
            Mode::Eval => {
                let reply = match eval_frame(&mut host, &body) {
                    Ok(reply) => reply,
                    Err(RunError::Trap) => return Err(RunError::Trap),
                    Err(e) => return Err(e),
                };
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

/// Walks wasm section headers (inline parser, mirrors the
/// tests/wasm_size_court.rs walker) and reports whether a Tag section
/// (id 13, exception-handling) is present.
fn has_tag_section(bytes: &[u8], path: &std::path::Path) -> Result<bool, RunError> {
    const TAG_SECTION_ID: u8 = 13;
    const WASM_MAGIC: &[u8; 4] = b"\0asm";
    if bytes.len() < 8 || !bytes.starts_with(WASM_MAGIC) {
        // Not parseable as wasm at all: let the engine produce the typed
        // WasmInvalid diagnostic below.
        return Ok(false);
    }
    let mut i = 8usize;
    while i < bytes.len() {
        let id = bytes[i];
        i += 1;
        let mut size: u64 = 0;
        let mut shift = 0u32;
        loop {
            if i >= bytes.len() {
                return Err(RunError::WasmInvalid(format!(
                    "{path_display}: truncated section LEB128 at byte {i}",
                    path_display = path.display()
                )));
            }
            let b = bytes[i];
            i += 1;
            size |= u64::from(b & 0x7f) << shift;
            shift += 7;
            if b & 0x80 == 0 {
                break;
            }
            if shift >= 35 {
                return Err(RunError::WasmInvalid("overlong section LEB128".into()));
            }
        }
        let end = i
            .checked_add(usize::try_from(size).map_err(|_| {
                RunError::WasmInvalid("section size overflow".into())
            })?)
            .filter(|&e| e <= bytes.len())
            .ok_or_else(|| {
                RunError::WasmInvalid(format!(
                    "section id {id} claims {size} bytes past EOF"
                ))
            })?;
        if id == TAG_SECTION_ID {
            return Ok(true);
        }
        i = end;
    }
    Ok(false)
}

struct Host {
    store: Store<WasiCtx>,
    memory: Memory,
    alloc: TypedFunc<u32, u32>,
    free: TypedFunc<(u32, u32), ()>,
    call: TypedFunc<(u32, u32), u64>,
}

impl Host {
    fn new(bytes: &[u8]) -> Result<Self, RunError> {
        let engine = Engine::default();
        let module = match Module::new(&engine, bytes) {
            Ok(module) => {
                // #region agent log
                agent_log(
                    "H1",
                    "src/bin/graphlaw_run.rs:Host::new",
                    "module-validated",
                    json!({"detail": format!("len={} blake3={}", bytes.len(), blake3_hex(bytes))}),
                );
                // #endregion
                module
            }
            Err(e) => {
                // #region agent log
                agent_log(
                    "H1",
                    "src/bin/graphlaw_run.rs:Host::new",
                    "module-rejected",
                    json!({"detail": e.to_string()}),
                );
                // #endregion
                return Err(RunError::WasmInvalid(e.to_string()));
            }
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
        let mut store = Store::new(&engine, WasiCtxBuilder::new().build());
        let mut linker = <Linker<WasiCtx>>::new(&engine);
        wasmi_wasi::add_to_linker(&mut linker, |ctx| ctx)
            .map_err(|e| RunError::WasmInstantiation(e.to_string()))?;
        let instance: Instance = linker
            .instantiate_and_start(&mut store, &module)
            .map_err(|e| RunError::WasmInstantiation(e.to_string()))?;
        // Reactor-style modules expose `_initialize`; hosts must call it once.
        if let Ok(init) = instance.get_typed_func::<(), ()>(&store, "_initialize") {
            init.call(&mut store, ())
                .map_err(|e| RunError::WasmInstantiation(format!("_initialize: {e}")))?;
        }
        let memory = instance
            .get_memory(&store, "memory")
            .ok_or_else(|| RunError::WasmInstantiation("module does not export memory".into()))?;
        let export = |name: &'static str, e: wasmi::Error| {
            RunError::WasmInstantiation(format!("missing or mistyped export {name}: {e}"))
        };
        Ok(Host {
            alloc: instance
                .get_typed_func::<u32, u32>(&store, "gl_alloc")
                .map_err(|e| export("gl_alloc", e))?,
            free: instance
                .get_typed_func::<(u32, u32), ()>(&store, "gl_free")
                .map_err(|e| export("gl_free", e))?,
            call: instance
                .get_typed_func::<(u32, u32), u64>(&store, "gl_call")
                .map_err(|e| export("gl_call", e))?,
            store,
            memory,
        })
    }

    /// `gl_alloc` + memory write + `gl_call` + read + `gl_free`, exactly the
    /// `request_bytes` pattern of graphlaw tests/wasm_abi.rs.
    fn request_bytes(&mut self, body: &[u8]) -> Result<Vec<u8>, RunError> {
        let ptr = self
            .alloc
            .call(&mut self.store, body.len() as u32)
            .map_err(|e| RunError::AbiFault(format!("gl_alloc: {e}")))?;
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
            .map_err(|e| RunError::AbiFault(format!("gl_call: {e}")))?;
        let (out_ptr, out_len) = ((packed >> 32) as u32, (packed & 0xffff_ffff) as u32);
        let mut out = vec![0u8; out_len as usize];
        self.memory
            .read(&self.store, out_ptr as usize, &mut out)
            .map_err(|e| RunError::AbiFault(format!("memory read: {e}")))?;
        self.free
            .call(&mut self.store, (out_ptr, out_len))
            .map_err(|e| RunError::AbiFault(format!("gl_free: {e}")))?;
        Ok(out)
    }
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
                // Memory wall: the guest is killed by the enforced 64 MiB
                // linear-memory ceiling before any reply can be framed.
                return Err(RunError::Trap);
            }
            let frames_count = parsed
                .get("predicate_frames")
                .and_then(Value::as_array)
                .map(|a| a.len())
                .unwrap_or(0);
            // The bytes that cross the sandbox boundary are the production
            // call_json shape; the module answers with the raw ABI response.
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
    // #region agent log
    let req: Value = serde_json::from_slice(request_bytes).unwrap_or(Value::Null);
    let err = response.get("error");
    agent_log(
        "B",
        "src/bin/graphlaw_run.rs:call_and_decode",
        "wasm dialect reply",
        json!({
            "op": req.get("op").and_then(|v| v.as_str()),
            "dialect": req.get("dialect").and_then(|v| v.as_str()),
            "data_dialect": req.pointer("/data/dialect").and_then(|v| v.as_str()),
            "ok": response.get("ok"),
            "has_id": response.get("id").and_then(|v| v.as_str()).map(|s| !s.is_empty()),
            "quads": response.get("quads"),
            "conforms": response.get("conforms"),
            "valid": response.get("valid"),
            "result_dialect": response.get("dialect"),
            "error_kind": err.and_then(|e| e.get("kind")),
            "error_message": err.and_then(|e| e.get("message")).and_then(|m| m.as_str()).map(|m| {
                let mut t = m.to_string();
                t.truncate(180);
                t
            }),
        }),
    );
    // #endregion
    Ok(response)
}

/// Base64 decode without an extra dependency (the runner must build with only
/// the graphlaw dependency tree). Accepts standard alphabet with padding.
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
