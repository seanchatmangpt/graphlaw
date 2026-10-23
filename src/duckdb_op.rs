//! ABI op `duckdb`: cursor-streamed analytical eval over frozen-duckdb,
//! linked into the wasm module with the rest of the kernel.
//!
//! Wire (JSON over the standard ABI):
//! - `exec`: `{"op":"duckdb","mode":"exec","views":[{name,glob,hive}],
//!   "query_ast":"<IRI>","ast":{"turtle","dialect"?},"subject_prefix",
//!   "vocab_prefix","params":[...]?,"chunk_rows"?}` — `ast.turtle` is the
//!   Turtle AST document, compiled via `compile_turtle` (SQLRDF-17: no raw
//!   SQL text admission; legacy `ast.text` is a typed refusal; wire `params`
//!   must one-for-one match the AST-carried literal parameters)
//!   → `{"ok":true,"receipt","query_hash","rows_emitted","cursor","more":bool}`
//! - `stream`: `{"op":"duckdb","mode":"stream","cursor","max_rows"}` →
//!   `{"ok":true,"nquads","rows","cursor"|null,"more":bool}`
//! - `close`: `{"op":"duckdb","mode":"close","cursor"}` → `{"ok":true}`
//!
//! DuckDB pragmas are enforced from compile-time constants (GAP-2): a caller
//! cannot override them from the wire. Errors are typed refusals (GAP-4) and
//! never trap. Cursor state is crash-only: it lives in this module's registry
//! and dies with the process/instance (documented ephemeral-cursor contract).

use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

use serde_json::{Value, json};

use crate::abi::{check_limit, fail_with_details, limit, str_field, term_json, Fail};
use crate::dialect::{Refusal, RefusalKind};
use crate::duckdb_eval::{GraphLawDuckDB, RdfTriple};
use crate::sql_ast::SqlLiteral;

/// DuckDB memory limit enforced on every connection (GAP-2: half the 64 MiB
/// sandbox wall; callers cannot override it from the wire).
pub const DUCKDB_MEMORY_LIMIT: &str = "32MiB";
/// DuckDB thread count enforced on graphlaw-embedded connections (GAP-2).
pub const DUCKDB_THREADS: u32 = 1;
/// Most rows a single `stream` page may carry (design bound; `chunk_rows` and
/// `max_rows` above it are typed refusals).
pub const MAX_CHUNK_ROWS: usize = 10_000;
/// Largest serialized page, in bytes (design bound); a page over it is a typed
/// `duckdb_chunk_bytes` ResourceLimit refusal and the cursor survives.
pub const MAX_CHUNK_BYTES: usize = 16 * 1024 * 1024;

/// Cursor registry, process-local (crash-only: worker kill invalidates).
fn cursors() -> &'static Mutex<HashMap<String, Cursor>> {
    static C: OnceLock<Mutex<HashMap<String, Cursor>>> = OnceLock::new();
    C.get_or_init(|| Mutex::new(HashMap::new()))
}

/// One open result cursor: row-major triples of the executed query.
struct Cursor {
    /// Triples grouped by source row (one `Vec` per row, in query order).
    rows: Vec<Vec<RdfTriple>>,
    /// Next row index to emit.
    next: usize,
}

/// Typed refusal for a DuckDB execution failure (`DuckDb` code, GAP-4).
fn duckdb_fail(message: impl std::fmt::Display) -> Fail {
    fail_with_details(
        Refusal {
            kind: RefusalKind::DuckDb,
            dialect: None,
            engine: None,
            message: format!("duckdb: {message}").into(),
        },
        json!({"code": "DuckDb", "message": message.to_string()}),
    )
}

/// Typed ResourceLimit refusal carrying a duckdb limit name.
fn duckdb_limit(name: &str, observed: usize, max: usize) -> Fail {
    limit(name, observed, max)
}

/// Open a pragma-enforced embedded connection (GAP-2: caller cannot override).
fn open_connection() -> Result<GraphLawDuckDB, Fail> {
    let conn = frozen_duckdb::Connection::open_in_memory().map_err(duckdb_fail)?;
    conn.execute_batch(&format!(
        "SET memory_limit='{}'; SET threads={}; SET preserve_insertion_order=false;",
        DUCKDB_MEMORY_LIMIT, DUCKDB_THREADS
    ))
    .map_err(duckdb_fail)?;
    Ok(GraphLawDuckDB::from_connection(conn))
}

