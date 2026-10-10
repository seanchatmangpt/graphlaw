//! Forward compatibility of the request decoder: every registry op example,
//! padded with unknown top-level request keys, must produce the same outcome
//! and the same response bytes as the unpadded request, on native and wasm.
//!
//! Examples come from `registry/op-examples.json` (schema
//! `graphlaw.op-examples/1`). Set `GRAPHLAW_WASM` to test a prebuilt module.
#![cfg(all(feature = "abi", not(target_arch = "wasm32")))]

mod common;

use serde_json::{Map, Value, json};

fn examples() -> Vec<Value> {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("registry/op-examples.json");
    let text =
        std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
    let doc: Value = serde_json::from_str(&text).expect("op-examples.json is JSON");
    assert_eq!(doc["schema"], "graphlaw.op-examples/1");
    let ex = doc["examples"].as_array().expect("examples array").clone();
    assert!(!ex.is_empty(), "examples are present");
    ex
}

/// The unknown-key injections, each a (label, key, value) triple.
fn injections() -> Vec<(&'static str, &'static str, Value)> {
    vec![
        ("pad", "pad", json!(null)),
        ("x", "x", json!(1)),
        (
            "nested",
            "unknown_nested",
            json!({"a": {"b": [1, "two", {"c": true}]}, "d": null}),
        ),
        ("huge", "unknown_blob", json!("z".repeat(64 * 1024))),
    ]
}

fn inject(req: &Value, items: &[(&'static str, &'static str, Value)]) -> Value {
    let mut o: Map<String, Value> = req.as_object().expect("request object").clone();
    for (_, k, v) in items {
        assert!(!o.contains_key(*k), "example already uses key `{k}`");
        o.insert((*k).to_string(), v.clone());
    }
    Value::Object(o)
}

fn envelope_ok(bytes: &[u8]) -> bool {
    let v: Value = serde_json::from_slice(bytes).expect("response is JSON");
    v["ok"] == true
}

// ------------------------------------------------------------ positive controls

#[test]
fn a_control_every_example_reaches_its_declared_outcome_on_both_runtimes() {
    let mut oks = 0;
    let mut refusals = 0;
    for ex in examples() {
        let id = ex["id"].as_str().unwrap().to_string();
        let req = &ex["request"];
        let n = common::native_bytes(req);
        let w = common::wasm_bytes(req);
        assert_eq!(n, w, "{id}: native == wasm");
        match ex["outcome"].as_str().unwrap() {
            "ok" => {
                assert!(envelope_ok(&w), "{id}: declared ok");
                oks += 1;
            }
            "refused" => {
                assert!(!envelope_ok(&w), "{id}: declared refused");
                refusals += 1;
            }
            other => panic!("{id}: unknown outcome {other}"),
        }
    }
    assert!(oks >= 14, "every op contributes an ok example, got {oks}");
    assert!(
        refusals >= 13,
        "every op but capabilities refuses, got {refusals}"
    );
}

#[test]
fn b_control_the_comparison_detects_a_real_request_difference() {
    let ex = examples();
    let first_ok = ex
        .iter()
        .find(|e| e["outcome"] == "ok" && e["op"] == "parse")
        .expect("a parse ok example");
    let base = common::wasm_bytes(&first_ok["request"]);
    let mut changed = first_ok["request"].clone();
    changed["op"] = json!("no_such_op");
    let other = common::wasm_bytes(&changed);
    assert_ne!(base, other, "an actual change alters the response bytes");
    assert!(envelope_ok(&base));
    assert!(!envelope_ok(&other));
}

// ------------------------------------------------------------ the property

#[test]
fn unknown_top_level_keys_never_change_any_response_byte() {
    let inj = injections();
    for ex in examples() {
        let id = ex["id"].as_str().unwrap().to_string();
        let req = &ex["request"];
        let base_w = common::wasm_bytes(req);
        let base_n = common::native_bytes(req);
        assert_eq!(base_n, base_w, "{id}: baseline native == wasm");

        let mut variants: Vec<(String, Value)> = inj
            .iter()
            .map(|one| (one.0.to_string(), inject(req, std::slice::from_ref(one))))
            .collect();
        variants.push(("all".into(), inject(req, &inj)));

        for (label, padded) in variants {
            let w = common::wasm_bytes(&padded);
            let n = common::native_bytes(&padded);
            assert_eq!(
                envelope_ok(&w),
                envelope_ok(&base_w),
                "{id} [{label}]: outcome unchanged"
            );
            assert_eq!(
                String::from_utf8_lossy(&w),
                String::from_utf8_lossy(&base_w),
                "{id} [{label}]: wasm bytes unchanged by unknown keys"
            );
            assert_eq!(
                String::from_utf8_lossy(&n),
                String::from_utf8_lossy(&base_n),
                "{id} [{label}]: native bytes unchanged by unknown keys"
            );
            assert_eq!(
                String::from_utf8_lossy(&n),
                String::from_utf8_lossy(&w),
                "{id} [{label}]: native == wasm with unknown keys"
            );
        }
    }
}

#[test]
fn unknown_keys_do_not_leak_into_any_response() {
    let inj = injections();
    for ex in examples() {
        let id = ex["id"].as_str().unwrap().to_string();
        let padded = inject(&ex["request"], &inj);
        let out = String::from_utf8(common::wasm_bytes(&padded)).expect("utf8 response");
        for (_, k, _) in &inj {
            let as_key = format!("\"{k}\":");
            assert!(
                !out.contains(&as_key),
                "{id}: response echoes unknown key `{k}`"
            );
        }
        assert!(
            !out.contains("zzzzzzzzzz"),
            "{id}: response echoes the injected blob"
        );
    }
}

#[test]
fn unknown_keys_on_the_capabilities_op_leave_the_advertised_surface_intact() {
    let base = common::wasm_bytes(&json!({"op": "capabilities"}));
    assert!(envelope_ok(&base));
    let padded = common::wasm_bytes(&json!({
        "op": "capabilities", "pad": 1, "x": {"y": [1, 2, 3]}}));
    assert_eq!(base, padded);
    let v: Value = serde_json::from_slice(&padded).unwrap();
    assert_eq!(v["ops"].as_array().unwrap().len(), 14);
}
