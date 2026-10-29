//! WP P08 — the sieve becomes law: result-row envelope courts.
//!
//! Legs to the mission contract:
//! 1. 10^6-row in-engine COPY→Parquet→mount→GROUP BY lands INSIDE the
//!    default envelope (rows_emitted == 64) with a sealed v2 receipt.
//! 2. An unbounded 10^6-row AST refuses `RowEnvelopeExceeded` with
//!    `observed == 65`, `max == 64` — EXACT, the mid-materialization
//!    witness (a post-loop guard would report 10^6, an absent guard would
//!    return Ok).
//! 3. The refusal is NOT cached: after a sieve refusal the C1 cache is
//!    empty; a raised-envelope execution of the same statement succeeds,
//!    stores, and a subsequent default-envelope consult of that cached
//!    entry re-refuses with `observed == 100` (the consult-side re-check,
//!    distinguishable from the execution-side `observed == 65`).
//! 4. RAW differential: the same unbounded SELECT through
//!    `raw_connection` pulls every row (100 ≥ 65) where eval refuses at
//!    65 — the sieve guards the eval path, not the raw substrate.
//! 5. Wire legs over `graphlaw::abi::call_json`: exec `max_rows` envelope
//!    (absent → 64, 0 refused, raised → passes), the exact `ResourceLimit`
//!    (`duckdb_result_rows`) mapping, document-mode `BatchMemberFailed`
//!    carrying `limit`/`observed`/`max`, and the stream `max_rows`
//!    page-size distinction (the cursor was sieved at exec; pages are
//!    free).
//! 6. Bounded-window exactness (the refusal-flip witness, no tree
//!    mutation): `max_rows = 10` over 20 rows → `observed == 11`;
//!    `max_rows = 20` over 20 rows → `Ok(20)`. The court fails under any
//!    of: guard removed (Ok), guard moved post-loop (observed would be 20,
//!    not 11), guard inverted to `>=` (the max=20 leg would refuse).
//!
//! Chicago: real frozen-duckdb connections, real Parquet files in
//! `std::env::temp_dir()` (removed on drop), real ABI calls, no mocks, no
//! sleeps.

use std::path::PathBuf;

use graphlaw::abi::call_json;
use graphlaw::duckdb_eval::{
    DEFAULT_ROW_ENVELOPE, EvaluatorError, GraphLawDuckDB, ROW_ENVELOPE_LIMIT,
};
use graphlaw::sql_ast::{CompiledQuery, compile_turtle};
use serde_json::{Value, json};

const SUBJECT: &str = "urn:s/";
const VOCAB: &str = "urn:v/";

/// Drop guard: the temp Parquet artifacts are leases, removed on scope exit
/// (even under a failed assert).
struct TempParquet(PathBuf);

impl Drop for TempParquet {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.0);
    }
}

/// Builds a real Parquet file with `rows` rows (`v` = i, `g` = i % groups)
/// IN-ENGINE (CREATE TABLE AS SELECT over `range`) and COPY→Parquet.
fn build_parquet(rows: usize, groups: usize, tag: &str) -> (TempParquet, String) {
    let path = std::env::temp_dir().join(format!(
        "graphlaw-sieve-court-{}-{}.parquet",
        std::process::id(),
        tag
    ));
    let path_str = path.to_str().expect("utf8 temp path").to_string();
    let db = GraphLawDuckDB::open_in_memory().expect("in-memory engine");
    db.raw_connection()
        .execute_batch(&format!(
            "CREATE TABLE raw AS SELECT i AS v, i % {groups} AS g FROM range({rows}) t(i); \
             COPY raw TO '{}' (FORMAT PARQUET);",
            path_str.replace('\'', "''")
        ))
        .expect("in-engine range seed + COPY TO parquet");
    (TempParquet(path), path_str)
}

/// Mounts `path` as view `name` on `db`.
fn mount(db: &GraphLawDuckDB, name: &str, path: &str) {
    db.register_parquet_view(name, path, false)
        .expect("parquet view mounts");
}