/// Validate and bind wire params to `SqlLiteral`s.
/// Whether a wire param equals an AST-carried literal (the SQLRDF-17
/// one-for-one wire-vs-admission-record check in `exec`).
fn sql_literal_matches(w: &SqlLiteral, a: &SqlLiteral) -> bool {
    match (w, a) {
        (SqlLiteral::String(x), SqlLiteral::String(y)) => x == y,
        (SqlLiteral::Integer(x), SqlLiteral::Integer(y)) => x == y,
        (SqlLiteral::Float(x), SqlLiteral::Float(y)) => x == y,
        (SqlLiteral::Boolean(x), SqlLiteral::Boolean(y)) => x == y,
        (SqlLiteral::Null, SqlLiteral::Null) => true,
        _ => false,
    }
}

fn params_of(v: &Value) -> Result<Vec<SqlLiteral>, Fail> {
    match v.get("params") {
        None | Some(Value::Null) => Ok(Vec::new()),
        Some(Value::Array(a)) => a
            .iter()
            .map(|p| {
                Ok(match p {
                    Value::String(s) => SqlLiteral::String(s.clone()),
                    Value::Number(n) => {
                        if let Some(i) = n.as_i64() {
                            SqlLiteral::Integer(i)
                        } else {
                            SqlLiteral::Float(n.as_f64().unwrap_or(0.0))
                        }
                    }
                    Value::Bool(b) => SqlLiteral::Boolean(*b),
                    Value::Null => SqlLiteral::Null,
                    other => {
                        return Err(duckdb_fail(format!(
                            "unsupported param `{other}` (string, number, boolean or null)"
                        )))
                    }
                })
            })
            .collect(),
        Some(_) => Err(duckdb_fail("`params` must be an array")),
    }
}

/// Mount one wire view (`{"name","glob","hive"?}`) as a DuckDB view.
///
/// `.csv` globs ride `read_csv_auto` (the GWS-4 injection pattern: the corpus
/// crosses the boundary as a file-view mount); everything else rides
/// `register_parquet_view` (hive-partition aware). View entries accept only
/// `{name, glob, hive}` — SQL and pragmas are never caller-supplied.
fn mount_view(db: &GraphLawDuckDB, view: &Value) -> Result<(), Fail> {
    let name = view
        .get("name")
        .and_then(Value::as_str)
        .ok_or_else(|| duckdb_fail("view needs a string `name`"))?;
    let glob = view
        .get("glob")
        .and_then(Value::as_str)
        .ok_or_else(|| duckdb_fail("view needs a string `glob`"))?;
    let hive = view.get("hive").and_then(Value::as_bool).unwrap_or(false);
    crate::sql_ast::validate_identifier(name)
        .map_err(|e| duckdb_fail(format!("view `name`: {e}")))?;
    if view.get("sql").is_some() || view.get("pragma").is_some() || view.get("query").is_some() {
        return Err(duckdb_fail(
            "view entries accept only {name, glob, hive}; SQL and pragmas are not caller-supplied",
        ));
    }
    if glob.ends_with(".csv") || glob.contains("*.csv") {
        // D1 (epoch-coverage audit): route through `register_csv_view` so the
        // documented MUST holds — prepared-handle flush + C1 cache bump after
        // the raw `CREATE OR REPLACE VIEW`, mirroring `register_parquet_view`.
        db.register_csv_view(name, glob).map_err(duckdb_fail)?;
    } else {
        db.register_parquet_view(name, glob, hive)
            .map_err(duckdb_fail)?;
    }
    Ok(())
}

/// Mint an opaque cursor id (BLAKE3 over a process-local counter, the query
/// hash and the row count; never from caller-supplied strings alone).
fn cursor_id(query_hash: &[u8; 32], rows: usize) -> String {
    static COUNTER: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    let n = COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let mut h = blake3::Hasher::new();
    h.update(b"graphlaw.duckdb.cursor.v1");
    h.update(query_hash);
    h.update(&rows.to_le_bytes());
    h.update(&n.to_le_bytes());
    hex::encode(h.finalize().as_bytes())
}

/// The ABI surface of op-15; called from `src/abi.rs::dispatch`.
pub fn dispatch(v: &Value) -> Result<Value, Fail> {
    match v
        .get("mode")
        .and_then(Value::as_str)
        .ok_or_else(|| duckdb_fail("missing string field `mode`"))?
    {
        "exec" => exec(v),
        "stream" => stream(v),
        "close" => close(v),
        other => Err(duckdb_fail(format!(
            "unknown mode `{other}` (exec|stream|close)"
        ))),
    }
}

