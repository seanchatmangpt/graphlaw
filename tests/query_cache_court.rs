//! C1 compiled-plan cache courts (designs/C1-cache.md, courts F1-F5).
//!
//! Chicago: real frozen-duckdb connections, real table readbacks, real
//! receipts. No mocks, no sleeps. A cache court proves reuse by observable
//! receipt state (the `cached` bit + consult_ms), not call counting.

use graphlaw::duckdb_eval::{ExecutionReceipt, GraphLawDuckDB};
use graphlaw::query_cache::{CacheEntry, QueryCache};
use graphlaw::sql_ast::{CompiledQuery, SqlLiteral, compile_turtle};

/// Real seeded table, per court (no shared state across courts).
fn seeded_db() -> GraphLawDuckDB {
    let db = GraphLawDuckDB::open_in_memory().unwrap();
    db.raw_connection()
        .execute_batch(
            "CREATE TABLE main.telemetry_events (batch_id VARCHAR, latency_ms INTEGER);
             INSERT INTO main.telemetry_events VALUES ('alpha', 480), ('beta', 10), ('gamma', 900);",
        )
        .unwrap();
    db
}

/// Admitted AST graph for a parameterized SelectStatement over the seeded
/// table (same shape as the sql_rdf_dialect base_doc court corpus).
fn select_doc(threshold: i64) -> String {
    format!(
        r#"
@prefix sql: <http://graphlaw.org/sql#> .
@prefix xsd: <http://www.w3.org/2001/XMLSchema#> .
@prefix ex:  <http://example.org/queries#> .

ex:Q a sql:SelectStatement ;
    sql:statementIndex 1 ;
    sql:fromTable [ sql:tableName "telemetry_events" ] ;
    sql:selectList ( [ sql:columnName "batch_id" ] ) ;
    sql:whereClause [
        sql:operator sql:GreaterThan ;
        sql:leftOperand [ sql:columnName "latency_ms" ] ;
        sql:rightOperand [ sql:literalValue "{threshold}"^^xsd:integer ]
    ] .
"#
    )
}

fn compile_select(threshold: i64) -> CompiledQuery {
    let compiled = compile_turtle(&select_doc(threshold)).unwrap();
    assert!(
        compiled.cacheable,
        "SelectStatement arm must classify cacheable (C1-a bit)"
    );
    compiled
}

/// Real DML through the eval surface, compiled via `compile_turtle`'s
/// command arms (SQLRDF-17: every admitted CompiledQuery carries Canon bytes
/// and a receipt anchor; a hand-built CompiledQuery is EvaluatorError::NoCanonicalGraph
/// at eval). Mutating statements are never cacheable
/// (C1-a: only the SelectStatement arm classifies cacheable).
fn dml_compiled(doc: &str) -> CompiledQuery {
    let compiled = compile_turtle(doc).expect("command arm must compile");
    assert!(!compiled.cacheable, "command arms are never cacheable");
    assert!(
        compiled.canon_graph_bytes.is_some(),
        "command arms carry canon bytes"
    );
    compiled
}

/// Admitted AST graph for `INSERT INTO telemetry_events (batch_id) VALUES ('delta')`
/// (single-column INSERT: the command arm carries one assignment).
fn insert_doc() -> String {
    r#"
@prefix sql: <http://graphlaw.org/sql#> .
@prefix ex:  <http://example.org/queries#> .

ex:I a sql:InsertStatement ;
    sql:statementIndex 2 ;
    sql:targetTable "telemetry_events" ;
    sql:assignColumn "batch_id" ;
    sql:assignValue "delta" .
"#
    .to_string()
}

/// Admitted AST graph for `ATTACH ':memory:' AS external_mem`.
fn attach_doc() -> String {
    r#"
@prefix sql: <http://graphlaw.org/sql#> .
@prefix ex:  <http://example.org/queries#> .

ex:A a sql:AttachStatement ;
    sql:statementIndex 3 ;
    sql:targetTable "external_mem" .
"#
    .to_string()
}

/// Evals the threshold select; None if it failed.
fn eval_select(db: &GraphLawDuckDB, threshold: i64) -> ExecutionReceipt {
    let compiled = compile_select(threshold);
    db.eval_ast_query(
        "http://example.org/queries#Q",
        &compiled,
        "urn:r:",
        "urn:p:",
    )
    .expect("select must evaluate")
}

