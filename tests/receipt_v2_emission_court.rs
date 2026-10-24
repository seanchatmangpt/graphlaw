//! Court for the receipt_v2 production emission (ADR-CANONARROW Phase 3,
//! `docs/jira/v26.10.22/ADR-canonarrow-dual-witness-egress.md` §3.1; OQ
//! resolutions: OQ-1(a) H_pre/H_post state sentinels, OQ-2 H_triples over
//! the SerTerm canonical form, OQ-4 dual-receipt additive).
//!
//! Chicago discipline: every execution arm runs a real frozen-duckdb
//! engine (`GraphLawDuckDB::open_in_memory`), real tables, real table
//! readbacks — no mocks. The v2 envelope on each receipt is verified
//! against a FULLY INDEPENDENT recomputation: the court re-derives every
//! fold input from the receipt's own public fields, rebuilds the byte
//! concatenation by hand (bypassing `attest::receipt_fold::receipt_v2`
//! entirely), and compares. B3sum-minted pins are out-of-band witnesses.
//!
//! Court responsibilities:
//! 1. v1 byte-preservation: pre-Phase-3 receipts serialize to exactly the
//!    pre-Phase-3 wire shape, and pre-Phase-3 JSON reads back.
//! 2. v2 present on tabular results, self-verifying from the receipt's own
//!    fields; cache-hit rides the stored seal; deterministic across
//!    independent engines.
//! 3. Sentinel arms on non-tabular results (`CANON_ARROW_EMPTY_V1`
//!    columnar arm, empty-encoding triples arm).
//! 4. The divergent-graph hash override re-seals the envelope.
//! 5. Domain digests pinned: state sentinel and empty-encoding triples arm.

use graphlaw::attest::receipt_fold::{receipt_v1, receipt_v2};
use graphlaw::duckdb_eval::{
    h_columnar_canon, h_triples_serterm, receipt_v2_state_sentinel, ExecutionReceipt,
    GraphLawDuckDB, RdfTriple,
};
use graphlaw::sql_ast::compile_turtle;
use purrdf::TermValue;

/// Independent hex helper (the `hex` crate is not a dev-dependency).
fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{:02x}", b)).collect()
}

/// Parse a 64-hex-char literal into a 32-byte digest.
fn d32(s: &str) -> [u8; 32] {
    assert_eq!(s.len(), 64, "vector literal must be 64 hex chars");
    let mut out = [0u8; 32];
    for (i, b) in out.iter_mut().enumerate() {
        *b = u8::from_str_radix(&s[2 * i..2 * i + 2], 16).expect("valid hex");
    }
    out
}

// --- Pinned out-of-band vectors (b3sum, 2026-10-24) --------------------------

/// BLAKE3(b"GLAW_RECEIPT_V2_STATE_SENTINEL_V1") — b3sum over the literal
/// 32-byte string. The OQ-1(a) state sentinel folded as H_pre/H_post.
const SENTINEL_PIN: &str = "7ac44b45bd8c91a1ee1a1035ba43494bd08054231a4ea6dcb9175d8cdfc04593";
/// BLAKE3(b"[]") — b3sum over the two-byte canonical empty-array encoding.
/// The OQ-2 H_triples arm on non-tabular results (real digest of the empty
/// SerTerm canonical encoding, not a sentinel).
const EMPTY_TRIPLES_PIN: &str = "d53d18c23212ea7b6300594bb89bce60218f6eff2b9d628b8cc42d3e79bbd5ab";
/// BLAKE3(b"CANON_ARROW_EMPTY_V1") — same settled b3sum pin as
/// tests/receipt_v2_court.rs (ADR §3.1 domain null digest).
const EMPTY_COLUMNAR_PIN: &str = "db2da6fc6ec068a661a7028f220524ea6fad4b3cbf7f72df336dd99fd44f7b0c";

// --- Independent recomputation helpers ---------------------------------------

