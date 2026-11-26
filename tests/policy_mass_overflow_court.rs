#![cfg(feature = "abi")]
//! Mass sum must be checked: u64 overflow refuses BadMass, never wraps.

use graphlaw::policy::{PolicyRefusalKind as K, admit};
use serde_json::{Value, json};

fn problem() -> Value {
    json!({
        "states": [{"id": "s0"}, {"id": "g", "facts": ["done"]}],
        "initial_states": ["s0"],
        "goal": {"facts": ["done"]},
        "transitions": [
            {"action": "flip", "from": "s0", "to": "g"},
            {"action": "flip", "from": "s0", "to": "s0"}
        ]
    })
}

fn policy(a: u64, b: u64) -> Value {
    json!({"policy": [{"state": "s0", "action": "flip", "outcomes": [
        {"state": "g", "probability_ppm": a},
        {"state": "s0", "probability_ppm": b}
    ]}]})
}

#[test]
fn overflowing_mass_refuses_bad_mass() {
    let h = u64::MAX / 2 + 1;
    let e = admit(&problem().to_string(), &policy(h, h).to_string()).unwrap_err();
    assert_eq!(e.kind, K::BadMass);
}

#[test]
fn exact_mass_still_admits() {
    assert!(admit(&problem().to_string(), &policy(500_000, 500_000).to_string()).is_ok());
}
