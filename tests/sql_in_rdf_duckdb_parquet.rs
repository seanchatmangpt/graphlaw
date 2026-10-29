// SPDX-FileCopyrightText: 2026 Sean Chatman <sean@graphlaw.org>
// SPDX-License-Identifier: MIT OR Apache-2.0

//! End-to-end integration court testing the combinatorial pipeline:
//! 1. Ingest RDF dataset (PurRDF).
//! 2. Project into 5-table Parquet profile (purrdf-columnar).
//! 3. Mount zero-copy virtual views in embedded DuckDB (GraphLawDuckDB).
//! 4. Lower declarative SQL-in-RDF AST into parameterized query (compile_turtle).
//! 5. Execute query on DuckDB against Parquet tables.
//! 6. Verify result triples and mint BLAKE3 ExecutionReceipt.

use graphlaw::duckdb_eval::GraphLawDuckDB;
use graphlaw::sql_ast::compile_turtle;
use purrdf_columnar::{Compression, project};
use std::fs;

#[test]
fn test_end_to_end_purrdf_sqlrdf_duckdb_pipeline() {
    let ttl_data = r#"
        @prefix ex: <http://example.org/> .
        @prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .

        ex:contract_001 a ex:Contract ;
            rdfs:label "Sovereign Analytics Master Agreement" ;
            ex:value 150000 .

        ex:contract_002 a ex:Contract ;
            rdfs:label "Air-gapped Cloud Tenant Provision" ;
            ex:value 75000 .
    "#;

    // 1. Parse via PurRDF
    let dataset = purrdf::parse_dataset(ttl_data.as_bytes(), "text/turtle", None)
        .expect("parse turtle into RdfDataset");

    // 2. Project into PurRDF 5-table Parquet
    let blobs = purrdf::ContentStore::default();
    let projection = project(dataset.as_ref(), &blobs).expect("project dataset into 5-table schema");
    let columnar_write = projection
        .to_parquet(Compression::Uncompressed)
        .expect("write to parquet");

    let temp_dir = std::env::temp_dir().join(format!(
        "graphlaw-e2e-pipeline-{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    fs::create_dir_all(&temp_dir).expect("create temp dir");

    for (table, bytes) in columnar_write.files.iter() {
        let file_path = temp_dir.join(table.file_name());
        fs::write(&file_path, bytes).expect("write parquet file");
    }

    // 3. Mount into embedded DuckDB
    let db = GraphLawDuckDB::open_in_memory().expect("open DuckDB");
    db.register_purrdf_parquet_views(&temp_dir)
        .expect("register 5-table Parquet views");

    // 4. Declarative SQL-in-RDF Query
    let query_turtle = r#"
        @prefix sql: <http://graphlaw.org/sql#> .
        @prefix xsd: <http://www.w3.org/2001/XMLSchema#> .
        @prefix ex:  <http://example.org/queries#> .

        ex:ParquetJoinQuery a sql:SelectStatement ;
            sql:statementIndex 1 ;
            sql:fromTable [
                sql:tableName "quads" ;
                sql:tableAlias "q"
            ] ;
            sql:selectList (
                [ sql:columnName "s" ]
                [ sql:columnName "p" ]
                [ sql:columnName "o" ]
            ) ;
            sql:limit 10 .
    "#;

    let compiled = compile_turtle(query_turtle).expect("compile SQL AST");

    // 5. Execute against Parquet-backed virtual views
    let receipt = db
        .eval_ast_query(
            "http://example.org/queries#ParquetJoinQuery",
            &compiled,
            "http://example.org/results",
            "http://example.org/vocab#",
        )
        .expect("eval AST query on Parquet views");

    // 6. Verify result & cryptographic receipt
    assert!(receipt.rows_emitted > 0, "must emit rows from parquet quads");
    assert!(!receipt.result_triples.is_empty(), "must produce RDF result triples");
    assert!(!receipt.query_hash.iter().all(|&b| b == 0), "BLAKE3 hash must be non-zero");

    let _ = fs::remove_dir_all(temp_dir);
}
