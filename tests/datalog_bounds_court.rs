#![cfg(feature = "abi")]
//! Court: exploding datalog (transitive closure over a long chain) refuses typed
//! instead of allocating unbounded. Real ABI dispatch, real evaluator.

use graphlaw::abi::call_json;
use serde_json::{Value, json};

fn closure_req(n: usize) -> Value {
    let (p, t) = ("http://e/p", "http://e/t");
    let facts: Vec<Value> = (0..n)
        .map(|i| json!([format!("http://e/n{i}"), p, format!("http://e/n{}", i + 1)]))
        .collect();
    json!({"op": "datalog", "rules": [
        {"head": ["?x", t, "?y"], "body": [["?x", p, "?y"]]},
        {"head": ["?x", t, "?z"], "body": [["?x", t, "?y"], ["?y", p, "?z"]]},
    ], "facts": facts})
}

#[test]
fn small_chain_still_evaluates() {
    let r = call_json(&closure_req(5));
    assert_eq!(r["ok"], true, "{r}");
}

#[test]
fn exploding_closure_refuses_typed() {
    // 99_000 seeded facts (under the input cap) derive ~4.9e9 closure facts.
    let r = call_json(&closure_req(99_000));
    assert_eq!(r["ok"], false);
    assert_eq!(r["error"]["kind"], "ResourceLimit");
    let limit = r["error"]["details"]["limit"].as_str().expect("ResourceLimit details.limit is a string");
    assert!(
        limit == "datalog_derived_facts" || limit == "datalog_join_steps",
        "unexpected limit {limit}"
    );
}
