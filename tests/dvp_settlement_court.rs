//! Case 1 — DvP atomic settlement court (CPMI-IOSCO P12 shape), black-box
//! over the in-wasm document pipeline (`tests/wasm_fibo_fixtures_eh.rs`
//! harness pattern: wasmtime 48 + `wasm_exceptions(true)`, p1 WASI,
//! gl_alloc/gl_call/gl_free, ENGINE_LOCK, GRAPHLAW_WASM resolution).
//!
//! The pipeline mode under test is SQLRDF-23 (`"document":true`) with the
//! SQLRDF-26 atomic flag (`"atomic":true` OR `sql:atomic true` on the
//! sequence root) — both landed in `src/duckdb_op.rs` (`exec_document_on`)
//! and `src/duckdb_eval.rs` (`eval_ast_document`), so the courts run in
//! atomic mode, not pipeline fallback.
//!
//! Courts:
//! - DVP-01 (committed leg): atomic 3-member document — debit payer cash,
//!   credit payee cash, transfer asset ownership — all bounded WHERE
//!   predicates. Asserts exact balance/ownership readback, `doc_hash`
//!   present, `rows_emitted == [1,1,1]`, `failed_at == null`.
//! - DVP-02 (aborted leg): a member that fails at the engine (WHERE naming a
//!   nonexistent column) → wire `BatchMemberFailed` with `member` named,
//!   `partial_receipt.failed_at` pinned, `status:"aborted"`,
//!   `state_delta:0`, and CRITICALLY the readback proves zero net state
//!   change (pre/post SELECT streams byte-identical).
//!   Honest gap probe (documented finding, asserted): "insufficient funds"
//!   encoded as a bounded WHERE that matches nothing is NOT a member
//!   failure — the engine has no zero-row-mutation refusal, so the document
//!   commits `ok:true` with the payer un-debited. One-legged settlement is
//!   caught at the conformance layer (DVP-03), not the execution layer.
//! - DVP-03 (anti-vacuity): a trace recording the asset transfer without
//!   cash settlement (single-UPDATE atomic doc) must NOT produce a
//!   conformance-passing pair — the court's own two-sided check
//!   (`settlement_conformant`) must catch the one-legged state.
//!
//! Typed loud SKIP discipline: missing artifact / stale-artifact trap →
//! `SKIP(typed)` and return. A SKIP is not a pass.
//!
//! Chicago: real module, real frozen-duckdb inside it, real SELECT readbacks;
//! no mocks. Fixtures are inline documents (tests/fixtures/fibo/ is another
//! lane's lease; no collision introduced).

#![cfg(all(feature = "abi", not(target_arch = "wasm32")))]

use std::path::PathBuf;
use std::sync::Mutex;

use serde_json::{Value, json};
use wasmtime::{Config, Engine, Instance, Linker, Memory, Module, Store, TypedFunc};
use wasmtime_wasi::WasiCtxBuilder;
use wasmtime_wasi::p1::{WasiP1Ctx, add_to_linker_sync};

static ENGINE_LOCK: Mutex<()> = Mutex::new(());

fn engine_lock() -> std::sync::MutexGuard<'static, ()> {
    ENGINE_LOCK.lock().unwrap_or_else(std::sync::PoisonError::into_inner)
}

// --------------------------------------------------------------- harness
// (verbatim shape from tests/wasm_fibo_fixtures_eh.rs)

fn wasm_path() -> Option<PathBuf> {
    if let Some(p) = std::env::var_os("GRAPHLAW_WASM") {
        let path = PathBuf::from(p);
        if !path.is_file() {
            panic!("GRAPHLAW_WASM is set but the artifact is missing: {}", path.display());
        }
        return Some(path);
    }
    let artifact = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("target/wasm32-wasip1/wasm/graphlaw_wasm.wasm");
    if artifact.is_file() {
        return Some(artifact);
    }
    None
}

struct Host {
    store: Store<WasiP1Ctx>,
    memory: Memory,
    alloc: TypedFunc<u32, u32>,
    free: TypedFunc<(u32, u32), ()>,
    call: TypedFunc<(u32, u32), u64>,
}

#[derive(Debug)]
enum HostError {
    Trap(String),
    Fatal(String),
}

