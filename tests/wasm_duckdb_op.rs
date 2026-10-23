//! Op-15 `duckdb` court (GWS-16): cursor-streamed analytical eval, native ABI
//! surface (`graphlaw::abi::call`).
//!
//! Frozen-duckdb is a host dependency. The wasm module does not dispatch
//! `duckdb` (`unknown op`). The op stays off the hashed 14-op table; that
//! absence is not a capability refusal. This court exercises the op through
//! `graphlaw::abi::call` on the host.
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
    assert!(
        nt1.contains("<https://example.org/vocab/total_cost>"),
        "{nt1}"
    );

    // Stream page 2 (last row) — cursor goes null, more=false.
    let drained = call(&json!({
        "op": "duckdb", "mode": "stream", "cursor": cursor, "max_rows": 10
    }));
    assert_eq!(drained["rows"], 1);
    assert_eq!(drained["more"], Value::Bool(false));
    assert!(
        drained["cursor"].is_null(),
        "cursor must be null when drained"
    );
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
    assert_eq!(
        ex2["error"]["details"]["limit"], "duckdb_chunk_rows",
        "{ex2}"
    );
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
    assert_eq!(
        ex["query_hash"],
        Value::String(native_hash),
        "query_hash diverges"
    );
    assert_eq!(
        ex["receipt"],
        Value::String(native_receipt_iri),
        "receipt IRI diverges"
    );
}

// ------------------------------------------------- (e) graph-backed lifecycle
//
// CTQ-2 lift: every statement class `compile_turtle` admits executes via
// op-15 exec (src/duckdb_op.rs — no statement-class gate; SQLRDF-17 still
// refuses raw `ast.text`; Connect/Disconnect remain REFUSAL-02 at compile).
// ABI `exec` opens a fresh in-memory connection per call, so the CREATE →
// INSERT → SELECT lifecycle rides ONE host `GraphLawDuckDB` connection via
// the same `compile_turtle` + `eval_ast_query` admission path the op uses,
// with real catalog/table readbacks and the §1.4 receipt assertions.

