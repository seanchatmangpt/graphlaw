//! WP P04 — byte-determinism court (lane P04-det, tree /Users/sac/graphlaw).
//!
//! Five legs, all real commands on real collaborators (Chicago: no mocks, no
//! sleeps):
//!
//! 1. **Seal stability** — N=100 iterations, each a FRESH
//!    `GraphLawDuckDB::open_in_memory` engine seeded identically and evaluated
//!    with the same admitted compile: `query_hash`, `receipt_v2_hex`,
//!    `columnar_digest_hex`, and the `result_triples` SerTerm bytes must be
//!    byte-identical every iteration. A BLAKE3 fold-chain over the per-
//!    iteration seals is the anti-vacuity witness: the final chain digest must
//!    differ from the iteration-0 chain state and from all-zero bytes (the
//!    loop is a real function of the 100 seals, not an optimized no-op).
//! 2. **Envelope-minus-clock** — the serialized `ExecutionReceipt` envelope
//!    with `execution_ms`/`consult_ms` redacted to JSON `null` must
//!    re-serialize byte-identically across N=5 fresh-engine runs.
//!    `cached` deliberately STAYS in the envelope: it is a bool and a real
//!    witness of the fresh-engine miss path (`false` every iteration).
//!    DISCLOSED SERDE ORDER: `serde_json::Value`'s map is `BTreeMap`-backed
//!    (serde_json 1.0.151 is resolved WITHOUT the `preserve_order` feature —
//!    its Cargo.lock dependency list is `itoa, memchr, serde, serde_core`, no
//!    `indexmap`), so the redacted re-serialization emits keys in
//!    ALPHABETICAL order. That alphabetical form is the canonical envelope
//!    compared across iterations; the library wire shape (manual
//!    `Serialize` on `ExecutionReceipt`) is untouched.
//! 3. **Version pins** — `h_triples_serterm` / `h_columnar_canon` golden
//!    digests over a frozen 6-triple fixture, minted via the `--nocapture`
//!    mint test below and pasted as consts with provenance. Permutation
//!    tripwires: reversing row order must flip both digests (row order is
//!    preserved in the row-major arm and reverses column/row order in the
//!    column-major arm), and swapping `object_ids` order
//!    (`[a, b]` vs `[b, a]`) must flip a receipt-chain record's chain hash
//!    (the ids fold into the frame via `build_admission_frame`'s meta JSON).
//! 4. **Cross-process identity** — the court self-spawns its own test binary
//!    (`std::env::current_exe`) N=3 times with `GLAW_P04_PROBE_CHILD=1`; each
//!    child computes a PURE-LIBRARY payload (no engine, no ambient state):
//!    `admitted_statement_hash` + `receipt_v2` with empty arms + a 3-record
//!    chain carrying 16 const `object_ids` + an attest `receipt_payload`
//!    rider (deterministic Ed25519). All 3 children must print byte-identical
//!    payloads AND equal the in-process recomputation.
//! 5. **Chain clock duality** — a 5-record ledger stamped by `FixedClock(42)`
//!    built twice yields byte-identical JSONL and validates ok; the
//!    `SystemClock`-stamped version self-verifies and its head differs from
//!    the `FixedClock(42)` head (guaranteed: 42 ns vs epoch-ns stamps fold
//!    into the frame bytes). Two SystemClock builds are NEVER compared —
//!    they could lawfully stamp identical nanoseconds; divergence between
//!    SystemClock builds is asserted nowhere in this court.
//!
//! # Re-mint protocol (leg 3 goldens)
//!
//! When the fixture in [`frozen_fixture`] changes ON PURPOSE (a documented
//! encoding change, never a drift):
//!
//! ```text
//! CARGO_TARGET_DIR=target-arr-P04-det \
//!   cargo test --test determinism_court p04_mint_golden_digests -- --nocapture
//! ```
//!
//! Paste the two printed `GOLDEN_*` candidate lines into the consts below,
//! updating the provenance comment (date, toolchain `rustc -V`, minting
//! command). Any UNPLANNED mismatch of a pasted golden is a drift alarm, not
//! something to re-mint away.

