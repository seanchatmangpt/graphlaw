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

#[test]
fn refuse_hook_fires_with_reason_and_emits_no_delta() {
    let doc = format!(
        "@prefix kh: <{}> . @prefix ex: <https://e/> .\n\
         ex:no-writes a kh:Hook ; kh:kind \"sparql\" ; kh:effect \"refuse\" ; \
           kh:reason \"unattended writes are refused\" ; kh:action ex:a ;\n\
           kh:query \"SELECT ?x WHERE {{ ?x a <https://e/Forbidden> }}\" .\n\
         ex:a a kh:Action ; kh:handler <http://seanchatmangpt.github.io/praxis/handler#sparql-construct> ;\n\
           kh:query \"CONSTRUCT {{ ?x <https://e/never> <https://e/emitted> }} WHERE {{ ?x a <https://e/Forbidden> }}\" .\n",
        graphlaw::hooks::KH
    );
    let pack = HookPack::load(&LawState::parse(doc.as_bytes(), Dialect::Turtle, None).unwrap())
        .expect("refuse pack loads");
    assert_eq!(pack.hooks()[0].effect, graphlaw::hooks::Effect::Refuse);
    let state = LawState::parse(
        b"<https://e/x> <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <https://e/Forbidden> .\n",
        Dialect::NTriples,
        None,
    )
    .unwrap();
    let before = state.quad_count();
    let m = pack.materialize(&state).unwrap();
    assert_eq!(m.firings.len(), 1);
    assert_eq!(m.verdicts.len(), 1);
    assert_eq!(
        m.verdicts[0].verdict,
        graphlaw::hooks::Verdict::Refuse("unattended writes are refused".into())
    );
    assert_eq!(m.verdicts[0].hook, "https://e/no-writes");
    // marker only: no CONSTRUCT delta merged
    assert_eq!(m.state.quad_count(), before + 2);
    // a refuse firing is once-per-lineage like any firing: fixpoint holds
    let again = pack.materialize(&m.state).unwrap();
    assert!(again.firings.is_empty() && again.verdicts.is_empty());
}

#[test]
fn refuse_hook_that_matches_nothing_produces_no_verdict() {
    let doc = format!(
        "@prefix kh: <{}> . @prefix ex: <https://e/> .\n\
         ex:quiet a kh:Hook ; kh:kind \"sparql\" ; kh:effect \"refuse\" ; kh:action ex:a ;\n\
           kh:query \"SELECT ?x WHERE {{ ?x a <https://e/Absent> }}\" .\n\
         ex:a a kh:Action ; kh:handler <http://seanchatmangpt.github.io/praxis/handler#sparql-construct> ;\n\
           kh:query \"CONSTRUCT {{ ?x ?p ?o }} WHERE {{ ?x ?p ?o }}\" .\n",
        graphlaw::hooks::KH
    );
    let pack = HookPack::load(&LawState::parse(doc.as_bytes(), Dialect::Turtle, None).unwrap())
        .expect("refuse pack loads");
    let state = LawState::parse(
        b"<https://e/x> <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <https://e/Unrelated> .\n",
        Dialect::NTriples,
        None,
    )
    .unwrap();
    let m = pack.materialize(&state).unwrap();
    assert!(m.firings.is_empty());
    assert!(m.verdicts.is_empty());
    // missing kh:reason falls back to kh:name in the verdict payload
    assert_eq!(pack.hooks()[0].name, "quiet");
    assert_eq!(pack.hooks()[0].reason, None);
}

#[test]
fn emit_delta_firing_surfaces_a_fired_verdict() {
    // regression: existing emit-delta packs parse and fire identically,
    // now additionally surfacing Verdict::Fired.
    let state = pack_file("fixtures/session-real-broad-topic.ttl");
    let m = pack().materialize(&state).unwrap();
    assert_eq!(m.verdicts.len(), m.firings.len());
    assert!(
        m.verdicts
            .iter()
            .all(|v| v.verdict == graphlaw::hooks::Verdict::Fired)
    );
}

#[test]
fn malformed_effect_value_is_a_typed_refusal() {
    let doc = format!(
        "@prefix kh: <{}> . @prefix ex: <https://e/> .\n\
         ex:h a kh:Hook ; kh:kind \"sparql\" ; kh:query \"SELECT * WHERE {{?s ?p ?o}}\" ; kh:effect \"refuse-now\" ; kh:action ex:a .\n\
         ex:a a kh:Action ; kh:handler <http://seanchatmangpt.github.io/praxis/handler#sparql-construct> ; kh:query \"CONSTRUCT {{?s ?p ?o}} WHERE {{?s ?p ?o}}\" .",
        graphlaw::hooks::KH
    );
    let e = HookPack::load(&LawState::parse(doc.as_bytes(), Dialect::Turtle, None).unwrap())
        .unwrap_err();
    assert_eq!(e.kind, RefusalKind::Unsupported);
    assert!(e.message.contains("kh:effect"), "{e}");
}
