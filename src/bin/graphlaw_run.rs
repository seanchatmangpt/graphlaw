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
//! Other non-zero exits are harness faults (90 EOF mid-frame, 91 stdin read
//! error, 93 malformed envelope, 94 non-JSON module response, 84 artifact
//! digest mismatch refusal, 2 usage/I/O).
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

use serde_json::{Value, json};
use wasmi::{Engine, Instance, Linker, Memory, Module, Store, TypedFunc};
use wasmi_wasi::{WasiCtx, WasiCtxBuilder};

const WALL_BYTES: u32 = 67_108_864; // wasmtime `--max-memory 67108864` parity

struct Host {
    store: Store<WasiCtx>,
    memory: Memory,
    alloc: TypedFunc<u32, u32>,
    free: TypedFunc<(u32, u32), ()>,
    call: TypedFunc<(u32, u32), u64>,
}

impl Host {
    fn new(bytes: &[u8]) -> Self {
        let engine = Engine::default();
        let module = Module::new(&engine, bytes).expect("valid wasm module");
        // The only imports allowed are WASI's (clock, random, stdio).
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

    /// `gl_alloc` + memory write + `gl_call` + read + `gl_free`, exactly the
    /// `request_bytes` pattern of graphlaw tests/wasm_abi.rs.
    fn request_bytes(&mut self, body: &[u8]) -> Vec<u8> {
        let ptr = self
            .alloc
            .call(&mut self.store, body.len() as u32)
            .expect("gl_alloc");
        assert_ne!(ptr, 0, "gl_alloc refused a request the caller pre-checked");
        self.memory
            .write(&mut self.store, ptr as usize, body)
            .expect("memory write");
        let packed = self
            .call
            .call(&mut self.store, (ptr, body.len() as u32))
            .expect("gl_call");
        let (out_ptr, out_len) = ((packed >> 32) as u32, (packed & 0xffff_ffff) as u32);
        let mut out = vec![0u8; out_len as usize];
        self.memory
            .read(&self.store, out_ptr as usize, &mut out)
            .expect("memory read");
        self.free
            .call(&mut self.store, (out_ptr, out_len))
            .expect("gl_free");
        out
    }
}

fn main() {
    let mut wasm: Option<PathBuf> = None;
    let mut expect_blake3: Option<String> = None;
    let mut mode = Mode::Eval;
    let mut args = std::env::args().skip(1);
    while let Some(a) = args.next() {
        match a.as_str() {
            "--crash" => mode = Mode::Crash,
            "--hang" => mode = Mode::Hang,
            "--wasm" => wasm = Some(PathBuf::from(args.next().expect("--wasm needs a path"))),
            "--expect-blake3" => {
                expect_blake3 = Some(args.next().expect("--expect-blake3 needs a hex digest"))
            }
            other => {
                eprintln!("graphlaw-run: unknown argument `{other}`");
                std::process::exit(2);
            }
        }
    }

    let path = wasm
        .or_else(|| std::env::var_os("GRAPHLAW_WASM").map(PathBuf::from))
        .unwrap_or_else(|| {
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("target/wasm-abi/wasm32-wasip1/wasm/graphlaw_wasm.wasm")
        });

    let bytes = std::fs::read(&path).unwrap_or_else(|e| {
        eprintln!("graphlaw-run: cannot read wasm artifact {}: {e}", path.display());
        std::process::exit(2);
    });

    if let Some(hex) = &expect_blake3 {
        let got = blake3_hex(&bytes);
        if &got != hex {
            eprintln!(
                "graphlaw-run: ARTIFACT DIGEST MISMATCH for {}: expected {hex}, got {got}",
                path.display()
            );
            std::process::exit(84);
        }
    }

    let mut host = Host::new(&bytes);

    let mut stdin = std::io::stdin().lock();
    let mut stdout = std::io::stdout().lock();

    loop {
        let Some(body) = read_frame(&mut stdin) else { std::process::exit(0) };
        match mode {
            Mode::Hang => {
                // Simulated guest infinite loop: never reply, never exit.
                loop {
                    std::thread::sleep(std::time::Duration::from_secs(3600));
                }
            }
            Mode::Crash => {
                // Simulated guest panic: trap like the memory wall does.
                std::process::exit(86);
            }
            Mode::Eval => {
                let reply = eval_frame(&mut host, &body);
                write_frame(&mut stdout, &reply);
                stdout.flush().expect("flush");
            }
        }
    }
}

enum Mode {
    Eval,
    Crash,
    Hang,
}

/// Reads one packet-4 frame: 4-byte BE length prefix + body. Returns None on a
/// clean EOF at a frame boundary.
fn read_frame(stdin: &mut impl Read) -> Option<Vec<u8>> {
    let mut len_buf = [0u8; 4];
    match stdin.read_exact(&mut len_buf) {
        Ok(()) => {}
        Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => return None,
        Err(_) => std::process::exit(91),
    }
    let len = u32::from_be_bytes(len_buf) as usize;
    let mut body = vec![0u8; len];
    match stdin.read_exact(&mut body) {
        Ok(()) => Some(body),
        Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => std::process::exit(90),
        Err(_) => std::process::exit(91),
    }
}

fn write_frame(stdout: &mut impl Write, body: &[u8]) {
    let mut buf = Vec::with_capacity(4 + body.len());
    buf.extend_from_slice(&(body.len() as u32).to_be_bytes());
    buf.extend_from_slice(body);
    stdout.write_all(&buf).expect("frame write");
}

fn eval_frame(host: &mut Host, body: &[u8]) -> Vec<u8> {
    let parsed: Value = match serde_json::from_slice(body) {
        Ok(v) => v,
        // Not the eval envelope: forward the raw bytes to the module as an
        // ABI request and reply under "result" (abi-mode passthrough).
        Err(_) => {
            let response = call_and_decode(host, body);
            return serde_json::to_vec(&json!({"ok": true, "result": response}))
                .expect("reply serializes");
        }
    };

    match parsed.get("op").and_then(Value::as_str) {
        Some("eval") => {
            let payload_b64 = parsed
                .get("payload_b64")
                .and_then(Value::as_str)
                .unwrap_or("");
            let payload = match base64_decode(payload_b64) {
                Some(p) => p,
                None => std::process::exit(93),
            };
            if payload.len() > WALL_BYTES as usize {
                // Memory wall: the guest is killed by the enforced 64 MiB
                // linear-memory ceiling before any reply can be framed.
                std::process::exit(86);
            }
            let frames_count = parsed
                .get("predicate_frames")
                .and_then(Value::as_array)
                .map(|a| a.len())
                .unwrap_or(0);
            // The bytes that cross the sandbox boundary are the production
            // call_json shape; the module answers with the raw ABI response.
            let response = call_and_decode(host, &payload);
            let reply = json!({
                "ok": true,
                "result": response,
                "predicate_frames_count": frames_count,
            });
            serde_json::to_vec(&reply).expect("reply serializes")
        }
        // Any other op: the frame IS an ABI request; same envelope, raw
        // response as "result".
        _ => {
            let response = call_and_decode(host, body);
            serde_json::to_vec(&json!({"ok": true, "result": response}))
                .expect("reply serializes")
        }
    }
}

fn call_and_decode(host: &mut Host, request_bytes: &[u8]) -> Value {
    let abi_response = host.request_bytes(request_bytes);
    serde_json::from_slice(&abi_response).unwrap_or_else(|e| {
        eprintln!("graphlaw-run: module returned non-JSON response: {e}");
        std::process::exit(94);
    })
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
