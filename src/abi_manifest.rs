//! Canonical ABI manifest: one machine-readable JSON document describing the
//! wasm FFI surface (`gl_alloc`/`gl_free`/`gl_call`), the request/response
//! limits, the 14-op op table, the response envelope shape, and the
//! 128-byte `OcelCausalFrame` layout — digested with BLAKE3 exactly like
//! `registry_blake3` (canonical bytes of the document without the digest key).
//!
//! The committed artifact is `contracts/abi-manifest.json`; the court in
//! `tests/abi_manifest_court.rs` regenerates it in-process and byte-compares.
//! A divergence with an unchanged [`crate::abi::ABI_VERSION`] is a contract
//! drift: either the change is additive (op-list superset only, classified
//! non-breaking per the `op_forward_compat` precedent) or `ABI_VERSION`
//! must be bumped.
#![cfg(feature = "abi")]

use serde_json::{Value, json};

use crate::abi::{
    ABI_VERSION, MAX_ATOMS_PER_FIELD, MAX_DATALOG_FACTS, MAX_DATALOG_RULES, MAX_JSON_DEPTH,
    MAX_LAW_STEPS, MAX_OUTSTANDING_ALLOC_BYTES, MAX_PLAN_ACTIONS, MAX_POLICY_ENTRIES,
    MAX_REQUEST_BYTES,
};
use crate::law::{
    DEFAULT_MAX_SKEW_SECS, N3_MAX_DERIVED_FACTS, N3_MAX_ITERATIONS, N3_MAX_TERM_BYTES,
    N3_MAX_TOTAL_BYTES, N3_MAX_TOTAL_STEPS,
};

/// One export of the wasm FFI surface, with its wasm-type signature.
pub struct Export {
    /// Exported symbol name (`gl_` prefix, per `wasm/src/ffi.rs`).
    pub name: &'static str,
    /// wasm parameter types.
    pub params: &'static [&'static str],
    /// wasm return type.
    pub result: &'static str,
    /// Contract summary.
    pub summary: &'static str,
}

/// The FFI exports, in call order (`wasm/src/ffi.rs`).
pub const EXPORTS: &[Export] = &[
    Export {
        name: "gl_alloc",
        params: &["i32"],
        result: "i32",
        summary: "reserve `len` bytes; returns ptr, or null over the request/outstanding limits",
    },
    Export {
        name: "gl_free",
        params: &["i32", "i32"],
        result: "()",
        summary: "release a buffer previously handed out by gl_alloc or gl_call",
    },
    Export {
        name: "gl_call",
        params: &["i32", "i32"],
        result: "i64",
        summary: "run one request; consumes the buffer; returns (out_ptr << 32) | out_len",
    },
];

/// One row of the 99-byte `to_hash_bytes` field table.
struct FrameField {
    offset: usize,
    len: usize,
    name: &'static str,
    ty: &'static str,
}

/// `OcelCausalFrame::to_hash_bytes` layout (`src/receipt_chain.rs`), LE.
const FRAME_FIELDS: &[FrameField] = &[
    FrameField { offset: 0, len: 8, name: "instruction_id", ty: "u64-le" },
    FrameField { offset: 8, len: 8, name: "fired_mask", ty: "u64-le" },
    FrameField { offset: 16, len: 8, name: "denial.0", ty: "u64-le" },
    FrameField { offset: 24, len: 32, name: "obj_refs", ty: "8x-u32-le" },
    FrameField { offset: 56, len: 8, name: "ts_ns", ty: "u64-le" },
    FrameField { offset: 64, len: 2, name: "activity_idx", ty: "u16-le" },
    FrameField { offset: 66, len: 1, name: "node_kind", ty: "u8" },
    FrameField { offset: 67, len: 32, name: "prior_hash", ty: "32x-u8-verbatim" },
];

/// `blake3:<hex>` of the canonical manifest bytes (digest key removed),
/// mirroring `registry_blake3`.
fn blake3_tagged(bytes: &[u8]) -> String {
    let digest = blake3::hash(bytes);
    use std::fmt::Write as _;
    let mut out = String::with_capacity(7 + 64);
    out.push_str("blake3:");
    for b in digest.as_bytes() {
        let _ = write!(out, "{b:02x}");
    }
    out
}

