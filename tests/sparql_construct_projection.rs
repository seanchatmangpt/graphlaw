//! CONSTRUCT projection of admitted candidate futures into the PDDL planning
//! frontier (`innovation/construct-futures.rq`). The projection is pure: it
//! emits exactly the handoff + standing facts for planning-eligible candidates,
//! never mutates its source, and emits nothing for an unadmitted candidate.

use graphlaw::dialect::Dialect;
use graphlaw::law::{LawState, Step};
use graphlaw::rdf::{SparqlEngine, SparqlRequest, SparqlResult};
use graphlaw::sparql::NativeSparqlEngine;

fn read(path: &str) -> String {
    std::fs::read_to_string(format!("{}/{path}", env!("CARGO_MANIFEST_DIR"))).unwrap()
}

fn load(path: &str) -> LawState {
    LawState::parse(read(path).as_bytes(), Dialect::Turtle, None).expect("pack fixture is RDF")
}

fn derived(path: &str) -> LawState {
    let rules = read("innovation/blue-ocean-triz.n3");
    load(path)
        .transition(&Step::DeriveN3 { rules: &rules })
        .expect("derive")
        .0
}

fn construct_quads(state: &LawState) -> usize {
    match NativeSparqlEngine::new()
        .query(
            state.dataset(),
            SparqlRequest {
                query: &read("innovation/construct-futures.rq"),
                base_iri: None,
                substitutions: &[],
            },
        )
        .expect("construct")
    {
        SparqlResult::Graph(g) => g.quad_count(),
        other => panic!("expected a graph, got {other:?}"),
    }
}

#[test]
fn reference_crown_projects_one_candidate_to_two_facts() {
    assert_eq!(
        construct_quads(&derived("innovation/industry-fixture.n3")),
        2
    );
}

#[test]
fn resource_caps_pack_projects_three_candidates_to_six_facts() {
    assert_eq!(
        construct_quads(&derived("innovation/resource-caps-errc.ttl")),
        6
    );
}

#[test]
fn construct_does_not_mutate_its_source() {
    let state = derived("innovation/resource-caps-errc.ttl");
    let (id, quads) = (state.id().to_string(), state.quad_count());
    let first = construct_quads(&state);
    let second = construct_quads(&state);
    assert_eq!(first, second, "projection is deterministic");
    assert_eq!((state.id(), state.quad_count()), (id.as_str(), quads));
}

#[test]
fn underived_source_projects_nothing() {
    // Before derivation no node is a CandidateFuture, so nothing is eligible.
    assert_eq!(
        construct_quads(&load("innovation/resource-caps-errc.ttl")),
        0
    );
}

#[test]
fn candidate_without_eligible_frontier_is_not_projected() {
    let state = LawState::parse(
        b"<https://e/c> a <https://praxis.chatman.io/innovation#CandidateFuture> .\n",
        Dialect::Turtle,
        None,
    )
    .unwrap();
    assert_eq!(construct_quads(&state), 0);
}
