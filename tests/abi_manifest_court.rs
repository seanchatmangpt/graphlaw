//! ABI-manifest court: the committed `contracts/abi-manifest.json` must be
//! byte-identical to what the library regenerates from live constants
//! (`abi::ABI_VERSION`, `registry::op_names`, `abi::{MAX_*}`, `law::{N3_*}`).
//!
//! Verdict rules:
//! - regenerated == committed: PASS.
//! - regenerated differs only by an op-list superset (plus the digest):
//!   classified NON-BREAKING (additive ops, `op_forward_compat` precedent:
//!   unknown top-level keys are ignored, new ops cannot affect old callers).
//! - any other divergence with an unchanged `abi::ABI_VERSION`: FAIL naming
//!   "bump ABI_VERSION".
//!
//! Also runs native golden vectors (one per op, plus null/zero-len/oversize/
//! depth-65 limit refusals) and the 99-byte `OcelCausalFrame::to_hash_bytes`
//! golden frame.
#![cfg(all(feature = "abi", not(target_arch = "wasm32")))]

use serde_json::{Value, json};

// ------------------------------------------------------------- manifest court

fn committed_path() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("contracts/abi-manifest.json")
}

/// Test-mode emitter: regenerate the committed artifact.
/// `cargo test -p graphlaw --test abi_manifest_court write_manifest -- --exact`
#[test]
fn write_manifest() {
    let path = committed_path();
    graphlaw::abi_manifest::write_manifest(&path).expect("write manifest");
    println!("wrote {}", path.display());
}

/// Breaking-field view: everything except the op list and the digest.
fn breaking_view(v: &Value) -> Value {
    let mut doc = v.clone();
    let o = doc.as_object_mut().expect("manifest object");
    o.remove("manifest_blake3");
    o.remove("ops");
    doc
}

#[test]
fn manifest_is_byte_identical_or_classified_additive() {
    let regenerated = graphlaw::abi_manifest::manifest_json_pretty();
    let committed =
        std::fs::read_to_string(committed_path()).expect("contracts/abi-manifest.json exists");

    if regenerated == committed {
        return;
    }
    // Divergence: classify before failing.
    let reg: Value = serde_json::from_str(&regenerated).expect("regenerated is JSON");
    let com: Value = serde_json::from_str(&committed).expect("committed is JSON");
    assert_eq!(
        com["abi_version"], reg["abi_version"],
        "abi_version changed in the committed file but the manifest was not regenerated"
    );
    let reg_ops: Vec<&str> = reg["ops"]
        .as_array()
        .expect("ops array")
        .iter()
        .map(|o| o["name"].as_str().expect("op name"))
        .collect();
    let com_ops: Vec<&str> = com["ops"]
        .as_array()
        .expect("ops array")
        .iter()
        .map(|o| o["name"].as_str().expect("op name"))
        .collect();
    let additive = breaking_view(&reg) == breaking_view(&com)
        && reg_ops.len() > com_ops.len()
        && reg_ops[..com_ops.len()] == com_ops[..];
    assert!(
        additive,
        "contracts/abi-manifest.json diverges from the live ABI in a breaking way \
         with unchanged ABI_VERSION {}: bump ABI_VERSION (and regenerate the manifest)",
        reg["abi_version"]
    );
    eprintln!(
        "NON-BREAKING: committed manifest op list is an additive subset \
         ({} -> {} ops); regenerate contracts/abi-manifest.json to refresh the digest",
        com_ops.len(),
        reg_ops.len()
    );
}

#[test]
fn manifest_blake3_matches_registry_style_digest() {
    let doc = graphlaw::abi_manifest::manifest_value();
    let digest = doc["manifest_blake3"].as_str().expect("digest present");
    assert!(digest.starts_with("blake3:"), "digest is blake3:<hex>");
}

// --------------------------------------------------------- native golden vectors

fn envelope(bytes: &[u8]) -> Value {
    serde_json::from_slice(bytes).expect("response is JSON")
}

