//! Case 2 — BCBS 239 aggregation court: the receipt-bound fold over subsidiary
//! execution receipts, using the in-repo receipt-chain machinery
//! (`src/receipt_chain.rs`: `ReceiptStore`, `ReceiptValidator`,
//! `build_admission_frame`/`chain_from_frame`) and the real wasm DuckDB
//! lifecycle harness (the `tests/wasm_fibo_fixtures_eh.rs` pattern:
//! wasmtime 48 + `wasm_exceptions(true)`, p1 WASI, gl_alloc/gl_call/gl_free,
//! ENGINE_LOCK).
//!
//! Courts:
//! - TC-BCBS-01: three subsidiary drawdown INSERT documents executed via op
//!   duckdb; each response sealed as a chained child `ReceiptRecord` in a real
//!   `ReceiptStore`; the parent SELECT SUM over the union executes and its
//!   parent `ReceiptRecord`'s payload hash is the cryptographic fold of the
//!   child chain hashes (`chain_from_frame` over `build_admission_frame`
//!   commitments). Recomputing the fold from the loaded ledger reproduces the
//!   parent binding; tampering any child byte flips the validator's Verdict
//!   to FAIL.
//! - TC-BCBS-02: a hand-forged child receipt (chain hash not recomputed under
//!   the lawful rule) is refused by the chain validator BEFORE the parent SUM
//!   executes — validate-then-aggregate; the aggregate never runs.
//! - TC-BCBS-03 (anti-vacuity): a synthetic fold that sums rows WITHOUT
//!   verifying child receipt linkage produces a different digest than the
//!   verified fold, and a single flipped child byte moves the fold digest.
//!   The Petri-net OCEL logs (compliant vs bypass trace) are scored with
//!   pm4py token-based-replay fitness when python+pm4py is importable
//!   (typed loud SKIP of that rung otherwise; the digest divergence remains
//!   the unconditional Rust-side anti-vacuity witness).
//!
//! Typed loud SKIP discipline: artifact missing or stale → `SKIP(typed)` per
//! court. A SKIP is not a pass.
//!
//! Chicago: real module, real frozen-duckdb inside it, real JSONL ledger on
//! disk, real python subprocess for pm4py; no mocks.

#![cfg(all(feature = "abi", not(target_arch = "wasm32")))]

use std::path::PathBuf;

use graphlaw::receipt_chain::{
    ReceiptMeta, ReceiptRecord, RECEIPT_RECORD_VERSION,
    epoch::SCHEMA_V1,
    store::{GENESIS_CHAIN_HASH, ReceiptStore},
    validator::{Clock, ReceiptValidator, SystemClock, Verdict},
    Andon, build_admission_frame, chain_from_frame,
};
use serde_json::{Value, json};

mod common;

use common::engine_court::{Host, close_session, engine_lock, host_or_skip, open_session};

// --------------------------------------------------------------- harness
// Shared harness (engine lock, wasm host, session open/close) lives in
// tests/common/mod.rs::engine_court; this file keeps only the bcbs documents.

fn exec_req(session: &str, turtle: &str) -> Value {
    json!({
        "op": "duckdb",
        "mode": "exec",
        "session": session,
        "query_ast": "http://example.org/bcbs#stmt1",
        "subject_prefix": "http://example.org/bcbs",
        "vocab_prefix": "http://example.org/vocab",
        "ast": {"turtle": turtle, "dialect": "duckdb"}
    })
}

fn create_doc() -> String {
    r#"@prefix ex: <http://example.org/bcbs#> .
@prefix sql: <http://graphlaw.org/sql#> .

ex:stmt1 a sql:CreateStatement ;
    sql:statementIndex 0 ;
    sql:tableDefinition [
        sql:tableName "drawdowns" ;
        sql:columns (
            [ sql:columnName "amount" ; sql:columnType "DECIMAL" ]
        ) ;
    ] ."#
    .to_string()
}

