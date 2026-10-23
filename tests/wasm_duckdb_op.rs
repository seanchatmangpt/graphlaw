//! Op-15 `duckdb` court (GWS-16): cursor-streamed analytical eval, native ABI
//! surface (`graphlaw::abi::call`).
//!
//! Frozen-duckdb is a required dependency. The wasm module dispatches `duckdb`.
//! The op stays off the hashed 14-op table until a digest bump; that absence
//! is not a capability refusal. frozen-duckdb 2.0.2 emits `static=duckdb` for
//! wasm32-wasip1. The link is witnessed (2026-10-22); the recorded pin
//! `332ea218…` was not replaced. `WASI_SDK_PATH` must be a wasi-sdk root
//! (`bin/clang`, `share/cmake/wasi-sdk-p1.cmake`). Without it,
//! frozen-duckdb-sys `build.rs` fails typed (`WASI_SDK_PATH required`). This court exercises the
//! op through `graphlaw::abi::call` on the host.
//!
//! Proven here:
//! (a) exec/stream/close happy path over an injected CSV (the GWS-4 injection
//!     pattern: the corpus crosses the boundary as a file-view mount).
//! (b) chunk-limit typed refusals (`duckdb_chunk_rows`, `duckdb_chunk_bytes`)
//!     — ResourceLimit, never a trap; the cursor survives the byte-limit one.
//! (c) raw_call fuzz: adversarial requests never panic; every response is a
//!     typed JSON envelope.
//! (d) native `builtin_eval_sql` <-> ABI op-15 receipt/query_hash identity.
#![cfg(all(feature = "abi", not(target_arch = "wasm32")))]

use serde_json::{Value, json};

/// Run one request through the ABI and return the parsed response.
fn call(req: &Value) -> Value {
    serde_json::from_slice(&graphlaw::abi::call(req.to_string().as_bytes()))
        .expect("response is JSON")
}

/// A temp CSV corpus (the injected data of this court); deleted on drop.
struct CsvFixture {
    path: std::path::PathBuf,
}

impl CsvFixture {
    fn write(name: &str, header: &str, rows: &[String]) -> Self {
        static SEQ: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
        let n = SEQ.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let mut path = std::env::temp_dir();
        path.push(format!("gws16_{name}_{}_{n}.csv", std::process::id()));
        std::fs::write(&path, header).expect("write csv header");
        for r in rows {
            use std::io::Write as _;
            let mut f = std::fs::OpenOptions::new()
                .append(true)
                .open(&path)
                .expect("open csv");
            writeln!(f, "{r}").expect("append csv row");
        }
        CsvFixture { path }
    }

    fn glob(&self) -> String {
        self.path.display().to_string()
    }
}

impl Drop for CsvFixture {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.path);
    }
}

fn telemetry_rows() -> Vec<String> {
    vec![
        "batch-001,120,450".into(),
        "batch-001,80,600".into(),
        "batch-002,250,720".into(),
    ]
}

const TELEMETRY_HEADER: &str = "batch_id,cost,latency\n";
const AGG_SQL: &str = "SELECT batch_id, SUM(cost) AS total_cost FROM telemetry WHERE (latency > $1) GROUP BY batch_id";
/// SQLRDF-17: the op contract admits only graph-backed queries. The AGG
/// query rides the same Turtle AST document on both the native builtin
/// path and the ABI op path; `threshold` is inlined as the AST literal.
fn agg_turtle(threshold: i64) -> String {
    format!(
        r#"@prefix sql: <http://graphlaw.org/sql#> .
@prefix xsd: <http://www.w3.org/2001/XMLSchema#> .
@prefix ex: <https://example.org/queries/> .
ex:q1 a sql:SelectStatement ;
    sql:statementIndex 1 ;
    sql:fromTable [ sql:tableName "telemetry" ] ;
    sql:selectList (
        [ sql:columnName "batch_id" ]
        [ sql:aliasName "total_cost" ; sql:expression [
            sql:function <http://graphlaw.org/functions#SUM> ;
            sql:argument [ sql:columnName "cost" ] ] ]
    ) ;
    sql:whereClause [
        sql:operator sql:GreaterThan ;
        sql:leftOperand [ sql:columnName "latency" ] ;
        sql:rightOperand [ sql:literalValue "{threshold}"^^xsd:integer ]
    ] ;
    sql:groupBy ( [ sql:columnName "batch_id" ] ) .
"#
    )
}
const Q1: &str = "https://example.org/queries/q1";

