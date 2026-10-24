//! Env-isolation court for the packet-4 stdio runners (C0b family). Black-box
//! subprocess court in the pattern of tests/graphlaw_run_eh_court.rs.
//!
//! Claim under court: a host secret planted in the runner's parent environment
//! (`SECRET_TEST_TOKEN`) never surfaces in the runner child's stdout or
//! stderr — not through the module's receipt/echo path (a real `capabilities`
//! request against the real DuckDB-linked artifact, driven both as a raw ABI
//! frame and as the smalt eval envelope), and not through any host
//! diagnostic.
//!
//! Anti-vacuity: the "token appears nowhere" checker is itself put on trial —
//! a positive control runs `/bin/sh -c 'printf %s "$SECRET_TEST_TOKEN"'`
//! through the same checker and REQUIRES the checker to flag the leak. A
//! checker that cannot detect a real echo is a vacuous gate and fails this
//! court.
//!
//! Binary resolution: `GRAPHLAW_RUN_EH` env, else the lane/debug target root;
//! a missing binary is built (`cargo build --bin graphlaw-run-eh --features
//! runner-eh`) before the court skips loudly. Artifact resolution mirrors the
//! host ladder (`GRAPHLAW_WASM` > lane target > `target/` > sibling lane
//! roots); no artifact is a loud typed SKIP (panic), never a silent pass.
//!
//! No fixed coordination sleeps: every child wait is the one bounded poll
//! (50 ms step, deadline-checked, loud kill on overrun) —
//! SLEEP-CLASS: POLL-STEP.

use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

const TOKEN_NAME: &str = "SECRET_TEST_TOKEN";
/// Distinctive canary value; deterministic (no RNG) so a leak is greppable
/// in the court output itself.
const TOKEN_VALUE: &str = "gl-env-court-canary-6f19e2ac";
const POLL_STEP: Duration = Duration::from_millis(50);
/// The 46 MB DuckDB-linked artifact is JIT-compiled on first spawn (cached
/// afterwards). A timeout here is a FAILURE (loud), not a pass.
const ARTIFACT_DEADLINE: Duration = Duration::from_secs(300);
const BIN_BUILD_DEADLINE: Duration = Duration::from_secs(1200);

// -------------------------------------------------- binary/artifact res --

fn manifest() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn lane_target_root() -> PathBuf {
    match std::env::var_os("CARGO_TARGET_DIR") {
        Some(d) if !d.is_empty() => {
            let p = PathBuf::from(&d);
            if p.is_absolute() {
                p
            } else {
                manifest().join(p)
            }
        }
        _ => manifest().join("target"),
    }
}

fn eh_bin() -> Option<PathBuf> {
    if let Some(p) = std::env::var_os("GRAPHLAW_RUN_EH") {
        let p = PathBuf::from(p);
        return p.exists().then_some(p);
    }
    [
        lane_target_root().join("debug/graphlaw-run-eh"),
        manifest().join("target/debug/graphlaw-run-eh"),
    ]
    .into_iter()
    .find(|p| p.exists())
}

/// Build the EH host if missing, then re-resolve; loud typed skip (panic) if
/// it still cannot be produced.
fn ensure_eh_bin() -> PathBuf {
    if let Some(p) = eh_bin() {
        return p;
    }
    eprintln!("ENV-COURT: graphlaw-run-eh not found; building (--features runner-eh)");
    let cargo = std::env::var("CARGO").unwrap_or_else(|_| "cargo".into());
    let mut child = Command::new(&cargo)
        .current_dir(manifest())
        .args(["build", "--bin", "graphlaw-run-eh", "--features", "runner-eh"])
        .stdin(Stdio::null())
        .spawn()
        .expect("spawn cargo build for graphlaw-run-eh");
    let start = Instant::now();
    let status = loop {
        match child.try_wait().expect("poll cargo build") {
            Some(s) => break s,
            None => {
                if start.elapsed() > BIN_BUILD_DEADLINE {
                    let _ = child.kill();
                    let _ = child.wait();
                    panic!(
                        "ENV-COURT BUILD TIMEOUT: cargo build --bin graphlaw-run-eh \
                         did not finish in {}s",
                        BIN_BUILD_DEADLINE.as_secs()
                    );
                }
                std::thread::sleep(POLL_STEP); // SLEEP-CLASS: POLL-STEP
            }
        }
    };
    if !status.success() {
        panic!("ENV-COURT BUILD FAILED: cargo build --bin graphlaw-run-eh exited {status}");
    }
    eh_bin().unwrap_or_else(|| {
        panic!(
            "GRAPH LAW ENV-COURT SKIP: graphlaw-run-eh still unresolved after build \
             (set GRAPHLAW_RUN_EH or the lane CARGO_TARGET_DIR)"
        )
    })
}

