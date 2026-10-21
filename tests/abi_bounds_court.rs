#![cfg(feature = "abi")]
//! Courts for the W06 ABI resource-bound gaps: `op_shacl` shape-depth guard,
//! `op_law` step cap, and `op_datalog` rule/fact caps. Every cap is refused at
//! exactly max+1 via the typed `ResourceLimit` details path, and a positive
//! control at exactly max must still succeed. Real ABI dispatch (`call_json`),
//! real parsers and reasoners — no mocks.

use graphlaw::abi::{MAX_DATALOG_FACTS, MAX_DATALOG_RULES, MAX_LAW_STEPS, call_json};
use serde_json::{Value, json};

fn assert_limit(r: &Value, limit: &str, observed: usize, max: usize) {
    assert_eq!(r["ok"], false, "{r}");
    assert_eq!(r["error"]["kind"], "ResourceLimit", "{r}");
    let d = &r["error"]["details"];
    assert_eq!(d["code"], "ResourceLimit", "{r}");
    assert_eq!(d["limit"], limit, "{r}");
    assert_eq!(d["observed"], observed, "{r}");
    assert_eq!(d["max"], max, "{r}");
}

fn shacl_prefix() -> &'static str {
    "@prefix sh: <http://www.w3.org/ns/shacl#> .\n@prefix ex: <http://example.org/> .\n"
}

/// `n` shapes linked by `sh:node` (acyclic chain) — same construction as
/// tests/shacl_depth_guard_court.rs, driven through the JSON ABI here.
fn node_chain(n: usize) -> String {
    let mut ttl = String::from(shacl_prefix());
    for i in 0..n {
        ttl.push_str(&format!(
            "ex:s{i} a sh:NodeShape ; sh:targetNode ex:x ; sh:property [ sh:path ex:p ] ; sh:node ex:s{} .\n",
            i + 1
        ));
    }
    ttl.push_str(&format!("ex:s{n} a sh:NodeShape .\n"));
    ttl
}

fn shacl_req(shapes: &str) -> Value {
    json!({"op": "shacl", "data": {"text": "", "dialect": "ntriples"}, "shapes": shapes})
}

fn law_req(steps: usize) -> Value {
    let steps: Vec<Value> = (0..steps).map(|_| json!({"step": "rdfs"})).collect();
    json!({"op": "law", "data": {"text": "", "dialect": "ntriples"}, "steps": steps})
}

fn datalog_req(rules: usize, facts: usize) -> Value {
    let rs: Vec<Value> = (0..rules)
        .map(|i| {
            json!({"head": [format!("urn:s{i}"), "urn:p", format!("urn:o{i}")],
                   "body": [[format!("urn:s{i}"), "urn:p", "?x"]]})
        })
        .collect();
    let fs: Vec<Value> = (0..facts)
        .map(|i| json!([format!("urn:a{i}"), "urn:p", format!("urn:b{i}")]))
        .collect();
    json!({"op": "datalog", "rules": rs, "facts": fs})
}

#[test]
fn op_shacl_refuses_shapes_deeper_than_cap_with_typed_resource_limit() {
    let max = graphlaw::shacl_guard::MAX_SHACL_VALIDATION_DEPTH;
    let r = call_json(&shacl_req(&node_chain(max + 1)));
    assert_limit(&r, "shacl_validation_depth", max + 1, max);
}

#[test]
fn op_shacl_admits_shapes_at_exactly_the_cap() {
    let max = graphlaw::shacl_guard::MAX_SHACL_VALIDATION_DEPTH;
    let r = call_json(&shacl_req(&node_chain(max)));
    assert_eq!(r["ok"], true, "{r}");
    assert_eq!(r["conforms"], true, "{r}");
}

#[test]
fn op_law_refuses_1001_steps_with_typed_resource_limit() {
    let r = call_json(&law_req(MAX_LAW_STEPS + 1));
    assert_limit(&r, "law_steps", MAX_LAW_STEPS + 1, MAX_LAW_STEPS);
}

#[test]
fn op_law_admits_exactly_1000_steps() {
    let r = call_json(&law_req(MAX_LAW_STEPS));
    assert_eq!(r["ok"], true, "{r}");
}

#[test]
fn op_datalog_refuses_over_cap_rules_with_typed_resource_limit() {
    let r = call_json(&datalog_req(MAX_DATALOG_RULES + 1, 1));
    assert_limit(&r, "datalog_rules", MAX_DATALOG_RULES + 1, MAX_DATALOG_RULES);
}

#[test]
fn op_datalog_admits_exactly_10_000_rules() {
    let r = call_json(&datalog_req(MAX_DATALOG_RULES, 1));
    assert_eq!(r["ok"], true, "{r}");
}

#[test]
fn op_datalog_refuses_over_cap_facts_with_typed_resource_limit() {
    let r = call_json(&datalog_req(1, MAX_DATALOG_FACTS + 1));
    assert_limit(&r, "datalog_facts", MAX_DATALOG_FACTS + 1, MAX_DATALOG_FACTS);
}

#[test]
fn op_datalog_admits_exactly_100_000_facts() {
    let r = call_json(&datalog_req(1, MAX_DATALOG_FACTS));
    assert_eq!(r["ok"], true, "{r}");
}
