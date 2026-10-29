//! N3 conformance corpus court — executes the upstream `purrdf` corpora/n3
//! cases through graphlaw's own admission surface (`graphlaw::n3`), closing
//! the DoD D2 item "corpus witnessed under graphlaw gates".
//!
//! The corpus inventory (`corpora/n3/n3/inventory.json`) is the driver: cases
//! are discovered, classified by `kind`, and executed — no case names are
//! hardcoded, so a corpus extension automatically extends this court.
//!
//! Kinds (per the upstream conformance-kit decoder contract):
//! - `query-evaluation`: parse the data document; the parsed fact set must
//!   equal the parsed expected artifact (N-Triples or N3), as sets.
//! - `rules`: `PreparedReasoner` over rules.n3, reasoning over facts.n3;
//!   every expected-inference triple must be present in derived ∪ explicit.
//! - `negative-syntax`: the document must refuse with a typed error.
//!
//! Chicago: real parser, real reasoner, real files; assertions on real output
//! state. The corpus tree is read cross-repo (sibling canonical checkout,
//! one-canonical-checkout law) — never written.

use graphlaw::n3::{parse_n3, PreparedReasoner, ReasonerOptions, Triple};
use serde_json::Value;
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

const CORPUS: &str = "/Users/sac/purrdf/corpora/n3";

fn corpus_root() -> PathBuf {
    PathBuf::from(CORPUS)
}

fn read(rel: &str) -> String {
    std::fs::read_to_string(corpus_root().join(rel))
        .unwrap_or_else(|e| panic!("corpus read {rel}: {e}"))
}

fn triple_key(t: &Triple) -> String {
    format!("{:?}", (t.s.clone(), t.p.clone(), t.o.clone()))
}

fn fact_set(doc: &graphlaw::n3::Document) -> BTreeSet<String> {
    doc.facts.iter().map(triple_key).collect()
}

fn expected_set(text: &str) -> BTreeSet<String> {
    let doc = parse_n3(text, None).expect("expected artifact must parse");
    fact_set(&doc)
}

#[test]
fn corpus_inventory_drives_the_court() {
    let inv: Value = serde_json::from_str(&read("n3/inventory.json")).expect("inventory JSON");
    assert_eq!(
        inv["contractVersion"], Value::from(1),
        "corpus contract version"
    );
    let cases = inv["cases"].as_array().expect("cases array");
    assert!(
        cases.len() >= 9,
        "corpus must carry at least the 9 registered cases, got {}",
        cases.len()
    );
    let mut kinds = std::collections::BTreeMap::new();
    for case in cases {
        let id = case["id"].as_str().expect("case id");
        let kind = case["kind"].as_str().expect("case kind");
        *kinds.entry(kind.to_string()).or_insert(0usize) += 1;
        match kind {
            "query-evaluation" => run_query_evaluation(id, case),
            "rules" => run_rules(id, case),
            "negative-syntax" => run_negative(id, case),
            other => panic!("unknown corpus kind {other} for {id} — extend this court"),
        }
    }
    assert!(kinds.contains_key("query-evaluation"), "positive parses covered");
    let registered_hits = cases
        .iter()
        .filter(|c| is_registered(c["id"].as_str().expect("id")))
        .count();
    assert_eq!(
        registered_hits,
        KNOWN_DIVERGENCES.len(),
        "register rows must match live corpus cases exactly (a case that passes now \
         breaks this count — close the divergence, remove the row)"
    );
    assert!(kinds.contains_key("rules"), "rule derivations covered");
    assert!(kinds.contains_key("negative-syntax"), "negative syntax covered");
}

fn case_data_path(case: &Value) -> String {
    let data = case["data"].as_array().expect("data array");
    let first = &data[0];
    first["path"].as_str().expect("data path").to_string()
}

