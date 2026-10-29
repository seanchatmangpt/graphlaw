//! ABI op `duckdb`: cursor-streamed analytical eval over frozen-duckdb.
//! Op `duckdb` (op-15) dispatches inside the module on every target; the
//! gates that compiled that dispatch arm out of wasm32 are removed. It is
//! unadvertised from `capabilities.ops` (and stays off the hashed 14-op
//! table) by digest-stability choice — not because the op is host-only.
//!
//! Wire (JSON over the standard ABI):
//! - `exec`: `{"op":"duckdb","mode":"exec","views":[{name,glob,hive}],
//!   "query_ast":"<IRI>","ast":{"turtle","dialect"?},"subject_prefix",
//!   "vocab_prefix","params":[...]?,"chunk_rows"?,"max_rows"?}` —
//!   `ast.turtle` is the Turtle AST document, compiled via `compile_turtle`
//!   (SQLRDF-17: no raw SQL text admission; legacy `ast.text` is a typed
//!   refusal; wire `params` must one-for-one match the AST-carried literal
//!   parameters). Every statement class compile_turtle admits executes
//!   (CTQ-2: SELECT, INSERT, UPDATE, DELETE, CREATE, COPY, ATTACH, MERGE,
//!   ...); raw SQL text remains a typed SQLRDF-17 refusal and
//!   Connect/Disconnect remain REFUSAL-02 at compile time. ATTACH
//!   additionally passes the SQLRDF-22 G6 path allowlist: env
//!   `GRAPHLAW_ATTACH_ROOTS` (colon-separated absolute roots); env
//!   unset/empty refuses every ATTACH (fail-closed default).
//!   → `{"ok":true,"receipt","query_hash","rows_emitted","cursor","more":bool}`.
//!   WP P08 (the sieve becomes law): exec `max_rows` is the RESULT-ROW
//!   ENVELOPE the eval runs under (absent → `DEFAULT_ROW_ENVELOPE` = 64; `0`
//!   refused; no upper cap — the caller owns its envelope). A result set
//!   over the envelope is a typed `ResourceLimit` refusal with
//!   `limit: "duckdb_result_rows"` and the exact `observed`/`max`, fired
//!   mid-materialization strictly before any receipt. On `document:true`
//!   the same envelope applies per member and rides the existing
//!   `BatchMemberFailed` envelope, which then carries
//!   `limit`/`observed`/`max` fields. This exec `max_rows` is DISTINCT from
//!   the stream-mode `max_rows`, which is only a PAGE SIZE over an
//!   already-sieved cursor (default `MAX_CHUNK_ROWS`, capped at
//!   `MAX_CHUNK_ROWS`, `duckdb_chunk_rows` ResourceLimit above the cap) and
//!   never re-sieves.
//!   Without `"session"`, exec opens a fresh in-memory connection per call, so
//!   a DDL/DML effect is per-call ephemeral — admission and receipt minting
//!   are the contract, not durable mutation.
//! - `stream`: `{"op":"duckdb","mode":"stream","cursor","max_rows"?}` →
//!   `{"ok":true,"nquads","rows","cursor"|null,"more":bool}` — `max_rows`
//!   here is a page size (see the exec note above), not the result-row
//!   envelope.
//! - `open`: `{"op":"duckdb","mode":"open","session":"<name>"}` →
//!   `{"ok":true,"session":"<name>"}` — opens one named in-memory connection
//!   (same GAP-2 pragmas: memory_limit 32 MiB, threads=1), registered in a
//!   process-local session registry. Duplicate session name → typed refusal;
//!   registry cap (16) → typed ResourceLimit.
//! - `exec`/`stream` with `"session"`: when the optional `"session"` field is
//!   present, `exec` runs against that named session connection instead of a
//!   fresh one, so a CREATE in one call is visible to a later INSERT/SELECT on
//!   the same session (the DDL→DML→SELECT lifecycle). Unknown session id →
//!   typed refusal (mirrors unknown-cursor). When absent, the fresh-connection
//!   behavior is unchanged (back-compat).
//! - `close`: `{"op":"duckdb","mode":"close","cursor"?,"session"?}` — drops a
//!   cursor and/or a session; unknown handles are idempotent-ok.
//!
//! Session state is process-local and crash-only, like cursor state: it lives
//! in this module's registry, is bounded by the 16-session cap, and dies with
//! the process; closing a session frees its connection.
//!
//! DuckDB pragmas are enforced from compile-time constants (GAP-2): a caller
//! cannot override them from the wire. Errors are typed refusals (GAP-4) and
//! never trap. Cursor state is crash-only: it lives in this module's registry
//! and dies with the process/instance (documented ephemeral-cursor contract).

use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

use serde_json::{Value, json};

use crate::abi::{Fail, check_limit, fail_with_details, limit, str_field, term_json};
use crate::dialect::{Refusal, RefusalKind};
use crate::duckdb_eval::{
    DEFAULT_ROW_ENVELOPE, GraphLawDuckDB, RdfTriple, ROW_ENVELOPE_LIMIT,
};
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

/// Max live sessions (bounded process-local resource; over-cap is a typed
/// ResourceLimit refusal).
pub const MAX_SESSIONS: usize = 16;

/// Session registry, process-local (crash-only, bounded by [`MAX_SESSIONS`];
/// closing a session frees its connection).
fn sessions() -> &'static Mutex<HashMap<String, GraphLawDuckDB>> {
    static S: OnceLock<Mutex<HashMap<String, GraphLawDuckDB>>> = OnceLock::new();
    S.get_or_init(|| Mutex::new(HashMap::new()))
}

/// One open result cursor: row-major triples of the executed query.
struct Cursor {
    /// Triples grouped by source row (one `Vec` per row, in query order).
    rows: Vec<Vec<RdfTriple>>,
    /// Next row index to emit.
    next: usize,
}

/// Read a mandatory string field of the duckdb op envelope as a TYPED
/// `DuckDb` refusal (never the top-level `Unsupported` from `abi::str_field`):
/// the op dispatches in-module and validates its own envelope, so a missing
/// mandatory field is an op-surface refusal with the same message text
/// ("missing string field `k`") the courts pin.
fn duckdb_str_field<'a>(v: &'a Value, k: &str) -> Result<&'a str, Fail> {
    v.get(k)
        .and_then(Value::as_str)
        .ok_or_else(|| duckdb_fail(format!("missing string field `{k}`")))
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

/// WP P08: reads the exec-mode `max_rows` wire field — the RESULT-ROW
/// ENVELOPE (the sieve) this exec call evaluates under, distinct from the
/// stream-mode `max_rows` which is only a PAGE SIZE over an already-sieved
/// cursor. Absent (or null) → [`DEFAULT_ROW_ENVELOPE`]. Present → must be an
/// unsigned integer; `0` is refused (an envelope of zero rows is not a
/// lawful eval; use a non-SELECT statement class instead). There is no upper
/// cap: the caller owns its envelope.
fn max_rows_of(v: &Value) -> Result<usize, Fail> {
    match v.get("max_rows") {
        None | Some(Value::Null) => Ok(DEFAULT_ROW_ENVELOPE),
        Some(n) => {
            let n = n
                .as_u64()
                .ok_or_else(|| duckdb_fail("`max_rows` must be an unsigned integer"))?
                as usize;
            if n == 0 {
                return Err(duckdb_fail("`max_rows` must be at least 1"));
            }
            Ok(n)
        }
    }
}

/// WP P08: maps an [`EvaluatorError`] to the op-surface refusal. The sieve
/// breach is a typed ResourceLimit (`limit: "duckdb_result_rows"` with the
/// exact `observed`/`max`), never a generic `DuckDb` envelope; every other
/// evaluator failure keeps the existing `DuckDb` mapping byte-identical.
fn map_eval_error(err: crate::duckdb_eval::EvaluatorError) -> Fail {
    match err {
        crate::duckdb_eval::EvaluatorError::RowEnvelopeExceeded { observed, max, .. } => {
            duckdb_limit(ROW_ENVELOPE_LIMIT, observed, max)
        }
        other => duckdb_fail(other),
    }
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
                        )));
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

