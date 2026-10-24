//! SQLRDF-25 Row 1 court — star modifiers `* EXCLUDE (…)` / `* REPLACE (…)`.
//! Ledger: docs/jira/v26.10.22/SQLRDF-25-remaining-gaps.md (Row 1). The
//! EMPTY_PROJECTION law is unchanged: `selectList ()` still refuses REFUSAL-01.
//! Chicago: real frozen-duckdb connection, real table readback, no mocks.

use graphlaw::sql_ast::compile_turtle;
use graphlaw::tier::{Admitted, admit_bytes_tiered};

use graphlaw::duckdb_eval::GraphLawDuckDB;

const EXCLUDE_DOC: &str = r#"
@prefix sql: <http://graphlaw.org/sql#> .
@prefix xsd: <http://www.w3.org/2001/XMLSchema#> .
@prefix ex:  <http://example.org/queries#> .

ex:Q a sql:SelectStatement ;
    sql:statementIndex 1 ;
    sql:fromTable [ sql:tableName "telemetry_events" ] ;
    sql:selectList (
        [ a sql:Star ;
          sql:exclude ( [ sql:columnName "latency_ms" ] ) ]
    ) .
"#;

const REPLACE_DOC: &str = r#"
@prefix sql: <http://graphlaw.org/sql#> .
@prefix xsd: <http://www.w3.org/2001/XMLSchema#> .
@prefix ex:  <http://example.org/queries#> .

ex:Q a sql:SelectStatement ;
    sql:statementIndex 1 ;
    sql:fromTable [ sql:tableName "telemetry_events" ] ;
    sql:selectList (
        [ a sql:Star ;
          sql:replace (
            [ sql:expression [
                sql:operator "+"^^<http://www.w3.org/2001/XMLSchema#string> ;
                sql:leftOperand [ sql:columnName "latency_ms" ] ;
                sql:rightOperand [ sql:literalValue "100"^^xsd:integer ] ] ;
              sql:aliasName "latency_ms" ] )
        ]
    ) .
"#;

fn db() -> GraphLawDuckDB {
    let db = GraphLawDuckDB::open_in_memory().unwrap();
    db.raw_connection()
        .execute_batch(
            "CREATE TABLE telemetry_events (batch_id VARCHAR, latency_ms INTEGER);
             INSERT INTO telemetry_events VALUES ('alpha', 480), ('beta', 10), ('gamma', 900);",
        )
        .unwrap();
    db
}

fn column_values(receipt: &graphlaw::duckdb_eval::ExecutionReceipt, column: &str) -> Vec<String> {
    let suffix = format!("/{column}");
    let mut values: Vec<String> = receipt
        .result_triples
        .iter()
        .filter(|t| t.predicate.ends_with(&suffix))
        .filter_map(|t| match &t.object {
            purrdf::TermValue::Literal { lexical_form, .. } => Some(lexical_form.clone()),
            _ => None,
        })
        .collect();
    values.sort();
    values
}

fn has_column(receipt: &graphlaw::duckdb_eval::ExecutionReceipt, column: &str) -> bool {
    let suffix = format!("/{column}");
    receipt
        .result_triples
        .iter()
        .any(|t| t.predicate.ends_with(&suffix))
}

#[test]
fn star_exclude_reads_back_without_the_excluded_column() {
    let compiled = compile_turtle(EXCLUDE_DOC).unwrap();
    assert!(
        compiled.sql.to_ascii_uppercase().contains("EXCLUDE"),
        "{}",
        compiled.sql
    );
    let admitted = admit_bytes_tiered(EXCLUDE_DOC.as_bytes(), None, None, None).unwrap();
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
    assert_eq!(receipt.rows_emitted, 3, "star keeps the full rowcount");
    // Anti-vacuity: removing the EXCLUDE node (or the arm) re-admits
    // latency_ms and these two assertions flip.
    assert!(
        !has_column(&receipt, "latency_ms"),
        "excluded column must be absent from the readback"
    );
    assert_eq!(column_values(&receipt, "batch_id"), vec!["alpha", "beta", "gamma"]);
}

#[test]
fn star_replace_reads_back_the_replacement_values() {
    let compiled = compile_turtle(REPLACE_DOC).unwrap();
    assert!(
        compiled.sql.to_ascii_uppercase().contains("REPLACE"),
        "{}",
        compiled.sql
    );
    let admitted = admit_bytes_tiered(REPLACE_DOC.as_bytes(), None, None, None).unwrap();
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
    assert_eq!(receipt.rows_emitted, 3);
    // latency_ms replaced by latency_ms + 100: 580, 110, 1000. A dropped or
    // identity REPLACE mutation flips this to 10/480/900.
    assert_eq!(
        column_values(&receipt, "latency_ms"),
        vec!["1000", "110", "580"]
    );
    assert_eq!(column_values(&receipt, "batch_id"), vec!["alpha", "beta", "gamma"]);
}

#[test]
fn empty_exclude_refuses_refusal_01() {
    // PENDING-LANE: malformed-member twin — `sql:exclude ()` (rdf:nil, zero
    // EXCLUDE members) must refuse typed REFUSAL-01, never silently render a
    // bare `*`. If the star lane has not landed the empty-EXCLUDE arm, this
    // test is the court that forces it.
    let doc = EXCLUDE_DOC.replace(
        "sql:exclude ( [ sql:columnName \"latency_ms\" ] )",
        "sql:exclude ( )",
    );
    let err = compile_turtle(&doc).unwrap_err().to_string();
    assert!(err.contains("REFUSAL-01"), "{err}");
}

#[test]
fn empty_projection_still_refuses_for_star_surface() {
    // D1 law unchanged: an empty projection list is never a zero-column
    // SELECT, even on the star-modifier surface.
    let doc = EXCLUDE_DOC.replace(
        "sql:selectList (\n        [ a sql:Star ;\n          sql:exclude ( [ sql:columnName \"latency_ms\" ] ) ]\n    )",
        "sql:selectList ( )",
    );
    let err = compile_turtle(&doc).unwrap_err().to_string();
    assert!(err.contains("REFUSAL-01"), "{err}");
    assert!(
        err.to_lowercase().contains("empty projection list"),
        "{err}"
    );
}