/// Simple graph-backed projection: `SELECT batch_id FROM telemetry`.
fn batch_turtle() -> String {
    r#"@prefix sql: <http://graphlaw.org/sql#> .
@prefix xsd: <http://www.w3.org/2001/XMLSchema#> .
@prefix ex: <https://example.org/queries/> .
ex:q2 a sql:SelectStatement ;
    sql:statementIndex 1 ;
    sql:fromTable [ sql:tableName "telemetry" ] ;
    sql:selectList ( [ sql:columnName "batch_id" ] ) .
"#
    .to_string()
}
const SUBJECT: &str = "https://example.org/data";
const VOCAB: &str = "https://example.org/vocab";

/// Exec the standard aggregation over the fixture at `threshold`.
fn exec_latency_gt(rows: &[String], threshold: i64) -> Value {
    let csv = CsvFixture::write("happy", TELEMETRY_HEADER, rows);
    call(&json!({
        "op": "duckdb",
        "mode": "exec",
        "views": [{"name": "telemetry", "glob": csv.glob()}],
        "query_ast": Q1,
        "ast": {"turtle": agg_turtle(threshold)},
        "subject_prefix": SUBJECT,
        "vocab_prefix": VOCAB,
        "params": [threshold],
    }))
}

// --------------------------------------------------------------- (a) happy path

#[test]
fn exec_stream_close_happy_path_over_injected_csv() {
    let ex = exec_latency_gt(&telemetry_rows(), 500);
    assert_eq!(ex["ok"], Value::Bool(true), "exec refused: {ex}");
    assert_eq!(ex["rows_emitted"], 2);
    assert_eq!(ex["more"], Value::Bool(true));
    let cursor = ex["cursor"].as_str().expect("cursor id").to_string();

    // Stream page 1 (one row).
    let s1 = call(&json!({
        "op": "duckdb", "mode": "stream", "cursor": cursor, "max_rows": 1
    }));
    assert_eq!(s1["ok"], Value::Bool(true), "stream refused: {s1}");
    assert_eq!(s1["rows"], 1);
    assert_eq!(s1["more"], Value::Bool(true));
    let nt1 = s1["nquads"].as_str().expect("nquads");
    assert!(nt1.contains("<https://example.org/data/row_1>"), "{nt1}");
    assert!(nt1.contains("<https://example.org/vocab/total_cost>"), "{nt1}");

    // Stream page 2 (last row) — cursor goes null, more=false.
    let drained = call(&json!({
        "op": "duckdb", "mode": "stream", "cursor": cursor, "max_rows": 10
    }));
    assert_eq!(drained["rows"], 1);
    assert_eq!(drained["more"], Value::Bool(false));
    assert!(drained["cursor"].is_null(), "cursor must be null when drained");
    let nt2 = drained["nquads"].as_str().expect("nquads");
    assert!(nt2.contains("<https://example.org/data/row_2>"), "{nt2}");

    // Draining an exhausted cursor yields an empty page (not an error).
    let s3 = call(&json!({
        "op": "duckdb", "mode": "stream", "cursor": cursor, "max_rows": 10
    }));
    assert_eq!(s3["ok"], Value::Bool(true));
    assert_eq!(s3["rows"], 0);
    assert!(s3["cursor"].is_null());

    // Close is idempotent-ok.
    let c = call(&json!({"op": "duckdb", "mode": "close", "cursor": cursor}));
    assert_eq!(c["ok"], Value::Bool(true));
    let c2 = call(&json!({"op": "duckdb", "mode": "close", "cursor": "never-existed"}));
    assert_eq!(c2["ok"], Value::Bool(true));
}

// --------------------------------------------------------------- (b) chunk limits

#[test]
fn chunk_row_limit_is_a_typed_refusal() {
    let ex = exec_latency_gt(&telemetry_rows(), 0);
    assert_eq!(ex["ok"], Value::Bool(true), "{ex}");
    let cursor = ex["cursor"].as_str().unwrap().to_string();
    let out = call(&json!({
        "op": "duckdb", "mode": "stream", "cursor": cursor, "max_rows": 10_001
    }));
    assert_eq!(out["ok"], Value::Bool(false), "{out}");
    assert_eq!(out["error"]["kind"], "ResourceLimit", "{out}");
    assert_eq!(out["error"]["details"]["code"], "ResourceLimit");
    assert_eq!(out["error"]["details"]["limit"], "duckdb_chunk_rows");
    assert_eq!(out["error"]["details"]["max"], 10_000);
    // The exec-bound chunk_rows cap refuses the same way.
    let csv = CsvFixture::write("rows", TELEMETRY_HEADER, &telemetry_rows());
    let ex2 = call(&json!({
        "op": "duckdb", "mode": "exec",
        "views": [{"name": "telemetry", "glob": csv.glob()}],
        "query_ast": Q1,
        "ast": {"turtle": batch_turtle()},
        "subject_prefix": SUBJECT,
        "vocab_prefix": VOCAB,
        "chunk_rows": 10_001,
    }));
    assert_eq!(ex2["error"]["details"]["limit"], "duckdb_chunk_rows", "{ex2}");
}

