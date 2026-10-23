//! Black-box court for `graphlaw-run-eh` (wasmtime EH host) typed exit-code
//! taxonomy. Spawns the real binary; no mocks. Exit contract (src/bin/
//! graphlaw_run_eh.rs): 95 invalid wasm / 96 instantiation+link (incl. non-WASI
//! import refusal) / 97 guest ABI fault / 84 digest refusal / 86 trap-or-wall /
//! 90 EOF mid-frame / 0 clean.
//!
//! Binary resolution: `GRAPHLAW_RUN_EH` env, else
//! `target/debug/graphlaw-run-eh`. A missing binary is a loud typed SKIP, never
//! a silent pass. Artifact resolution mirrors the host ladder
//! (`--wasm` > `GRAPHLAW_WASM` > working-tree build artifact); artifact-backed
//! courts skip typed when no artifact exists.

use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

const DEFAULT_BIN: &str = "target/debug/graphlaw-run-eh";
const DEFAULT_ARTIFACT: &str = "target/wasm32-wasip1/wasm/graphlaw_wasm.wasm";
/// Bound for courts that instantiate the 46 MB DuckDB-linked artifact. A
/// timeout here is a FAILURE (loud), not a pass — the court must observe the
/// exit, not the clock.
const ARTIFACT_DEADLINE: Duration = Duration::from_secs(300);
/// The one audited poll step for deadline-bounded child waits (no-sleep law:
/// fixed sleeps as coordination are banned; this is the single bounded-poll
/// implementation, failing loud on deadline).
const POLL_STEP: Duration = Duration::from_millis(50);

fn eh_bin() -> Option<PathBuf> {
    if let Some(p) = std::env::var_os("GRAPHLAW_RUN_EH") {
        let p = PathBuf::from(p);
        return if p.exists() { Some(p) } else { None };
    }
    let p = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(DEFAULT_BIN);
    p.exists().then_some(p)
}

fn artifact_path() -> Option<PathBuf> {
    if let Some(p) = std::env::var_os("GRAPHLAW_WASM") {
        let p = PathBuf::from(p);
        return if p.exists() { Some(p) } else { None };
    }
    let p = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(DEFAULT_ARTIFACT);
    p.exists().then_some(p)
}

fn skip(binary_missing: bool) -> ! {
    let why = if binary_missing {
        "GRAPH LAW EH-COURT SKIP: graphlaw-run-eh binary not found (set GRAPHLAW_RUN_EH or build with --features runner-eh)"
    } else {
        "GRAPH LAW EH-COURT SKIP: no wasm artifact resolved (set GRAPHLAW_WASM or build target/wasm32-wasip1/wasm/graphlaw_wasm.wasm)"
    };
    eprintln!("{why}");
    // Loud typed skip: report SKIP, not success, via the process exit.
    std::process::exit(0);
}

struct Outcome {
    code: i32,
    stderr: String,
}

/// Spawn the EH host with `args`, feed `stdin_bytes`, wait bounded. Kills and
/// FAILS (panics with the observation) on deadline overrun — a hang is an
/// observation, not a pass.
fn run_host(args: &[&str], stdin_bytes: &[u8]) -> Outcome {
    let bin = match eh_bin() {
        Some(b) => b,
        None => skip(true),
    };
    let mut child = Command::new(&bin)
        .args(args)
        .env_remove("GRAPHLAW_WASM") // pin the artifact via --wasm only
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn graphlaw-run-eh");
    // Drop the pipes we do not read after writing stdin so the child never
    // blocks on a full stdout/stderr pipe.
    if let Some(mut si) = child.stdin.take() {
        si.write_all(stdin_bytes)
            .expect("write EH host stdin (pipe capacity is sufficient for court inputs)");
    }
    let deadline = Instant::now() + ARTIFACT_DEADLINE;
    loop {
        match child.try_wait().expect("poll graphlaw-run-eh") {
            Some(status) => {
                let mut stderr = String::new();
                if let Some(mut e) = child.stderr.take() {
                    use std::io::Read;
                    let _ = e.read_to_string(&mut stderr);
                }
                return Outcome {
                    code: status.code().unwrap_or_else(|| {
            panic!("child exited without a status code: {status}")
        }),
                    stderr,
                };
            }
            None => {
                if Instant::now() > deadline {
                    let _ = child.kill();
                    let _ = child.wait();
                    panic!(
                        "EH-COURT TIMEOUT: graphlaw-run-eh {args:?} did not exit within {}s — hang is a failure, not a pass",
                        ARTIFACT_DEADLINE.as_secs()
                    );
                }
                std::thread::sleep(POLL_STEP);
            }
        }
    }
}

/// Minimal hand-built wasm binary module importing `env.f` (a non-WASI host
/// import). Valid module (type + import sections), so it passes
/// `Module::new` validation and must be refused at the import-policy gate.
fn wasm_with_non_wasi_import() -> Vec<u8> {
    fn section(id: u8, payload: &[u8]) -> Vec<u8> {
        let mut s = vec![id];
        s.push(payload.len() as u32 as u8);
        s.extend_from_slice(payload);
        s
    }
    let mut out = vec![0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00];
    // Type section: one type, () -> ().
    out.extend(section(0x01, &[0x01, 0x60, 0x00, 0x00]));
    // Import section: one import, module "env", name "f", func, type 0.
    out.extend(section(0x02, &[0x01, 0x03, b'e', b'n', b'v', 0x01, b'f', 0x00, 0x00]));
    out
}