impl Host {
    fn new(bytes: &[u8]) -> Result<Self, HostError> {
        let fail = |e: wasmtime::Error, stage: &'static str| {
            let s = format!("{stage}: {e}");
            if e.to_string().contains("uninitialized element") {
                HostError::Trap(s)
            } else {
                HostError::Fatal(s)
            }
        };
        let mut cfg = Config::new();
        cfg.wasm_exceptions(true);
        let engine = Engine::new(&cfg).map_err(|e| HostError::Fatal(format!("engine: {e}")))?;
        let module = Module::new(&engine, bytes)
            .map_err(|e| HostError::Fatal(format!("module: {e}")))?;
        for i in module.imports() {
            if i.module() != "wasi_snapshot_preview1" {
                return Err(HostError::Fatal(format!(
                    "unexpected host import {}::{}",
                    i.module(),
                    i.name()
                )));
            }
        }
        let mut store = Store::new(
            &engine,
            WasiCtxBuilder::new()
                .inherit_stdio()
                .inherit_env()
                .preopened_dir("/tmp", "/tmp", wasmtime_wasi::FsPerms::ReadWrite)
                .map_err(|e| HostError::Fatal(format!("preopen /tmp: {e}")))?
                .build_p1(),
        );
        let mut linker = <Linker<WasiP1Ctx>>::new(&engine);
        add_to_linker_sync(&mut linker, |ctx| ctx)
            .map_err(|e| HostError::Fatal(format!("wasi linker: {e}")))?;
        let instance: Instance = linker
            .instantiate(&mut store, &module)
            .map_err(|e| fail(e, "instantiate"))?;
        if let Ok(init) = instance.get_typed_func::<(), ()>(&mut store, "_initialize") {
            init.call(&mut store, ()).map_err(|e| fail(e, "_initialize"))?;
        }
        let memory = instance
            .get_memory(&mut store, "memory")
            .ok_or_else(|| HostError::Fatal("module does not export memory".into()))?;
        let missing = |name: &'static str, e: wasmtime::Error| {
            HostError::Fatal(format!("missing or mistyped export {name}: {e}"))
        };
        Ok(Host {
            alloc: instance
                .get_typed_func::<u32, u32>(&mut store, "gl_alloc")
                .map_err(|e| missing("gl_alloc", e))?,
            free: instance
                .get_typed_func::<(u32, u32), ()>(&mut store, "gl_free")
                .map_err(|e| missing("gl_free", e))?,
            call: instance
                .get_typed_func::<(u32, u32), u64>(&mut store, "gl_call")
                .map_err(|e| missing("gl_call", e))?,
            store,
            memory,
        })
    }

    fn request_bytes(&mut self, body: &[u8]) -> Result<Vec<u8>, HostError> {
        let ptr = self
            .alloc
            .call(&mut self.store, body.len() as u32)
            .map_err(|e| fail_classify("gl_alloc", e))?;
        if ptr == 0 {
            return Err(HostError::Fatal("gl_alloc refused a pre-checked request".into()));
        }
        self.memory
            .write(&mut self.store, ptr as usize, body)
            .map_err(|e| HostError::Fatal(format!("memory write: {e}")))?;
        let packed = self
            .call
            .call(&mut self.store, (ptr, body.len() as u32))
            .map_err(|e| fail_classify("gl_call", e))?;
        let (out_ptr, out_len) = ((packed >> 32) as u32, (packed & 0xffff_ffff) as u32);
        let mut out = vec![0u8; out_len as usize];
        self.memory
            .read(&self.store, out_ptr as usize, &mut out)
            .map_err(|e| HostError::Fatal(format!("memory read: {e}")))?;
        self.free
            .call(&mut self.store, (out_ptr, out_len))
            .map_err(|e| fail_classify("gl_free", e))?;
        Ok(out)
    }

    fn request(&mut self, req: &Value) -> Result<Value, HostError> {
        let out = self.request_bytes(req.to_string().as_bytes())?;
        serde_json::from_slice(&out)
            .map_err(|e| HostError::Fatal(format!("response is not JSON: {e}")))
    }

    fn exec(&mut self, req: &Value) -> Value {
        let r = self.request(req).expect("request transport");
        assert_eq!(r["ok"], json!(true), "exec request failed: {req} -> {r}");
        r
    }

    fn stream_all(&mut self, session: &str, cursor: &str) -> String {
        let mut body = String::new();
        loop {
            let page = self.exec(&json!({
                "op": "duckdb", "mode": "stream", "session": session,
                "cursor": cursor, "max_rows": 100
            }));
            if let Some(b) = page["nquads"].as_str() {
                body.push_str(b);
            }
            if page["more"].as_bool() != Some(true) {
                return body;
            }
        }
    }
}

