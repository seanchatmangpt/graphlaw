//! Negative preconditions (`pre_not`) and negative goals (`goal_not`): a plan
//! that violates `(not p)` is refused at that step with the offending triple
//! named. Real states, real replay, no test doubles.

use graphlaw::dialect::Dialect;
use graphlaw::law::{LawError, LawState};
use graphlaw::plan::{Action, Plan, Triple};

const AT: &str = "<urn:p:at>";
const OCC: &str = "<urn:p:occupied>";

fn at(o: &str) -> String {
    format!("<urn:p:robot> {AT} <urn:p:{o}> .\n")
}

fn occupied(c: &str) -> String {
    format!("<urn:c:{c}> {OCC} <urn:v:yes> .\n")
}

fn state(text: &str) -> LawState {
    LawState::parse(text.as_bytes(), Dialect::NTriples, None).unwrap()
}

/// Move a -> b, allowed only when `b` is not occupied.
fn move_into_b() -> Plan {
    Plan {
        actions: vec![Action {
            name: "a-b".into(),
            pre: at("a"),
            pre_not: occupied("b"),
            add: at("b"),
            del: at("a"),
        }],
        goal: at("b"),
        ..Default::default()
    }
}

#[test]
fn a_move_into_an_occupied_cell_is_refused_naming_the_forbidden_triple() {
    let s = state(&format!("{}{}", at("a"), occupied("b")));
    match move_into_b().admit(&s) {
        Err(LawError::PlanRefused {
            index,
            action,
            missing,
            violated_absent,
        }) => {
            assert_eq!(index, 0);
            assert_eq!(action, "a-b");
            assert!(missing.is_empty());
            assert_eq!(violated_absent.len(), 1);
            assert!(
                violated_absent[0].contains("urn:c:b"),
                "{violated_absent:?}"
            );
            assert!(violated_absent[0].contains("urn:p:occupied"));
        }
        other => panic!("expected PlanRefused, got {other:?}"),
    }
    let msg = move_into_b().admit(&s).unwrap_err().to_string();
    assert!(msg.contains("forbidden"), "{msg}");
}

#[test]
fn the_same_move_with_the_cell_free_is_admitted() {
    let admitted = move_into_b().admit(&state(&at("a"))).unwrap();
    assert_eq!(admitted.receipts.len(), 1);
}

#[test]
fn unmet_goal_not_is_refused_after_the_last_action_and_met_is_admitted() {
    let mut p = move_into_b();
    p.actions[0].pre_not = String::new();
    p.goal_not = at("b"); // contradicts the goal effect: robot ends at b
    match p.admit(&state(&at("a"))) {
        Err(LawError::PlanRefused {
            index,
            action,
            violated_absent,
            ..
        }) => {
            assert_eq!(index, 1);
            assert_eq!(action, "<goal>");
            assert_eq!(violated_absent.len(), 1);
            assert!(violated_absent[0].contains("urn:p:b"));
        }
        other => panic!("expected PlanRefused at <goal>, got {other:?}"),
    }
    p.goal_not = at("a"); // robot left a: absent, admitted
    assert!(p.admit(&state(&at("a"))).is_ok());
}

/// Digest of the positive-only plan `a-b` computed independently from the
/// pre-negation canonical JSON (sha256 of the bytes, outside this crate).
const POSITIVE_DIGEST: &str =
    "sha256:6f39711c204fee70b841604e719937e44dd127a9895d3127efe23153065aa1cc";

#[test]
fn digest_of_an_existing_positive_plan_is_unchanged() {
    let mut p = move_into_b();
    p.actions[0].pre_not = String::new();
    // Same plan as the independent computation: N-Triples without trailing newlines.
    p.actions[0].pre = at("a").trim_end().into();
    p.actions[0].add = at("b").trim_end().into();
    p.actions[0].del = at("a").trim_end().into();
    p.goal = at("b").trim_end().into();
    assert_eq!(p.digest(), POSITIVE_DIGEST);
    assert!(!p.canonical_json().contains("pre_not"));
    assert!(!p.canonical_json().contains("goal_not"));
    // Non-empty negation changes the digest.
    assert_ne!(move_into_b().digest(), POSITIVE_DIGEST);
    let mut q = p.clone();
    q.goal_not = at("a");
    assert_ne!(q.digest(), POSITIVE_DIGEST);
}

