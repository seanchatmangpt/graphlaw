//! SQLRDF-25 Row 5 court — JSON operators `->` / `->>` (`sql:jsonExtract` /
//! `sql:jsonExtractText`). Ledger:
//! docs/jira/v26.10.22/SQLRDF-25-remaining-gaps.md (Row 5). Chicago: real
//! frozen-duckdb connection, real table readback, no mocks, no sleeps.

use graphlaw::sql_ast::compile_turtle;
use graphlaw::tier::{Admitted, admit_bytes_tiered};

const BASE: &str = r#"
@prefix sql: <http://graphlaw.org/sql#> .
@prefix xsd: <http://www.w3.org/2001/XMLSchema#> .
@prefix ex:  <http://example.org/queries#> .

ex:Q a sql:SelectStatement ;
    sql:statementIndex 1 ;
    sql:fromTable [ sql:tableName "json_events" ] ;
    sql:selectList (
        [ sql:columnName "id" ]
        [ sql:aliasName "extracted_json" ; sql:expression [
            sql:operator sql:JsonExtract ;
            sql:leftOperand [ sql:columnName "payload" ] ;
            sql:rightOperand [ sql:literalValue "$.k" ] ] ]
        [ sql:aliasName "extracted_text" ; sql:expression [
            sql:operator sql:JsonExtractText ;
            sql:leftOperand [ sql:columnName "payload" ] ;
            sql:rightOperand [ sql:literalValue "$.k" ] ] ]
    ) .
"#;

#[test]
fn json_operators_admit_and_read_back_distinct_values() {
    // Both operator IRIs admit through the section 4 operator path.
    let compiled = compile_turtle(BASE).unwrap();
    let sql_upper = compiled.sql.to_ascii_uppercase();
    assert!(
        sql_upper.contains("->") && sql_upper.contains("->>"),
        "both tokens must render: {}",
        compiled.sql
    );

    let admitted = admit_bytes_tiered(BASE.as_bytes(), None, None, None).unwrap();
    let compiled = match admitted {
        Admitted::Query(q) => q,
        other => panic!("expected a compiled query, got {other:?}"),
    };

    let db = GraphLawDuckDB::open_in_memory().unwrap();
    db.raw_connection()
        .execute_batch(
            "CREATE TABLE json_events (id INTEGER, payload JSON);
             INSERT INTO json_events VALUES
               (1, '{\"k\": \"v1\"}'),
               (2, '{\"k\": \"v2\"}');",
        )
        .unwrap();

    let receipt = db
        .eval_ast_query(
            "http://example.org/queries#Q",
            &compiled,
            "http://example.org/data",
            "http://example.org/vocab",
        )
        .unwrap();
    assert_eq!(receipt.rows_emitted, 2, "exact rowcount");

    // The two tokens must produce DIFFERENT observables: `->` yields the
    // JSON-quoted text, `->>` the bare text. Swapping the tokens flips both.
    let as_json = column_values(&receipt, "extracted_json");
    let as_text = column_values(&receipt, "extracted_text");
    assert_eq!(as_json, vec!["\"v1\"", "\"v2\""], "arrow form is quoted");
    assert_eq!(as_text, vec!["v1", "v2"], "double-arrow form is bare text");
}

#[test]
fn unknown_json_operator_token_refuses_refusal_01() {
    // Anti-vacuity twin: an operator token outside the enum refuses typed
    // before any engine call; the JSON arms are additions to a closed enum,
    // so a near-miss spelling cannot slip through.
    let doc = BASE.replace("sql:JsonExtractText", "\"JSON-ARROW\"^^<http://www.w3.org/2001/XMLSchema#string>");
    let err = compile_turtle(&doc).unwrap_err().to_string();
    assert!(err.contains("REFUSAL-01"), "{err}");
    assert!(err.contains("JSON-ARROW"), "{err}");
}

fn column_values(
    receipt: &graphlaw::duckdb_eval::ExecutionReceipt,
    column: &str,
) -> Vec<String> {
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

use graphlaw::duckdb_eval::GraphLawDuckDB;