/// GROUP BY reduction AST: `SELECT g, COUNT(v) AS n FROM {view} GROUP BY g`
/// — one row per group.
fn reduce_turtle(view: &str) -> String {
    format!(
        r#"
@prefix sql: <http://graphlaw.org/sql#> .
@prefix ex: <http://example.org/sieve#> .
ex:reduce a sql:SelectStatement ;
    sql:statementIndex 1 ;
    sql:fromTable [ sql:tableName "{view}" ] ;
    sql:selectList (
        [ sql:columnName "g" ]
        [ sql:aliasName "n" ; sql:expression [
            sql:function <http://graphlaw.org/functions#COUNT> ;
            sql:argument [ sql:columnName "v" ] ] ]
    ) ;
    sql:groupBy ( [ sql:columnName "g" ] ) .
"#
    )
}

/// Unbounded projection AST: `SELECT g FROM {view}` — one row per source
/// row, so a `rows`-row view emits `rows` rows.
fn unbounded_turtle(view: &str) -> String {
    format!(
        r#"
@prefix sql: <http://graphlaw.org/sql#> .
@prefix ex: <http://example.org/sieve#> .
ex:unbounded a sql:SelectStatement ;
    sql:statementIndex 1 ;
    sql:fromTable [ sql:tableName "{view}" ] ;
    sql:selectList ( [ sql:columnName "g" ] ) .
"#
    )
}

/// Compiles `turtle`, failing the test on a compile regression.
fn compile(turtle: &str) -> CompiledQuery {
    compile_turtle(turtle).expect("SELECT AST compiles")
}

/// Asserts the error is exactly the sieve refusal with the given values and
/// the wire limit name.
fn assert_sieve_refusal(err: EvaluatorError, want_observed: usize, want_max: usize) {
    match err {
        EvaluatorError::RowEnvelopeExceeded {
            limit, observed, max,
        } => {
            assert_eq!(limit, ROW_ENVELOPE_LIMIT, "wire limit name");
            assert_eq!(observed, want_observed, "exact observed rows");
            assert_eq!(max, want_max, "exact envelope");
        }
        other => panic!("expected RowEnvelopeExceeded, got: {other:?}"),
    }
}

/// Raw control leg: pulls EVERY row of `SELECT g FROM {view}` through
/// `raw_connection` — the differential against the sieved eval path.
fn raw_pull_all(db: &GraphLawDuckDB, view: &str) -> usize {
    let mut stmt = db
        .raw_connection()
        .prepare(&format!("SELECT g FROM {view}"))
        .expect("raw prepare");
    stmt.query_map([], |r| r.get::<_, i64>(0))
        .expect("raw query")
        .map(|r| r.expect("raw row"))
        .count()
}

// ---------------------------------------------------------------------------
// Leg 1: 10^6-row in-engine COPY→Parquet→mount→GROUP BY — inside envelope.
// ---------------------------------------------------------------------------

/// The mission leg: a million source rows reduce IN THE ENGINE to exactly
/// the 64 group rows the default envelope admits, with a sealed receipt.
/// Only the reduced surface ever materializes.
#[test]
fn million_row_groupby_reduction_lands_inside_envelope_with_sealed_receipt() {
    let (parquet, path_str) = build_parquet(1_000_000, 64, "1m");
    let db = GraphLawDuckDB::open_in_memory().expect("in-memory engine");
    mount(&db, "sieve_1m", &path_str);

    let compiled = compile(&reduce_turtle("sieve_1m"));
    let receipt = db
        .eval_ast_query("urn:sieve:reduce1m", &compiled, SUBJECT, VOCAB)
        .expect("GROUP BY reduction is inside the default envelope");

    assert_eq!(receipt.rows_emitted, 64, "64 groups, exactly the envelope");
    assert_eq!(receipt.result_triples.len(), 128, "64 rows x 2 columns");
    assert_ne!(receipt.query_hash, [0u8; 32], "receipt hash minted");
    assert!(!receipt.cached, "first execution, not a consult hit");
    assert!(
        receipt.receipt_v2_hex.is_some(),
        "v2 seal present on the receipt"
    );

    drop(parquet);
}

