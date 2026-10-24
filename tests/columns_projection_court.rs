//! SQLRDF-25 Row 2 court — `COLUMNS('pattern')` projection item
//! (`sql:Columns` / `sql:columnsPattern`). Ledger:
//! docs/jira/v26.10.22/SQLRDF-25-remaining-gaps.md (Row 2). The
//! EMPTY_PROJECTION law is unchanged. Chicago: real frozen-duckdb, real
//! table readback, no mocks, no sleeps.

use graphlaw::sql_ast::compile_turtle;
use graphlaw::tier::{Admitted, admit_bytes_tiered};

use graphlaw::duckdb_eval::GraphLawDuckDB;

const COLUMNS_DOC: &str = r#"
@prefix sql: <http://graphlaw.org/sql#> .
@prefix xsd: <http://www.w3.org/2001/XMLSchema#> .
@prefix ex:  <http://example.org/queries#> .

ex:Q a sql:SelectStatement ;
    sql:statementIndex 1 ;
    sql:fromTable [ sql:tableName "customer_ledger" ] ;
    sql:selectList (
        [ a sql:Columns ; sql:columnsPattern "^user_" ]
    ) .
"#;

fn db() -> GraphLawDuckDB {
    let db = GraphLawDuckDB::open_in_memory().unwrap();
    db.raw_connection()
        .execute_batch(
            "CREATE TABLE customer_ledger (user_id INTEGER, user_name VARCHAR, order_total DECIMAL);
             INSERT INTO customer_ledger VALUES (7, 'ada', 12.50), (8, 'grace', 4.25);",
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

#[test]
fn columns_regex_projection_reads_back_the_matched_column_set() {
    let compiled = compile_turtle(COLUMNS_DOC).unwrap();
    assert!(
        compiled.sql.to_ascii_uppercase().contains("COLUMNS("),
        "{}",
        compiled.sql
    );
    let admitted = admit_bytes_tiered(COLUMNS_DOC.as_bytes(), None, None, None).unwrap();
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
    assert_eq!(receipt.rows_emitted, 2, "exact rowcount");
    // Anti-vacuity: mutating the regex (e.g. "^order_") or dropping the
    // COLUMNS arm flips both assertions.
    // Real readback over the receipt's result triples (the real collaborator
    // state), inlined at the assert site.
    assert!(
        receipt
            .result_triples
            .iter()
            .any(|t| t.predicate.ends_with("/user_id")),
        "projected column user_id must appear"
    );
    assert!(
        receipt
            .result_triples
            .iter()
            .any(|t| t.predicate.ends_with("/user_name")),
        "projected column user_name must appear"
    );
    assert!(
        !receipt
            .result_triples
            .iter()
            .any(|t| t.predicate.ends_with("/order_total")),
        "non-matching column must be absent"
    );
    assert_eq!(column_values(&receipt, "user_name"), vec!["ada", "grace"]);
}

#[test]
fn columns_lambda_form_refuses_refusal_01() {
    // PENDING-LANE: the lambda form is refused typed until specified
    // (COLUMNS_LAMBDA arm) — a lambda node must never render SQL.
    let doc = COLUMNS_DOC.replace(
        "sql:columnsPattern \"^user_\"",
        "sql:columnsLambda [ sql:columnName \"user_id\" ]",
    );
    let err = compile_turtle(&doc).unwrap_err().to_string();
    assert!(err.contains("REFUSAL-01"), "{err}");
}

#[test]
fn empty_columns_pattern_refuses_refusal_01() {
    // PENDING-LANE: malformed-member twin — an empty pattern string must
    // refuse typed REFUSAL-01 before any engine call, never render
    // `COLUMNS('')`.
    let doc = COLUMNS_DOC.replace("\"^user_\"", "\"\"");
    let err = compile_turtle(&doc).unwrap_err().to_string();
    assert!(err.contains("REFUSAL-01"), "{err}");
}

#[test]
fn empty_projection_still_refuses_for_columns_surface() {
    let doc = COLUMNS_DOC.replace(
        "sql:selectList (\n        [ a sql:Columns ; sql:columnsPattern \"^user_\" ]\n    )",
        "sql:selectList ( )",
    );
    let err = compile_turtle(&doc).unwrap_err().to_string();
    assert!(err.contains("REFUSAL-01"), "{err}");
    assert!(err.to_lowercase().contains("empty projection list"), "{err}");
}