#[test]
fn chunk_byte_limit_is_a_typed_refusal_and_cursor_survives() {
    // Ten ~1.9 MiB batch_id strings (under DuckDB's 2,000,000-byte CSV line
    // cap): one page of all rows (~19 MiB of N-Triples) must refuse typed;
    // the cursor survives for a smaller page.
    let rows: Vec<String> = (0..10)
        .map(|_| format!("{},1,1", "x".repeat(1_900_000)))
        .collect();
    let csv = CsvFixture::write("wide", TELEMETRY_HEADER, &rows);
    let ex = call(&json!({
        "op": "duckdb", "mode": "exec",
        "views": [{"name": "telemetry", "glob": csv.glob()}],
        "query_ast": "https://example.org/queries/q2",
        "ast": {"turtle": batch_turtle()},
        "subject_prefix": SUBJECT,
        "vocab_prefix": VOCAB,
    }));
    assert_eq!(ex["ok"], Value::Bool(true), "{ex}");
    let cursor = ex["cursor"].as_str().unwrap().to_string();
    let out = call(&json!({
        "op": "duckdb", "mode": "stream", "cursor": cursor, "max_rows": 10
    }));
    assert_eq!(out["ok"], Value::Bool(false), "{out}");
    assert_eq!(out["error"]["kind"], "ResourceLimit");
    assert_eq!(out["error"]["details"]["limit"], "duckdb_chunk_bytes");
    assert_eq!(out["error"]["details"]["code"], "ResourceLimit");
    // Cursor survives: a smaller page still streams.
    let retry = call(&json!({
        "op": "duckdb", "mode": "stream", "cursor": cursor, "max_rows": 1
    }));
    assert_eq!(retry["ok"], Value::Bool(true), "{retry}");
    assert_eq!(retry["rows"], 1);
}

// --------------------------------------------------------------- (c) fuzz

#[test]
fn raw_call_fuzz_never_traps() {
    let corpus: Vec<Value> = vec![
        json!({}),
        json!({"op": "duckdb"}),
        json!({"op": "duckdb", "mode": null}),
        json!({"op": "duckdb", "mode": 7}),
        json!({"op": "duckdb", "mode": "EXEC"}),
        json!({"op": "duckdb", "mode": "exec", "views": "telemetry"}),
        json!({"op": "duckdb", "mode": "exec", "views": [{"name": "v", "sql": "ATTACH 'x' AS y"}]}),
        json!({"op": "duckdb", "mode": "exec", "views": [{"name": "1bad; DROP TABLE x", "glob": "/etc/passwd"}]}),
        json!({"op": "duckdb", "mode": "exec", "views": [{"name": "v", "glob": "*.csv", "pragma": "SET memory_limit='2GB'"}]}),
        json!({"op": "duckdb", "mode": "exec", "query_ast": "q", "ast": {"turtle": "SELECT * FROM read_csv_auto('/etc/passwd')"}, "subject_prefix": "s", "vocab_prefix": "v"}),
        json!({"op": "duckdb", "mode": "exec", "query_ast": "q", "ast": {"turtle": "not a turtle document"}, "subject_prefix": "s", "vocab_prefix": "v"}),
        json!({"op": "duckdb", "mode": "exec", "query_ast": "q", "ast": {"text": "SELECT 1"}}),
        json!({"op": "duckdb", "mode": "exec", "query_ast": "q", "ast": {"text": "SELECT 1"}, "subject_prefix": "s", "vocab_prefix": "v"}),
        json!({"op": "duckdb", "mode": "exec", "query_ast": "q", "ast": {"turtle": batch_turtle()}, "subject_prefix": "s", "vocab_prefix": "v", "params": [[1, 2]]}),
        json!({"op": "duckdb", "mode": "exec", "query_ast": "q", "ast": {"turtle": batch_turtle()}, "subject_prefix": "s", "vocab_prefix": "v", "chunk_rows": 0}),
        json!({"op": "duckdb", "mode": "stream", "cursor": null}),
        json!({"op": "duckdb", "mode": "stream", "cursor": 5}),
        json!({"op": "duckdb", "mode": "stream", "cursor": "", "max_rows": -1}),
        json!({"op": "duckdb", "mode": "stream", "cursor": "x", "max_rows": 1e18}),
        json!({"op": "duckdb", "mode": "close", "cursor": {}}),
        json!({"op": "duckdb", "mode": "close"}),
        json!({"op": "DUCKDB", "mode": "exec"}),
    ];
    for (i, req) in corpus.iter().enumerate() {
        let out = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            serde_json::from_slice::<Value>(&graphlaw::abi::call(req.to_string().as_bytes()))
                .expect("json")
        }))
        .unwrap_or_else(|_| panic!("case {i} trapped: {req}"));
        assert!(
            out.get("ok").is_some(),
            "case {i} produced a non-envelope response: {req} -> {out}"
        );
        if out["ok"] == Value::Bool(false) {
            assert!(
                out["error"]["kind"].is_string(),
                "case {i} refusal is untyped: {out}"
            );
        }
    }
    // Unknown op: typed refusal, not a trap.
    let unknown = call(&json!({"op": "duckdb_no_such"}));
    assert_eq!(unknown["error"]["kind"], "Unsupported");
}

