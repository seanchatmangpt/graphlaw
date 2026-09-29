#![cfg(feature = "abi")]
//! G1: every refusal from the `law`/`policy` ops carries machine-readable
//! `error.details` next to the unchanged `kind/engine/dialect/message`.
//! Real SHACL engine, real plan replay, real leases (no mocks).

use graphlaw::abi::call_json;
use serde_json::{Value, json};

pub const SHAPES: &str = "@prefix sh: <http://www.w3.org/ns/shacl#> .\n\
@prefix ex: <https://e/> .\n\
ex:S a sh:NodeShape ; sh:targetClass ex:T ;\n\
  sh:property [ sh:path ex:name ; sh:minCount 1 ] .\n";

pub const TWO_BAD: &str = "<https://e/a> a <https://e/T> .\n<https://e/b> a <https://e/T> .\n";

fn at(o: &str) -> String {
    format!("<urn:p:robot> <urn:p:at> <urn:p:{o}> .\n")
}

fn err_of(req: Value) -> Value {
    let r = call_json(&req);
    assert_eq!(r["ok"], false, "{r}");
    r["error"].clone()
}

#[test]
fn shacl_refusal_carries_one_structured_entry_per_violation() {
    let e = err_of(json!({"op": "law",
        "data": {"text": TWO_BAD, "dialect": "turtle"},
        "steps": [{"step": "shacl", "shapes": SHAPES}]}));
    // compatibility: legacy fields unchanged
    assert_eq!(e["kind"], "EngineRejected");
    assert_eq!(e["engine"], "PurRdf");
    assert_eq!(e["dialect"], "Turtle");
    assert_eq!(e["message"], "SHACL admission refused: 2 violation(s)");
    let d = &e["details"];
    assert_eq!(d["code"], "NotAdmitted");
    let vs = d["violations"].as_array().unwrap();
    assert_eq!(vs.len(), 2, "{d}");
    let mut focus: Vec<&str> = vs.iter().map(|v| v["focus"].as_str().unwrap()).collect();
    focus.sort();
    assert!(focus[0].contains("https://e/a") && focus[1].contains("https://e/b"));
    for v in vs {
        assert!(
            v["path"].as_str().unwrap().contains("https://e/name"),
            "{v}"
        );
        assert!(v["component"].as_str().unwrap().contains("MinCount"), "{v}");
        assert!(v["message"].is_string() && v["severity"].is_string(), "{v}");
    }
}

#[test]
fn rust_variant_keeps_count_and_results_in_agreement() {
    use graphlaw::dialect::Dialect;
    use graphlaw::law::{LawError, LawState, Step};
    let s = LawState::parse(TWO_BAD.as_bytes(), Dialect::Turtle, None).unwrap();
    match s.transition(&Step::AdmitShacl { shapes_ttl: SHAPES }) {
        Err(LawError::NotAdmitted {
            violations,
            results,
        }) => {
            assert_eq!(violations, 2);
            assert_eq!(results.len(), violations);
            assert!(results.iter().all(|r| r.path.is_some()));
        }
        other => panic!("expected NotAdmitted, got {other:?}"),
    }
}

#[test]
fn plan_refusal_carries_index_action_and_unmet() {
    let e = err_of(json!({"op": "law",
        "data": {"text": at("a"), "dialect": "ntriples"},
        "steps": [{"step": "plan", "plan": {
            "actions": [{"name": "a-b", "pre": at("a"), "add": at("b"), "del": at("a")},
                        {"name": "b-c", "pre": at("z"), "add": at("c"), "del": at("b")}],
            "goal": at("c")}}]}));
    let d = &e["details"];
    assert_eq!(d["code"], "PlanRefused");
    assert_eq!(d["index"], 1);
    assert_eq!(d["action"], "b-c");
    let unmet = d["unmet"].as_array().unwrap();
    assert_eq!(unmet.len(), 1);
    assert!(unmet[0].as_str().unwrap().contains("urn:p:z"));
    assert!(
        e["message"]
            .as_str()
            .unwrap()
            .contains("plan refused at step 1")
    );
}

#[test]
fn policy_refusal_carries_kind_state_action() {
    let problem = json!({
        "states": [{"id": "s0"}, {"id": "g", "facts": ["done"]}],
        "initial_states": ["s0"], "goal": {"facts": ["done"]},
        "transitions": [
            {"action": "flip", "from": "s0", "to": "g", "probability_ppm": 500000},
            {"action": "flip", "from": "s0", "to": "s0", "probability_ppm": 500000}]});
    let policy = json!({"policy": [{"state": "s0", "action": "flip", "outcomes": [
        {"state": "g", "probability_ppm": 400000},
        {"state": "s0", "probability_ppm": 500000}]}]});
    let e = err_of(json!({"op": "policy", "problem": problem, "policy": policy}));
    let d = &e["details"];
    assert_eq!(d["code"], "PolicyRefused");
    assert_eq!(d["policy_kind"], "BadMass");
    assert_eq!(d["state"], "s0");
    assert_eq!(d["action"], "flip");
    assert!(
        e["message"]
            .as_str()
            .unwrap()
            .contains("policy refused (BadMass)")
    );
}

#[test]
fn lease_refusal_carries_reason_lease_id_step() {
    let req = |scope: Value, now: u64| {
        json!({"op": "law",
            "data": {"text": "<urn:a:x> <urn:a:p> <urn:a:y> .\n", "dialect": "ntriples"},
            "lease": {"id": "L9", "holder": "h", "ceiling": "construct", "scope": scope,
                      "expires_unix": 100},
            "now_unix": now, "unverified_lease": true,
            "steps": [{"step": "rdfs"}]})
    };
    for (r, reason) in [
        (req(json!(["derive:rdfs"]), 100), "expired"),
        (req(json!(["derive:n3"]), 1), "out_of_scope"),
    ] {
        let d = err_of(r)["details"].clone();
        assert_eq!(d["code"], "LeaseRefused");
        assert_eq!(d["reason"], reason);
        assert_eq!(d["lease_id"], "L9");
        assert_eq!(d["step"], "derive:rdfs");
    }
}

#[test]
fn receipt_required_carries_step() {
    let e = err_of(json!({"op": "law",
        "data": {"text": "<urn:a:x> <urn:a:p> <urn:a:y> .\n", "dialect": "ntriples"},
        "steps": [{"step": "require-receipt", "step_name": "derive:rdfs"}]}));
    assert_eq!(
        e["details"],
        json!({"code": "ReceiptRequired", "step": "derive:rdfs"})
    );
}

#[test]
fn every_law_error_variant_has_a_code() {
    // Engine-level refusals (unparseable N3 rules) inside `law` still get a code.
    let e = err_of(json!({"op": "law",
        "data": {"text": "<urn:a:x> <urn:a:p> <urn:a:y> .\n", "dialect": "ntriples"},
        "steps": [{"step": "n3", "rules": "{ ?x ?p"}]}));
    assert_eq!(e["details"]["code"], "Refused");
}
