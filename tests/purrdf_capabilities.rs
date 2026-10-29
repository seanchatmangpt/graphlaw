//! Chicago capability court for the `graphlaw::purrdf` facade (pinned purrdf 3.0.1).
//!
//! Chicago (classicist) discipline: every leg below drives REAL collaborators —
//! the pinned purrdf crates through GraphLaw's unstable re-export
//! (`src/lib.rs`: `pub use purrdf;`) — and asserts on final state: parsed
//! datasets, folded graphs, SARIF bytes, engine answer rows. No mocks, no
//! stubs, no sleeps. A leg that merely NAMES a type is not here; each one
//! calls through the facade and reads the answer back.
//!
//! Structural falsifier (anti-vacuity at file scope): every path in this file
//! routes through `graphlaw::purrdf::`. Deleting `pub use purrdf;` from
//! `src/lib.rs` must turn this file's compile green into red. A bare
//! `purrdf::` extern path would make that falsifier vacuous, so there are
//! none.
//!
//! Per-leg anti-vacuity: each test's doc comment names the operator inversion
//! that must flip it. A court that cannot flip is not a court.
//!
//! Pinned surface: purrdf 3.0.1 (git: seanchatmangpt/purrdf, rev-pinned). Legs that
//! exist only on newer upstream pins are TODO-annotated, not silently absent.

// The whole test file imports the facade ONCE, under the graphlaw path. No
// `extern crate purrdf`, no bare `purdff::` anywhere below.
use graphlaw::purrdf;
use graphlaw::purrdf::{
    ContentStore, NativeRdfFormat, RdfDataset, RdfDatasetBuilder, RdfLiteral, SparqlEngine,
    SparqlRequest, SparqlResult, TermRef, TermValue, parse_dataset, serialize_dataset_to_format,
};
use std::ops::ControlFlow;
use std::sync::Arc;

// ── shared fixture helpers ──────────────────────────────────────────────────

/// Render one solution cell so row assertions can be exact strings: the IRI
/// for named nodes, the lexical form for literals, `UNBOUND` for `None`.
fn cell(t: &Option<TermValue>) -> String {
    match t {
        Some(TermValue::Iri(iri)) => iri.clone(),
        Some(TermValue::Literal { lexical_form, .. }) => lexical_form.clone(),
        Some(other) => format!("{other:?}"),
        None => "UNBOUND".to_owned(),
    }
}

/// A SELECT result's rows as string grids, in engine order.
fn rows(result: SparqlResult) -> Vec<Vec<String>> {
    let SparqlResult::Solutions {
        variables,
        rows,
        ..
    } = result
    else {
        panic!("expected a SELECT result, got {result:?}");
    };
    let _ = &variables;
    rows.iter()
        .map(|row| row.iter().map(cell).collect())
        .collect()
}

/// Evaluate `query` against `dataset` through the facade's native engine.
fn ask(
    dataset: &Arc<RdfDataset>,
    query: &str,
) -> Result<SparqlResult, graphlaw::purrdf::RdfDiagnostic> {
    use graphlaw::purrdf::sparql::NativeSparqlEngine;
    NativeSparqlEngine::new().query(
        dataset,
        SparqlRequest {
            query,
            base_iri: None,
            substitutions: &[],
        },
    )
}

// ── 1. root codecs + document base ──────────────────────────────────────────

/// `parse_dataset` ingests Turtle (including `@base`-resolved relative IRIs),
/// `serialize_dataset_to_format` re-emits it, and a relative reference with
/// nothing in scope is a HARD failure carrying the shared diagnostic code —
/// never a fabricated base.
///
/// Anti-vacuity: weakening the ingest to invent a default base, or turning the
/// no-base refusal into an empty parse, flips either the exact-quad assertion
/// or the exact `iri-relative-no-base` code match.
#[test]
fn root_codecs_roundtrip_and_relative_no_base_hard_fails() {
    let turtle = r#"
@prefix ex: <https://example.org/> .
@base <https://example.org/base/> .
<rel> <https://example.org/p> <https://example.org/o> .
ex:cat ex:says "meow" .
"#;
    let dataset = parse_dataset(turtle.as_bytes(), "text/turtle", None).expect("valid Turtle");
    assert_eq!(dataset.quad_count(), 2, "one relative-IRI quad, one literal quad");

    // Egress round trip: the same bytes fold back to the same answer.
    let out = serialize_dataset_to_format(&*dataset, NativeRdfFormat::Turtle, None)
        .expect("Turtle egress");
    let text = String::from_utf8(out.bytes).expect("utf-8 Turtle");
    let reparsed =
        parse_dataset(text.as_bytes(), "text/turtle", None).expect("reparse of own egress");
    assert_eq!(reparsed.quad_count(), 2);
    assert!(text.contains("meow"), "the literal survives the round trip");

    // No base in scope: the relative reference refuses by its shared code.
    let doc = "<rel> <https://example.org/p> <https://example.org/o> .\n";
    let error = parse_dataset(doc.as_bytes(), "text/turtle", None).expect_err("no base");
    assert_eq!(error.code, "iri-relative-no-base");
}

// ── 2. columnar: five-table Parquet codec ───────────────────────────────────

/// `columnar::write` projects a real dataset + blob store into the five
/// Parquet tables and `columnar::read` folds the files back into an equal
/// dataset.
///
/// Anti-vacuity: a writer that emitted no quad rows, or a reader that always
/// returned an empty dataset, flips the quad-count readback; `Table::ALL`
/// shrinking below five flips the schema-count pin.
#[test]
fn columnar_write_read_roundtrip_pins_five_tables() {
    use purrdf::columnar::{Compression, Table};

    let mut builder = RdfDatasetBuilder::new();
    let s = builder.intern_iri("https://example.org/s").expect("intern");
    let p = builder.intern_iri("https://example.org/p").expect("intern");
    let o = builder.intern_iri("https://example.org/o").expect("intern");
    let lit = builder.intern_literal(RdfLiteral::simple("payload")).expect("intern");
    builder.push_quad(s, p, o, None);
    builder.push_quad(s, p, lit, None);
    let dataset = builder.freeze().expect("fixture validates");

    let blobs = ContentStore::new();
    let written = purrdf::columnar::write(&dataset, &blobs, Compression::Uncompressed)
        .expect("five-table projection");
    assert_eq!(Table::ALL.len(), 5, "the fixed five-table schema");

    let readback = purrdf::columnar::read(&written.files).expect("fold Parquet back");
    assert_eq!(readback.dataset.quad_count(), 2, "both quads survive");
    assert!(readback.blobs.is_empty(), "no blobs were projected");
}