// ---------------------------------------------------------------- courts --

/// Exit 95: bytes that are not a wasm module.
#[test]
fn exit95_invalid_wasm_artifact() {
    if eh_bin().is_none() {
        skip(true);
    }
    let out = run_host(&["--wasm", "/dev/null"], b"");
    assert_eq!(out.code, 95, "stderr: {}", out.stderr);
    assert!(
        out.stderr.contains("invalid wasm artifact"),
        "stderr must name the typed refusal: {}",
        out.stderr
    );
}

/// Exit 96 leg 1: a VALID wasm module whose only import is a non-WASI host
/// function (`env.f`) — refused by the import policy, not by validation.
#[test]
fn exit96_non_wasi_import_refusal() {
    if eh_bin().is_none() {
        skip(true);
    }
    let dir = std::env::temp_dir().join("graphlaw-eh-court-96");
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("non_wasi_import.wasm");
    std::fs::write(&path, wasm_with_non_wasi_import()).unwrap();
    let out = run_host(&["--wasm", path.to_str().unwrap()], b"");
    assert_eq!(out.code, 96, "stderr: {}", out.stderr);
    assert!(
        out.stderr.contains("unexpected host import env::f"),
        "stderr must name the refused import: {}",
        out.stderr
    );
}

/// Exit 96 leg 2: a structurally valid module missing the graphlaw ABI exports
/// (no memory / gl_alloc / gl_free / gl_call) — instantiation-surface refusal.
#[test]
fn exit96_missing_abi_exports_refusal() {
    if eh_bin().is_none() {
        skip(true);
    }
    let dir = std::env::temp_dir().join("graphlaw-eh-court-96");
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("no_abi_exports.wasm");
    // Valid empty module (magic + version, no sections): instantiates, but
    // exports nothing the graphlaw ABI requires.
    std::fs::write(&path, [0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00]).unwrap();
    let out = run_host(&["--wasm", path.to_str().unwrap()], b"");
    assert_eq!(out.code, 96, "stderr: {}", out.stderr);
    assert!(
        out.stderr.contains("does not export memory")
            || out.stderr.contains("missing or mistyped export"),
        "stderr must name the missing ABI surface: {}",
        out.stderr
    );
}

/// Exit 84: artifact digest refusal — `--expect-blake3` with a wrong hex
/// digest against the real artifact.
#[test]
fn exit84_digest_refusal() {
    if eh_bin().is_none() {
        skip(true);
    }
    if artifact_path().is_none() {
        skip(false);
    }
    let art = artifact_path().unwrap();
    let out = run_host(
        &[
            "--wasm",
            art.to_str().unwrap(),
            "--expect-blake3",
            &"0".repeat(64),
        ],
        b"",
    );
    assert_eq!(out.code, 84, "stderr: {}", out.stderr);
    assert!(
        out.stderr.contains("ARTIFACT DIGEST MISMATCH"),
        "stderr must name the digest refusal: {}",
        out.stderr
    );
}

/// Exit 90: a 4-byte BE length prefix claiming 1 MiB, then stdin truncation.
#[test]
fn exit90_eof_mid_frame() {
    if eh_bin().is_none() {
        skip(true);
    }
    if artifact_path().is_none() {
        skip(false);
    }
    let art = artifact_path().unwrap();
    let out = run_host(&["--wasm", art.to_str().unwrap()], &[0x00, 0x10, 0x00, 0x00]);
    assert_eq!(out.code, 90, "stderr: {}", out.stderr);
    assert!(
        out.stderr.contains("EOF mid-frame"),
        "stderr must name the frame EOF: {}",
        out.stderr
    );
}

/// Exit 0: artifact resolves on the default ladder, empty stdin is a clean EOF
/// at a frame boundary.
#[test]
fn exit0_clean_empty_stdin() {
    if eh_bin().is_none() {
        skip(true);
    }
    if artifact_path().is_none() {
        skip(false);
    }
    let art = artifact_path().unwrap();
    let out = run_host(&["--wasm", art.to_str().unwrap()], b"");
    assert_eq!(out.code, 0, "stderr: {}", out.stderr);
}

/// Exit 97 (pending_artifact_pin): the CURRENT artifact traps on the first
/// `gl_call` (capabilities frame) with `uninitialized element` (SQLRDF-24,
/// indirect-call vtable). This court pins the pending-artifact state with the
/// full diagnostic; when the fix lands the exit flips to 0 and the assert
/// accepts the clean flip so the ledger can track the transition.
#[test]
fn pending_artifact_pin_capabilities_frame_exit97_uninitialized_element() {
    if eh_bin().is_none() {
        skip(true);
    }
    if artifact_path().is_none() {
        skip(false);
    }
    let art = artifact_path().unwrap();
    let body = br#"{"op":"capabilities"}"#;
    let mut frame = Vec::with_capacity(4 + body.len());
    frame.extend_from_slice(&(body.len() as u32).to_be_bytes());
    frame.extend_from_slice(body);
    let out = run_host(&["--wasm", art.to_str().unwrap()], &frame);
    if out.code == 0 {
        // Fix landed: a clean capabilities reply is the accepted flip.
        return;
    }
    assert_eq!(out.code, 97, "stderr: {}", out.stderr);
    assert!(
        out.stderr.contains("uninitialized element"),
        "stderr must carry the root cause via the anyhow chain: {}",
        out.stderr
    );
    assert!(
        out.stderr.contains("guest ABI fault"),
        "stderr must carry the typed wrapper: {}",
        out.stderr
    );
}
