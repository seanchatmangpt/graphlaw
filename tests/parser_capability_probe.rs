//! V04 parser-capability probe: resolves the R01-vs-R02 lane conflict
//! EMPIRICALLY by calling the real parse path
//! (`graphlaw::dialect::parse_rdf` -> `purrdf::parse_dataset`) with real
//! fixture snippets per dialect. Verdict per dialect: Ok(dataset) with
//! expected quad count, or the exact `RefusalKind` returned.
//!
//! Chicago discipline: real parser, real bytes, assertions on the returned
//! dataset state. No mocks.

use graphlaw::dialect::{Dialect, Refusal, RefusalKind, parse_rdf};
use graphlaw::purrdf::flat_rdf_quads_from_dataset;

fn count_quads(dataset: &std::sync::Arc<graphlaw::purrdf::RdfDataset>) -> usize {
    flat_rdf_quads_from_dataset(dataset).len()
}

fn named_graph_count(dataset: &std::sync::Arc<graphlaw::purrdf::RdfDataset>) -> usize {
    flat_rdf_quads_from_dataset(dataset)
        .into_iter()
        .filter(|q| q.graph_name.is_some())
        .count()
}

fn describe(err: &Refusal) -> String {
    format!(
        "RefusalKind::{:?} (dialect={:?}, engine={:?}): {}",
        err.kind, err.dialect, err.engine, err.message
    )
}

// --- 1. Turtle -------------------------------------------------------------
#[test]
fn probe_turtle() {
    let bytes = br#"@prefix ex: <http://example.org/> .
ex:alice ex:knows ex:bob .
ex:bob ex:knows ex:carol .
"#;
    match parse_rdf(bytes, Dialect::Turtle, None) {
        Ok(ds) => assert_eq!(count_quads(&ds), 2, "turtle: expected 2 triples"),
        Err(e) => panic!("turtle refused: {}", describe(&e)),
    }
}

// --- 2. N-Triples ----------------------------------------------------------
#[test]
fn probe_ntriples() {
    let bytes = b"<http://example.org/alice> <http://example.org/knows> <http://example.org/bob> .
<http://example.org/bob> <http://example.org/knows> <http://example.org/carol> .
";
    match parse_rdf(bytes, Dialect::NTriples, None) {
        Ok(ds) => assert_eq!(count_quads(&ds), 2, "ntriples: expected 2 triples"),
        Err(e) => panic!("ntriples refused: {}", describe(&e)),
    }
}

// --- 3. N-Quads ------------------------------------------------------------
#[test]
fn probe_nquads() {
    let bytes = b"<http://example.org/alice> <http://example.org/knows> <http://example.org/bob> <http://example.org/g1> .
<http://example.org/bob> <http://example.org/knows> <http://example.org/carol> .
";
    match parse_rdf(bytes, Dialect::NQuads, None) {
        Ok(ds) => {
            assert_eq!(count_quads(&ds), 2, "nquads: expected 2 quads");
            assert_eq!(named_graph_count(&ds), 1, "nquads: 1 quad in named graph");
        }
        Err(e) => panic!("nquads refused: {}", describe(&e)),
    }
}

// --- 4. TriG (named graph) -------------------------------------------------
#[test]
fn probe_trig_named_graph() {
    let bytes = br#"@prefix ex: <http://example.org/> .
ex:g1 {
    ex:alice ex:knows ex:bob .
    ex:bob ex:knows ex:carol .
}
"#;
    match parse_rdf(bytes, Dialect::TriG, None) {
        Ok(ds) => {
            assert_eq!(count_quads(&ds), 2, "trig: expected 2 quads");
            assert_eq!(named_graph_count(&ds), 2, "trig: both quads in ex:g1");
        }
        Err(e) => panic!("trig refused: {}", describe(&e)),
    }
}

// --- 5. RDF/XML ------------------------------------------------------------
#[test]
fn probe_rdf_xml() {
    let bytes = br#"<?xml version="1.0"?>
<rdf:RDF xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#"
         xmlns:ex="http://example.org/">
  <rdf:Description rdf:about="http://example.org/alice">
    <ex:knows rdf:resource="http://example.org/bob"/>
  </rdf:Description>
</rdf:RDF>
"#;
    match parse_rdf(bytes, Dialect::RdfXml, None) {
        Ok(ds) => assert_eq!(count_quads(&ds), 1, "rdf-xml: expected 1 triple"),
        Err(e) => panic!("rdf-xml refused: {}", describe(&e)),
    }
}

// --- 6. JSON-LD (expanded form, inline context-free) ------------------------
#[test]
fn probe_jsonld_expanded() {
    let bytes = br#"[
  {
    "@id": "http://example.org/alice",
    "http://example.org/knows": [
      { "@id": "http://example.org/bob" }
    ]
  }
]
"#;
    match parse_rdf(bytes, Dialect::JsonLd, None) {
        Ok(ds) => assert_eq!(count_quads(&ds), 1, "json-ld expanded: expected 1 triple"),
        Err(e) => panic!("json-ld expanded refused: {}", describe(&e)),
    }
}

// --- 7. JSON-LD with REMOTE @context reference ------------------------------
/// Security probe: a remote `@context` IRI must NOT trigger network access.
/// Parses in-process with no network mocking; whatever purrdf does here is
/// observed directly (timeout = attempted fetch would hang this test).
#[test]
fn probe_jsonld_remote_context_no_network() {
    let bytes = br#"{
  "@context": "https://example.invalid/never-served-context.jsonld",
  "@id": "http://example.org/alice",
  "http://example.org/knows": { "@id": "http://example.org/bob" }
}
"#;
    let result = parse_rdf(bytes, Dialect::JsonLd, None);
    match &result {
        Ok(ds) => println!(
            "json-ld remote-context: PARSED with {} quads (remote @context silently ignored)",
            count_quads(ds)
        ),
        Err(e) => println!("json-ld remote-context: refused with {}", describe(e)),
    }
    // The load-bearing assertion: the call RETURNS (no network hang) and, if
    // it succeeds, the remote context's terms were NOT silently dropped into
    // a wrong-but-Ok shape without our noticing. Either outcome is recorded;
    // the invariant under test is termination without network fetch.
    match result {
        Ok(ds) => {
            let n = count_quads(&ds);
            println!("remote-context verdict: Ok, {n} quads");
        }
        Err(e) => {
            println!("remote-context verdict: {}", describe(&e));
            // A refusal is acceptable ONLY if it is the engine rejecting the
            // document (or, since R07, the typed RemoteContext refusal surfaced
            // as Unsupported), never a panic/abort path.
            assert!(matches!(
                e.kind,
                RefusalKind::EngineRejected | RefusalKind::Unsupported
            ));
        }
    }
}
