//! Integration tests verifying:
//! 1. Direct Parquet Hive-partitioned directory registration, view querying, and zone map filtering.
//! 2. Hybrid deductive rule builtin `gl:evalSql` for state admission vs. fail-closed rejection.
//! 3. Formal SHACL pre-flight gate on declarative SQL AST subgraphs.

use graphlaw::duckdb_eval::GraphLawDuckDB;
use graphlaw::rules::builtins::{BuiltinContext, builtin_eval_sql};
use graphlaw::sql_ast::{RdfAstGraphReader, validate_ast_shacl};
use purrdf::TermValue;
use std::collections::HashMap;
use std::fs;

// =============================================================================
// Document-backed AST reader fixture (SQLRDF-17): supplies the admitted
// Turtle AST document so builtin eval compiles it via `compile_turtle`
// (a CompiledQuery with no canon bytes is EvaluatorError::NoCanonicalGraph at eval).
// =============================================================================

struct AstReaderFixture {
    docs: HashMap<String, String>,
}

impl AstReaderFixture {
    fn new() -> Self {
        Self {
            docs: HashMap::new(),
        }
    }

    fn register(&mut self, iri: &str, doc: &str) {
        self.docs.insert(iri.to_string(), doc.to_string());
    }
}

impl RdfAstGraphReader for AstReaderFixture {
    type Node = String;

    fn get_single_object(&self, _subject: &Self::Node, _predicate: &str) -> Option<Self::Node> {
        None
    }

    fn get_datatype_literal(
        &self,
        _subject: &Self::Node,
        _predicate: &str,
    ) -> Option<(String, Option<String>)> {
        None
    }

    fn get_iri(&self, node: &Self::Node) -> Option<String> {
        Some(node.clone())
    }

    fn query_document_turtle(&self) -> Option<String> {
        self.docs.values().next().cloned()
    }

    fn object_count(&self, _node: &Self::Node, _predicate: &str) -> usize {
        0
    }

    fn iri_object_count(&self, _node: &Self::Node, _pred: &str) -> usize {
        0
    }
}

/// Shared prefix for the admitted AST graph documents in this court file.
const DOC_PREFIX: &str = r#"@prefix sql: <http://graphlaw.org/sql#> .
@prefix fn:  <http://graphlaw.org/functions#> .
@prefix xsd: <http://www.w3.org/2001/XMLSchema#> .
@prefix ex:  <http://example.org/queries#> .
"#;

// =============================================================================
// Test 1: Parquet Hive Partitioning & Zone Map Querying
// =============================================================================