/// One native vector per registry op, driven from the committed examples.
#[test]
fn one_native_vector_per_op_reaches_its_declared_outcome() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("registry/op-examples.json");
    let doc: Value = serde_json::from_str(&std::fs::read_to_string(&path).expect("read examples"))
        .expect("examples are JSON");
    assert_eq!(doc["schema"], "graphlaw.op-examples/1");
    let examples = doc["examples"].as_array().expect("examples array");
    let mut ops_seen = std::collections::BTreeSet::new();
    for ex in examples {
        let id = ex["id"].as_str().expect("id");
        let op = ex["op"].as_str().expect("op");
        let response = envelope(&graphlaw::abi::call(
            serde_json::to_vec(&ex["request"])
                .expect("request serializes")
                .as_slice(),
        ));
        match ex["outcome"].as_str().expect("outcome") {
            "ok" => assert_eq!(response["ok"], json!(true), "{id}"),
            "refused" => assert_eq!(response["ok"], json!(false), "{id}"),
            other => panic!("{id}: unknown outcome {other}"),
        }
        ops_seen.insert(op.to_owned());
    }
    assert_eq!(
        ops_seen,
        graphlaw::registry::op_names()
            .into_iter()
            .map(str::to_owned)
            .collect(),
        "every one of the 14 ops contributed at least one native vector"
    );
}

#[test]
fn zero_len_request_is_a_typed_error_never_a_trap() {
    let v = envelope(&graphlaw::abi::call(&[]));
    assert_eq!(v["ok"], json!(false));
    assert!(
        v["error"]["message"]
            .as_str()
            .expect("message")
            .starts_with("request is not JSON"),
        "zero-len vector: {:?}",
        v["error"]
    );
}

#[test]
fn oversize_request_refuses_at_max_request_bytes_plus_one() {
    let mut body = serde_json::to_vec(&json!({"op": "capabilities"})).expect("serialize");
    body.resize(graphlaw::abi::MAX_REQUEST_BYTES + 1, b' ');
    let v = envelope(&graphlaw::abi::call(&body));
    assert_eq!(v["ok"], json!(false));
    assert_eq!(v["error"]["kind"], json!("ResourceLimit"));
    assert_eq!(v["error"]["details"]["code"], json!("ResourceLimit"));
    assert_eq!(v["error"]["details"]["limit"], json!("request_bytes"));
    assert_eq!(
        v["error"]["details"]["max"],
        json!(graphlaw::abi::MAX_REQUEST_BYTES)
    );
}

#[test]
fn depth_65_request_refuses_at_max_json_depth_plus_one() {
    let mut req = String::from("{\"op\":\"capabilities\"}");
    for _ in 0..graphlaw::abi::MAX_JSON_DEPTH + 1 {
        req = format!("[{req}]"); // each bracket adds one nesting level
    }
    let v = envelope(&graphlaw::abi::call(req.as_bytes()));
    assert_eq!(v["ok"], json!(false));
    assert_eq!(v["error"]["details"]["limit"], json!("json_depth"));
    assert_eq!(
        v["error"]["details"]["max"],
        json!(graphlaw::abi::MAX_JSON_DEPTH)
    );
}

#[test]
fn manifest_export_signatures_match_the_documented_ffi() {
    let doc = graphlaw::abi_manifest::manifest_value();
    let exports = doc["exports"].as_array().expect("exports");
    let names: Vec<&str> = exports
        .iter()
        .map(|e| e["name"].as_str().unwrap())
        .collect();
    assert_eq!(names, vec!["gl_alloc", "gl_free", "gl_call"]);
    let by_name = |n: &str| {
        exports
            .iter()
            .find(|e| e["name"] == json!(n))
            .expect("export present")
            .clone()
    };
    assert_eq!(by_name("gl_alloc")["params"], json!(["i32"]));
    assert_eq!(by_name("gl_alloc")["result"], json!("i32"));
    assert_eq!(by_name("gl_free")["params"], json!(["i32", "i32"]));
    assert_eq!(by_name("gl_free")["result"], json!("()"));
    assert_eq!(by_name("gl_call")["params"], json!(["i32", "i32"]));
    assert_eq!(by_name("gl_call")["result"], json!("i64"));
}

