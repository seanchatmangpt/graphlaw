//! Law-state transitions over the innovation reference pack, end to end:
//! Turtle (PurRDF) -> N3 derivation (Eyeron) -> SHACL admission (PurRDF)
//! -> SPARQL discovery and CONSTRUCT (PurRDF), every step receipted.

use graphlaw::dialect::{Dialect, RefusalKind};
use graphlaw::law::{LawError, LawState, Step};
use graphlaw::rdf::{SparqlEngine, SparqlRequest, SparqlResult};
use graphlaw::sparql::NativeSparqlEngine;

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
    let shapes = read("innovation/candidate.shacl.ttl");
    let s0 = fixture();
    assert_eq!(select_count(&s0, &read("innovation/opportunities.rq")), 0);

    let (s1, r1) = s0
        .transition(&Step::DeriveN3 { rules: &rules })
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
    let (s1, _) = fixture()
        .transition(&Step::DeriveN3 { rules: &rules })
        .unwrap();
    let (s2, r) = s1.transition(&Step::DeriveN3 { rules: &rules }).unwrap();
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
        b"<https://e/c> <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <https://praxis.chatman.io/innovation#CandidateFuture> .\n",
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
    let bad_rules = fixture().transition(&Step::DeriveN3 {
        rules: "{ ?x <p> ?y } => ",
    });
    match bad_rules {
        Err(LawError::Refused(r)) => assert_eq!(r.dialect, Some(Dialect::N3)),
        other => panic!("expected refusal, got {other:?}"),
    }
}