fn fail_classify(stage: &str, e: wasmtime::Error) -> HostError {
    let s = format!("{stage}: {e}");
    if e.to_string().contains("uninitialized element") {
        HostError::Trap(s)
    } else {
        HostError::Fatal(s)
    }
}

fn try_host() -> Result<Host, String> {
    let path = wasm_path().ok_or_else(|| "artifact-missing".to_string())?;
    let bytes = std::fs::read(&path).map_err(|e| format!("fatal: read {}: {e}", path.display()))?;
    match Host::new(&bytes) {
        Ok(h) => Ok(h),
        Err(HostError::Trap(e)) => Err(format!("trap: {e}")),
        Err(HostError::Fatal(e)) => Err(format!("fatal: {e}")),
    }
}

fn host_or_skip(test_name: &str) -> Option<Host> {
    match try_host() {
        Ok(h) => Some(h),
        Err(reason) => {
            let kind = if reason == "artifact-missing" {
                "artifact not built"
            } else if reason.starts_with("trap:") {
                "first gl_call trapped on uninitialized element (stale/mismatched artifact)"
            } else {
                "host construction failed"
            };
            eprintln!(
                "// SKIP(typed) [{test_name}]: {kind} — reason: {reason} — \
                 suite pending-artifact; build with `WASI_SDK_PATH=<wasi-sdk> \
                 cargo build -p graphlaw-wasm --target wasm32-wasip1 --profile wasm`"
            );
            None
        }
    }
}

// --------------------------------------------------------------- fixtures
// (inline — tests/fixtures/fibo/ is another lane's lease)

const PREFIXES: &str = r#"@prefix ex: <http://example.org/dvp#> .
@prefix sql: <http://graphlaw.org/sql#> .
@prefix xsd: <http://www.w3.org/2001/XMLSchema#> .
"#;

fn exec_req(session: &str, turtle: &str) -> Value {
    json!({
        "op": "duckdb",
        "mode": "exec",
        "session": session,
        "query_ast": "http://example.org/dvp#doc",
        "subject_prefix": "http://example.org/dvp",
        "vocab_prefix": "http://example.org/vocab",
        "ast": {"turtle": turtle, "dialect": "duckdb"}
    })
}

/// Document exec request: the whole Turtle AST document through the
/// SQLRDF-23 document pipeline, with the SQLRDF-26 atomic flag.
fn doc_req(session: &str, turtle: &str, atomic: bool) -> Value {
    let mut req = exec_req(session, turtle);
    req["document"] = json!(true);
    req["atomic"] = json!(atomic);
    req
}

