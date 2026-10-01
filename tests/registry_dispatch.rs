//! Registry-vs-dispatch court (feature `abi`, Chicago: real `graphlaw::abi::call`, no doubles).
//!
//! The capability registry (`src/registry.rs`) is the source of truth for the public op set;
//! `abi::dispatch` is the executable. This court fails when they drift in either direction.
#![cfg(feature = "abi")]

use graphlaw::abi::call_json;
use graphlaw::registry;
use serde_json::{Value, json};
use std::collections::BTreeSet;

fn names() -> Vec<String> {
    registry::op_names()
        .into_iter()
        .map(|s| s.to_string())
        .collect()
}

/// Arm names of `fn dispatch`, parsed from source: one `"op" => ...` per line, up to the
/// `other =>` fallthrough.
fn dispatch_arms() -> Vec<String> {
    let src = include_str!("../src/abi.rs");
    let start = src
        .find("fn dispatch(")
        .expect("fn dispatch present in src/abi.rs");
    let body = &src[start..];
    let end = body
        .find("unknown op")
        .expect("dispatch fallthrough `unknown op` present");
    let mut arms = Vec::new();
    for line in body[..end].lines() {
        let t = line.trim_start();
        if let Some(rest) = t.strip_prefix('"')
            && let Some(q) = rest.find('"')
            && rest[q + 1..].trim_start().starts_with("=>")
        {
            arms.push(rest[..q].to_string());
        }
    }
    arms
}

fn is_unknown_op(resp: &Value) -> bool {
    resp["ok"] == json!(false)
        && resp["error"]["message"]
            .as_str()
            .map(|m| m.contains("unknown op"))
            .unwrap_or(false)
}

#[test]
fn positive_control_dispatch_arms_are_parsed_and_nonempty() {
    let arms = dispatch_arms();
    assert!(!arms.is_empty(), "no dispatch arms parsed from src/abi.rs");
    assert!(arms.contains(&"capabilities".to_string()));
    assert!(arms.contains(&"policy".to_string()));
    let live = call_json(&json!({"op": "capabilities"}));
    assert_eq!(live["ok"], json!(true));
}

#[test]
fn dispatch_arms_equal_registry_op_names_as_sets() {
    let arms = dispatch_arms();
    let arm_set: BTreeSet<_> = arms.iter().cloned().collect();
    assert_eq!(
        arm_set.len(),
        arms.len(),
        "duplicate dispatch arm: {arms:?}"
    );
    let reg = names();
    let reg_set: BTreeSet<_> = reg.iter().cloned().collect();
    assert_eq!(reg_set.len(), reg.len(), "duplicate registry op: {reg:?}");
    let only_dispatch: Vec<_> = arm_set.difference(&reg_set).collect();
    let only_registry: Vec<_> = reg_set.difference(&arm_set).collect();
    assert!(
        only_dispatch.is_empty() && only_registry.is_empty(),
        "drift: dispatch-only {only_dispatch:?}, registry-only {only_registry:?}"
    );
}

#[test]
fn every_registry_op_is_dispatched_not_unknown() {
    for name in names() {
        let resp = call_json(&json!({"op": name}));
        assert!(
            !is_unknown_op(&resp),
            "registry op `{name}` fell through to `unknown op`: {resp}"
        );
    }
}

#[test]
fn op_absent_from_registry_is_unsupported_unknown_op() {
    let absent = "definitely_not_a_graphlaw_op";
    assert!(!names().iter().any(|n| n == absent));
    let resp = call_json(&json!({"op": absent}));
    assert_eq!(resp["ok"], json!(false));
    assert_eq!(resp["error"]["kind"], json!("Unsupported"));
    let msg = resp["error"]["message"].as_str().expect("message string");
    assert!(msg.contains("unknown op"), "message: {msg}");
    assert!(is_unknown_op(&resp));
}