// ── 3. gts: container engine + RDF adapter ──────────────────────────────────

/// The GTS `Writer` serializes folded graph state and the facade-level
/// `read_graph` adapter folds the container bytes back to the same terms and
/// quads. Empty input is not a GTS graph and errs.
///
/// Anti-vacuity: a writer that emitted nothing, or a reader that folded any
/// byte string, flips the term/quad readback or the empty-input error.
#[test]
fn gts_writer_to_read_graph_fold() {
    use purrdf::gts;
    use purrdf::gts::model::{Graph, Term, TermKind};

    let mut graph = Graph::default();
    for value in [
        "https://example.org/s",
        "https://example.org/p",
        "https://example.org/o",
    ] {
        graph.terms.push(Term {
            kind: TermKind::Iri,
            value: Some(value.to_owned()),
            datatype: None,
            lang: None,
            direction: None,
            reifier: None,
            triple: None,
        });
    }
    graph.quads.push((0, 1, 2, None));

    let bytes = gts::writer::Writer::deterministic(&graph, "purrdf-test")
        .expect("deterministic GTS writer")
        .into_bytes();
    assert!(!bytes.is_empty(), "the writer emitted a container");

    let folded = gts::read_graph(&bytes, false).expect("the fold answers");
    assert!(folded.diagnostics.is_empty());
    assert_eq!(folded.quads.len(), 1, "the quad row survives");
    assert_eq!(folded.terms.len(), 3, "all three terms survive");
    assert!(
        folded
            .terms
            .iter()
            .any(|term| term.value.as_deref() == Some("https://example.org/o")),
        "the object term is read back by value"
    );

    assert!(
        gts::read_graph(&[], false).is_err(),
        "empty input is not a GTS graph"
    );
}

// ── 4. sparql: aggregate evaluation ─────────────────────────────────────────

/// COUNT under GROUP BY under ORDER BY answers the exact bag: engineering has
/// two members, ops has one, ordered by department.
///
/// Anti-vacuity: an engine that returned unordered bags, miscounted groups, or
/// dropped the projection flips the exact row grid.
#[test]
fn sparql_count_group_by_order_exact_rows() {
    let mut builder = RdfDatasetBuilder::new();
    let dept = builder.intern_iri("https://example.org/dept").expect("intern");
    let eng = builder.intern_literal(RdfLiteral::simple("eng")).expect("intern");
    let ops = builder.intern_literal(RdfLiteral::simple("ops")).expect("intern");
    for (id, team) in [("a", "eng"), ("b", "ops"), ("c", "eng")] {
        let (e, d) = (
            builder.intern_iri(&format!("https://example.org/{id}")).expect("intern"),
            if team == "ops" { ops } else { eng },
        );
        builder.push_quad(e, dept, d, None);
    }
    let dataset = builder.freeze().expect("fixture validates");

    let result = ask(
        &dataset,
        "PREFIX ex: <https://example.org/> \
         SELECT ?dept (COUNT(?e) AS ?n) WHERE { ?e ex:dept ?dept } \
         GROUP BY ?dept ORDER BY ?dept",
    )
    .expect("the aggregate answers");
    assert_eq!(
        rows(result),
        vec![
            vec!["eng".to_owned(), "2".to_owned()],
            vec!["ops".to_owned(), "1".to_owned()],
        ],
        "exact groups, exact counts, exact order"
    );
}

// ── 5. shapes: SHACL MinCount ───────────────────────────────────────────────

/// `shapes::engine::validate_graphs` runs real SHACL over a shapes+data pair:
/// the one node missing `ex:name` is the one violation, carried under the
/// MinCount component IRI.
///
/// Anti-vacuity: a validator that always conforms, or that reports violations
/// without their component, flips the count or the component-IRI match.
#[test]
fn shapes_mincount_violation_count_and_component() {
    use purrdf::shapes;

    let data = r#"<https://example.org/alice> <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <https://example.org/Person> .
<https://example.org/bob> <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <https://example.org/Person> .
<https://example.org/alice> <https://example.org/name> "Alice" .
"#;
    let shapes_ttl = r#"
@prefix sh: <http://www.w3.org/ns/shacl#> .
<https://example.org/PersonShape> a sh:NodeShape ;
    sh:targetClass <https://example.org/Person> ;
    sh:property [
        sh:path <https://example.org/name> ;
        sh:minCount 1 ;
    ] .
"#;
    let report = shapes::engine::validate_graphs(data, shapes_ttl, None).expect("validation runs");
    assert!(!report.conforms, "bob has no ex:name");
    assert_eq!(report.results.len(), 1, "exactly one violation");
    let violation = &report.results[0];
    assert_eq!(
        violation.source_constraint_component.as_str(),
        "http://www.w3.org/ns/shacl#MinCountConstraintComponent",
        "the component IRI names the constraint that fired"
    );
    assert_eq!(violation.focus_value(), "https://example.org/bob");
}

// ── 6. shex: ShExC round trip ───────────────────────────────────────────────

