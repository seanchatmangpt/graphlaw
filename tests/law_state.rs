//! Law-state transitions over the innovation reference pack, end to end:
//! Turtle (PurRDF) -> N3 derivation (Eyeron) -> SHACL admission (PurRDF)
//! -> SPARQL discovery and CONSTRUCT (PurRDF), every step receipted.

use graphlaw::dialect::{Dialect, RefusalKind};
use graphlaw::law::{LawError, LawState, Step};
use graphlaw::rdf::{SparqlEngine, SparqlRequest, SparqlResult};
use graphlaw::sparql::NativeSparqlEngine;
use graphlaw::tier::validate_n3_rules;

fn read(path: &str) -> String {
    std::fs::read_to_string(format!("{}/{path}", env!("CARGO_MANIFEST_DIR"))).unwrap()
}

fn fixture() -> LawState {
    LawState::parse(
        read("innovation/industry-fixture.n3").as_bytes(),
        Dialect::Turtle,
        None,
    )
    .expect("fixture is plain RDF")
}

fn select_count(state: &LawState, q: &str) -> usize {
    match NativeSparqlEngine::new()
        .query(
            state.dataset(),
            SparqlRequest {
                query: q,
                base_iri: None,
                substitutions: &[],
            },
        )
        .expect("sparql")
    {
        SparqlResult::Solutions { rows, .. } => rows.len(),
        other => panic!("expected solutions, got {other:?}"),
    }
}

#[test]
fn innovation_pipeline_derives_admits_and_projects() {
    let rules = read("innovation/blue-ocean-triz.n3");
    let ruleset = validate_n3_rules(rules.as_bytes(), None).expect("rules validate");
    let shapes = read("innovation/candidate.shacl.ttl");
    let s0 = fixture();
    assert_eq!(select_count(&s0, &read("innovation/opportunities.rq")), 0);

    let (s1, r1) = s0
        .transition(&Step::DeriveN3 {
            rules: &rules,
            ruleset: &ruleset,
        })
        .expect("derive");
    assert_eq!(r1.step, "derive:n3");
    assert_eq!(r1.authority.authority, "eyeron");
    assert_eq!((r1.parent.as_str(), r1.child.as_str()), (s0.id(), s1.id()));
    assert_ne!(s0.id(), s1.id());
    assert!(
        r1.added >= 20,
        "expected the full ERRC surface, got {}",
        r1.added
    );
    // Eliminate, Reduce, Raise, Create.
    assert_eq!(select_count(&s1, &read("innovation/opportunities.rq")), 4);

    let (s2, r2) = s1
        .transition(&Step::AdmitShacl {
            shapes_ttl: &shapes,
        })
        .expect("admit");
    assert_eq!(
        (r2.step, r2.added, r2.authority.authority),
        ("admit:shacl", 0, "purrdf::shapes")
    );
    assert_eq!(s2.id(), s1.id());

    let construct = read("innovation/construct-futures.rq");
    match NativeSparqlEngine::new()
        .query(
            s2.dataset(),
            SparqlRequest {
                query: &construct,
                base_iri: None,
                substitutions: &[],
            },
        )
        .expect("construct")
    {
        SparqlResult::Graph(g) => assert_eq!(g.quad_count(), 2, "PDDL handoff + standing"),
        other => panic!("expected a graph, got {other:?}"),
    }
}

#[test]
fn derivation_is_idempotent_and_content_addressed() {
    let rules = read("innovation/blue-ocean-triz.n3");
    let ruleset = validate_n3_rules(rules.as_bytes(), None).expect("rules validate");
    let (s1, _) = fixture()
        .transition(&Step::DeriveN3 {
            rules: &rules,
            ruleset: &ruleset,
        })
        .unwrap();
    let (s2, r) = s1
        .transition(&Step::DeriveN3 {
            rules: &rules,
            ruleset: &ruleset,
        })
        .unwrap();
    assert_eq!(r.added, 0);
    assert_eq!(s1.id(), s2.id());
}

#[test]
fn identity_ignores_statement_order_and_blank_labels() {
    let a = LawState::parse(
        b"_:x <https://e/p> \"1\" .\n<https://e/s> <https://e/q> _:x .\n",
        Dialect::NTriples,
        None,
    )
    .unwrap();
    let b = LawState::parse(
        b"<https://e/s> <https://e/q> _:other .\n_:other <https://e/p> \"1\" .\n",
        Dialect::NTriples,
        None,
    )
    .unwrap();
    assert_eq!(a.id(), b.id());
    let c = LawState::parse(
        b"<https://e/s> <https://e/q> \"2\" .\n",
        Dialect::NTriples,
        None,
    )
    .unwrap();
    assert_ne!(a.id(), c.id());
}

#[test]
fn shacl_gate_refuses_incomplete_candidate_without_creating_state() {
    let shapes = read("innovation/candidate.shacl.ttl");
    let bad = LawState::parse(
        b"<https://e/c> <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <https://graphlaw.chatman.io/innovation#CandidateFuture> .\n",
        Dialect::NTriples,
        None,
    )
    .unwrap();
    match bad.transition(&Step::AdmitShacl {
        shapes_ttl: &shapes,
    }) {
        Err(LawError::NotAdmitted { violations, .. }) => assert!(violations >= 8, "{violations}"),
        other => panic!("expected refusal, got {other:?}"),
    }
}

