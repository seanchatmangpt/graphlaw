//! The self-monitoring pack's hook is data: a `kh:query` literal inside
//! `hook.ttl`. PurRDF reads the literal out of the RDF and PurRDF executes it
//! against the captured session fixtures.

use praxis_graphlaw::rdf::{SparqlEngine, SparqlRequest, SparqlResult, TermValue};
use praxis_graphlaw::sparql::NativeSparqlEngine;
use praxis_graphlaw::{dialect::Dialect, law::LawState};

fn load(path: &str) -> LawState {
    let bytes = std::fs::read(format!(
        "{}/packs/self-monitoring-pack/{path}",
        env!("CARGO_MANIFEST_DIR")
    ))
    .unwrap();
    LawState::parse(&bytes, Dialect::Turtle, None).unwrap()
}

fn rows(state: &LawState, query: &str) -> Vec<Vec<Option<TermValue>>> {
    match NativeSparqlEngine::new()
        .query(
            state.dataset(),
            SparqlRequest {
                query,
                base_iri: None,
                substitutions: &[],
            },
        )
        .expect("sparql")
    {
        SparqlResult::Solutions { rows, .. } => rows,
        other => panic!("expected solutions, got {other:?}"),
    }
}

fn hook_query() -> String {
    let hook = load("hook.ttl");
    let r = rows(
        &hook,
        "SELECT ?q WHERE { <http://seanchatmangpt.github.io/packs/self-monitoring#derive_escalation_obligation> <http://seanchatmangpt.github.io/praxis/kh#query> ?q }",
    );
    assert_eq!(r.len(), 1, "exactly one kh:query");
    match r[0][0].as_ref().expect("bound") {
        TermValue::Literal { lexical_form, .. } => lexical_form.clone(),
        other => panic!("kh:query is not a literal: {other:?}"),
    }
}

#[test]
fn hook_query_is_valid_sparql_and_fires_on_broad_topic_session() {
    let q = hook_query();
    let broad = rows(&load("fixtures/session-real-broad-topic.ttl"), &q);
    assert_eq!(broad.len(), 3, "the pack documents a 3-pair counterfactual");
}

#[test]
fn hook_query_is_quiet_on_the_narrow_session() {
    let q = hook_query();
    let narrow = rows(&load("fixtures/session-real.ttl"), &q);
    assert!(
        narrow.len() < 3,
        "narrow session must not exhibit the broad pattern, got {}",
        narrow.len()
    );
}