/// `parse_shexc → to_shexc → parse_shexc` is the identity on the serialized
/// form: the serializer emits absolute IRIs, so the re-parse must reproduce it
/// byte for byte.
///
/// Anti-vacuity: a serializer that emitted relative references (the re-parse
/// fails) or a parser that accepted its own output only loosely (the strings
/// diverge) flips the identity assertion.
#[test]
fn shex_parse_serialize_reparse_identity() {
    use purrdf::shex;

    let schema = "PREFIX ex: <https://example.org/>\nex:Cat { ex:says . }";
    let first = shex::parse_shexc(schema, None).expect("valid ShExC");
    let serialized = shex::to_shexc(&first);
    let second = shex::parse_shexc(&serialized, None).expect("the serializer's own output");
    assert_eq!(
        shex::to_shexc(&second),
        serialized,
        "reparse is the identity on the serialized form"
    );
}

// ── 7. entail: RDFS closure, combined surrogates, OWL-Direct vs Simple ─────

/// Three real reasoning answers over one fixture: rdfs9 re-types the instance
/// (and fires, by rule id); the combined approach lowers the Horn TBox and
/// reports an empty witness set (there is no existential axiom to witness);
/// and the OWL-Direct lane answers through its own regime where the Simple
/// lane is a bare copy.
///
/// Anti-vacuity: a closure that dropped rdfs9, a combined lane that fabricated
/// witnesses, or an OWL-Direct call that silently degraded to Simple flips the
/// rule-id / surrogates / regime assertions.
#[test]
fn entail_rdfs9_surrogates_and_owl_direct_vs_simple() {
    use purrdf::entail::{self, Materialization, Regime, RuleId};

    const SUBCLASS: &str = "http://www.w3.org/2000/01/rdf-schema#subClassOf";
    const TYPE: &str = "http://www.w3.org/1999/02/22-rdf-syntax-ns#type";
    const CAT: &str = "https://example.org/Cat";
    const ANIMAL: &str = "https://example.org/Animal";
    const TOM: &str = "https://example.org/tom";

    let mut builder = RdfDatasetBuilder::new();
    let (cat, animal, tom) = (
        builder.intern_iri(CAT).expect("intern"),
        builder.intern_iri(ANIMAL).expect("intern"),
        builder.intern_iri(TOM).expect("intern"),
    );
    let (sub, ty) = (
        builder.intern_iri(SUBCLASS).expect("intern"),
        builder.intern_iri(TYPE).expect("intern"),
    );
    builder.push_quad(cat, sub, animal, None);
    builder.push_quad(tom, ty, cat, None);
    let dataset = builder.freeze().expect("fixture validates");

    // rdfs9: the instance is re-typed through the subclass edge.
    let (closure, report) = entail::materialize(&dataset, Materialization::Rdfs).expect("rdfs");
    assert!(
        report.rules_fired().iter().any(|&(rule, n)| rule == RuleId::Rdfs9 && n >= 1),
        "rdfs9 is named in the fired-rule table"
    );
    assert!(
        closure.quad_refs().any(|quad| {
            matches!(quad.s, TermRef::Iri(s) if s == TOM)
                && matches!(quad.p, TermRef::Iri(p) if p == TYPE)
                && matches!(quad.o, TermRef::Iri(o) if o == ANIMAL)
        }),
        "tom is an Animal in the closed dataset"
    );

    // The combined approach lowers this Horn TBox (no existential axioms) and
    // reports exactly zero surrogate witnesses.
    let bgp = [entail::QTriple {
        s: entail::QNode::Term(TermValue::iri(TOM)),
        p: entail::QNode::Term(TermValue::iri(TYPE)),
        o: entail::QNode::Var("c".to_owned()),
    }];
    let combined = entail::materialize_combined(&dataset, &bgp)
        .expect("the combined lane runs")
        .expect("the TBox lowers");
    assert!(
        combined.surrogates.is_empty(),
        "an axiom-free TBox mints no existential witnesses"
    );

    // Simple is the bare copy; OWL-Direct answers under its own regime name
    // and never shrinks the answer.
    let (simple, _) = entail::materialize(&dataset, Materialization::Simple).expect("simple");
    assert_eq!(simple.quad_count(), 2, "an identity closure adds nothing");
    let (dl, dl_report) =
        entail::materialize(&dataset, Materialization::OwlDirect(&[])).expect("owl-direct");
    assert_eq!(dl_report.regime(), Regime::OwlDirect);
    assert!(dl.quad_count() >= simple.quad_count());
}

// ── 8. geo: geof: over the SPARQL scalar seam ───────────────────────────────