#[test]
fn host_lifecycle_create_insert_select_aggregate_with_receipts() {
    use graphlaw::duckdb_eval::GraphLawDuckDB;
    use graphlaw::sql_ast::compile_turtle;

    const Q_PREFIX: &str = "https://example.org/queries";

    // CREATE via the admitted CreateStatement expression form.
    let create_doc = r#"
@prefix sql: <http://graphlaw.org/sql#> .
@prefix ex:  <https://example.org/queries#> .
ex:create a sql:CreateStatement ;
    sql:statementIndex 1 ;
    sql:tableDefinition [ sql:tableName "telemetry" ; sql:columns (
        [ sql:columnName "batch_id" ; sql:columnType "VARCHAR" ]
        [ sql:columnName "cost" ; sql:columnType "BIGINT" ]
        [ sql:columnName "latency" ; sql:columnType "INTEGER" ]
    ) ] .
"#;
    let compiled_create = compile_turtle(create_doc).expect("CREATE compiles");
    assert!(
        compiled_create.sql.starts_with("CREATE TABLE"),
        "{}",
        compiled_create.sql
    );
    let db = GraphLawDuckDB::open_in_memory().expect("duckdb");
    let create_receipt = db
        .eval_ast_query(
            "https://example.org/queries#create",
            &compiled_create,
            SUBJECT,
            VOCAB,
        )
        .expect("CREATE executes");
    assert_eq!(create_receipt.rows_emitted, 0);
    assert_ne!(create_receipt.query_hash, [0u8; 32]);
    // Real readback: the table exists in the live DuckDB catalog.
    let tables: i64 = db
        .raw_connection()
        .query_row(
            "SELECT count(*)::BIGINT FROM duckdb_tables() \
             WHERE schema_name='main' AND table_name='telemetry'",
            [],
            |r| r.get(0),
        )
        .expect("catalog readback");
    assert_eq!(tables, 1, "CREATE must be visible in duckdb_tables()");

    // INSERT via the admitted scalar InsertStatement form: exactly ONE
    // assignColumn/assignValue pair per statement, so each INSERT appends
    // one row with one populated column (others NULL).
    let insert_doc = |col: &str, val: &str| {
        format!(
            r#"
@prefix sql: <http://graphlaw.org/sql#> .
@prefix xsd: <http://www.w3.org/2001/XMLSchema#> .
@prefix ex:  <https://example.org/queries#> .
ex:insert a sql:InsertStatement ;
    sql:statementIndex 1 ;
    sql:targetTable "telemetry" ;
    sql:assignColumn "{col}" ;
    sql:assignValue "{val}"^^xsd:string .
"#
        )
    };
    for (col, val) in [("cost", "120"), ("cost", "450")] {
        let doc = insert_doc(col, val);
        let compiled = compile_turtle(&doc).expect("INSERT compiles");
        assert!(compiled.sql.starts_with("INSERT INTO"), "{}", compiled.sql);
        let receipt = db
            .eval_ast_query(
                "https://example.org/queries#insert",
                &compiled,
                SUBJECT,
                VOCAB,
            )
            .expect("INSERT executes");
        // DuckDB projects the INSERT change-count as exactly one result row.
        assert_eq!(receipt.rows_emitted, 1, "INSERT count row: {receipt:?}");
        assert_ne!(receipt.query_hash, [0u8; 32]);
    }

    // Real readback: the two rows landed.
    let rows: i64 = db
        .raw_connection()
        .query_row("SELECT count(*)::BIGINT FROM telemetry", [], |r| r.get(0))
        .expect("table readback");
    assert_eq!(rows, 2);

    // SELECT aggregate over the same connection: exact rows_emitted + exact
    // values read back from the receipt's own triple projection. Rows are
    // (cost=120, latency NULL) and (cost=450, latency NULL), so the compiled
    // WHERE cost > 400 admits exactly one row and SUM(cost) = 450; a dropped
    // WHERE would emit 1 row too but SUM 570, so the VALUE is the mutation
    // fence.
    let agg_doc = r#"
@prefix sql: <http://graphlaw.org/sql#> .
@prefix xsd: <http://www.w3.org/2001/XMLSchema#> .
@prefix ex:  <https://example.org/queries#> .
ex:agg a sql:SelectStatement ;
    sql:statementIndex 1 ;
    sql:fromTable [ sql:tableName "telemetry" ] ;
    sql:selectList (
        [ sql:aliasName "total_cost" ; sql:expression [
            sql:function <http://graphlaw.org/functions#SUM> ;
            sql:argument [ sql:columnName "cost" ] ] ]
    ) ;
    sql:whereClause [
        sql:operator sql:GreaterThan ;
        sql:leftOperand [ sql:columnName "cost" ] ;
        sql:rightOperand [ sql:literalValue "400"^^xsd:integer ]
    ] .
"#;
    let agg = compile_turtle(agg_doc).expect("SELECT compiles");
    let receipt = db
        .eval_ast_query(&format!("{Q_PREFIX}/agg"), &agg, SUBJECT, VOCAB)
        .expect("SELECT executes");
    assert_ne!(receipt.query_hash, [0u8; 32]);
    assert_eq!(receipt.rows_emitted, 1, "WHERE cost > 400 admits one row");
    let totals: Vec<String> = receipt
        .result_triples
        .iter()
        .filter(|t| t.predicate.ends_with("/total_cost"))
        .filter_map(|t| match &t.object {
            graphlaw::purrdf::TermValue::Literal { lexical_form, .. } => {
                Some(lexical_form.clone())
            }
            _ => None,
        })
        .collect();
    assert_eq!(totals, vec!["450".to_string()], "SUM over the admitted row set");
}

