//! `Step::DeriveN3` admission gate: a carried RuleSet whose rule count differs
//! from the rules text is refused as `TierMismatch`; a matching one admits.

use graphlaw::dialect::{Dialect, RefusalKind};
use graphlaw::law::{LawError, LawState, Step};
use graphlaw::tier::validate_n3_rules;

const DATA: &str = "@prefix : <https://e/> .\n:a :p :b .\n";
const ONE_RULE: &str = "@prefix : <https://e/> .\n{ ?x :p ?y } => { ?y :q ?x } .\n";
const TWO_RULES: &str = "@prefix : <https://e/> .\n\
    { ?x :p ?y } => { ?y :q ?x } .\n{ ?x :p ?y } => { ?x :r ?y } .\n";

fn state() -> LawState {
    LawState::parse(DATA.as_bytes(), Dialect::Turtle, None).expect("data parses")
}

#[test]
fn mismatched_rule_count_is_tier_mismatch() {
    let carried = validate_n3_rules(ONE_RULE.as_bytes(), None).expect("one rule validates");
    let text_rules = validate_n3_rules(TWO_RULES.as_bytes(), None).expect("two rules validate");
    assert_ne!(carried.rule_count(), text_rules.rule_count());
    match state().transition(&Step::DeriveN3 {
        rules: TWO_RULES,
        ruleset: &carried,
    }) {
        Err(LawError::Refused(r)) => {
            assert_eq!(r.kind, RefusalKind::TierMismatch);
            assert_eq!(r.dialect, Some(Dialect::N3));
        }
        other => panic!("expected TierMismatch refusal, got {other:?}"),
    }
}

#[test]
fn matching_rule_count_still_admits() {
    let carried = validate_n3_rules(TWO_RULES.as_bytes(), None).expect("validates");
    let (_, receipt) = state()
        .transition(&Step::DeriveN3 {
            rules: TWO_RULES,
            ruleset: &carried,
        })
        .expect("matching ruleset admits");
    assert!(receipt.added >= 1);
}