/// Fully independent `H_columnar` recomputation from the receipt's public
/// fields: group the result triples by predicate in first-appearance
/// (declared projection) order; per column, u64-LE byte length prefix then
/// the serde JSON array of that column's triples. Re-implemented here from
/// the specification, not calling `h_columnar_canon`.
fn independent_h_columnar(triples: &[RdfTriple]) -> [u8; 32] {
    if triples.is_empty() {
        return d32(EMPTY_COLUMNAR_PIN);
    }
    let mut order: Vec<String> = Vec::new();
    let mut columns: std::collections::HashMap<String, Vec<&RdfTriple>> =
        std::collections::HashMap::new();
    for t in triples {
        if !columns.contains_key(&t.predicate) {
            order.push(t.predicate.clone());
        }
        columns.entry(t.predicate.clone()).or_default().push(t);
    }
    let mut hasher = blake3::Hasher::new();
    for pred in &order {
        let column = columns[pred].clone();
        let buf = serde_json::to_vec(&column).expect("plain-data serialize");
        hasher.update(&(buf.len() as u64).to_le_bytes());
        hasher.update(&buf);
    }
    *hasher.finalize().as_bytes()
}

/// Fully independent `receipt_v2` fold: hand-built byte concatenation
/// `"GLAW_RECEIPT_V2" || H_query || H_pre || H_post || H_triples ||
/// H_columnar || le64(epoch)` — bypasses `attest::receipt_fold::receipt_v2`
/// so the emission is verified against the ADR byte law itself.
fn independent_v2_fold(
    h_query: &[u8; 32],
    h_pre: &[u8; 32],
    h_post: &[u8; 32],
    h_triples: &[u8; 32],
    h_columnar: &[u8; 32],
    epoch: u64,
) -> [u8; 32] {
    let mut h = blake3::Hasher::new();
    h.update(b"GLAW_RECEIPT_V2");
    h.update(h_query);
    h.update(h_pre);
    h.update(h_post);
    h.update(h_triples);
    h.update(h_columnar);
    h.update(&epoch.to_le_bytes());
    *h.finalize().as_bytes()
}

/// Independent `H_triples` recomputation: BLAKE3 over the whole SerTerm
/// canonical JSON array, built and hashed in one buffer (a different code
/// path from the streamed `h_triples_serterm`).
fn independent_h_triples(triples: &[RdfTriple]) -> [u8; 32] {
    let buf = serde_json::to_vec(&triples).expect("plain-data serialize");
    *blake3::hash(&buf).as_bytes()
}

/// Verify the Phase 3 fields on `receipt` against a fully independent
/// recomputation (all fold inputs re-derived from the receipt's own public
/// fields; the fold rebuilt by hand). `epoch` is the C1 plan-cache epoch the
/// receipt sealed under.
fn assert_self_verifying(receipt: &ExecutionReceipt, epoch: u64) {
    let v2_hex = receipt
        .receipt_v2_hex
        .as_ref()
        .expect("Phase 3 receipt must carry receipt_v2_hex");
    let col_hex = receipt
        .columnar_digest_hex
        .as_ref()
        .expect("Phase 3 receipt must carry columnar_digest_hex");

    let h_triples = independent_h_triples(&receipt.result_triples);
    let h_columnar = independent_h_columnar(&receipt.result_triples);
    assert_eq!(
        col_hex,
        &hex(&h_columnar),
        "columnar_digest_hex does not match the independent columnar arm"
    );
    let sentinel = d32(SENTINEL_PIN);
    let v2 = independent_v2_fold(
        &receipt.query_hash,
        &sentinel,
        &sentinel,
        &h_triples,
        &h_columnar,
        epoch,
    );
    assert_eq!(
        v2_hex,
        &hex(&v2),
        "receipt_v2_hex does not match the independent hand-built fold"
    );
}

// --- Fixture: real engine + real table ---------------------------------------

/// Admitted compile of `SELECT region, v FROM <table>` (sql:SelectStatement).
fn compile_select(table: &str) -> graphlaw::sql_ast::CompiledQuery {
    let doc = format!(
        r#"@prefix sql: <http://graphlaw.org/sql#> .
@prefix xsd: <http://www.w3.org/2001/XMLSchema#> .
@prefix ex: <http://example.org/v2e#> .
ex:q a sql:SelectStatement ;
    sql:statementIndex 1 ;
    sql:fromTable [ sql:tableName "{table}" ] ;
    sql:selectList ( [ sql:columnName "region" ] [ sql:columnName "v" ] ) ."#
    );
    compile_turtle(&doc).unwrap_or_else(|e| panic!("{table}: admitted compile refused: {e}"))
}

/// Fresh engine with a real 3-row table `revenue(region TEXT, v BIGINT)`,
/// one NULL cell in the last row (partial-cell adversarial case: the
/// row-major stream skips nulls, the columnar arm groups what exists).
fn engine_with_table() -> GraphLawDuckDB {
    let db = GraphLawDuckDB::open_in_memory().expect("in-memory engine");
    db.raw_connection()
        .execute_batch(
            "CREATE TABLE revenue (region TEXT, v BIGINT);\n
             INSERT INTO revenue VALUES ('east', 10), ('west', 20), ('north', NULL);",
        )
        .expect("seed table");
    db
}

