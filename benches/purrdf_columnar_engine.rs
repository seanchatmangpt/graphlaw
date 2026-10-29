// SPDX-FileCopyrightText: 2026 Sean Chatman <sean@graphlaw.org>
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Divan benchmarks for PurRDF 5-table Parquet columnar projection and embedded DuckDB analytics.
//!
//! Measures:
//! 1. `project_and_encode_parquet`: Transforming resident RDF graphs into the PurRDF 5-table Parquet format.
//! 2. `mount_purrdf_parquet_views`: Registering zero-copy DuckDB virtual views over the Parquet files.
//! 3. `columnar_join_query`: Executing a relational join (`quads` + `terms`) over the mounted Parquet files in DuckDB.
//! 4. `columnar_reconstruction_read`: Strict decoding and structural validation of 5 Parquet files back into `purrdf::RdfDataset`.

use divan::Bencher;
use graphlaw::duckdb_eval::GraphLawDuckDB;
use purrdf_columnar::{Compression, project, read};
use std::fs;
use std::hint::black_box;
use std::sync::Mutex;

static ENGINE_LOCK: Mutex<()> = Mutex::new(());

fn generate_sample_turtle(num_entities: usize) -> String {
    let mut ttl = String::from(
        "@prefix ex: <http://example.org/> .\n\
         @prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .\n\
         @prefix xsd: <http://www.w3.org/2001/XMLSchema#> .\n",
    );
    for i in 0..num_entities {
        ttl.push_str(&format!(
            "ex:entity_{i} a ex:Entity ;\n\
             \trdfs:label \"Entity {i}\" ;\n\
             \tex:value {i} ;\n\
             \tex:related ex:entity_{} .\n",
            (i + 1) % num_entities
        ));
    }
    ttl
}


#[divan::bench(args = [100, 1000])]
fn project_and_encode_parquet(bencher: Bencher, scale: usize) {
    let ttl = generate_sample_turtle(scale);
    let dataset = purrdf::parse_dataset(ttl.as_bytes(), "text/turtle", None).unwrap();
    let blobs = purrdf::ContentStore::default();

    bencher.bench(|| {
        let projection = project(dataset.as_ref(), &blobs).unwrap();
        let columnar_write = projection.to_parquet(Compression::Uncompressed).unwrap();
        black_box(columnar_write);
    });
}

#[divan::bench(args = [100, 1000])]
fn columnar_reconstruction_read(bencher: Bencher, scale: usize) {
    let ttl = generate_sample_turtle(scale);
    let dataset = purrdf::parse_dataset(ttl.as_bytes(), "text/turtle", None).unwrap();
    let blobs = purrdf::ContentStore::default();
    let projection = project(dataset.as_ref(), &blobs).unwrap();
    let columnar_write = projection.to_parquet(Compression::Uncompressed).unwrap();
    let files = columnar_write.files;

    bencher.bench(|| {
        let reconstructed = read(&files).unwrap();
        black_box(reconstructed);
    });
}

#[divan::bench(args = [100, 1000])]
fn mount_purrdf_parquet_views(bencher: Bencher, scale: usize) {
    let _guard = ENGINE_LOCK.lock().unwrap();
    let ttl = generate_sample_turtle(scale);
    let dataset = purrdf::parse_dataset(ttl.as_bytes(), "text/turtle", None).unwrap();
    let blobs = purrdf::ContentStore::default();
    let projection = project(dataset.as_ref(), &blobs).unwrap();
    let columnar_write = projection.to_parquet(Compression::Uncompressed).unwrap();

    let temp_dir = std::env::temp_dir().join(format!("graphlaw-bench-mount-{}", scale));
    let _ = fs::create_dir_all(&temp_dir);
    for (table, bytes) in columnar_write.files.iter() {
        let _ = fs::write(temp_dir.join(table.file_name()), bytes);
    }

    bencher.bench_local(|| {
        let db = GraphLawDuckDB::open_in_memory().unwrap();
        db.register_purrdf_parquet_views(&temp_dir).unwrap();
        black_box(db);
    });

    let _ = fs::remove_dir_all(&temp_dir);
}

#[divan::bench(args = [100, 1000])]
fn columnar_join_query(bencher: Bencher, scale: usize) {
    let _guard = ENGINE_LOCK.lock().unwrap();
    let ttl = generate_sample_turtle(scale);
    let dataset = purrdf::parse_dataset(ttl.as_bytes(), "text/turtle", None).unwrap();
    let blobs = purrdf::ContentStore::default();
    let projection = project(dataset.as_ref(), &blobs).unwrap();
    let columnar_write = projection.to_parquet(Compression::Uncompressed).unwrap();

    let temp_dir = std::env::temp_dir().join(format!("graphlaw-bench-query-{}", scale));
    let _ = fs::create_dir_all(&temp_dir);
    for (table, bytes) in columnar_write.files.iter() {
        let _ = fs::write(temp_dir.join(table.file_name()), bytes);
    }

    let db = GraphLawDuckDB::open_in_memory().unwrap();
    db.register_purrdf_parquet_views(&temp_dir).unwrap();

    let join_sql = r#"
        SELECT s_term.lex AS subject, o_term.lex AS label
        FROM quads q
        JOIN terms s_term ON q.s = s_term.id
        JOIN terms p_term ON q.p = p_term.id
        JOIN terms o_term ON q.o = o_term.id
        WHERE p_term.lex = 'http://www.w3.org/2000/01/rdf-schema#label';
    "#;

    bencher.bench_local(|| {
        let mut stmt = db.raw_connection().prepare(join_sql).unwrap();
        let rows = stmt
            .query_map(frozen_duckdb::params![], |row| {
                let s: String = row.get(0)?;
                let l: String = row.get(1)?;
                Ok((s, l))
            })
            .unwrap();
        let count = rows.count();
        black_box(count);
    });

    let _ = fs::remove_dir_all(&temp_dir);
}

fn main() {
    divan::main();
}