use graphlaw::attest::receipt_fold::{canon_arrow_empty_v1, receipt_v2};
use graphlaw::attest::{
    hex_decode, hex_encode, receipt_payload, sign_bytes, verify_bytes, ReceiptFields, SigningKey,
    TrustedKeys,
};
use graphlaw::duckdb_eval::{
    admitted_statement_hash, canonical_ast_graph_bytes, h_columnar_canon, h_triples_serterm,
    receipt_v2_state_sentinel, GraphLawDuckDB, RdfTriple,
};
use graphlaw::receipt_chain::validator::{Clock, FixedClock, ReceiptValidator, SystemClock};
use graphlaw::receipt_chain::{Andon, ReceiptRecord, RECEIPT_RECORD_VERSION};
use graphlaw::receipt_chain::epoch::SCHEMA_V1;
use graphlaw::sql_ast::{compile_turtle, CompiledQuery, SqlLiteral};
use purrdf::TermValue;

// ---------------------------------------------------------------------------
// Shared constants
// ---------------------------------------------------------------------------

/// Seal-stability iteration count (leg 1).
const N_SEAL_ITERATIONS: usize = 100;
/// Envelope-minus-clock iteration count (leg 2).
const N_ENVELOPE_ITERATIONS: usize = 5;
/// Cross-process child count (leg 4).
const N_CHILDREN: usize = 3;
/// Child-mode environment toggle.
const ENV_CHILD: &str = "GLAW_P04_PROBE_CHILD";
/// Fold epoch for the probe's hand-built v2 envelope.
const P04_EPOCH: u64 = 42;

/// The admitted SELECT document every leg evaluates (identical bytes every
/// call — graph-backed compile, so `H_query = BLAKE3(Canon(G_AST)||P_params)`).
const P04_GRAPH_DOC: &str = r#"@prefix sql: <http://graphlaw.org/sql#> .
@prefix xsd: <http://www.w3.org/2001/XMLSchema#> .
@prefix ex: <http://example.org/p04#> .
ex:q a sql:SelectStatement ;
    sql:statementIndex 1 ;
    sql:fromTable [ sql:tableName "revenue" ] ;
    sql:selectList ( [ sql:columnName "region" ] [ sql:columnName "v" ] ) ."#;

/// Identical seed data for every fresh engine (3 rows, one NULL cell: the
/// row-major stream skips it, 2 columns x 3 rows minus 1 = 5 triples).
const P04_SEED_SQL: &str = "CREATE TABLE revenue (region TEXT, v BIGINT);\n\
     INSERT INTO revenue VALUES ('east', 10), ('west', 20), ('north', NULL);";

/// 16 const object ids for the probe chain (leg 4), 6/5/5 across 3 records.
const OBJ_IDS: [&str; 16] = [
    "urn:p04:obj:00",
    "urn:p04:obj:01",
    "urn:p04:obj:02",
    "urn:p04:obj:03",
    "urn:p04:obj:04",
    "urn:p04:obj:05",
    "urn:p04:obj:06",
    "urn:p04:obj:07",
    "urn:p04:obj:08",
    "urn:p04:obj:09",
    "urn:p04:obj:10",
    "urn:p04:obj:11",
    "urn:p04:obj:12",
    "urn:p04:obj:13",
    "urn:p04:obj:14",
    "urn:p04:obj:15",
];

/// Const stamps for the probe chain records (pure-lib: no clock at all).
const PROBE_CHAIN_TS: [u64; 3] = [111, 222, 333];

// ---------------------------------------------------------------------------
// Leg 3 golden pins — minted per the re-mint protocol header. PROVENANCE:
// minted 2026-10-26 by lane P04-det, rustc 1.98.0 (88d9e12ae 2026-08-18),
// via CARGO_TARGET_DIR=target-arr-P04-det cargo test --test determinism_court
// p04_mint_golden_digests -- --nocapture. The digests are pure functions of
// `frozen_fixture()`; any mismatch against a real re-mint is a drift alarm.
// ---------------------------------------------------------------------------

/// BLAKE3 over the SerTerm row-major JSON encoding of `frozen_fixture()`.
const GOLDEN_H_TRIPLES_SERTERM: &str =
    "248cd97b412685441df24bf44b52461a001f8ee8c70bd0a2c8a16e41b151a324";
/// BLAKE3 over the column-major canonical encoding of `frozen_fixture()`.
const GOLDEN_H_COLUMNAR_CANON: &str =
    "4d51ca3c8cf5bc73780cb8cd3bed526293fa6b3fe34c5ad10cf3f8e8d14e8ff4";

