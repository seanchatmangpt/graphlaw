use graphlaw::{BACKEND_AUTHORITIES, LEGACY_FALLBACK_ENABLED};

const _: () = assert!(!LEGACY_FALLBACK_ENABLED);

#[test]
fn authority_manifest_names_external_backends() {
    assert!(BACKEND_AUTHORITIES.iter().any(|a| a.authority == "purrdf"));
    assert!(BACKEND_AUTHORITIES.iter().any(|a| a.authority == "eyeron"));
}

#[test]
fn purrdf_parses_rdf_without_graphlaw_parser_code() {
    let dataset = graphlaw::rdf::parse_dataset(
        b"<https://example.org/s> <https://example.org/p> <https://example.org/o> .\n",
        "application/n-triples",
        None,
    )
    .expect("valid RDF");
    assert_eq!(dataset.quad_count(), 1);
}

#[test]
fn eyeron_executes_n3_without_graphlaw_reasoner_code() {
    let program = r#"
        @prefix : <https://example.org/> .
        :s :kind :Human .
        { ?x :kind :Human . } => { ?x :kind :Mortal . } .
    "#;
    let output = graphlaw::n3::reason(program).expect("valid N3");
    assert!(output.contains("Mortal"), "{output}");
}

#[test]
fn purrdf_executes_sparql_without_graphlaw_query_engine() {
    use graphlaw::rdf::{RdfDatasetBuilder, RdfLiteral, SparqlEngine, SparqlRequest, SparqlResult};
    use graphlaw::sparql::NativeSparqlEngine;

    let mut builder = RdfDatasetBuilder::new();
    let cat = builder.intern_iri("https://example.org/cat");
    let says = builder.intern_iri("https://example.org/says");
    let meow = builder.intern_literal(RdfLiteral::simple("meow"));
    builder.push_quad(cat, says, meow, None);
    let dataset = builder.freeze().expect("valid dataset");

    let engine = NativeSparqlEngine::new();
    let result = engine
        .query(
            &dataset,
            SparqlRequest {
                query: "SELECT ?what WHERE { <https://example.org/cat> <https://example.org/says> ?what }",
                base_iri: None,
                substitutions: &[],
            },
        )
        .expect("SPARQL evaluates");

    match result {
        SparqlResult::Solutions { rows, .. } => assert_eq!(rows.len(), 1),
        other => panic!("expected solution rows, got {other:?}"),
    }
}

#[test]
fn purrdf_executes_datalog_without_graphlaw_fixpoint_code() {
    use graphlaw::datalog::{
        clause::{ClauseAtom, ClauseTerm, DlClause},
        seminaive::{compile, evaluate},
        store::RelationStore,
    };

    let rule = DlClause::datalog(
        ClauseAtom::positive(
            ClauseTerm::var("x"),
            "https://example.org/ancestor",
            ClauseTerm::var("y"),
        ),
        vec![ClauseAtom::positive(
            ClauseTerm::var("x"),
            "https://example.org/parent",
            ClauseTerm::var("y"),
        )],
    );
    let executable = compile(vec![rule]).expect("safe stratified Datalog");

    let mut facts = RelationStore::new();
    facts.insert(
        "<https://example.org/alice>",
        "<https://example.org/parent>",
        "<https://example.org/bob>",
        RelationStore::DEFAULT_GRAPH,
    );
    let evaluation = evaluate(&executable, facts).expect("least fixpoint");
    assert!(evaluation.facts().contains(
        "<https://example.org/alice>",
        "<https://example.org/ancestor>",
        "<https://example.org/bob>",
        RelationStore::DEFAULT_GRAPH,
    ));
}

#[test]
fn purrdf_executes_shacl_without_graphlaw_validator() {
    let data = r#"
<https://example.org/alice> <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <https://example.org/Person> .
<https://example.org/alice> <https://example.org/age> "not-an-integer" .
"#;
    let shapes = r#"
@prefix sh: <http://www.w3.org/ns/shacl#> .
@prefix ex: <https://example.org/> .
@prefix xsd: <http://www.w3.org/2001/XMLSchema#> .
ex:PersonShape a sh:NodeShape ;
    sh:targetClass ex:Person ;
    sh:property [ sh:path ex:age ; sh:datatype xsd:integer ] .
"#;
    let report =
        graphlaw::shacl::engine::validate_graphs(data, shapes, None).expect("SHACL evaluates");
    assert!(!report.conforms);
    assert_eq!(report.results.len(), 1, "{:?}", report.results);
    let r = &report.results[0];
    assert_eq!(r.focus_node.to_string(), "<https://example.org/alice>");
    assert!(
        r.source_constraint_component
            .to_string()
            .contains("DatatypeConstraintComponent"),
        "component: {}",
        r.source_constraint_component
    );
}

#[test]
fn purrdf_parses_shex_without_graphlaw_shexc_parser() {
    let schema = graphlaw::shex::parse_shexc(
        "PREFIX ex: <https://example.org/>\nex:Cat { ex:says . }",
        None,
    )
    .expect("valid ShExC");
    graphlaw::shex::check_structure(&schema).expect("well-formed ShEx");
}

#[test]
fn purrdf_exposes_owl_rl_entailment_without_graphlaw_ruleset() {
    use graphlaw::entailment::Regime;
    assert_eq!(
        Regime::from_iri("http://www.w3.org/ns/entailment/RDFS"),
        Some(Regime::Rdfs)
    );
    assert_eq!(
        Regime::from_iri("http://www.w3.org/ns/entailment/OWL-RL"),
        Some(Regime::OwlRl)
    );
}
