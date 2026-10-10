//! Chicago-TDD tests for the M_1_to_2 migration-epoch port: the v1-legacy
//! strict reader (`ReceiptRecordV1Legacy`, 14 fields, `deny_unknown_fields`),
//! the `MigrationReceipt` (`MigrationReceipt::new` pins
//! `CeilingLevel::LegacyObserved`), and the frozen wire constant
//! `MIGRATION_LAW_1_TO_2` (`"M_1_to_2"`). Real serde over real JSON, real
//! chain-hash recomputation — no doubles.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use graphlaw::receipt_chain::{
    Andon, RECEIPT_RECORD_VERSION, ReceiptRecord,
    epoch::{
        CeilingLevel, MIGRATION_LAW_1_TO_2, MigrationReceipt, ReceiptRecordV1Legacy, SCHEMA_V1,
        SCHEMA_V2, read_receipt_epoch,
    },
};

/// A lawful v1 wire-shape record (exactly the 14 pre-epoch keys — no
/// `schema`, no `v2`), chained-hash-consistent: `chain_hash_hex` is the real
/// recomputation of the ported `ReceiptRecord` over the shared fields, so
/// both readers must agree on it.
fn v1_wire_json() -> (String, ReceiptRecord) {
    let mut record = ReceiptRecord {
        version: RECEIPT_RECORD_VERSION,
        instruction_id: 7,
        activity_idx: 3,
        activity: Some("ggen.sync".to_string()),
        node_kind: 2,
        ts_ns: 1_728_470_400_000_000_000,
        duration_ms: Some(42),
        origin: None,
        payload_hash_hex: "ab".repeat(32),
        prev_chain_hash_hex: "cd".repeat(32),
        chain_hash_hex: String::new(),
        andon: Andon::Green,
        obligation_count: 5,
        object_ids: vec!["law:instr7".to_string()],
        signature_hex: None,
        schema: SCHEMA_V1.to_string(),
        v2: None,
        chain_rule: None,
    };
    let chain_hash = record.recompute_chain_hash().expect("recompute");
    record.chain_hash_hex = hex::encode(chain_hash);

    let mut json = serde_json::to_value(&record).expect("serialize");
    if let serde_json::Value::Object(ref mut map) = json {
        // Strip the post-epoch keys so the fixture models the true
        // pre-epoch (v1-only) wire shape the legacy reader would see.
        map.remove("schema");
        map.remove("chain_rule");
    }
    let wire = serde_json::to_string_pretty(&json).expect("serialize wire");
    (wire, record)
}

/// Round-trip: the v1-legacy wire shape deserializes into
/// `ReceiptRecordV1Legacy` with all 14 fields intact and reserializes to the
/// same JSON (field order aside, value-equal).
#[test]
fn v1_legacy_round_trip_preserves_all_14_fields() {
    let (wire, record) = v1_wire_json();
    let legacy: ReceiptRecordV1Legacy = serde_json::from_str(&wire).expect("parse legacy");
    assert_eq!(legacy.version, RECEIPT_RECORD_VERSION);
    assert_eq!(legacy.instruction_id, record.instruction_id);
    assert_eq!(legacy.activity_idx, record.activity_idx);
    assert_eq!(legacy.activity, record.activity);
    assert_eq!(legacy.node_kind, record.node_kind);
    assert_eq!(legacy.ts_ns, record.ts_ns);
    assert_eq!(legacy.duration_ms, record.duration_ms);
    assert_eq!(legacy.payload_hash_hex, record.payload_hash_hex);
    assert_eq!(legacy.prev_chain_hash_hex, record.prev_chain_hash_hex);
    assert_eq!(legacy.chain_hash_hex, record.chain_hash_hex);
    assert_eq!(legacy.andon, record.andon);
    assert_eq!(legacy.obligation_count, record.obligation_count);
    assert_eq!(legacy.object_ids, record.object_ids);
    assert_eq!(legacy.signature_hex, record.signature_hex);

    let round: ReceiptRecordV1Legacy =
        serde_json::from_str(&serde_json::to_string(&legacy).expect("serialize")).expect("parse");
    assert_eq!(round, legacy);
}