/// Parse a 64-hex-char literal into 32 bytes (court-local, no deps).
fn d32(s: &str) -> [u8; 32] {
    assert_eq!(s.len(), 64, "golden literal must be 64 hex chars, got {}", s.len());
    let mut out = [0u8; 32];
    for (i, b) in out.iter_mut().enumerate() {
        *b = u8::from_str_radix(&s[2 * i..2 * i + 2], 16).expect("valid hex");
    }
    out
}

// ---------------------------------------------------------------------------
// Shared fixtures
// ---------------------------------------------------------------------------

/// Admitted compile of the fixed SELECT (identical every call).
fn compile_p04_select() -> CompiledQuery {
    compile_turtle(P04_GRAPH_DOC).expect("P04 SELECT must compile")
}

/// Fresh in-memory engine with the fixed seed table.
fn engine_with_table() -> GraphLawDuckDB {
    let db = GraphLawDuckDB::open_in_memory().expect("in-memory engine");
    db.raw_connection().execute_batch(P04_SEED_SQL).expect("seed table");
    db
}

/// One fresh engine, one eval of the fixed compile — the seal-under-test.
fn seal_once(iri: &str) -> graphlaw::duckdb_eval::ExecutionReceipt {
    let db = engine_with_table();
    let compiled = compile_p04_select();
    db.eval_ast_query(iri, &compiled, "urn:p04:s", "urn:p04:p")
        .expect("P04 eval")
}

/// The frozen 6-triple fixture (leg 3): 2 predicates x 3 rows. Row-major and
/// column-major arms both group deterministically over these exact terms.
fn frozen_fixture() -> Vec<RdfTriple> {
    const XSD_INT: &str = "http://www.w3.org/2001/XMLSchema#integer";
    let triple = |row: &str, pred: &str, obj: TermValue| RdfTriple {
        subject: format!("urn:p04:row/{row}"),
        predicate: format!("urn:p04:p/{pred}"),
        object: obj,
    };
    vec![
        triple("1", "alpha", TermValue::simple_literal("one")),
        triple("1", "beta", TermValue::typed_literal("10", XSD_INT)),
        triple("2", "alpha", TermValue::simple_literal("two")),
        triple("2", "beta", TermValue::typed_literal("20", XSD_INT)),
        triple("3", "alpha", TermValue::simple_literal("three")),
        triple("3", "beta", TermValue::typed_literal("30", XSD_INT)),
    ]
}

/// Build one lawfully-chained `ReceiptRecord` (chain hash recomputed and
/// stamped). `object_ids` order is hash-relevant (leg 3 tripwire).
fn chain_record(instruction_id: u64, ts_ns: u64, object_ids: &[String], prev: &[u8; 32]) -> ReceiptRecord {
    let mut rec = ReceiptRecord {
        version: RECEIPT_RECORD_VERSION,
        instruction_id,
        activity_idx: 0,
        activity: None,
        node_kind: 0,
        ts_ns,
        duration_ms: None,
        origin: None,
        payload_hash_hex: hex::encode(
            blake3::hash(format!("P04-RECORD-{instruction_id}").as_bytes()).as_bytes(),
        ),
        prev_chain_hash_hex: hex::encode(prev),
        chain_hash_hex: String::new(),
        andon: Andon::Green,
        obligation_count: 0,
        object_ids: object_ids.to_vec(),
        signature_hex: None,
        schema: SCHEMA_V1.to_string(),
        v2: None,
        chain_rule: None,
    };
    let h = rec.recompute_chain_hash().expect("chain recompute");
    rec.chain_hash_hex = hex::encode(h);
    rec
}

fn strs(items: &[&str]) -> Vec<String> {
    items.iter().map(|s| (*s).to_string()).collect()
}

// ---------------------------------------------------------------------------
// Leg 4 probe payload (pure library — no engine, no clock, no ambient state)
// ---------------------------------------------------------------------------

struct PureLibPayload {
    h_query: [u8; 32],
    v2_empty: [u8; 32],
    chain_head: [u8; 32],
    attest_payload: String,
    attest_att_json: String,
    composite: [u8; 32],
}

