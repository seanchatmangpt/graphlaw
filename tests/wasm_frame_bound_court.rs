//! Frame-bound court for the packet-4 stdio runners (R3,
//! docs/jira/v26.10.23/design-read-frame-bound.md). Black-box subprocess
//! court in the pattern of tests/graphlaw_run_eh_court.rs: spawns the real
//! binaries, feeds hand-built packet-4 frames over real pipes, asserts on
//! exit codes, stderr text, and WHO consumed the body bytes (timing).
//!
//! Contract under court (graphlaw-run-eh — the sole packet-4 runner since the
//! wasmi retirement, docs/jira/v26.10.23/ADR-wasmi-retirement.md):
//! - a frame length prefix > `graphlaw::abi::MAX_REQUEST_BYTES` (16 MiB) is
//!   refused from the prefix ALONE: exit 92, stderr names length AND cap,
//!   zero body bytes read, no reply frame written — the leg holds the child's
//!   stdin OPEN with undrained padding so a body-reading implementation
//!   blocks until the deadline and fails;
//! - a frame of exactly MAX_REQUEST_BYTES is admitted end to end (proves the
//!   check is `>` not `>=`);
//! - cap+1 is refused (boundary twin);
//! - a zero-length frame followed by EOF exits 0 (degenerate-but-lawful).
//!
//! Fixture module: the subject of this court is the RUNNER's frame layer, not
//! the guest, so the legs mount a minimal hand-built wasm module exporting
//! `memory`/`gl_alloc`/`gl_free`/`gl_call` (gl_call answers a fixed JSON
//! response from a data segment). It is a real module: the engine validates,
//! instantiate, and execute it; the ABI env-court separately exercises the
//! real DuckDB artifact's reply path. A missing runner binary is built
//! (`cargo build --bin <bin> --features <feat>`); a binary that cannot be
//! produced is a loud typed SKIP (panic), never a silent pass.
//!
//! No fixed coordination sleeps: every child wait is the one bounded poll
//! (50 ms step, deadline-checked, loud kill on overrun) —
//! SLEEP-CLASS: POLL-STEP.

use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

const CAP: usize = 16 * 1024 * 1024; // mirrors graphlaw::abi::MAX_REQUEST_BYTES
const POLL_STEP: Duration = Duration::from_millis(50);
/// Stub-module legs: the fixture loads in milliseconds; the deadline exists
/// to turn a hang into a loud failure.
const STUB_DEADLINE: Duration = Duration::from_secs(90);
/// Phase-1 window of the oversize legs: while stdin is OPEN and undrained, a
/// bound-checking runner has everything it needs (4 prefix bytes) and must
/// exit inside this window. Only after it lapses do we close stdin to let a
/// body-reading (unbound) runner reach its EOF-mid-frame outcome for the pin.
const OPEN_STDIN_WINDOW: Duration = Duration::from_secs(15);
const BIN_BUILD_DEADLINE: Duration = Duration::from_secs(1200);

// ------------------------------------------------------------- fixtures --

fn uleb32(mut v: u32, out: &mut Vec<u8>) {
    loop {
        let mut b = (v & 0x7f) as u8;
        v >>= 7;
        if v != 0 {
            b |= 0x80;
        }
        out.push(b);
        if v == 0 {
            break;
        }
    }
}

fn sleb64(mut v: i64, out: &mut Vec<u8>) {
    loop {
        let byte = (v & 0x7f) as u8;
        v >>= 7;
        let done = (v == 0 && byte & 0x40 == 0) || (v == -1 && byte & 0x40 != 0);
        out.push(if done { byte } else { byte | 0x80 });
        if done {
            break;
        }
    }
}

fn section(id: u8, payload: &[u8]) -> Vec<u8> {
    let mut s = vec![id];
    uleb32(payload.len() as u32, &mut s);
    s.extend_from_slice(payload);
    s
}

