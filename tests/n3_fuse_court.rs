//! N3 inference-fuse surfacing court.
//!
//! A `{ premise } => false .` rule is a semantic verdict: when its premise
//! becomes provable mid-derivation, the run must surface as the typed
//! `RefusalKind::Fuse` refusal (`details.code "Fuse"` over the ABI), never
//! as a silent empty derivation (`Ok("")` — the engine reports `Complete`
//! with `derived` cleared, so without the typed arm the run looks like a
//! successful empty answer).
//!
//! Three rungs, all real engines, no mocks:
//!   1. pre-engine: `law::reason_n3_bounded` error kind only (no receipt);
//!   2. law step: `Step::DeriveN3` surfaces `LawError::Refused(Fuse)`;
//!   3. ABI op: `{"op":"n3"}` answers the refused envelope
//!      `error.kind == "Fuse"` + `error.details.code == "Fuse"`.
//!
//! Anti-vacuity, in-court: the same data + rule chain *without* the fuse
//! rule ADMITS the forbidden join (`:alice :invalidJoin true` derived), so
//! deleting the fuse rule from the fixture flips refuse→admit; and deleting
//! the `Fuse` arm from the engine flips the op court to red. Guards pin
//! that the fuse fires only when its premise is actually provable.

#![cfg(feature = "abi")]

use graphlaw::abi::call;
use graphlaw::dialect::{Dialect, RefusalKind};
use graphlaw::law::{LawState, N3Error, Step, reason_n3_bounded};
use graphlaw::tier::validate_n3_rules;
use serde_json::{Value, json};

const NS: &str = "http://example.org/n3-fuse/";

/// Data + rule chain deriving the forbidden join, plus the fuse rule
/// `{ ?q :invalidJoin true } => false .` (dispatch-specified shape).
fn fused_program() -> String {
    format!(
        "@prefix : <{NS}> .\n\
         :alice :parent :bob .\n\
         :bob :parent :carol .\n\
         {{ ?a :parent ?b . ?b :parent ?c }} => {{ ?a :invalidJoin true }} .\n\
         {{ ?q :invalidJoin true }} => false .\n"
    )
}

/// The same fixture minus the fuse rule: the flip pair.
fn unfused_program() -> String {
    format!(
        "@prefix : <{NS}> .\n\
         :alice :parent :bob .\n\
         :bob :parent :carol .\n\
         {{ ?a :parent ?b . ?b :parent ?c }} => {{ ?a :invalidJoin true }} .\n"
    )
}

/// Chain-incomplete data: no grandparent join derivable, fuse premise never
/// provable. The fuse must NOT fire (trigger semantics, not always-fire).
fn incomplete_program() -> String {
    format!(
        "@prefix : <{NS}> .\n\
         :alice :parent :bob .\n\
         {{ ?a :parent ?b . ?b :parent ?c }} => {{ ?a :invalidJoin true }} .\n\
         {{ ?q :invalidJoin true }} => false .\n"
    )
}

/// Data-only text for the `Step::DeriveN3` rung (state carries the data;
/// the rules text is the step argument).
fn data_text() -> String {
    format!(
        "@prefix : <{NS}> .\n\
         :alice :parent :bob .\n\
         :bob :parent :carol .\n"
    )
}

/// Rules text for the `Step::DeriveN3` rung: the join chain + the fuse.
fn fused_rules() -> String {
    format!(
        "@prefix : <{NS}> .\n\
         {{ ?a :parent ?b . ?b :parent ?c }} => {{ ?a :invalidJoin true }} .\n\
         {{ ?q :invalidJoin true }} => false .\n"
    )
}

/// The DeriveN3-rung rules minus the fuse rule: the flip pair at law level.
fn unfused_rules() -> String {
    format!(
        "@prefix : <{NS}> .\n\
         {{ ?a :parent ?b . ?b :parent ?c }} => {{ ?a :invalidJoin true }} .\n"
    )
}

fn run(req: Value) -> Value {
    serde_json::from_slice(&call(req.to_string().as_bytes())).expect("GraphLaw ABI returns JSON")
}