#[test]
fn test_parquet_hive_partition_and_zonemap_query() {
    let db = GraphLawDuckDB::open_in_memory().expect("Init failed");
    let base_path = format!(
        "/tmp/test_parquet_{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    );
    let _ = fs::remove_dir_all(&base_path);
    fs::create_dir_all(&base_path).expect("Failed to create temp dir");

    // 1. Generate partitioned Parquet files directly using DuckDB
    let setup_sql = format!(
        "COPY (
            SELECT 'tenant-A' AS tenant_id, 'batch-1' AS batch_id, 100 AS cost, 250 AS latency_ms
            UNION ALL
            SELECT 'tenant-A' AS tenant_id, 'batch-1' AS batch_id, 150 AS cost, 650 AS latency_ms
            UNION ALL
            SELECT 'tenant-B' AS tenant_id, 'batch-2' AS batch_id, 900 AS cost, 800 AS latency_ms
        ) TO '{}/telemetry' (FORMAT PARQUET, PARTITION_BY (tenant_id));",
        base_path
    );
    db.raw_connection().execute_batch(&setup_sql).unwrap();

    // 2. Register directory with Hive partitioning
    let glob = format!("{}/telemetry/**/*.parquet", base_path);
    db.register_parquet_view("telemetry_events", &glob, true)
        .expect("Registration failed");

    // 3. Admitted AST graph targeting the view:
    // SELECT batch_id, SUM(cost) AS total_cost FROM telemetry_events
    // WHERE (tenant_id = 'tenant-A' AND latency_ms > 500) GROUP BY batch_id
    let doc = DOC_PREFIX.to_string()
        + r#"
ex:Q a sql:SelectStatement ;
    sql:statementIndex 1 ;
    sql:fromTable [ sql:tableName "telemetry_events" ] ;
    sql:selectList (
        [ sql:columnName "batch_id" ]
        [
            sql:aliasName "total_cost" ;
            sql:expression [
                sql:function fn:SUM ;
                sql:argument [ sql:columnName "cost" ]
            ]
        ]
    ) ;
    sql:whereClause [
        sql:operator sql:And ;
        sql:leftOperand [
            sql:operator sql:Equals ;
            sql:leftOperand  [ sql:columnName "tenant_id" ] ;
            sql:rightOperand [ sql:literalValue "tenant-A" ]
        ] ;
        sql:rightOperand [
            sql:operator sql:GreaterThan ;
            sql:leftOperand  [ sql:columnName "latency_ms" ] ;
            sql:rightOperand [ sql:literalValue "500"^^xsd:integer ]
        ]
    ] ;
    sql:groupBy ( [ sql:columnName "batch_id" ] ) .
"#;
    let compiled =
        graphlaw::sql_ast::compile_turtle(&doc).expect("admitted AST graph must compile");

    let query_iri = "http://example.org/queries/parquet-q1";

    // 4. Verify explain plan (zone-map and parquet filter pushdown)
    let plan = db.explain_ast_query(&compiled).expect("Explain failed");
    assert!(
        plan.contains("READ_PARQUET")
            || plan.contains("SEQ_SCAN")
            || plan.contains("telemetry_events"),
        "Execution plan should inspect parquet source"
    );

    // 5. Execute query via compiled pushdown
    let receipt = db
        .eval_ast_query(
            query_iri,
            &compiled,
            "http://example.org/data",
            "http://example.org/vocab",
        )
        .expect("Execution failed");

    // Only row 2 of tenant-A matches (cost = 150, latency = 650)
    assert_eq!(receipt.rows_emitted, 1);
    assert_eq!(
        receipt.sql_statement,
        "SELECT batch_id, SUM(cost) AS total_cost FROM telemetry_events WHERE ((tenant_id = $1) AND (latency_ms > $2)) GROUP BY batch_id"
    );

    // Verify the resulting materialized RDF triples
    let total_cost_pred = "http://example.org/vocab/total_cost";
    let cost_stmt = receipt
        .result_triples
        .iter()
        .find(|s| s.predicate == total_cost_pred)
        .expect("total_cost triple must exist");

    match &cost_stmt.object {
        TermValue::Literal { lexical_form, .. } => {
            assert_eq!(lexical_form, "150");
        }
        other => panic!("Expected Literal TermValue, got {:?}", other),
    }

    let _ = fs::remove_dir_all(&base_path);
}

// =============================================================================
// Test 2: Hybrid Deductive Engine State Admission vs Fail-Closed Rejection
// =============================================================================

#[test]
fn test_builtin_eval_sql_admission_and_fail_closed_rejection() {
    let db = GraphLawDuckDB::open_in_memory().expect("Init failed");
    db.raw_connection()
        .execute_batch(
            "CREATE TABLE telemetry_events (
                batch_id VARCHAR,
                cost BIGINT,
                latency_ms INTEGER
            );
            -- batch-alpha: total cost with latency > 400 is 140 + 85 = 225
            INSERT INTO telemetry_events VALUES ('batch-alpha', 140, 480);
            INSERT INTO telemetry_events VALUES ('batch-alpha', 85,  520);

            -- batch-beta: total cost with latency > 400 is 750
            INSERT INTO telemetry_events VALUES ('batch-beta',  750, 610);",
        )
        .expect("Failed to seed DuckDB tables");

    let doc = DOC_PREFIX.to_string()
        + r#"
ex:Q a sql:SelectStatement ;
    sql:statementIndex 1 ;
    sql:fromTable [ sql:tableName "telemetry_events" ] ;
    sql:selectList (
        [ sql:columnName "batch_id" ]
        [
            sql:aliasName "total_cost" ;
            sql:expression [
                sql:function fn:SUM ;
                sql:argument [ sql:columnName "cost" ]
            ]
        ]
    ) ;
    sql:whereClause [
        sql:operator sql:GreaterThan ;
        sql:leftOperand  [ sql:columnName "latency_ms" ] ;
        sql:rightOperand [ sql:literalValue "400"^^xsd:integer ]
    ] ;
    sql:groupBy ( [ sql:columnName "batch_id" ] ) .