#[test]
/// F1: two connections, one cache entry. A receipt stored by connection A
/// is served to connection B from the shared cache — cached bit set,
/// consult_ms below the stored execution_ms, triples identical.
fn f1_two_connections_one_entry() {
    let cache = std::sync::Arc::new(QueryCache::new());
    let db_a = GraphLawDuckDB::open_in_memory_with_cache(cache.clone()).unwrap();
    let db_b = GraphLawDuckDB::open_in_memory_with_cache(cache.clone()).unwrap();
    for db in [&db_a, &db_b] {
        db.raw_connection()
            .execute_batch(
                "CREATE TABLE main.telemetry_events (batch_id VARCHAR, latency_ms INTEGER);
                 INSERT INTO main.telemetry_events VALUES ('alpha', 480), ('cross_conn', 777);",
            )
            .unwrap();
    }

    let miss = eval_select(&db_a, 400);
    assert!(!miss.cached, "first eval on A must be a miss");
    assert_eq!(miss.rows_emitted, 2, "alpha + cross_conn rows");
    assert_eq!(cache.len(), 1, "exactly one entry stored");
    assert_eq!(cache.epoch(), 0, "selects never bump the epoch");

    let hit = eval_select(&db_b, 400);
    assert!(
        hit.cached,
        "connection B must be served from the shared entry"
    );
    assert!(
        hit.consult_ms <= miss.execution_ms,
        "consult must not cost more than the stored exec"
    );
    assert_eq!(
        miss.result_triples, hit.result_triples,
        "hit rows must equal stored rows"
    );
    assert_eq!(
        miss.query_hash, hit.query_hash,
        "hash identity survives the cache roundtrip"
    );
    assert_eq!(cache.len(), 1, "hit must not add entries");
}

#[test]
/// F2: a flipped literal parameter forks to a different cache key —
/// `H_query = BLAKE3(Canon(G_AST) || P_params)` (sql-rdf Rec. section 1.4
/// first formula) folds P_params into the key, so two thresholds occupy two
/// entries and neither serves the other's rows.
fn f2_flipped_literal_forks() {
    let db = seeded_db();
    let r400 = eval_select(&db, 400);
    let r401 = eval_select(&db, 401);
    assert_ne!(r400.query_hash, r401.query_hash, "P_params is in the key");
    assert!(
        !r400.cached && !r401.cached,
        "two distinct keys, both misses"
    );
    assert_eq!(cache_len(&db), 2, "two entries after two distinct misses");

    let again400 = eval_select(&db, 400);
    assert!(again400.cached);
    assert_eq!(again400.query_hash, r400.query_hash);
    assert_eq!(
        again400.result_triples, r400.result_triples,
        "threshold 400 must never be served 401's rows"
    );
}

#[test]
/// F3: DML is never stored. The INSERT executes for real (row readback
/// proves it landed), yet the cache stays empty and a subsequent select
/// still evaluates (not served from cache).
fn f3_dml_never_stored() {
    let db = seeded_db();
    let dml = dml_compiled(&insert_doc());
    let dml_receipt = db
        .eval_ast_query("http://example.org/queries#Q", &dml, "urn:r:", "urn:p:")
        .expect("DML executes through the same surface");
    assert!(!dml_receipt.cached);

    // Real readback: the INSERT landed.
    let conn = db.raw_connection();
    let mut stmt = conn
        .prepare("SELECT count(*) FROM main.telemetry_events WHERE batch_id = 'delta'")
        .unwrap();
    let mut rows = stmt.query([]).unwrap();
    let row = rows.next().unwrap().unwrap();
    let landed: i64 = row.get(0).unwrap();
    assert_eq!(
        landed, 1,
        "INSERT must really land, not be skipped by the cache"
    );

    assert_eq!(cache_len(&db), 0, "DML must never be stored");
    assert_eq!(cache_epoch(&db), 1, "successful DML bumps the epoch once");

    let sel = eval_select(&db, 400);
    assert!(
        !sel.cached,
        "post-DML select is a fresh miss at the new epoch"
    );
    assert_eq!(cache_len(&db), 1);
}

#[test]
/// F4: raw-SQL through `raw_connection` is never cached and never
/// consults the cache — the cache remains empty, and raw use does not
/// bump the epoch by itself (the caller must bump; this court pins the
/// contract that raw use is invisible unless `bump` is called).
fn f4_raw_sql_never_cached() {
    let db = seeded_db();
    // Seed a select into the cache first.
    let stored = eval_select(&db, 400);
    assert!(!stored.cached);

    // Raw mutating SQL through the raw connection: invisible to the cache
    // until the caller bumps.
    db.raw_connection()
        .execute_batch("INSERT INTO main.telemetry_events VALUES ('raw_row', 5);")
        .unwrap();
    assert_eq!(cache_len(&db), 1, "raw SQL neither stores nor consults");
    assert_eq!(cache_epoch(&db), 0, "raw use does not auto-bump");

    let hit = eval_select(&db, 400);
    assert!(
        hit.cached,
        "cache still serves the pre-raw entry (caller must bump)"
    );

    // The contract: caller-side bump invalidates.
    db.shared_cache().bump();
    let miss = eval_select(&db, 400);
    assert!(
        !miss.cached,
        "after an explicit bump the entry must be gone"
    );
}

