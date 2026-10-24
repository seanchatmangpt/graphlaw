//! SQLRDF-22 G6 — ATTACH allowlist bypass court (`tests/attach_bypass_court.rs`).
//!
//! The bypass: `enforce_attach_policy_with` (src/duckdb_op.rs) classified a
//! statement by trimming leading whitespace only, so a comment-camouflaged
//! ATTACH — `/* harmless */ ATTACH '/etc/passwd' AS x`, `-- note\nATTACH …`,
//! nested `/* /* */ */ ATTACH …`, `/*!*/ ATTACH …` — classified as non-ATTACH
//! (empty leading token) and the allowlist was skipped while the DuckDB lexer
//! happily executes all of them as ATTACH. The landed fix is the comment-aware
//! [`graphlaw` internal] `leading_sql_token` scanner (whitespace, `--` and
//! nesting-aware `/* */` comments skipped, quote-aware stop).
//!
//! What this court owns vs. what the src lane owns:
//! - The policy seam `enforce_attach_policy_with` is private to the crate, so
//!   the literal attack strings above are asserted against the seam by the
//!   unit courts inside `src/duckdb_op.rs::tests`
//!   (`attach_behind_block_comment_hits_policy`,
//!   `attach_behind_line_comment_hits_policy`,
//!   `attach_behind_nested_block_comment_hits_policy`,
//!   `attach_behind_whitespace_hits_policy`). This court pins the PUBLIC wire:
//!   every ATTACH that reaches the policy through `graphlaw::abi::call_json`
//!   with a path outside the allowlist (or with the env unset — fail-closed)
//!   must answer the stable typed refusal
//!   `attach path outside GRAPHLAW_ATTACH_ROOTS allowlist: <path>` with error
//!   kind `DuckDb`, and the admitted form must actually execute.
//! - Chicago: real frozen-duckdb connections, real file databases, real table
//!   and catalog readbacks. No mocks, no sleeps.
//!
//! Refusal register: the refusal is a typed `DuckDb` refusal on the op surface
//! (docs/refusals.md); it is never a generic engine error.

use serde_json::{json, Value};
use std::sync::Mutex;

/// Stable, load-bearing refusal text (src/duckdb_op.rs `enforce_attach_policy_with`).
const STABLE_REFUSAL: &str = "attach path outside GRAPHLAW_ATTACH_ROOTS allowlist";

/// The attack path named by the task's threat model. It is outside any
/// allowlisted root under BOTH env states (unset: fail-closed; set: not under
/// a root), so every refusal assertion below is valid regardless of which of
/// this file's tests currently owns the env window.
const ATTACK_PATH: &str = "/etc/passwd";

/// Process-local serialization of the env window. `GRAPHLAW_ATTACH_ROOTS` is
/// read inside the crate on every wire ATTACH; this file has exactly one test
/// that mutates the env, and every test takes this lock around wire ATTACH
/// calls, so `set_var` never races a concurrent `env::var` read inside the
/// policy (edition-2024 `set_var` is unsafe; see the SAFETY comment at the
/// mutation site). Mutex poisoning is tolerated: a panicked holder must not
/// cascade-red the sibling tests.
static ATTACH_WIRE: Mutex<()> = Mutex::new(());

/// Compile the court's AttachStatement Turtle document (the same admission
/// shape the GWS-4 court uses) for `path`/`alias`.
fn attach_turtle(path: &str, alias: &str) -> String {
    format!(
        r#"@prefix sql: <http://graphlaw.org/sql#> .
@prefix ex: <http://example.org/attach#> .
ex:A a sql:AttachStatement ;
    sql:statementIndex 1 ;
    sql:attachTarget [ sql:databasePath "{path}" ; sql:attachAlias "{alias}" ] .
"#
    )
}

/// Wire exec of one ATTACH document through the real ABI.
/// When `session` is Some, dispatches against that persistent session;
/// when None, dispatches against a fresh in-memory connection.
fn exec_attach(session: Option<&str>, path: &str, alias: &str) -> Value {
    let mut req = json!({
        "op": "duckdb",
        "mode": "exec",
        "query_ast": "http://example.org/attach#q",
        "ast": {"turtle": attach_turtle(path, alias)},
        "subject_prefix": "urn:attach-court",
        "vocab_prefix": "urn:attach-court/v",
    });
    if let Some(s) = session {
        req["session"] = json!(s);
    }
    graphlaw::abi::call_json(&req)
}

