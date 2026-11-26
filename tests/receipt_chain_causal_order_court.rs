//! Causal-order court (R4b): ordering is carried by `instruction_id` and
//! `prev_chain_hash_hex` linkage; `ts_ns` is observational with a bounded
//! skew allowance (`validator::MAX_TS_SKEW_NS`).

use graphlaw::receipt_chain::validator::{FixedClock, ReceiptValidator};
use graphlaw::receipt_chain::{Andon, RECEIPT_RECORD_VERSION, ReceiptRecord, epoch::SCHEMA_V1};

const BASE_TS: u64 = 1_000_000_000;

fn chain_with_ts(ts: &[u64]) -> Vec<ReceiptRecord> {
    let mut records = Vec::new();
    let mut prev = [0u8; 32];
    for (idx, t) in ts.iter().enumerate() {
        let i = idx as u64 + 1;
        let mut record = ReceiptRecord {
            version: RECEIPT_RECORD_VERSION,
            instruction_id: i,
            activity_idx: 0,
            activity: None,
            node_kind: 0,
            ts_ns: *t,
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
        let h = record.recompute_chain_hash().expect("recompute");
        record.chain_hash_hex = hex::encode(h);
        prev = h;
        records.push(record);
    }
    records
}

fn validate(records: &[ReceiptRecord]) -> graphlaw::receipt_chain::validator::Verdict {
    ReceiptValidator::validate(records, &FixedClock(u64::MAX))
}

/// (a) Correct linkage, ts_ns skewed 1 ms backwards: verifies OK.
#[test]
fn one_ms_backward_skew_with_valid_linkage_verifies() {
    let records = chain_with_ts(&[BASE_TS, BASE_TS + 10_000_000, BASE_TS + 10_000_000 - 1_000_000]);
    let v = validate(&records);
    assert!(v.ok, "verdict: {v:?}");
}

/// (a') Even a large backward ts jump passes while linkage is intact.
#[test]
fn large_backward_ts_with_valid_linkage_verifies() {
    let records = chain_with_ts(&[BASE_TS + 500_000_000, BASE_TS]);
    let v = validate(&records);
    assert!(v.ok, "verdict: {v:?}");
}

/// (b) Broken prev_chain_hash fails regardless of timestamps.
#[test]
fn broken_linkage_fails_with_any_timestamps() {
    for ts in [
        [BASE_TS, BASE_TS + 1, BASE_TS + 2],
        [BASE_TS, BASE_TS - 1_000_000 + 5, BASE_TS],
        [BASE_TS + 900_000_000, BASE_TS, BASE_TS],
    ] {
        let mut records = chain_with_ts(&ts);
        records[1].prev_chain_hash_hex = "ab".repeat(32);
        let v = validate(&records);
        assert!(!v.ok, "broken linkage must fail for ts {ts:?}: {v:?}");
    }
}

/// (c) Reordered chain with consistent (ascending) timestamps but broken
/// linkage fails.
#[test]
fn reordered_chain_with_consistent_timestamps_fails() {
    let mut records = chain_with_ts(&[BASE_TS, BASE_TS + 1_000, BASE_TS + 2_000]);
    records.swap(1, 2);
    // Re-stamp timestamps ascending so ts order looks consistent after swap.
    for (i, r) in records.iter_mut().enumerate() {
        r.ts_ns = BASE_TS + (i as u64) * 1_000;
    }
    let v = validate(&records);
    assert!(!v.ok, "reordered chain must fail: {v:?}");
}

fn monotonic_outcome(
    v: &graphlaw::receipt_chain::validator::Verdict,
) -> &graphlaw::receipt_chain::validator::CheckOutcome {
    &v.stages
        .iter()
        .find(|s| s.stage == "monotonic")
        .expect("monotonic stage present")
        .outcome
}

/// (d) Future-ts boundary under a real FixedClock: a ts exactly at
/// `now + MAX_TS_SKEW_NS` admits; one ns beyond refuses in the `monotonic` stage.
#[test]
fn future_ts_within_skew_admits_and_beyond_skew_refuses() {
    use graphlaw::receipt_chain::validator::MAX_TS_SKEW_NS;
    let now = BASE_TS;
    let clock = FixedClock(now);

    let within = chain_with_ts(&[now - 10, now + MAX_TS_SKEW_NS]);
    let v = ReceiptValidator::validate(&within, &clock);
    assert!(v.ok, "within skew must admit: {v:?}");
    assert!(monotonic_outcome(&v).is_pass(), "{v:?}");

    let beyond = chain_with_ts(&[now - 10, now + MAX_TS_SKEW_NS + 1]);
    let v = ReceiptValidator::validate(&beyond, &clock);
    assert!(!v.ok, "beyond skew must refuse: {v:?}");
    assert!(
        matches!(monotonic_outcome(&v), graphlaw::receipt_chain::validator::CheckOutcome::Fail(m) if m.contains("record 1") && m.contains("future")),
        "{v:?}"
    );
}

/// (e) A backward ts step beyond skew with broken linkage is still refused
/// (by `chain_linkage`), and the `monotonic` stage does not judge ts regression.
#[test]
fn broken_linkage_with_large_ts_regression_is_refused_by_linkage_stage() {
    let mut records = chain_with_ts(&[BASE_TS + 500_000_000, BASE_TS]);
    records[1].prev_chain_hash_hex = "ab".repeat(32);
    let v = ReceiptValidator::validate(&records, &FixedClock(u64::MAX));
    assert!(!v.ok, "{v:?}");
    assert!(monotonic_outcome(&v).is_pass(), "{v:?}");
    let linkage = v.stages.iter().find(|s| s.stage == "chain_linkage").unwrap();
    assert!(!linkage.outcome.is_pass(), "{v:?}");
}
