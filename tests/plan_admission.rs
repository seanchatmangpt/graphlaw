//! Independent plan admission: a planner proposes, `Plan::admit` replays.
//! Real states and real receipts throughout; no test doubles.

use graphlaw::dialect::Dialect;
use graphlaw::law::{LawError, LawState, Step};
use graphlaw::plan::{Action, Plan};

const AT: &str = "<urn:p:at>";

fn t(s: &str, o: &str) -> String {
    format!("<urn:p:{s}> {AT} <urn:p:{o}> .\n")
}

fn start() -> LawState {
    LawState::parse(t("robot", "a").as_bytes(), Dialect::NTriples, None).unwrap()
}

fn mv(name: &str, from: &str, to: &str) -> Action {
    Action {
        name: name.into(),
        pre: t("robot", from),
        add: t("robot", to),
        del: t("robot", from),
    }
}

fn plan() -> Plan {
    Plan {
        actions: vec![
            mv("a-b", "a", "b"),
            mv("b-c", "b", "c"),
            mv("c-d", "c", "d"),
        ],
        goal: t("robot", "d"),
    }
}

#[test]
fn valid_plan_reaches_goal_with_one_receipt_per_action() {
    let s = start();
    let a = plan().admit(&s).expect("plan is valid");
    assert_eq!(a.receipts.len(), 3);
    assert_eq!(a.receipts[0].parent, s.id());
    assert_eq!(a.receipts[2].child, a.state.id());
    for w in a.receipts.windows(2) {
        assert_eq!(w[0].child, w[1].parent, "receipts chain state to state");
    }
    let expect = LawState::parse(t("robot", "d").as_bytes(), Dialect::NTriples, None).unwrap();
    assert_eq!(a.state.id(), expect.id());
}

#[test]
fn replay_is_byte_identical() {
    let a = plan().admit(&start()).unwrap();
    let b = plan().admit(&start()).unwrap();
    assert_eq!(a.receipts, b.receipts);
    assert_eq!(a.state.id(), b.state.id());
}

#[test]
fn falsifier_missing_precondition_in_middle_action_is_refused_at_that_step() {
    let mut p = plan();
    p.actions[1].pre = t("robot", "z"); // robot is never at z
    match p.admit(&start()) {
        Err(LawError::PlanRefused {
            index,
            action,
            missing,
        }) => {
            assert_eq!(index, 1);
            assert_eq!(action, "b-c");
            assert_eq!(missing.len(), 1);
            assert!(missing[0].contains("urn:p:z"));
        }
        other => panic!("expected PlanRefused at 1, got {other:?}"),
    }
}

#[test]
fn unmet_goal_is_refused_after_the_last_action() {
    let mut p = plan();
    p.goal = t("robot", "elsewhere");
    match p.admit(&start()) {
        Err(LawError::PlanRefused { index, action, .. }) => {
            assert_eq!(index, 3);
            assert_eq!(action, "<goal>");
        }
        other => panic!("expected goal refusal, got {other:?}"),
    }
}

#[test]
fn blank_node_atoms_are_refused_not_admitted() {
    let mut p = plan();
    p.actions[0].add = "_:x <urn:p:at> <urn:p:b> .\n".into();
    assert!(matches!(p.admit(&start()), Err(LawError::Refused(_))));
}

#[test]
fn step_plan_transition_matches_admit() {
    let p = plan();
    let (child, r) = start().transition(&Step::Plan { plan: &p }).unwrap();
    assert_eq!(child.id(), p.admit(&start()).unwrap().state.id());
    assert_eq!(r.step, "admit:plan");
}

#[test]
fn deleting_an_absent_triple_is_a_no_op() {
    let mut p = plan();
    p.actions[0].del.push_str(&t("ghost", "nowhere"));
    assert!(p.admit(&start()).is_ok());
}
