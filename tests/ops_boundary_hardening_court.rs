#![cfg(feature = "abi")]
//! Hardening courts for shacl_guard base threading, duckdb cursor/row caps,
//! policy entry counting, and registry/clock source laws. Real ABI dispatch.

use graphlaw::abi::call_json;
use serde_json::{Value, json};

fn limit_name(r: &Value) -> &str {
    assert_eq!(r["ok"], false, "{r}");
    assert_eq!(r["error"]["kind"], "ResourceLimit", "{r}");
    r["error"]["details"]["limit"]
        .as_str()
        .unwrap_or_else(|| panic!("ResourceLimit details.limit must be a string: {r}"))
}

/// (1) the guard parses with the caller's `base`: a relative-IRI shapes graph
/// wider than MAX_SHACL_DEGREE is measured, not skipped.
#[test]
fn shacl_guard_threads_base() {
    let kids: Vec<String> = (0..graphlaw::shacl_guard::MAX_SHACL_DEGREE + 1)
        .map(|i| format!("<c{i}>"))
        .collect();
    let ttl = format!(
        "@prefix sh: <http://www.w3.org/ns/shacl#> .\n<s> a sh:NodeShape ; sh:node {} .\n",
        kids.join(", ")
    );
    let over = graphlaw::shacl_guard::check_shape_limits_with_base(&ttl, Some("http://e.org/"));
    assert!(over.is_err(), "base-parsed guard must see the degree: {over:?}");
    let r = call_json(&json!({
        "op": "shacl", "data": {"text": "", "dialect": "ntriples"},
        "shapes": ttl, "base": "http://e.org/"
    }));
    assert_eq!(limit_name(&r), "shacl_validation_degree", "{r}");
}

const PFX: &str = "@prefix sql: <http://graphlaw.org/sql#> .\n@prefix ex: <http://example.org/q#> .\n";

/// The cursor registry is process-global: every duckdb court serializes here.
static REGISTRY: std::sync::Mutex<()> = std::sync::Mutex::new(());

fn ast_req(body: &str, session: Option<&str>) -> Value {
    let mut r = json!({
        "op": "duckdb", "mode": "exec", "query_ast": "urn:q",
        "ast": {"turtle": format!("{PFX}{body}")},
        "subject_prefix": "urn:row/", "vocab_prefix": "urn:p/",
    });
    if let Some(s) = session {
        r["session"] = json!(s);
    }
    r
}

fn create_req() -> Value {
    ast_req("ex:C a sql:CreateStatement ; sql:statementIndex 1 ; sql:targetTable \"t_hc\" ; sql:assignColumn \"a\" .\n", None)
}

fn create_in(session: &str, table: &str) -> Value {
    ast_req(
        &format!("ex:C a sql:CreateStatement ; sql:statementIndex 1 ; sql:targetTable \"{table}\" ; sql:assignColumn \"a\" .\n"),
        Some(session),
    )
}

fn insert_in(session: &str, table: &str) -> Value {
    ast_req(
        &format!("ex:I a sql:InsertStatement ; sql:statementIndex 1 ; sql:targetTable \"{table}\" ; sql:assignColumn \"a\" ; sql:assignValue 1 .\n"),
        Some(session),
    )
}

fn select_in(session: &str, table: &str) -> Value {
    ast_req(
        &format!("ex:S a sql:SelectStatement ; sql:statementIndex 1 ; sql:fromTable [ sql:tableName \"{table}\" ] ; sql:selectList ( [ sql:columnName \"a\" ] ) .\n"),
        Some(session),
    )
}

fn open_session(name: &str) {
    let r = call_json(&json!({"op": "duckdb", "mode": "open", "session": name}));
    assert_eq!(r["ok"], true, "{r}");
}

fn close_session(name: &str) {
    call_json(&json!({"op": "duckdb", "mode": "close", "session": name}));
}

/// Session with table `t` holding one row.
fn seeded(name: &str) {
    open_session(name);
    for r in [call_json(&create_in(name, "t")), call_json(&insert_in(name, "t"))] {
        assert_eq!(r["ok"], true, "{r}");
        // DML reports its affected-row count as a row, so INSERT mints a cursor.
        if let Some(c) = r["cursor"].as_str() {
            call_json(&json!({"op": "duckdb", "mode": "close", "cursor": c}));
        }
    }
}

fn fill_cursors(session: &str) -> Vec<String> {
    (0..graphlaw::duckdb_op::MAX_CURSORS)
        .map(|i| {
            let r = call_json(&select_in(session, "t"));
            assert_eq!(r["ok"], true, "cursor {i}: {r}");
            r["cursor"].as_str().unwrap_or_else(|| panic!("SELECT mints a cursor: {r}")).to_string()
        })
        .collect()
}

fn close_cursors(cs: &[String]) {
    for c in cs {
        let r = call_json(&json!({"op": "duckdb", "mode": "close", "cursor": c}));
        assert_eq!(r["ok"], true, "{r}");
    }
}