// --------------------------------------------------------------- (d) identity

/// Document-backed RDF-AST reader serving the admitted AGG Turtle AST
/// document — the real `builtin_eval_sql` front door, with a graph that
/// holds exactly one query (SQLRDF-17 contract: document-backed readers
/// only).
struct FixedQueryReader(String);

impl graphlaw::sql_ast::RdfAstGraphReader for FixedQueryReader {
    type Node = String;

    fn get_single_object(&self, _s: &String, _p: &str) -> Option<String> {
        None
    }
    fn get_datatype_literal(&self, _s: &String, _p: &str) -> Option<(String, Option<String>)> {
        None
    }
    fn get_iri(&self, _n: &String) -> Option<String> {
        None
    }
    fn object_count(&self, _n: &String, _p: &str) -> usize {
        0
    }
    fn iri_object_count(&self, _n: &String, _p: &str) -> usize {
        0
    }
    fn query_document_turtle(&self) -> Option<String> {
        Some(self.0.clone())
    }
}

#[test]
fn builtin_eval_sql_and_abi_op_receipt_query_hash_byte_identity() {
    use graphlaw::duckdb_eval::GraphLawDuckDB;
    use graphlaw::rules::builtins::{BuiltinContext, builtin_eval_sql};
    use graphlaw::sql_ast::compile_turtle;

    // Both admission paths compile the SAME Turtle AST document.
    let turtle = agg_turtle(500);

    // Path A: the real native builtin (the gl:evalSql front door).
    let db = GraphLawDuckDB::open_in_memory().expect("duckdb");
    db.raw_connection()
        .execute_batch(
            "CREATE TABLE telemetry (batch_id VARCHAR, cost BIGINT, latency INTEGER);
             INSERT INTO telemetry VALUES ('batch-001', 120, 450);
             INSERT INTO telemetry VALUES ('batch-001', 80, 600);
             INSERT INTO telemetry VALUES ('batch-002', 250, 720);",
        )
        .expect("seed");
    let ctx = BuiltinContext { db: &db };
    let result = builtin_eval_sql(
        &[Q1, SUBJECT, VOCAB],
        &["receipt", "rowsEmitted", "queryHash"],
        &ctx,
        &FixedQueryReader(turtle.clone()),
    )
    .expect("builtin eval");
    assert_eq!(result.rows_emitted, 2);
    let native_hash = hex::encode(result.query_hash);
    let native_receipt_iri = match &result.bindings[0].1 {
        graphlaw::purrdf::TermValue::Iri(iri) => iri.clone(),
        other => panic!("binding 0 is not an IRI: {other:?}"),
    };

    // Path B: ABI op-15 exec over an identical CSV-injected table, same AST
    // document.
    let compiled = compile_turtle(&turtle).expect("compile");
    let ex = exec_latency_gt(&telemetry_rows(), 500);
    assert_eq!(ex["sql"], Value::String(compiled.sql.clone()), "{ex}");
    assert_eq!(ex["rows_emitted"], Value::from(result.rows_emitted));
    assert_eq!(ex["query_hash"], Value::String(native_hash), "query_hash diverges");
    assert_eq!(ex["receipt"], Value::String(native_receipt_iri), "receipt IRI diverges");
}

// ------------------------------------------------- (e) SQLRDF-17 raw-SQL refusal

/// SQLRDF-17 dynamic falsifier: feeding raw SQL text to the op must return
/// the typed refusal, never an executed statement.
#[test]
fn raw_sql_text_is_refused_not_executed() {
    let csv = CsvFixture::write("raw", TELEMETRY_HEADER, &telemetry_rows());
    let ex = call(&json!({
        "op": "duckdb",
        "mode": "exec",
        "views": [{"name": "telemetry", "glob": csv.glob()}],
        "query_ast": Q1,
        "ast": {"text": AGG_SQL},
        "subject_prefix": SUBJECT,
        "vocab_prefix": VOCAB,
        "params": [500],
    }));
    assert_eq!(ex["ok"], Value::Bool(false), "raw SQL must not execute: {ex}");
    let msg = ex["error"]["details"]["message"].as_str().unwrap_or_default();
    assert!(msg.contains("SQLRDF-17"), "{ex}");
    assert!(msg.contains("ast.turtle"), "{ex}");
}