/// SQLRDF-22 G6 — ATTACH path admission policy. Env `GRAPHLAW_ATTACH_ROOTS`
/// (colon-separated absolute roots) is the only attach surface: a compiled
/// statement whose SQL lexically begins with `ATTACH` (the same prefix
/// classification as `duckdb_eval::mutating_sql`'s ATTACH class) may attach
/// only a database file that canonicalizes (best-effort
/// `std::fs::canonicalize`; failure to canonicalize is a refusal) under at
/// least one allowlisted root. The env unset or empty is fail-closed: every
/// ATTACH is refused. A `':memory:'` path never canonicalizes, so it is
/// refused under this policy too (fail-closed default; file-backed attach
/// inside a root is the admitted form). Violations are typed `DuckDb`
/// refusals, never engine errors. The env read is a thin wrapper over the
/// pure [`enforce_attach_policy_with`]; courts drive the pure seam because
/// the crate is `#![forbid(unsafe_code)]` and edition-2024 `set_var` is
/// unsafe.
fn enforce_attach_policy(sql: &str) -> Result<(), Fail> {
    let roots = std::env::var("GRAPHLAW_ATTACH_ROOTS").ok();
    enforce_attach_policy_with(sql, roots.as_deref())
}

/// Pure core of the ATTACH allowlist: `roots` is `None` when the env is
/// unset and `Some("")` when it is empty — both fail closed.
pub fn leading_sql_token(sql: &str) -> Option<String> {
    let b: Vec<char> = sql.chars().collect();
    let n = b.len();
    let mut i = 0;
    loop {
        while i < n && b[i].is_whitespace() {
            i += 1;
        }
        if i >= n {
            return None;
        }
        if b[i] == '-' && i + 1 < n && b[i + 1] == '-' {
            while i < n && b[i] != '\n' {
                i += 1;
            }
            continue;
        }
        // Block comment /* ... */, nesting-aware
        if b[i] == '/' && i + 1 < n && b[i + 1] == '*' {
            let mut depth = 1usize;
            i += 2;
            while i < n && depth > 0 {
                if b[i] == '/' && i + 1 < n && b[i + 1] == '*' {
                    depth += 1;
                    i += 2;
                } else if b[i] == '*' && i + 1 < n && b[i + 1] == '/' {
                    depth -= 1;
                    i += 2;
                } else {
                    i += 1;
                }
            }
            continue;
        }
        // A leading `'`/`"` literal is one opaque token — a `/*` inside it
        // is path text, not a comment — and can never be the ATTACH keyword.
        if b[i] == '\'' || b[i] == '"' {
            let q = b[i];
            let (inner, _) = consume_quoted(&b, i);
            return Some(format!("{q}{inner}{q}").to_ascii_uppercase());
        }
        // First real token char: ASCII-alpha run = leading keyword.
        if b[i].is_ascii_alphabetic() {
            let start = i;
            while i < n && b[i].is_ascii_alphabetic() {
                i += 1;
            }
            let token: String = b[start..i].iter().collect();
            return Some(token.to_ascii_uppercase());
        }
        return None;
    }
}

/// Count top-level `;` separators — quote- and comment-aware (the scanner's
/// state machine minus the token capture). A `;` inside a string literal or
/// comment does not separate statements.
fn top_level_semicolon_count(sql: &str) -> usize {
    let b: Vec<char> = sql.chars().collect();
    let n = b.len();
    let mut i = 0;
    let mut in_line_comment = false;
    let mut block_depth = 0usize;
    let mut in_single = false;
    let mut in_double = false;
    let mut count = 0usize;
    while i < n {
        let c = b[i];
        if in_line_comment {
            if c == '\n' { in_line_comment = false; }
            i += 1;
            continue;
        }
        if block_depth > 0 {
            if c == '/' && i + 1 < n && b[i + 1] == '*' { block_depth += 1; i += 2; continue; }
            if c == '*' && i + 1 < n && b[i + 1] == '/' { block_depth -= 1; i += 2; continue; }
            i += 1;
            continue;
        }
        if in_single {
            if c == '\'' {
                if i + 1 < n && b[i + 1] == '\'' { i += 2; continue; }
                in_single = false;
            }
            i += 1;
            continue;
        }
        if in_double {
            if c == '"' { in_double = false; }
            i += 1;
            continue;
        }
        if c == '-' && i + 1 < n && b[i + 1] == '-' { in_line_comment = true; i += 2; continue; }
        if c == '/' && i + 1 < n && b[i + 1] == '*' { block_depth += 1; i += 2; continue; }
        if c == '\'' { in_single = true; i += 1; continue; }
        if c == '"' { in_double = true; i += 1; continue; }
        if c == ';' { count += 1; }
        i += 1;
    }
    count
}

/// Consume the `'`/`"` quoted literal starting at `i`; returns the literal's
/// inner text (doubled-quote escapes folded to one quote — the same
/// single-quote rule as the COPY digest lane's `copy_to_path`) and the index
/// just past the closing quote. Unterminated literal: the text so far and
/// the end index (the engine syntax-errors; nothing admits on it).
fn consume_quoted(cs: &[char], i: usize) -> (String, usize) {
    let q = cs[i];
    let n = cs.len();
    let mut j = i + 1;
    let mut inner = String::new();
    while j < n {
        if cs[j] == q {
            if j + 1 < n && cs[j + 1] == q {
                inner.push(q); // `''` inside a literal renders one `'`
                j += 2;
            } else {
                j += 1;
                return (inner, j);
            }
        } else {
            inner.push(cs[j]);
            j += 1;
        }
    }
    (inner, j)
}

/// The ATTACH path candidate: the `'…'` string literal that follows the
/// leading keyword (trivia skipped both before and after the keyword) —
/// never the first quote character in raw text, which a decoy literal inside
/// the leading comment (`/* '/in-root/x' */ ATTACH '/etc/evil'`) would
/// otherwise smuggle past the allowlist while the engine attaches the real
/// argument. `None` when no literal follows the leading keyword (nothing to
/// validate: typed refusal, fail-closed).
fn first_string_literal(sql: &str) -> Option<String> {
    let cs: Vec<char> = sql.chars().collect();
    let mut i = skip_trivia(&cs, 0);
    // Skip the leading keyword token (this extractor only runs once the
    // policy classified the statement as ATTACH).
    while i < cs.len() && (cs[i].is_ascii_alphanumeric() || cs[i] == '_') {
        i += 1;
    }
    i = skip_trivia(&cs, i);
    if i < cs.len() && cs[i] == '\'' {
        Some(consume_quoted(&cs, i).0)
    } else {
        None
    }
}

/// Index of the first non-trivia character at/after `i`. Whitespace, `--`
/// line comments (consumed through the newline) and NESTING-AWARE `/* */`
/// block comments are trivia. Returns `cs.len()` at end of input, including
/// the unterminated-block-comment case (fail-closed: callers see no token
/// and the engine syntax-errors).
fn skip_trivia(cs: &[char], mut i: usize) -> usize {
    let n = cs.len();
    loop {
        if i >= n {
            return i;
        }
        let c = cs[i];
        if c.is_whitespace() {
            i += 1;
        } else if c == '-' && i + 1 < n && cs[i + 1] == '-' {
            // `--` line comment: consumed through the newline.
            i += 2;
            while i < n && cs[i] != '\n' {
                i += 1;
            }
            if i < n {
                i += 1;
            }
        } else if c == '/' && i + 1 < n && cs[i + 1] == '*' {
            // `/* */` block comment, nesting-aware depth counter.
            i += 2;
            let mut depth = 1usize;
            loop {
                if i >= n {
                    return i; // unterminated: EOF, fail-closed
                }
                if cs[i] == '/' && i + 1 < n && cs[i + 1] == '*' {
                    depth += 1;
                    i += 2;
                } else if cs[i] == '*' && i + 1 < n && cs[i + 1] == '/' {
                    depth -= 1;
                    i += 2;
                    if depth == 0 {
                        break;
                    }
                } else {
                    i += 1;
                }
            }
        } else {
            return i;
        }
    }
}