/// The `geof:` family answers through real query text with the caller-supplied
/// vocabulary: `sfWithin` keeps exactly the inside point and the square itself,
/// `distance` emits the exact `xsd:double` lexical form, and a registered-but-
/// unimplemented function aborts the query BY NAME rather than answering.
///
/// Anti-vacuity: a topological predicate that answered false for everything,
/// a distance that rounded differently, or an unimplemented function that
/// answered `false`/`unbound` instead of refusing each flip one assertion.
#[test]
fn geo_sfwithin_distance_and_unimplemented_abort_by_name() {
    use purrdf::geo::functions;
    use purrdf::geo::vocab::{GeoVocab, GeoVocabBuilder};
    use purrdf::geo::{Crs, GeoTerm};
    use purrdf::sparql::{QueryOptions, UserFunctionRegistry};

    const GEO: &str = "http://example.org/geo#";
    const GEOF: &str = "http://example.org/geof/";
    const CRS: &str = "http://example.org/crs/planar";
    const METRE: &str = "http://example.org/unit/metre";
    const SF: &str = "http://example.org/sf#";
    const XSD_DOUBLE: &str = "http://www.w3.org/2001/XMLSchema#double";

    let crs = Crs::new(CRS).expect("an absolute CRS IRI");
    let vocab: GeoVocab = GeoVocabBuilder::new(GEO, GEOF, crs.clone(), crs.clone())
        .expect("non-empty namespaces")
        .declare_crs_unit(&crs, METRE)
        .expect("a fresh declaration")
        .declare_metre(METRE)
        .expect("a non-empty metre IRI")
        .declare_simple_features_namespace(SF)
        .expect("a non-empty namespace")
        .build();
    let engine = graphlaw::purrdf::sparql::NativeSparqlEngine::new();
    let bound_registry = engine
        .bind_functions(
            {
                let mut reg = UserFunctionRegistry::default();
                functions::register(&mut reg, &vocab);
                reg
            },
            graphlaw::purrdf::sparql::ExtensionEnv::empty(),
        )
        .expect("native functions bind");

    let mut builder = RdfDatasetBuilder::new();
    let geom = builder.intern_iri(&format!("{GEO}geom")).expect("intern");
    for (id, wkt) in [
        ("a", "POINT(1 1)"),    // inside the square
        ("b", "POINT(9 9)"),    // outside it
        ("c", "POLYGON((0 0,4 0,4 4,0 4,0 0))"), // the square itself
    ] {
        let (s, o) = (
            builder.intern_iri(&format!("https://example.org/{id}")).expect("intern"),
            builder.intern_literal(
                RdfLiteral::typed(format!("<{CRS}> {wkt}"), format!("{GEO}wktLiteral")),
            ).expect("intern"),
        );
        builder.push_quad(s, geom, o, None);
    }
    let dataset = builder.freeze().expect("fixture validates");

    let evaluate = |query: &str| -> Result<SparqlResult, graphlaw::purrdf::RdfDiagnostic> {
        engine.query_with_options_view(
            &*dataset,
            SparqlRequest {
                query,
                base_iri: None,
                substitutions: &[],
            },
            QueryOptions::new().with_functions(&bound_registry),
        )
    };

    // sfWithin keeps the inside point and the reflexive square; the outside
    // point and the malformed literal's row are dropped, and the query still
    // answers.
    let within = format!(
        "SELECT ?s WHERE {{ \
         ?s <{GEO}geom> ?wkt . <https://example.org/c> <{GEO}geom> ?other . \
         FILTER(<{GEOF}sfWithin>(?wkt, ?other)) }} ORDER BY ?s"
    );
    assert_eq!(
        rows(evaluate(&within).expect("the filter answers")),
        vec![
            vec!["https://example.org/a".to_owned()],
            vec!["https://example.org/c".to_owned()],
        ],
        "the inside point and the square itself are within it"
    );

    // distance: the 3-4-5 triangle's hypotenuse as the canonical xsd:double
    // LEXICAL form — bytes, not a numeric comparison.
    let distance = format!(
        "SELECT ?d WHERE {{ BIND(<{GEOF}distance>(\"<{CRS}> POINT(0 0)\"^^<{GEO}wktLiteral>, \
         \"<{CRS}> POINT(3 4)\"^^<{GEO}wktLiteral>, <{METRE}>) AS ?d) }}"
    );
    let SparqlResult::Solutions { rows: got, .. } =
        evaluate(&distance).expect("distance answers")
    else {
        panic!("expected SELECT solutions");
    };
    let _ = purrdf::geo::GeoTerm::WktLiteral; // named through the facade; the answer is asserted as text
    assert_eq!(got.len(), 1);
    match &got[0][0] {
        Some(TermValue::Literal {
            lexical_form,
            datatype,
            ..
        }) => {
            assert_eq!(lexical_form, "5.0E0", "the exact canonical lexical form");
            assert_eq!(datatype.as_str(), XSD_DOUBLE);
        }
        other => panic!("expected an xsd:double literal, got {other:?}"),
    }

    // `geof:transform` is registered and deliberately unimplemented: it fails
    // the query BY NAME instead of answering.
    let transform = format!(
        "SELECT ?g WHERE {{ BIND(<{GEOF}transform>(\"<{CRS}> POINT(0 0)\"^^<http://www.opengis.net/ont/geosparql#wktLiteral>, <{CRS}>) AS ?g) }}"
    );
    let error = evaluate(&transform).expect_err("unimplemented geof: must refuse");
    let rendered = format!("{error:?}");
    assert!(
        rendered.contains("transform"),
        "the refusal names the function: {rendered}"
    );
}

// ── 9. text: deterministic full-text search ────────────────────────────────

/// Verbatim from the pinned umbrella (`purrdf-2.0.2/src/lib.rs:324-359`,
/// `facade_exposes_full_text_search`), routed through `graphlaw::purrdf`.
/// A real index is built over a real dataset and a real retrieval is ranked
/// out of it, so a re-export that compiles while exposing nothing usable
/// fails here.
///
/// Anti-vacuity: an index that never ranks, or ranks the wrong document,
/// flips the `document_count` / exact-row assertions.
#[test]
fn text_full_text_search_ranks_the_holding_document() {
    use purrdf::text;

    let mut builder = RdfDatasetBuilder::new();
    let note = builder.intern_iri("https://example.org/note").expect("intern");
    let subject = builder.intern_iri("https://example.org/a").expect("intern");
    let literal = builder.intern_literal(RdfLiteral::simple("the quick brown fox")).expect("intern");
    builder.push_quad(subject, note, literal, None);
    let dataset = builder.freeze().expect("the facade fixture validates");

    let config = text::TextIndexConfig::new(
        vec![TermValue::iri("https://example.org/note")],
        text::GraphSelector::Any,
        text::Analyzer::empty_lexicon(),
    )
    .expect("one IRI predicate is a well-formed configuration");
    let index =
        text::TextIndex::from_dataset(&*dataset, &config).expect("the facade index builds");
    assert_eq!(index.document_count(), 1);

    let needle = vec!["quick".to_owned()];
    let ranked = text::select(
        &index,
        &needle,
        &text::PartitionFilter::unconstrained(),
        None,
        None,
    )
    .expect("the facade ranks");
    assert_eq!(
        ranked
            .iter()
            .map(|row| (row.document, row.partition_rank))
            .collect::<Vec<_>>(),
        vec![(0, 1)],
        "the one document holding the needle is its partition's rank one"
    );
}