/// Hand-built wasm module: exports `memory` (1024 pages = 64 MiB, the wall),
/// `gl_alloc` (returns a fixed 32 MiB pointer, size-independent),
/// `gl_free` (nop), `gl_call` (returns `(ptr,len)` packed u64 pointing at a
/// data-segment response). No imports, no start section.
fn stub_module() -> Vec<u8> {
    const RESP: &[u8] = br#"{"ok":true,"stub":true}"#;
    const BODY_PTR: i64 = 32 * 1024 * 1024; // gl_alloc's fixed body pointer
    const RESP_PTR: i64 = 1024 * 1024; // data-segment base
    let packed = (RESP_PTR << 32) | RESP.len() as i64;

    let mut out = vec![0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00];
    // Type section (1): (i32)->i32, (i32,i32)->(), (i32,i32)->i64.
    let mut t = Vec::new();
    uleb32(3, &mut t);
    t.extend([0x60, 0x01, 0x7f, 0x01, 0x7f]);
    t.extend([0x60, 0x02, 0x7f, 0x7f, 0x00]);
    t.extend([0x60, 0x02, 0x7f, 0x7f, 0x01, 0x7e]);
    out.extend(section(1, &t));
    // Function section (3): type indices 0,1,2.
    out.extend(section(3, &[0x03, 0x00, 0x01, 0x02]));
    // Memory section (5): one memory, min 1024 pages.
    let mut m = Vec::new();
    uleb32(1, &mut m);
    m.push(0x00);
    uleb32(1024, &mut m);
    out.extend(section(5, &m));
    // Export section (7).
    let name = |n: &str, out: &mut Vec<u8>| {
        uleb32(n.len() as u32, out);
        out.extend_from_slice(n.as_bytes());
    };
    let mut e = Vec::new();
    uleb32(4, &mut e);
    name("memory", &mut e);
    e.push(0x02);
    uleb32(0, &mut e);
    name("gl_alloc", &mut e);
    e.push(0x00);
    uleb32(0, &mut e);
    name("gl_free", &mut e);
    e.push(0x00);
    uleb32(1, &mut e);
    name("gl_call", &mut e);
    e.push(0x00);
    uleb32(2, &mut e);
    out.extend(section(7, &e));
    // Code section (10).
    let mut f0 = Vec::new();
    uleb32(0, &mut f0); // no locals
    f0.push(0x41); // i32.const
    sleb64(BODY_PTR, &mut f0);
    f0.push(0x0b); // end
    let mut f1 = Vec::new();
    uleb32(0, &mut f1);
    f1.push(0x0b); // end (nop)
    let mut f2 = Vec::new();
    uleb32(0, &mut f2);
    f2.push(0x42); // i64.const
    sleb64(packed, &mut f2);
    f2.push(0x0b); // end
    let code = |body: &[u8], out: &mut Vec<u8>| {
        uleb32(body.len() as u32, out);
        out.extend_from_slice(body);
    };
    let mut c = Vec::new();
    uleb32(3, &mut c);
    code(&f0, &mut c);
    code(&f1, &mut c);
    code(&f2, &mut c);
    out.extend(section(10, &c));
    // Data section (11): active segment at RESP_PTR.
    let mut d = Vec::new();
    uleb32(1, &mut d);
    d.push(0x00); // active, memory 0
    d.push(0x41); // i32.const offset
    sleb64(RESP_PTR, &mut d);
    d.push(0x0b);
    uleb32(RESP.len() as u32, &mut d);
    d.extend_from_slice(RESP);
    out.extend(section(11, &d));
    out
}

fn write_stub() -> PathBuf {
    let dir = std::env::temp_dir().join("graphlaw-frame-bound-court");
    std::fs::create_dir_all(&dir).expect("create court fixture dir");
    let path = dir.join("gl_stub.wasm");
    std::fs::write(&path, stub_module()).expect("write stub module");
    path
}

// -------------------------------------------------- binary resolution ----

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

