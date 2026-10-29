// SPDX-FileCopyrightText: 2026 Sean Chatman <sean@graphlaw.org>
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Verification court for PurRDF 5-table Parquet columnar integration with embedded DuckDB.
//!
//! Confirms:
//! 1. Encoding an admitted RDF dataset into `purrdf-columnar` 5-table Parquet files.
//! 2. Mounting all 5 tables as zero-copy DuckDB views (`terms`, `quads`, etc.).
//! 3. Executing relational queries joining `quads` with `terms` via embedded DuckDB.
//! 4. Round-tripping Parquet files back through `purrdf_columnar::read` and verifying structural fidelity.

use graphlaw::duckdb_eval::GraphLawDuckDB;
use purrdf_columnar::{Compression, ParquetFiles, Table, project, read};
use std::fs;

#[test]
fn test_purrdf_columnar_duckdb_roundtrip_and_query() {
    let ttl = r#"
        @prefix ex: <http://example.org/> .
        @prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .

        ex:SovereigntyEngine a ex:System ;
            rdfs:label "GraphLaw Instant Insights" ;
            ex:vectorSpeed 1000000 ;
            ex:status "ALIVE" .

        ex:LitigationCase a ex:Case ;
            rdfs:label "In re Enterprise Audit" ;
            ex:uses ex:SovereigntyEngine .
    "#;

    let dataset = purrdf::parse_dataset(ttl.as_bytes(), "text/turtle", None)
        .expect("parse turtle into RdfDataset");

    let blobs = purrdf::ContentStore::default();
    let projection = project(dataset.as_ref(), &blobs).expect("project dataset into 5-table schema");
    let columnar_write = projection
        .to_parquet(Compression::Uncompressed)
        .expect("write to parquet");

    // Persist to a temporary directory
    let temp_dir = std::env::temp_dir().join(format!(
        "graphlaw-purrdf-columnar-test-{}",
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

    // Initialize embedded DuckDB and mount the 5 tables
    let db = GraphLawDuckDB::open_in_memory().expect("open DuckDB in memory");
    db.register_purrdf_parquet_views(&temp_dir)
        .expect("register PurRDF 5-table Parquet views");

    // Verify row counts and queries directly via DuckDB
    let mut stmt = db
        .raw_connection()
        .prepare("SELECT count(*) FROM terms")
        .expect("prepare terms count");
    let terms_count: i64 = stmt
        .query_row(frozen_duckdb::params![], |row| row.get(0))
        .expect("query terms count");
    assert!(terms_count > 0, "terms table must have rows");

    let mut stmt = db
        .raw_connection()
        .prepare("SELECT count(*) FROM quads")
        .expect("prepare quads count");
    let quads_count: i64 = stmt
        .query_row(frozen_duckdb::params![], |row| row.get(0))
        .expect("query quads count");
    assert!(quads_count > 0, "quads table must have rows");

    // Execute semantic join in DuckDB: Find labels of entities
    let join_sql = r#"
        SELECT s_term.lex AS subject, o_term.lex AS label
        FROM quads q
        JOIN terms s_term ON q.s = s_term.id
        JOIN terms p_term ON q.p = p_term.id
        JOIN terms o_term ON q.o = o_term.id
        WHERE p_term.lex = 'http://www.w3.org/2000/01/rdf-schema#label'
        ORDER BY subject;
    "#;

    let mut join_stmt = db.raw_connection().prepare(join_sql).expect("prepare join");
    let rows = join_stmt
        .query_map(frozen_duckdb::params![], |row| {
            let s: String = row.get(0)?;
            let l: String = row.get(1)?;
            Ok((s, l))
        })
        .expect("query map");

    let results: Vec<(String, String)> = rows.map(|r| r.unwrap()).collect();
    assert_eq!(results.len(), 2);
    assert_eq!(results[0].0, "http://example.org/LitigationCase");
    assert_eq!(results[0].1, "In re Enterprise Audit");
    assert_eq!(results[1].0, "http://example.org/SovereigntyEngine");
    assert_eq!(results[1].1, "GraphLaw Instant Insights");

    // Now read back through purrdf_columnar::read
    let mut file_array: [Vec<u8>; 5] = Default::default();
    for (i, table) in Table::ALL.iter().enumerate() {
        let bytes = fs::read(temp_dir.join(table.file_name())).expect("read parquet file");
        file_array[i] = bytes;
    }
    let parquet_files = ParquetFiles::from_array(file_array);
    let columnar_read = read(&parquet_files).expect("purrdf_columnar::read succeeds");

    // Verify dataset canonical form matches
    let orig_canon = purrdf::try_canonicalize(&dataset).expect("canon orig");
    let read_canon = purrdf::try_canonicalize(&columnar_read.dataset).expect("canon read");
    assert_eq!(
        orig_canon.nquads, read_canon.nquads,
        "reconstructed dataset matches byte-for-byte in canonical N-Quads"
    );

    // Clean up temp dir
    let _ = fs::remove_dir_all(temp_dir);
}
