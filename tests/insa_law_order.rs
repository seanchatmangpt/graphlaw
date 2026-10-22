//! insa vow-order law checked by graphlaw's LawState/N3 surface
//! (lane L117, L76 rank-2 consumer side).
//!
//! Subject: ggen-marketplace packs/insa-law-pack/ontology.ttl — the canonical
//! projection of insa-replaw's eight-vow typestate chain (Source -> Pattern ->
//! Field -> Motion -> Receiver -> Fruit -> Receipt -> Rest, terminal Rest
//! self-edge) as 8 `insa:VowTransition` individuals with fromState/toState.
//!
//! Legs:
//! (a) chain structure court — SPARQL over the loaded LawState asserts the
//!     8-transition linear walk Source..Rest with the terminal self-edge;
//! (b) vow-trace ordering court — a trace is modeled as Turtle fact list
//!     (step ordinals + edge bindings) and checked two ways:
//!       1. law machinery: `Step::DeriveN3` (bounded Eyeron N3, src/law.rs
//!          :433-443 limits) derives `insa:linkOk` for each lawful adjacent
//!          pair; a lawful 8-edge trace derives 7, a transposed trace 6;
//!       2. SPARQL over the combined graph: a violation row (toState of the
//!          earlier step's edge != fromState of the later step's edge)
//!          EXISTS for the transposition and NOT EXISTS for the good trace.
//!
//! The blake3 of the pack ontology is pinned in-header; any drift in the pack
//! fails loudly here before any consumer trusts the chain.


use graphlaw::dialect::Dialect;
use graphlaw::law::{LawState, Step};
use graphlaw::tier::validate_n3_rules;
use graphlaw::rdf::{SparqlEngine, SparqlRequest, SparqlResult};
use graphlaw::sparql::NativeSparqlEngine;

/// Path to the pack ontology (canonical checkout; override for other workspaces).
const ONTOLOGY_PATH: &str = "/Users/sac/ggen-marketplace/packs/insa-law-pack/ontology.ttl";

/// blake3 of packs/insa-law-pack/ontology.ttl (raw file bytes), computed at
/// pin time from the canonical file.
const ONTOLOGY_BLAKE3: &str = "19d9ec90cb861e8b1e320ef01e5e0d8a54022b19357f2b4856f379417dcc79d7";

/// Vow chain edges in lawful order (ontology ordinals 1..8).
const LAWFUL_EDGES: [&str; 8] = [
    "tx-source-pattern",
    "tx-pattern-field",
    "tx-field-motion",
    "tx-motion-receiver",
    "tx-receiver-fruit",
    "tx-fruit-receipt",
    "tx-receipt-rest",
    "tx-rest-seal",
];

/// True when the pack ontology exists at its canonical (or overridden) path.
/// CI machines without the sibling marketplace checkout must skip loudly
/// rather than panic (same pattern as refusal_crosswalk_differential.rs).
fn ontology_available() -> bool {
    let path = std::env::var("INSA_LAW_PACK_ONTOLOGY").unwrap_or_else(|_| ONTOLOGY_PATH.to_string());
    std::path::Path::new(&path).is_file()
}

fn ontology_bytes() -> Vec<u8> {
    let path = std::env::var("INSA_LAW_PACK_ONTOLOGY").unwrap_or_else(|_| ONTOLOGY_PATH.to_string());
    std::fs::read(&path)
        .unwrap_or_else(|e| panic!("read {path}: {e}"))
}

/// Loud typed skip: name the missing pack path so CI logs show why, never a
/// silent pass or a bare panic on a machine without the sibling checkout.
fn require_ontology() -> bool {
    if ontology_available() {
        return true;
    }
    let path = std::env::var("INSA_LAW_PACK_ONTOLOGY").unwrap_or_else(|_| ONTOLOGY_PATH.to_string());
    eprintln!(
        "BLOCKED: insa-law-pack ontology absent at {path} \
         (set INSA_LAW_PACK_ONTOLOGY to override); courts skipped"
    );
    false
}

fn pinned_ontology() -> LawState {
    let bytes = ontology_bytes();
    let mut h = blake3::Hasher::new();
    h.update(&bytes);
    let digest: [u8; 32] = *h.finalize().as_bytes();
    let got: String = digest.iter().map(|b| format!("{b:02x}")).collect();
    assert_eq!(
        got, ONTOLOGY_BLAKE3,
        "insa-law-pack ontology.ttl drifted from the pinned blake3"
    );
    LawState::parse(&bytes, Dialect::Turtle, None).expect("ontology parses as Turtle")
}