fn enforce_attach_policy_with(sql: &str, roots: Option<&str>) -> Result<(), Fail> {
    fn outside(path: &str) -> Fail {
        duckdb_fail(format!(
            "attach path outside GRAPHLAW_ATTACH_ROOTS allowlist: {path}"
        ))
    }
    if leading_sql_token(sql).as_deref() != Some("ATTACH") {
        return Ok(());
    }
    // Multi-statement guard (SQLRDF-22 G6 red-team v20-v22): frozen-duckdb's
    // prepare executes all INTERMEDIATE statements of a multi-statement
    // string before the final one reaches this policy. A benign-leading
    // `SELECT 1; ATTACH 'f' AS evil` would run the attach inside prepare,
    // past this allowlist. Fail closed: op-15 admits single statements only.
    if top_level_semicolon_count(sql) > 1 {
        return Err(duckdb_fail(
            "multi-statement SQL is not admitted through op-15; admit each statement separately",
        ));
    }
    // The validated path is the first `'…'` literal AFTER the leading
    // trivia (see [`first_string_literal`]) — never a decoy quote buried in
    // a comment. No literal at all is the `<none>` refusal (fail-closed;
    // the engine would syntax-error on it anyway).
    let raw = first_string_literal(sql).ok_or_else(|| outside("<none>"))?;
    let roots = roots.unwrap_or_default();
    if roots.is_empty() {
        return Err(outside(&raw));
    }
    // Canonicalize is best-effort and load-bearing: a path that does not
    // resolve (missing file, `:memory:`, relative junk) is refused.
    let canonical = std::fs::canonicalize(&raw).map_err(|_| outside(&raw))?;
    for root in roots.split(':') {
        if root.is_empty() || !root.starts_with('/') {
            continue;
        }
        if let Ok(croot) = std::fs::canonicalize(root) {
            // Path::starts_with is component-wise, so `/roots/a2b` cannot
            // satisfy the sibling-prefix `/roots/a`.
            if canonical.starts_with(&croot) {
                return Ok(());
            }
        }
    }
    Err(outside(&raw))
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
        "open" => open_session(v),
        "exec" => exec(v),
        "stream" => stream(v),
        "close" => close(v),
        other => Err(duckdb_fail(format!(
            "unknown mode `{other}` (open|exec|stream|close)"
        ))),
    }
}

/// `mode: "exec"`: mount views, bind/run the AST SQL, open a cursor.
/// When a `session` field is present, executes against the named persistent
/// session connection; otherwise a fresh in-memory connection (back-compat).
fn exec(v: &Value) -> Result<Value, Fail> {
    match v.get("session").and_then(Value::as_str) {
        Some(name) => {
            let map = sessions().lock().expect("session registry lock");
            let db = map
                .get(name)
                .ok_or_else(|| duckdb_fail(format!("unknown or expired session `{name}`")))?;
            exec_on(v, db)
        }
        None => {
            let db = open_connection()?;
            exec_on(v, &db)
        }
    }
}