// --- (1) v1 byte-preservation -------------------------------------------------

/// Pre-Phase-3 receipts (all additive fields `None`) serialize to exactly
/// the pre-Phase-3 wire shape: the 8 v1 keys, in v1 order, nothing appended
/// — and that same JSON reads back losslessly with the Phase 3 fields
/// defaulted to `None`.
#[test]
fn v1_wire_shape_is_byte_preserved() {
    let receipt = ExecutionReceipt {
        query_iri: "urn:v2e:q".into(),
        query_hash: [0u8; 32],
        sql_statement: "SELECT 1".into(),
        rows_emitted: 0,
        execution_ms: 1.5,
        result_triples: vec![],
        cached: false,
        consult_ms: 0.0,
        artifact_digest_hex: None,
        artifact_bytes: None,
        artifact_warning: None,
        columnar_digest_hex: None,
        receipt_v2_hex: None,
    };
    let got = serde_json::to_string(&receipt).expect("serialize");
    // The exact v1 wire shape: 8 keys, fixed order, pinned values. The
    // query_hash expansion is mechanical (32 zero bytes); the load-bearing
    // pin is the key set/order and the absence of any Phase 3 key.
    let expected = format!(
        r#"{{"query_iri":"urn:v2e:q","query_hash":[{}],"sql_statement":"SELECT 1","rows_emitted":0,"execution_ms":1.5,"result_triples":[],"cached":false,"consult_ms":0.0}}"#,
        vec!["0"; 32].join(",")
    );
    assert_eq!(got, expected, "v1 wire shape drifted — pre-Phase-3 consumers would see new keys");

    // The same JSON reads back losslessly, additive fields defaulted.
    let back: ExecutionReceipt = serde_json::from_str(&got).expect("deserialize");
    assert_eq!(back, receipt, "round-trip must be lossless");
    assert!(back.columnar_digest_hex.is_none() && back.receipt_v2_hex.is_none());
}

/// A receipt carrying the Phase 3 fields rides them on the wire exactly
/// when present (12 keys), and round-trips losslessly.
#[test]
fn phase3_keys_ride_the_wire_only_when_present() {
    let receipt = ExecutionReceipt {
        query_iri: "urn:v2e:q".into(),
        query_hash: [9u8; 32],
        sql_statement: "SELECT 2".into(),
        rows_emitted: 1,
        execution_ms: 2.0,
        result_triples: vec![],
        cached: false,
        consult_ms: 0.0,
        artifact_digest_hex: Some("aa".repeat(32)),
        artifact_bytes: Some(64),
        artifact_warning: None,
        columnar_digest_hex: Some("bb".repeat(32)),
        receipt_v2_hex: Some("cc".repeat(32)),
    };
    let got = serde_json::to_string(&receipt).expect("serialize");
    // Exact wire key set: 12 keys when every optional field is Some.
    let wire: serde_json::Value = serde_json::from_str(&got).expect("wire is JSON");
    let keys = wire.as_object().expect("wire is an object").clone();
    assert_eq!(keys.len(), 12, "12 wire keys when all optional fields ride");
    for key in [
        "query_iri",
        "query_hash",
        "sql_statement",
        "rows_emitted",
        "execution_ms",
        "result_triples",
        "cached",
        "consult_ms",
        "artifact_digest_hex",
        "artifact_bytes",
        "columnar_digest_hex",
        "receipt_v2_hex",
    ] {
        assert!(keys.contains_key(key), "missing wire key {key}");
    }
    let back: ExecutionReceipt = serde_json::from_str(&got).expect("deserialize");
    assert_eq!(back, receipt, "round-trip must be lossless");
}

// --- (2) v2 present on tabular results ---------------------------------------