/// (2) exec max_rows above the cap, and the cursor registry cap.
#[test]
fn duckdb_caps_are_typed_limits() {
    let _g = REGISTRY.lock().unwrap_or_else(|e| e.into_inner());
    let mut req = create_req();
    req["max_rows"] = json!(graphlaw::duckdb_op::MAX_EXEC_ROWS + 1);
    assert_eq!(limit_name(&call_json(&req)), "duckdb_exec_max_rows");
    req["max_rows"] = json!(u64::MAX);
    assert_eq!(limit_name(&call_json(&req)), "duckdb_exec_max_rows");

    seeded("caps");
    let cs = fill_cursors("caps");
    let r = call_json(&select_in("caps", "t"));
    assert_eq!(limit_name(&r), "duckdb_cursors", "{r}");
    close_cursors(&cs);
    close_session("caps");
}

#[test]
fn exec_drain_cycles_never_hit_the_cap() {
    let _g = REGISTRY.lock().unwrap_or_else(|e| e.into_inner());
    seeded("cycles");
    for i in 0..100 {
        let r = call_json(&select_in("cycles", "t"));
        assert_eq!(r["ok"], true, "cycle {i}: {r}");
        let c = r["cursor"].as_str().expect("cursor").to_string();
        let s = call_json(&json!({"op": "duckdb", "mode": "stream", "cursor": c}));
        assert_eq!(s["ok"], true, "{s}");
        assert_eq!(s["more"], false, "{s}");
        assert!(s["cursor"].is_null(), "{s}");
    }
    close_session("cycles");
}

#[test]
fn drained_cursor_restream_is_refused() {
    let _g = REGISTRY.lock().unwrap_or_else(|e| e.into_inner());
    seeded("restream");
    let r = call_json(&select_in("restream", "t"));
    let c = r["cursor"].as_str().expect("cursor").to_string();
    let s = call_json(&json!({"op": "duckdb", "mode": "stream", "cursor": c}));
    assert_eq!(s["ok"], true, "{s}");
    let again = call_json(&json!({"op": "duckdb", "mode": "stream", "cursor": c}));
    assert_eq!(again["ok"], false, "{again}");
    assert_eq!(again["error"]["kind"], "DuckDb", "{again}");
    let again_msg = again["error"]["message"]
        .as_str()
        .unwrap_or_else(|| panic!("DuckDb refusal carries string error.message: {again}"));
    assert!(again_msg.contains("unknown or expired cursor"), "{again}");
    close_session("restream");
}

#[test]
fn zero_row_exec_mints_no_cursor() {
    let _g = REGISTRY.lock().unwrap_or_else(|e| e.into_inner());
    open_session("zero");
    let r = call_json(&create_in("zero", "t"));
    assert_eq!(r["ok"], true, "{r}");
    assert_eq!(r["rows_emitted"], 0, "{r}");
    assert!(r["cursor"].is_null(), "{r}");
    assert_eq!(r["more"], false, "{r}");
    close_session("zero");
}

#[test]
fn full_registry_refuses_insert_before_it_runs() {
    let _g = REGISTRY.lock().unwrap_or_else(|e| e.into_inner());
    seeded("full");
    let cs = fill_cursors("full");
    let r = call_json(&insert_in("full", "t"));
    assert_eq!(limit_name(&r), "duckdb_cursors", "{r}");
    close_cursors(&cs);
    // Row count unchanged: the refused INSERT never ran.
    let r = call_json(&select_in("full", "t"));
    assert_eq!(r["rows_emitted"], 1, "{r}");
    let c = r["cursor"].as_str().expect("cursor").to_string();
    call_json(&json!({"op": "duckdb", "mode": "close", "cursor": c}));
    close_session("full");
}

/// (4) policy entries are counted by the shape policy::admit reads.
#[test]
fn policy_object_form_is_counted() {
    let n = graphlaw::abi::MAX_POLICY_ENTRIES + 1;
    let entries: Vec<Value> = (0..n)
        .map(|i| json!({"state": format!("s{i}"), "action": "a", "outcomes": []}))
        .collect();
    for policy in [
        json!({"policy": entries.clone()}),
        Value::String(json!({"policy": entries.clone()}).to_string()),
    ] {
        let r = call_json(&json!({"op": "policy", "problem": {}, "policy": policy}));
        assert_eq!(limit_name(&r), "policy_entries", "{r}");
    }
}

/// (5)+(6) source laws: one clock read in op_law; no registry `.expect`.
#[test]
fn source_laws() {
    let gov = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/src/abi/ops/governance.rs")).unwrap();
    assert_eq!(gov.matches("SystemClock.now_unix()").count(), 1, "single clock read");
    assert!(!gov.contains("&SystemClock,"), "no per-call SystemClock");
    let dd = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/src/duckdb_op.rs")).unwrap();
    let prod = dd.split("#[cfg(test)]").next().unwrap();
    assert!(!prod.contains("registry lock\")"), "registry mutex expect");
    assert!(prod.contains("lock_configuration=true"));
}