/// The old reader refuses a v2 receipt: `schema`/`v2` are unknown keys to
/// `deny_unknown_fields`, so a genuine v2 JSON must fail to parse. This is
/// the concrete half of "old code refuses new receipts".
#[test]
fn v1_legacy_reader_refuses_v2_wire_shape() {
    let (wire, _) = v1_wire_json();
    let mut wire_val: serde_json::Value = serde_json::from_str(&wire).expect("value");
    // Upgrade the fixture to a v2 wire shape: declare the schema and hang a
    // minimal (well-formed) v2 payload off it.
    let obj = wire_val.as_object_mut().expect("object");
    obj.insert(
        "schema".to_string(),
        serde_json::Value::String(SCHEMA_V2.to_string()),
    );
    obj.insert(
        "v2".to_string(),
        serde_json::json!({
            "admission": {"Recorded": []},
            "standing_ceiling": "Green",
            "equivalence": {
                "source": "Unknown", "compiled_binary": "Unknown", "docs": "Unknown",
                "tests": "Unknown", "receipts": "Unknown", "evidence": "Unknown",
                "gates": "Unknown", "config": "Unknown"
            },
            "obligation_count": "Unknown",
            "andon": "Green",
            "promotion_eligible": true
        }),
    );
    let v2_wire = serde_json::to_string(&wire_val).expect("serialize");

    let err = serde_json::from_str::<ReceiptRecordV1Legacy>(&v2_wire)
        .expect_err("legacy reader must refuse v2");
    let msg = err.to_string();
    assert!(
        msg.contains("schema") || msg.contains("v2"),
        "refusal must name the unknown key, got: {msg}"
    );

    // The same JSON through the current reader: accepted, epoch reads as
    // the v2 payload (not legacy-bounded).
    let record: ReceiptRecord = serde_json::from_str(&v2_wire)
        .unwrap_or_else(|e| panic!("current reader accepts failed: {e}; wire={v2_wire}"));
    assert_eq!(record.schema, SCHEMA_V2);
    assert!(record.v2.is_some());
}

/// Differential: a v1 receipt verifies identically through the ported
/// types. The same v1 wire JSON parses into BOTH the legacy reader and the
/// current `ReceiptRecord`; all shared fields agree, the current reader's
/// recomputed chain hash matches the wire's `chain_hash_hex` (so the legacy
/// hash is still the lawful hash), and `read_receipt_epoch` reports the
/// legacy-bounded epoch view.
#[test]
fn v1_receipt_verifies_identically_through_both_readers() {
    let (wire, record) = v1_wire_json();

    let legacy: ReceiptRecordV1Legacy = serde_json::from_str(&wire).expect("legacy parse");
    let current: ReceiptRecord = serde_json::from_str(&wire).expect("current parse");

    // Every shared field agrees across the two readers.
    assert_eq!(legacy.version, current.version);
    assert_eq!(legacy.instruction_id, current.instruction_id);
    assert_eq!(legacy.activity_idx, current.activity_idx);
    assert_eq!(legacy.activity, current.activity);
    assert_eq!(legacy.node_kind, current.node_kind);
    assert_eq!(legacy.ts_ns, current.ts_ns);
    assert_eq!(legacy.duration_ms, current.duration_ms);
    assert_eq!(legacy.payload_hash_hex, current.payload_hash_hex);
    assert_eq!(legacy.prev_chain_hash_hex, current.prev_chain_hash_hex);
    assert_eq!(legacy.chain_hash_hex, current.chain_hash_hex);
    assert_eq!(legacy.andon, current.andon);
    assert_eq!(legacy.obligation_count, current.obligation_count);
    assert_eq!(legacy.object_ids, current.object_ids);
    assert_eq!(legacy.signature_hex, current.signature_hex);

    // The wire's stored hash is the lawful recomputation over the shared
    // fields — i.e. the v1-legacy record verifies identically today.
    let recomputed = current.recompute_chain_hash().expect("recompute");
    assert_eq!(hex::encode(recomputed), legacy.chain_hash_hex);
    assert_eq!(hex::encode(recomputed), record.chain_hash_hex);

    // Epoch view of the v1 record: legacy-bounded, not a real v2 payload.
    let epoch_view = read_receipt_epoch(&current).expect("epoch read");
    assert_eq!(epoch_view.standing_ceiling, CeilingLevel::LegacyObserved);
}

/// `MigrationReceipt::new` pins the migration identity, both schema ends,
/// the carry-forward/unknown inventories, and — with no constructor
/// argument for it — the `LegacyObserved` ceiling. Wire round-trip preserves
/// all of it.
#[test]
fn migration_receipt_new_pins_legacy_observed_ceiling_and_round_trips() {
    assert_eq!(MIGRATION_LAW_1_TO_2, "M_1_to_2");
    let receipt = MigrationReceipt::new("aa".repeat(32), "bb".repeat(32));
    assert_eq!(receipt.migration_law, "M_1_to_2");
    assert_eq!(receipt.from_schema, SCHEMA_V1);
    assert_eq!(receipt.to_schema, SCHEMA_V2);
    assert_eq!(receipt.final_v1_chain_hash_hex, "aa".repeat(32));
    assert_eq!(receipt.first_v2_chain_hash_hex, "bb".repeat(32));
    assert_eq!(
        receipt.carries_forward,
        vec![
            "chain_hash_lineage".to_string(),
            "object_ids".to_string(),
            "instruction_id_sequence".to_string(),
        ]
    );
    assert_eq!(
        receipt.becomes_unknown,
        vec![
            "admission".to_string(),
            "equivalence".to_string(),
            "obligation_count".to_string(),
        ]
    );
    assert_eq!(receipt.resulting_ceiling, CeilingLevel::LegacyObserved);

    // Wire round-trip, byte-equal through serde.
    let wire = serde_json::to_string(&receipt).expect("serialize");
    let parsed: MigrationReceipt = serde_json::from_str(&wire).expect("parse");
    assert_eq!(parsed, receipt);
    assert_eq!(
        serde_json::to_value(&parsed).expect("serialize"),
        serde_json::to_value(&receipt).expect("serialize")
    );
}