#[test]
fn fuse_fires_at_pre_engine_boundary() {
    match reason_n3_bounded(&fused_program()) {
        Err(N3Error::Fuse { report }) => {
            assert!(
                report.contains("# Inference fuse triggered."),
                "report must carry the fuse header, got: {report}"
            );
            assert!(
                report.contains("invalidJoin"),
                "report must name the fired premise, got: {report}"
           );
        }
        Err(other) => panic!("expected N3Error::Fuse, got {other:?}"),
        Ok(derived) => panic!("fuse must refuse, got silent derivation: {derived:?}"),
    }
}

#[test]
fn fuse_refuses_at_op_surface() {
    let out = run(json!({"op": "n3", "text": fused_program()}));
    assert_eq!(
        out["ok"], false,
        "fuse must refuse at the op surface (a silent empty derivation answers ok:true), got {out}"
    );
    assert_eq!(out["error"]["kind"], "Fuse", "typed kind, got {out}");
    assert_eq!(out["error"]["dialect"], "N3", "got {out}");
    assert_eq!(out["error"]["engine"], "PurRdf", "got {out}");
    assert_eq!(
        out["error"]["details"]["code"], "Fuse",
        "details.code must be the stable string code, got {out}"
    );
    let message = out["error"]["message"]
        .as_str()
        .expect("refusal carries a message");
    assert!(
        message.contains("# Inference fuse triggered."),
        "message must embed the fuse report, got: {message}"
    );
}

#[test]
fn derive_n3_step_refuses_fused_rules() {
    let rules = fused_rules();
    let ruleset = validate_n3_rules(rules.as_bytes(), None).expect("rules admit as N3");
    let state =
        LawState::parse(data_text().as_bytes(), Dialect::Turtle, None).expect("data admits");
    let outcome = state.transition(&Step::DeriveN3 {
        rules: &rules,
        ruleset: &ruleset,
    });
    match outcome {
        Err(graphlaw::law::LawError::Refused(refusal)) => {
            assert_eq!(refusal.kind, RefusalKind::Fuse, "got {refusal:?}");
            assert_eq!(refusal.dialect, Some(Dialect::N3), "got {refusal:?}");
            let message = refusal.message.as_str();
            assert!(
                message.contains("# Inference fuse triggered."),
                "message must embed the fuse report, got: {message}"
            );
        }
        Err(other) => panic!("expected LawError::Refused(Fuse), got {other:?}"),
        Ok(_) => panic!("fused rules must refuse the DeriveN3 step"),
    }
}

#[test]
fn without_fuse_rule_derivation_admits() {
    // Flip pair: identical chain minus the fuse rule admits, deriving the
    // forbidden join. Deleting the fuse rule from the fixture flips the
    // refuse verdicts above to admits.
    let out = run(json!({"op": "n3", "text": unfused_program()}));
    assert_eq!(out["ok"], true, "unfused program must admit, got {out}");
    let derived = out["derived"]
        .as_str()
        .expect("admitted derivation carries derived text");
    assert!(
        derived.contains("invalidJoin"),
        "the forbidden join is genuinely derived on the unfused fixture, got: {derived}"
    );

    // Law-step side of the flip: the same rules minus the fuse admit.
    let rules = unfused_rules();
    let ruleset = validate_n3_rules(rules.as_bytes(), None).expect("rules admit as N3");
    let state =
        LawState::parse(data_text().as_bytes(), Dialect::Turtle, None).expect("data admits");
    let (_next, receipt) = state
        .transition(&Step::DeriveN3 {
            rules: &rules,
            ruleset: &ruleset,
        })
        .expect("unfused DeriveN3 admits");
    assert!(receipt.added >= 1, "expected the join in the closure");
}

#[test]
fn fuse_does_not_fire_without_provable_premise() {
    let out = run(json!({"op": "n3", "text": incomplete_program()}));
    assert_eq!(
        out["ok"], true,
        "an unprovable fuse premise must not fire, got {out}"
    );
    let derived = out["derived"].as_str().expect("derived text");
    assert!(
        !derived.contains("invalidJoin"),
        "no join is derivable from incomplete data, got: {derived}"
    );
}