/// `mode: "exec"`: mount views, bind/run the AST SQL, open a cursor.
fn exec(v: &Value) -> Result<Value, Fail> {
    let db = open_connection()?;
    if let Some(views) = v.get("views") {
        let arr = views
            .as_array()
            .ok_or_else(|| duckdb_fail("`views` must be an array"))?;
        check_limit("duckdb_views", arr.len(), 64)?;
        for view in arr {
            mount_view(&db, view)?;
        }
    }
    let query_ast = str_field(v, "query_ast")?;
    let ast = v
        .get("ast")
        .filter(|a| a.is_object())
        .ok_or_else(|| duckdb_fail("missing `ast` object {\"turtle\": ...}"))?;
    // SQLRDF-17 (C2 hole P1): the op admits ONLY graph-backed queries. The
    // legacy `ast.text` raw-SQL field is refused typed — raw SQL text never
    // executes and never mints a receipt. The caller supplies the Turtle AST
    // document; it is compiled by the same `compile_turtle` law the native
    // builtin uses, so the receipt is BLAKE3(Canon(G_AST) || P_params)
    // (sql-rdf Rec. section 1.4) with canon pinned at compile time.
    if ast.get("text").is_some() {
        return Err(duckdb_fail(
            "SQLRDF-17 refused: legacy `ast.text` raw SQL is not an admitted admission path — supply the Turtle AST document as `ast.turtle` (compiled via compile_turtle)",
        ));
    }
    let turtle = str_field(ast, "turtle")?;
    if let Some(d) = ast.get("dialect") {
        let d = d.as_str().unwrap_or("");
        if !d.is_empty() && !d.eq_ignore_ascii_case("duckdb") {
            return Err(duckdb_fail(format!("unsupported ast dialect `{d}`")));
        }
    }
    let params = params_of(v)?;
    let mut compiled = crate::sql_ast::compile_turtle(turtle)
        .map_err(|e| duckdb_fail(format!("AST compile refusal: {e}")))?;
    // SQLRDF-17 F2(a): refuse non-query / catalog-mutating statements —
    // exec opens a fresh in-memory connection per call, so a receipted DDL
    // effect is unreachable; only the SelectStatement arm of compile_turtle
    // (the `cacheable` classification bit) is admissible here.
    if !compiled.cacheable {
        return Err(duckdb_fail(
            "SQLRDF-17/F2 refused: op-15 exec admits only graph-backed SELECT statements; catalog-mutating or non-query statements have no replayable effect on the per-call in-memory connection",
        ));
    }
    // Wire `params`, when supplied, must match the AST-carried literal
    // parameters one-for-one: the AST document is the admission record, so
    // its literals ARE P_params; extra or missing wire params are a typed
    // refusal, never a silent re-bind.
    if params.len() != compiled.params.len()
        || params
            .iter()
            .zip(compiled.params.iter())
            .any(|(w, a)| !sql_literal_matches(w, a))
    {
        return Err(duckdb_fail(format!(
            "SQLRDF-17 refused: wire params ({}) do not match the AST-carried literal parameters ({}); the Turtle AST document is the admission record and its literals are P_params",
            params.len(),
            compiled.params.len()
        )));
    }
    compiled.params = params;
    let subject_prefix = str_field(v, "subject_prefix")?;
    let vocab_prefix = str_field(v, "vocab_prefix")?;
    if let Some(n) = v.get("chunk_rows") {
        let n = n
            .as_u64()
            .ok_or_else(|| duckdb_fail("`chunk_rows` must be an unsigned integer"))?
            as usize;
        if n == 0 {
            return Err(duckdb_fail("`chunk_rows` must be at least 1"));
        }
        check_limit("duckdb_chunk_rows", n, MAX_CHUNK_ROWS)?;
    }
    let receipt = db
        .eval_ast_query(query_ast, &compiled, subject_prefix, vocab_prefix)
        .map_err(duckdb_fail)?;
    let mut rows: Vec<Vec<RdfTriple>> = Vec::new();
    for t in receipt.result_triples {
        match rows.last_mut() {
            Some(last) if last.last().map(|p| p.subject == t.subject).unwrap_or(false) => {
                last.push(t)
            }
            _ => rows.push(vec![t]),
        }
    }
    let rows_emitted = receipt.rows_emitted;
    let more = !rows.is_empty();
    let hex_hash = hex::encode(receipt.query_hash);
    // Same receipt-IRI mint as `rules::builtins::builtin_eval_sql`:
    // `{subject_prefix}/receipt_{query_hash[0..16]}` — byte-identity court.
    let receipt_iri = format!("{}/receipt_{}", subject_prefix, &hex_hash[0..16]);
    let id = cursor_id(&receipt.query_hash, rows.len());
    cursors()
        .lock()
        .expect("cursor registry lock")
        .insert(id.clone(), Cursor { rows, next: 0 });
    Ok(json!({
        "receipt": receipt_iri,
        "query_hash": hex_hash,
        "sql": receipt.sql_statement,
        "rows_emitted": rows_emitted,
        "cursor": id,
        "more": more,
    }))
}

