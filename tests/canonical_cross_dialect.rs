//! R03 cross-dialect canonical-identity court (V05 lane).
//!
//! Same RDF content serialized in different dialects must yield the identical
//! RDFC-1.0 + BLAKE3 canonical id (`LawState::from_dataset`), a named graph
//! must be distinguished from the default graph, blank-node relabeling must
//! not move identity, N3 must be refused as `Unsupported`, and the whole
//! court must be non-vacuous (a one-quad perturbation moves the id).
//!
//! Dialects that PurRDF refuses for a given fixture are skipped with a typed
//! marker and counted in the run receipt — never silently ignored.
use std::collections::BTreeMap;

use graphlaw::dialect::{Dialect, RefusalKind};
use graphlaw::law::LawState;

/// Per-test pass/skip ledger. Skips are typed, never silent.
#[derive(Default)]
struct Ledger {
    pass: BTreeMap<&'static str, usize>,
    skip: BTreeMap<&'static str, String>,
}

impl Ledger {
    fn note_pass(&mut self, dialect: &'static str) {
        *self.pass.entry(dialect).or_insert(0) += 1;
    }
    fn note_skip(&mut self, dialect: &'static str, why: String) {
        self.skip.insert(dialect, why);
    }
    fn passes(&self) -> usize {
        self.pass.values().sum()
    }
}

/// Parse `text` as `dialect`; on refusal, record a typed skip and return None.
fn admit_or_skip(
    ledger: &mut Ledger,
    name: &'static str,
    text: &str,
    dialect: Dialect,
    what: &str,
) -> Option<LawState> {
    match LawState::parse(text.as_bytes(), dialect, None) {
        Ok(state) => {
            ledger.note_pass(name);
            Some(state)
        }
        Err(r) => {
            assert_eq!(
                r.kind,
                RefusalKind::Unsupported,
                "{what}: {dialect:?} refused with non-Unsupported kind ({:?}): {}",
                r.kind,
                r.message
            );
            ledger.note_skip(name, format!("{what}: {}", r.message));
            None
        }
    }
}

// ---------------------------------------------------------------- fixtures

/// Pure 3-triple graph (no blank nodes), same content in every dialect.
const PURE_TTL: &str = r#"@prefix ex: <http://example.org/> .
ex:alice ex:knows ex:bob .
ex:bob ex:knows ex:carol .
ex:carol ex:knows ex:alice .
"#;

const PURE_NT: &str = "<http://example.org/alice> <http://example.org/knows> <http://example.org/bob> .\n\
<http://example.org/bob> <http://example.org/knows> <http://example.org/carol> .\n\
<http://example.org/carol> <http://example.org/knows> <http://example.org/alice> .\n";

const PURE_RDFXML: &str = r#"<?xml version="1.0"?>
<rdf:RDF xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#"
         xmlns:ex="http://example.org/">
  <rdf:Description rdf:about="http://example.org/alice">
    <ex:knows rdf:resource="http://example.org/bob"/>
  </rdf:Description>
  <rdf:Description rdf:about="http://example.org/bob">
    <ex:knows rdf:resource="http://example.org/carol"/>
  </rdf:Description>
  <rdf:Description rdf:about="http://example.org/carol">
    <ex:knows rdf:resource="http://example.org/alice"/>
  </rdf:Description>
</rdf:RDF>
"#;

const PURE_JSONLD: &str = r#"{
  "@context": {"ex": "http://example.org/"},
  "@graph": [
    {"@id": "ex:alice", "ex:knows": {"@id": "ex:bob"}},
    {"@id": "ex:bob", "ex:knows": {"@id": "ex:carol"}},
    {"@id": "ex:carol", "ex:knows": {"@id": "ex:alice"}}
  ]
}"#;

/// TriG default-graph form (no graph block): same three triples.
const PURE_TRIG: &str = r#"@prefix ex: <http://example.org/> .
ex:alice ex:knows ex:bob .
ex:bob ex:knows ex:carol .
ex:carol ex:knows ex:alice .
"#;

/// Same three triples as the pure graph, placed in one named graph.
const DATASET_TRIG: &str = r#"@prefix ex: <http://example.org/> .
ex:g {
  ex:alice ex:knows ex:bob .
  ex:bob ex:knows ex:carol .
  ex:carol ex:knows ex:alice .
}
"#;

const DATASET_NQ: &str = "<http://example.org/alice> <http://example.org/knows> <http://example.org/bob> <http://example.org/g> .\n\
<http://example.org/bob> <http://example.org/knows> <http://example.org/carol> <http://example.org/g> .\n\
<http://example.org/carol> <http://example.org/knows> <http://example.org/alice> <http://example.org/g> .\n";

/// Default-graph (no graph term) form of the same three triples.
const DEFAULT_GRAPH_NQ: &str = "<http://example.org/alice> <http://example.org/knows> <http://example.org/bob> .\n\
<http://example.org/bob> <http://example.org/knows> <http://example.org/carol> .\n\
<http://example.org/carol> <http://example.org/knows> <http://example.org/alice> .\n";

/// Blank-node cycle graph (isomorphic under relabeling).
const BNODE_NT: &str = "_:x <http://example.org/knows> _:y .\n\
_:y <http://example.org/knows> _:z .\n\
_:z <http://example.org/knows> _:x .\n";

const BNODE_NT_RELABELED: &str = "_:p <http://example.org/knows> _:q .\n\
_:q <http://example.org/knows> _:r .\n\
_:r <http://example.org/knows> _:p .\n";