fn find_bin(name: &str) -> Option<PathBuf> {
    if let Some(p) = std::env::var_os(bin_env(name)) {
        let p = PathBuf::from(p);
        return p.exists().then_some(p);
    }
    let root = lane_target_root();
    let candidates = [
        root.join("debug").join(name),
        manifest().join("target/debug").join(name),
    ];
    candidates.into_iter().find(|p| p.exists())
}

fn bin_env(name: &str) -> String {
    if name == "graphlaw-run-eh" {
        "GRAPHLAW_RUN_EH".into()
    } else {
        "GRAPHLAW_RUN".into()
    }
}

/// Build a runner binary into the lane target root if it is missing
/// (`cargo build --bin <name> --features <feature>`), then re-resolve. The
/// inner build shares the outer target dir — cargo's per-target-dir flock
/// serializes it, the tests/common/mod.rs precedent. Inherit stdio so the
/// operator watches the build. Bounded, loud on failure.
fn ensure_bin(name: &str, feature: &str) -> PathBuf {
    if let Some(p) = find_bin(name) {
        return p;
    }
    eprintln!("FRAME-COURT: {name} not found; building (--features {feature})");
    let cargo = std::env::var("CARGO").unwrap_or_else(|_| "cargo".into());
    let mut child = Command::new(&cargo)
        .current_dir(manifest())
        .args(["build", "--bin", name, "--features", feature])
        .stdin(Stdio::null())
        .spawn()
        .expect("spawn cargo build for runner binary");
    let deadline = Instant::now() + BIN_BUILD_DEADLINE;
    loop {
        match child.try_wait().expect("poll cargo build") {
            Some(status) => {
                if !status.success() {
                    panic!(
                        "FRAME-COURT BUILD FAILED: `cargo build --bin {name} --features {feature}` \
                         exited {status}"
                    );
                }
                break;
            }
            None => {
                if Instant::now() > deadline {
                    let _ = child.kill();
                    let _ = child.wait();
                    panic!(
                        "FRAME-COURT BUILD TIMEOUT: cargo build --bin {name} did not finish in {}s",
                        BIN_BUILD_DEADLINE.as_secs()
                    );
                }
                std::thread::sleep(POLL_STEP); // SLEEP-CLASS: POLL-STEP
            }
        }
    }
    find_bin(name).unwrap_or_else(|| {
        panic!(
            "GRAPH LAW FRAME-COURT SKIP: {name} still unresolved after build \
             (set GRAPHLAW_RUN / GRAPHLAW_RUN_EH or the lane CARGO_TARGET_DIR)"
        )
    })
}

// ---------------------------------------------------------- run harness --

struct Outcome {
    code: i32,
    stdout: Vec<u8>,
    stderr: String,
}

enum StdinPlan {
    /// Feed bytes, then hold the pipe OPEN for `window` while polling. If the
    /// child has not exited by then, close stdin (lets an unbound runner hit
    /// its EOF-mid-frame outcome) and keep polling to the deadline.
    /// `open_exit` reports whether the child exited while stdin was open.
    Padded {
        bytes: Vec<u8>,
        window: Duration,
    },
    /// Feed bytes, close stdin immediately, poll to the deadline.
    Close(Vec<u8>),
}