#[test]
fn capabilities_ops_equal_registry_names_in_order_with_fourteen_entries() {
    let resp = call_json(&json!({"op": "capabilities"}));
    let ops: Vec<String> = resp["ops"]
        .as_array()
        .expect("ops array")
        .iter()
        .map(|v| v.as_str().expect("op name string").to_string())
        .collect();
    assert_eq!(ops, names());
    assert_eq!(ops.len(), 14);
}

#[test]
fn every_registry_op_declares_request_and_responses() {
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/registry/capability-registry.json"
    );
    let text = std::fs::read_to_string(path).unwrap_or_else(|e| {
        panic!("emitted registry missing at {path}: {e} (run graphlaw-registry --write)")
    });
    let reg: Value = serde_json::from_str(&text).expect("registry JSON parses");
    let ops = reg["ops"].as_array().expect("ops array");
    let listed: Vec<String> = ops
        .iter()
        .map(|o| o["name"].as_str().expect("op name").to_string())
        .collect();
    assert_eq!(
        listed,
        names(),
        "emitted registry op order differs from src/registry.rs"
    );
    for op in ops {
        let name = op["name"].as_str().unwrap();
        assert!(
            op["request"]["fields"].is_array(),
            "{name}: request.fields not an array"
        );
        let responses = op["responses"]
            .as_array()
            .unwrap_or_else(|| panic!("{name}: responses missing"));
        assert!(!responses.is_empty(), "{name}: empty responses");
        for v in responses {
            let fields = v["fields"]
                .as_array()
                .unwrap_or_else(|| panic!("{name}: variant fields missing"));
            assert!(
                !fields.is_empty(),
                "{name}: variant with no response fields"
            );
        }
        // Every op except `capabilities` takes at least one request field.
        if name != "capabilities" {
            assert!(
                !op["request"]["fields"].as_array().unwrap().is_empty(),
                "{name}: empty request fields"
            );
        }
        assert!(
            !op["summary"].as_str().unwrap_or("").is_empty(),
            "{name}: empty summary"
        );
    }
}

// ---------------------------------------------------------------- closed vocabularies
//
// The registry closed vocabularies (regimes, law steps, dialect names and aliases, ShEx schema
// dialects) are literals in `src/registry.rs`; the executable matches on separate literals in
// `src/abi.rs`. These courts tie the two: every registry value is accepted by dispatch, and a
// value outside the vocabulary is refused with a typed message.

const NT_DATA: &str = "<urn:a:C> <http://www.w3.org/2000/01/rdf-schema#subClassOf> <urn:a:D> .\n\
                       <urn:a:x> <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <urn:a:C> .\n";

fn message(resp: &Value) -> String {
    resp["error"]["message"].as_str().unwrap_or("").to_string()
}

#[test]
fn positive_control_every_registry_regime_runs_entail() {
    for regime in registry::REGIMES {
        let resp = call_json(&json!({
            "op": "entail", "regime": regime,
            "data": {"text": NT_DATA, "dialect": "ntriples"}
        }));
        assert_eq!(resp["ok"], json!(true), "regime `{regime}` refused: {resp}");
        assert!(resp["nquads"].is_string(), "regime `{regime}`: {resp}");
    }
}

#[test]
fn negative_regime_outside_registry_is_refused_unknown_regime() {
    let resp = call_json(&json!({
        "op": "entail", "regime": "definitely-not-a-regime",
        "data": {"text": NT_DATA, "dialect": "ntriples"}
    }));
    assert_eq!(resp["ok"], json!(false));
    assert!(message(&resp).contains("unknown regime"), "{resp}");
}

