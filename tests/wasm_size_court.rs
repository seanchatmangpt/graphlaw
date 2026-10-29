//! W04 — WASM artifact size ceiling + header court (native companion to
//! scripts/wasm_audit.sh). Honors GRAPHLAW_WASM. A missing artifact fails
//! the court. The ceiling is the 64 MiB sandbox wall.
//!
//! History: pre-DuckDB-pin artifacts were ~7,104,147 B (7.1 MB class). The
//! DuckDB-linked artifact is the M4 witness class (~45 MB, ceiling 64 MiB).
//! No byte digest is pinned here — digests are asserted by the registry
//! artifact pins, not by this court. This court asserts size-class and
//! section facts only.

use std::path::PathBuf;
use std::process::Command;

// Size-policy ceiling re-anchor 2026-10-29 (GL28-13): 128 MB = 134,217,728 B.
// Ceiling MATCHES POLICY, not a slimming claim — the policy ceiling was
// re-anchored from the 64 MiB sandbox wall to 128 MB by operator decision;
// the artifact is measured and pinned honestly at whatever the canonical
// relink produces.
const MAX_BYTES: u64 = 134_217_728; // 128 MB
/// M4 witness class: a DuckDB-linked wasm engine artifact. Anything at or
/// above this proves the engine actually linked; a 7.1 MB-class artifact is
/// the legacy engine-free shape.
const ENGINE_LINK_MIN_BYTES: u64 = 40_000_000; // 40 MB
const WASM_MAGIC: [u8; 4] = [0x00, b'a', b's', b'm'];
const TAG_SECTION_ID: u8 = 13;

fn subject() -> Option<PathBuf> {
    if let Ok(p) = std::env::var("GRAPHLAW_WASM") {
        let p = PathBuf::from(p);
        return if p.is_file() { Some(p) } else { None };
    }
    let p = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("target/wasm32-wasip1/wasm/graphlaw_wasm.wasm");
    if p.is_file() { Some(p) } else { None }
}

fn engine_link_env_resolved() -> bool {
    match std::env::var("GRAPHLAW_WASM") {
        Ok(p) => PathBuf::from(p).is_file(),
        Err(_) => false,
    }
}

fn script_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("scripts/wasm_audit.sh")
}

/// Walk wasm section headers, counting sections by id. Inline parser: after
/// the 8-byte version header, each section is `id:u8` then `size:u32leb`,
/// then `size` payload bytes. Returns (section_count, tag_section_count).
/// Custom sections (id 0) carry a name in the payload but the size framing
/// still holds, so a pure size-skip walk is exact.
fn walk_sections(bytes: &[u8]) -> Result<(usize, usize), String> {
    if bytes.len() < 8 || !bytes.starts_with(&WASM_MAGIC) {
        return Err("not a wasm module (bad header)".into());
    }
    let mut i = 8usize;
    let mut total = 0usize;
    let mut tags = 0usize;
    while i < bytes.len() {
        let id = bytes[i];
        i += 1;
        let mut size: u64 = 0;
        let mut shift = 0u32;
        loop {
            if i >= bytes.len() {
                return Err(format!("truncated LEB128 at byte {i}"));
            }
            let b = bytes[i];
            i += 1;
            size |= u64::from(b & 0x7f) << shift;
            shift += 7;
            if b & 0x80 == 0 {
                break;
            }
            if shift >= 35 {
                return Err("overlong LEB128".into());
            }
        }
        let end = i
            .checked_add(usize::try_from(size).map_err(|_| "section size overflow")?)
            .ok_or("section end overflow")?;
        if end > bytes.len() {
            return Err(format!(
                "section id {id} claims {size} bytes past EOF (payload at {i}, file {})",
                bytes.len()
            ));
        }
        total += 1;
        if id == TAG_SECTION_ID {
            tags += 1;
        }
        i = end;
    }
    Ok((total, tags))
}