fn run_frame(bin: &Path, artifact: &Path, plan: StdinPlan, deadline: Duration) -> Outcome {
    let mut child = Command::new(bin)
        .arg("--wasm")
        .arg(artifact)
        .env_remove("GRAPHLAW_WASM") // pin the artifact via --wasm only
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn runner");
    let mut si = child.stdin.take();
    if let Some(handle) = si.as_mut() {
        match &plan {
            StdinPlan::Padded { bytes, .. } | StdinPlan::Close(bytes) => {
                handle
                    .write_all(bytes)
                    .expect("write frame bytes (child consumes as we write)");
            }
        }
        if matches!(plan, StdinPlan::Close(_)) {
            drop(si.take()); // EOF at the frame boundary
        }
        // Padded: the handle stays in `si` — the pipe stays OPEN.
    }
    let child_stdout = child.stdout.take();
    let child_stderr = child.stderr.take();
    // Drain both pipes CONCURRENTLY with the wait: a refusal path that emits
    // a large diagnostic (e.g. a per-frame guest backtrace) can saturate the
    // 64 KiB pipe buffer; a saturation deadlock would masquerade as a hang.
    let stdout_handle = std::thread::spawn(move || {
        let mut buf = Vec::new();
        if let Some(mut o) = child_stdout {
            use std::io::Read;
            let _ = o.read_to_end(&mut buf);
        }
        buf
    });
    let stderr_handle = std::thread::spawn(move || {
        let mut s = String::new();
        if let Some(mut e) = child_stderr {
            use std::io::Read;
            let _ = e.read_to_string(&mut s);
        }
        s
    });
    let start = Instant::now();
    let status = loop {
        match child.try_wait().expect("poll runner") {
            Some(s) => break s,
            None => {
                if let StdinPlan::Padded { window, .. } = &plan {
                    if si.is_some() && start.elapsed() > *window {
                        // Phase 1 lapsed: the runner did NOT exit with stdin
                        // open — release stdin and observe what it does at
                        // EOF (a bound-checking runner never reaches here).
                        drop(si.take());
                    }
                }
                if start.elapsed() > deadline {
                    let _ = child.kill();
                    let _ = child.wait();
                    panic!(
                        "FRAME-COURT TIMEOUT: runner {bin:?} did not exit within {}s — \
                         hang is a failure, not a pass",
                        deadline.as_secs()
                    );
                }
                std::thread::sleep(POLL_STEP); // SLEEP-CLASS: POLL-STEP
            }
        }
    };
    let stdout = stdout_handle.join().expect("stdout reader thread");
    let stderr = stderr_handle.join().expect("stderr reader thread");
    Outcome {
        code: status.code().unwrap_or_else(|| panic!("no exit code: {status}")),
        stdout,
        stderr,
    }
}

/// Wraps `body` in a packet-4 frame (4-byte BE length prefix).
fn frame(body: &[u8]) -> Vec<u8> {
    let mut f = Vec::with_capacity(4 + body.len());
    f.extend_from_slice(&(body.len() as u32).to_be_bytes());
    f.extend_from_slice(body);
    f
}

/// Asserts the reply envelope on stdout: one packet-4 frame whose body is
/// JSON with `ok == true`.
fn assert_reply(stdout: &[u8], context: &str) {
    assert!(
        stdout.len() >= 4,
        "{context}: no reply frame on stdout ({} bytes)",
        stdout.len()
    );
    let len = u32::from_be_bytes([stdout[0], stdout[1], stdout[2], stdout[3]]) as usize;
    assert!(
        stdout.len() >= 4 + len,
        "{context}: reply frame truncated (claims {len}, have {})",
        stdout.len() - 4
    );
    let v: serde_json::Value = serde_json::from_slice(&stdout[4..4 + len])
        .unwrap_or_else(|e| panic!("{context}: reply is not JSON: {e}"));
    assert_eq!(
        v.get("ok").and_then(|o| o.as_bool()),
        Some(true),
        "{context}: reply envelope must be ok:true — {v}"
    );
}

// ---------------------------------------------------------------- courts --

// ---- graphlaw-run-eh (wasmtime host; src/bin/graphlaw_run_eh.rs — spec side).
// The graphlaw-run (wasmi host) legs were retired with the runner on
// 2026-10-24 per docs/jira/v26.10.23/ADR-wasmi-retirement.md; these EH twins
// carry the same frame-bound contract.

/// EH twin of the zero-length leg.
#[test]
fn eh_zero_len_frame_clean_exit0() {
    let bin = ensure_bin("graphlaw-run-eh", "runner-eh");
    let stub = write_stub();
    let out = run_frame(&bin, &stub, StdinPlan::Close(frame(&[])), STUB_DEADLINE);
    assert_eq!(out.code, 0, "stderr: {}", out.stderr);
    assert_reply(&out.stdout, "EH zero-length frame");
}

