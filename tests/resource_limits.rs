#![cfg(feature = "abi")]
//! G5: documented resource caps are enforced before heavy work and refuse with
//! a typed `ResourceLimit` (`error.details = {code, limit, observed, max}`).
//! Real ABI, real parsers, real N3 reasoner (no mocks).

use graphlaw::abi::{
    MAX_ATOMS_PER_FIELD, MAX_BODY_ATOMS, MAX_DATALOG_FACTS, MAX_DATALOG_RULES,
    MAX_EMBEDDED_JSON_BYTES, MAX_ENTAIL_QUADS, MAX_JSON_DEPTH, MAX_LAW_STEPS, MAX_NAME_BYTES,
    MAX_PLAN_ACTIONS, MAX_PLAN_ATOMS, MAX_POLICY_ENTRIES, MAX_REQUEST_BYTES, call, call_json,
    call_with_response_cap,
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

#[test]
fn pre_not_and_goal_not_count_toward_the_atoms_per_field_cap() {
    let nt = |n: usize| {
        (0..n)
            .map(|i| format!("<urn:p:s> <urn:p:p> <urn:p:o{i}> .\n"))
            .collect::<String>()
    };
    let goal_not = json!({"op": "law", "data": {"text": at("a"), "dialect": "ntriples"},
        "steps": [{"step": "plan", "plan": {"actions": [], "goal_not": nt(MAX_ATOMS_PER_FIELD + 1)}}]});
    assert_limit(
        &call_json(&goal_not),
        "atoms_per_field",
        MAX_ATOMS_PER_FIELD + 1,
        MAX_ATOMS_PER_FIELD,
    );
    let pre_not = json!({"op": "law", "data": {"text": at("a"), "dialect": "ntriples"},
        "steps": [{"step": "plan", "plan": {"actions": [
            {"name": "x", "pre_not": nt(MAX_ATOMS_PER_FIELD + 1)}], "goal": ""}}]});
    assert_limit(
        &call_json(&pre_not),
        "atoms_per_field",
        MAX_ATOMS_PER_FIELD + 1,
        MAX_ATOMS_PER_FIELD,
    );
}

#[test]
fn response_over_cap_is_refused_and_at_cap_still_works() {
    let req = br#"{"op":"capabilities"}"#;
    let len = call(req).len();
    let ok = parse(call_with_response_cap(req, len));
    assert_eq!(ok["ok"], true, "{ok}");
    assert_limit(
        &parse(call_with_response_cap(req, len - 1)),
        "response_bytes",
        len,
        len - 1,
    );
}

#[test]
fn law_step_count_over_cap_is_refused_and_at_cap_runs() {
    let req = |n: usize| {
        let steps: Vec<Value> = (0..n).map(|_| json!({"step": "record-receipts"})).collect();
        json!({"op": "law", "data": {"text": at("a"), "dialect": "ntriples"}, "steps": steps})
    };
    assert_limit(
        &call_json(&req(MAX_LAW_STEPS + 1)),
        "law_steps",
        MAX_LAW_STEPS + 1,
        MAX_LAW_STEPS,
    );
    let ok = call_json(&req(MAX_LAW_STEPS));
    assert_eq!(ok["ok"], true, "{ok}");
}

#[test]
fn plan_total_atoms_over_cap_is_refused_and_at_cap_admitted() {
    // Each field stays under MAX_ATOMS_PER_FIELD; only the per-plan total trips.
    let per = MAX_ATOMS_PER_FIELD;
    let adds = |actions: usize| {
        let acts: Vec<Value> = (0..actions)
            .map(|a| {
                let nt: String = (0..per)
                    .map(|i| format!("<urn:p:s{a}> <urn:p:p> <urn:p:o{i}> .\n"))
                    .collect();
                json!({"name": format!("a{a}"), "add": nt})
            })
            .collect();
        json!({"op": "law", "data": {"text": at("a"), "dialect": "ntriples"},
            "steps": [{"step": "plan", "plan": {"actions": acts, "goal": ""}}]})
    };
    let fit = MAX_PLAN_ATOMS / per;
    assert_limit(
        &call_json(&adds(fit + 1)),
        "plan_atoms",
        (fit + 1) * per,
        MAX_PLAN_ATOMS,
    );
    let ok = call_json(&adds(fit));
    assert_eq!(
        ok["ok"],
        true,
        "{}",
        ok.to_string().chars().take(300).collect::<String>()
    );
}

#[test]
fn action_name_over_cap_is_refused_and_at_cap_admitted() {
    let req = |n: usize| {
        json!({"op": "law", "data": {"text": at("a"), "dialect": "ntriples"},
            "steps": [{"step": "plan", "plan": {"actions": [{"name": "x".repeat(n)}], "goal": ""}}]})
    };
    assert_limit(
        &call_json(&req(MAX_NAME_BYTES + 1)),
        "name_bytes",
        MAX_NAME_BYTES + 1,
        MAX_NAME_BYTES,
    );
    let ok = call_json(&req(MAX_NAME_BYTES));
    assert_eq!(ok["ok"], true, "{ok}");
}

/// A datalog rule that never fires: `?x <urn:none:i> ?y` has no matching fact.
fn dead_rule(i: usize, body_atoms: usize) -> Value {
    let body: Vec<Value> = (0..body_atoms)
        .map(|_| json!(["?x", format!("urn:none:{i}"), "?y"]))
        .collect();
    json!({"head": ["?x", format!("urn:q:{i}"), "?y"], "body": body})
}

#[test]
fn datalog_rule_count_over_cap_is_refused_and_at_cap_works() {
    let req = |n: usize| {
        let rules: Vec<Value> = (0..n).map(|i| dead_rule(i, 1)).collect();
        json!({"op": "datalog", "rules": rules, "facts": [["urn:s", "urn:p", "urn:o"]]})
    };
    assert_limit(
        &call_json(&req(MAX_DATALOG_RULES + 1)),
        "datalog_rules",
        MAX_DATALOG_RULES + 1,
        MAX_DATALOG_RULES,
    );
    let ok = call_json(&req(MAX_DATALOG_RULES));
    assert_eq!(
        ok["ok"],
        true,
        "{}",
        ok.to_string().chars().take(300).collect::<String>()
    );
}

#[test]
fn datalog_body_atoms_over_cap_is_refused_and_at_cap_works() {
    let req = |n: usize| json!({"op": "datalog", "rules": [dead_rule(0, n)], "facts": [["urn:s", "urn:p", "urn:o"]]});
    assert_limit(
        &call_json(&req(MAX_BODY_ATOMS + 1)),
        "datalog_body_atoms",
        MAX_BODY_ATOMS + 1,
        MAX_BODY_ATOMS,
    );
    let ok = call_json(&req(MAX_BODY_ATOMS));
    assert_eq!(
        ok["ok"],
        true,
        "{}",
        ok.to_string().chars().take(300).collect::<String>()
    );
}

#[test]
fn datalog_fact_count_over_cap_is_refused_and_at_cap_works() {
    let req = |n: usize| {
        let facts: Vec<Value> = (0..n)
            .map(|i| json!(["urn:s", "urn:p", format!("urn:o{i}")]))
            .collect();
        json!({"op": "datalog", "rules": [dead_rule(0, 1)], "facts": facts})
    };
    assert_limit(
        &call_json(&req(MAX_DATALOG_FACTS + 1)),
        "datalog_facts",
        MAX_DATALOG_FACTS + 1,
        MAX_DATALOG_FACTS,
    );
    let ok = call_json(&req(MAX_DATALOG_FACTS));
    assert_eq!(
        ok["ok"],
        true,
        "{}",
        ok.to_string().chars().take(300).collect::<String>()
    );
}

#[test]
fn entail_input_quads_over_cap_is_refused_and_at_cap_works() {
    let req = |n: usize| {
        let nt: String = (0..n)
            .map(|i| format!("<urn:p:s> <urn:p:p> <urn:p:o{i}> .\n"))
            .collect();
        json!({"op": "entail", "regime": "simple", "data": {"text": nt, "dialect": "ntriples"}})
    };
    assert_limit(
        &call_json(&req(MAX_ENTAIL_QUADS + 1)),
        "entail_quads",
        MAX_ENTAIL_QUADS + 1,
        MAX_ENTAIL_QUADS,
    );
    let ok = call_json(&req(MAX_ENTAIL_QUADS));
    assert_eq!(
        ok["ok"],
        true,
        "{}",
        ok.to_string().chars().take(300).collect::<String>()
    );
}

fn tiny_policy_problem() -> Value {
    json!({
        "states": [{"id": "s0"}, {"id": "g", "facts": ["done"]}],
        "initial_states": ["s0"], "goal": {"facts": ["done"]},
        "transitions": [{"action": "go", "from": "s0", "to": "g", "probability_ppm": 1000000}]})
}

fn tiny_policy_entries() -> String {
    json!([{"state": "s0", "action": "go",
        "outcomes": [{"state": "g", "probability_ppm": 1000000}]}])
    .to_string()
}

#[test]
fn embedded_json_string_bytes_over_cap_is_refused_and_at_cap_works() {
    let body = tiny_policy_entries();
    let padded = |total: usize| format!("{}{body}", " ".repeat(total - body.len()));
    let req = |policy: String| json!({"op": "policy", "problem": tiny_policy_problem(), "policy": policy});
    assert_limit(
        &call_json(&req(padded(MAX_EMBEDDED_JSON_BYTES + 1))),
        "embedded_json_bytes",
        MAX_EMBEDDED_JSON_BYTES + 1,
        MAX_EMBEDDED_JSON_BYTES,
    );
    let ok = call_json(&req(padded(MAX_EMBEDDED_JSON_BYTES)));
    assert_eq!(
        ok["ok"],
        true,
        "{}",
        ok.to_string().chars().take(300).collect::<String>()
    );
}

#[test]
fn embedded_json_string_depth_over_cap_is_refused() {
    // Brackets inside a string are invisible to the outer depth scan.
    let deep = |d: usize| format!("{}{}", "[".repeat(d), "]".repeat(d));
    let req = |policy: String| json!({"op": "policy", "problem": tiny_policy_problem(), "policy": policy});
    assert_limit(
        &call_json(&req(deep(MAX_JSON_DEPTH + 1))),
        "embedded_json_depth",
        MAX_JSON_DEPTH + 1,
        MAX_JSON_DEPTH,
    );
    // at the cap the depth guard passes (the policy itself is then judged on its merits)
    let r = call_json(&req(deep(MAX_JSON_DEPTH)));
    assert_ne!(r["error"]["kind"], "ResourceLimit", "{r}");
}