// N3 (rule syntax, Eyeron-owned) — must be refused as Unsupported.
const N3_DOC: &str = r#"@prefix ex: <http://example.org/> .
{ ex:alice ex:knows ex:bob . } => { ex:bob ex:knows ex:alice . } .
"#;

// ------------------------------------------------------------------ courts

#[test]
fn graph_court_pure_graph_id_identical_across_dialects() {
    let mut ledger = Ledger::default();
    let sources: Vec<(&'static str, &str, Dialect)> = vec![
        ("turtle", PURE_TTL, Dialect::Turtle),
        ("ntriples", PURE_NT, Dialect::NTriples),
        ("nquads", DEFAULT_GRAPH_NQ, Dialect::NQuads),
        ("trig", PURE_TRIG, Dialect::TriG),
        ("rdf-xml", PURE_RDFXML, Dialect::RdfXml),
        ("json-ld", PURE_JSONLD, Dialect::JsonLd),
    ];
    let mut ids: Vec<String> = Vec::new();
    for (name, text, dialect) in sources {
        if let Some(state) = admit_or_skip(&mut ledger, name, text, dialect, "pure graph court") {
            assert_eq!(state.quad_count(), 3, "{name}: expected 3 quads");
            ids.push(state.id().to_string());
        }
    }
    eprintln!(
        "graph-court receipt: passes={:?} skips={:?}",
        ledger.pass, ledger.skip
    );
    assert!(
        ledger.passes() >= 2,
        "court is vacuous: fewer than 2 dialects parsed the pure graph"
    );
    let first = &ids[0];
    for (i, id) in ids.iter().enumerate() {
        assert_eq!(
            id, first,
            "dialect {i} drifted from dialect 0 on identical content"
        );
    }
}

#[test]
fn dataset_court_named_graph_id_matches_across_trig_and_nquads() {
    let mut ledger = Ledger::default();
    let trig = admit_or_skip(
        &mut ledger,
        "trig",
        DATASET_TRIG,
        Dialect::TriG,
        "dataset court",
    );
    let nq = admit_or_skip(
        &mut ledger,
        "nquads",
        DATASET_NQ,
        Dialect::NQuads,
        "dataset court",
    );
    eprintln!(
        "dataset-court receipt: passes={:?} skips={:?}",
        ledger.pass, ledger.skip
    );
    match (trig, nq) {
        (Some(a), Some(b)) => {
            assert_eq!(a.quad_count(), 3, "trig named-graph quad count");
            assert_eq!(b.quad_count(), 3, "nquads named-graph quad count");
            assert_eq!(
                a.id(),
                b.id(),
                "same named-graph contents in trig vs nquads must share one id"
            );
        }
        (a, b) => panic!(
            "dataset court could not run: trig={:?} nquads={:?} (skips={:?})",
            a.is_some(),
            b.is_some(),
            ledger.skip
        ),
    }
}

#[test]
fn dataset_court_named_graph_differs_from_default_graph() {
    let named = LawState::parse(DATASET_NQ.as_bytes(), Dialect::NQuads, None)
        .expect("nquads named-graph fixture must parse");
    let default = LawState::parse(DEFAULT_GRAPH_NQ.as_bytes(), Dialect::NQuads, None)
        .expect("nquads default-graph fixture must parse");
    assert_ne!(
        named.id(),
        default.id(),
        "identical triples in a named graph vs the default graph MUST yield different ids"
    );
}

#[test]
fn blank_node_court_relabeling_and_cross_dialect_id() {
    // Relabeling within one dialect must not move identity.
    let a = LawState::parse(BNODE_NT.as_bytes(), Dialect::NQuads, None)
        .expect("bnode nquads fixture must parse");
    let b = LawState::parse(BNODE_NT_RELABELED.as_bytes(), Dialect::NQuads, None)
        .expect("relabeled bnode nquads fixture must parse");
    assert_eq!(
        a.id(),
        b.id(),
        "blank-node relabeling must not move identity"
    );

    // Cross-dialect: bnode graph parsed as NTriples vs NQuads (default graph)
    // must land on the same canonical id (extends canonical_abi bnode court).
    let mut ledger = Ledger::default();
    let nt = admit_or_skip(
        &mut ledger,
        "ntriples",
        BNODE_NT,
        Dialect::NTriples,
        "bnode cross-dialect court",
    );
    eprintln!(
        "bnode-court receipt: passes={:?} skips={:?}",
        ledger.pass, ledger.skip
    );
    if let Some(nt) = nt {
        assert_eq!(
            a.id(),
            nt.id(),
            "bnode graph identity must not depend on triples-vs-quads surface dialect"
        );
    }
}

#[test]
fn n3_is_refused_unsupported_for_canonical() {
    let r = LawState::parse(N3_DOC.as_bytes(), Dialect::N3, None);
    let err = r.expect_err("N3 rule document must be refused by the canonical path");
    assert_eq!(
        err.kind,
        RefusalKind::Unsupported,
        "N3 refusal must be typed Unsupported, got {:?}: {}",
        err.kind,
        err.message
    );
}

#[test]
fn nonvacuity_one_quad_perturbation_moves_id() {
    let base = LawState::parse(DEFAULT_GRAPH_NQ.as_bytes(), Dialect::NQuads, None)
        .expect("base fixture must parse");
    let perturbed_text = DEFAULT_GRAPH_NQ.replace("example.org/carol", "example.org/mallory");
    let perturbed = LawState::parse(perturbed_text.as_bytes(), Dialect::NQuads, None)
        .expect("perturbed fixture must parse");
    assert_ne!(
        base.id(),
        perturbed.id(),
        "court is VACUOUS: a one-quad perturbation failed to move the canonical id"
    );
}