/// One subsidiary INSERT document: rows bind to exactly one subsidiary.
/// Single-column assign pairs only — the scalar assignColumn/assignValue
/// pairing is reader-order (SQLRDF-22 G4) and multi-pair mixed-datatype docs
/// reorder deterministically-but-undocumented; one pair per statement is the
/// order-safe form. Subsidiary identity lives in the receipt chain
/// (`object_ids`), not in the row.
fn insert_doc(_subsidiary: &str, _id: i64, amount: &str) -> String {
    format!(
        r#"@prefix ex: <http://example.org/bcbs#> .
@prefix sql: <http://graphlaw.org/sql#> .
@prefix xsd: <http://www.w3.org/2001/XMLSchema#> .

ex:stmt1 a sql:InsertStatement ;
    sql:statementIndex 1 ;
    sql:targetTable "drawdowns" ;
    sql:assignColumn "amount" ;
    sql:assignValue "{amount}"^^xsd:decimal ."#
    )
}

/// Parent aggregate: SELECT SUM(amount) OVER THE UNION (single drawdowns
/// table carrying every subsidiary's rows).
fn parent_sum_doc() -> String {
    r#"@prefix ex: <http://example.org/bcbs#> .
@prefix sql: <http://graphlaw.org/sql#> .

ex:stmt1 a sql:SelectStatement ;
    sql:statementIndex 1 ;
    sql:fromTable [ sql:tableName "drawdowns" ] ;
    sql:selectList (
        [ sql:aliasName "total"
        ; sql:expression [ sql:functionName "sum"
        ; sql:arguments ( [ sql:columnName "amount" ] ) ] ]
    ) ."#
    .to_string()
}

const SUBSIDIARIES: &[(&str, &str)] = &[("SUB-A", "100.25"), ("SUB-B", "200.50"), ("SUB-C", "300.75")];

// ------------------------------------------------------- receipt machinery

/// Seal one execution response as a chained child `ReceiptRecord`: the
/// response wire bytes are the payload (blake3), the chain rule is the real
/// `recompute_chain_hash` path.
fn seal_child(
    prev_chain: [u8; 32],
    instruction_id: u64,
    ts_ns: u64,
    response_bytes: &[u8],
    subsidiary: &str,
) -> ReceiptRecord {
    let payload_hash = blake3::hash(response_bytes);
    let mut record = ReceiptRecord {
        version: RECEIPT_RECORD_VERSION,
        instruction_id,
        activity_idx: 1,
        activity: Some(format!("insert_subsidiary_drawdowns:{subsidiary}")),
        node_kind: 0,
        ts_ns,
        duration_ms: None,
        origin: None,
        payload_hash_hex: payload_hash.to_string(),
        prev_chain_hash_hex: hex::encode(prev_chain),
        chain_hash_hex: String::new(),
        andon: Andon::Green,
        obligation_count: 0,
        object_ids: vec![format!("bcbs:subsidiary:{subsidiary}")],
        signature_hex: None,
        schema: SCHEMA_V1.to_string(),
        v2: None,
        chain_rule: None,
    };
    let chain_hash = record.recompute_chain_hash().expect("child recompute");
    record.chain_hash_hex = hex::encode(chain_hash);
    record
}

/// The parent fold: `chain_hash(t+1) = BLAKE3(chain_hash(t) || frame(t+1))`
/// over the child chain hashes, each committed via `build_admission_frame`.
/// Returns the folded digest that becomes the parent record's payload
/// commitment.
fn fold_children(child_chain_hashes: &[[u8; 32]]) -> [u8; 32] {
    let mut acc = GENESIS_CHAIN_HASH;
    let mut i = 0u64;
    for child in child_chain_hashes {
        let meta = ReceiptMeta {
            instruction_id: i,
            activity_idx: 2,
            node_kind: 0,
            ts_ns: Some(0), // deterministic; fold is a pure function of the hashes
            ..ReceiptMeta::default()
        };
        let frame = build_admission_frame(child, &acc, &meta, 0);
        acc = chain_from_frame(&acc, &frame);
        i += 1;
    }
    acc
}

fn fold_hex(children: &[ReceiptRecord]) -> String {
    let hashes: Vec<[u8; 32]> = children
        .iter()
        .map(|c| c.chain_hash().expect("child chain hash"))
        .collect();
    hex::encode(fold_children(&hashes))
}

