//! FM-CHAIN-009 golden test: graphlaw::receipt_chain must reproduce the
//! chain hashes of ggen's committed TCPS example receipt byte-for-byte, and
//! survive a real JSON round trip (FM-CHAIN-014's regression shape).
//!
//! The fixture is the verbatim `record` object of ggen's
//! `examples/tcps-generated/.ggen-v2/receipt.json` (sealed 2026-07-22, before
//! the F1 fold), canonically committed at `tests/fixtures/fm_chain_009.json`
//! (byte-identical pin mirrored in affidavit-core, seam #28).
//! Passing IS the byte-exactness proof for the 99-byte frame packing and the
//! BLAKE3 chain rule.

use graphlaw::receipt_chain::epoch::{
    AndonLevel, CeilingLevel, ComponentLevels, ReceiptEpochV2Builder, SCHEMA_V1, SCHEMA_V2,
};
use graphlaw::receipt_chain::{
    ChainRule, ChainStanding, ChainVerification, RECEIPT_RECORD_VERSION, ReceiptRecord,
};

/// The committed fixture's `record` object, verbatim JSON. Canonical copy:
    /// `tests/fixtures/fm_chain_009.json`; this pin makes any one-sided edit
    /// fail loudly (shared-fixture drift guard, seam #28).
    const FIXTURE_BLAKE3: &str = "d52282d8c48e9021ff6f78e413d8a79dbffbc6b28e28db9a9d855f5810bb3185";

    /// Path of the canonical fixture, relative to CARGO_MANIFEST_DIR.
    const FIXTURE_PATH: &str = "tests/fixtures/fm_chain_009.json";

    /// Load the fixture bytes and verify the pin before any test consumes it.
    fn fixture_bytes() -> Vec<u8> {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(FIXTURE_PATH);
        let bytes = std::fs::read(&path).expect("fixture file readable");
        let got = hex::encode(blake3::hash(&bytes).as_bytes());
        assert_eq!(got, FIXTURE_BLAKE3, "fixture drift: {FIXTURE_PATH} changed; if intentional, re-pin FIXTURE_BLAKE3 AND mirror the byte-identical copy in affidavit-core/tests/fixtures/fm_chain_009.json");
        bytes
    }

    /// Fixture as a &str for serde_json::from_str consumers.
    fn fixture_json() -> String {
        String::from_utf8(fixture_bytes()).expect("fixture is utf-8")
    }

    #[test]
    fn fixture_pin_detects_one_sided_edits() {
        // Re-hashes independently of fixture_bytes' internal check so a pin
        // regression is a distinct, named failure.
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(FIXTURE_PATH);
        let bytes = std::fs::read(&path).expect("fixture file readable");
        assert_eq!(
            hex::encode(blake3::hash(&bytes).as_bytes()),
            FIXTURE_BLAKE3,
            "FM-CHAIN-009 fixture edited without re-pinning in both repos"
        );
    }

#[test]
fn receipt_record_version_is_frozen() {
    assert_eq!(RECEIPT_RECORD_VERSION, 1);
}

/// FM-CHAIN-009: the committed head recomputes to its stored chain hash under
/// the base rule, reproduces the exact FM-CHAIN-009 fold mismatch under the
/// fold rule, and verifies as capped legacy.
#[test]
fn committed_tcps_head_recomputes_to_its_stored_chain_hash() {
    let record: ReceiptRecord =
        serde_json::from_str(&fixture_json()).expect("fixture record deserializes");

    assert_eq!(record.schema, SCHEMA_V2);
    assert!(record.v2.is_some());
    assert!(record.chain_rule.is_none());
    assert_eq!(record.version, 1);
    assert_eq!(record.activity.as_deref(), Some("ggen.sync"));

    let base = hex::encode(
        record
            .recompute_chain_hash_under(ChainRule::Base)
            .expect("base recompute"),
    );
    assert_eq!(base, record.chain_hash_hex);
    assert!(
        record.chain_hash_hex.starts_with("d04c6d08"),
        "head {}",
        record.chain_hash_hex
    );

    let fold = hex::encode(
        record
            .recompute_chain_hash_under(ChainRule::V2Fold)
            .expect("fold recompute"),
    );
    assert!(fold.starts_with("bebae299"), "fold recompute {fold}");

    assert_eq!(
        record.verify_chain().expect("verify"),
        ChainVerification::Verified(ChainStanding::LegacyV2Unbound)
    );

    // Falsifier on the real bytes: tampering the committed head's v2 payload
    // never yields a fully-bound standing.
    let mut forged = record.clone();
    forged.v2.as_mut().expect("v2 present").standing_ceiling = CeilingLevel::Red;
    assert_ne!(
        forged.verify_chain().expect("verify"),
        ChainVerification::Verified(ChainStanding::FullyBound)
    );
}

/// FM-CHAIN-014 shape: a v2 record's chain hash must survive a real
/// serialize/deserialize round trip (the JSONL persist/reload boundary).
#[test]
fn recompute_chain_hash_survives_a_real_json_round_trip() {
    let epoch = ReceiptEpochV2Builder::new(
        CeilingLevel::Green,
        ComponentLevels::uniform(AndonLevel::Green),
    )
    .build()
    .expect("epoch builds");

    let mut record: ReceiptRecord =
        serde_json::from_str(&fixture_json()).expect("fixture record deserializes");
    record.schema = SCHEMA_V2.to_string();
    record.v2 = Some(epoch);
    let chain = record
        .recompute_chain_hash()
        .expect("recompute at write time");
    record.chain_hash_hex = hex::encode(chain);

    let serialized = serde_json::to_string(&record).expect("serialize record");
    let reloaded: ReceiptRecord = serde_json::from_str(&serialized).expect("deserialize record");
    let recomputed_after_round_trip = reloaded
        .recompute_chain_hash()
        .expect("recompute after round trip");
    assert_eq!(
        hex::encode(recomputed_after_round_trip),
        reloaded.chain_hash_hex,
        "chain hash must survive a real JSON round trip (FM-CHAIN-014)"
    );
}

/// The v1 invariant: a record with no v2 payload hashes identically under
/// both rules (the fold is a strict no-op).
#[test]
fn v1_record_folds_to_the_base_hash() {
    let mut record: ReceiptRecord =
        serde_json::from_str(&fixture_json()).expect("fixture record deserializes");
    record.schema = SCHEMA_V1.to_string();
    record.v2 = None;
    record.chain_rule = None;
    let base = record
        .recompute_chain_hash_under(ChainRule::Base)
        .expect("base");
    let fold = record
        .recompute_chain_hash_under(ChainRule::V2Fold)
        .expect("fold");
    assert_eq!(base, fold);
    record.chain_hash_hex = hex::encode(base);
    assert_eq!(
        record.verify_chain().expect("verify"),
        ChainVerification::Verified(ChainStanding::FullyBound)
    );
}