fn artifact_path() -> Option<PathBuf> {
    if let Some(p) = std::env::var_os("GRAPHLAW_WASM") {
        let p = PathBuf::from(p);
        return p.exists().then_some(p);
    }
    let mut candidates = vec![
        lane_target_root().join("wasm32-wasip1/wasm/graphlaw_wasm.wasm"),
        manifest().join("target/wasm32-wasip1/wasm/graphlaw_wasm.wasm"),
    ];
    // Sibling lane/coordinator build roots (leases — resolved last, sorted
    // for determinism, and the resolved path is printed in the court output).
    if let Ok(entries) = std::fs::read_dir(manifest()) {
        let mut globs: Vec<PathBuf> = entries
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .filter(|p| p.is_dir() && p.file_name().is_some_and(|n| n.to_string_lossy().starts_with("target")))
            .flat_map(|root| {
                [
                    root.join("wasm32-wasip1/wasm/graphlaw_wasm.wasm"),
                    root.join("wasm-abi/wasm32-wasip1/wasm/graphlaw_wasm.wasm"),
                ]
            })
            .filter(|p| p.exists())
            .collect();
        globs.sort();
        candidates.extend(globs);
    }
    candidates.into_iter().find(|p| p.exists())
}

// ------------------------------------------------------------- harness --

struct Outcome {
    code: i32,
    stdout: Vec<u8>,
    stderr: String,
}

/// Spawn `bin --wasm artifact` with the token planted in its environment,
/// feed one packet-4 frame, wait bounded.
fn run_host_frame(bin: &Path, artifact: &Path, frame_body: &[u8], label: &str) -> Outcome {
    let mut child = Command::new(bin)
        .arg("--wasm")
        .arg(artifact)
        .env_remove("GRAPHLAW_WASM") // pin the artifact via --wasm only
        .env(TOKEN_NAME, TOKEN_VALUE) // the plant under test
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap_or_else(|e| panic!("ENV-COURT: spawn {label} failed: {e}"));
    let mut frame = Vec::with_capacity(4 + frame_body.len());
    frame.extend_from_slice(&(frame_body.len() as u32).to_be_bytes());
    frame.extend_from_slice(frame_body);
    if let Some(mut si) = child.stdin.take() {
        si.write_all(&frame)
            .expect("write frame (child consumes as we write)");
    }
    let child_stdout = child.stdout.take();
    let child_stderr = child.stderr.take();
    // Drain both pipes CONCURRENTLY with the wait: the host can emit a
    // per-frame guest backtrace on refusal paths (fuel exhaustion), and a
    // pipe saturation deadlock here would masquerade as a runner hang.
    let stdout_handle = std::thread::spawn(move || {
        let mut buf = Vec::new();
        if let Some(mut o) = child_stdout {
            let _ = o.read_to_end(&mut buf);
        }
        buf
    });
    let stderr_handle = std::thread::spawn(move || {
        let mut s = String::new();
        if let Some(mut e) = child_stderr {
            let _ = e.read_to_string(&mut s);
        }
        s
    });
    let start = Instant::now();
    let status = loop {
        match child.try_wait().expect("poll runner") {
            Some(s) => break s,
            None => {
                if start.elapsed() > ARTIFACT_DEADLINE {
                    let _ = child.kill();
                    let _ = child.wait();
                    panic!(
                        "ENV-COURT TIMEOUT: {label} did not exit within {}s — \
                         hang is a failure, not a pass",
                        ARTIFACT_DEADLINE.as_secs()
                    );
                }
                std::thread::sleep(POLL_STEP); // SLEEP-CLASS: POLL-STEP
            }
        }
    };
    let stdout = stdout_handle.join().expect("stdout reader thread");
    let stderr = stderr_handle.join().expect("stderr reader thread");
    Outcome {
        code: status.code().unwrap_or_else(|| panic!("{label}: no exit code: {status}")),
        stdout,
        stderr,
    }
}

/// The checker under trial: does the planted secret surface anywhere in the
/// child's stdio? Returns true when the token (name OR value) is present.
fn token_surfaces(stdout: &[u8], stderr: &str) -> bool {
    let stdout_str = String::from_utf8_lossy(stdout);
    stdout_str.contains(TOKEN_VALUE)
        || stdout_str.contains(TOKEN_NAME)
        || stderr.contains(TOKEN_VALUE)
        || stderr.contains(TOKEN_NAME)
}

/// The op must produce a REAL observable outcome before the absence assertion
/// runs — a spawn that died before the module reply path cannot speak for the
/// reply path. Exit 0 = reply path ran (reply parsed); exit 97 = guest trap
/// past the frame layer; exit 98 = the typed fuel-exhaustion bound (the guest
/// ran under metering until the budget bound fired). Each branch requires
/// real child output; anything else is a typed failure.
fn assert_real_outcome(out: &Outcome, label: &str) {
    match out.code {
        0 => {
            assert!(
                out.stdout.len() >= 4,
                "{label}: exit 0 but no reply frame on stdout"
            );
            let len =
                u32::from_be_bytes([out.stdout[0], out.stdout[1], out.stdout[2], out.stdout[3]])
                    as usize;
            assert!(
                out.stdout.len() >= 4 + len,
                "{label}: reply frame truncated"
            );
            let v: serde_json::Value = serde_json::from_slice(&out.stdout[4..4 + len])
                .unwrap_or_else(|e| panic!("{label}: reply not JSON: {e}"));
            assert_eq!(
                v.get("ok").and_then(|o| o.as_bool()),
                Some(true),
                "{label}: reply envelope must be ok:true — {v}"
            );
        }
        97 | 98 => {
            assert!(
                !out.stderr.trim().is_empty(),
                "{label}: exit {} but stderr carries no guest diagnostic — \
                 the pinned refusal path did not run",
                out.code
            );
        }
        other => panic!(
            "{label}: unexpected exit {other} — stderr: {}",
            out.stderr
        ),
    }
}