// ---------------------------------------------------------------------------
// Leg 2: unbounded 10^6-row AST refuses at 65/64, exact.
// ---------------------------------------------------------------------------

/// The same million-row view WITHOUT the reduction: the sieve stops the
/// eval at row 65 (mid-materialization, strictly pre-receipt). The exact
/// `observed == 65` is the flip witness: a post-loop guard would report
/// 1_000_000, an absent guard would return Ok.
#[test]
fn million_row_unbounded_ast_refuses_at_65_exact() {
    let (parquet, path_str) = build_parquet(1_000_000, 64, "1m-unbounded");
    let db = GraphLawDuckDB::open_in_memory().expect("in-memory engine");
    mount(&db, "sieve_1m", &path_str);

    let compiled = compile(&unbounded_turtle("sieve_1m"));
    let err = db
        .eval_ast_query("urn:sieve:unbounded1m", &compiled, SUBJECT, VOCAB)
        .expect_err("unbounded result set must refuse");
    assert_sieve_refusal(err, 65, DEFAULT_ROW_ENVELOPE);

    drop(parquet);
}

// ---------------------------------------------------------------------------
// Leg 3: refusal not cached + consult-side envelope re-check.
// ---------------------------------------------------------------------------

/// (a) After a sieve refusal the C1 cache is EMPTY (a refusal stores
/// nothing). (b) The same statement under a raised envelope executes,
/// emits all 100 rows, and STORES (cacheable SELECT). (c) A default-
/// envelope re-eval consults that cached 100-row entry and refuses with
/// `observed == 100` — the consult-side re-check, provably distinct from
/// the execution-side refusal (`observed == 65`).
#[test]
fn sieve_refusal_is_not_cached_and_consult_side_recheck_fires() {
    let (parquet, path_str) = build_parquet(100, 10, "100");
    let db = GraphLawDuckDB::open_in_memory().expect("in-memory engine");
    mount(&db, "sieve_100", &path_str);
    let compiled = compile(&unbounded_turtle("sieve_100"));

    // (a) execution-side refusal, nothing stored.
    let err = db
        .eval_ast_query("urn:sieve:u100", &compiled, SUBJECT, VOCAB)
        .expect_err("100-row result set over the 64 envelope must refuse");
    assert_sieve_refusal(err, 65, DEFAULT_ROW_ENVELOPE);
    assert!(
        db.shared_cache().is_empty(),
        "a refusal must never store a cache entry"
    );

    // (b) raised envelope executes and stores.
    let receipt = db
        .eval_ast_query_bounded("urn:sieve:u100", &compiled, SUBJECT, VOCAB, 128)
        .expect("raised envelope admits the full 100-row result");
    assert_eq!(receipt.rows_emitted, 100);
    assert!(!receipt.cached);
    assert_eq!(db.shared_cache().len(), 1, "successful eval stores");

    // (c) default envelope consults the cached 100-row entry: the
    // consult-side re-check refuses with the CACHED count (100), not the
    // execution-side stop row (65).
    let err = db
        .eval_ast_query("urn:sieve:u100", &compiled, SUBJECT, VOCAB)
        .expect_err("consult hit over the envelope must refuse");
    assert_sieve_refusal(err, 100, DEFAULT_ROW_ENVELOPE);

    drop(parquet);
}

// ---------------------------------------------------------------------------
// Leg 4: raw differential + bounded-window exactness (flip witness).
// ---------------------------------------------------------------------------