/// Assert the typed attach-policy refusal envelope: `ok:false`, error kind
/// `DuckDb` (never a generic engine error), the stable allowlist message, and
/// the raw attack path carried verbatim.
fn assert_typed_attach_refusal(out: &Value, raw_path: &str, ctx: &str) {
    assert_eq!(
        out["ok"],
        serde_json::Value::Bool(false),
        "{ctx}: attack must refuse, not execute: {out}"
    );
    assert_eq!(
        out["error"]["kind"], "DuckDb",
        "{ctx}: refusal must be the typed DuckDb refusal: {out}"
    );
    let message = out["error"]["message"]
        .as_str()
        .unwrap_or_else(|| panic!("{ctx}: refusal envelope carries a message string: {out}"));
    assert!(
        message.contains(STABLE_REFUSAL),
        "{ctx}: refusal must carry the stable allowlist message: {out}"
    );
    assert!(
        message.contains(raw_path),
        "{ctx}: refusal must carry the raw path verbatim: {out}"
    );
}

/// The comment-camouflage attack strings (the task's corpus, path-substituted
/// with the real outside-root database file). Each is engine-live: the DuckDB
/// lexer executes it as an ATTACH. The typed refusal each string must produce
/// is asserted at the policy seam by src/duckdb_op.rs::tests (see the module
/// doc); this list drives the liveness legs so the seam classification is
/// provably load-bearing and never vacuous.
fn camouflage_corpus(db_path: &str) -> Vec<(&'static str, String)> {
    vec![
        (
            "block_comment",
            format!("/* harmless */ ATTACH '{db_path}' AS camo_block"),
        ),
        (
            "line_comment",
            format!("-- note\nATTACH '{db_path}' AS camo_line"),
        ),
        (
            "nested_block_comment",
            format!("/* /* */ */ ATTACH '{db_path}' AS camo_nested"),
        ),
        (
            "bang_comment",
            format!("/*!*/ ATTACH '{db_path}' AS camo_bang"),
        ),
        (
            "leading_tab",
            format!("\tATTACH '{db_path}' AS camo_tab"),
        ),
        (
            "leading_whitespace",
            format!(" \t \r\n ATTACH '{db_path}' AS camo_ws"),
        ),
    ]
}