/// Real engine, real table, real tabular result: the receipt carries a v2
/// seal that self-verifies against a fully independent hand-built fold, the
/// columnar arm matches the independent column-major recomputation, v2
/// differs from the v1 fold over the same inputs (OQ-4 domain separation in
/// production), a cache hit rides the stored seal, and the seal is
/// deterministic across independent engines.
#[test]
fn v2_present_on_tabular_results_and_self_verifies() {
    let db = engine_with_table();
    let compiled = compile_select("revenue");
    let epoch_pre = db.shared_cache().epoch();
    let receipt = db
        .eval_ast_query("urn:v2e:tabular", &compiled, "urn:v2e:s", "urn:v2e:p")
        .expect("tabular eval");

    // Real readback first (the execution actually happened).
    assert_eq!(receipt.rows_emitted, 3, "3 rows materialized");
    assert_eq!(receipt.result_triples.len(), 5, "2 columns x 3 rows, one NULL cell skipped");
    let v_cell = receipt
        .result_triples
        .iter()
        .find(|t| t.predicate == "urn:v2e:p/v")
        .expect("v column projected");
    assert!(
        matches!(&v_cell.object, TermValue::Literal { lexical_form, .. } if lexical_form == "20" || lexical_form == "10"),
        "integer cell readback"
    );

    // Phase 3 fields present and self-verifying.
    assert!(receipt.receipt_v2_hex.is_some(), "v2 must be present on tabular results");
    assert!(receipt.columnar_digest_hex.is_some());
    assert!(!receipt.cached);
    assert_self_verifying(&receipt, epoch_pre);
    // Non-mutating statement: the C1 epoch did not move, so the receipt is
    // verifiable from its own fields plus the live cache epoch.
    assert_eq!(db.shared_cache().epoch(), epoch_pre, "SELECT must not bump");

    // OQ-4: v2 differs from the v1 fold over the same production inputs.
    let h_triples = h_triples_serterm(&receipt.result_triples);
    let h_columnar = h_columnar_canon(&receipt.result_triples);
    let sentinel = receipt_v2_state_sentinel();
    let v2 = receipt_v2(
        &receipt.query_hash,
        sentinel,
        sentinel,
        &h_triples,
        &h_columnar,
        epoch_pre,
    );
    let v1 = receipt_v1(
        &receipt.query_hash,
        sentinel,
        sentinel,
        &h_triples,
        epoch_pre,
    );
    assert_ne!(v2, v1, "domain separation failed on production inputs");
    assert_eq!(receipt.receipt_v2_hex.as_deref(), Some(hex(&v2).as_str()));

    // Cache hit: the consult path rides the stored seal unchanged.
    let hit = db
        .eval_ast_query("urn:v2e:tabular", &compiled, "urn:v2e:s", "urn:v2e:p")
        .expect("cache-hit eval");
    assert!(hit.cached, "second eval of a cacheable SELECT must hit");
    assert_eq!(hit.receipt_v2_hex, receipt.receipt_v2_hex, "cached seal rides");
    assert_eq!(hit.columnar_digest_hex, receipt.columnar_digest_hex);
    assert_eq!(hit.result_triples, receipt.result_triples);
}

/// The v2 seal is deterministic across independent engines: two fresh
/// engines, same seed data, same admitted compile — identical seal (the
/// fold excludes wall-clock, so execution time cannot leak into identity).
#[test]
fn v2_is_deterministic_across_independent_engines() {
    let compiled = compile_select("revenue");
    let seal = |iri: &str| -> ExecutionReceipt {
        let db = engine_with_table();
        let epoch_pre = db.shared_cache().epoch();
        let r = db
            .eval_ast_query(iri, &compiled, "urn:v2e:s", "urn:v2e:p")
            .expect("eval");
        assert_self_verifying(&r, epoch_pre);
        r
    };
    let a = seal("urn:v2e:engine-a");
    let b = seal("urn:v2e:engine-b");
    assert_ne!(a.query_iri, b.query_iri, "independent engines");
    assert_eq!(
        a.receipt_v2_hex, b.receipt_v2_hex,
        "same admitted statement + data must seal identically"
    );
    assert_eq!(a.columnar_digest_hex, b.columnar_digest_hex);
}

// --- (3) sentinel arms on non-tabular results --------------------------------