// ------------------------------------------------- (f1) ABI session lifecycle
//
/// Mirrors the in-src court `session_lifecycle_create_insert_select`
/// (src/duckdb_op.rs tests) but entirely through `graphlaw::abi::call_json`
/// with the named-session surface: open → CREATE → INSERT ×2 → SELECT SUM →
/// stream readback (exact values) → close. The typed refusals are courted on
/// the same session: unknown-session exec, duplicate open, post-close exec.
#[test]
fn abi_session_lifecycle_create_insert_select_readback() {
    // Unique-per-court session name: the registry is process-local and
    // shared with the in-src unit courts.
    const SESSION: &str = "abi-court";

    let create = r#"
@prefix sql: <http://graphlaw.org/sql#> .
@prefix ex: <http://example.org/sessions#> .
ex:C a sql:CreateStatement ;
    sql:statementIndex 1 ;
    sql:targetTable "abi_session_events" ;
    sql:assignColumn "cost" .
"#;
    let insert = |val: i64| {
        format!(
            r#"
@prefix sql: <http://graphlaw.org/sql#> .
@prefix xsd: <http://www.w3.org/2001/XMLSchema#> .
@prefix ex: <http://example.org/sessions#> .
ex:I a sql:InsertStatement ;
    sql:statementIndex 1 ;
    sql:targetTable "abi_session_events" ;
    sql:assignColumn "cost" ;
    sql:assignValue "{val}"^^xsd:integer .
"#
        )
    };
    let select_sum = r#"
@prefix sql: <http://graphlaw.org/sql#> .
@prefix ex: <http://example.org/sessions#> .
ex:S a sql:SelectStatement ;
    sql:statementIndex 1 ;
    sql:fromTable [ sql:tableName "abi_session_events" ] ;
    sql:selectList (
        [ sql:aliasName "total_cost" ; sql:expression [
            sql:function <http://graphlaw.org/functions#SUM> ;
            sql:argument [ sql:columnName "cost" ] ] ]
    ) .
"#;

    let exec = |turtle: String| {
        call(&json!({
            "op": "duckdb",
            "mode": "exec",
            "session": SESSION,
            "query_ast": "urn:q",
            "ast": {"turtle": turtle},
            "subject_prefix": "urn:row/",
            "vocab_prefix": "urn:p/",
        }))
    };

    // (1) Open the named session.
    let open = call(&json!({"op": "duckdb", "mode": "open", "session": SESSION}));
    assert_eq!(open["ok"], Value::Bool(true), "{open}");
    assert_eq!(open["session"], SESSION, "{open}");

    // (2) Duplicate open on the live name is a typed refusal.
    let dup = call(&json!({"op": "duckdb", "mode": "open", "session": SESSION}));
    assert_eq!(dup["ok"], Value::Bool(false), "{dup}");
    assert!(
        dup["error"]["message"]
            .as_str()
            .unwrap_or_else(|| panic!("envelope missing error.message"))
            .contains("already exists"),
        "{dup}"
    );

    // (3) CREATE on the session connection.
    let c = exec(create.to_string());
    assert_eq!(c["ok"], Value::Bool(true), "{c}");

    // (4) INSERT ×2 — each lands on the SAME persistent connection.
    for val in [120_i64, 450] {
        let i = exec(insert(val));
        assert_eq!(i["ok"], Value::Bool(true), "{i}");
        assert_eq!(i["rows_emitted"], 1, "{i}");
    }

    // (5) SELECT SUM over the session: rows_emitted=1 and the streamed value
    // is exact — SUM=570 proves both INSERTs persisted on one connection
    // (a fresh-connection exec would hit the missing-table error instead).
    let s = exec(select_sum.to_string());
    assert_eq!(s["ok"], Value::Bool(true), "{s}");
    assert_eq!(s["rows_emitted"], 1, "{s}");
    let cur = s["cursor"].as_str().expect("select cursor").to_string();

    // (6) Stream the cursor and assert the exact aggregate value.
    let page = call(&json!({
        "op": "duckdb", "mode": "stream", "cursor": cur, "max_rows": 10
    }));
    assert_eq!(page["ok"], Value::Bool(true), "{page}");
    assert_eq!(page["rows"], 1, "{page}");
    let nt = page["nquads"].as_str().expect("nquads");
    assert!(
        nt.contains("\"570\""),
        "SUM(cost) over the two persisted rows must read back exactly 570: {page}"
    );

    // (7) Close the session, then exec on it is a typed refusal
    // (unknown or expired), and the name is freed.
    let closed = call(&json!({"op": "duckdb", "mode": "close", "session": SESSION}));
    assert_eq!(closed["ok"], Value::Bool(true), "{closed}");
    let post = exec(create.to_string());
    assert_eq!(post["ok"], Value::Bool(false), "{post}");
    assert_eq!(post["error"]["kind"], "DuckDb", "{post}");
    assert!(
        post["error"]["message"]
            .as_str()
            .unwrap_or_else(|| panic!("envelope missing error.message"))
            .contains("unknown or expired session"),
        "{post}"
    );

    // (8) Unknown-session exec (never opened) is the same typed refusal.
    let unknown = call(&json!({
        "op": "duckdb", "mode": "exec", "session": "abi-court-never-opened",
        "query_ast": "urn:q",
        "ast": {"turtle": select_sum},
        "subject_prefix": "urn:row/",
        "vocab_prefix": "urn:p/",
    }));
    assert_eq!(unknown["ok"], Value::Bool(false), "{unknown}");
    assert_eq!(unknown["error"]["kind"], "DuckDb", "{unknown}");
    assert!(
        unknown["error"]["message"]
            .as_str()
            .unwrap_or_else(|| panic!("envelope missing error.message"))
            .contains("unknown or expired session"),
        "{unknown}"
    );
}