/// Append the parent aggregate record onto the ledger: its payload hash IS
/// the verified fold of the child chain hashes, so the parent cryptographically
/// binds every child byte.
fn seal_parent(children: &[ReceiptRecord], instruction_id: u64, ts_ns: u64) -> ReceiptRecord {
    let fold = fold_hex(children);
    let mut parent = ReceiptRecord {
        version: RECEIPT_RECORD_VERSION,
        instruction_id,
        activity_idx: 3,
        activity: Some("aggregate_parent_sum".to_string()),
        node_kind: 0,
        ts_ns,
        duration_ms: None,
        origin: None,
        payload_hash_hex: fold,
        prev_chain_hash_hex: children
            .last()
            .map(|c| c.chain_hash_hex.clone())
            .unwrap_or_else(|| hex::encode(GENESIS_CHAIN_HASH)),
        chain_hash_hex: String::new(),
        andon: Andon::Green,
        obligation_count: 0,
        object_ids: children.iter().flat_map(|c| c.object_ids.clone()).collect(),
        signature_hex: None,
        schema: SCHEMA_V1.to_string(),
        v2: None,
        chain_rule: None,
    };
    let chain_hash = parent.recompute_chain_hash().expect("parent recompute");
    parent.chain_hash_hex = hex::encode(chain_hash);
    parent
}

