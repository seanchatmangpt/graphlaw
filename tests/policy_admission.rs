#![cfg(feature = "abi")]
//! Real-data falsifiers for FOND strong-cyclic policy admission (no mocks).

use graphlaw::policy::{PolicyRefusalKind as K, admit};
use serde_json::{Value, json};

fn retry_problem() -> Value {
    json!({
        "states": [{"id": "s0"}, {"id": "g", "facts": ["done"]}],
        "initial_states": ["s0"],
        "goal": {"facts": ["done"]},
        "transitions": [
            {"action": "flip", "from": "s0", "to": "g", "probability_ppm": 500000},
            {"action": "flip", "from": "s0", "to": "s0", "probability_ppm": 500000}
        ]
    })
}

fn retry_policy() -> Value {
    json!({"solved": true, "policy": [{
    "state": "s0", "action": "flip",
    "outcomes": [
        {"state": "g", "probability_ppm": 500000},
        {"state": "s0", "probability_ppm": 500000}
    ]}]})
}

fn run(
    p: &Value,
    pol: &Value,
) -> Result<graphlaw::policy::PolicyAdmitted, graphlaw::policy::PolicyRefused> {
    admit(&p.to_string(), &pol.to_string())
}

#[test]
fn ferroplan_retry_loop_policy_is_admitted() {
    let a = run(&retry_problem(), &retry_policy()).expect("admitted");
    assert_eq!(a.reachable, vec!["g", "s0"]);
    assert_eq!(a.goal_states, vec!["g"]);
    assert_eq!(a.entries, vec![("s0".to_string(), "flip".to_string())]);
    let nt = a.to_ntriples();
    assert!(nt.contains("STRONG_CYCLIC") && nt.contains("\"flip\""));
    assert_eq!(
        nt,
        run(&retry_problem(), &retry_policy())
            .unwrap()
            .to_ntriples()
    );
}

#[test]
fn bare_array_policy_is_admitted() {
    let pol = retry_policy()["policy"].clone();
    assert!(run(&retry_problem(), &pol).is_ok());
}

#[test]
fn deleted_entry_is_missing_entry() {
    let e = run(&retry_problem(), &json!({"policy": []})).unwrap_err();
    assert_eq!(e.kind, K::MissingEntry);
    assert_eq!(e.state, "s0");
}

#[test]
fn invented_outcome_is_refused() {
    let mut pol = retry_policy();
    pol["policy"][0]["outcomes"][0]["state"] = json!("s9");
    let e = run(&retry_problem(), &pol).unwrap_err();
    assert_eq!(e.kind, K::InventedOutcome);
    assert_eq!(e.action, "flip");
}

#[test]
fn skewed_probabilities_are_bad_mass() {
    let mut pol = retry_policy();
    pol["policy"][0]["outcomes"][0]["probability_ppm"] = json!(600000);
    let e = run(&retry_problem(), &pol).unwrap_err();
    assert_eq!(e.kind, K::BadMass);
}

#[test]
fn policy_that_loops_forever_is_dead_end() {
    // Policy always picks `stay`, whose only outcome is s0 itself; the goal is
    // reachable in the problem (via flip) but not under this policy.
    let mut p = retry_problem();
    p["transitions"]
        .as_array_mut()
        .unwrap()
        .push(json!({"action": "stay", "from": "s0", "to": "s0", "probability_ppm": 1000000}));
    let pol = json!({"policy": [{"state": "s0", "action": "stay",
        "outcomes": [{"state": "s0", "probability_ppm": 1000000}]}]});
    let e = run(&p, &pol).unwrap_err();
    assert_eq!(e.kind, K::DeadEnd);
    assert_eq!(e.state, "s0");
}

#[test]
fn malformed_json_and_duplicates_are_malformed() {
    assert_eq!(admit("{", "[]").unwrap_err().kind, K::Malformed);
    let mut pol = retry_policy();
    let dup = pol["policy"][0].clone();
    pol["policy"].as_array_mut().unwrap().push(dup);
    assert_eq!(run(&retry_problem(), &pol).unwrap_err().kind, K::Malformed);
}