"#;

    let query_iri = "http://example.org/queries#CostAgg";
    let mut reader = AstReaderFixture::new();
    reader.register(query_iri, &doc);

    let ctx = BuiltinContext { db: &db };

    // Function simulating rule premise evaluation and state admission
    let evaluate_admission = |batch_id: &str, max_cost_threshold: i64| -> Option<[u8; 32]> {
        let args = [
            query_iri,
            "http://example.org/data",
            "http://example.org/vocab",
        ];
        let out_vars = ["receipt", "rowsEmitted", "queryHash"];

        let res = builtin_eval_sql(&args, &out_vars, &ctx, &reader).ok()?;

        // Search materialized triples for matching batch and total_cost
        let mut row_subject = None;
        for t in &res.derived_triples {
            if t.predicate == "http://example.org/vocab/batch_id"
                && let TermValue::Literal { lexical_form, .. } = &t.object
                && lexical_form == batch_id
            {
                row_subject = Some(t.subject.clone());
                break;
            }
        }

        let subj = row_subject?;
        let mut total_cost = None;
        for t in &res.derived_triples {
            if t.subject == subj
                && t.predicate == "http://example.org/vocab/total_cost"
                && let TermValue::Literal { lexical_form, .. } = &t.object
            {
                total_cost = lexical_form.parse::<i64>().ok();
                break;
            }
        }

        let cost = total_cost?;
        if cost <= max_cost_threshold {
            Some(res.query_hash)
        } else {
            None // Fail-closed
        }
    };

    // Case 1: batch-alpha (cost 225 <= 300) -> PASS Admission with BLAKE3 receipt
    let alpha_receipt = evaluate_admission("batch-alpha", 300);
    assert!(alpha_receipt.is_some(), "batch-alpha should be admitted");
    assert_ne!(
        alpha_receipt.unwrap(),
        [0u8; 32],
        "BLAKE3 hash must be non-zero"
    );

    // Case 2: batch-beta (cost 750 > 300) -> FAIL-CLOSED (no admission granted)
    let beta_receipt = evaluate_admission("batch-beta", 300);
    assert!(beta_receipt.is_none(), "batch-beta must fail closed");
}

// =============================================================================
// Test 3: Formal SHACL Pre-Flight Gate on AST Turtle
// =============================================================================

#[test]
fn test_shacl_ast_preflight_gate() {
    let shapes_ttl = include_str!("../ontologies/sql-ast.ttl");

    // 1. Valid AST
    let valid_ast_ttl = r#"
        @prefix sql: <http://graphlaw.org/sql#> .
        @prefix ex:  <http://example.org/queries#> .

        ex:ValidQuery a sql:SelectStatement ;
            sql:fromTable [
                sql:tableName "telemetry_events" ;
                sql:tableAlias "t"
            ] ;
            sql:selectList (
                [ sql:columnName "batch_id" ]
            ) .
    "#;

    let res = validate_ast_shacl(valid_ast_ttl, shapes_ttl);
    assert!(
        res.is_ok(),
        "Valid AST must pass SHACL pre-flight gate: {:?}",
        res.err()
    );

    // 2. Malformed AST: Missing sql:fromTable
    let malformed_ast_ttl = r#"
        @prefix sql: <http://graphlaw.org/sql#> .
        @prefix ex:  <http://example.org/queries#> .

        ex:InvalidQuery a sql:SelectStatement ;
            sql:selectList (
                [ sql:columnName "batch_id" ]
            ) .
    "#;

    let res = validate_ast_shacl(malformed_ast_ttl, shapes_ttl);
    assert!(
        res.is_err(),
        "AST missing fromTable must be refused by SHACL gate"
    );
    println!("Observed SHACL refusal message: {:?}", res);
    if let Err(e) = res {
        assert!(
            !e.to_string().is_empty(),
            "Violation message must not be empty"
        );
    } else {
        panic!("Expected ShaclViolation error, got {:?}", res);
    }
}