// ── 10. validate: SARIF 2.1.0 boundary ─────────────────────────────────────

/// `validate_to_sarif_string` renders a real SHACL report as a SARIF 2.1.0
/// log, deterministically: two runs over the same input are byte-identical.
/// The assertion is a plain string CONTAINS over the emitted JSON — the
/// `purrdf_lex` lexer surface is a newer pin and is deliberately not used.
///
/// Anti-vacuity: a version bump, a timestamp leaking into the log, or an
/// unstable serialization flips the version match or the byte equality.
#[test]
fn validate_sarif_version_pin_and_byte_determinism() {
    use purrdf::validate;

    let data = r#"<https://example.org/alice> <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <https://example.org/Person> .
<https://example.org/bob> <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <https://example.org/Person> .
<https://example.org/alice> <https://example.org/name> "Alice" .
"#;
    let shapes_ttl = r#"
@prefix sh: <http://www.w3.org/ns/shacl#> .
<https://example.org/PersonShape> a sh:NodeShape ;
    sh:targetClass <https://example.org/Person> ;
    sh:property [
        sh:path <https://example.org/name> ;
        sh:minCount 1 ;
    ] .
"#;
    assert_eq!(validate::SARIF_VERSION, "2.1.0");
    let options = validate::SarifOptions::default();
    let first = validate::validate_to_sarif_string(shapes_ttl, None, data, &options, &[])
        .expect("the SARIF boundary renders");
    let second = validate::validate_to_sarif_string(shapes_ttl, None, data, &options, &[])
        .expect("the SARIF boundary renders again");
    assert_eq!(first, second, "byte-deterministic across runs");
    assert!(first.contains("2.1.0"), "the SARIF version is in the log");
    assert!(
        first.contains("MinCountConstraintComponent"),
        "the violation's component IRI is carried into the log"
    );
}

// ── 11. markdown: structural slicer ────────────────────────────────────────

/// Verbatim from the pinned umbrella (`purrdf-2.0.2/src/lib.rs:300-316`,
/// `facade_exposes_markdown_slicer`), routed through `graphlaw::purrdf`. The
/// returned model is asked a containment question — which unit holds a given
/// byte — rather than merely named.
///
/// Anti-vacuity: a slicer that returned no claims, or a model that cannot
/// locate the verse's first byte, flips either assertion.
#[test]
fn markdown_slicer_slices_and_locates() {
    use purrdf::markdown;

    let vocabulary =
        markdown::Vocabulary::standard().expect("the designated namespace validates");
    let profile = markdown::Profile::new("facade-fixture", 1, vocabulary);
    let doc = markdown::SourceDocument {
        id: "https://example.org/guide",
        bytes: b"# Title\n\n1. A verse the facade can reach.\n",
    };
    let claims = markdown::slice_markdown(&doc, &profile).expect("the facade slices");
    assert!(!claims.is_empty(), "a sliced document states claims");
    let model = markdown::analyze(&doc, &profile).expect("the facade analyzes");
    let unit = model
        .unit_at(u64::from(b"# Title\n\n".len() as u32))
        .expect("the verse holds its first byte");
    assert_eq!(unit.verse(), Some(1));
}

// ── 12. datalog: the types the entailment surface carries ─────────────────

/// The datalog module is re-exported because entail's public types CARRY it:
/// a `ReasoningReport` hands out a `datalog::cache::ContractHash` and a
/// `datalog::seminaive::BudgetReport`, and the evaluator's refusals are
/// `datalog::seminaive::EvalError`s. A consumer that stores or matches on
/// them must be able to WRITE those type names through the facade.
///
/// Anti-vacuity: the type annotations name the facade paths — deleting the
/// `datalog` module re-export breaks this leg's compile; a nondeterministic
/// budget or calculus hash flips the double-run equality.
#[test]
fn datalog_type_surface_contract_budget_and_eval_error() {
    use purrdf::datalog::cache::ContractHash;
    use purrdf::datalog::seminaive::{BudgetReport, BudgetResource, EvalError};
    use purrdf::entail::{Materialization, Regime};

    let mut builder = RdfDatasetBuilder::new();
    let (cat, animal) = (
        builder.intern_iri("https://example.org/Cat").expect("intern"),
        builder.intern_iri("https://example.org/Animal").expect("intern"),
    );
    let sub = builder.intern_iri("http://www.w3.org/2000/01/rdf-schema#subClassOf").expect("intern");
    builder.push_quad(cat, sub, animal, None);
    let dataset = builder.freeze().expect("fixture validates");

    let (_, report) =
        purrdf::entail::materialize(&dataset, Materialization::Rdfs).expect("rdfs");
    // The calculus identity, written through the datalog module's own type.
    let hash: ContractHash = report.contract_hash();
    assert_eq!(hash.to_hex().len(), 64, "a BLAKE3 digest, hex-rendered");
    // The budget, written through the datalog module's own type. Two runs over
    // one input report the same numbers — the budget is a function of the data.
    let budget: BudgetReport = report.budget();
    let (_, report_again) =
        purrdf::entail::materialize(&dataset, Materialization::Rdfs).expect("rdfs again");
    assert_eq!(budget, report_again.budget(), "deterministic consumption");
    assert_eq!(report.regime(), Regime::Rdfs);

    // The refusal type is writable and matchable through the facade: the
    // `#[non_exhaustive]` enum demands the catch-all arm.
    let refused: EvalError<BudgetReport> = EvalError::BudgetExhausted {
        resource: BudgetResource::JoinSteps,
        report: BudgetReport::new(11, 7, 5),
    };
    match refused {
        EvalError::BudgetExhausted { resource, report } => {
            assert_eq!(resource, BudgetResource::JoinSteps);
            assert_eq!(report.join_steps(), 11);
            assert_eq!(report.stored_facts(), 7);
            assert_eq!(report.term_arena_bytes(), 5);
        }
        other => panic!("the constructed refusal must round-trip, got {other:?}"),
    }
}

