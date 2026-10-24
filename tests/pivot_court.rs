//! SQLRDF-25 Row 3 court — PIVOT / UNPIVOT statement classes
//! (`sql:PivotStatement` / `sql:UnpivotStatement`). Ledger:
//! docs/jira/v26.10.22/SQLRDF-25-remaining-gaps.md (Row 3); PIVOT column-set
//! readback is the conformance observable named by the parent ticket.
//! Chicago: real frozen-duckdb, real table readback, no mocks, no sleeps.

use graphlaw::sql_ast::compile_turtle;
use graphlaw::tier::{Admitted, admit_bytes_tiered};

use graphlaw::duckdb_eval::GraphLawDuckDB;

const PIVOT_DOC: &str = r#"
@prefix sql: <http://graphlaw.org/sql#> .
@prefix xsd: <http://www.w3.org/2001/XMLSchema#> .
@prefix ex:  <http://example.org/queries#> .

ex:Q a sql:PivotStatement ;
    sql:statementIndex 1 ;
    sql:pivotTable "region_sales" ;
    sql:pivotColumn "region" ;
    sql:pivotAggregate "sum" ;
    sql:pivotAggregateColumn "amount" ;
    sql:pivotInValue "east" , "west" .
"#;

const UNPIVOT_DOC: &str = r#"
@prefix sql: <http://graphlaw.org/sql#> .
@prefix xsd: <http://www.w3.org/2001/XMLSchema#> .
@prefix ex:  <http://example.org/queries#> .

ex:Q a sql:UnpivotStatement ;
    sql:statementIndex 1 ;
    sql:pivotTable "region_wide" ;
    sql:unpivotColumn "east" , "west" ;
    sql:unpivotNameColumn "region" ;
    sql:unpivotValueColumn "amount" .
"#;

fn db() -> GraphLawDuckDB {
    let db = GraphLawDuckDB::open_in_memory().unwrap();
    db.raw_connection()
        .execute_batch(
            "CREATE TABLE region_sales (region VARCHAR, amount INTEGER);
             INSERT INTO region_sales VALUES ('east', 100), ('east', 50), ('west', 70);",
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
fn pivot_statement_admits_and_reads_back_the_pivoted_column_set() {
    // PENDING-LANE: the compile_statement_arm PIVOT/UNPIVOT dispatch (extract
    // + render arms) is the pivot lane's in-flight edit; the SHACL shapes in
    // ontologies/sql-ast.ttl are landed. If the dispatch is missing, this
    // happy path refuses and the court stands as the gap witness.
    let compiled = compile_turtle(PIVOT_DOC).unwrap();
    let sql_upper = compiled.sql.to_ascii_uppercase();
    assert!(sql_upper.contains("PIVOT"), "{}", compiled.sql);
    let admitted = admit_bytes_tiered(PIVOT_DOC.as_bytes(), None, None, None).unwrap();
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
    assert_eq!(receipt.rows_emitted, 1, "one pivoted row");
    // The pivot column set: east -> 150, west -> 70. Removing the IN list (or
    // the aggregate) flips the column set / row count.
    // Real readback over the receipt's result triples (the real collaborator
    // state), inlined at the assert site.
    assert!(
        receipt
            .result_triples
            .iter()
            .any(|t| t.predicate.ends_with("/east")),
        "pivoted column east must appear"
    );
    assert!(
        receipt
            .result_triples
            .iter()
            .any(|t| t.predicate.ends_with("/west")),
        "pivoted column west must appear"
    );
    assert_eq!(column_values(&receipt, "east"), vec!["150"]);
    assert_eq!(column_values(&receipt, "west"), vec!["70"]);
}

#[test]
fn unpivot_statement_admits_and_reads_back_long_format() {
    // PENDING-LANE: UNPIVOT dispatch is the pivot lane's in-flight edit; see
    // the PIVOT twin above.
    let admitted = admit_bytes_tiered(UNPIVOT_DOC.as_bytes(), None, None, None).unwrap();
    let compiled = match admitted {
        Admitted::Query(q) => q,
        other => panic!("expected a compiled query, got {other:?}"),
    };

    let db = db();
    db.raw_connection()
        .execute_batch(
            "CREATE TABLE region_wide (east INTEGER, west INTEGER);
             INSERT INTO region_wide VALUES (150, 70);",
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
    assert_eq!(receipt.rows_emitted, 2, "long format: one row per region");
    assert_eq!(column_values(&receipt, "region"), vec!["east", "west"]);
    assert_eq!(column_values(&receipt, "amount"), vec!["150", "70"]);
}

#[test]
fn empty_pivot_in_list_refuses_refusal_01() {
    // Anti-vacuity twin: a PIVOT with an empty IN list (rdf:nil) refuses
    // typed REFUSAL-01 (the SHACL shape demands >= 1 sql:pivotInValue), never
    // renders a zero-column pivot.
    let doc = PIVOT_DOC.replace("sql:pivotInValue \"east\" , \"west\"", "sql:pivotInValue ( )");
    let err = compile_turtle(&doc).unwrap_err().to_string();
    assert!(err.contains("REFUSAL-01"), "{err}");
}