/// The pure-library determinism probe. `param_tag` feeds the admitted-statement
/// params so the anti-vacuity leg can perturb one input and watch the
/// composite move.
fn pure_lib_payload(param_tag: &str) -> PureLibPayload {
    // (a) admitted-statement receipt over the fixed graph + fixed params.
    let canon = canonical_ast_graph_bytes(P04_GRAPH_DOC).expect("AST graph canonicalizes");
    let params = [SqlLiteral::String(param_tag.to_string())];
    let h_query = admitted_statement_hash(&canon, &params);

    // (b) receipt_v2 with EMPTY arms: sentinel H_pre/H_post, empty-encoding
    //     H_triples, domain-null H_columnar.
    let sentinel = receipt_v2_state_sentinel();
    let v2_empty = receipt_v2(
        &h_query,
        sentinel,
        sentinel,
        &h_triples_serterm(&[]),
        &canon_arrow_empty_v1(),
        P04_EPOCH,
    );

    // (c) 3-record chain carrying the 16 const object ids (6/5/5 split).
    let splits: [&[&str]; 3] = [&OBJ_IDS[0..6], &OBJ_IDS[6..11], &OBJ_IDS[11..16]];
    let mut prev = [0u8; 32];
    for (i, ids) in splits.iter().enumerate() {
        let rec = chain_record((i + 1) as u64, PROBE_CHAIN_TS[i], &strs(ids), &prev);
        prev = rec.chain_hash().expect("probe record chain hash");
    }
    let chain_head = prev;

    // (d) attest rider: canonical payload + deterministic Ed25519 signature.
    let fields = ReceiptFields {
        parent: "urn:p04:parent",
        child: "urn:p04:child",
        step: "p04:probe",
        authority: "duckdb",
        revision: "26.10.26",
        added: 3,
        lease_id: None,
        plan_blake3: None,
        subject_blake3: Some("blake3:p04-subject"),
    };
    let attest_payload = receipt_payload(&fields);
    let key = SigningKey::from_seed([7u8; 32]);
    let att = sign_bytes(&key, attest_payload.as_bytes());
    let mut trusted = TrustedKeys::new();
    trusted.insert(key.verifying_key());
    verify_bytes(attest_payload.as_bytes(), &att, &trusted)
        .expect("probe rider must self-verify before it is printed");
    let attest_att_json = att.to_json();

    // Composite fold (fixed-size inputs + the two variable strings; unambiguous).
    let mut h = blake3::Hasher::new();
    h.update(b"P04-PROBE-V1");
    h.update(&h_query);
    h.update(&v2_empty);
    h.update(&chain_head);
    h.update(attest_payload.as_bytes());
    h.update(attest_att_json.as_bytes());
    let composite = *h.finalize().as_bytes();

    PureLibPayload {
        h_query,
        v2_empty,
        chain_head,
        attest_payload,
        attest_att_json,
        composite,
    }
}

/// The canonical printed form of the probe (child stdout == parent parse).
fn render_probe(p: &PureLibPayload) -> String {
    format!(
        "h_query={}\nreceipt_v2_empty={}\nchain_head={}\nattest_payload={}\nattest_att={}\ncomposite={}\n",
        hex_encode(&p.h_query),
        hex_encode(&p.v2_empty),
        hex_encode(&p.chain_head),
        p.attest_payload,
        p.attest_att_json,
        hex_encode(&p.composite),
    )
}

const PROBE_BEGIN: &str = "P04_PROBE_BEGIN\n";
const PROBE_END: &str = "\nP04_PROBE_END";

/// Extract the probe block from libtest stdout (bracketed by markers so the
/// harness noise around it is irrelevant).
fn parse_probe(stdout: &str) -> String {
    let start = stdout
        .find(PROBE_BEGIN)
        .unwrap_or_else(|| panic!("no P04_PROBE_BEGIN in stdout:\n{stdout}"));
    let body = &stdout[start + PROBE_BEGIN.len()..];
    let end = body
        .rfind(PROBE_END)
        .unwrap_or_else(|| panic!("no P04_PROBE_END in stdout:\n{stdout}"));
    body[..end].to_string()
}

// ---------------------------------------------------------------------------
// Leg 1 — seal stability across 100 fresh engines
// ---------------------------------------------------------------------------

