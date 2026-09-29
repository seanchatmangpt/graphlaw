#![cfg(feature = "abi")]
//! G5: documented resource caps are enforced before heavy work and refuse with
//! a typed `ResourceLimit` (`error.details = {code, limit, observed, max}`).
//! Real ABI, real parsers, real N3 reasoner (no mocks).

use graphlaw::abi::{
    MAX_ATOMS_PER_FIELD, MAX_JSON_DEPTH, MAX_PLAN_ACTIONS, MAX_POLICY_ENTRIES, MAX_REQUEST_BYTES,
    call, call_json,
};
use serde_json::{Value, json};

fn parse(bytes: Vec<u8>) -> Value {
    serde_json::from_slice(&bytes).expect("response is JSON")
}

fn assert_limit(r: &Value, limit: &str, observed: usize, max: usize) {
    assert_eq!(r["ok"], false, "{r}");
    assert_eq!(r["error"]["kind"], "ResourceLimit", "{r}");
    let d = &r["error"]["details"];
    assert_eq!(d["code"], "ResourceLimit", "{r}");
    assert_eq!(d["limit"], limit, "{r}");
    assert_eq!(d["observed"], observed, "{r}");
    assert_eq!(d["max"], max, "{r}");
}

fn at(o: &str) -> String {
    format!("<urn:p:robot> <urn:p:at> <urn:p:{o}> .\n")
}

fn plan_req(n: usize) -> Value {
    let actions: Vec<Value> = (0..n)
        .map(|i| json!({"name": format!("a{i}"), "pre": "", "add": "", "del": ""}))
        .collect();
    json!({"op": "law", "data": {"text": at("a"), "dialect": "ntriples"},
        "steps": [{"step": "plan", "plan": {"actions": actions, "goal": ""}}]})
}

/// `{"op":"capabilities","pad":"xxxx..."}` of exactly `total` bytes.
fn padded_body(total: usize) -> Vec<u8> {
    let head = br#"{"op":"capabilities","pad":""#;
    let tail = br#""}"#;
    let mut b = Vec::with_capacity(total);
    b.extend_from_slice(head);
    b.resize(total - tail.len(), b'x');
    b.extend_from_slice(tail);
    assert_eq!(b.len(), total);
    b
}

#[test]
fn request_over_cap_is_refused_and_at_cap_still_works() {
    let over = MAX_REQUEST_BYTES + 1;
    assert_limit(
        &parse(call(&padded_body(over))),
        "request_bytes",
        over,
        MAX_REQUEST_BYTES,
    );
    let ok = parse(call(&padded_body(MAX_REQUEST_BYTES)));
    assert_eq!(ok["ok"], true, "{ok}");
}

fn nested(depth: usize) -> Vec<u8> {
    let mut s = String::from(r#"{"op":"capabilities","x":"#);
    s.push_str(&"[".repeat(depth - 1));
    s.push_str(&"]".repeat(depth - 1));
    s.push('}');
    s.into_bytes()
}

#[test]
fn json_depth_over_cap_is_refused_and_at_cap_still_works() {
    assert_limit(
        &parse(call(&nested(100))),
        "json_depth",
        100,
        MAX_JSON_DEPTH,
    );
    // depth = 1 (object) + (depth-1) arrays == MAX_JSON_DEPTH
    let ok = parse(call(&nested(MAX_JSON_DEPTH)));
    assert_eq!(ok["ok"], true, "{ok}");
    // brackets inside strings do not count
    let s = format!(r#"{{"op":"capabilities","x":"{}"}}"#, "[".repeat(500));
    assert_eq!(parse(call(s.as_bytes()))["ok"], true);
}

#[test]
fn plan_action_count_over_cap_is_refused_and_at_cap_admitted() {
    assert_limit(
        &call_json(&plan_req(MAX_PLAN_ACTIONS + 1)),
        "plan_actions",
        MAX_PLAN_ACTIONS + 1,
        MAX_PLAN_ACTIONS,
    );
    let ok = call_json(&plan_req(MAX_PLAN_ACTIONS));
    assert_eq!(
        ok["ok"],
        true,
        "{}",
        ok.to_string().chars().take(300).collect::<String>()
    );
}

#[test]
fn atoms_per_field_over_cap_is_refused() {
    let nt = |n: usize| {
        (0..n)
            .map(|i| format!("<urn:p:s> <urn:p:p> <urn:p:o{i}> .\n"))
            .collect::<String>()
    };
    let req = |n: usize| {
        json!({"op": "law", "data": {"text": at("a"), "dialect": "ntriples"},
        "steps": [{"step": "plan", "plan": {"actions": [], "goal": nt(n)}}]})
    };
    assert_limit(
        &call_json(&req(MAX_ATOMS_PER_FIELD + 1)),
        "atoms_per_field",
        MAX_ATOMS_PER_FIELD + 1,
        MAX_ATOMS_PER_FIELD,
    );
}

#[test]
fn policy_entries_over_cap_are_refused_and_small_policy_still_works() {
    let problem = json!({
        "states": [{"id": "s0"}, {"id": "g", "facts": ["done"]}],
        "initial_states": ["s0"], "goal": {"facts": ["done"]},
        "transitions": [{"action": "go", "from": "s0", "to": "g", "probability_ppm": 1000000}]});
    let entry = json!({"state": "s0", "action": "go",
        "outcomes": [{"state": "g", "probability_ppm": 1000000}]});
    let big: Vec<Value> = vec![entry.clone(); MAX_POLICY_ENTRIES + 1];
    assert_limit(
        &call_json(&json!({"op": "policy", "problem": problem, "policy": {"policy": big}})),
        "policy_entries",
        MAX_POLICY_ENTRIES + 1,
        MAX_POLICY_ENTRIES,
    );
    let ok = call_json(&json!({"op": "policy", "problem": problem, "policy": [entry]}));
    assert_eq!(ok["ok"], true, "{ok}");
}

#[test]
fn nonterminating_n3_rules_refuse_with_resource_limit_not_a_hang() {
    // Each derived triple mints a fresh blank node that triggers the rule again.
    let req = json!({"op": "n3",
        "text": "@prefix : <urn:n:> .\n:a :s :b .\n{ ?x :s ?y } => { ?y :s _:n } .\n"});
    let r = call_json(&req);
    assert_eq!(r["ok"], false, "{r}");
    assert_eq!(r["error"]["kind"], "ResourceLimit", "{r}");
    assert_eq!(r["error"]["details"]["code"], "ResourceLimit");
    assert_eq!(r["error"]["details"]["limit"], "n3_iterations");
    assert_eq!(
        r["error"]["details"]["max"],
        graphlaw::law::N3_MAX_ITERATIONS
    );
    // a terminating rule set is unaffected
    let ok = call_json(&json!({"op": "n3",
        "text": "@prefix : <urn:n:> .\n:a :p :b .\n{ ?x :p ?y } => { ?y :q ?x } .\n"}));
    assert_eq!(ok["ok"], true, "{ok}");
}