/// Non-tabular result (SELECT over a real empty table): the columnar arm
/// rides the ADR §3.1 domain null digest `CANON_ARROW_EMPTY_V1`, the
/// triples arm is the real digest of the empty canonical encoding
/// (`BLAKE3(b"[]")`), and the v2 seal self-verifies over those sentinel
/// arms.
#[test]
fn sentinel_arms_on_non_tabular_results() {
    let db = GraphLawDuckDB::open_in_memory().expect("in-memory engine");
    db.raw_connection()
        .execute_batch("CREATE TABLE empty_t (region TEXT, v BIGINT);")
        .expect("seed empty table");
    let compiled = compile_select("empty_t");
    let epoch_pre = db.shared_cache().epoch();
    let receipt = db
        .eval_ast_query("urn:v2e:empty", &compiled, "urn:v2e:s", "urn:v2e:p")
        .expect("non-tabular eval");

    assert_eq!(receipt.rows_emitted, 0, "no rows");
    assert!(receipt.result_triples.is_empty(), "no triples");
    assert_eq!(
        receipt.columnar_digest_hex.as_deref(),
        Some(EMPTY_COLUMNAR_PIN),
        "non-tabular columnar arm must be CANON_ARROW_EMPTY_V1"
    );
    // The empty-encoding triples arm is the pinned BLAKE3(b"[]").
    assert_eq!(hex(&h_triples_serterm(&[])), EMPTY_TRIPLES_PIN);
    // Full self-verification over the sentinel arms (independent fold).
    assert_self_verifying(&receipt, epoch_pre);
    // And the seal differs from the tabular arm's seal machinery: v2 over
    // the sentinel arms differs from the v1 fold (domain separation).
    let sentinel = d32(SENTINEL_PIN);
    let empty_col = d32(EMPTY_COLUMNAR_PIN);
    let empty_tri = d32(EMPTY_TRIPLES_PIN);
    let v2 = independent_v2_fold(
        &receipt.query_hash,
        &sentinel,
        &sentinel,
        &empty_tri,
        &empty_col,
        epoch_pre,
    );
    assert_eq!(
        receipt.receipt_v2_hex.as_deref(),
        Some(hex(&v2).as_str()),
        "sentinel-arm seal does not reproduce"
    );
    assert_ne!(
        v2,
        receipt_v1(&receipt.query_hash, &sentinel, &sentinel, &empty_tri, epoch_pre),
        "sentinel-arm v2 must differ from v1"
    );
}

// --- (4) divergent-graph override re-seals ------------------------------------

/// `eval_admitted_ast` with a divergent graph document overrides H_query;
/// the v2 envelope (which folds H_query) is re-sealed so the receipt stays
/// self-consistent with its own fields.
#[test]
fn divergent_graph_override_reseals_v2() {
    let db = engine_with_table();
    let compiled = compile_select("revenue");
    let same_doc = compiled.ast_graph_turtle.clone().expect("graph-backed compile");
    // Divergent document: a comment-bearing variant canonicalizes to a
    // DIFFERENT graph (extra triple), so the override arm fires.
    let divergent = format!(
        "{same_doc}\nex:q sql:annotationNote \"divergent\" .\n"
    );
    assert_ne!(same_doc, divergent);
    let epoch_pre = db.shared_cache().epoch();
    let receipt = db
        .eval_admitted_ast(&divergent, "urn:v2e:divergent", &compiled, "urn:v2e:s", "urn:v2e:p")
        .expect("divergent eval");

    let baseline = db
        .eval_ast_query("urn:v2e:baseline", &compiled, "urn:v2e:s", "urn:v2e:p")
        .expect("baseline eval");
    assert_ne!(
        receipt.query_hash, baseline.query_hash,
        "divergent graph must override the receipt hash"
    );
    assert_self_verifying(&receipt, epoch_pre);
}

// --- (5) domain digest pins ----------------------------------------------------

/// The OQ-1(a) sentinel is the pinned domain digest and differs from the
/// ADR §3.1 null digest (the two arms must never collide).
#[test]
fn sentinel_digest_is_pinned_and_domain_separated() {
    let got = receipt_v2_state_sentinel();
    assert_eq!(hex(got), SENTINEL_PIN, "state sentinel drifted");
    assert_ne!(got, &d32(EMPTY_COLUMNAR_PIN), "sentinel must not equal the null digest");
    assert_ne!(got, &[0u8; 32], "sentinel must be non-trivial");
}

/// The OQ-2 empty-encoding triples arm is the pinned BLAKE3(b"[]") — a real
/// digest of the empty SerTerm canonical form (distinct from both the state
/// sentinel and the columnar null digest).
#[test]
fn h_triples_empty_arm_is_pinned() {
    assert_eq!(hex(&h_triples_serterm(&[])), EMPTY_TRIPLES_PIN);
    assert_eq!(hex(&independent_h_triples(&[])), EMPTY_TRIPLES_PIN);
    assert_ne!(h_triples_serterm(&[]), *receipt_v2_state_sentinel());
}