/// The raw substrate pulls every 100 rows (≥ 65) where the sieved eval
/// refuses at 65 — the sieve is an eval-path law, not an engine cap.
///
/// Flip witness in the same court: `max_rows = 10` over 20 rows refuses
/// with `observed == 11` (mid-materialization exactness — a post-loop
/// guard would say 20), and `max_rows = 20` over 20 rows returns Ok(20)
/// (the boundary is `>`, so a full fit passes; inverted to `>=` it would
/// refuse).
#[test]
fn raw_pulls_every_row_the_sieve_refuses_and_window_boundaries_are_exact() {
    let (parquet, path_str) = build_parquet(100, 10, "100-raw");
    let db = GraphLawDuckDB::open_in_memory().expect("in-memory engine");
    mount(&db, "sieve_100", &path_str);
    let compiled = compile(&unbounded_turtle("sieve_100"));

    // Raw control: the same unbounded SELECT, no sieve — all 100 rows.
    let raw = raw_pull_all(&db, "sieve_100");
    assert!(raw >= 65, "raw differential requires raw >= 65: got {raw}");
    assert_eq!(raw, 100, "raw pulls every row the eval path refuses");

    // Sieved differential on the identical view.
    let err = db
        .eval_ast_query("urn:sieve:raw-diff", &compiled, SUBJECT, VOCAB)
        .expect_err("eval refuses where raw pulls");
    assert_sieve_refusal(err, 65, DEFAULT_ROW_ENVELOPE);

    // Bounded-window exactness: guard fires at envelope + 1, not later.
    let err = db
        .eval_ast_query_bounded("urn:sieve:w10", &compiled, SUBJECT, VOCAB, 10)
        .expect_err("100-row result over a 10 envelope must refuse");
    assert_sieve_refusal(err, 11, 10);

    // Boundary is `>`: an EXACT fit passes. Fresh 20-row view so the
    // statement's result set is exactly the envelope.
    let (parquet20, path20) = build_parquet(20, 4, "20-exact");
    let db20 = GraphLawDuckDB::open_in_memory().expect("in-memory engine");
    mount(&db20, "sieve_20", &path20);
    let compiled20 = compile(&unbounded_turtle("sieve_20"));
    let ok = db20
        .eval_ast_query_bounded("urn:sieve:w20", &compiled20, SUBJECT, VOCAB, 20)
        .expect("20 rows fit a 20 envelope exactly (boundary is >)");
    assert_eq!(ok.rows_emitted, 20);
    assert!(!ok.cached, "first execution, not a consult hit");

    drop(parquet);
    drop(parquet20);
}

// ---------------------------------------------------------------------------
// Legs 5-10: wire surface via graphlaw::abi::call_json.
// ---------------------------------------------------------------------------

/// Exec request over the mounted parquet view; `extra` merged in. Each
/// exec opens its own fresh in-memory connection and mounts the view, so
/// every leg is independent and stateless across calls.
fn wire_exec(glob: &str, turtle: &str, extra: Value) -> Value {
    let mut req = json!({
        "op": "duckdb",
        "mode": "exec",
        "views": [{"name": "sieve_w80", "glob": glob, "hive": false}],
        "query_ast": "urn:sieve:wire",
        "ast": {"turtle": turtle},
        "subject_prefix": SUBJECT,
        "vocab_prefix": VOCAB,
    });
    let obj = req.as_object_mut().expect("object");
    for (k, v) in extra.as_object().expect("object") {
        obj.insert(k.clone(), v.clone());
    }
    call_json(&req)
}

/// Builds the 80-row / 16-group wire fixture.
fn wire_fixture(tag: &str) -> (TempParquet, String) {
    build_parquet(80, 16, tag)
}

/// Wire leg W1: exec with `max_rows` ABSENT → default envelope (64) in
/// force; the 16-group reduction passes and answers ok:true.
#[test]
fn wire_exec_absent_max_rows_uses_default_envelope_reduction_passes() {
    let (parquet, glob) = wire_fixture("w1");
    let out = wire_exec(&glob, &reduce_turtle("sieve_w80"), json!({}));
    assert_eq!(out["ok"], serde_json::Value::Bool(true), "{out}");
    assert_eq!(out["rows_emitted"], 16, "16 groups inside the default 64: {out}");

    drop(parquet);
}