// ----------------------------------------------------------------- frame golden

use graphlaw::receipt_chain::frame::{DenialPolarity, OcelCausalFrame, PackedObjRef};

/// The golden frame: fixed field values, zero padding.
fn golden_frame() -> OcelCausalFrame {
    OcelCausalFrame {
        instruction_id: 0x0102_0304_0506_0708,
        fired_mask: 0x1122_3344_5566_7788,
        denial: DenialPolarity::PRECONDITION_FAILED,
        obj_refs: [
            PackedObjRef(1),
            PackedObjRef(2),
            PackedObjRef(3),
            PackedObjRef(4),
            PackedObjRef(5),
            PackedObjRef(6),
            PackedObjRef(7),
            PackedObjRef(8),
        ],
        ts_ns: 0xDEAD_BEEF_CAFE_BABE,
        activity_idx: 0xBEEF,
        node_kind: 0x2A,
        pad: [0u8; 5],
        prior_hash: [0xA5u8; 32],
    }
}

/// Independent little-endian writer over the documented 99-byte layout.
fn expected_hash_bytes() -> [u8; 99] {
    let f = golden_frame();
    let mut buf = [0u8; 99];
    let le = |out: &mut [u8], v: u64| {
        for (i, b) in out.iter_mut().enumerate() {
            *b = ((v >> (i * 8)) & 0xFF) as u8;
        }
    };
    le(&mut buf[0..8], f.instruction_id);
    le(&mut buf[8..16], f.fired_mask);
    le(&mut buf[16..24], f.denial.0);
    for (i, r) in f.obj_refs.iter().enumerate() {
        le(&mut buf[24 + i * 4..28 + i * 4], r.0 as u64);
    }
    le(&mut buf[56..64], f.ts_ns);
    buf[64] = (f.activity_idx & 0xFF) as u8;
    buf[65] = (f.activity_idx >> 8) as u8;
    buf[66] = f.node_kind;
    buf[67..99].copy_from_slice(&f.prior_hash);
    buf
}

/// Pinned BLAKE3 digest of the golden frame's `to_hash_bytes`.
const GOLDEN_FRAME_HASH_HEX: &str = "0807060504030201887766554433221100ff0000000000000100000002000000030000000400000005000000060000000700000008000000bebafecaefbeaddeefbe2aa5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5";

#[test]
fn frame_hash_bytes_match_the_pinned_golden() {
    let bytes = golden_frame().to_hash_bytes();
    assert_eq!(
        bytes,
        expected_hash_bytes(),
        "recomputed vs documented layout"
    );
    let hex: String = bytes.iter().map(|b| format!("{b:02x}")).collect();
    assert_eq!(
        hex, GOLDEN_FRAME_HASH_HEX,
        "pinned golden digest of to_hash_bytes"
    );
}

#[test]
fn manifest_frame_spec_matches_the_real_struct_and_layout() {
    let frame = doc_frame_spec();
    assert_eq!(
        frame["size_bytes"],
        json!(std::mem::size_of::<OcelCausalFrame>())
    );
    assert_eq!(frame["hash_bytes_len"], json!(99));
    let fields = frame["fields"].as_array().expect("fields");
    let mut expected = 0;
    for f in fields {
        let off = f["offset"].as_u64().expect("offset") as usize;
        let len = f["len"].as_u64().expect("len") as usize;
        assert_eq!(off, expected, "field {} contiguous", f["name"]);
        expected += len;
    }
    assert_eq!(expected, 99, "field table covers exactly the hash bytes");
}

fn doc_frame_spec() -> Value {
    graphlaw::abi_manifest::manifest_value()["frame"].clone()
}