// ---------------------------------------------------------------- courts --

/// Anti-vacuity leg: the checker MUST flag a real env echo and MUST stay
/// silent on clean output. A court whose absence-assertion cannot fail is
/// not a court.
#[test]
fn checker_positive_control_detects_a_real_env_echo() {
    let out = Command::new("/bin/sh")
        .arg("-c")
        .arg(format!("printf %s \"${TOKEN_NAME}\""))
        .env(TOKEN_NAME, TOKEN_VALUE)
        .output()
        .expect("spawn /bin/sh positive control");
    let stdout = String::from_utf8_lossy(&out.stdout).to_string();
    assert_eq!(
        stdout, TOKEN_VALUE,
        "positive control must actually echo the token (checker input is real)"
    );
    assert!(
        token_surfaces(out.stdout.as_slice(), &stdout),
        "VACUOUS GATE: checker failed to flag a REAL token echo"
    );
    assert!(
        !token_surfaces(br#"{"ok":true,"result":{}}"#, ""),
        "checker flagged clean output — false positive poisons the court"
    );
}

/// The dispatch leg: EH host + real DuckDB-linked artifact, raw `capabilities`
/// ABI frame. The planted token must appear NOWHERE in child stdout/stderr.
#[test]
fn eh_host_capabilities_op_never_surfaces_env() {
    let bin = ensure_eh_bin();
    let art = artifact_path().unwrap_or_else(|| {
        panic!(
            "GRAPH LAW ENV-COURT SKIP: no wasm artifact resolved (set GRAPHLAW_WASM \
             or build target/wasm32-wasip1/wasm/graphlaw_wasm.wasm)"
        )
    });
    eprintln!("ENV-COURT: artifact = {}", art.display());
    let out = run_host_frame(&bin, &art, br#"{"op":"capabilities"}"#, "eh capabilities");
    assert_real_outcome(&out, "eh capabilities");
    assert!(
        !token_surfaces(&out.stdout, &out.stderr),
        "ENV LEAK: SECRET_TEST_TOKEN surfaced in the runner child's stdio\n\
         stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&out.stdout),
        out.stderr
    );
}

/// Second ingress shape: the smalt eval envelope (`payload_b64`) carrying the
/// same capabilities request — the base64/decode/forward path must be as
/// env-silent as the raw frame path.
#[test]
fn eh_host_eval_envelope_never_surfaces_env() {
    let bin = ensure_eh_bin();
    let art = artifact_path().unwrap_or_else(|| {
        panic!(
            "GRAPH LAW ENV-COURT SKIP: no wasm artifact resolved (set GRAPHLAW_WASM \
             or build target/wasm32-wasip1/wasm/graphlaw_wasm.wasm)"
        )
    });
    let envelope = serde_json::json!({
        "op": "eval",
        "payload_b64": base64_encode(br#"{"op":"capabilities"}"#),
        "predicate_frames": [],
    })
    .to_string();
    let out = run_host_frame(&bin, &art, envelope.as_bytes(), "eh eval envelope");
    assert_real_outcome(&out, "eh eval envelope");
    assert!(
        !token_surfaces(&out.stdout, &out.stderr),
        "ENV LEAK: SECRET_TEST_TOKEN surfaced via the eval-envelope path\n\
         stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&out.stdout),
        out.stderr
    );
}

/// Standard-alphabet base64 with padding (encode side of the runner's
/// hand-rolled decoder).
fn base64_encode(bytes: &[u8]) -> String {
    const ALPHABET: &[u8; 64] =
        b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity(bytes.len().div_ceil(3) * 4);
    for chunk in bytes.chunks(3) {
        let b = [
            chunk[0],
            chunk.get(1).copied().unwrap_or(0),
            chunk.get(2).copied().unwrap_or(0),
        ];
        let n = (u32::from(b[0]) << 16) | (u32::from(b[1]) << 8) | u32::from(b[2]);
        out.push(ALPHABET[(n >> 18) as usize & 63] as char);
        out.push(ALPHABET[(n >> 12) as usize & 63] as char);
        out.push(if chunk.len() > 1 {
            ALPHABET[(n >> 6) as usize & 63] as char
        } else {
            '='
        });
        out.push(if chunk.len() > 2 {
            ALPHABET[n as usize & 63] as char
        } else {
            '='
        });
    }
    out
}