/// Wire court, fail-closed leg + positive twin. This is the only test in the
/// file that mutates `GRAPHLAW_ATTACH_ROOTS`; the ATTACH_WIRE lock serializes
/// its env window against every other test's wire ATTACH reads.
#[test]
fn wire_fail_closed_refusal_then_inside_root_attach_executes() {
    let _guard = ATTACH_WIRE
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());

    // (1) Typed precondition: the fail-closed leg requires the env unset in
    // this process (same precondition the G6(c) and GWS-4 courts assume).
    assert!(
        std::env::var("GRAPHLAW_ATTACH_ROOTS").is_err(),
        "court assumes GRAPHLAW_ATTACH_ROOTS is unset (fail-closed default)"
    );

    // (2) Fail-closed attack: `/etc/passwd` ATTACH over the wire is the typed
    // refusal — kind DuckDb, stable message, raw path — never an engine error.
    let out = exec_attach(None, ATTACK_PATH, "att_fc");
    assert_typed_attach_refusal(&out, ATTACK_PATH, "fail-closed env-unset attach");

    // (3) The surface is unpoisoned: a benign CREATE through the same dispatch
    // path still executes after the refusal.
    let create = graphlaw::abi::call_json(&json!({
        "op": "duckdb",
        "mode": "exec",
        "query_ast": "http://example.org/attach#create",
        "ast": {"turtle": r#"@prefix sql: <http://graphlaw.org/sql#> .
@prefix ex: <http://example.org/attach#> .
ex:C a sql:CreateStatement ;
    sql:statementIndex 1 ;
    sql:targetTable "attach_court_unpoisoned" ;
    sql:assignColumn "x" .
"#},
        "subject_prefix": "urn:attach-court",
        "vocab_prefix": "urn:attach-court/v",
    }));
    assert_eq!(
        create["ok"],
        serde_json::Value::Bool(true),
        "surface unpoisoned after the typed refusal: {create}"
    );

    // (4) Positive twin setup: a real allowlisted temp-dir root holding a REAL
    // DuckDB database file (canonicalize must resolve; no fakes).
    let root = std::env::temp_dir().join(format!("attach-court-root-{}", std::process::id()));
    std::fs::create_dir_all(&root).expect("create allowlist root");
    let db_path = root.join("inside.duckdb");
    drop(frozen_duckdb::Connection::open(&db_path).expect("mint real inside-root duckdb file"));

    // (5) The env window (see SAFETY discipline on ATTACH_WIRE).
    // SAFETY: this is the file's only env mutation; every wire ATTACH in this
    // file (including the in-crate `env::var` read inside the policy) holds
    // ATTACH_WIRE across the read, so no concurrent getenv runs during
    // set_var/remove_var. macOS libc makes setenv/getenv themselves
    // thread-safe as a second layer.
    unsafe { std::env::set_var("GRAPHLAW_ATTACH_ROOTS", &root) };

    // (6) Positive twin: the inside-root ATTACH EXECUTES over the wire on a persistent session.
    let pos_session = format!("attach-court-pos-{}", std::process::id());
    let open_res = graphlaw::abi::call_json(&json!({
        "op": "duckdb",
        "mode": "open",
        "session": pos_session,
    }));
    assert_eq!(open_res["ok"], serde_json::Value::Bool(true), "open session: {open_res}");
    let out = exec_attach(Some(&pos_session), db_path.to_str().expect("utf8 temp path"), "att_root");
    assert_eq!(
        out["ok"],
        serde_json::Value::Bool(true),
        "inside-root ATTACH must execute: {out}"
    );
    // Executed-SQL readback: the receipt echoes the admitted ATTACH verbatim —
    // what executed is exactly what the allowlist admitted.
    assert_eq!(
        out["sql"],
        json!(format!("ATTACH '{}' AS att_root", db_path.to_str().expect("utf8"))),
        "receipt echoes the admitted ATTACH: {out}"
    );
    assert_eq!(out["rows_emitted"], 0, "{out}");
    assert!(
        out["cursor"].as_str().is_some_and(|c| !c.is_empty()),
        "cursor minted: {out}"
    );
    assert!(
        out["receipt"]
            .as_str()
            .is_some_and(|r| r.starts_with("urn:attach-court/receipt_")),
        "receipt IRI minted from the subject prefix: {out}"
    );

    // (7) Real state readback that the attach STUCK in the session's engine:
    // the identical attach on the same session now fails inside the ENGINE
    // (duplicate alias named in the error) — it passed the policy, so only the
    // engine can be the refuser.
    let dup = exec_attach(Some(&pos_session), db_path.to_str().expect("utf8"), "att_root");
    assert_eq!(dup["ok"], serde_json::Value::Bool(false), "{dup}");
    assert_eq!(dup["error"]["kind"], "DuckDb", "{dup}");
    let dup_msg = dup["error"]["message"].as_str().unwrap_or("");
    assert!(
        dup_msg.contains("att_root"),
        "duplicate attach must be an engine duplicate-alias error naming att_root \
         (proves leg (6) persisted a real attachment): {dup}"
    );

    // (8) With the env set, the allowlist still gates: a real database file
    // OUTSIDE the root is the typed refusal, raw path carried.
    let outside = std::env::temp_dir().join(format!("attach-court-outside-{}.duckdb", std::process::id()));
    drop(frozen_duckdb::Connection::open(&outside).expect("mint real outside-root duckdb file"));
    let out = exec_attach(
        None,
        outside.to_str().expect("utf8"),
        "att_out",
    );
    assert_typed_attach_refusal(&out, outside.to_str().expect("utf8"), "env-set outside-root attach");

    // (9) Env restoration + disk cleanup.
    // SAFETY: same ATTACH_WIRE discipline as the set_var above.
    unsafe { std::env::remove_var("GRAPHLAW_ATTACH_ROOTS") };
    assert!(std::env::var("GRAPHLAW_ATTACH_ROOTS").is_err(), "env restored");
    let _ = std::fs::remove_dir_all(&root);
    let _ = std::fs::remove_file(&outside);
}