#[test]
fn absence_is_tracked_through_state_not_checked_statically() {
    // Step 0 parks something in cell c (adds occupied(c)); step 1 requires
    // `not occupied(c)`. The start state is free, so a static check passes;
    // state tracking must refuse at step 1.
    let p = Plan {
        actions: vec![
            Action {
                name: "park".into(),
                add: occupied("c"),
                ..Default::default()
            },
            Action {
                name: "enter-c".into(),
                pre_not: occupied("c"),
                add: at("c"),
                ..Default::default()
            },
        ],
        ..Default::default()
    };
    match p.admit(&state(&at("a"))) {
        Err(LawError::PlanRefused {
            index,
            action,
            violated_absent,
            ..
        }) => {
            assert_eq!(index, 1);
            assert_eq!(action, "enter-c");
            assert!(violated_absent[0].contains("urn:c:c"));
        }
        other => panic!("expected refusal at step 1, got {other:?}"),
    }
}

#[test]
fn a_delete_earlier_makes_a_later_not_precondition_hold() {
    let p = Plan {
        actions: vec![
            Action {
                name: "vacate".into(),
                del: occupied("b"),
                ..Default::default()
            },
            Action {
                name: "enter-b".into(),
                pre_not: occupied("b"),
                add: at("b"),
                ..Default::default()
            },
        ],
        ..Default::default()
    };
    assert!(p.admit(&state(&occupied("b"))).is_ok());
}

#[test]
fn builders_express_negation() {
    let t = |s: &str, p: &str, o: &str| Triple::iri(s, p, o).unwrap();
    let built = Plan::builder()
        .action("a-b")
        .requires(t("urn:p:robot", "urn:p:at", "urn:p:a"))
        .requires_not(t("urn:c:b", "urn:p:occupied", "urn:v:yes"))
        .adds(t("urn:p:robot", "urn:p:at", "urn:p:b"))
        .deletes(t("urn:p:robot", "urn:p:at", "urn:p:a"))
        .goal(t("urn:p:robot", "urn:p:at", "urn:p:b"))
        .goal_not(t("urn:p:robot", "urn:p:at", "urn:p:a"))
        .build()
        .unwrap();
    let trim = |s: String| s.trim_end().to_string();
    let mut by_hand = move_into_b();
    by_hand.actions[0].pre = trim(at("a"));
    by_hand.actions[0].pre_not = trim(occupied("b"));
    by_hand.actions[0].add = trim(at("b"));
    by_hand.actions[0].del = trim(at("a"));
    by_hand.goal = trim(at("b"));
    by_hand.goal_not = trim(at("a"));
    assert_eq!(built.digest(), by_hand.digest());
    assert_eq!(built.actions[0].pre_not, trim(occupied("b")));
    let err = Plan::builder()
        .requires_not(t("urn:c:b", "urn:p:occupied", "urn:v:yes"))
        .build()
        .unwrap_err();
    assert!(err.to_string().contains("requires_not"), "{err}");
}

#[test]
fn blank_nodes_in_pre_not_are_refused() {
    let mut p = move_into_b();
    p.actions[0].pre_not = "_:x <urn:p:occupied> <urn:v:yes> .".into();
    assert!(matches!(
        p.admit(&state(&at("a"))),
        Err(LawError::Refused(_))
    ));
}

#[cfg(feature = "abi")]
mod abi {
    use super::{at, occupied};
    use graphlaw::abi::call_json;
    use serde_json::json;

    pub fn req(start: &str) -> serde_json::Value {
        json!({"op": "law", "data": {"text": start, "dialect": "ntriples"},
            "steps": [{"step": "plan", "plan": {
                "actions": [{"name": "a-b", "pre": at("a"), "pre_not": occupied("b"),
                             "add": at("b"), "del": at("a")}],
                "goal": at("b"), "goal_not": at("a")}}]})
    }

    #[test]
    fn abi_details_carry_violated_absent_and_free_cell_is_admitted() {
        let r = call_json(&req(&format!("{}{}", at("a"), occupied("b"))));
        assert_eq!(r["ok"], false, "{r}");
        let d = &r["error"]["details"];
        assert_eq!(d["code"], "PlanRefused");
        assert_eq!(d["index"], 0);
        assert_eq!(d["unmet"].as_array().unwrap().len(), 0);
        let v = d["violated_absent"].as_array().unwrap();
        assert_eq!(v.len(), 1);
        assert!(v[0].as_str().unwrap().contains("urn:c:b"));
        assert!(
            r["error"]["message"]
                .as_str()
                .unwrap()
                .contains("forbidden")
        );
        assert_eq!(call_json(&req(&at("a")))["ok"], true);
    }
}
