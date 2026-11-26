#![cfg(feature = "abi")]
//! Non-string `facts` / `unsafe_states` entries fail closed as Malformed.

use graphlaw::policy::{PolicyRefusalKind as K, admit};
use serde_json::{Value, json};

fn problem(facts: Value, unsafe_states: Value) -> Value {
    json!({
        "states": [{"id": "s0", "facts": facts}, {"id": "g", "facts": ["done"]}],
        "initial_states": ["s0"],
        "goal": {"facts": ["done"]},
        "unsafe_states": unsafe_states,
        "transitions": [
            {"action": "flip", "from": "s0", "to": "g"},
            {"action": "flip", "from": "s0", "to": "s0"}
        ]
    })
}

fn policy() -> Value {
    json!({"policy": [{"state": "s0", "action": "flip", "outcomes": [
        {"state": "g", "probability_ppm": 500000},
        {"state": "s0", "probability_ppm": 500000}
    ]}]})
}

fn run(p: Value) -> Result<graphlaw::policy::PolicyAdmitted, graphlaw::policy::PolicyRefused> {
    admit(&p.to_string(), &policy().to_string())
}

#[test]
fn all_string_lists_admit() {
    assert!(run(problem(json!(["a"]), json!(["nowhere"]))).is_ok());
}

#[test]
fn non_string_facts_refuse_malformed() {
    for bad in [json!(1), json!({"x": 1}), json!(null)] {
        let e = run(problem(json!(["a", bad]), json!([]))).unwrap_err();
        assert_eq!(e.kind, K::Malformed);
    }
}

#[test]
fn non_string_unsafe_states_refuse_malformed() {
    for bad in [json!(1), json!({"x": 1}), json!(null)] {
        let e = run(problem(json!(["a"]), json!(["nowhere", bad]))).unwrap_err();
        assert_eq!(e.kind, K::Malformed);
    }
}
