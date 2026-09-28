use praxis_graphlaw::TripleStore;

#[test]
fn standalone_public_api_parses_queries_and_materializes() {
    let mut store = TripleStore::from(
        "@prefix ex: <http://example.org/> .\n\
         ex:alice ex:knows ex:bob .\n",
    );

    assert_eq!(store.len(), 1, "one asserted triple should be admitted");

    let rows = store
        .query(
            "SELECT ?o WHERE { \
             <http://example.org/alice> <http://example.org/knows> ?o . \
             }",
        )
        .expect("standalone SPARQL query should execute");

    assert_eq!(rows.len(), 1, "the asserted object should be queryable");

    let inferred = store
        .materialize()
        .expect("empty-rule materialization should succeed");
    assert!(inferred.is_empty(), "no rules means no derived triples");
    assert!(store.check_denials().is_empty(), "no denial rules were loaded");
    assert_eq!(store.len(), 1, "materialization must preserve asserted state");
}

#[test]
fn standalone_public_api_is_deterministic_for_same_input() {
    let input = "@prefix ex: <http://example.org/> .\n\
                 ex:b ex:p ex:two .\n\
                 ex:a ex:p ex:one .\n";
    let first = TripleStore::from(input).content_to_string();
    let second = TripleStore::from(input).content_to_string();

    assert_eq!(first, second, "canonical public serialization must replay");
}
