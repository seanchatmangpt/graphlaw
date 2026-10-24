//! SQLRDF-25 Row 4 court — composite literals (struct / list) and accessors
//! (`sql:structLiteral` / `sql:listLiteral` / `sql:indexAccess` /
//! `sql:sliceAccess`) with the depth cap of 64. Ledger:
//! docs/jira/v26.10.22/SQLRDF-25-remaining-gaps.md (Row 4). Chicago: real
//! frozen-duckdb, real table readback, no mocks, no sleeps.

use graphlaw::sql_ast::compile_turtle;
use graphlaw::tier::{Admitted, admit_bytes_tiered};

use graphlaw::duckdb_eval::GraphLawDuckDB;

// PENDING-LANE: the RDF surface for composite literals is the target
// vocabulary from the ledger (sql:structLiteral / sql:listLiteral nodes with
// an rdf:List of members); the exact member predicates follow the composite
// lane's extractor. If the member predicate names differ, this court is the
// conformance witness that forces the reconciliation.
const LIST_DOC_HEAD: &str = r#"
@prefix sql: <http://graphlaw.org/sql#> .
@prefix xsd: <http://www.w3.org/2001/XMLSchema#> .
@prefix ex:  <http://example.org/queries#> .
@prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .

ex:Q a sql:SelectStatement ;
    sql:statementIndex 1 ;
    sql:fromTable [ sql:tableName "sensor_batches" ] ;
    sql:selectList (
        [ sql:aliasName "second_reading" ; sql:expression [
            a sql:IndexAccess ;
            sql:indexBase [ a sql:ListLiteral ;
                sql:listItems ( [ sql:literalValue "10"^^xsd:integer ]
                                [ sql:literalValue "20"^^xsd:integer ]
                                [ sql:literalValue "30"^^xsd:integer ] ) ] ;
            sql:indexIndex [ sql:literalValue "2"^^xsd:integer ] ] ]
    ) .
"#;

const STRUCT_DOC: &str = r#"
@prefix sql: <http://graphlaw.org/sql#> .
@prefix xsd: <http://www.w3.org/2001/XMLSchema#> .
@prefix ex:  <http://example.org/queries#> .
@prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .

ex:Q a sql:SelectStatement ;
    sql:statementIndex 1 ;
    sql:fromTable [ sql:tableName "sensor_batches" ] ;
    sql:selectList (
        [ sql:aliasName "packed" ; sql:expression [
            a sql:StructLiteral ;
            sql:structFields ( [ sql:fieldKey "kind" ;
                                 sql:fieldValue [ sql:literalValue "batch" ] ]
                               [ sql:fieldKey "n" ;
                                 sql:fieldValue [ sql:literalValue "3"^^xsd:integer ] ] ) ] ]
    ) .
"#;

fn db() -> GraphLawDuckDB {
    let db = GraphLawDuckDB::open_in_memory().unwrap();
    db.raw_connection()
        .execute_batch(
            "CREATE TABLE sensor_batches (batch_id VARCHAR, readings INTEGER[]);
             INSERT INTO sensor_batches VALUES ('b1', [10, 20, 30]);",
        )
        .unwrap();
    db
}

#[test]
fn list_literal_index_access_reads_back_the_element() {
    let admitted = admit_bytes_tiered(LIST_DOC_HEAD.as_bytes(), None, None, None).unwrap();
    let compiled = match admitted {
        Admitted::Query(q) => q,
        other => panic!("expected a compiled query, got {other:?}"),
    };

    let db = db();
    let receipt = db
        .eval_ast_query(
            "http://example.org/queries#Q",
            &compiled,
            "http://example.org/data",
            "http://example.org/vocab",
        )
        .unwrap();
    assert_eq!(receipt.rows_emitted, 1);
    // Real readback over the receipt's result triples (the real collaborator
    // state), inlined at the assert site.
    let mut second_reading: Vec<String> = receipt
        .result_triples
        .iter()
        .filter(|t| t.predicate.ends_with("/second_reading"))
        .filter_map(|t| match &t.object {
            purrdf::TermValue::Literal { lexical_form, .. } => Some(lexical_form.clone()),
            _ => None,
        })
        .collect();
    second_reading.sort();
    // Anti-vacuity: corrupting the member index (1-based 2 -> 3) flips this
    // to "30".
    assert_eq!(second_reading, vec!["20"]);
}

#[test]
fn struct_literal_reads_back_its_fields() {
    let admitted = admit_bytes_tiered(STRUCT_DOC.as_bytes(), None, None, None).unwrap();
    let compiled = match admitted {
        Admitted::Query(q) => q,
        other => panic!("expected a compiled query, got {other:?}"),
    };

    let db = db();
    let receipt = db
        .eval_ast_query(
            "http://example.org/queries#Q",
            &compiled,
            "http://example.org/data",
            "http://example.org/vocab",
        )
        .unwrap();
    assert_eq!(receipt.rows_emitted, 1);
    let mut packed: Vec<String> = receipt
        .result_triples
        .iter()
        .filter(|t| t.predicate.ends_with("/packed"))
        .filter_map(|t| match &t.object {
            purrdf::TermValue::Literal { lexical_form, .. } => Some(lexical_form.clone()),
            _ => None,
        })
        .collect();
    packed.sort();
    assert_eq!(packed.len(), 1, "one struct value read back: {packed:?}");
    assert!(packed[0].contains("batch"), "{:?}", packed[0]);
}

#[test]
fn depth_overflow_accessor_refuses_refusal_01_composite_depth() {
    // Anti-vacuity twin via the SHACL mirror route: purrdf's parse_dataset
    // refuses a literally 65-deep blank-node document (parse-depth limit)
    // before the compiler's COMPOSITE_DEPTH arm could fire, so the court uses
    // a shallow document whose struct literal carries the declared-depth
    // property `sql:compositeDepth "65"^^xsd:integer`. The shape
    // `sql:ListLiteralShape` (ontologies/sql-ast.ttl, sh:maxInclusive 64,
    // arm 48 COMPOSITE_DEPTH) refuses it typed at admission.
    let doc = LIST_DOC_HEAD.replace(
        "a sql:ListLiteral ;",
        "a sql:ListLiteral ; sql:compositeDepth \"65\"^^xsd:integer ;",
    );
    let err = compile_turtle(&doc).unwrap_err().to_string();
    assert!(err.contains("REFUSAL-01"), "{err}");
    assert!(err.contains("depth"), "{err}");
}

#[test]
fn malformed_struct_field_key_refuses_refusal_01() {
    // COMPOSITE_MEMBER arm: a field key outside ^[a-zA-Z_][a-zA-Z0-9_]*$
    // refuses typed before any engine call.
    let doc = STRUCT_DOC.replace("sql:fieldKey \"kind\"", "sql:fieldKey \"kind'; drop table x\"");
    let err = compile_turtle(&doc).unwrap_err().to_string();
    assert!(err.contains("REFUSAL-01"), "{err}");
}