#[test]
fn p04_leg1_seal_stability_100_fresh_engines() {
    let iri = "urn:p04:leg1";
    let mut fold = [0u8; 32];
    let mut fold_0: Option<[u8; 32]> = None;
    let mut baseline: Option<(
        [u8; 32],
        Option<String>,
        Option<String>,
        Vec<u8>,
    )> = None;

    for i in 0..N_SEAL_ITERATIONS {
        let r = seal_once(iri);
        assert!(!r.cached, "iteration {i}: fresh engine must be a cache miss");
        let v2 = r.receipt_v2_hex.clone().expect("receipt_v2_hex present");
        let col = r.columnar_digest_hex.clone().expect("columnar_digest_hex present");
        let triples = serde_json::to_vec(&r.result_triples).expect("SerTerm triples bytes");

        let seal = (r.query_hash, Some(v2.clone()), Some(col.clone()), triples);
        match &baseline {
            None => baseline = Some(seal),
            Some(b) => {
                assert_eq!(r.query_hash, b.0, "iteration {i}: query_hash drifted");
                assert_eq!(
                    Some(v2.clone()),
                    b.1,
                    "iteration {i}: receipt_v2_hex drifted"
                );
                assert_eq!(
                    Some(col.clone()),
                    b.2,
                    "iteration {i}: columnar_digest_hex drifted"
                );
                assert_eq!(
                    serde_json::to_vec(&r.result_triples).expect("triples bytes"),
                    b.3,
                    "iteration {i}: result_triples bytes drifted"
                );
            }
        }

        // BLAKE3 fold-chain over every seal (anti-vacuity witness).
        let mut hasher = blake3::Hasher::new();
        hasher.update(&fold);
        hasher.update(&hex_decode(&v2).expect("v2 hex decodes"));
        fold = *hasher.finalize().as_bytes();
        if i == 0 {
            fold_0 = Some(fold);
        }
    }

    assert_eq!(baseline.as_ref().expect("at least one seal").1.is_some(), true);
    // Anti-vacuity: the chain is a real function of all 100 seals.
    assert_ne!(
        fold,
        fold_0.expect("iteration-0 chain state"),
        "fold chain must transform past iteration 0"
    );
    assert_ne!(fold, [0u8; 32], "fold chain must be non-trivial");
}

// ---------------------------------------------------------------------------
// Leg 2 — envelope minus the clock
// ---------------------------------------------------------------------------

#[test]
fn p04_leg2_envelope_minus_clock_byte_equal() {
    let iri = "urn:p04:leg2";
    let mut canonical: Option<String> = None;

    for i in 0..N_ENVELOPE_ITERATIONS {
        let r = seal_once(iri);
        let raw = serde_json::to_string(&r).expect("envelope serializes");
        let mut v: serde_json::Value = serde_json::from_str(&raw).expect("envelope is JSON");

        // The clock fields are real numbers before redaction.
        assert!(
            v.get("execution_ms").expect("execution_ms key").is_number(),
            "iteration {i}: execution_ms must be a JSON number pre-redaction"
        );
        assert!(
            v.get("consult_ms").expect("consult_ms key").is_number(),
            "iteration {i}: consult_ms must be a JSON number pre-redaction"
        );
        // `cached` STAYS — it is a bool (fresh-engine miss: false).
        assert_eq!(
            v.get("cached").expect("cached key"),
            &serde_json::Value::Bool(false),
            "iteration {i}: fresh-engine envelope must carry cached:false"
        );

        // REDACT: execution_ms/consult_ms -> Null, re-serialize.
        // DISCLOSED SERDE ORDER: serde_json::Value maps are BTreeMap-backed
        // (no `preserve_order` feature in the resolved serde_json 1.0.151),
        // so this re-serialization emits keys in ALPHABETICAL order. That
        // alphabetical form is the canonical envelope under comparison.
        v["execution_ms"] = serde_json::Value::Null;
        v["consult_ms"] = serde_json::Value::Null;
        let redacted = serde_json::to_string(&v).expect("redacted envelope serializes");

        match &canonical {
            None => {
                canonical = Some(redacted.clone());
                // Anti-vacuity tripwire: the comparison is content-sensitive —
                // nulling a DIFFERENT field (the v2 seal) must move the bytes.
                let mut w = v.clone();
                w["receipt_v2_hex"] = serde_json::Value::Null;
                let mutated = serde_json::to_string(&w).expect("mutated envelope serializes");
                assert_ne!(
                    redacted, mutated,
                    "nulling receipt_v2_hex must change the canonical envelope"
                );
                // And the redaction itself is real: the seal arms still ride.
                assert!(v.get("receipt_v2_hex").expect("v2 key").is_string());
                assert!(v.get("columnar_digest_hex").expect("columnar key").is_string());
            }
            Some(c) => assert_eq!(
                &redacted, c,
                "iteration {i}: clock-redacted envelope drifted byte-wise"
            ),
        }
    }
}