#[test]
fn positive_control_every_registry_shex_schema_dialect_reaches_the_parser() {
    let reg = registry::registry_value();
    let shex = reg["ops"]
        .as_array()
        .unwrap()
        .iter()
        .find(|o| o["name"] == json!("shex"))
        .expect("shex op");
    let field = shex["request"]["fields"]
        .as_array()
        .unwrap()
        .iter()
        .find(|f| f["name"] == json!("schema_dialect"))
        .expect("schema_dialect field");
    let values: Vec<&str> = field["enum"]
        .as_array()
        .expect("schema_dialect declares an enum")
        .iter()
        .map(|v| v.as_str().unwrap())
        .collect();
    assert_eq!(values, ["shexc", "shexj"]);
    for value in values {
        // Empty schema text is not valid in either syntax; the point is that the dialect
        // name is accepted and the refusal (if any) comes from the parser, not the router.
        let resp = call_json(&json!({
            "op": "shex", "schema_dialect": value,
            "data": {"text": NT_DATA, "dialect": "ntriples"},
            "schema": "", "map": "<urn:a:x>@<urn:s:S>"
        }));
        assert!(
            !message(&resp).contains("unknown schema_dialect"),
            "schema_dialect `{value}` not dispatched: {resp}"
        );
    }
}

#[test]
fn negative_shex_schema_dialect_outside_registry_is_refused_not_parsed_as_shexc() {
    let good = "PREFIX ex: <urn:s:>\nex:S { }";
    let control = call_json(&json!({
        "op": "shex", "schema_dialect": "shexc",
        "data": {"text": NT_DATA, "dialect": "ntriples"},
        "schema": good, "map": "<urn:a:x>@<urn:s:S>"
    }));
    assert_eq!(control["ok"], json!(true), "shexc control: {control}");
    for bad in ["turtle", "ShExC", "", "shex"] {
        let resp = call_json(&json!({
            "op": "shex", "schema_dialect": bad,
            "data": {"text": NT_DATA, "dialect": "ntriples"},
            "schema": good, "map": "<urn:a:x>@<urn:s:S>"
        }));
        assert_eq!(resp["ok"], json!(false), "`{bad}` was accepted: {resp}");
        assert!(
            message(&resp).contains("unknown schema_dialect"),
            "`{bad}`: {resp}"
        );
    }
}

#[test]
fn positive_control_every_registry_law_step_is_dispatched() {
    for step in registry::LAW_STEPS {
        // No lease and no step fields: the request must get past the step-name match and
        // be refused (or succeed) for a reason other than `unknown step`.
        let resp = call_json(&json!({
            "op": "law",
            "data": {"text": NT_DATA, "dialect": "ntriples"},
            "steps": [{"step": step.name}]
        }));
        assert!(
            !message(&resp).contains("unknown step"),
            "law step `{}` fell through to `unknown step`: {resp}",
            step.name
        );
    }
}

#[test]
fn negative_law_step_outside_registry_is_refused_unknown_step() {
    assert!(!registry::LAW_STEPS.iter().any(|s| s.name == "no-such-step"));
    let resp = call_json(&json!({
        "op": "law",
        "data": {"text": NT_DATA, "dialect": "ntriples"},
        "steps": [{"step": "no-such-step"}]
    }));
    assert_eq!(resp["ok"], json!(false));
    assert!(message(&resp).contains("unknown step"), "{resp}");
}

#[test]
fn positive_control_every_registry_dialect_name_and_alias_is_accepted() {
    for row in registry::RDF_DIALECTS
        .iter()
        .chain(registry::OTHER_DIALECTS)
    {
        for name in std::iter::once(&row.name).chain(row.aliases) {
            let d = graphlaw::abi::dialect_by_name(name)
                .unwrap_or_else(|e| panic!("dialect name `{name}` not accepted: {e}"));
            assert_eq!(
                format!("{d:?}"),
                row.response_name,
                "dialect `{name}` response name"
            );
        }
    }
}

#[test]
fn negative_dialect_name_outside_registry_is_refused() {
    assert!(graphlaw::abi::dialect_by_name("definitely-not-a-dialect").is_err());
    let resp = call_json(&json!({
        "op": "parse", "text": NT_DATA, "dialect": "definitely-not-a-dialect"
    }));
    assert_eq!(resp["ok"], json!(false));
    assert!(message(&resp).contains("unknown dialect"), "{resp}");
}
