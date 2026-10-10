//! Chicago-TDD tests for the ported praxis-core validator + JSONL store
//! (`graphlaw::receipt_chain::{validator, store}`). Real filesystem (temp
//! dirs), real chain hashes — no doubles.

use graphlaw::receipt_chain::store::{GENESIS_CHAIN_HASH, ReceiptStore};
use graphlaw::receipt_chain::validator::{CheckOutcome, FixedClock, ReceiptValidator};
use graphlaw::receipt_chain::{Andon, RECEIPT_RECORD_VERSION, ReceiptRecord, epoch::SCHEMA_V1};

fn chained_records(n: u64) -> Vec<ReceiptRecord> {
    let mut records = Vec::new();
    let mut prev = [0u8; 32];
    for i in 1..=n {
        let mut record = ReceiptRecord {
            version: RECEIPT_RECORD_VERSION,
            instruction_id: i,
            activity_idx: 0,
            activity: None,
            node_kind: 0,
            ts_ns: i * 1000,
            duration_ms: None,
            origin: None,
            payload_hash_hex: format!("{i:02x}").repeat(32),
            prev_chain_hash_hex: hex::encode(prev),
            chain_hash_hex: String::new(),
            andon: Andon::Green,
            obligation_count: 0,
            object_ids: vec![format!("law:instr{i}")],
            signature_hex: None,
            schema: SCHEMA_V1.to_string(),
            v2: None,
            chain_rule: None,
        };
        let chain_hash = record.recompute_chain_hash().expect("recompute");
        record.chain_hash_hex = hex::encode(chain_hash);
        prev = chain_hash;
        records.push(record);
    }
    records
}

fn temp_store_dir(tag: &str) -> std::path::PathBuf {
    let d = std::env::temp_dir().join(format!("graphlaw-rcv-{tag}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    d
}

/// Round-trip: write 3 lawfully chained records, read them back, verify every
/// stage passes and the recomputed chain holds.
#[test]
fn round_trip_write_load_verify_chain() {
    let dir = temp_store_dir("roundtrip");
    let store = ReceiptStore::open(&dir).expect("open");
    let records = chained_records(3);
    for r in &records {
        store.append(r).expect("append");
    }

    let loaded = store.load_all().expect("load_all");
    assert_eq!(loaded.len(), 3);
    assert_eq!(loaded, records, "byte-level wire round trip");

    let verdict = ReceiptValidator::validate(&loaded, &FixedClock(u64::MAX));
    assert!(verdict.ok, "verdict: {verdict:?}");
    assert_eq!(verdict.records_checked, 3);
    for stage in &verdict.stages {
        assert!(
            stage.outcome.is_pass() || matches!(stage.outcome, CheckOutcome::Skip(_)),
            "stage {} did not pass: {:?}",
            stage.stage,
            stage.outcome
        );
    }
    std::fs::remove_dir_all(&dir).expect("cleanup");
}

/// A version-2 record is refused by the schema stage with the exact praxis
/// error text (error-naming parity).
#[test]
fn version_2_record_refused_with_praxis_error_text() {
    let mut records = chained_records(1);
    records[0].version = 2;
    let verdict = ReceiptValidator::validate(&records, &FixedClock(u64::MAX));
    assert!(!verdict.ok);
    let stage = verdict
        .stages
        .iter()
        .find(|s| s.stage == "schema")
        .expect("schema stage present");
    let expected =
        format!("record 0: unsupported schema version 2 (expected {RECEIPT_RECORD_VERSION})");
    match &stage.outcome {
        CheckOutcome::Fail(msg) => assert_eq!(*msg, expected, "error text parity"),
        other => panic!("expected schema Fail, got {other:?}"),
    }
}

/// Flipping one hex character of the middle record's payload hash fails
/// `chain_recompute` with the tamper message.
#[test]
fn tampered_middle_record_fails_verification() {
    let mut records = chained_records(3);
    let c = records[1]
        .payload_hash_hex
        .chars()
        .next()
        .expect("nonempty");
    let replacement = if c == 'a' { 'b' } else { 'a' };
    records[1]
        .payload_hash_hex
        .replace_range(0..1, &replacement.to_string());

    let verdict = ReceiptValidator::validate(&records, &FixedClock(u64::MAX));
    assert!(!verdict.ok);
    let stage = verdict
        .stages
        .iter()
        .find(|s| s.stage == "chain_recompute")
        .expect("chain_recompute stage present");
    match &stage.outcome {
        CheckOutcome::Fail(msg) => assert!(
            msg.contains(
                "recomputed chain hash does not match stored chain_hash_hex (tamper detected)"
            ),
            "unexpected message: {msg}"
        ),
        other => panic!("expected chain_recompute Fail, got {other:?}"),
    }
}

/// Loading an empty (nonexistent-ledger) store is Ok-empty and yields the
/// genesis chain hash.
#[test]
fn empty_store_load_is_ok_empty() {
    let dir = temp_store_dir("empty");
    let store = ReceiptStore::open(&dir).expect("open");
    assert_eq!(
        store.load_all().expect("load_all"),
        Vec::<ReceiptRecord>::new()
    );
    assert_eq!(
        store.last_chain_hash().expect("genesis"),
        GENESIS_CHAIN_HASH
    );
    assert!(
        !store.path().exists(),
        "ledger not created before first append"
    );
    std::fs::remove_dir_all(&dir).expect("cleanup");
}
