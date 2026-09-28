//! Receipt feedback (R_t -> O*_{t+1}) and plan-digest content. Real states,
//! real receipts, no test doubles.

use graphlaw::dialect::Dialect;
use graphlaw::law::{LawError, LawState, Step};
use graphlaw::plan::{Action, Plan};
use graphlaw::receipt;

fn base() -> LawState {
    LawState::parse(
        b"<urn:a:C> <http://www.w3.org/2000/01/rdf-schema#subClassOf> <urn:a:D> .\n\
          <urn:a:x> <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <urn:a:C> .\n",
        Dialect::NTriples,
        None,
    )
    .unwrap()
}

const GATE: Step = Step::RequireReceipt {
    step: "derive:rdfs",
};

#[test]
fn second_admission_refused_without_receipt_admitted_with_it() {
    let (s1, r) = base().transition(&Step::EntailRdfs).unwrap();
    match s1.transition(&GATE) {
        Err(LawError::ReceiptRequired { step }) => assert_eq!(step, "derive:rdfs"),
        other => panic!("expected ReceiptRequired, got {other:?}"),
    }
    let recorded = receipt::record(&s1, &r).unwrap();
    let (same, gate_receipt) = recorded.transition(&GATE).expect("receipt recorded");
    assert_eq!(same.id(), recorded.id(), "gate never changes state");
    assert_eq!(gate_receipt.step, "admit:require-receipt");
}

#[test]
fn receipt_for_a_different_step_does_not_satisfy_the_gate() {
    let (s1, r) = base().transition(&Step::EntailRdfs).unwrap();
    let recorded = receipt::record(&s1, &r).unwrap();
    let other = Step::RequireReceipt {
        step: "derive:owl-rl",
    };
    assert!(matches!(
        recorded.transition(&other),
        Err(LawError::ReceiptRequired { .. })
    ));
}

#[test]
fn recorded_receipt_content_changes_the_state_id() {
    let (s1, r) = base().transition(&Step::EntailRdfs).unwrap();
    let a = receipt::record(&s1, &r).unwrap();
    let mut r2 = r.clone();
    r2.added += 1;
    let b = receipt::record(&s1, &r2).unwrap();
    assert_ne!(a.id(), b.id());
    assert_ne!(a.id(), s1.id());
    let again = receipt::record(&s1, &r).unwrap();
    assert_eq!(a.id(), again.id(), "recording is deterministic");
}

fn plan(to: &str) -> Plan {
    let t = |o: &str| format!("<urn:p:robot> <urn:p:at> <urn:p:{o}> .\n");
    Plan {
        actions: vec![Action {
            name: "a-b".into(),
            pre: t("a"),
            add: t(to),
            del: t("a"),
        }],
        goal: t(to),
    }
}

#[test]
fn plan_digest_is_stable_and_sensitive_to_actions_and_goal() {
    let start = LawState::parse(
        b"<urn:p:robot> <urn:p:at> <urn:p:a> .\n",
        Dialect::NTriples,
        None,
    )
    .unwrap();
    let a = plan("b").admit(&start).unwrap();
    let b = plan("b").admit(&start).unwrap();
    assert_eq!(a.plan_digest, b.plan_digest);
    assert_eq!(a.plan_digest, plan("b").digest());
    assert!(a.plan_digest.starts_with("sha256:") && a.plan_digest.len() == 71);
    let mutated = plan("c");
    assert_ne!(a.plan_digest, mutated.digest());
    let mut goal_only = plan("b");
    goal_only.goal = String::new();
    assert_ne!(plan("b").digest(), goal_only.digest());
}

#[cfg(feature = "abi")]
mod abi {
    use graphlaw::abi::{ABI_VERSION, call_json};
    use serde_json::json;

    const NT: &str = "<urn:a:C> <http://www.w3.org/2000/01/rdf-schema#subClassOf> <urn:a:D> .\n";

    fn law(steps: serde_json::Value) -> serde_json::Value {
        call_json(
            &json!({"op": "law", "data": {"text": NT, "dialect": "ntriples"}, "steps": steps}),
        )
    }

    #[test]
    fn abi_record_then_require_admits_and_bare_require_refuses() {
        let with = law(json!([
            {"step": "rdfs"},
            {"step": "record-receipts"},
            {"step": "require-receipt", "step_name": "derive:rdfs"}
        ]));
        assert_eq!(with["ok"], true, "{with}");
        let without = law(json!([
            {"step": "rdfs"},
            {"step": "require-receipt", "step_name": "derive:rdfs"}
        ]));
        assert_eq!(without["ok"], false);
        assert!(
            without["error"]["message"]
                .as_str()
                .unwrap()
                .contains("receipt required")
        );
    }

    #[test]
    fn abi_plan_receipts_carry_digest_and_index() {
        let at = |o: &str| format!("<urn:p:robot> <urn:p:at> <urn:p:{o}> .\n");
        let run = |to: &str| {
            call_json(
                &json!({"op": "law", "data": {"text": at("a"), "dialect": "ntriples"},
                "steps": [{"step": "plan", "plan": {
                    "actions": [{"name": "m", "pre": at("a"), "add": at(to), "del": at("a")}],
                    "goal": at(to)}}]}),
            )
        };
        let (x, x2, y) = (run("b"), run("b"), run("c"));
        assert_eq!(x["ok"], true, "{x}");
        assert_eq!(x["receipts"][0]["index"], 0);
        assert_eq!(
            x["receipts"][0]["plan_sha256"],
            x2["receipts"][0]["plan_sha256"]
        );
        assert_ne!(
            x["receipts"][0]["plan_sha256"],
            y["receipts"][0]["plan_sha256"]
        );
    }

    #[test]
    fn capabilities_report_abi_version() {
        let c = call_json(&json!({"op": "capabilities"}));
        assert_eq!(c["abi_version"], ABI_VERSION);
    }
}