/// Wire leg W2: the unbounded 80-row projection with `max_rows` absent
/// refuses as a typed ResourceLimit naming `duckdb_result_rows` with the
/// exact observed==65 / max==64.
#[test]
fn wire_exec_unbounded_refuses_resource_limit_duckdb_result_rows() {
    let (parquet, glob) = wire_fixture("w2");
    let out = wire_exec(&glob, &unbounded_turtle("sieve_w80"), json!({}));
    assert_eq!(out["ok"], serde_json::Value::Bool(false), "{out}");
    assert_eq!(
        out["error"]["kind"], "ResourceLimit",
        "typed ResourceLimit, not a generic DuckDb envelope: {out}"
    );
    let details = &out["error"]["details"];
    assert_eq!(details["code"], "ResourceLimit", "{out}");
    assert_eq!(details["limit"], ROW_ENVELOPE_LIMIT, "{out}");
    assert_eq!(details["observed"], 65, "exact mid-materialization stop: {out}");
    assert_eq!(details["max"], 64, "default envelope on the wire: {out}");

    drop(parquet);
}

/// Wire leg W3: the SAME unbounded projection with `max_rows: 200` passes
/// (all 80 rows) — the caller owns its envelope — and the returned cursor
/// pages by stream `max_rows` (page size, not a re-sieve): pages of 10,
/// then the remaining 70 in one final page.
#[test]
fn wire_exec_raised_envelope_passes_and_stream_max_rows_is_a_page_size() {
    let (parquet, glob) = wire_fixture("w3");
    let out = wire_exec(
        &glob,
        &unbounded_turtle("sieve_w80"),
        json!({"max_rows": 200}),
    );
    assert_eq!(out["ok"], serde_json::Value::Bool(true), "{out}");
    assert_eq!(out["rows_emitted"], 80, "raised envelope admits all 80: {out}");
    assert_eq!(out["more"], serde_json::Value::Bool(true), "{out}");
    let cursor = out["cursor"]
        .as_str()
        .expect("cursor id on the ok response")
        .to_string();

    // Stream page 1: max_rows is a PAGE SIZE (10), not a sieve.
    let page1 = call_json(&json!({
        "op": "duckdb", "mode": "stream", "cursor": cursor, "max_rows": 10
    }));
    assert_eq!(page1["ok"], serde_json::Value::Bool(true), "{page1}");
    assert_eq!(page1["rows"], 10, "page size honored: {page1}");
    assert_eq!(page1["more"], serde_json::Value::Bool(true), "{page1}");

    // Stream page 2: the remaining 70 rows clear the cursor in one page.
    let page2 = call_json(&json!({
        "op": "duckdb", "mode": "stream", "cursor": cursor, "max_rows": 200
    }));
    assert_eq!(page2["ok"], serde_json::Value::Bool(true), "{page2}");
    assert_eq!(page2["rows"], 70, "remainder drained in one page: {page2}");
    assert_eq!(page2["cursor"], serde_json::Value::Null, "{page2}");
    assert_eq!(page2["more"], serde_json::Value::Bool(false), "{page2}");

    drop(parquet);
}

/// Wire leg W4: `max_rows: 0` is refused (an envelope of zero rows is not
/// a lawful eval).
#[test]
fn wire_exec_zero_max_rows_refused() {
    let (parquet, glob) = wire_fixture("w4");
    let out = wire_exec(
        &glob,
        &unbounded_turtle("sieve_w80"),
        json!({"max_rows": 0}),
    );
    assert_eq!(out["ok"], serde_json::Value::Bool(false), "{out}");
    let message = out["error"]["message"].as_str().unwrap_or("");
    assert!(
        message.contains("at least 1"),
        "zero envelope refused with the typed message: {out}"
    );

    drop(parquet);
}

/// Document member sequence for the wire legs: member 0 is the 16-row
/// reduction, member 1 is `tail_turtle` (the sieve probe member).
fn wire_doc_turtle(tail_turtle: &str) -> String {
    format!(
        r#"
@prefix sql: <http://graphlaw.org/sql#> .
@prefix xsd: <http://www.w3.org/2001/XMLSchema#> .
@prefix ex: <http://example.org/sieve#> .
sql:doc a sql:StatementSequence ;
    sql:statements ( ex:m0 ex:m1 ) .
ex:m0 a sql:SelectStatement ;
    sql:statementIndex 0 ;
    sql:fromTable [ sql:tableName "sieve_w80" ] ;
    sql:selectList (
        [ sql:columnName "g" ]
        [ sql:aliasName "n" ; sql:expression [
            sql:function <http://graphlaw.org/functions#COUNT> ;
            sql:argument [ sql:columnName "v" ] ] ]
    ) ;
    sql:groupBy ( [ sql:columnName "g" ] ) .
{tail_turtle}
"#
    )
}