#[test]
/// F5: ATTACH through the eval surface bumps the epoch, so an entry stored
/// before the ATTACH misses afterwards — even for the identical compiled
/// query and parameters.
fn f5_attach_bump_forces_miss() {
    let db = seeded_db();
    let pre = eval_select(&db, 400);
    assert!(!pre.cached);
    assert_eq!(cache_epoch(&db), 0);

    let attach = dml_compiled(&attach_doc());
    db.eval_ast_query("http://example.org/queries#Q", &attach, "urn:r:", "urn:p:")
        .expect("ATTACH executes through the eval surface");
    assert_eq!(cache_epoch(&db), 1, "ATTACH bumps once");
    assert_eq!(cache_len(&db), 0, "bump drops all entries");

    let post = eval_select(&db, 400);
    assert!(
        !post.cached,
        "identical query+params must miss after ATTACH"
    );
    assert_eq!(cache_epoch(&db), 1, "selects never bump");
}

#[test]
/// register_parquet_view bumps the epoch (a new view changes the data
/// surface every cached entry may have read).
fn parquet_view_registration_bumps() {
    let db = seeded_db();
    let stored = eval_select(&db, 400);
    assert!(!stored.cached);
    // A failing registration must still leave the cache untouched.
    let before = cache_epoch(&db);
    let _ = db.register_parquet_view("nope_v", "/nonexistent/path/*.parquet", false);
    // Registration with a real (missing) file fails at DuckDB; whether the
    // bump fired must not matter because no cached entry may survive a
    // mutating surface change — pin the successful-path bump with a real
    // parquet file instead.
    let dir = std::env::temp_dir().join(format!("gl_c1_parquet_{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let file = dir.join("part-0.parquet");
    db.raw_connection()
        .execute_batch(&format!(
            "COPY (SELECT * FROM (VALUES ('alpha', 480)) t(batch_id, latency_ms)) TO '{}' (FORMAT PARQUET);",
            file.display()
        ))
        .unwrap();
    db.register_parquet_view("telemetry_v", &file.display().to_string(), false)
        .expect("real parquet view registers");
    assert_eq!(
        cache_epoch(&db),
        before + 1,
        "successful view registration bumps once"
    );
    let miss = eval_select(&db, 400);
    assert!(!miss.cached, "post-view-registration eval misses");
    std::fs::remove_dir_all(&dir).ok();
}

#[test]
/// Unit-level: epoch mismatch is a miss and evicts; grounding divergence is
/// a miss for the same hash; store over the same key replaces.
fn cache_unit_epoch_and_grounding() {
    let cache = QueryCache::new();
    let h = [7u8; 32];
    let receipt = ExecutionReceipt {
        query_iri: "urn:q".into(),
        query_hash: h,
        sql_statement: "SELECT 1".into(),
        rows_emitted: 1,
        execution_ms: 1.0,
        result_triples: vec![],
        cached: false,
        consult_ms: 0.0,
        artifact_digest_hex: None,
        artifact_bytes: None,
        artifact_warning: None,
        // LANE receipt_v2 minimal unblock (compile-freeze SLA, disclosed):
        // two additive Phase 3 fields from src/duckdb_eval.rs.
        columnar_digest_hex: None,
        receipt_v2_hex: None,
    };
    let make_entry = |grounding: Vec<u8>, epoch: u64| CacheEntry {
        h_query: h,
        receipt: receipt.clone(),
        grounding,
        epoch,
        stored_at: graphlaw::query_cache::now_ms(),
    };
    cache.store(make_entry(b"canon-a".to_vec(), 0));
    assert!(
        cache.consult(h, Some(b"canon-a")).is_some(),
        "matching grounding hits"
    );

    cache.bump();
    assert!(
        cache.consult(h, Some(b"canon-a")).is_none(),
        "post-bump is a miss"
    );

    cache.store(make_entry(b"canon-a".to_vec(), cache.epoch()));
    assert!(
        cache.consult(h, Some(b"canon-b")).is_none(),
        "divergent grounding is a miss"
    );
    assert!(
        cache.consult(h, Some(b"canon-a")).is_some(),
        "matching grounding still hits"
    );
}

#[test]
/// SPOF Tier-1 #3 court: a panic in a lock-holding closure poisons the
/// mutex, but the cache must recover (`PoisonError::into_inner`) — entries
/// stored before the panic survive, and store/consult/len/bump all work
/// afterwards. The benign twin (`f1`) asserts normal unpoisoned behavior;
/// this court must flip if poison recovery is removed.
fn poisoned_mutex_recovers() {
    use std::panic::{AssertUnwindSafe, catch_unwind};

    let cache = QueryCache::new();
    let h = [11u8; 32];
    let receipt = ExecutionReceipt {
        query_iri: "urn:pre".into(),
        query_hash: h,
        sql_statement: "SELECT 1".into(),
        rows_emitted: 1,
        execution_ms: 1.0,
        result_triples: vec![],
        cached: false,
        consult_ms: 0.0,
        artifact_digest_hex: None,
        artifact_bytes: None,
        artifact_warning: None,
        // LANE receipt_v2 minimal unblock (compile-freeze SLA, disclosed).
        columnar_digest_hex: None,
        receipt_v2_hex: None,
    };
    cache.store(CacheEntry {
        h_query: h,
        receipt: receipt.clone(),
        grounding: b"canon".to_vec(),
        epoch: 0,
        stored_at: graphlaw::query_cache::now_ms(),
    });
    assert_eq!(cache.len(), 1);

    // Panic while actually holding the lock, caught at the test boundary.
    let panicked = catch_unwind(AssertUnwindSafe(|| {
        cache.with_entries(|_entries| {
            panic!("holder panic while the cache mutex is locked");
        })
    }));
    assert!(
        panicked.is_err(),
        "the panic must really happen (anti-vacuity)"
    );

    // Recovery: the pre-panic entry is intact and every surface works.
    assert_eq!(cache.len(), 1, "pre-panic entry survives the poison");
    let hit = cache
        .consult(h, Some(b"canon".as_slice()))
        .expect("cache must serve after a poisoned-holder panic");
    assert!(hit.0.cached, "served as a real hit");
    cache.store(CacheEntry {
        h_query: [22u8; 32],
        receipt,
        grounding: vec![],
        epoch: 0,
        stored_at: graphlaw::query_cache::now_ms(),
    });
    assert_eq!(cache.len(), 2, "post-panic store lands");
    assert!(cache.consult([22u8; 32], None).is_some());
    let bumped = cache.bump();
    assert_eq!(bumped, 1);
    assert_eq!(cache.len(), 0, "bump clears after recovery");
}

#[test]
/// SQLRDF-22 G3 positive court: a receipt WITH the artifact fields set
/// round-trips through the manual Serialize/Deserialize path — the keys are
/// emitted when Some and restored on read.
fn receipt_artifact_fields_roundtrip_serde() {
    let receipt = ExecutionReceipt {
        query_iri: "urn:q".into(),
        query_hash: [9u8; 32],
        sql_statement: "COPY ... TO 'out.bin'".into(),
        rows_emitted: 3,
        execution_ms: 2.5,
        result_triples: vec![],
        cached: false,
        consult_ms: 0.0,
        artifact_digest_hex: Some("deadbeef".into()),
        artifact_bytes: Some(42),
        artifact_warning: None,
        // LANE receipt_v2 minimal unblock (compile-freeze SLA, disclosed):
        // absent Phase 3 fields on a pre-Phase-3-shaped receipt.
        columnar_digest_hex: None,
        receipt_v2_hex: None,
    };
    let json = serde_json::to_string(&receipt).expect("serialize");
    assert!(
        json.contains("\"artifact_digest_hex\":\"deadbeef\"")
            && json.contains("\"artifact_bytes\":42"),
        "Some artifact fields must be emitted on the wire"
    );
    let back: ExecutionReceipt = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(back, receipt, "artifact fields must survive the roundtrip");
}

// -- cache witnesses through the public surface -----------------------------

fn cache_len(db: &GraphLawDuckDB) -> usize {
    db.shared_cache().len()
}

fn cache_epoch(db: &GraphLawDuckDB) -> u64 {
    db.shared_cache().epoch()
}

// SqlLiteral imported to keep the surface identical to sibling courts; the
// parameter fork in F2 travels through compile_turtle's own binding.
#[allow(unused)]
fn _type_witness(p: SqlLiteral) {}