// ---------------------------------------------------------------------------
// Leg 3 — version pins + permutation tripwires (+ the --nocapture mint)
// ---------------------------------------------------------------------------

/// MINT ONLY (never asserted): prints the golden candidates for
/// `frozen_fixture()`. Run per the re-mint protocol header.
#[test]
fn p04_mint_golden_digests() {
    let fixture = frozen_fixture();
    println!("GOLDEN_H_TRIPLES_SERTERM={}", hex_encode(&h_triples_serterm(&fixture)));
    println!("GOLDEN_H_COLUMNAR_CANON={}", hex_encode(&h_columnar_canon(&fixture)));
}

/// The pinned digests + both permutation tripwires.
#[test]
fn p04_leg3_golden_pins_and_permutation_tripwires() {
    let fixture = frozen_fixture();

    // Version pins.
    assert_eq!(
        h_triples_serterm(&fixture),
        d32(GOLDEN_H_TRIPLES_SERTERM),
        "h_triples_serterm drifted from its pinned golden (re-mint only with a documented encoding change)"
    );
    assert_eq!(
        h_columnar_canon(&fixture),
        d32(GOLDEN_H_COLUMNAR_CANON),
        "h_columnar_canon drifted from its pinned golden (re-mint only with a documented encoding change)"
    );
    // The two arms are genuinely different byte encodings of the same rows.
    assert_ne!(
        h_triples_serterm(&fixture),
        h_columnar_canon(&fixture),
        "row-major and column-major arms must not collide on the fixture"
    );

    // Tripwire 1: row order is hash-relevant in BOTH arms.
    let mut reversed = fixture.clone();
    reversed.reverse();
    assert_ne!(
        h_triples_serterm(&reversed),
        d32(GOLDEN_H_TRIPLES_SERTERM),
        "row order must change the row-major digest"
    );
    assert_ne!(
        h_columnar_canon(&reversed),
        d32(GOLDEN_H_COLUMNAR_CANON),
        "row order must change the column-major digest"
    );

    // Tripwire 2: object_ids order is chain-hash-relevant.
    let ids_a = strs(&["urn:p04:sw:a", "urn:p04:sw:b"]);
    let ids_b: Vec<String> = ids_a.iter().rev().cloned().collect();
    let rec_ab = chain_record(1, 42, &ids_a, &[0u8; 32]);
    let rec_ba = chain_record(1, 42, &ids_b, &[0u8; 32]);
    assert_ne!(
        rec_ab.chain_hash_hex, rec_ba.chain_hash_hex,
        "[a,b] vs [b,a] object_ids must chain to different hashes"
    );
    // The swap is a DIFFERENT lawful ledger entry, not a corruption: each
    // record's stored chain hash recomputes under its own fields.
    assert!(matches!(
        rec_ab.verify_chain().expect("verify ab"),
        graphlaw::receipt_chain::ChainVerification::Verified(_)
    ));
    assert!(matches!(
        rec_ba.verify_chain().expect("verify ba"),
        graphlaw::receipt_chain::ChainVerification::Verified(_)
    ));
}

// ---------------------------------------------------------------------------
// Leg 4 — cross-process probe identity
// ---------------------------------------------------------------------------

/// Child mode: prints the pure-lib probe when (and only when) invoked by the
/// parent test. Inert in ordinary suite runs (env unset -> immediate pass).
#[test]
fn p04_probe_child() {
    if std::env::var_os(ENV_CHILD).is_none() {
        return;
    }
    let rendered = render_probe(&pure_lib_payload("P04"));
    println!("{PROBE_BEGIN}{rendered}{PROBE_END}");
}