/// Wire leg W5: `document:true` with member 1 over the envelope → typed
/// `BatchMemberFailed` naming member 1 AND carrying the sieve limit fields
/// (`limit`/`observed`/`max`), with member 0 in the partial receipt.
#[test]
fn wire_document_member_over_envelope_is_batch_member_failed_with_limit_fields() {
    let (parquet, glob) = wire_fixture("w5");
    let tail =
        unbounded_turtle("sieve_w80").replace("ex:unbounded", "ex:m1");
    let turtle = wire_doc_turtle(&tail);
    let out = call_json(&json!({
        "op": "duckdb",
        "mode": "exec",
        "document": true,
        "views": [{"name": "sieve_w80", "glob": glob, "hive": false}],
        "query_ast": "urn:sieve:doc",
        "ast": {"turtle": turtle},
        "subject_prefix": SUBJECT,
        "vocab_prefix": VOCAB,
    }));
    assert_eq!(out["ok"], serde_json::Value::Bool(false), "{out}");
    assert_eq!(out["error"]["kind"], "DuckDb", "{out}");
    let details = &out["error"]["details"];
    assert_eq!(details["code"], "BatchMemberFailed", "{out}");
    assert_eq!(details["member"], 1, "the over-envelope member: {out}");
    assert_eq!(details["limit"], ROW_ENVELOPE_LIMIT, "sieve limit field: {out}");
    assert_eq!(details["observed"], 65, "{out}");
    assert_eq!(details["max"], 64, "{out}");
    let partial = &details["partial_receipt"];
    assert_eq!(partial["failed_at"], 1, "{out}");
    assert_eq!(
        partial["rows_emitted"],
        serde_json::json!([16]),
        "member 0 executed and committed: {out}"
    );
    assert_eq!(partial["members"].as_array().map(Vec::len), Some(1), "{out}");

    drop(parquet);
}

/// Wire leg W6: `document:true` with BOTH members inside the envelope →
/// ok:true, per-member rows [16, 5] (member 1: GROUP BY with a WHERE
/// g > 10 → groups 11..15).
#[test]
fn wire_document_within_envelope_answers_ok_with_per_member_rows() {
    let (parquet, glob) = wire_fixture("w6");
    let tail = format!(
        r#"ex:m1 a sql:SelectStatement ;
    sql:statementIndex 1 ;
    sql:fromTable [ sql:tableName "sieve_w80" ] ;
    sql:selectList (
        [ sql:columnName "g" ]
        [ sql:aliasName "n" ; sql:expression [
            sql:function <http://graphlaw.org/functions#COUNT> ;
            sql:argument [ sql:columnName "v" ] ] ]
    ) ;
    sql:whereClause [
        sql:operator sql:GreaterThan ;
        sql:leftOperand [ sql:columnName "g" ] ;
        sql:rightOperand [ sql:literalValue "10"^^xsd:integer ]
    ] ;
    sql:groupBy ( [ sql:columnName "g" ] ) .
"#
    );
    let turtle = wire_doc_turtle(&tail);
    let out = call_json(&json!({
        "op": "duckdb",
        "mode": "exec",
        "document": true,
        "views": [{"name": "sieve_w80", "glob": glob, "hive": false}],
        "query_ast": "urn:sieve:doc-ok",
        "ast": {"turtle": turtle},
        "subject_prefix": SUBJECT,
        "vocab_prefix": VOCAB,
    }));
    assert_eq!(out["ok"], serde_json::Value::Bool(true), "{out}");
    assert_eq!(
        out["rows_emitted"],
        serde_json::json!([16, 5]),
        "member rows [16 groups, 5 groups with g > 10]: {out}"
    );
    assert_eq!(out["failed_at"], serde_json::Value::Null, "{out}");

    drop(parquet);
}