// ------------------------------------------------- (f) SQLRDF-17 raw-SQL refusal

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
    assert_eq!(
        ex["ok"],
        Value::Bool(false),
        "raw SQL must not execute: {ex}"
    );
    let msg = ex["error"]["details"]["message"].as_str().unwrap_or_else(|| {
        panic!("refusal envelope missing error.details.message: {ex}")
    });
    assert!(msg.contains("SQLRDF-17"), "{ex}");
    assert!(msg.contains("ast.turtle"), "{ex}");
}

// ------------------------------------------------- (g) graph-backed DML via ABI

/// CTQ-2 lift witness at the ABI wall: a graph-backed InsertStatement (not
/// raw SQL) passes admission and REACHES the engine through `mode: exec` —
/// the observable is the DuckDB-level catalog error (a CSV-backed view is
/// not an insertable table on the per-call connection), never a SQLRDF-17
/// admission refusal. Raw SQL text stays typed-refused by (f); admission and
/// execution with receipts are courted host-side in (e) on one connection.
#[test]
fn graph_backed_insert_reaches_the_engine_via_abi() {
    let csv = CsvFixture::write("dml", TELEMETRY_HEADER, &telemetry_rows());
    let insert_turtle = r#"
@prefix sql: <http://graphlaw.org/sql#> .
@prefix xsd: <http://www.w3.org/2001/XMLSchema#> .
@prefix ex: <https://example.org/queries/> .
ex:q3 a sql:InsertStatement ;
    sql:statementIndex 1 ;
    sql:targetTable "telemetry" ;
    sql:assignColumn "batch_id" ;
    sql:assignValue "batch-009"^^xsd:string .
"#;
    let ex = call(&json!({
        "op": "duckdb",
        "mode": "exec",
        "views": [{"name": "telemetry", "glob": csv.glob()}],
        "query_ast": "https://example.org/queries/q3",
        "ast": {"turtle": insert_turtle},
        "subject_prefix": SUBJECT,
        "vocab_prefix": VOCAB,
    }));
    // Past admission: the refusal is the engine's catalog error, NOT a
    // SQLRDF-17/compile refusal. The SELECT-only lift is exactly this flip.
    assert_eq!(ex["ok"], Value::Bool(false), "{ex}");
    assert_eq!(ex["error"]["kind"], "DuckDb", "{ex}");
    let msg = ex["error"]["details"]["message"].as_str().unwrap_or_else(|| {
        panic!("refusal envelope missing error.details.message: {ex}")
    });
    assert!(
        msg.contains("Catalog Error"),
        "expected engine-level catalog error past admission: {ex}"
    );
    assert!(
        !msg.contains("SQLRDF-17"),
        "graph-backed DML must not be admission-refused: {ex}"
    );
}
