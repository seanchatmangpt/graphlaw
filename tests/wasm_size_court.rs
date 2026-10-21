//! W04 — WASM artifact size ceiling + header court (native companion to
//! scripts/wasm_audit.sh). Honors GRAPHLAW_WASM; when the artifact is absent
//! the skip is named and counted, never silent (same discipline as
//! wasi_patch_guard.rs).

use std::path::PathBuf;
use std::process::Command;

const MAX_BYTES: u64 = 26_214_400; // 25 MiB
const WASM_MAGIC: [u8; 4] = [0x00, b'a', b's', b'm'];

fn subject() -> Option<PathBuf> {
    if let Ok(p) = std::env::var("GRAPHLAW_WASM") {
        let p = PathBuf::from(p);
        return if p.is_file() { Some(p) } else { None };
    }
    let p = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("target/wasm32-wasip1/wasm/graphlaw_wasm.wasm");
    if p.is_file() {
        Some(p)
    } else {
        None
    }
}

fn script_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("scripts/wasm_audit.sh")
}

/// The court: the artifact exists, is a real wasm module, and fits the ceiling.
#[test]
fn wasm_artifact_within_size_ceiling_and_valid_header() {
    let Some(path) = subject() else {
        println!(
            "NOTICE [wasm-size-court-skip]: no wasm artifact found (set GRAPHLAW_WASM or build \
             with `cargo build -p graphlaw-wasm --target wasm32-wasip1 --release`); the size \
             court is NOT covered by this run."
        );
        return;
    };

    let bytes = std::fs::read(&path)
        .unwrap_or_else(|e| panic!("WASM_SIZE_COURT: cannot read {}: {e}", path.display()));
    let len = bytes.len() as u64;
    assert!(
        bytes.starts_with(&WASM_MAGIC),
        "WASM_SIZE_COURT: {} lacks \\0asm header magic — not a wasm module",
        path.display()
    );
    assert!(
        len <= MAX_BYTES,
        "WASM_SIZE_COURT: {} is {} bytes, exceeding the {}-byte (25 MiB) ceiling by {} bytes. \
         Slim the artifact (features/opt-levels/strip) before landing.",
        path.display(),
        len,
        MAX_BYTES,
        len - MAX_BYTES
    );
    println!(
        "NOTICE [wasm-size-verified]: {} = {} bytes <= {} ceiling; header \\0asm valid.",
        path.display(),
        len,
        MAX_BYTES
    );
}

/// Non-vacuity: the SCRIPT court must actually refuse a fat fixture. A padded
/// file over the ceiling MUST produce exit 1; a court that accepts it is vacuous.
#[test]
fn wasm_audit_script_refuses_fat_fixture() {
    let tmp = std::env::temp_dir().join("graphlaw_wasm_audit_fatfixture.bin");
    std::fs::write(&tmp, vec![0u8; MAX_BYTES as usize + 1])
        .expect("write fat fixture");
    let out = Command::new("bash")
        .arg(script_path())
        .env("WASM_AUDIT_FATFIXTURE", &tmp)
        .output()
        .expect("run wasm_audit.sh");
    let _ = std::fs::remove_file(&tmp);
    assert_eq!(
        out.status.code(),
        Some(1),
        "WASM_SIZE_COURT_VACUOUS: wasm_audit.sh accepted a fixture over the {}-byte ceiling \
         (exit {:?}); the court cannot detect a breach. stderr: {}",
        MAX_BYTES,
        out.status.code(),
        String::from_utf8_lossy(&out.stderr)
    );
    println!(
        "NOTICE [wasm-court-non-vacuous]: script exit 1 on fat fixture over {}-byte ceiling.",
        MAX_BYTES
    );
}

/// Non-vacuity: missing subject must be a loud exit 3, not a silent pass.
#[test]
fn wasm_audit_script_loud_missing_subject() {
    let out = Command::new("bash")
        .arg(script_path())
        .env("WASM_AUDIT_FATFIXTURE", "/nonexistent/graphlaw.wasm")
        .output()
        .expect("run wasm_audit.sh");
    assert_eq!(
        out.status.code(),
        Some(3),
        "WASM_SIZE_COURT_VACUOUS: missing subject did not exit 3 (got {:?})",
        out.status.code()
    );
    assert!(
        !out.stderr.is_empty(),
        "WASM_SIZE_COURT_VACUOUS: missing-subject refusal must be loud (stderr)"
    );
}