/// Schema + seed documents. Account balances start at 1000.00 each; the
/// asset is owned by PAYER.
fn setup(h: &mut Host, session: &str) {
    h.exec(&exec_req(
        session,
        &format!(
            r#"{PREFIXES}
ex:schema1 a sql:CreateStatement ;
    sql:statementIndex 0 ;
    sql:tableDefinition [
        sql:tableName "dvp_accounts" ;
        sql:columns (
            [ sql:columnName "acct" ; sql:columnType "VARCHAR" ]
            [ sql:columnName "balance" ; sql:columnType "DECIMAL" ]
        ) ;
    ] ."#
        ),
    ));
    h.exec(&exec_req(
        session,
        &format!(
            r#"{PREFIXES}
ex:schema2 a sql:CreateStatement ;
    sql:statementIndex 0 ;
    sql:tableDefinition [
        sql:tableName "dvp_assets" ;
        sql:columns (
            [ sql:columnName "asset_id" ; sql:columnType "VARCHAR" ]
            [ sql:columnName "owner" ; sql:columnType "VARCHAR" ]
        ) ;
    ] ."#
        ),
    ));
    // Seed: PAYER 1000, PAYEE 1000, asset A1 owned by PAYER. The seed is a
    // 3+-member sequence, so it must go through the document exec path — the
    // single-statement exec path refuses non-single-root documents
    // (REFUSAL-01, MIXED_DOCUMENT_ROOTS). Every member carries exactly ONE
    // assign pair: multi-pair INSERT/UPDATE ladders depend on the compiler's
    // document-order pairing scan, which does not survive the module round
    // trip (witnessed: VALUES (1000.00, 'PAYEE') in-module vs correct
    // pairing host-side); single-pair members are order-immune.
    h.exec(&doc_req(
        session,
        &format!(
            r#"{PREFIXES}
ex:seed a sql:StatementSequence ;
    sql:statements ( ex:s0 ex:s1 ex:s2 ex:s3 ex:s4 ex:s5 ) .
ex:s0 a sql:InsertStatement ;
    sql:statementIndex 0 ;
    sql:targetTable "dvp_accounts" ;
    sql:assignColumn "acct" ; sql:assignValue "PAYER" .
ex:s1 a sql:InsertStatement ;
    sql:statementIndex 1 ;
    sql:targetTable "dvp_accounts" ;
    sql:assignColumn "acct" ; sql:assignValue "PAYEE" .
ex:s2 a sql:InsertStatement ;
    sql:statementIndex 2 ;
    sql:targetTable "dvp_assets" ;
    sql:assignColumn "asset_id" ; sql:assignValue "A1" .
ex:s3 a sql:UpdateStatement ;
    sql:statementIndex 3 ;
    sql:targetTable "dvp_accounts" ;
    sql:assignColumn "balance" ; sql:assignValue "1000.00"^^xsd:decimal ;
    sql:whereColumn "acct" ; sql:whereValue "PAYER" .
ex:s4 a sql:UpdateStatement ;
    sql:statementIndex 4 ;
    sql:targetTable "dvp_accounts" ;
    sql:assignColumn "balance" ; sql:assignValue "1000.00"^^xsd:decimal ;
    sql:whereColumn "acct" ; sql:whereValue "PAYEE" .
ex:s5 a sql:UpdateStatement ;
    sql:statementIndex 5 ;
    sql:targetTable "dvp_assets" ;
    sql:assignColumn "owner" ; sql:assignValue "PAYER" ;
    sql:whereColumn "asset_id" ; sql:whereValue "A1" ."#
        ),
        true,
    ));
}

fn select_doc(root: &str, table: &str, columns: &[&str]) -> String {
    let list: Vec<String> = columns
        .iter()
        .map(|c| format!("[ sql:columnName \"{c}\" ]"))
        .collect();
    format!(
        r#"{PREFIXES}
ex:{root} a sql:SelectStatement ;
    sql:statementIndex 0 ;
    sql:fromTable [ sql:tableName "{table}" ] ;
    sql:selectList ( {} ) ."#,
        list.join(" ")
    )
}

fn readback(h: &mut Host, session: &str) -> (String, String) {
    let cash = h.exec(&exec_req(session, &select_doc("selc", "dvp_accounts", &["acct", "balance"])));
    let cash_body = h.stream_all(session, cash["cursor"].as_str().expect("cursor id"));
    let asset = h.exec(&exec_req(session, &select_doc("sela", "dvp_assets", &["asset_id", "owner"])));
    let asset_body = h.stream_all(session, asset["cursor"].as_str().expect("cursor id"));
    (cash_body, asset_body)
}

fn open_session(h: &mut Host, name: &str) {
    let r = h.exec(&json!({"op": "duckdb", "mode": "open", "session": name}));
    assert_eq!(r["session"], json!(name), "session open reply: {r}");
}

fn close_session(h: &mut Host, name: &str) {
    h.exec(&json!({"op": "duckdb", "mode": "close", "session": name}));
}

/// One UPDATE sequence member (bounded WHERE).
fn update_member(idx: usize, node: &str, table: &str, set_col: &str, set_val: &str, where_col: &str, where_val: &str) -> String {
    format!(
        r#"ex:{node} a sql:UpdateStatement ;
    sql:statementIndex {idx} ;
    sql:targetTable "{table}" ;
    sql:assignColumn "{set_col}" ; sql:assignValue "{set_val}"^^xsd:decimal ;
    sql:whereColumn "{where_col}" ; sql:whereValue "{where_val}" ."#
    )
}