/// Run a SPARQL SELECT; return rows as term strings (RDF label form).
fn rows(state: &LawState, query: &str) -> Vec<String> {
    match NativeSparqlEngine::new().query(
        state.dataset(),
        SparqlRequest {
            query,
            base_iri: None,
            substitutions: &[],
        },
    )
    .expect("SPARQL executes")
    {
        SparqlResult::Solutions { rows, .. } => rows
            .iter()
            .map(|r| {
                r.iter()
                    .map(|t| format!("{t:?}"))
                    .collect::<Vec<_>>()
                    .join("|")
            })
            .collect(),
        o => panic!("expected solutions, got {o:?}"),
    }
}

/// Extract the bare IRI text from a Debug-format term field
/// (rows serialize each term as `Some(Iri("urn:..."))`).
fn iri(field: &str) -> &str {
    match field.find("Iri(\"") {
        Some(a) => {
            let a = a + 5;
            let b = field[a..].find('"').expect("Iri closes") + a;
            &field[a..b]
        }
        // Non-IRI term (e.g. a literal ordinal): return the field verbatim.
        None => field,
    }
}

fn scalar(state: &LawState, query: &str) -> usize {
    rows(state, query).len()
}

/// Trace facts: 8 steps, each bound to a vow edge, chained by insa:next.
fn trace_turtle(edges: [&str; 8]) -> String {
    let mut s = String::from("@prefix insa: <urn:insa-law:> .\n");
    for (i, e) in edges.iter().enumerate() {
        let n = i + 1;
        if n < 8 {
            s.push_str(&format!(
                "insa:step{n} insa:edge insa:{e} ; insa:next insa:step{} .\n",
                n + 1
            ));
        } else {
            s.push_str(&format!("insa:step{n} insa:edge insa:{e} .\n"));
        }
    }
    s
}

fn combined(edges: [&str; 8]) -> LawState {
    let mut bytes = ontology_bytes();
    bytes.extend_from_slice(trace_turtle(edges).as_bytes());
    LawState::parse(&bytes, Dialect::Turtle, None).expect("combined graph parses")
}

/// N3 rules: derive linkOk for each adjacent lawful pair.
const LINK_RULES: &str = "\
@prefix insa: <urn:insa-law:> .\
{ ?a insa:next ?b . ?a insa:edge ?ta . ?ta insa:toState ?m . \
?b insa:edge ?tb . ?tb insa:fromState ?m . } => { ?a insa:linkOk insa:true . } .";

fn derive_link_ok(state: &LawState) -> LawState {
    let ruleset = validate_n3_rules(LINK_RULES.as_bytes(), None).expect("LINK_RULES validate");
    match state.transition(&Step::DeriveN3 { rules: LINK_RULES, ruleset: &ruleset }) {
        Ok((child, _receipt)) => child,
        Err(e) => panic!("DeriveN3 refused: {e}"),
    }
}

// ---------------------------------------------------------------------------
// (a) chain structure court
// ---------------------------------------------------------------------------

#[test]
fn ontology_matches_pinned_blake3() {
    if !require_ontology() {
        return;
    }
    let _ = pinned_ontology();
}

#[test]
fn exactly_8_transitions_form_the_linear_chain() {
    if !require_ontology() {
        return;
    }
    let ont = pinned_ontology();

    let n = scalar(
        &ont,
        "SELECT ?tx WHERE { ?tx a <urn:insa-law:VowTransition> }",
    );
    assert_eq!(n, 8, "expected exactly 8 VowTransition individuals");

    // Walk: fetch (ordinal, transition, from, to) rows and verify in Rust
    // (ordering done locally, not relied on from the engine).
    let got = rows(
        &ont,
        "SELECT ?tx ?ord ?from ?to WHERE { \
         ?tx a <urn:insa-law:VowTransition> ; \
         <urn:insa-law:ordinal> ?ord ; \
         <urn:insa-law:fromState> ?from ; \
         <urn:insa-law:toState> ?to . }",
    );
    assert_eq!(got.len(), 8);

    // ordinals are exactly 1..8, one per edge, in lawful edge order
    let mut by_edge: Vec<(String, String, String, String)> = Vec::new(); // (edge, ord, from, to)
    for r in &got {
        let cols: Vec<&str> = r.split('|').collect();
        assert_eq!(cols.len(), 4, "row shape: {r}");
        by_edge.push((
            iri(cols[0]).to_string(),
            iri(cols[1]).to_string(),
            iri(cols[2]).to_string(),
            iri(cols[3]).to_string(),
        ));
    }
    by_edge.sort_by(|a, b| a.1.cmp(&b.1));

    for (i, (edge, _ord, from, to)) in by_edge.iter().enumerate() {
        assert!(
            edge.ends_with(LAWFUL_EDGES[i]),
            "ordinal {} is {edge}, expected {}",
            i + 1,
            LAWFUL_EDGES[i]
        );
        assert!(
            from.ends_with(&format!("state-{}", state_stem(i))),
            "edge {} fromState {from}",
            LAWFUL_EDGES[i]
        );
        let want_to = if i == 7 {
            "state-rest".to_string() // terminal self-edge
        } else {
            format!("state-{}", state_stem(i + 1))
        };
        assert!(
            to.ends_with(&want_to),
            "edge {} toState {to}, expected {want_to}",
            LAWFUL_EDGES[i]
        );
    }
}