#[test]
fn rdfs_and_owlrl_entailment_are_receipted_transitions() {
    let s = LawState::parse(
        b"<https://e/Cat> <http://www.w3.org/2000/01/rdf-schema#subClassOf> <https://e/Animal> .\n\
          <https://e/tom> <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <https://e/Cat> .\n",
        Dialect::NTriples,
        None,
    )
    .unwrap();
    for (step, name) in [
        (Step::EntailRdfs, "derive:rdfs"),
        (Step::EntailOwlRl, "derive:owl-rl"),
    ] {
        let (child, r) = s.transition(&step).unwrap();
        assert_eq!(r.step, name);
        assert!(r.added >= 1);
        assert_eq!(
            select_count(
                &child,
                "SELECT * WHERE { <https://e/tom> a <https://e/Animal> }"
            ),
            1,
            "{name}"
        );
    }
}

#[test]
fn engine_refusals_are_typed_and_name_the_engine() {
    let err =
        LawState::parse(b"<https://e/s> <https://e/p> .\n", Dialect::Turtle, None).unwrap_err();
    assert_eq!(err.kind, RefusalKind::EngineRejected);
    assert_eq!(err.dialect, Some(Dialect::Turtle));
    let empty_ruleset = validate_n3_rules(b"", None).expect("empty rule set validates");
    let bad_rules = fixture().transition(&Step::DeriveN3 {
        rules: "{ ?x <p> ?y } => ",
        ruleset: &empty_ruleset,
    });
    match bad_rules {
        Err(LawError::Refused(r)) => assert_eq!(r.dialect, Some(Dialect::N3)),
        other => panic!("expected refusal, got {other:?}"),
    }
}

#[test]
fn unknown_step_capability_refuses_typed_never_panics() {
    // A capability string missing from BACKEND_AUTHORITIES must produce a
    // typed refusal carrying the UNKNOWN-STEP-CAPABILITY marker, not a panic.
    let err = graphlaw::law::authority_for_capability("totally-unknown-capability")
        .expect_err("unknown capability must refuse");
    match &err {
        LawError::Refused(r) => {
            assert_eq!(r.kind, RefusalKind::Unsupported);
            let msg = r.message.as_str();
            assert!(
                msg.contains("[UNKNOWN-STEP-CAPABILITY]"),
                "message missing stable marker: {msg}"
            );
            assert!(
                msg.contains("totally-unknown-capability"),
                "message must name the offending capability: {msg}"
            );
            assert!(
                msg.contains("BACKEND_AUTHORITIES"),
                "message must carry recovery text: {msg}"
            );
        }
        other => panic!("expected LawError::Refused, got {other:?}"),
    }
    // Known capability still resolves to a real authority (Chicago readback).
    let known = graphlaw::law::authority_for_capability("SHACL").expect("SHACL is declared");
    assert_eq!(known.authority, "purrdf::shapes");
}

#[test]
fn transition_with_unknown_capability_refuses_typed_end_to_end() {
    // K10 callsite court: the UNKNOWN-STEP-CAPABILITY refusal must fire through
    // the PUBLIC transition path (the law.rs authority callsite), not just from
    // the helper in isolation — otherwise reverting the callsite to a fallback
    // authority kills no test. Uses the doc(hidden) test-only probe step so the
    // capability string can be absent from BACKEND_AUTHORITIES.
    use graphlaw::law::Step;

    let s = LawState::parse(
        b"<https://e/s> <https://e/p> <https://e/o> .\n",
        Dialect::NTriples,
        None,
    )
    .unwrap();

    // Unknown capability: the whole transition must refuse typed, and refuse
    // without minting a child state or receipt.
    match s.transition(&Step::TestCapabilityProbe("court-unknown-capability")) {
        Err(LawError::Refused(r)) => {
            assert_eq!(r.kind, RefusalKind::Unsupported);
            let msg = r.message.as_str();
            assert!(
                msg.contains("[UNKNOWN-STEP-CAPABILITY]"),
                "message missing stable marker: {msg}"
            );
            assert!(
                msg.contains("court-unknown-capability"),
                "message must name the offending capability: {msg}"
            );
        }
        other => panic!("expected typed refusal through transition(), got {other:?}"),
    }

    // Chicago readback: a declared capability still resolves on the same path.
    // This arm is the mutant-killer: if the authority callsite is reverted to a
    // fallback (helper forced Ok / unknown capabilities laundered through a
    // declared authority), the first arm above succeeds instead of refusing and
    // this test fails.
    let (child, r) = s
        .transition(&Step::TestCapabilityProbe("RDF/RDFS/OWL-RL entailment"))
        .expect("declared capability must transition");
    assert_eq!(r.step, "test:capability-probe");
    assert_eq!(r.authority.authority, "purrdf::entail");
    assert_eq!((r.parent.as_str(), r.child.as_str()), (s.id(), child.id()));
    assert_eq!(child.id(), s.id(), "probe is an identity transition");
    assert_eq!(r.added, 0);
}