fn update_member_var(idx: usize, node: &str, table: &str, set_col: &str, set_val: &str, where_col: &str, where_val: &str) -> String {
    // varchar-valued SET (asset ownership transfer)
    format!(
        r#"ex:{node} a sql:UpdateStatement ;
    sql:statementIndex {idx} ;
    sql:targetTable "{table}" ;
    sql:assignColumn "{set_col}" ; sql:assignValue "{set_val}" ;
    sql:whereColumn "{where_col}" ; sql:whereValue "{where_val}" ."#
    )
}

fn sequence_doc(atomic: bool, members: &[String]) -> String {
    // Canonical one-root shape (REFUSAL-01): the sequence root lists member
    // IRIs in sql:statements ( ... ); member bodies sit at top level.
    let iris: Vec<String> = (0..members.len()).map(|i| format!("ex:m{i}")).collect();
    let iris_list = iris.join(" ");
    let atomic_triple = if atomic { " ;\n    sql:atomic true" } else { "" };
    format!(
        r#"{PREFIXES}
ex:dvp1 a sql:StatementSequence{atomic_triple} ;
    sql:statements ( {iris_list} ) .
{members}"#,
        members = members.join("\n")
    )
}

/// The court's own conformance check: a DvP settlement pair conforms only
/// when BOTH sides observed — cash moved (payer debited to 900 AND payee
/// credited to 1100) and the asset changed hands (owner PAYEE, no PAYER
/// ownership row remains). This is the predicate DVP-03 attacks.
fn settlement_conformant(cash_body: &str, asset_body: &str) -> bool {
    let payer_debited = cash_body.contains("900.00");
    let payee_credited = cash_body.contains("1100.00");
    let payer_pre_debit_gone = !cash_body.contains("1000.00");
    let asset_transferred = asset_body.contains("PAYEE") && !asset_body.contains("PAYER");
    payer_debited && payee_credited && payer_pre_debit_gone && asset_transferred
}

// ------------------------------------------------------------- DVP-01

#[test]
fn dvp_01_committed_atomic_settlement() {
    let _guard = engine_lock();
    let Some(mut h) = host_or_skip("DVP-01") else { return };
    const SESSION: &str = "dvp-01";
    open_session(&mut h, SESSION);
    setup(&mut h, SESSION);

    let doc = sequence_doc(true, &[
        update_member(0, "m0", "dvp_accounts", "balance", "900.00", "acct", "PAYER"),
        update_member(1, "m1", "dvp_accounts", "balance", "1100.00", "acct", "PAYEE"),
        update_member_var(2, "m2", "dvp_assets", "owner", "PAYEE", "asset_id", "A1"),
    ]);

    let r = h.exec(&doc_req(SESSION, &doc, true));
    assert!(r["doc_hash"].is_string(), "doc_hash missing: {r}");
    assert_eq!(r["doc_hash"].as_str().unwrap().len(), 64, "doc_hash is 32-byte hex: {r}");
    assert_eq!(r["failed_at"], Value::Null, "no member may fail: {r}");
    assert_eq!(r["rows_emitted"], json!([1, 1, 1]), "one row per member: {r}");
    assert_eq!(r["members"].as_array().map(Vec::len), Some(3), "3 members receipted: {r}");

    let (cash_body, asset_body) = readback(&mut h, SESSION);
    assert!(
        cash_body.contains("PAYER") && cash_body.contains("PAYEE"),
        "both accounts read back: {cash_body}"
    );
    assert!(
        settlement_conformant(&cash_body, &asset_body),
        "committed settlement must be two-sided; cash: {cash_body} | asset: {asset_body}"
    );

    close_session(&mut h, SESSION);
}

// ------------------------------------------------------------- DVP-02