/// Engine-vs-corpus divergence register (the tests/w3c_conformance.rs
/// protocol): a registered case must keep failing exactly as described — if
/// the engine later implements the proposed semantics, the case passes, the
/// register counts break, and this court forces the register row's removal.
/// Registering requires a named semantic reason, never a bare skip.
const KNOWN_DIVERGENCES: &[(&str, &str)] = &[
    (
        "literals-collections-conforming",
        "corpus expected artifact materializes `( 1 2 3 )` as rdf:first/rdf:rest \
         blank-node collections (RDF-level spelling, independently authored); the \
         engine's document model keeps N3 lists as first-class Term::List values \
         (N3-submission dialect) — the fact-level comparison therefore differs by \
         design until/unless the engine gains an RDF-materialization projection.",
    ),
    (
        "prefixes-base-conforming",
        "corpus n3t:proposed semantics: undeclared default prefix `:name` resolving          against @base (manifest comment: \"prefixed names stay on the corpus base\").          The engine requires an explicit `@prefix :` declaration (typed UnknownPrefix) —          base-as-default-prefix is not implemented.",
    ),
];

fn is_registered(id: &str) -> bool {
    KNOWN_DIVERGENCES.iter().any(|(c, _)| *c == id)
}

fn run_query_evaluation(id: &str, case: &Value) {
    let data = read(&case_data_path(case));
    let expected_rel = case["expected"].as_str().expect("expected path");
    let expected_text = read(expected_rel);
    let parse_outcome = parse_n3(&data, None);
    if id == "prefixes-base-conforming" {
        let err = parse_outcome.err().unwrap_or_else(|| {
            panic!(
                "{id}: registered divergence broken — the engine now accepts this case; \
                 remove the register row and require the full comparison"
            )
        });
        assert!(
            err.message.to_lowercase().contains("prefix"),
            "{id}: registered divergence must fail on the prefix mechanism, got: {}",
            err.message
        );
        return;
    }
    let doc = parse_outcome.unwrap_or_else(|e| panic!("{id}: must parse: {}", e.message));
    let got = fact_set(&doc);
    let want = expected_set(&expected_text);
    if id == "literals-collections-conforming" {
        assert_ne!(
            got, want,
            "{id}: registered divergence broken — the engine now materializes RDF \
             collections like the expected artifact; remove the register row and \
             require the full comparison"
        );
        return;
    }
    assert_eq!(
        got, want,
        "{id}: parsed facts must equal the expected artifact byte-semantically"
    );
}

fn run_rules(id: &str, case: &Value) {
    // rules cases carry rules.n3 + facts.n3 + expected-inferences.n3 in the
    // case directory (the manifest path locates it).
    let manifest = case["manifest"].as_str().expect("manifest");
    let base = Path::new(manifest).parent().expect("case dir from manifest path");
    let rules = read(&base.join("rules.n3").to_string_lossy());
    let facts = read(&base.join("facts.n3").to_string_lossy());
    let expected = read(&base.join("expected-inferences.n3").to_string_lossy());

    let rules_doc = parse_n3(&rules, None).unwrap_or_else(|e| panic!("{id}: rules parse: {}", e.message));
    let facts_doc = parse_n3(&facts, None).unwrap_or_else(|e| panic!("{id}: facts parse: {}", e.message));

    let prepared = PreparedReasoner::new(rules_doc);
    let result = prepared.reason(&facts_doc, &ReasonerOptions::default());
    assert!(
        result.is_complete(),
        "{id}: rules case must reach a complete fixpoint (limits: {:?})",
        result.limits_reached
    );

    let mut have: BTreeSet<String> = result
        .derived
        .iter()
        .chain(result.explicit.iter())
        .map(triple_key)
        .collect();
    for t in &facts_doc.facts {
        have.insert(triple_key(t));
    }
    let want = expected_set(&expected);
    let missing: Vec<&String> = want.difference(&have).collect();
    assert!(
        missing.is_empty(),
        "{id}: expected inferences absent from derivation: {missing:?}"
    );
}

fn run_negative(id: &str, case: &Value) {
    let data = read(&case_data_path(case));
    let err = parse_n3(&data, None)
        .err()
        .unwrap_or_else(|| panic!("{id}: negative-syntax case must be REFUSED, not parsed"));
    assert!(!err.message.is_empty(), "{id}: refusal must carry a message");
}