// ── 13. viz: cross-graph projection counts ─────────────────────────────────

/// A facts quad, a claims reifier, and a provenance annotation project to
/// per-graph (statements, assertions, relations) counts of (1,1,0) and
/// (1,0,1) under graph filters — visual reifier roles stay independent of the
/// carrier's graph.
///
/// Anti-vacuity: a projection that reassigned the reifier's graph, dropped
/// cross-graph occurrences, or miscounted rows flips the exact count triples.
#[test]
fn viz_cross_graph_projection_counts() {
    use purrdf::viz::{
        self, VizGraphId, VizGraphPolicy, VizGraphInput, VizInputQuad, VizInputReifier,
        VizInputStatement, VizSpec, VizTermValue, VizValueRef,
    };

    const EX: &str = "https://example.org/";
    const KNOWS: &str = "https://example.org/knows";

    let iri = |local: &str| TermValue::iri(format!("{EX}{local}"));

    let nquads = r#"<https://example.org/alice> <https://example.org/knows> <https://example.org/bob> <https://example.org/facts> .
<https://example.org/claim> <http://www.w3.org/1999/02/22-rdf-syntax-ns#reifies> <<( <https://example.org/alice> <https://example.org/knows> <https://example.org/bob> )>> <https://example.org/claims> .
<https://example.org/claim> <https://example.org/confidence> "0.8"^^<http://www.w3.org/2001/XMLSchema#decimal> <https://example.org/provenance> .
"#;
    let dataset = parse_dataset(nquads.as_bytes(), "application/n-quads", None)
        .expect("the RDF 1.2 fixture parses");
    assert_eq!(dataset.quad_count(), 2, "the reifies row is a reifier, not a quad");
    assert_eq!(dataset.reifiers().count(), 1);

    let graph_iri = |model: &viz::VizProjection, id: &VizGraphId| -> String {
        let graph = model.graphs.iter().find(|g| &g.id == id).expect("graph");
        let Some(VizValueRef::Term { id }) = &graph.term else {
            panic!("fixture graph must be named");
        };
        let term = model.terms.iter().find(|t| &t.id == id).expect("term");
        let VizTermValue::Iri { value } = &term.value else {
            panic!("fixture graph must be an IRI");
        };
        value.clone()
    };

    for (graph, statements, assertions, relations) in [
        ("facts", 1, 1, 0),
        ("claims", 1, 0, 1),
        ("provenance", 0, 0, 1),
    ] {
        let spec = VizSpec {
            graph_policy: VizGraphPolicy::Include(vec![format!("{EX}{graph}")]),
            ..VizSpec::default()
        };
        let model = viz::project_dataset(&dataset, &spec).expect("project");
        assert_eq!(model.statements.len(), statements, "{graph}: statements");
        assert_eq!(model.assertions.len(), assertions, "{graph}: assertions");
        assert_eq!(model.relations.len(), relations, "{graph}: relations");
        assert_eq!(model.graphs.len(), 1, "{graph}: one graph context");
        assert_eq!(
            graph_iri(&model, &model.graphs[0].id),
            format!("{EX}{graph}"),
            "{graph}: the retained context is the selected one"
        );
    }

    // The explicit-input surface answers identically to the parsed dataset.
    let input = VizGraphInput {
        quads: vec![
            VizInputQuad {
                subject: iri("alice"),
                predicate: KNOWS.to_owned(),
                object: iri("bob"),
                graph_name: Some(iri("facts")),
            },
            VizInputQuad {
                subject: iri("claim"),
                predicate: "https://example.org/confidence".to_owned(),
                object: TermValue::Literal {
                    lexical_form: "0.8".to_owned(),
                    datatype: "http://www.w3.org/2001/XMLSchema#decimal".to_owned(),
                    language: None,
                    direction: None,
                },
                graph_name: Some(iri("provenance")),
            },
        ],
        reifiers: vec![VizInputReifier {
            reifier: iri("claim"),
            statement: VizInputStatement {
                subject: iri("alice"),
                predicate: KNOWS.to_owned(),
                object: iri("bob"),
            },
            graph_name: Some(iri("claims")),
        }],
        annotations: Vec::new(),
    };
    let spec = VizSpec::default();
    let model = viz::project_dataset(&dataset, &spec).expect("project");
    assert_eq!(
        model,
        viz::project_graph_input(&input, &spec).expect("explicit input"),
        "parsed and explicit inputs share one visual model"
    );
}

// ── 14. foundations: iri + xsd ─────────────────────────────────────────────

/// The zero-dependency leaves answer through the facade: IRI parse/CURIE
/// expansion/contraction, and the XSD temporal value space's epoch, which
/// renders its canonical lexical form.
///
/// Anti-vacuity: a broken expander, a lost contraction, or a wrong epoch
/// lexical form flips the exact-string assertions.
#[test]
fn foundations_iri_curies_and_xsd_epoch() {
    use purrdf::iri;
    use purrdf::xsd::temporal;

    let parsed = iri::parse("https://example.org/cat").expect("a valid IRI");
    assert_eq!(parsed.as_str(), "https://example.org/cat");

    let mut prefixes = iri::PrefixMap::new();
    prefixes.insert("ex", "https://example.org/ns#");
    assert_eq!(
        iri::expand_curie("ex:Thing", &prefixes),
        Some("https://example.org/ns#Thing".to_owned()),
    );
    assert_eq!(
        iri::contract("https://example.org/ns#Thing", &prefixes),
        Some("ex:Thing".to_owned()),
    );

    let epoch = temporal::datetime_epoch();
    assert_eq!(
        epoch.canonical_lexical(),
        "1970-01-01T00:00:00Z",
        "the Unix epoch in XSD canonical form"
    );
}

// ── 15. events: the streaming sink contract ────────────────────────────────