fn exec_on(v: &Value, db: &GraphLawDuckDB) -> Result<Value, Fail> {
    if let Some(purrdf_dir) = v.get("purrdf_parquet_dir").and_then(Value::as_str) {
        let p = std::path::Path::new(purrdf_dir);
        db.register_purrdf_parquet_views(p).map_err(duckdb_fail)?;
    }
    if let Some(views) = v.get("views") {
        let arr = views
            .as_array()
            .ok_or_else(|| duckdb_fail("`views` must be an array"))?;
        check_limit("duckdb_views", arr.len(), 64)?;
        for view in arr {
            mount_view(&db, view)?;
        }
    }
    let query_ast = duckdb_str_field(v, "query_ast")?;
    let ast = v
        .get("ast")
        .filter(|a| a.is_object())
        .ok_or_else(|| duckdb_fail("missing `ast` object {\"turtle\": ...}"))?;
    // SQLRDF-17 (C2 hole P1): the op admits ONLY graph-backed statements —
    // every statement class compile_turtle admits (CTQ-2), never raw SQL
    // text. The legacy `ast.text` raw-SQL field is refused typed — raw SQL
    // text never executes and never mints a receipt. The caller supplies the
    // Turtle AST
    // document; it is compiled by the same `compile_turtle` law the native
    // builtin uses, so the receipt is BLAKE3(Canon(G_AST) || P_params)
    // (sql-rdf Rec. section 1.4) with canon pinned at compile time.
    if ast.get("text").is_some() {
        return Err(duckdb_fail(
            "SQLRDF-17 refused: legacy `ast.text` raw SQL is not an admitted admission path — supply the Turtle AST document as `ast.turtle` (compiled via compile_turtle)",
        ));
    }
    let turtle = duckdb_str_field(ast, "turtle")?;
    if let Some(d) = ast.get("dialect") {
        let d = d.as_str().unwrap_or("");
        if !d.is_empty() && !d.eq_ignore_ascii_case("duckdb") {
            return Err(duckdb_fail(format!("unsupported ast dialect `{d}`")));
        }
    }
    // SQLRDF-23 (op half): `"document":true` routes the whole Turtle AST
    // document through `compile_turtle_document` + `eval_ast_document` and
    // answers with the BatchReceipt wire shape instead of the single-statement
    // envelope. The legacy single-statement path below is untouched.
    if v.get("document").and_then(Value::as_bool).unwrap_or(false) {
        return exec_document_on(v, db, turtle);
    }
    let params = params_of(v)?;
    let mut compiled = crate::sql_ast::compile_turtle(turtle)
        .map_err(|e| duckdb_fail(format!("AST compile refusal: {e}")))?;
    // CTQ-2: no statement-class gate here beyond the SQLRDF-22 G6 ATTACH
    // path allowlist (below): anything compile_turtle admitted Anything compile_turtle admitted
    // (SELECT, INSERT, UPDATE, DELETE, CREATE, COPY, ATTACH, MERGE, ...)
    // executes; the cacheable bit only steers the C1 cache inside the
    // evaluator. Connect/Disconnect never reach this point (REFUSAL-02 at
    // compile_turtle); raw SQL text never reaches this point (SQLRDF-17
    // gate above). The per-call in-memory connection makes every DDL/DML
    // effect ephemeral by construction — the receipt, not durable mutation,
    // is the replayable artifact.
    // SQLRDF-22 G6: ATTACH statements additionally pass the env allowlist
    // (`GRAPHLAW_ATTACH_ROOTS`) before the engine sees them.
    enforce_attach_policy(&compiled.sql)?;
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
    let subject_prefix = duckdb_str_field(v, "subject_prefix")?;
    let vocab_prefix = duckdb_str_field(v, "vocab_prefix")?;
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
        .eval_ast_query_bounded(
            query_ast,
            &compiled,
            subject_prefix,
            vocab_prefix,
            max_rows_of(v)?,
        )
        .map_err(map_eval_error)?;
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

/// SQLRDF-23 (op half): `"document":true` exec path. Compiles the whole
/// Turtle AST document via [`crate::sql_ast::compile_turtle_document`] and
/// evaluates it member-by-member via [`GraphLawDuckDB::eval_ast_document`]
/// on the target connection (named session or fresh — the same routing as
/// the single-statement path). Each member passes the SQLRDF-22 G6 ATTACH
/// allowlist before the engine sees it. Success answers the BatchReceipt
/// wire shape: `{"query_iri","doc_hash","members":[...],"rows_emitted",
/// "failed_at":null}`. A mid-sequence member failure is the typed
/// `BatchMemberFailed` refusal envelope naming the failing member and
/// carrying the partial receipt (`members[0..index]` executed and committed)
/// — never a generic DB error and never a silent swallow.
fn exec_document_on(v: &Value, db: &GraphLawDuckDB, turtle: &str) -> Result<Value, Fail> {
    let query_iri = duckdb_str_field(v, "query_ast")?;
    let subject_prefix = duckdb_str_field(v, "subject_prefix")?;
    let vocab_prefix = duckdb_str_field(v, "vocab_prefix")?;
    let params = params_of(v)?;
    if !params.is_empty() {
        // SQLRDF-17 one-for-one law, document form: the Turtle AST document
        // is the admission record for EVERY member; wire-level params cannot
        // be bound one-for-one against a member ladder, so a non-empty wire
        // param list is a typed refusal (never a silent re-bind).
        return Err(duckdb_fail(
            "SQLRDF-17 refused: document exec does not bind wire `params` — each sequence member's literals are carried in the Turtle AST document itself",
        ));
    }
    let document = crate::sql_ast::compile_turtle_document(turtle)
        .map_err(|e| duckdb_fail(format!("AST compile refusal: {e}")))?;
    for member in &document.statements {
        enforce_attach_policy(&member.sql)?;
    }
    // SQLRDF-26: `atomic` is an optional wire flag on the document exec op.
    // Absent (or false) keeps the SQLRDF-23 pipeline path byte-identical;
    // `true` selects BEGIN/COMMIT/ROLLBACK execution inside
    // `eval_ast_document`. The Turtle root's `sql:atomic true` ORs in.
    let wire_atomic = match v.get("atomic") {
        None | Some(Value::Null) => false,
        Some(b) => b
            .as_bool()
            .ok_or_else(|| duckdb_fail("`atomic` must be a boolean when present"))?,
    };
    let max_rows = max_rows_of(v)?;
    match db.eval_ast_document_bounded(
        query_iri,
        &document,
        subject_prefix,
        vocab_prefix,
        wire_atomic || document.atomic,
        max_rows,
    ) {
        Ok(receipt) => {
            let members = serde_json::to_value(&receipt.members)
                .map_err(|e| duckdb_fail(format!("receipt serialization: {e}")))?;
            Ok(json!({
                "query_iri": receipt.query_iri,
                "doc_hash": hex::encode(receipt.doc_hash),
                "members": members,
                "rows_emitted": receipt.rows_emitted,
                "failed_at": receipt.failed_at,
                "status": receipt.status,
                "state_delta": receipt.state_delta,
            }))
        }
        Err(crate::duckdb_eval::EvaluatorError::BatchMemberFailed {
            index,
            source,
            partial,
        }) => {
            let partial_receipt = serde_json::to_value(&partial)
                .map_err(|e| duckdb_fail(format!("receipt serialization: {e}")))?;
            // WP P08: when the failing member breached the sieve, the
            // BatchMemberFailed envelope carries the limit fields too
            // (`limit`/`observed`/`max`) so a decoder sees WHY member
            // `index` failed without unwrapping `source`. The envelope
            // shape itself is unchanged (BatchMemberFailed stays
            // BatchMemberFailed).
            let sieve = match &*source {
                crate::duckdb_eval::EvaluatorError::RowEnvelopeExceeded {
                    observed, max, ..
                } => Some((observed, max)),
                _ => None,
            };
            let mut details = json!({
                "code": "BatchMemberFailed",
                "member": index,
                "message": format!("member {index} failed: {source}"),
                "partial_receipt": partial_receipt,
            });
            if let Some((&observed, &max)) = sieve {
                details["limit"] = json!(ROW_ENVELOPE_LIMIT);
                details["observed"] = json!(observed);
                details["max"] = json!(max);
            }
            Err(fail_with_details(
                Refusal {
                    kind: RefusalKind::DuckDb,
                    dialect: None,
                    engine: None,
                    message: format!(
                        "duckdb: member {index} of document {query_iri} failed: {source}"
                    )
                    .into(),
                },
                details,
            ))
        }
        Err(other) => Err(map_eval_error(other)),
    }
}

/// `mode: "stream"`: emit the next page of the cursor as N-Triples lines.
fn stream(v: &Value) -> Result<Value, Fail> {
    let cursor = duckdb_str_field(v, "cursor")?;
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
        return Err(duckdb_limit(
            "duckdb_chunk_bytes",
            nt.len(),
            MAX_CHUNK_BYTES,
        ));
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

/// `mode: "open"`: open a named persistent in-memory session connection with
/// the same enforced pragmas as every graphlaw connection (GAP-2). Duplicate
/// names are a typed refusal; the registry cap is a typed ResourceLimit.
fn open_session(v: &Value) -> Result<Value, Fail> {
    let name = str_field(v, "session")?;
    if name.is_empty() {
        return Err(duckdb_fail("`session` must be a non-empty string"));
    }
    let mut map = sessions().lock().expect("session registry lock");
    if map.contains_key(name) {
        return Err(duckdb_fail(format!(
            "session `{name}` already exists (close it before reopening)"
        )));
    }
    check_limit("duckdb_sessions", map.len() + 1, MAX_SESSIONS)?;
    map.insert(name.to_string(), open_connection()?);
    Ok(json!({"session": name}))
}

/// `mode: "close"`: drop a cursor and/or a session; unknown handles are
/// idempotent-ok so a host crash-recovery path cannot wedge on close.
fn close(v: &Value) -> Result<Value, Fail> {
    if let Some(cursor) = v.get("cursor").and_then(Value::as_str) {
        cursors()
            .lock()
            .expect("cursor registry lock")
            .remove(cursor);
    }
    if let Some(session) = v.get("session").and_then(Value::as_str) {
        sessions()
            .lock()
            .expect("session registry lock")
            .remove(session);
    }
    Ok(json!({}))
}

/// Unit tests for the op surface (native; the integration court is
/// tests/wasm_duckdb_op.rs).
#[cfg(test)]
mod tests {
    use serde_json::{Value, json};

    use super::{enforce_attach_policy_with, leading_sql_token};

    #[test]
    fn unknown_cursor_is_a_typed_refusal() {
        let out = crate::abi::call_json(&json!({
            "op": "duckdb", "mode": "stream", "cursor": "nope", "max_rows": 10
        }));
        assert_eq!(out["ok"], serde_json::Value::Bool(false));
        assert_eq!(out["error"]["kind"], "DuckDb");
    }

    /// CTQ-2: the graph-backed InsertStatement is admitted and executes
    /// against the real in-memory DuckDB. The per-call connection has no
    /// `telemetry_events` table, so the observable is the ENGINE-level
    /// error (table does not exist) — the SQLRDF-17/F2 admission refusal is
    /// gone and the statement reached execution. (Positive end-to-end insert
    /// with row readback is courted in src/duckdb_eval.rs on a seeded table.)
    #[test]
    fn insert_statement_executes_past_admission() {
        let turtle = r#"
@prefix sql: <http://graphlaw.org/sql#> .
@prefix ex: <http://example.org/queries#> .
ex:I a sql:InsertStatement ;
    sql:statementIndex 1 ;
    sql:targetTable "telemetry_events" ;
    sql:assignColumn "batch_id" ;
    sql:assignValue "x" .
"#;
        let out = crate::abi::call_json(&json!({
            "op": "duckdb",
            "mode": "exec",
            "query_ast": "urn:insert",
            "ast": {"turtle": turtle},
            "subject_prefix": "urn:row/",
            "vocab_prefix": "urn:p/",
        }));
        assert_eq!(out["ok"], serde_json::Value::Bool(false), "{out}");
        let kind = out["error"]["kind"].as_str().unwrap_or("");
        let message = out["error"]["message"].as_str().unwrap_or("");
        assert_eq!(
            kind, "DuckDb",
            "insert must reach the DuckDB engine, not the admission gate: {out}"
        );
        assert!(
            message.contains("telemetry_events"),
            "engine error must name the insert target: {out}"
        );
    }

    /// CTQ-2 DDL lifecycle: a graph-backed CREATE TABLE executes on the real
    /// in-memory DuckDB with a minted receipt and cursor (rows_emitted == 0).
    #[test]
    fn create_table_exec_executes_on_real_duckdb() {
        let turtle = r#"
@prefix sql: <http://graphlaw.org/sql#> .
@prefix ex: <http://example.org/queries#> .
ex:C a sql:CreateStatement ;
    sql:statementIndex 1 ;
    sql:targetTable "telemetry_events" ;
    sql:assignColumn "batch_id" .
"#;
        let out = crate::abi::call_json(&json!({
            "op": "duckdb",
            "mode": "exec",
            "query_ast": "urn:create",
            "ast": {"turtle": turtle},
            "subject_prefix": "urn:row/",
            "vocab_prefix": "urn:p/",
        }));
        assert_eq!(out["ok"], serde_json::Value::Bool(true), "{out}");
        assert_eq!(out["rows_emitted"], 0, "{out}");
        assert!(
            out["cursor"].as_str().is_some_and(|c| !c.is_empty()),
            "cursor minted: {out}"
        );
        assert_eq!(out["more"], serde_json::Value::Bool(false), "{out}");
        // Real readback: the CREATE actually ran on this connection — the
        // cursor's receipt SQL text is the CREATE statement.
        assert!(
            out["sql"].as_str().unwrap_or("").starts_with("CREATE TABLE telemetry_events"),
            "{out}"
        );
    }

    /// Hostile twin: hand-built raw SQL text (`ast.text`) stays refused even
    /// though every compile_turtle statement class is now admitted.
    #[test]
    fn raw_sql_text_still_refused() {
        let out = crate::abi::call_json(&json!({
            "op": "duckdb",
            "mode": "exec",
            "query_ast": "urn:evil",
            "ast": {"text": "SELECT 42; DROP TABLE telemetry_events"},
            "subject_prefix": "urn:row/",
            "vocab_prefix": "urn:p/",
        }));
        assert_eq!(out["ok"], serde_json::Value::Bool(false), "{out}");
        let message = out["error"]["message"].as_str().unwrap_or("");
        assert!(
            message.contains("SQLRDF-17"),
            "raw SQL text must stay a typed SQLRDF-17 refusal: {out}"
        );
    }

    #[test]
    fn close_of_unknown_cursor_is_ok() {
        let out = crate::abi::call_json(&json!({
            "op": "duckdb", "mode": "close", "cursor": "nope"
        }));
        assert_eq!(out["ok"], serde_json::Value::Bool(true));
    }

    /// SQLRDF-21 acceptance (b): the full CREATE → INSERT → SELECT lifecycle
    /// over one persistent named session. Without the session surface this
    /// court cannot pass — the fresh-connection exec makes each statement
    /// per-call ephemeral.
    #[test]
    fn session_lifecycle_create_insert_select() {
        let create = r#"
@prefix sql: <http://graphlaw.org/sql#> .
@prefix ex: <http://example.org/sessions#> .
ex:C a sql:CreateStatement ;
    sql:statementIndex 1 ;
    sql:targetTable "telemetry_events" ;
    sql:assignColumn "batch_id" .
"#;
        let insert = r#"
@prefix sql: <http://graphlaw.org/sql#> .
@prefix xsd: <http://www.w3.org/2001/XMLSchema#> .
@prefix ex: <http://example.org/sessions#> .
ex:I a sql:InsertStatement ;
    sql:statementIndex 1 ;
    sql:targetTable "telemetry_events" ;
    sql:assignColumn "batch_id" ;
    sql:assignValue "42"^^xsd:integer .
"#;
        let select = r#"
@prefix sql: <http://graphlaw.org/sql#> .
@prefix ex: <http://example.org/sessions#> .
ex:S a sql:SelectStatement ;
    sql:statementIndex 1 ;
    sql:fromTable [ sql:tableName "telemetry_events" ] ;
    sql:selectList ( [ sql:columnName "batch_id" ] ) .
"#;
        let exec = |turtle: &'static str, session: &str| {
            crate::abi::call_json(&json!({
                "op": "duckdb",
                "mode": "exec",
                "session": session,
                "query_ast": "urn:q",
                "ast": {"turtle": turtle},
                "subject_prefix": "urn:row/",
                "vocab_prefix": "urn:p/",
            }))
        };
        let open = crate::abi::call_json(&json!({
            "op": "duckdb", "mode": "open", "session": "lifecycle-court"
        }));
        assert_eq!(open["ok"], serde_json::Value::Bool(true), "{open}");
        assert_eq!(open["session"], "lifecycle-court", "{open}");

        let c = exec(create, "lifecycle-court");
        assert_eq!(c["ok"], serde_json::Value::Bool(true), "{c}");
        let i = exec(insert, "lifecycle-court");
        assert_eq!(i["ok"], serde_json::Value::Bool(true), "{i}");
        assert_eq!(i["rows_emitted"], 1, "{i}");

        let s = exec(select, "lifecycle-court");
        assert_eq!(s["ok"], serde_json::Value::Bool(true), "{s}");
        assert_eq!(s["rows_emitted"], 1, "{s}");
        let cur = s["cursor"].as_str().expect("select cursor");
        let page = crate::abi::call_json(&json!({
            "op": "duckdb", "mode": "stream", "cursor": cur, "max_rows": 10
        }));
        assert_eq!(page["ok"], serde_json::Value::Bool(true), "{page}");
        let nt = page["nquads"].as_str().expect("nquads");
        assert!(
            nt.contains("sess-row-1") || nt.contains("\"42\""),
            "the inserted value must read back exactly: {page}"
        );
    }

    /// Back-compat twin: identical ops WITHOUT a session keep the
    /// fresh-connection behavior — the CREATE's effect is per-call ephemeral,
    /// so the SELECT on a later call hits the engine's missing-table error
    /// (typed DuckDb), exactly as the legacy contract states.
    #[test]
    fn no_session_stays_fresh_connection() {
        let create = r#"
@prefix sql: <http://graphlaw.org/sql#> .
@prefix ex: <http://example.org/sessions#> .
ex:C a sql:CreateStatement ;
    sql:statementIndex 1 ;
    sql:targetTable "fresh_events" ;
    sql:assignColumn "batch_id" .
"#;
        let select = r#"
@prefix sql: <http://graphlaw.org/sql#> .
@prefix ex: <http://example.org/sessions#> .
ex:S a sql:SelectStatement ;
    sql:statementIndex 1 ;
    sql:fromTable [ sql:tableName "fresh_events" ] ;
    sql:selectList ( [ sql:columnName "batch_id" ] ) .
"#;
        let c = crate::abi::call_json(&json!({
            "op": "duckdb", "mode": "exec",
            "query_ast": "urn:c", "ast": {"turtle": create},
            "subject_prefix": "urn:row/", "vocab_prefix": "urn:p/",
        }));
        assert_eq!(c["ok"], serde_json::Value::Bool(true), "{c}");
        let s = crate::abi::call_json(&json!({
            "op": "duckdb", "mode": "exec",
            "query_ast": "urn:s", "ast": {"turtle": select},
            "subject_prefix": "urn:row/", "vocab_prefix": "urn:p/",
        }));
        assert_eq!(s["ok"], serde_json::Value::Bool(false), "{s}");
        assert_eq!(s["error"]["kind"], "DuckDb", "{s}");
        assert!(
            s["error"]["message"]
                .as_str()
                .unwrap_or("")
                .contains("fresh_events"),
            "engine error must name the missing table: {s}"
        );
    }

    #[test]
    fn unknown_session_is_a_typed_refusal() {
        let out = crate::abi::call_json(&json!({
            "op": "duckdb", "mode": "exec", "session": "nope",
            "query_ast": "urn:q",
            "ast": {"turtle": "@prefix sql: <http://graphlaw.org/sql#> . ex:q a sql:SelectStatement ."},
            "subject_prefix": "urn:row/", "vocab_prefix": "urn:p/",
        }));
        assert_eq!(out["ok"], serde_json::Value::Bool(false), "{out}");
        assert_eq!(out["error"]["kind"], "DuckDb", "{out}");
        assert!(
            out["error"]["message"]
                .as_str()
                .unwrap_or("")
                .contains("unknown or expired session"),
            "{out}"
        );
    }

    #[test]
    fn duplicate_session_name_is_a_typed_refusal() {
        let open = crate::abi::call_json(&json!({
            "op": "duckdb", "mode": "open", "session": "dup-court"
        }));
        assert_eq!(open["ok"], serde_json::Value::Bool(true), "{open}");
        let again = crate::abi::call_json(&json!({
            "op": "duckdb", "mode": "open", "session": "dup-court"
        }));
        assert_eq!(again["ok"], serde_json::Value::Bool(false), "{again}");
        assert!(
            again["error"]["message"]
                .as_str()
                .unwrap_or("")
                .contains("already exists"),
            "{again}"
        );
        let freed = crate::abi::call_json(&json!({
            "op": "duckdb", "mode": "close", "session": "dup-court"
        }));
        assert_eq!(freed["ok"], serde_json::Value::Bool(true), "{freed}");
    }

    /// Closing a session frees it: subsequent use is a typed refusal, and a
    /// fresh open of the same name succeeds (registry slot reclaimed).
    #[test]
    fn close_frees_session() {
        let open = crate::abi::call_json(&json!({
            "op": "duckdb", "mode": "open", "session": "frees-court"
        }));
        assert_eq!(open["ok"], serde_json::Value::Bool(true), "{open}");
        let closed = crate::abi::call_json(&json!({
            "op": "duckdb", "mode": "close", "session": "frees-court"
        }));
        assert_eq!(closed["ok"], serde_json::Value::Bool(true), "{closed}");
        let use_after = crate::abi::call_json(&json!({
            "op": "duckdb", "mode": "exec", "session": "frees-court",
            "query_ast": "urn:q",
            "ast": {"turtle": "@prefix sql: <http://graphlaw.org/sql#> . ex:q a sql:SelectStatement ."},
            "subject_prefix": "urn:row/", "vocab_prefix": "urn:p/",
        }));
        assert_eq!(use_after["ok"], serde_json::Value::Bool(false), "{use_after}");
        assert!(
            use_after["error"]["message"]
                .as_str()
                .unwrap_or("")
                .contains("unknown or expired session"),
            "{use_after}"
        );
        let reopen = crate::abi::call_json(&json!({
            "op": "duckdb", "mode": "open", "session": "frees-court"
        }));
        assert_eq!(reopen["ok"], serde_json::Value::Bool(true), "{reopen}");
        // Leave the registry as we found it (process-local hygiene).
        crate::abi::call_json(&json!({
            "op": "duckdb", "mode": "close", "session": "frees-court"
        }));
    }

    fn attach_turtle(path: &str) -> String {
        format!(
            r#"
@prefix sql: <http://graphlaw.org/sql#> .
@prefix ex: <http://example.org/attach#> .
ex:A a sql:AttachStatement ;
    sql:statementIndex 1 ;
    sql:attachTarget [ sql:databasePath "{path}" ; sql:attachAlias "g6root" ] .
"#
        )
    }

    fn attach_exec_with(path: &str, roots: Option<&str>) -> Result<(), crate::abi::Fail> {
        // Drive the pure policy seam directly (the crate is
        // `#![forbid(unsafe_code)]`, so edition-2024 `set_var` is out); the
        // seam is exactly the function `exec_on` calls after the env read,
        // so the courts cover the same admission surface.
        enforce_attach_policy_with(
            &crate::sql_ast::compile_turtle(&attach_turtle(path))
                .expect("ATTACH doc compiles")
                .sql,
            roots,
        )
    }

    fn attach_exec(path: &str, session: &str) -> serde_json::Value {
        crate::abi::call_json(&json!({
            "op": "duckdb", "mode": "exec", "session": session,
            "query_ast": "urn:attach",
            "ast": {"turtle": attach_turtle(path)},
            "subject_prefix": "urn:row/", "vocab_prefix": "urn:p/",
        }))
    }

    /// SQLRDF-22 G6 (a): ATTACH of a real file DB inside the allowlisted
    /// pid-keyed root is ADMITTED by the policy seam, and the compiled ATTACH
    /// then executes on a real DuckDB connection with the alias reading back
    /// through a real CREATE/INSERT/SELECT lifecycle. (The roots value is
    /// injected into the pure seam because the crate is
    /// `#![forbid(unsafe_code)]` and edition-2024 `set_var` is unsafe; the
    /// seam is byte-identical to the one `exec_on` calls after its safe env
    /// read, and court (c) proves the ABI exec path consults the policy.)
    #[test]
    fn attach_inside_root_executes_and_reads_back() {
        let root = format!("/tmp/graphlaw-g6-attach-{}", std::process::id());
        std::fs::create_dir_all(&root).unwrap();
        let db_path = format!("{root}/inside.duckdb");
        // The file must exist for canonicalize AND be a valid DuckDB
        // database: create it by actually opening a connection at the path
        // (real file DB, no mocks), then release the handle.
        drop(frozen_duckdb::Connection::open(&db_path).unwrap());
        let sql = crate::sql_ast::compile_turtle(&attach_turtle(&db_path))
            .expect("ATTACH doc compiles")
            .sql;
        enforce_attach_policy_with(&sql, Some(&root)).expect("inside-root ATTACH admits");
        // Real execution + alias readback on a real connection.
        let conn = frozen_duckdb::Connection::open_in_memory().unwrap();
        conn.execute_batch(&sql).unwrap();
        conn.execute_batch(
            "CREATE TABLE g6root.main.t (x INTEGER); INSERT INTO g6root.main.t VALUES (42);",
        )
        .unwrap();
        let n: String = conn
            .query_row(
                "SELECT count(*)::VARCHAR FROM g6root.main.t WHERE x = 42",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(n, "1", "the attached file DB reads back through the alias");
        let _ = std::fs::remove_dir_all(&root);
    }

    /// SQLRDF-22 G6 (b): a path outside the root is a typed refusal carrying
    /// the stable message and the raw path, and the op surface stays
    /// unpoisoned — a follow-up CREATE through the same ABI exec path
    /// executes on a real session connection.
    #[test]
    fn attach_outside_root_refused_connection_unpoisoned() {
        let root = format!("/tmp/graphlaw-g6-attach-out-{}", std::process::id());
        std::fs::create_dir_all(&root).unwrap();
        let outside = format!("/tmp/graphlaw-g6-outside-{}.duckdb", std::process::id());
        std::fs::File::create(&outside).unwrap();
        let err = attach_exec_with(&outside, Some(&root))
            .expect_err("outside-root ATTACH must be refused");
        let rendered = format!("{err:?}");
        assert!(
            rendered.contains("attach path outside GRAPHLAW_ATTACH_ROOTS allowlist")
                && rendered.contains(&outside),
            "{rendered}"
        );
        let _ = std::fs::remove_dir_all(&root);
        let _ = std::fs::remove_file(&outside);
        // The connection is unpoisoned: a follow-up CREATE via the ABI exec
        // path (named session, real DuckDB) succeeds after the refusal.
        let open = crate::abi::call_json(&json!({
            "op": "duckdb", "mode": "open", "session": "g6-unpoisoned"
        }));
        assert_eq!(open["ok"], serde_json::Value::Bool(true), "{open}");
        let create = crate::abi::call_json(&json!({
            "op": "duckdb", "mode": "exec", "session": "g6-unpoisoned",
            "query_ast": "urn:c",
            "ast": {"turtle": r#"
@prefix sql: <http://graphlaw.org/sql#> .
@prefix ex: <http://example.org/attach#> .
ex:C a sql:CreateStatement ;
    sql:statementIndex 1 ;
    sql:targetTable "g6_unpoisoned" ;
    sql:assignColumn "x" .
"#},
            "subject_prefix": "urn:row/", "vocab_prefix": "urn:p/",
        }));
        assert_eq!(create["ok"], serde_json::Value::Bool(true), "{create}");
        crate::abi::call_json(&json!({
            "op": "duckdb", "mode": "close", "session": "g6-unpoisoned"
        }));
    }

    /// SQLRDF-22 G6 (c): roots unset (`None`) or empty is fail-closed —
    /// every ATTACH is a typed refusal — and the full ABI exec path enforces
    /// it: the real env read in `exec_on` feeds the same seam, so with the
    /// env unset (test process default) the wire ATTACH refuses typed.
    #[test]
    fn attach_env_unset_refused() {
        assert!(std::env::var("GRAPHLAW_ATTACH_ROOTS").is_err(),
            "court assumes the env is unset in the test process");
        let open = crate::abi::call_json(&json!({
            "op": "duckdb", "mode": "open", "session": "g6-unset"
        }));
        assert_eq!(open["ok"], serde_json::Value::Bool(true), "{open}");
        let out = attach_exec("/tmp/graphlaw-g6-nowhere.duckdb", "g6-unset");
        assert_eq!(out["ok"], serde_json::Value::Bool(false), "{out}");
        assert_eq!(out["error"]["kind"], "DuckDb", "{out}");
        assert!(
            out["error"]["message"]
                .as_str()
                .unwrap_or("")
                .contains("GRAPHLAW_ATTACH_ROOTS"),
            "{out}"
        );
        crate::abi::call_json(&json!({
            "op": "duckdb", "mode": "close", "session": "g6-unset"
        }));
        // Empty-string roots fail closed identically at the pure seam.
        assert!(attach_exec_with("/tmp/graphlaw-g6-nowhere.duckdb", Some("")).is_err());
    }

    // -----------------------------------------------------------------------
    // E1 re-land unit courts: the ATTACH policy classifies the leading token
    // with the quote-aware, nesting-aware `leading_sql_token` scanner and
    // validates the first string literal AFTER the trivia. The literal attack
    // strings here are the corpus that tests/attach_bypass_court.rs
    // liveness-courts against the real engine.
    // -----------------------------------------------------------------------

    /// Shared envelope assert: the camouflaged ATTACH is the typed allowlist
    /// refusal carrying the raw path verbatim, never an engine error.
    fn assert_camo_refusal(sql: &str, ctx: &str) {
        let err = enforce_attach_policy_with(sql, None)
            .expect_err(&format!("{ctx}: camouflaged ATTACH must hit the allowlist policy"));
        let rendered = format!("{err:?}");
        assert!(
            rendered.contains("attach path outside GRAPHLAW_ATTACH_ROOTS allowlist"),
            "{ctx}: stable allowlist message: {rendered}"
        );
        assert!(
            rendered.contains("/etc/passwd"),
            "{ctx}: raw attack path carried verbatim: {rendered}"
        );
    }

    /// `/* harmless */ ATTACH …` must hit the policy (the pre-fix
    /// classification returned an empty leading token and skipped it while
    /// the engine executed the attach).
    #[test]
    fn attach_behind_block_comment_hits_policy() {
        assert_camo_refusal(
            "/* harmless */ ATTACH '/etc/passwd' AS camo_block",
            "block comment",
        );
    }

    /// `-- note\nATTACH …` must hit the policy.
    #[test]
    fn attach_behind_line_comment_hits_policy() {
        assert_camo_refusal("-- note\nATTACH '/etc/passwd' AS camo_line", "line comment");
    }

    /// `/* /* */ */ ATTACH …` must hit the policy: the scanner consumes the
    /// nested comment with a depth counter, exactly as the engine's lexer
    /// does (a first-`*/` scan would mis-close it).
    #[test]
    fn attach_behind_nested_block_comment_hits_policy() {
        assert_camo_refusal(
            "/* /* */ */ ATTACH '/etc/passwd' AS camo_nested",
            "nested block comment",
        );
    }

    /// Leading whitespace (spaces, tabs, CRLF) must not hide the ATTACH.
    #[test]
    fn attach_behind_whitespace_hits_policy() {
        assert_camo_refusal(
            " \t \r\n ATTACH '/etc/passwd' AS camo_ws",
            "leading whitespace",
        );
    }

    /// Positive twin at the pure seam: a plain single-statement ATTACH of a
    /// real DuckDB file inside an allowlisted root ADMITS (real file, real
    /// canonicalize — no mocks).
    #[test]
    fn attach_plain_inside_root_admits() {
        let root = format!("/tmp/graphlaw-e1-attach-root-{}", std::process::id());
        std::fs::create_dir_all(&root).unwrap();
        let db_path = format!("{root}/inside.duckdb");
        drop(frozen_duckdb::Connection::open(&db_path).unwrap());
        let sql = format!("ATTACH '{db_path}' AS e1root");
        enforce_attach_policy_with(&sql, Some(&root)).expect("plain inside-root ATTACH admits");
        let _ = std::fs::remove_dir_all(&root);
    }

    /// A comment-camouflaged NON-attach statement stays outside the policy:
    /// the scanner sees SELECT and the allowlist is not consulted.
    #[test]
    fn select_behind_block_comment_stays_allowed() {
        enforce_attach_policy_with("/* x */ SELECT 1", None)
            .expect("non-ATTACH behind a block comment stays allowed");
    }

    /// `/*` inside the path literal is path text, not a comment: the refusal
    /// carries the verbatim glob path (a comment-blind lexer would truncate
    /// or mangle it at the `/*`).
    #[test]
    fn attach_path_glob_literal_is_path_text() {
        let err = enforce_attach_policy_with(
            "/* glob */ ATTACH '/tmp/graphlaw-e1/*.duckdb' AS globby",
            None,
        )
        .expect_err("fail-closed refusal for the unresolvable glob path");
        assert!(
            format!("{err:?}").contains("/tmp/graphlaw-e1/*.duckdb"),
            "glob path carried verbatim as literal text: {err:?}"
        );
    }

    /// Exact-token pins at the scanner and at the policy: `ATTACHER` is a
    /// different token from `ATTACH` (the old take_while-alphabetic prefix
    /// classified it as ATTACH); an unterminated block comment yields None
    /// (the engine syntax-errors — the safe, fail-closed outcome); a quote
    /// terminates the word run so `ATTACH'db'` still classifies as ATTACH;
    /// `/*!*/` is a closed block comment; a leading literal is one opaque
    /// token (its `/*` is not a comment).
    #[test]
    fn leading_sql_token_exact_tokens() {
        assert_eq!(
            leading_sql_token("ATTACH '/tmp/x' AS t").as_deref(),
            Some("ATTACH")
        );
        assert_eq!(
            leading_sql_token("ATTACHER '/tmp/x' AS t").as_deref(),
            Some("ATTACHER")
        );
        assert_ne!(
            leading_sql_token("ATTACHER '/tmp/x' AS t").as_deref(),
            Some("ATTACH")
        );
        // ATTACHER never reaches the allowlist: it is not an ATTACH statement.
        assert!(enforce_attach_policy_with("ATTACHER '/etc/passwd' AS t", None).is_ok());
        // Unterminated block comments (flat and nested) are None.
        assert_eq!(leading_sql_token("/* unterminated"), None);
        assert_eq!(leading_sql_token("/* /* nested */"), None);
        assert_eq!(leading_sql_token(""), None);
        // A quote terminates the word run.
        assert_eq!(
            leading_sql_token("ATTACH'/tmp/x' AS t").as_deref(),
            Some("ATTACH")
        );
        // The bang comment is a closed block comment.
        assert_eq!(
            leading_sql_token("/*!*/ ATTACH '/tmp/x' AS t").as_deref(),
            Some("ATTACH")
        );
        // A leading literal is opaque token text: its /* is not a comment.
        assert_eq!(leading_sql_token("'/*'").as_deref(), Some("'/*'"));
    }

    /// The policy validates the first literal AFTER the trivia, not the first
    /// quote character in raw text: a decoy in-root path buried in the
    /// leading comment must not admit the real outside-root path the engine
    /// will attach. (This is the discriminator for the
    /// `first_string_literal` extractor: the raw first-quote scan it replaced
    /// would extract the decoy and ADMIT.)
    #[test]
    fn attach_path_decoy_in_leading_comment_refuses_real_path() {
        let root = format!("/tmp/graphlaw-e1-decoy-root-{}", std::process::id());
        std::fs::create_dir_all(&root).unwrap();
        let inside = format!("{root}/inside.duckdb");
        drop(frozen_duckdb::Connection::open(&inside).unwrap());
        let block = format!("/* '{inside}' */ ATTACH '/etc/passwd' AS decoy");
        let err = enforce_attach_policy_with(&block, Some(&root)).expect_err(
            "decoy in-root literal in the comment must not admit the outside-root path",
        );
        assert!(
            format!("{err:?}").contains("/etc/passwd"),
            "the REAL argument is what got validated and refused: {err:?}"
        );
        let line = format!("-- '{inside}'\nATTACH '/etc/passwd' AS decoy2");
        assert!(
            enforce_attach_policy_with(&line, Some(&root)).is_err(),
            "line-comment decoy variant refuses too"
        );
        let _ = std::fs::remove_dir_all(&root);
    }

    // -----------------------------------------------------------------------
    // SQLRDF-23 (op half, lane G6) courts: `"document":true` exec over real
    // frozen-duckdb via compile_turtle_document + eval_ast_document. Chicago:
    // real connections, real table readbacks, no mocks, no sleeps.
    // -----------------------------------------------------------------------

    fn doc_events_turtle(select_table: &str) -> String {
        format!(
            r#"
@prefix sql: <http://graphlaw.org/sql#> .
@prefix xsd: <http://www.w3.org/2001/XMLSchema#> .
@prefix ex: <http://example.org/doc#> .
sql:doc a sql:StatementSequence ;
    sql:statements ( ex:c ex:i ex:s ) .
ex:c a sql:CreateStatement ;
    sql:statementIndex 0 ;
    sql:targetTable "doc_events" ;
    sql:assignColumn "batch_id" .
ex:i a sql:InsertStatement ;
    sql:statementIndex 1 ;
    sql:targetTable "doc_events" ;
    sql:assignColumn "batch_id" ;
    sql:assignValue "42"^^xsd:integer .
ex:s a sql:SelectStatement ;
    sql:statementIndex 2 ;
    sql:fromTable [ sql:tableName "{select_table}" ] ;
    sql:selectList ( [ sql:columnName "batch_id" ] ) .
"#
        )
    }

    fn doc_exec_request(turtle: &str) -> Value {
        json!({
            "op": "duckdb",
            "mode": "exec",
            "document": true,
            "query_ast": "urn:doc-court",
            "ast": {"turtle": turtle},
            "subject_prefix": "urn:row/",
            "vocab_prefix": "urn:p/",
        })
    }

    /// Court (a): 3-member CREATE→INSERT→SELECT sequence through call_json
    /// answers ok:true with the BatchReceipt shape (rows_emitted [0,1,1]) and
    /// the final member's SELECT actually emitted the inserted row — the
    /// members' result triples name the real row on a real connection.
    #[test]
    fn document_exec_three_member_sequence_with_readback() {
        let out = crate::abi::call_json(&doc_exec_request(&doc_events_turtle("doc_events")));
        assert_eq!(out["ok"], serde_json::Value::Bool(true), "{out}");
        assert_eq!(
            out["query_iri"], "urn:doc-court",
            "{out}"
        );
        assert_eq!(
            out["rows_emitted"],
            serde_json::json!([0, 1, 1]),
            "CREATE=0, INSERT=1, SELECT=1: {out}"
        );
        assert_eq!(out["failed_at"], serde_json::Value::Null, "{out}");
        assert_eq!(
            out["doc_hash"].as_str().map(str::len),
            Some(64),
            "hex BLAKE3 doc_hash: {out}"
        );
        // Real readback: member 2 (the SELECT) materialized the row 42 that
        // member 1 inserted on the same fresh connection.
        let member = &out["members"][2];
        assert!(
            member["sql_statement"]
                .as_str()
                .unwrap_or("")
                .contains("doc_events"),
            "{out}"
        );
        assert_eq!(
            member["rows_emitted"], 1,
            "SELECT emitted the inserted row: {out}"
        );
        let triples = member["result_triples"]
            .as_array()
            .expect("result triples");
        assert!(
            triples
                .iter()
                .any(|t| t["object"]["Literal"]["lexical_form"] == "42"),
            "the inserted batch_id 42 crossed the SELECT: {triples:?}"
        );
    }

    /// Court (b): member 2 (the SELECT, reading a missing table) fails
    /// mid-sequence → typed BatchMemberFailed envelope naming "member 2",
    /// carrying the partial receipt with members [0..2] executed and
    /// failed_at = 2.
    #[test]
    fn document_exec_failing_member_yields_typed_batch_member_failed() {
        let out = crate::abi::call_json(&doc_exec_request(&doc_events_turtle("no_such_table")));
        assert_eq!(out["ok"], serde_json::Value::Bool(false), "{out}");
        assert_eq!(
            out["error"]["kind"], "DuckDb",
            "typed refusal, never a generic trap: {out}"
        );
        let details = &out["error"]["details"];
        assert_eq!(details["code"], "BatchMemberFailed", "{out}");
        assert_eq!(details["member"], 2, "{out}");
        let message = details["message"].as_str().unwrap_or("");
        assert!(
            message.contains("member 2"),
            "envelope must name the failing member: {out}"
        );
        // Partial receipt: members 0 (CREATE) and 1 (INSERT) executed and
        // committed; only the SELECT failed.
        let partial = &details["partial_receipt"];
        assert_eq!(partial["failed_at"], 2, "{out}");
        assert_eq!(partial["rows_emitted"], serde_json::json!([0, 1]), "{out}");
        assert_eq!(
            partial["members"].as_array().map(Vec::len),
            Some(2),
            "members [0..2] executed: {out}"
        );
        assert_eq!(
            partial["members"][0]["sql_statement"]
                .as_str()
                .unwrap_or("")
                .starts_with("CREATE TABLE doc_events"),
            true,
            "{out}"
        );
    }

    /// SQLRDF-26: wire-level `"atomic": true` — success envelope carries
    /// `status`/`state_delta`; a mid-sequence failure carries a partial
    /// receipt marked `status:"aborted"`, `state_delta:0` (atomic rollback),
    /// and the wire flag composes with the Turtle `sql:atomic` root flag.
    #[test]
    fn document_exec_atomic_wire_flag_aborted_partial_and_success_fields() {
        // Success: atomic true, 3-member CREATE/INSERT/SELECT.
        let mut req = doc_exec_request(&doc_events_turtle("doc_events"));
        req["atomic"] = serde_json::Value::Bool(true);
        let out = crate::abi::call_json(&req);
        assert_eq!(out["ok"], serde_json::Value::Bool(true), "{out}");
        assert_eq!(out["status"], "partial", "{out}");
        assert_eq!(out["state_delta"], 3, "{out}");

        // Failure twin: SELECT member reads a missing table inside atomic
        // mode -> partial_receipt marks aborted with state_delta 0. The
        // Turtle root also carries sql:atomic true — the OR of the wire and
        // compile flags is exercised on the same op surface.
        let ttl = doc_events_turtle("no_such_table")
            .replacen("sql:doc a sql:StatementSequence ;", "sql:doc a sql:StatementSequence ;\n    sql:atomic true ;", 1);
        let mut req2 = doc_exec_request(&ttl);
        req2["atomic"] = serde_json::Value::Bool(true);
        let out = crate::abi::call_json(&req2);
        assert_eq!(out["ok"], serde_json::Value::Bool(false), "{out}");
        let details = &out["error"]["details"];
        assert_eq!(details["code"], "BatchMemberFailed", "{out}");
        let partial = &details["partial_receipt"];
        assert_eq!(partial["status"], "aborted", "{out}");
        assert_eq!(partial["state_delta"], 0, "{out}");
    }

    /// SQLRDF-26: pipeline mode stays byte-identical when `atomic` is
    /// absent — the success envelope must NOT carry the SQLRDF-23 status
    /// change, only the additive fields (status "partial", state_delta =
    /// member count).
    #[test]
    fn document_exec_pipeline_absent_atomic_additive_only() {
        let out = crate::abi::call_json(&doc_exec_request(&doc_events_turtle("doc_events")));
        assert_eq!(out["ok"], serde_json::Value::Bool(true), "{out}");
        assert_eq!(out["status"], "partial", "{out}");
        assert_eq!(out["state_delta"], 3, "{out}");
        assert_eq!(out["failed_at"], serde_json::Value::Null, "{out}");
    }

    /// Court (c): the legacy single-statement exec request is untouched —
    /// no `document` flag, same wire shape, real cursor + receipt IRI.
    #[test]
    fn legacy_exec_without_document_flag_unchanged() {
        let turtle = r#"
@prefix sql: <http://graphlaw.org/sql#> .
@prefix ex: <http://example.org/legacy#> .
ex:C a sql:CreateStatement ;
    sql:statementIndex 1 ;
    sql:targetTable "legacy_events" ;
    sql:assignColumn "batch_id" .
"#;
        let out = crate::abi::call_json(&json!({
            "op": "duckdb",
            "mode": "exec",
            "query_ast": "urn:legacy",
            "ast": {"turtle": turtle},
            "subject_prefix": "urn:row/",
            "vocab_prefix": "urn:p/",
        }));
        assert_eq!(out["ok"], serde_json::Value::Bool(true), "{out}");
        assert_eq!(out["rows_emitted"], 0, "{out}");
        assert!(out["cursor"].as_str().is_some(), "legacy cursor minted: {out}");
        assert!(out["receipt"].as_str().unwrap_or("").starts_with("urn:row//receipt_"), "{out}");
    }
}