/// Document-mode court: the per-member policy gate runs BEFORE any member
/// executes, so a sequence whose later member is an outside-allowlist ATTACH
/// refuses typed with nothing executed — and leaves the session usable.
#[test]
fn wire_document_member_attach_refuses_typed() {
    let _guard = ATTACH_WIRE
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let session = format!("attach-court-doc-{}", std::process::id());
    let open_res = graphlaw::abi::call_json(&json!({
        "op": "duckdb",
        "mode": "open",
        "session": session,
    }));
    assert_eq!(open_res["ok"], serde_json::Value::Bool(true), "open session: {open_res}");
    let turtle = r#"@prefix sql: <http://graphlaw.org/sql#> .
@prefix ex: <http://example.org/attach#> .
ex:doc a sql:StatementSequence ;
    sql:statements ( ex:m0 ex:m1 ) .
ex:m0 a sql:CreateStatement ;
    sql:statementIndex 0 ;
    sql:targetTable "attach_doc_benign" ;
    sql:assignColumn "x" .
ex:m1 a sql:AttachStatement ;
    sql:statementIndex 1 ;
    sql:attachTarget [ sql:databasePath "/etc/passwd" ; sql:attachAlias "att_doc" ] .
"#;
    let out = graphlaw::abi::call_json(&json!({
        "op": "duckdb",
        "mode": "exec",
        "document": true,
        "session": session,
        "query_ast": "http://example.org/attach#doc",
        "ast": {"turtle": turtle},
        "subject_prefix": "urn:attach-court",
        "vocab_prefix": "urn:attach-court/v",
    }));
    // The member gate refuses typed BEFORE eval_ast_document: kind DuckDb,
    // stable message, raw path — no partial execution, no generic DB error.
    assert_typed_attach_refusal(&out, ATTACK_PATH, "document member attach");

    // Unpoisoned surface: a benign single-statement exec on the same session
    // still executes.
    let create = graphlaw::abi::call_json(&json!({
        "op": "duckdb",
        "mode": "exec",
        "session": session,
        "query_ast": "http://example.org/attach#create",
        "ast": {"turtle": r#"@prefix sql: <http://graphlaw.org/sql#> .
@prefix ex: <http://example.org/attach#> .
ex:C a sql:CreateStatement ;
    sql:statementIndex 1 ;
    sql:targetTable "attach_doc_unpoisoned" ;
    sql:assignColumn "x" .
"#},
        "subject_prefix": "urn:attach-court",
        "vocab_prefix": "urn:attach-court/v",
    }));
    assert_eq!(
        create["ok"],
        serde_json::Value::Bool(true),
        "surface unpoisoned after the document-member refusal: {create}"
    );
}

/// Anti-vacuity liveness court for the literal attack-string corpus: every
/// camouflage string IS an ATTACH for the real DuckDB engine — it executes and
/// the alias reads back from the real catalog. This is the property the
/// comment-aware scanner must classify on (the pre-fix whitespace-only
/// scanner classified these as non-ATTACH and skipped the allowlist: the
/// bypass). The typed-refusal assertions for the same literal strings live at
/// the policy seam in src/duckdb_op.rs::tests; if those seam courts and this
/// liveness court ever disagree, either the scanner regressed or the engine
/// changed — both are loud.
#[test]
fn camouflage_attack_strings_are_engine_live_attach_statements() {
    // A real DuckDB database file OUTSIDE any allowlisted root (the env is
    // not consulted on this leg: the engine has no allowlist).
    let db_path = std::env::temp_dir().join(format!("attach-court-camo-{}.duckdb", std::process::id()));
    drop(frozen_duckdb::Connection::open(&db_path).expect("mint real camo duckdb file"));
    let db_path = db_path.to_str().expect("utf8 temp path");

    for (label, attack) in camouflage_corpus(db_path) {
        let conn = frozen_duckdb::Connection::open_in_memory().expect("real engine connection");
        // The attack executes — the engine sees an ATTACH, not a comment.
        conn.execute_batch(&attack)
            .unwrap_or_else(|e| panic!("[{label}] camouflaged ATTACH must execute: {e}"));
        // Real catalog readback: the alias is attached on this connection.
        let alias = match label {
            "block_comment" => "camo_block",
            "line_comment" => "camo_line",
            "nested_block_comment" => "camo_nested",
            "bang_comment" => "camo_bang",
            "leading_tab" => "camo_tab",
            "leading_whitespace" => "camo_ws",
            other => panic!("unknown corpus label: {other}"),
        };
        let n: String = conn
            .query_row(
                "SELECT count(*)::VARCHAR FROM duckdb_databases() WHERE database_name = ?",
                [alias],
                |r| r.get(0),
            )
            .unwrap_or_else(|e| panic!("[{label}] catalog readback: {e}"));
        assert_eq!(n, "1", "[{label}] alias {alias} must be attached: {attack}");
    }
    let _ = std::fs::remove_file(db_path);
}
