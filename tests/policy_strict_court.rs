#![cfg(feature = "abi")]
//! Strict policy parsing: missing / wrong-typed required arrays and over-cap
//! inputs refuse Malformed instead of being silently ignored.

use graphlaw::policy::{Entry, Outcome, PolicyRefusalKind as K, Problem, admit, admit_parsed};
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};

fn problem() -> Value {
    json!({
        "states": [{"id": "s0"}, {"id": "g", "facts": ["done"]}],
        "initial_states": ["s0"],
        "goal": {"facts": ["done"]},
        "unsafe_states": ["nowhere"],
        "transitions": [{"action": "go", "from": "s0", "to": "g"}]
    })
}

fn policy() -> Value {
    json!({"policy": [{"state": "s0", "action": "go",
        "outcomes": [{"state": "g", "probability_ppm": 1000000}]}]})
}

fn run(p: &Value, pol: &Value) -> Result<(), K> {
    admit(&p.to_string(), &pol.to_string()).map(|_| ()).map_err(|e| e.kind)
}

#[test]
fn baseline_admits() {
    assert_eq!(run(&problem(), &policy()), Ok(()));
}

#[test]
fn wrong_typed_unsafe_states_refuses() {
    for bad in [json!("s0"), json!({"s0": 1}), json!(7)] {
        let mut p = problem();
        p["unsafe_states"] = bad;
        assert_eq!(run(&p, &policy()), Err(K::Malformed));
    }
}

#[test]
fn missing_or_wrong_typed_required_arrays_refuse() {
    for key in ["states", "initial_states", "transitions"] {
        let mut p = problem();
        p.as_object_mut().unwrap().remove(key);
        assert_eq!(run(&p, &policy()), Err(K::Malformed), "missing {key}");
        let mut p = problem();
        p[key] = json!("x");
        assert_eq!(run(&p, &policy()), Err(K::Malformed), "typed {key}");
    }
    let mut p = problem();
    p["goal"] = json!({"facts": "done"});
    assert_eq!(run(&p, &policy()), Err(K::Malformed));
    let mut p = problem();
    p.as_object_mut().unwrap().remove("goal");
    assert_eq!(run(&p, &policy()), Err(K::Malformed));
    let mut p = problem();
    p["states"][0]["facts"] = json!("done");
    assert_eq!(run(&p, &policy()), Err(K::Malformed));
}

#[test]
fn missing_or_wrong_typed_outcomes_refuse() {
    for bad in [None, Some(json!("g")), Some(json!({}))] {
        let mut pol = policy();
        match bad {
            None => {
                pol["policy"][0].as_object_mut().unwrap().remove("outcomes");
            }
            Some(v) => pol["policy"][0]["outcomes"] = v,
        }
        assert_eq!(run(&problem(), &pol), Err(K::Malformed));
    }
}

#[test]
fn explicit_null_optional_arrays_are_absent() {
    let mut p = problem();
    p["unsafe_states"] = json!(null);
    assert_eq!(run(&p, &policy()), Ok(()));
    let mut p = problem();
    p["states"][1]["facts"] = json!(null);
    // `g` loses its goal fact, so it is no longer a goal: the policy dead-ends
    // at a missing entry instead of being refused as Malformed.
    assert_eq!(run(&p, &policy()), Err(K::MissingEntry));
    let mut p = problem();
    p["states"][0]["facts"] = json!(null);
    assert_eq!(run(&p, &policy()), Ok(()));
}

/// Problem whose initial state is already a goal: admits with any number of
/// unreferenced policy entries / declared states / transitions, so only the
/// cap check can refuse.
fn goal_problem() -> Problem {
    Problem {
        states: BTreeMap::from([("s0".to_string(), BTreeSet::from(["done".to_string()]))]),
        initial: vec!["s0".into()],
        goal_facts: BTreeSet::from(["done".to_string()]),
        unsafe_states: BTreeSet::new(),
        transitions: BTreeSet::new(),
    }
}

fn cap_refusal(problem: &Problem, policy: &[Entry], what: &str) {
    let err = admit_parsed(problem, policy).unwrap_err();
    assert_eq!(err.kind, K::Malformed);
    assert!(
        err.message.contains(what) && err.message.contains("exceeds cap"),
        "must be the cap refusal, got: {}",
        err.message
    );
}

#[test]
fn over_cap_policy_entries_refuse_only_on_cap() {
    let cap = graphlaw::abi::MAX_POLICY_ENTRIES;
    let entries = |n: usize| -> Vec<Entry> {
        (0..n)
            .map(|i| Entry {
                state: format!("e{i}"),
                action: "a".into(),
                outcomes: vec![Outcome { state: "s0".into(), probability_ppm: 1_000_000 }],
            })
            .collect()
    };
    admit_parsed(&goal_problem(), &entries(cap)).expect("exactly cap admits");
    cap_refusal(&goal_problem(), &entries(cap + 1), "policy entries");
}

#[test]
fn over_cap_states_refuse_only_on_cap() {
    let cap = graphlaw::abi::MAX_POLICY_ENTRIES;
    let with_states = |n: usize| {
        let mut p = goal_problem();
        for i in 0..n - 1 {
            p.states.insert(format!("x{i}"), BTreeSet::new());
        }
        p
    };
    assert_eq!(with_states(cap).states.len(), cap);
    admit_parsed(&with_states(cap), &[]).expect("exactly cap admits");
    cap_refusal(&with_states(cap + 1), &[], "states");
}

#[test]
fn over_cap_transitions_refuse_only_on_cap() {
    let cap = graphlaw::abi::MAX_POLICY_ENTRIES;
    let with_transitions = |n: usize| {
        let mut p = goal_problem();
        for i in 0..n {
            p.transitions.insert(("a".into(), "s0".into(), format!("t{i}")));
        }
        p
    };
    admit_parsed(&with_transitions(cap), &[]).expect("exactly cap admits");
    cap_refusal(&with_transitions(cap + 1), &[], "transitions");
}

#[test]
fn cap_mirrors_abi_constant() {
    assert_eq!(graphlaw::abi::MAX_POLICY_ENTRIES, 100_000);
}