/// EH twin of the exactly-at-cap leg.
#[test]
fn eh_at_cap_frame_admitted_and_answered() {
    let bin = ensure_bin("graphlaw-run-eh", "runner-eh");
    let stub = write_stub();
    let mut body = br#"{"op":"capabilities"}"#.to_vec();
    body.resize(CAP, b' ');
    assert_eq!(body.len(), CAP);
    let out = run_frame(&bin, &stub, StdinPlan::Close(frame(&body)), STUB_DEADLINE);
    assert_eq!(out.code, 0, "stderr: {}", out.stderr);
    assert_reply(&out.stdout, "EH at-cap frame");
}

/// EH twin of the 0xFFFFFFFF leg, in the repo's pending-pin shape: when the
/// bound is landed the leg asserts the full contract (exit 92 + length + cap
/// + no reply); while `src/bin/graphlaw_run_eh.rs` still allocates before the
/// check, the leg pins the observed pending state (exit 90 after stdin close)
/// LOUDLY — a pin is not a pass of the spec, it is the ledger entry that
/// flips green the moment the EH mirror lands.
#[test]
fn eh_oversize_0xffffffff_exit92_before_body() {
    let bin = ensure_bin("graphlaw-run-eh", "runner-eh");
    let stub = write_stub();
    let mut bytes = 0xFFFFFFFFu32.to_be_bytes().to_vec();
    bytes.extend(std::iter::repeat(0x20u8).take(4096));
    let out = run_frame(
        &bin,
        &stub,
        StdinPlan::Padded {
            bytes,
            window: OPEN_STDIN_WINDOW,
        },
        STUB_DEADLINE,
    );
    if out.code == 92 {
        assert!(
            out.stderr.contains("frame length 4294967295")
                && out.stderr.contains("exceeds cap 16777216"),
            "stderr must name length and cap: {}",
            out.stderr
        );
        assert!(
            out.stdout.is_empty(),
            "no reply frame may precede the oversize abort"
        );
        return;
    }
    // Pending pin: the EH host at HEAD allocates the 4 GiB body, blocks on
    // the pipe, and reaches EOF-mid-frame (90) only once stdin closes.
    assert_eq!(
        out.code, 90,
        "expected the landed contract (92) or the pinned pending state (90); \
         stderr: {} — if this is another code the EH frame layer changed shape",
        out.stderr
    );
    assert!(
        out.stderr.contains("EOF mid-frame"),
        "pending pin must observe the pre-mirror mechanism: {}",
        out.stderr
    );
    eprintln!(
        "EH FRAME-BOUND PENDING PIN: graphlaw_run_eh.rs answered 90 (EOF mid-frame) — \
         the R3 mirror (exit 92, refuse-before-allocate) is not landed on the EH host; \
         this leg flips to the strict 92 asserts when it lands"
    );
}

/// EH twin of the cap+1 boundary leg, same pending-pin shape.
#[test]
fn eh_over_cap_by_one_exit92() {
    let bin = ensure_bin("graphlaw-run-eh", "runner-eh");
    let stub = write_stub();
    let mut bytes = ((CAP + 1) as u32).to_be_bytes().to_vec();
    bytes.extend(std::iter::repeat(0x20u8).take(4096));
    let out = run_frame(
        &bin,
        &stub,
        StdinPlan::Padded {
            bytes,
            window: OPEN_STDIN_WINDOW,
        },
        STUB_DEADLINE,
    );
    if out.code == 92 {
        assert!(
            out.stderr.contains("frame length 16777217")
                && out.stderr.contains("exceeds cap 16777216"),
            "stderr must name length and cap: {}",
            out.stderr
        );
        return;
    }
    assert_eq!(
        out.code, 90,
        "expected 92 (landed) or 90 (pinned pending); stderr: {}",
        out.stderr
    );
    eprintln!(
        "EH FRAME-BOUND PENDING PIN: cap+1 answered 90 — EH mirror not landed"
    );
}