/// A source MUST stop at the first `ControlFlow::Break` and MUST call
/// `finish` exactly once at the end of a NON-cancelled drive. A sink that
/// breaks at the second quad is therefore answered with two quads and no
/// finish; the same sink without the break sees all three and the finish.
///
/// Anti-vacuity: a source that kept feeding after a Break, or that called
/// `finish` on a cancelled drive, flips the `finished`/count readbacks.
#[test]
fn events_sink_breaks_at_second_quad_and_finish_stays_uncalled() {
    use purrdf::events::{
        EventError, EventQuad, EventTerm, EventTermId, RdfEventSink, RdfEventSource, ScopeId,
    };

    /// Breaks the drive at the second quad.
    struct BreakAtTwo {
        quads: usize,
        finished: bool,
        break_at: Option<usize>,
    }

    impl BreakAtTwo {
        fn breaking() -> Self {
            Self {
                quads: 0,
                finished: false,
                break_at: Some(2),
            }
        }
        fn draining() -> Self {
            Self {
                quads: 0,
                finished: false,
                break_at: None,
            }
        }
    }

    impl RdfEventSink for BreakAtTwo {
        fn term(
            &mut self,
            _id: EventTermId,
            _term: EventTerm<'_>,
        ) -> Result<ControlFlow<()>, EventError> {
            Ok(ControlFlow::Continue(()))
        }

        fn quad(&mut self, _q: EventQuad) -> Result<ControlFlow<()>, EventError> {
            self.quads += 1;
            Ok(if self.break_at == Some(self.quads) {
                ControlFlow::Break(())
            } else {
                ControlFlow::Continue(())
            })
        }

        fn reifier(
            &mut self,
            _reifier: EventTermId,
            _triple: graphlaw::purrdf::events::EventTriple,
        ) -> Result<ControlFlow<()>, EventError> {
            Ok(ControlFlow::Continue(()))
        }

        fn annotation(
            &mut self,
            _reifier: EventTermId,
            _p: EventTermId,
            _o: EventTermId,
        ) -> Result<ControlFlow<()>, EventError> {
            Ok(ControlFlow::Continue(()))
        }

        fn open_scope(&mut self) -> Result<ScopeId, EventError> {
            Ok(ScopeId(1))
        }

        fn close_scope(&mut self, _scope: ScopeId) -> Result<ControlFlow<()>, EventError> {
            Ok(ControlFlow::Continue(()))
        }

        fn finish(&mut self) -> Result<(), EventError> {
            self.finished = true;
            Ok(())
        }
    }

    /// A three-quad fixture source driven straight through the trait.
    struct ThreeQuads;

    impl RdfEventSource for ThreeQuads {
        fn drive<S: RdfEventSink + ?Sized>(&self, sink: &mut S) -> Result<(), EventError> {
            for _ in 0..3 {
                if sink.quad(EventQuad {
                    s: EventTermId(0),
                    p: EventTermId(1),
                    o: EventTermId(2),
                    g: None,
                })? == ControlFlow::Break(())
                {
                    return Ok(());
                }
            }
            sink.finish()
        }
    }

    let mut breaker = BreakAtTwo::breaking();
    ThreeQuads.drive(&mut breaker).expect("the drive answers");
    assert_eq!(breaker.quads, 2, "the drive stops at the break");
    assert!(!breaker.finished, "a cancelled drive never finishes");

    let mut drainer = BreakAtTwo::draining();
    ThreeQuads.drive(&mut drainer).expect("the drive answers");
    assert_eq!(drainer.quads, 3);
    assert!(drainer.finished, "a non-cancelled drive finishes exactly once");
}

// ── 16. cdt: FOLD/UNFOLD end to end ────────────────────────────────────────

/// The SEP-0009 operators answer end to end from real query text through the
/// native engine: `UNFOLD` over `"[1,1,2]"^^cdt:List` yields the exact
/// (element, 1-based index) rows, and FOLD re-folds them to the original
/// lexical form — the two nouns are inverses.
///
/// Anti-vacuity: an operator that reordered rows, 0-based its index, dropped
/// duplicates, or failed the inverse fold flips the exact row grid or the ASK.
#[test]
fn cdt_fold_unfold_exact_rows_and_inverse() {
    use purrdf::sparql::QueryOptions;

    const PFX: &str = "PREFIX cdt: <http://w3id.org/awslabs/neptune/SPARQL-CDTs/> \
                       PREFIX xsd: <http://www.w3.org/2001/XMLSchema#> ";
    let empty = RdfDatasetBuilder::new().freeze().expect("empty dataset");
    let evaluate = |query: &str| ask(&empty, &format!("{PFX}{query}")).expect("cdt answers");
    let evaluate_options = |query: &str| {
        use graphlaw::purrdf::sparql::NativeSparqlEngine;
        NativeSparqlEngine::new()
            .query_with_options_view(
                &empty,
                SparqlRequest {
                    query: &format!("{PFX}{query}"),
                    base_iri: None,
                    substitutions: &[],
                },
                QueryOptions::EMPTY,
            )
            .expect("cdt answers")
    };

    // The four readings: list element plus 1-based xsd:integer index, in
    // list order, duplicates preserved.
    let unfolded = rows(evaluate(
        "SELECT ?e ?i WHERE { BIND(\"[1,1,2]\"^^cdt:List AS ?l) UNFOLD(?l AS ?e, ?i) }",
    ));
    assert_eq!(
        unfolded,
        vec![
            vec!["1".to_owned(), "1".to_owned()],
            vec!["1".to_owned(), "2".to_owned()],
            vec!["2".to_owned(), "3".to_owned()],
        ],
        "list order, duplicates preserved, index 1-based"
    );

    // The same query through the options-bearing public entry point answers
    // identically.
    let via_options = rows(evaluate_options(
        "SELECT ?e WHERE { BIND(\"[1,1,2]\"^^cdt:List AS ?l) UNFOLD(?l AS ?e) }",
    ));
    assert_eq!(
        via_options,
        vec![vec!["1".to_owned()], vec!["1".to_owned()], vec!["2".to_owned()]],
    );

    // FOLD inverts UNFOLD over a list with no nulls.
    let SparqlResult::Boolean(yes) = evaluate(
        "ASK { { SELECT (FOLD(?e ORDER BY ?i) AS ?back) WHERE { \
         BIND(\"[3,1,2]\"^^cdt:List AS ?l) UNFOLD(?l AS ?e, ?i) } } \
         FILTER(?back = \"[3,1,2]\"^^cdt:List) }",
    ) else {
        panic!("expected an ASK answer");
    };
    assert!(yes, "fold(unfold(l)) = l");
}