/// The manifest document without `manifest_blake3`.
fn base_manifest() -> Value {
    let ops: Vec<Value> = crate::registry::op_names()
        .into_iter()
        .enumerate()
        .map(|(i, name)| json!({"name": name, "order": i + 1}))
        .collect();
    let exports: Vec<Value> = EXPORTS
        .iter()
        .map(|e| json!({
            "name": e.name,
            "params": e.params,
            "result": e.result,
            "summary": e.summary,
        }))
        .collect();
    let frame_fields: Vec<Value> = FRAME_FIELDS
        .iter()
        .map(|f| json!({"name": f.name, "offset": f.offset, "len": f.len, "ty": f.ty}))
        .collect();
    json!({
        "schema": "graphlaw.abi-manifest/1",
        "graphlaw_version": env!("CARGO_PKG_VERSION"),
        "abi_version": ABI_VERSION,
        "exports": exports,
        "limits": {
            "max_request_bytes": MAX_REQUEST_BYTES,
            "max_json_depth": MAX_JSON_DEPTH,
            "max_plan_actions": MAX_PLAN_ACTIONS,
            "max_atoms_per_field": MAX_ATOMS_PER_FIELD,
            "max_policy_entries": MAX_POLICY_ENTRIES,
            "max_outstanding_alloc_bytes": MAX_OUTSTANDING_ALLOC_BYTES,
            "max_law_steps": MAX_LAW_STEPS,
            "max_datalog_rules": MAX_DATALOG_RULES,
            "max_datalog_facts": MAX_DATALOG_FACTS,
            "n3_max_iterations": N3_MAX_ITERATIONS,
            "n3_max_term_bytes": N3_MAX_TERM_BYTES,
            "n3_max_total_bytes": N3_MAX_TOTAL_BYTES,
            "n3_max_derived_facts": N3_MAX_DERIVED_FACTS,
            "n3_max_total_steps": N3_MAX_TOTAL_STEPS,
            "default_max_skew_secs": DEFAULT_MAX_SKEW_SECS,
        },
        "ops": ops,
        "notes": "op `duckdb` (op-15) is dispatched by the wasm module (`src/abi.rs` routes it to `duckdb_op::dispatch`). It stays off this hashed 14-op table so `registry_blake3` does not move until an explicit digest bump. Absence from `capabilities` is not a SQL refusal and is not REFUSAL-04. SQL-in-RDF admits inside the module through `compile_turtle`. frozen-duckdb 2.0.2 emits `static=duckdb` for wasm32-wasip1. The wasm32-wasip1 link is witnessed (2026-10-22); the recorded pin `332ea218…` was not replaced. `WASI_SDK_PATH` must be a wasi-sdk root (`bin/clang`, `share/cmake/wasi-sdk-p1.cmake`). Without it, frozen-duckdb-sys `build.rs` fails typed (`WASI_SDK_PATH required`). Bounds enforced in src/duckdb_op.rs: DuckDB memory_limit=32MiB and threads=1 compile-time pragmas (GAP-2), chunk rows <= 10000 and chunk bytes <= 16 MiB typed ResourceLimit refusals (GAP-4), MAX_OUTSTANDING_ALLOC_BYTES is 16 MiB for the wasm32 target via split-config (GAP-1)",
        "response_envelope": {
            "success": {"ok": true, "extra": "op-specific fields"},
            "refusal": {"ok": false, "error": {
                "kind": "string", "engine": "string|null",
                "dialect": "string|null", "message": "string",
                "details": "object|absent",
            }},
        },
        "frame": {
            "struct": "OcelCausalFrame",
            "size_bytes": 128,
            "align_bytes": 64,
            "hash_bytes_len": 99,
            "hash_input": "to_hash_bytes() little-endian field table",
            "fields": frame_fields,
            "prior_hash_semantics": "BLAKE3 of the preceding frame's to_hash_bytes; genesis is BLAKE3 of 32 zero bytes",
        },
    })
}

/// The complete manifest document, including `manifest_blake3`.
#[must_use]
pub fn manifest_value() -> Value {
    let mut doc = base_manifest();
    let digest = blake3_tagged(crate::registry::canonical(&doc).as_bytes());
    if let Some(o) = doc.as_object_mut() {
        o.insert("manifest_blake3".into(), json!(digest));
    }
    doc
}

/// The committed `contracts/abi-manifest.json` bytes: sorted keys,
/// 2-space indent, LF, trailing newline.
#[must_use]
pub fn manifest_json_pretty() -> String {
    let mut out = serde_json::to_string_pretty(&manifest_value()).expect("manifest is JSON");
    out.push('\n');
    out
}

/// Write `contracts/abi-manifest.json` relative to the crate root.
///
/// # Errors
/// Propagates filesystem errors.
pub fn write_manifest(path: &std::path::Path) -> std::io::Result<()> {
    // Atomic replace: the abi_manifest_court runs `write_manifest` concurrently
    // with `manifest_is_byte_identical_or_classified_additive` reading the same
    // committed file; an in-place truncate raced a reader into
    // "EOF while parsing a value" (witnessed under full-suite parallel load).
    // Write-then-rename never exposes a truncated file.
    let tmp = path.with_extension("json.tmp");
    std::fs::write(&tmp, manifest_json_pretty())?;
    std::fs::rename(&tmp, path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn digest_is_stable_and_self_verifying() {
        let doc = manifest_value();
        let digest = doc["manifest_blake3"].as_str().expect("digest").to_owned();
        let mut base = doc.clone();
        base.as_object_mut().unwrap().remove("manifest_blake3");
        assert_eq!(digest, blake3_tagged(crate::registry::canonical(&base).as_bytes()));
    }

    #[test]
    fn frame_field_table_covers_99_bytes_exactly() {
        let mut expected = 0;
        for f in FRAME_FIELDS {
            assert_eq!(f.offset, expected, "field {} not contiguous", f.name);
            expected += f.len;
        }
        assert_eq!(expected, 99);
    }
}
