//! Runs every example in `registry/op-examples.json` on the native ABI and on
//! the compiled wasm module: responses must be byte-identical, the outcome and
//! refusal identity must match the example, and the file must cover every op.
#![cfg(all(feature = "abi", not(target_arch = "wasm32")))]

mod common;

use std::collections::{BTreeMap, BTreeSet};

use serde_json::Value;

fn examples() -> Vec<Value> {
    let doc: Value = serde_json::from_str(&common::read("registry/op-examples.json"))
        .expect("op-examples.json is JSON");
    assert_eq!(doc["schema"], "graphlaw.op-examples/1");
    assert_eq!(doc["abi_version"], 1);
    doc["examples"].as_array().expect("examples array").clone()
}

fn abi_ops() -> Vec<String> {
    common::native(&serde_json::json!({"op": "capabilities"}))["ops"]
        .as_array()
        .expect("ops array")
        .iter()
        .map(|o| o.as_str().expect("op name").to_string())
        .collect()
}

fn s(v: &Value, k: &str) -> String {
    v[k].as_str()
        .unwrap_or_else(|| panic!("`{k}` must be a string in {v}"))
        .to_string()
}

#[test]
fn every_example_matches_native_and_wasm() {
    // Positive control first: a known-good request passes the harness.
    let control = common::ok(&serde_json::json!({"op": "capabilities"}));
    assert_eq!(control["abi_version"], 1);

    let mut per_op: BTreeMap<String, (usize, usize)> = BTreeMap::new();
    for ex in examples() {
        let id = s(&ex, "id");
        let op = s(&ex, "op");
        assert_eq!(ex["request"]["op"], op.as_str(), "{id}: request op");
        let entry = per_op.entry(op.clone()).or_default();
        match s(&ex, "outcome").as_str() {
            "ok" => {
                entry.0 += 1;
                let r = common::ok(&ex["request"]);
                assert_eq!(r["ok"], true, "{id}");
                assert!(ex["refusal_kind"].is_null(), "{id}: ok has no refusal_kind");
                assert!(ex["refusal_code"].is_null(), "{id}: ok has no refusal_code");
            }
            "refused" => {
                entry.1 += 1;
                let r = common::refused(&ex["request"]);
                assert_eq!(
                    r["error"]["kind"], ex["refusal_kind"],
                    "{id}: error.kind (response: {r})"
                );
                let code = r["error"]
                    .get("details")
                    .and_then(|d| d.get("code"))
                    .cloned()
                    .unwrap_or(Value::Null);
                assert_eq!(code, ex["refusal_code"], "{id}: error.details.code ({r})");
            }
            other => panic!("{id}: unknown outcome `{other}`"),
        }
    }
    for (op, (ok_n, ref_n)) in &per_op {
        println!(
            "op-examples {op}: {ok_n} ok, {ref_n} refused, {} total",
            ok_n + ref_n
        );
    }
}

#[test]
fn examples_file_is_structurally_sound() {
    let exs = examples();
    let ops = abi_ops();
    assert_eq!(ops.len(), 14, "ABI exposes 14 ops: {ops:?}");

    let mut ids = BTreeSet::new();
    for ex in &exs {
        let id = s(ex, "id");
        assert!(ids.insert(id.clone()), "duplicate example id {id}");
        let op = s(ex, "op");
        assert!(id.starts_with(&format!("{op}.")), "{id}: id is <op>.<slug>");
        assert!(ops.contains(&op), "{id}: unknown op {op}");
        assert!(ex["description"].is_string(), "{id}: description");
        assert!(
            ex["positive_control"].is_boolean(),
            "{id}: positive_control"
        );
        assert!(ex["request"].is_object(), "{id}: request object");
        for k in ["refusal_kind", "refusal_code"] {
            assert!(
                ex[k].is_null() || ex[k].is_string(),
                "{id}: {k} is string or null"
            );
        }
        let text = ex["request"].to_string();
        assert!(!text.contains("signed_lease"), "{id}: no signatures");
        assert!(text.is_ascii(), "{id}: ASCII-only request");
    }

    for op in &ops {
        let mine: Vec<&Value> = exs.iter().filter(|e| e["op"] == op.as_str()).collect();
        assert!(
            mine.iter().any(|e| e["outcome"] == "ok"),
            "{op}: needs an ok example"
        );
        if op != "capabilities" {
            assert!(
                mine.iter().any(|e| e["outcome"] == "refused"),
                "{op}: needs a refused example"
            );
        }
        // ok examples come before refused ones, and a positive control leads.
        let first_refused = mine.iter().position(|e| e["outcome"] == "refused");
        let last_ok = mine.iter().rposition(|e| e["outcome"] == "ok");
        if let (Some(fr), Some(lo)) = (first_refused, last_ok) {
            assert!(lo < fr, "{op}: ok examples must precede refused examples");
        }
        assert_eq!(
            mine[0]["positive_control"], true,
            "{op}: first example is the positive control"
        );
    }

    // examples appear grouped in ABI op order
    let mut order: Vec<&str> = Vec::new();
    for ex in &exs {
        let op = ex["op"].as_str().unwrap();
        if order.last() != Some(&op) {
            assert!(!order.contains(&op), "{op}: examples are not contiguous");
            order.push(op);
        }
    }
    assert_eq!(order, ops.iter().map(String::as_str).collect::<Vec<_>>());
}

#[test]
fn examples_conform_to_the_schema_file() {
    let schema: Value = serde_json::from_str(&common::read("registry/op-examples.schema.json"))
        .expect("schema is JSON");
    assert_eq!(
        schema["$schema"],
        "https://json-schema.org/draft/2020-12/schema"
    );
    let item = &schema["properties"]["examples"]["items"];
    let required: Vec<&str> = item["required"]
        .as_array()
        .expect("item.required")
        .iter()
        .map(|v| v.as_str().unwrap())
        .collect();
    let props = item["properties"].as_object().expect("item.properties");
    for ex in examples() {
        let obj = ex.as_object().unwrap();
        for r in &required {
            assert!(obj.contains_key(*r), "{}: missing {r}", ex["id"]);
        }
        for k in obj.keys() {
            assert!(props.contains_key(k), "{}: key {k} not in schema", ex["id"]);
        }
    }
}
