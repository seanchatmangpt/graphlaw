//! Knowledge hooks (`kh:`) executed over the self-monitoring pack. Every query
//! is parsed and evaluated by PurRDF; GraphLaw only orchestrates.

use graphlaw::dialect::{Dialect, RefusalKind};
use graphlaw::hooks::HookPack;
use graphlaw::law::{LawState, Step};
use graphlaw::rdf::{SparqlEngine, SparqlRequest, SparqlResult};
use graphlaw::sparql::NativeSparqlEngine;

fn pack_file(p: &str) -> LawState {
    let bytes = std::fs::read(format!(
        "{}/packs/self-monitoring-pack/{p}",
        env!("CARGO_MANIFEST_DIR")
    ))
    .unwrap();
    LawState::parse(&bytes, Dialect::Turtle, None).unwrap()
}

fn obligations(s: &LawState) -> usize {
    match NativeSparqlEngine::new()
        .query(
            s.dataset(),
            SparqlRequest {
                query: "SELECT ?e WHERE { ?e a <http://seanchatmangpt.github.io/packs/self-monitoring#EscalationObligation> }",
                base_iri: None,
                substitutions: &[],
            },
        )
        .unwrap()
    {
        SparqlResult::Solutions { rows, .. } => rows.len(),
        o => panic!("{o:?}"),
    }
}

fn pack() -> HookPack {
    HookPack::load(&pack_file("hook.ttl")).expect("pack loads")
}

#[test]
fn pack_loads_with_its_declared_shape() {
    let p = pack();
    assert_eq!(p.hooks().len(), 1);
    assert_eq!(p.hooks()[0].name, "derive_escalation_obligation");
    assert_eq!(p.hooks()[0].priority, 1);
}

#[test]
fn broad_topic_session_derives_one_obligation_per_pair() {
    let state = pack_file("fixtures/session-real-broad-topic.ttl");
    assert_eq!(obligations(&state), 0);
    let m = pack().materialize(&state).unwrap();
    assert_eq!(
        obligations(&m.state),
        3,
        "the pack documents a 3-pair counterfactual"
    );
    assert_eq!(m.firings.len(), 3);
    assert!(
        m.firings.iter().all(|f| f.added >= 6),
        "type + two links + reason + 2 firing markers"
    );
}

#[test]
fn materialization_is_a_fixpoint() {
    let state = pack_file("fixtures/session-real-broad-topic.ttl");
    let once = pack().materialize(&state).unwrap();
    let twice = pack().materialize(&once.state).unwrap();
    assert!(twice.firings.is_empty(), "a saturated state fires nothing");
    assert_eq!(twice.state.id(), once.state.id());
    assert_eq!(obligations(&twice.state), obligations(&once.state));
}

#[test]
fn hooks_are_a_receipted_transition() {
    let state = pack_file("fixtures/session-real-broad-topic.ttl");
    let pack = pack();
    let (child, r) = state.transition(&Step::Hooks { pack: &pack }).unwrap();
    assert_eq!(r.step, "derive:hooks");
    assert_eq!(r.authority.authority, "purrdf::sparql");
    assert!(r.added >= 12);
    assert_ne!(state.id(), child.id());
}

#[test]
fn narrow_session_fires_fewer_than_broad() {
    let narrow = pack()
        .materialize(&pack_file("fixtures/session-real.ttl"))
        .unwrap();
    assert!(obligations(&narrow.state) < 3);
}

fn hook_ttl(body: &str) -> LawState {
    let doc = format!(
        "@prefix kh: <{}> . @prefix ex: <https://e/> .\n{body}",
        graphlaw::hooks::KH
    );
    LawState::parse(doc.as_bytes(), Dialect::Turtle, None).unwrap()
}

#[test]
fn unsupported_or_incomplete_hooks_are_refused_not_skipped() {
    let bad_kind = hook_ttl(
        "ex:h a kh:Hook ; kh:kind \"datalog\" ; kh:query \"SELECT * WHERE {?s ?p ?o}\" ; kh:effect \"emit-delta\" ; kh:action ex:a .\n\
         ex:a a kh:Action ; kh:handler <http://seanchatmangpt.github.io/praxis/handler#sparql-construct> ; kh:query \"CONSTRUCT {?s ?p ?o} WHERE {?s ?p ?o}\" .",
    );
    let e = HookPack::load(&bad_kind).unwrap_err();
    assert_eq!(e.kind, RefusalKind::Unsupported);
    assert!(e.message.contains("kh:kind"), "{e}");

    let no_action = hook_ttl(
        "ex:h a kh:Hook ; kh:kind \"sparql\" ; kh:query \"SELECT * WHERE {?s ?p ?o}\" ; kh:effect \"emit-delta\" .",
    );
    assert!(
        HookPack::load(&no_action)
            .unwrap_err()
            .message
            .contains("declared")
    );

    let bad_query = hook_ttl(
        "ex:h a kh:Hook ; kh:kind \"sparql\" ; kh:query \"SELEKT nope\" ; kh:effect \"emit-delta\" ; kh:action ex:a .\n\
         ex:a a kh:Action ; kh:handler <http://seanchatmangpt.github.io/praxis/handler#sparql-construct> ; kh:query \"CONSTRUCT {?s ?p ?o} WHERE {?s ?p ?o}\" .",
    );
    assert!(HookPack::load(&bad_query).is_err());
}

#[test]
fn hooks_chain_by_priority_to_a_fixpoint() {
    // h1 derives ex:B from ex:A; h2 (later priority) derives ex:C from ex:B.
    let mk = |n: u8, from: &str, to: &str, prio: u8| {
        format!(
            "ex:h{n} a kh:Hook ; kh:kind \"sparql\" ; kh:effect \"emit-delta\" ; kh:priority {prio} ; kh:action ex:a{n} ;\n\
               kh:query \"SELECT ?x WHERE {{ ?x a <https://e/{from}> }}\" .\n\
             ex:a{n} a kh:Action ; kh:handler <http://seanchatmangpt.github.io/praxis/handler#sparql-construct> ;\n\
               kh:query \"CONSTRUCT {{ ?x a <https://e/{to}> }} WHERE {{ ?x a <https://e/{from}> }}\" .\n"
        )
    };
    let pack = HookPack::load(&hook_ttl(&(mk(2, "B", "C", 2) + &mk(1, "A", "B", 1)))).unwrap();
    assert_eq!(pack.hooks()[0].iri, "https://e/h1", "ordered by priority");
    let data = LawState::parse(
        b"<https://e/x> <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <https://e/A> .\n",
        Dialect::NTriples,
        None,
    )
    .unwrap();
    let m = pack.materialize(&data).unwrap();
    assert_eq!(
        m.state.quad_count(),
        3 + 2 * 2,
        "A, B, C plus two firing markers each"
    );
    assert_eq!(m.firings.len(), 2);
}