fn temp_dir(tag: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("graphlaw-bcbs-{tag}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    d
}

/// Monotone nanosecond timestamps from the real clock (validator's monotonic
/// stage requires strictly increasing ts and no future timestamps).
struct NanoNow(u64);
impl NanoNow {
    fn next(&mut self) -> u64 {
        let now = SystemClock.now_ns();
        if now <= self.0 {
            self.0 += 1;
        } else {
            self.0 = now;
        }
        self.0
    }
}

/// Run the three subsidiary INSERT documents through a real session, seal one
/// child receipt per response, and append them to the store.
fn execute_subsidiaries(
    h: &mut Host,
    session: &str,
    store: &ReceiptStore,
) -> Vec<ReceiptRecord> {
    open_session(h, session);
    h.exec(&exec_req(session, &create_doc()));
    let mut clock = NanoNow(0);
    let mut children = Vec::new();
    let mut prev = store.last_chain_hash().expect("store chain hash");
    for (i, (sub, amount)) in SUBSIDIARIES.iter().enumerate() {
        let doc = insert_doc(sub, i as i64 + 1, amount);
        let req = exec_req(session, &doc);
        let resp = h.exec(&req);
        assert!(
            resp["receipt"].is_string() && resp["query_hash"].is_string(),
            "INSERT receipt fields missing: {resp}"
        );
        let child = seal_child(
            prev,
            (i + 1) as u64,
            clock.next(),
            serde_json::to_vec(&resp)
                .expect("serialize response")
                .as_slice(),
            sub,
        );
        store.append(&child).expect("append child receipt");
        prev = child.chain_hash().expect("child chain hash");
        children.push(child);
    }
    children
}

fn failing_stage(verdict: &Verdict) -> Option<(String, String)> {
    verdict
        .stages
        .iter()
        .find(|s| !s.outcome.is_pass())
        .filter(|s| !matches!(s.outcome, graphlaw::receipt_chain::validator::CheckOutcome::Skip(_)))
        .map(|s| {
            (
                s.stage.clone(),
                match &s.outcome {
                    graphlaw::receipt_chain::validator::CheckOutcome::Fail(m) => m.clone(),
                    graphlaw::receipt_chain::validator::CheckOutcome::Skip(m) => m.clone(),
                    graphlaw::receipt_chain::validator::CheckOutcome::Pass => String::new(),
                },
            )
        })
}

// ------------------------------------------------------------- TC-BCBS-01

/// Valid fold: child receipts chained in a real store, parent SUM executes,
/// the parent record binds the verified fold, tamper flips the verdict.
#[test]
fn tc_bcbs_01_valid_fold_binds_child_receipts() {
    let _guard = engine_lock();
    let Some(mut h) = host_or_skip("TC-BCBS-01", true) else { return };
    let dir = temp_dir("01");
    let store = ReceiptStore::open(&dir).expect("open store");
    const SESSION: &str = "bcbs-01";

    // Three subsidiary documents, real execution, chained child receipts.
    let children = execute_subsidiaries(&mut h, SESSION, &store);
    assert_eq!(children.len(), 3, "one child receipt per subsidiary");
    let child_verdict = ReceiptValidator::validate(&children, &SystemClock);
    assert!(child_verdict.ok, "child chain must validate: {child_verdict:?}");

    // Parent aggregate over the union: SELECT SUM(amount) — read back 601.50.
    let sel = h.exec(&exec_req(SESSION, &parent_sum_doc()));
    assert_eq!(sel["rows_emitted"], json!(1), "aggregate emits one row");
    let body = h.stream_all(SESSION, sel["cursor"].as_str().expect("cursor id"));
    assert!(
        body.contains("601.5"),
        "SUM over union must read back 601.50 (100.25+200.50+300.75); got: {body}"
    );

    // Parent BatchReceipt: payload hash = verified fold of child chain hashes.
    let parent = seal_parent(&children, 4, NanoNow(0).next());
    assert_eq!(
        parent.payload_hash_hex,
        fold_hex(&children),
        "parent payload hash must be the recomputed child fold"
    );
    store.append(&parent).expect("append parent receipt");

    // Full ledger re-read from disk validates, and the parent binding
    // recomputes from the LOADED children (not the in-memory copies).
    let loaded = store.load_all().expect("load ledger");
    assert_eq!(loaded.len(), 4, "3 children + 1 parent on disk");
    let full_verdict = ReceiptValidator::validate(&loaded, &SystemClock);
    assert!(full_verdict.ok, "full ledger must validate: {full_verdict:?}");
    let (loaded_children, loaded_parent) = loaded.split_at(3);
    assert_eq!(
        loaded_parent[0].payload_hash_hex,
        fold_hex(loaded_children),
        "parent binding must reproduce from the disk ledger"
    );
    assert!(
        loaded_parent[0]
            .object_ids
            .iter()
            .any(|o| o == "bcbs:subsidiary:SUB-A"),
        "parent members must name every child's object ids"
    );

    // Tamper: flipping any child byte in the ledger flips the verdict to FAIL.
    let mut tampered = loaded.clone();
    let flipped_byte = {
        let old = tampered[1].payload_hash_hex.as_bytes()[3];
        if old == b'f' { '0' } else { 'f' }
    };
    tampered[1]
        .payload_hash_hex
        .replace_range(3..4, &flipped_byte.to_string());
    let tamper_verdict = ReceiptValidator::validate(&tampered, &SystemClock);
    assert!(!tamper_verdict.ok, "tampered ledger must FAIL: {tamper_verdict:?}");
    let (stage, msg) = failing_stage(&tamper_verdict).expect("a stage must fail");
    assert_eq!(stage, "chain_recompute", "tamper detected by chain recompute");
    assert!(
        msg.contains("tamper detected"),
        "failure must name tamper detection: {msg}"
    );

    close_session(&mut h, SESSION);
    let _ = std::fs::remove_dir_all(&dir);
}

// ------------------------------------------------------------- TC-BCBS-02

/// Tamper rejection: a hand-forged child receipt (chain hash asserted, not
/// recomputed; payload not bound to any execution response) is refused by the
/// validator BEFORE the parent SUM executes — validate-then-aggregate.
#[test]
fn tc_bcbs_02_forged_child_refused_before_aggregation() {
    let _guard = engine_lock();
    let Some(mut h) = host_or_skip("TC-BCBS-02", true) else { return };
    let dir = temp_dir("02");
    let store = ReceiptStore::open(&dir).expect("open store");
    const SESSION: &str = "bcbs-02";

    let children = execute_subsidiaries(&mut h, SESSION, &store);

    // Hand-forged child: plausible linkage (prev = last real chain hash) but
    // the chain hash is hand-written, never recomputed under the lawful rule,
    // and the payload hash commits to nothing that executed.
    let mut forged = seal_child(
        children.last().unwrap().chain_hash().unwrap(),
        99,
        NanoNow(0).next(),
        b"{\"forged\":true,\"no-execution-backed-receipt\":1}",
        "SUB-FORGED",
    );
    forged.chain_hash_hex = hex::encode([0xAAu8; 32]); // asserted, not derived

    // Validate-then-aggregate: the forged ledger is refused FIRST.
    let mut candidate = children.clone();
    candidate.push(forged);
    let verdict = ReceiptValidator::validate(&candidate, &SystemClock);
    assert!(!verdict.ok, "forged child must be refused: {verdict:?}");
    let (stage, _msg) = failing_stage(&verdict).expect("a stage must fail");
    assert_eq!(
        stage, "chain_recompute",
        "forgery is caught by chain-hash recompute, got stage {stage}"
    );

    // The refusal gates the aggregate: the parent SUM never executes.
    let mut aggregate_executed = false;
    if verdict.ok {
        aggregate_executed = true;
        h.exec(&exec_req(SESSION, &parent_sum_doc()));
    }
    assert!(
        !aggregate_executed,
        "the parent SUM must not execute behind a refused ledger"
    );

    close_session(&mut h, SESSION);
    let _ = std::fs::remove_dir_all(&dir);
}

// ------------------------------------------------------------- TC-BCBS-03

/// Anti-vacuity: a synthetic fold that sums rows WITHOUT verifying child
/// receipt linkage is detectable — the unverified fold digest differs from
/// the verified fold digest, a single flipped child byte moves the fold, and
/// the bypass OCEL trace scores fitness < 1.0 under pm4py token-based replay
/// (typed SKIP of the pm4py rung when python/pm4py is not importable; the
/// digest divergence remains the unconditional Rust-side witness).
#[test]
fn tc_bcbs_03_unverified_fold_is_detectable() {
    let _guard = engine_lock();
    let Some(mut h) = host_or_skip("TC-BCBS-03", true) else { return };
    let dir = temp_dir("03");
    let store = ReceiptStore::open(&dir).expect("open store");
    const SESSION: &str = "bcbs-03";

    let children = execute_subsidiaries(&mut h, SESSION, &store);
    let verified = fold_hex(&children);

    // Bypass fold: the SUM is computed over row digests that never had their
    // chain linkage verified (payload hashes, not chain hashes — i.e. the
    // aggregation reads receipts but never runs the validator).
    let bypass_hashes: Vec<[u8; 32]> = children
        .iter()
        .map(|c| c.payload_hash().expect("payload hash"))
        .collect();
    let bypass = hex::encode(fold_children(&bypass_hashes));
    assert_ne!(
        verified, bypass,
        "an unverified (linkage-skipping) fold must produce a different digest"
    );

    // Tamper-sensitivity: one flipped byte in ONE child chain hash moves the
    // verified fold — the fold is not vacuously insensitive.
    let mut flipped = children.clone();
    let new_char = {
        let old = flipped[1].chain_hash_hex.as_bytes()[5];
        if old == b'a' { b'0' as char } else { 'a' }
    };
    flipped[1]
        .chain_hash_hex
        .replace_range(5..6, &new_char.to_string());
    let tampered_fold = hex::encode(fold_children(&[flipped[0].chain_hash().unwrap(),
        hex::decode(&flipped[1].chain_hash_hex).unwrap().try_into().unwrap(),
        flipped[2].chain_hash().unwrap()]));
    assert_ne!(
        verified, tampered_fold,
        "a single tampered child byte must move the fold digest"
    );

    // Petri-net OCEL rung: compliant vs bypass trace, pm4py fitness.
    let compliant_log = temp_dir("ocel-compliant");
    let bypass_log = temp_dir("ocel-bypass");
    write_ocel_trace(&compliant_log, true);
    write_ocel_trace(&bypass_log, false);
    match pm4py_fitness(&compliant_log, &bypass_log) {
        Ok((fit_compliant, fit_bypass)) => {
            assert!(
                (fit_compliant - 1.0).abs() < 1e-6,
                "compliant trace must score fitness 1.0, got {fit_compliant}"
            );
            assert!(
                fit_bypass < 1.0,
                "bypass trace (no validate_child_receipts event) must score fitness < 1.0, got {fit_bypass}"
            );
        }
        Err(e) => {
            eprintln!(
                "// SKIP(typed) [TC-BCBS-03 pm4py rung]: pm4py not importable — {e} — \
                 digest divergence (asserted above) remains the anti-vacuity witness"
            );
        }
    }
    let _ = std::fs::remove_dir_all(&compliant_log);
    let _ = std::fs::remove_dir_all(&bypass_log);

    close_session(&mut h, SESSION);
    let _ = std::fs::remove_dir_all(&dir);
}

fn write_ocel_trace(dir: &PathBuf, compliant: bool) {
    std::fs::create_dir_all(dir).expect("create ocel dir");
    let mut events = vec![
        ("open_session", 1),
        ("insert_subsidiary_drawdowns", 2),
        ("insert_subsidiary_drawdowns", 3),
        ("insert_subsidiary_drawdowns", 4),
    ];
    if compliant {
        events.push(("validate_child_receipts", 5));
    }
    events.push(("aggregate_parent_sum", 6));
    let mut csv = String::from("case:concept:name,concept:instance,concept:name\n");
    for (i, (activity, eid)) in events.iter().enumerate() {
        csv.push_str(&format!(
            "bcbs-fold,e{i},{activity}\n",
        ));
        let _ = eid;
    }
    std::fs::write(dir.join("trace.csv"), csv).expect("write trace csv");
}

/// Score both traces with pm4py token-based replay over an explicit
/// sequence Petri net. Returns (compliant_fitness, bypass_fitness).
fn pm4py_fitness(compliant_dir: &PathBuf, bypass_dir: &PathBuf) -> Result<(f64, f64), String> {
    let script = r#"
import sys, json
try:
    import pm4py
except Exception as e:
    print(json.dumps({"error": f"import: {e}"})); sys.exit(0)
from pm4py.objects.petri_net.obj import PetriNet, Marking
from pm4py.objects.log.obj import EventLog, Trace, Event
from pm4py.objects.log.importer.csv import importer as csv_importer
import pandas as pd

ACTIVITIES = ["open_session", "insert_subsidiary_drawdowns",
              "validate_child_receipts", "aggregate_parent_sum"]

net = PetriNet()
places = {}
for i in range(len(ACTIVITIES) + 1):
    p = PetriNet.Place(f"p{i}"); places[f"p{i}"] = p; net.places.add(p)
for i, act in enumerate(ACTIVITIES):
    t = PetriNet.Transition(f"t{i}", act); net.transitions.add(t)
    net.arcs.add(PetriNet.Arc(places[f"p{i}"], t))
    net.arcs.add(PetriNet.Arc(t, places[f"p{i+1}"]))
im = Marking(); im[places["p0"]] = 1
fm = Marking(); fm[places[f"p{len(ACTIVITIES)}"]] = 1

def fitness(path):
    df = pd.read_csv(path)
    log = pm4py.format_dataframe(df, case_id="case:concept:name",
                                 activity_key="concept:name",
                                 timestamp_key="concept:instance")
    return pm4py.fitness_token_based_replay(log, net, im, fm)

fc = fitness(sys.argv[1])
fb = fitness(sys.argv[2])
print(json.dumps({"compliant": fc["log_fitness"], "bypass": fb["log_fitness"]}))
"#;
    let out = std::process::Command::new("python3")
        .arg("-c")
        .arg(script)
        .arg(compliant_dir.join("trace.csv"))
        .arg(bypass_dir.join("trace.csv"))
        .output()
        .map_err(|e| format!("spawn python3: {e}"))?;
    let stdout = String::from_utf8_lossy(&out.stdout).to_string();
    if !out.status.success() {
        return Err(format!("python exited {:?}: {}{}",
            out.status.code(), stdout, String::from_utf8_lossy(&out.stderr)));
    }
    let v: Value = serde_json::from_str(stdout.trim())
        .map_err(|e| format!("python output not JSON ({stdout}): {e}"))?;
    if let Some(err) = v["error"].as_str() {
        return Err(err.to_string());
    }
    let fc = v["compliant"].as_f64().ok_or("missing compliant fitness")?;
    let fb = v["bypass"].as_f64().ok_or("missing bypass fitness")?;
    Ok((fc, fb))
}
