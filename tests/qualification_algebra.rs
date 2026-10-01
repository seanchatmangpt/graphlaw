#![cfg(feature = "abi")]

use graphlaw::qualification::{
    Observation, Standing, compose_courts, evaluate, generate_falsifier, temporal_rule_holds,
};
use serde_json::{Value, json};

fn parsed(text: &str) -> Value {
    serde_json::from_str(text).expect("fixture is valid JSON")
}

#[test]
fn anti_vacuity_requires_an_observed_attack() {
    assert_eq!(
        evaluate(Observation {
            attempt_observed: false,
            violation_observed: false,
            independent_observer: true,
            postconditions_satisfied: true,
            durable_evidence: true,
            crash_observed: false,
            crash_reconciled: false,
        }),
        Standing::Unknown
    );

    assert_eq!(
        evaluate(Observation {
            attempt_observed: true,
            violation_observed: true,
            independent_observer: true,
            postconditions_satisfied: false,
            durable_evidence: true,
            crash_observed: false,
            crash_reconciled: false,
        }),
        Standing::Refused
    );
}

#[test]
fn observer_and_crash_reconciliation_are_load_bearing() {
    let independent_missing = Observation {
        attempt_observed: true,
        violation_observed: false,
        independent_observer: false,
        postconditions_satisfied: true,
        durable_evidence: true,
        crash_observed: false,
        crash_reconciled: false,
    };
    assert_eq!(evaluate(independent_missing), Standing::Unknown);

    let transient_only = Observation {
        attempt_observed: true,
        violation_observed: false,
        independent_observer: true,
        postconditions_satisfied: true,
        durable_evidence: false,
        crash_observed: false,
        crash_reconciled: false,
    };
    assert_eq!(evaluate(transient_only), Standing::Unknown);

    let crash_unknown = Observation {
        attempt_observed: true,
        violation_observed: false,
        independent_observer: true,
        postconditions_satisfied: true,
        durable_evidence: true,
        crash_observed: true,
        crash_reconciled: false,
    };
    assert_eq!(evaluate(crash_unknown), Standing::Unknown);

    assert_eq!(
        evaluate(Observation {
            crash_reconciled: true,
            ..crash_unknown
        }),
        Standing::Qualified
    );
}

#[test]
fn semantic_laws_generate_adversarial_test_families() {
    let cases = [
        (
            include_str!("../qualification/laws/exact_subject.json"),
            json!({"subject_digest": format!("sha256:{}", "a".repeat(64))}),
        ),
        (
            include_str!("../qualification/laws/collaborator_identity.json"),
            json!({"collaborator_digest": "urn:chicago:real-collaborator"}),
        ),
        (
            include_str!("../qualification/laws/replay_identity.json"),
            json!({"replay_digest": format!("sha256:{}", "b".repeat(64))}),
        ),
        (
            include_str!("../qualification/laws/attempt_presence.json"),
            json!({"attempt_observed": true}),
        ),
        (
            include_str!("../qualification/laws/temporal_order.json"),
            json!({"event_order": ["prepare", "actuate"]}),
        ),
        (
            include_str!("../qualification/laws/authority_ceiling.json"),
            json!({"authority": "NONE"}),
        ),
        (
            include_str!("../qualification/laws/observer_independence.json"),
            json!({"observer_independent": true}),
        ),
        (
            include_str!("../qualification/laws/crash_reconciliation.json"),
            json!({"reconciled": true}),
        ),
        (
            include_str!("../qualification/laws/bounded_autonomy.json"),
            json!({"autonomy_limit": 8, "autonomy_steps": 8}),
        ),
    ];

    for (law_text, seed) in cases {
        let law = parsed(law_text);
        let attack = generate_falsifier(&law, &seed).expect("supported law mutation");
        assert_eq!(attack["authority"], "NONE");
        assert_eq!(attack["consequence"], "EVIDENCE_ONLY");
        assert_eq!(attack["law"], law["law"]);
        assert_eq!(attack["expected"], law["falsifier"]["expect"]);
        assert_ne!(attack["candidate"], seed);
    }
}

#[test]
fn fibo_is_a_monotone_child_profile_not_a_parallel_test_system() {
    let universal = parsed(include_str!("../qualification/profiles/universal.json"));
    let fibo = parsed(include_str!(
        "../qualification/profiles/fibo_high_value_finance.json"
    ));
    let effective = compose_courts(&universal, &fibo).expect("valid inheritance");

    for court in universal["courts"].as_array().expect("universal courts") {
        assert!(
            effective.contains(court.as_str().expect("court id")),
            "FIBO dropped inherited court {court}"
        );
    }
    for finance_court in [
        "transaction_identity",
        "amount_currency_integrity",
        "counterparty_identity",
        "authorization_ceiling",
        "settlement_reconciliation",
    ] {
        assert!(effective.contains(finance_court));
    }
}

#[test]
fn universal_profile_has_executable_court_assets_for_every_general_rule() {
    let fixtures = [
        include_str!("../qualification/courts/exact_subject.json"),
        include_str!("../qualification/courts/real_collaborator.json"),
        include_str!("../qualification/courts/anti_vacuity.json"),
        include_str!("../qualification/courts/independent_observer.json"),
        include_str!("../qualification/courts/temporal_order.json"),
        include_str!("../qualification/courts/identity_mutation.json"),
        include_str!("../qualification/courts/differential.json"),
        include_str!("../qualification/courts/bounded_autonomy.json"),
        include_str!("../qualification/courts/crash_reconcile.json"),
        include_str!("../qualification/courts/replay.json"),
    ];
    let names: std::collections::BTreeSet<_> = fixtures
        .into_iter()
        .map(parsed)
        .map(|fixture| fixture["court"].as_str().expect("court").to_owned())
        .collect();

    let profile = parsed(include_str!("../qualification/profiles/universal.json"));
    for court in profile["courts"].as_array().expect("universal courts") {
        assert!(names.contains(court.as_str().expect("court id")));
    }
}

#[test]
fn ocel_temporal_order_is_attacked_not_merely_asserted() {
    let ocel = parsed(include_str!(
        "../qualification/ocel/chicago_ocel_profile.json"
    ));
    let events = [
        "attempt_started",
        "mutation_applied",
        "observer_sampled",
        "attempt_completed",
    ];

    for rule in ocel["ordering"].as_array().expect("ordering rules") {
        let rule = rule.as_str().expect("ordering rule");
        assert!(temporal_rule_holds(&events, rule));
    }

    let swapped = [
        "attempt_started",
        "observer_sampled",
        "mutation_applied",
        "attempt_completed",
    ];
    assert!(!temporal_rule_holds(
        &swapped,
        "mutation_applied<observer_sampled"
    ));
}

#[test]
fn ecosystem_witnesses_are_exact_and_authority_free() {
    let witnesses = parsed(include_str!(
        "../qualification/examples/ecosystem_witnesses.json"
    ));
    assert_eq!(witnesses["authority"], "NONE");
    assert_eq!(witnesses["consequence"], "EVIDENCE_ONLY");
    for witness in witnesses["witnesses"].as_array().expect("witnesses") {
        let sha = witness["sha"].as_str().expect("exact sha");
        assert_eq!(sha.len(), 40);
        assert!(!witness["repo"].as_str().expect("repo").is_empty());
        assert!(!witness["role"].as_str().expect("role").is_empty());
    }
}