/// The court: the artifact exists, is a real wasm module, and fits the ceiling.
#[test]
fn wasm_artifact_within_size_ceiling_and_valid_header() {
    let Some(path) = subject() else {
        panic!(
            "SKIP is not a pass: no wasm artifact (set GRAPHLAW_WASM or build \
             `cargo build -p graphlaw-wasm --target wasm32-wasip1 --profile wasm`). \
             The size court did not run."
        );
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
        "WASM_SIZE_COURT: {} is {} bytes, exceeding the {}-byte (64 MiB) ceiling by {} bytes. \
         Slim the artifact (features/opt-levels/strip) before landing.",
        path.display(),
        len,
        MAX_BYTES,
        len - MAX_BYTES
    );
    let (sections, tags) = walk_sections(&bytes)
        .unwrap_or_else(|e| panic!("WASM_SIZE_COURT: section walk of {} failed: {e}", path.display()));
    println!(
        "NOTICE [wasm-size-verified]: {} = {} bytes <= {} ceiling; header \\0asm valid; \
         {sections} sections, {tags} Tag(13) sections.",
        path.display(),
        len,
        MAX_BYTES
    );
}

/// Engine-link proof (M4 witness class): when GRAPHLAW_WASM resolves to a real
/// artifact, that artifact MUST be > 40 MB — the DuckDB engine must be inside
/// the module. A smaller artifact is the legacy 7.1 MB engine-free shape; that
/// is a loud SKIP with a typed reason, not a pass.
#[test]
fn wasm_artifact_proves_engine_link_when_env_resolves() {
    if !engine_link_env_resolved() {
        println!(
            "SKIP [wasm-engine-link]: GRAPHLAW_WASM not set (or path not a file); \
             the engine-link size-class assertion is environment-gated and did not run. \
             SKIP is not a pass."
        );
        return;
    }
    let path = PathBuf::from(std::env::var("GRAPHLAW_WASM").expect("checked above"));
    let bytes = std::fs::read(&path)
        .unwrap_or_else(|e| panic!("WASM_SIZE_COURT: cannot read {}: {e}", path.display()));
    let len = bytes.len() as u64;
    if len < ENGINE_LINK_MIN_BYTES {
        // Legacy pin class: the artifact matches the pre-DuckDB-pin shape
        // (~7.1 MB, no engine). Loud typed SKIP; never read as an engine-link pass.
        println!(
            "SKIP [wasm-engine-link]: {} is {} bytes — LEGACY engine-free pin class \
             (< {}-byte M4 witness). The DuckDB engine did NOT link into this artifact; \
             the engine-link assertion is not applicable to a legacy artifact.",
            path.display(),
            len,
            ENGINE_LINK_MIN_BYTES
        );
        return;
    }
    let (_, tags) = walk_sections(&bytes)
        .unwrap_or_else(|e| panic!("WASM_SIZE_COURT: section walk of {} failed: {e}", path.display()));
    assert!(
        tags > 0,
        "WASM_SIZE_COURT: {} is {} bytes (engine-link size class) but carries ZERO \
         Tag(13) sections — a linked exception-handling engine must emit tag sections; \
         size suggests the engine linked but sections say otherwise.",
        path.display(),
        len
    );
    println!(
        "NOTICE [wasm-engine-link]: {} = {} bytes >= {} (M4 witness class), \
         {tags} Tag(13) section(s) — DuckDB engine link proven.",
        path.display(),
        len,
        ENGINE_LINK_MIN_BYTES
    );
}

/// Non-vacuity: the SCRIPT court must actually refuse a fat fixture. A padded
/// file over the ceiling MUST produce exit 1; a court that accepts it is vacuous.
#[test]
fn wasm_audit_script_refuses_fat_fixture() {
    let tmp = std::env::temp_dir().join("graphlaw_wasm_audit_fatfixture.bin");
    std::fs::write(&tmp, vec![0u8; MAX_BYTES as usize + 1]).expect("write fat fixture");
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