/// `mode: "stream"`: emit the next page of the cursor as N-Triples lines.
fn stream(v: &Value) -> Result<Value, Fail> {
    let cursor = str_field(v, "cursor")?;
    let max_rows = match v.get("max_rows") {
        None | Some(Value::Null) => MAX_CHUNK_ROWS,
        Some(n) => {
            let n = n
                .as_u64()
                .ok_or_else(|| duckdb_fail("`max_rows` must be an unsigned integer"))?
                as usize;
            if n == 0 {
                return Err(duckdb_fail("`max_rows` must be at least 1"));
            }
            check_limit("duckdb_chunk_rows", n, MAX_CHUNK_ROWS)?;
            n
        }
    };
    let mut map = cursors().lock().expect("cursor registry lock");
    let Some(c) = map.get_mut(cursor) else {
        return Err(duckdb_fail(format!("unknown or expired cursor `{cursor}`")));
    };
    let page: Vec<&Vec<RdfTriple>> = c.rows[c.next.min(c.rows.len())..]
        .iter()
        .take(max_rows)
        .collect();
    let mut nt = String::new();
    for row in &page {
        for t in row.iter() {
            nt.push_str(&ntriples_line(t));
            nt.push('\n');
        }
    }
    if nt.len() > MAX_CHUNK_BYTES {
        // Typed refusal (GAP-4); the cursor is untouched — retry smaller.
        return Err(duckdb_limit("duckdb_chunk_bytes", nt.len(), MAX_CHUNK_BYTES));
    }
    c.next += page.len();
    let remaining = c.rows.len() - c.next;
    let more = remaining > 0;
    Ok(json!({
        "nquads": nt,
        "rows": page.len(),
        "cursor": if more { Some(cursor) } else { None },
        "more": more,
    }))
}

/// Render one RDF triple as an N-Triples line.
fn ntriples_line(t: &RdfTriple) -> String {
    let obj = match &t.object {
        purrdf::TermValue::Iri(i) => format!("<{i}>"),
        purrdf::TermValue::Blank { label, .. } => format!("_:{label}"),
        other => {
            let lit = term_json(other);
            let value = lit["value"].as_str().unwrap_or("");
            match lit.get("datatype").and_then(Value::as_str) {
                Some(dt) if dt.ends_with("#string") => format!("\"{}\"", escape_nt(value)),
                Some(dt) => format!("\"{}\"^^<{dt}>", escape_nt(value)),
                None => format!("\"{}\"", escape_nt(value)),
            }
        }
    };
    format!("<{}> <{}> {} .", t.subject, t.predicate, obj)
}

/// N-Triples string escaping (backslash and quote).
fn escape_nt(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"")
}

/// `mode: "close"`: drop a cursor; unknown cursors are idempotent-ok so a
/// host crash-recovery path cannot wedge on close.
fn close(v: &Value) -> Result<Value, Fail> {
    let cursor = str_field(v, "cursor")?;
    cursors()
        .lock()
        .expect("cursor registry lock")
        .remove(cursor);
    Ok(json!({}))
}

/// Unit tests for the op surface (native; the integration court is
/// tests/wasm_duckdb_op.rs).
#[cfg(test)]
mod tests {
    use serde_json::json;

    #[test]
    fn unknown_cursor_is_a_typed_refusal() {
        let out = crate::abi::call_json(&json!({
            "op": "duckdb", "mode": "stream", "cursor": "nope", "max_rows": 10
        }));
        assert_eq!(out["ok"], serde_json::Value::Bool(false));
        assert_eq!(out["error"]["kind"], "DuckDb");
    }

    #[test]
    fn close_of_unknown_cursor_is_ok() {
        let out = crate::abi::call_json(&json!({
            "op": "duckdb", "mode": "close", "cursor": "nope"
        }));
        assert_eq!(out["ok"], serde_json::Value::Bool(true));
    }
}