#[test]
fn p04_leg4_cross_process_probe_identity() {
    let in_process = render_probe(&pure_lib_payload("P04"));
    let exe = std::env::current_exe().expect("current_exe resolves to this test binary");

    let mut children: Vec<String> = Vec::new();
    for child in 0..N_CHILDREN {
        let out = std::process::Command::new(&exe)
            .args(["p04_probe_child", "--exact", "--nocapture"])
            .env(ENV_CHILD, "1")
            .output()
            .unwrap_or_else(|e| panic!("child {child}: spawn failed: {e}"));
        assert!(
            out.status.success(),
            "child {child}: harness exited {:?}\nstdout:\n{}\nstderr:\n{}",
            out.status.code(),
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr),
        );
        children.push(parse_probe(&String::from_utf8_lossy(&out.stdout)));
    }

    assert_eq!(children.len(), N_CHILDREN);
    for (i, payload) in children.iter().enumerate() {
        assert_eq!(
            payload, &in_process,
            "child {i} payload differs from the in-process recomputation:\n{payload}\n---\n{in_process}"
        );
    }
    for pair in children.windows(2) {
        assert_eq!(
            pair[0], pair[1],
            "cross-process payloads diverged:\n{}\n---\n{}",
            pair[0], pair[1]
        );
    }

    // Anti-vacuity: one perturbed input moves the composite (the fold is a
    // real function of the payload bytes, not a constant).
    let perturbed = render_probe(&pure_lib_payload("P04-ALTERNATE"));
    assert_ne!(
        perturbed, in_process,
        "a different param_tag must produce a different probe"
    );
}

// ---------------------------------------------------------------------------
// Leg 5 — chain clock duality
// ---------------------------------------------------------------------------

/// Build the fixed 5-record ledger + its JSONL wire bytes for the given stamps.
fn build_p04_ledger(stamps: [u64; 5]) -> (Vec<ReceiptRecord>, String) {
    let mut prev = [0u8; 32];
    let mut records = Vec::new();
    let mut jsonl = String::new();
    for (i, ts) in stamps.iter().enumerate() {
        let rec = chain_record((i + 1) as u64, *ts, &strs(&OBJ_IDS[0..5]), &prev);
        jsonl.push_str(&serde_json::to_string(&rec).expect("record JSON"));
        jsonl.push('\n');
        prev = rec.chain_hash().expect("chain hash");
        records.push(rec);
    }
    (records, jsonl)
}

#[test]
fn p04_leg5_chain_clock_duality() {
    // FixedClock leg: identical stamps -> byte-identical JSONL, validator ok.
    let fixed_clock = FixedClock(42);
    let (fixed_a, jsonl_a) = build_p04_ledger([fixed_clock.now_ns(); 5]);
    let (_, jsonl_b) = build_p04_ledger([fixed_clock.now_ns(); 5]);
    assert_eq!(
        jsonl_a, jsonl_b,
        "two FixedClock(42) builds must be byte-identical on the wire"
    );
    let verdict = ReceiptValidator::validate(&fixed_a, &fixed_clock);
    assert!(verdict.ok, "FixedClock(42) ledger must validate: {verdict:?}");
    assert_eq!(verdict.records_checked, 5);

    // SystemClock leg: epoch-ns stamps, max-guarded non-decreasing (observe,
    // never sleep). Self-verifies; head differs from the FixedClock(42) head
    // GUARANTEED (42 ns vs epoch-ns stamps fold into the 99-byte frame).
    let sys = SystemClock;
    let mut stamps = [0u64; 5];
    let mut prev_ts = 0u64;
    for slot in stamps.iter_mut() {
        prev_ts = prev_ts.max(sys.now_ns());
        *slot = prev_ts;
    }
    let (system, jsonl_sys) = build_p04_ledger(stamps);
    let sys_verdict = ReceiptValidator::validate(&system, &sys);
    assert!(sys_verdict.ok, "SystemClock ledger must self-verify: {sys_verdict:?}");

    let head_fixed = fixed_a[4].chain_hash().expect("fixed head");
    let head_sys = system[4].chain_hash().expect("system head");
    assert_ne!(
        head_fixed, head_sys,
        "the stamp class is chain-hash-relevant: FixedClock(42) head cannot equal the epoch-ns head"
    );
    // The two builds' wire forms differ exactly because the stamps differ —
    // but two SystemClock builds are NEVER compared (they could lawfully
    // stamp identical ns; see the mission note and header disclosure).
    assert_ne!(jsonl_a, jsonl_sys);

    // Anti-vacuity: the validator must refuse a tampered ledger (one flipped
    // hex char in the middle record's payload hash is a tamper, not a pass).
    let mut tampered = fixed_a.clone();
    let c = tampered[2].payload_hash_hex.chars().next().expect("nonempty");
    let flip = if c == 'a' { 'b' } else { 'a' };
    tampered[2].payload_hash_hex.replace_range(0..1, &flip.to_string());
    let bad = ReceiptValidator::validate(&tampered, &fixed_clock);
    assert!(!bad.ok, "tampered ledger must fail validation: {bad:?}");
}