#[test]
fn dvp_02_aborted_atomic_settlement_zero_net_change() {
    let _guard = engine_lock();
    let Some(mut h) = host_or_skip("DVP-02") else { return };
    const SESSION: &str = "dvp-02";
    open_session(&mut h, SESSION);
    setup(&mut h, SESSION);

    let (pre_cash, pre_asset) = readback(&mut h, SESSION);

    // Member 0 (payer debit) executes; member 1 fails at the engine — its
    // WHERE names a column that does not exist in dvp_accounts.
    let doc = sequence_doc(true, &[
        update_member(0, "m0", "dvp_accounts", "balance", "900.00", "acct", "PAYER"),
        update_member(1, "m1", "dvp_accounts", "balance", "1100.00", "margin_acct", "PAYEE"),
        update_member_var(2, "m2", "dvp_assets", "owner", "PAYEE", "asset_id", "A1"),
    ]);

    let r = h
        .request(&doc_req(SESSION, &doc, true))
        .expect("request transport");
    assert_eq!(r["ok"], json!(false), "failed member must fail the document: {r}");
    let msg = r["error"]["message"]
        .as_str()
        .unwrap_or_else(|| panic!("error.message missing from failure envelope: {r}"));
    assert!(
        msg.contains("member 1 of document"),
        "failure message must name the failing member index: {msg}"
    );
    let details = &r["error"]["details"];
    assert_eq!(
        details["code"], json!("BatchMemberFailed"),
        "typed member-failure envelope: {r}"
    );
    assert_eq!(details["member"], json!(1), "failed_at member index: {r}");
    let partial = &details["partial_receipt"];
    assert_eq!(
        partial["failed_at"], json!(1),
        "partial receipt pins failed_at: {r}"
    );
    assert_eq!(
        partial["status"], json!("aborted"),
        "atomic abort status: {r}"
    );
    assert_eq!(
        partial["state_delta"], json!(0),
        "atomic abort undoes every member effect: {r}"
    );
    assert_eq!(
        partial["doc_hash"].as_str().map(str::len),
        Some(64),
        "partial receipt carries the document hash: {r}"
    );

    // CRITICAL: zero net state change — pre/post readbacks byte-identical.
    let (post_cash, post_asset) = readback(&mut h, SESSION);
    assert_eq!(
        pre_cash, post_cash,
        "cash table must be byte-identical across the aborted atomic document"
    );
    assert_eq!(
        pre_asset, post_asset,
        "asset table must be byte-identical across the aborted atomic document"
    );

    // Honest gap probe (typed finding, not a pass): "insufficient funds"
    // encoded as a bounded WHERE that matches NOTHING is not a member
    // failure — the engine has no zero-row-mutation refusal, so this
    // document commits with the payer un-debited. One-legged settlement is
    // caught at the conformance layer (DVP-03), not the execution layer.
    let ghost = sequence_doc(true, &[
        update_member(0, "g0", "dvp_accounts", "balance", "900.00", "acct", "GHOST-ACCT"),
        update_member(1, "g1", "dvp_accounts", "balance", "1100.00", "acct", "PAYEE"),
        update_member_var(2, "g2", "dvp_assets", "owner", "PAYEE", "asset_id", "A1"),
    ]);
    let g = h.exec(&doc_req(SESSION, &ghost, true));
    assert_eq!(g["failed_at"], Value::Null, "empty-WHERE-match commits: {g}");
    assert_eq!(
        g["rows_emitted"], json!([0, 1, 1]),
        "the ghost debit matches zero rows: {g}"
    );
    let (gap_cash, _) = readback(&mut h, SESSION);
    assert!(
        !settlement_conformant(&gap_cash, &post_asset),
        "the committed empty-debit document must NOT read as a conformant settlement"
    );

    close_session(&mut h, SESSION);
}

// ------------------------------------------------------------- DVP-03

