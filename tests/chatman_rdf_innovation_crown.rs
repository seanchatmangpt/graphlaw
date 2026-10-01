//! Crown invariants of the RDF Blue Ocean / TRIZ innovation pack (see
//! `innovation/README.md`): full ERRC surface, TRIZ principles preserved
//! without a winner, zero human/agent mediation, and Post-AGI denial rules as
//! real falsifiers. Also pins the GraphLaw resource-caps ERRC encoded in
//! `innovation/resource-caps-errc.ttl`.

use graphlaw::dialect::Dialect;
use graphlaw::law::{LawState, Step};
use graphlaw::rdf::{SparqlEngine, SparqlRequest, SparqlResult};
use graphlaw::sparql::NativeSparqlEngine;

const INV: &str = "https://praxis.chatman.io/innovation#";

fn read(path: &str) -> String {
    std::fs::read_to_string(format!("{}/{path}", env!("CARGO_MANIFEST_DIR"))).unwrap()
}

fn rules() -> String {
    read("innovation/blue-ocean-triz.n3")
}

fn load(path: &str) -> LawState {
    LawState::parse(read(path).as_bytes(), Dialect::Turtle, None).expect("pack fixture is RDF")
}

fn derive(state: &LawState) -> LawState {
    state
        .transition(&Step::DeriveN3 { rules: &rules() })
        .expect("derive")
        .0
}

fn count(state: &LawState, query: &str) -> usize {
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
        SparqlResult::Solutions { rows, .. } => rows.len(),
        other => panic!("expected solutions, got {other:?}"),
    }
}

fn errc(state: &LawState, action: &str) -> usize {
    count(
        state,
        &format!("SELECT ?s WHERE {{ ?s <{INV}errcAction> <{INV}{action}> }}"),
    )
}

/// Reference fixture plus extra Turtle statements (for falsifiers).
fn fixture_with(extra: &str) -> LawState {
    let text = format!("{}\n{extra}\n", read("innovation/industry-fixture.n3"));
    LawState::parse(text.as_bytes(), Dialect::Turtle, None).expect("fixture + extra is RDF")
}

#[test]
fn reference_fixture_derives_every_errc_quadrant() {
    let s = derive(&load("innovation/industry-fixture.n3"));
    for action in ["Eliminate", "Reduce", "Raise", "Create"] {
        assert_eq!(errc(&s, action), 1, "{action}");
    }
}

#[test]
fn resource_caps_errc_derives_4_2_2_3() {
    let s = derive(&load("innovation/resource-caps-errc.ttl"));
    assert_eq!(errc(&s, "Eliminate"), 4);
    assert_eq!(errc(&s, "Reduce"), 2);
    assert_eq!(errc(&s, "Raise"), 2);
    assert_eq!(errc(&s, "Create"), 3);
    assert_eq!(count(&s, &read("innovation/opportunities.rq")), 11);
}

#[test]
fn triz_principles_are_all_preserved_and_no_winner_is_selected() {
    let s = derive(&load("innovation/industry-fixture.n3"));
    let principles = count(
        &s,
        &format!("SELECT ?p WHERE {{ ?f <{INV}considerPrinciple> ?p }}"),
    );
    assert_eq!(principles, 2, "SeparationInTime + Intermediary");
    // No predicate in the closure selects or ranks a principle or candidate.
    assert_eq!(
        count(
            &s,
            "SELECT ?s WHERE { ?s ?p ?o . FILTER(CONTAINS(STR(?p), \"select\") || CONTAINS(STR(?p), \"winner\") || CONTAINS(STR(?p), \"chosen\")) }"
        ),
        0
    );
}

#[test]
fn candidates_require_neither_human_nor_agent_mediation() {
    for path in [
        "innovation/industry-fixture.n3",
        "innovation/resource-caps-errc.ttl",
    ] {
        let s = derive(&load(path));
        let candidates = count(
            &s,
            &format!("SELECT ?f WHERE {{ ?f a <{INV}CandidateFuture> }}"),
        );
        assert!(candidates >= 1, "{path}");
        for facet in ["humanMediation", "agentMediation"] {
            assert_eq!(
                count(
                    &s,
                    &format!(
                        "SELECT ?f WHERE {{ ?f a <{INV}CandidateFuture> ; <{INV}{facet}> <{INV}None> }}"
                    )
                ),
                candidates,
                "{path} {facet}"
            );
            assert_eq!(
                count(
                    &s,
                    &format!("SELECT ?f WHERE {{ ?f <{INV}{facet}> <{INV}Required> }}")
                ),
                0,
                "{path} {facet}"
            );
        }
    }
}

#[test]
fn candidates_carry_no_authority_or_actuation() {
    let s = derive(&load("innovation/resource-caps-errc.ttl"));
    assert_eq!(
        count(
            &s,
            "SELECT ?s WHERE { ?s ?p ?o . FILTER(CONTAINS(STR(?p), \"authority\") || CONTAINS(STR(?p), \"actuat\")) }"
        ),
        0
    );
}

#[test]
fn denial_refuses_required_agent_when_deterministic_morphism_exists() {
    let bad = fixture_with(&format!(
        "<https://praxis.chatman.io/innovation/demo#direct-outcome> a <{INV}CandidateFuture> ;\n  <{INV}agentMediation> <{INV}Required> ;\n  <{INV}hasEquivalentDeterministicMorphism> <{INV}True> ."
    ));
    assert!(
        bad.transition(&Step::DeriveN3 { rules: &rules() }).is_err(),
        "denial rule must refuse"
    );
}

#[test]
fn denial_refuses_contradictory_none_and_required_agent_mediation() {
    // The derived `agentMediation None` meets the asserted `Required`.
    let bad = fixture_with(&format!(
        "<https://praxis.chatman.io/innovation/demo#direct-outcome> a <{INV}CandidateFuture> ;\n  <{INV}agentMediation> <{INV}Required> ."
    ));
    assert!(
        bad.transition(&Step::DeriveN3 { rules: &rules() }).is_err(),
        "contradictory mediation must be refused"
    );
}

#[test]
fn clean_fixture_is_admitted_so_the_denials_are_not_vacuous() {
    assert!(
        load("innovation/industry-fixture.n3")
            .transition(&Step::DeriveN3 { rules: &rules() })
            .is_ok()
    );
}