// TODO(P4, purrdf 3.0.1 pin): the retrieval leg (PURREMB nearest-neighbour
// retrieval over the `knn` property-function seam) and the `xsd::temporal`
// rfc3339 arm are gated on the coordinator's purrdf 3.0.1 bump commit; both
// surfaces are absent from the 2.0.2 pin this file pins. They land as legs 17
// and 18 with that bump — see the P4 plan's 18-row spec table.

// ── 17. the facade net ─────────────────────────────────────────────────────

/// Verbatim from the pinned umbrella (`purrdf-2.0.2/src/lib.rs:487-538`,
/// `facade_exposes_the_completed_umbrella`), routed through `graphlaw::purrdf`:
/// columnar, gts, sparql, iri, events, entail, validate, the pack cache, and
/// the governed/lazy query execution surfaces all answer from the one facade.
///
/// Anti-vacuity: this is the structural net itself — dropping ANY of those
/// module re-exports (or the `pub use purrdf;` this file guards) breaks the
/// compile or flips an assertion inside it.
#[test]
fn facade_exposes_the_completed_umbrella() {
    use purrdf::{PackBuilder, PagedQueryLimits, PagedQueryEvidence, RdfDatasetBuilder};
    use purrdf::{columnar, entail, events, gts, iri, sparql, validate};

    assert_eq!(columnar::Table::ALL.len(), 5);

    // gts: the container engine (its `model`) and the rdf-level adapter
    // (`read_graph`) are both reachable under the one `gts` module.
    assert_ne!(format!("{:?}", gts::model::TermKind::Iri), "");
    let adapter: fn(&[u8], bool) -> _ = gts::read_graph;
    assert!(
        adapter(&[], false).is_err(),
        "empty input is not a GTS graph"
    );

    // sparql: parser (algebra) + engine (eval) + results.
    assert_ne!(format!("{:?}", sparql::SparqlResultsFormat::Json), "");

    // foundations.
    assert!(iri::parse("https://example.org/x").is_ok());
    assert_ne!(format!("{:?}", events::TextDirection::Ltr), "");

    // entail: the entailment regimes are reachable through the facade.
    assert_eq!(
        entail::Regime::from_iri("http://www.w3.org/ns/entailment/RDFS"),
        Some(entail::Regime::Rdfs)
    );

    // validate: the SARIF reporting boundary is reachable through the facade.
    assert_eq!(validate::SARIF_VERSION, "2.1.0");

    // The supported umbrella also exposes the succinct dataset-pack cache
    // boundary; downstreams never need to depend on purrdf-core directly.
    let empty = RdfDatasetBuilder::new().freeze().expect("empty dataset");
    let pack = PackBuilder::build_bytes(&empty).expect("pack empty dataset");
    let restored = purrdf::restore_pack(&pack).expect("restore empty dataset");
    assert_eq!(restored.quad_count(), 0);

    // Fallible lazy query execution is reachable from the umbrella alone.
    let limits = PagedQueryLimits::new(1, 1024);
    assert_eq!(limits.max_pages, 1);
    let _: Option<sparql::CompleteSparqlResult<PagedQueryEvidence>> = None;

    // So is governed query execution: the ceilings, the outcome, and the certified
    // partial answers all resolve unambiguously under the one `sparql` module, so a
    // consumer that sets a budget never has to name the evaluator crate.
    let governors = sparql::QueryGovernors::UNBOUNDED.with_max_answers(1);
    assert!(governors.is_engaged());
    let _: Option<sparql::GovernedOutcome> = None;
    let _: Option<sparql::PartialAnswers> = None;
    let _: Option<sparql::PartialSparqlResult> = None;
    let _: Option<sparql::BudgetExhausted> = None;
    let _: Option<sparql::GovernedEvidence<PagedQueryEvidence>> = None;
}

// ── 18. n3: Notation3 reasoner via PurRDF facade ───────────────────────────

/// `purrdf::n3` exposes the complete Notation3 parsing, reasoning, and RDF 1.2
/// compatibility API powered by Eyeron, reachable directly through the `purrdf`
/// umbrella facade.
///
/// Anti-vacuity: an N3 reasoner that drops rules or fails to derive `:Socrates a :Mortal`
/// flips the string assertion.
#[test]
fn purrdf_exposes_n3_reasoner() {
    use purrdf::n3;

    let n3_code = r#"
        @prefix : <http://example.org/> .
        @prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .
        :Socrates a :Human .
        :Human rdfs:subClassOf :Mortal .
        { ?s a ?a . ?a rdfs:subClassOf ?b . } => { ?s a ?b . } .
    "#;
    let derived = n3::reason(n3_code).expect("purrdf::n3 reasoning evaluates");
    assert!(
        derived.contains(":Socrates a :Mortal"),
        "purrdf::n3 derives Socrates mortality: {derived}"
    );

    // Verify AST parsing and reason_document with options
    let doc = n3::parse_n3(n3_code, None).expect("parse n3 document");
    let options = n3::ReasonerOptions::default();
    let result = n3::reason_document(&doc, &options);
    assert!(result.is_complete(), "N3 reasoning completed without limits");
    assert!(!result.derived.is_empty(), "Derived triples emitted");
}