/// Vow state stem for chain index 0..8 (Source..Rest).
fn state_stem(i: usize) -> &'static str {
    [
        "source",
        "pattern",
        "field",
        "motion",
        "receiver",
        "fruit",
        "receipt",
        "rest",
    ][i]
}

#[test]
fn chain_length_declaration_matches_reality() {
    if !require_ontology() {
        return;
    }
    let ont = pinned_ontology();
    let got = rows(
        &ont,
        "SELECT ?n WHERE { <urn:insa-law:pack> <urn:insa-law:chainLength> ?n }",
    );
    assert_eq!(got.len(), 1, "one chainLength declaration");
    assert!(
        got[0].contains(&"8".to_string()),
        "chainLength literal should be 8, got {}",
        got[0]
    );
}

// ---------------------------------------------------------------------------
// (b) vow-trace ordering court
// ---------------------------------------------------------------------------

#[test]
fn lawful_trace_passes_n3_and_sparql_legs() {
    if !require_ontology() {
        return;
    }
    let edges = LAWFUL_EDGES;
    let state = combined(edges);

    // Leg 1: law machinery (bounded N3 derivation) derives all 7 linkOk facts.
    let derived = derive_link_ok(&state);
    let ok_count = scalar(
        &derived,
        "SELECT ?a WHERE { ?a <urn:insa-law:linkOk> ?o }",
    );
    assert_eq!(ok_count, 7, "all 7 adjacent pairs lawful");

    // Leg 2: SPARQL over the combined graph — no violation row exists.
    let violations = rows(&state, VIOLATION_QUERY);
    assert!(
        violations.is_empty(),
        "lawful trace must have no violations, got {violations:?}"
    );
}

#[test]
fn transposed_pair_is_flagged_by_both_legs() {
    if !require_ontology() {
        return;
    }
    // Transpose edges 5 and 4 (receiver->fruit before motion->receiver).
    let mut edges = LAWFUL_EDGES;
    edges.swap(3, 4);
    let state = combined(edges);

    // Leg 1: law machinery — swapping adjacent edges 4,5 breaks the 3
    // adjacencies touching those steps (3-4, 4-5, 5-6): 7 total, 3 broken.
    let derived = derive_link_ok(&state);
    let ok_count = scalar(
        &derived,
        "SELECT ?a WHERE { ?a <urn:insa-law:linkOk> ?o }",
    );
    assert_eq!(ok_count, 4, "swapping adjacent edges 4,5 breaks 3 adjacencies");

    // Leg 2: SPARQL violation rows EXIST exactly at the three broken
    // adjacencies (step3,step4), (step4,step5), (step5,step6).
    let violations = rows(&state, VIOLATION_QUERY);
    assert_eq!(
        violations.len(), 3,
        "three adjacent pairs violate the chain: {violations:?}"
    );
    let mut broken: Vec<String> = violations
        .iter()
        .map(|v| {
            let cols: Vec<&str> = v.split('|').collect();
            format!(
                "{}->{}",
                iri(cols[0]).rsplit(':').next().unwrap(),
                iri(cols[1]).rsplit(':').next().unwrap()
            )
        })
        .collect();
    broken.sort();
    assert_eq!(
        broken,
        vec![
            "step4->step3".to_string(),
            "step5->step4".to_string(),
            "step6->step5".to_string(),
        ]
    );
}

#[test]
fn reversed_trace_is_flagged() {
    if !require_ontology() {
        return;
    }
    // Stronger corruption: fully reversed chain (Rest edge first).
    let mut edges = LAWFUL_EDGES;
    edges.reverse();
    let state = combined(edges);
    let violations = rows(&state, VIOLATION_QUERY);
    assert!(
        violations.len() >= 6,
        "reversal breaks most adjacencies, got {}",
        violations.len()
    );
}

/// SPARQL over the combined (ontology + trace) graph: an adjacency is a
/// violation when the earlier step's edge toState differs from the later
/// step's edge fromState.
const VIOLATION_QUERY: &str = "\
SELECT ?a ?b WHERE { \
?b <urn:insa-law:next> ?a . \
?b <urn:insa-law:edge> ?tb . ?tb <urn:insa-law:toState> ?m . \
?a <urn:insa-law:edge> ?ta . ?ta <urn:insa-law:fromState> ?m2 . \
FILTER(?m != ?m2) }";