#[test]
fn dvp_03_one_legged_trace_rejected_by_conformance_check() {
    let _guard = engine_lock();
    let Some(mut h) = host_or_skip("DVP-03") else { return };
    const SESSION: &str = "dvp-03";
    open_session(&mut h, SESSION);
    setup(&mut h, SESSION);

    // The attack trace: the asset leg alone, recorded as an atomic document.
    // It commits cleanly — the execution layer has no cross-table settlement
    // law — so the CONFORMANCE check is the only thing standing between this
    // trace and a passed settlement.
    let doc = sequence_doc(true, &[update_member_var(
        0, "m0", "dvp_assets", "owner", "PAYEE", "asset_id", "A1",
    )]);
    let r = h.exec(&doc_req(SESSION, &doc, true));
    assert_eq!(r["failed_at"], Value::Null, "single-member doc commits: {r}");
    assert_eq!(r["rows_emitted"], json!([1]), "asset leg executes: {r}");
    assert!(
        r["doc_hash"].as_str().map(str::len) == Some(64),
        "single-leg receipt still carries doc_hash: {r}"
    );

    let (cash_body, asset_body) = readback(&mut h, SESSION);
    assert!(
        asset_body.contains("PAYEE"),
        "the asset leg really did execute: {asset_body}"
    );
    assert!(
        !settlement_conformant(&cash_body, &asset_body),
        "ANTI-VACUITY: the one-legged trace must NOT pass the court's own \
         conformance check — cash never moved"
    );

    // Twin: with the cash legs ALSO landed (DVP-01's document), the same
    // check passes — proving the check is not vacuously false.
    let full = sequence_doc(true, &[
        update_member(0, "c0", "dvp_accounts", "balance", "900.00", "acct", "PAYER"),
        update_member(1, "c1", "dvp_accounts", "balance", "1100.00", "acct", "PAYEE"),
    ]);
    h.exec(&doc_req(SESSION, &full, true));
    let (cash_full, asset_full) = readback(&mut h, SESSION);
    assert!(
        settlement_conformant(&cash_full, &asset_full),
        "the two-sided settlement must pass the same check (non-vacuity twin)"
    );

    close_session(&mut h, SESSION);
}

/// Regression (banking inventory witness, 2026-10-23): a two-column INSERT
/// with ASYMMETRIC value types (plain string `acct`, `xsd:decimal` `balance`)
/// must compile with columns and values in matching document order. Under the
/// inverted pairing (independent traversal orders for assignColumn and
/// assignValue) this renders `VALUES (1000.00, 'PAYER')` for document order
/// `(acct 'PAYER', balance 1000.00)` — the exact swap that aborted the atomic
/// DvP seed batch at member 1 on the DECIMAL conversion. Anti-vacuity: this
/// assertion fails if the pairing in `SqlAst::assign_pairs` (src/sql_ast.rs)
/// is re-inverted to independent reader-order traversal, because purrdf's
/// `quads()` emits the two assignValue literals in the opposite order for
/// this document (witnessed in the G4 recovery scanner rationale).
#[test]
fn two_pair_asymmetric_string_decimal_insert_pairs_by_index() {
    let t = "@prefix ex: <http://example.org/dvp#> .\n@prefix sql: <http://graphlaw.org/sql#> .\n@prefix xsd: <http://www.w3.org/2001/XMLSchema#> .\n\nex:seed a sql:StatementSequence ;\n    sql:statements ( ex:seed1 ) .\nex:seed1 a sql:InsertStatement ;\n    sql:statementIndex 0 ;\n    sql:targetTable \"dvp_accounts\" ;\n    sql:assignColumn \"acct\" ; sql:assignValue \"PAYER\" ;\n    sql:assignColumn \"balance\" ; sql:assignValue \"1000.00\"^^xsd:decimal .\n";
    let d = graphlaw::sql_ast::compile_turtle_document(t).unwrap();
    assert_eq!(d.statements.len(), 1, "exactly one compiled statement");
    assert_eq!(
        d.statements[0].sql,
        "INSERT INTO dvp_accounts (acct, balance) VALUES ('PAYER', 1000.00)",
        "values must pair with columns by index in document order"
    );

    // Single-root control: same pairing law on the bare-INSERT path.
    let t2 = "@prefix ex: <http://example.org/dvp#> .\n@prefix sql: <http://graphlaw.org/sql#> .\n@prefix xsd: <http://www.w3.org/2001/XMLSchema#> .\nex:s1 a sql:InsertStatement ;\n    sql:statementIndex 0 ;\n    sql:targetTable \"dvp_accounts\" ;\n    sql:assignColumn \"acct\" ; sql:assignValue \"PAYEE\" ;\n    sql:assignColumn \"balance\" ; sql:assignValue \"1000.00\"^^xsd:decimal .\n";
    let d2 = graphlaw::sql_ast::compile_turtle(t2).unwrap();
    assert_eq!(
        d2.sql,
        "INSERT INTO dvp_accounts (acct, balance) VALUES ('PAYEE', 1000.00)"
    );
}
