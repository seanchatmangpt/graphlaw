//! Crash/concurrency courts for `receipt_chain::store::ReceiptStore`.
//! Real temp dirs, real files, real threads — no doubles.

use std::io::Write as _;
use std::sync::{Arc, Barrier};

use graphlaw::receipt_chain::store::ReceiptStore;
use graphlaw::receipt_chain::{Andon, RECEIPT_RECORD_VERSION, ReceiptRecord, epoch::SCHEMA_V1};

fn chained(i: u64, prev: [u8; 32]) -> ReceiptRecord {
    let mut record = ReceiptRecord {
        version: RECEIPT_RECORD_VERSION,
        instruction_id: i,
        activity_idx: 0,
        activity: None,
        node_kind: 0,
        ts_ns: i * 1000,
        duration_ms: None,
        origin: None,
        payload_hash_hex: format!("{:02x}", i % 256).repeat(32),
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
    record
}

fn scratch(tag: &str) -> std::path::PathBuf {
    let d = std::env::temp_dir().join(format!("graphlaw-rcs-crash-{tag}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    d
}

#[test]
fn torn_tail_is_truncated_and_chain_continues() {
    let dir = scratch("torn");
    let store = ReceiptStore::open(&dir).unwrap();
    store.append_with(|p| Ok(chained(1, p))).unwrap();
    store.append_with(|p| Ok(chained(2, p))).unwrap();
    // Simulate a crash mid-append: partial JSON, no newline.
    let mut f = std::fs::OpenOptions::new().append(true).open(store.path()).unwrap();
    f.write_all(b"{\"version\":1,\"instruction_id\":3,\"activ").unwrap();
    drop(f);
    // Law change (R2): readers tolerate the unterminated crash fragment that
    // append_with would repair; only mid-file corruption is refused on load.
    assert_eq!(store.load_all().unwrap().len(), 2, "torn tail ignored, not fatal");
    assert_eq!(
        store.last_chain_hash().unwrap(),
        hex::decode(&store.load_all().unwrap()[1].chain_hash_hex).unwrap()[..]
    );

    store.append_with(|p| Ok(chained(3, p))).unwrap();
    let all = store.load_all().unwrap();
    assert_eq!(all.len(), 3);
    for w in all.windows(2) {
        assert_eq!(w[1].prev_chain_hash_hex, w[0].chain_hash_hex);
    }
    assert!(std::fs::read(store.path()).unwrap().ends_with(b"\n"));
}

#[test]
fn complete_record_missing_newline_is_kept() {
    let dir = scratch("nonl");
    let store = ReceiptStore::open(&dir).unwrap();
    let r1 = chained(1, [0; 32]);
    std::fs::write(store.path(), serde_json::to_string(&r1).unwrap()).unwrap();
    store.append_with(|p| Ok(chained(2, p))).unwrap();
    let all = store.load_all().unwrap();
    assert_eq!(all.len(), 2);
    assert_eq!(all[0], r1);
    assert_eq!(all[1].prev_chain_hash_hex, r1.chain_hash_hex);
}

#[test]
fn mid_file_corruption_is_refused_not_repaired() {
    let dir = scratch("corrupt");
    let store = ReceiptStore::open(&dir).unwrap();
    store.append_with(|p| Ok(chained(1, p))).unwrap();
    let mut f = std::fs::OpenOptions::new().append(true).open(store.path()).unwrap();
    f.write_all(b"garbage line\n").unwrap();
    drop(f);
    let before = std::fs::read(store.path()).unwrap();
    let err = store.append_with(|p| Ok(chained(2, p))).unwrap_err();
    assert!(format!("{err:?}").contains("RECEIPT-STORE-CORRUPT"));
    assert_eq!(std::fs::read(store.path()).unwrap(), before, "ledger untouched");
}

#[test]
fn concurrent_appenders_serialize_without_fork() {
    const THREADS: u64 = 8;
    const PER: u64 = 10;
    let dir = scratch("race");
    let barrier = Arc::new(Barrier::new(THREADS as usize));
    let mut hs = Vec::new();
    for t in 0..THREADS {
        let (dir, barrier) = (dir.clone(), barrier.clone());
        hs.push(std::thread::spawn(move || {
            // Separate store handle (separate open file description) per thread.
            let store = ReceiptStore::open(&dir).unwrap();
            barrier.wait();
            for i in 0..PER {
                store.append_with(|p| Ok(chained(t * PER + i + 1, p))).unwrap();
            }
        }));
    }
    for h in hs {
        h.join().unwrap();
    }
    let all = ReceiptStore::open(&dir).unwrap().load_all().unwrap();
    assert_eq!(all.len() as u64, THREADS * PER);
    assert_eq!(all[0].prev_chain_hash_hex, hex::encode([0u8; 32]));
    for w in all.windows(2) {
        assert_eq!(w[1].prev_chain_hash_hex, w[0].chain_hash_hex, "chain forked");
    }
}

#[test]
fn load_all_refuses_mid_file_corruption() {
    let dir = scratch("load-corrupt");
    let store = ReceiptStore::open(&dir).unwrap();
    store.append_with(|p| Ok(chained(1, p))).unwrap();
    let mut f = std::fs::OpenOptions::new().append(true).open(store.path()).unwrap();
    f.write_all(b"garbage line\n").unwrap();
    drop(f);
    assert!(store.load_all().is_err());
    assert!(store.last_chain_hash().is_err());
}

#[test]
fn load_all_includes_complete_record_missing_newline() {
    let dir = scratch("load-nonl");
    let store = ReceiptStore::open(&dir).unwrap();
    let r1 = chained(1, [0; 32]);
    std::fs::write(store.path(), serde_json::to_string(&r1).unwrap()).unwrap();
    assert_eq!(store.load_all().unwrap(), vec![r1]);
}

#[test]
fn readers_never_observe_torn_state_under_concurrent_appends() {
    let dir = scratch("readers");
    let writer_dir = dir.clone();
    let writer = std::thread::spawn(move || {
        let store = ReceiptStore::open(&writer_dir).unwrap();
        for i in 1..=40u64 {
            store.append_with(|p| Ok(chained(i, p))).unwrap();
        }
    });
    let store = ReceiptStore::open(&dir).unwrap();
    // Bounded by the writer finishing, not by time.
    while !writer.is_finished() {
        let all = store.load_all().expect("reader must never fail mid-append");
        for w in all.windows(2) {
            assert_eq!(w[1].prev_chain_hash_hex, w[0].chain_hash_hex);
        }
    }
    writer.join().unwrap();
    assert_eq!(store.load_all().unwrap().len(), 40);
}

#[test]
fn first_append_creates_ledger_under_lock_without_fork() {
    // Many threads race the very first append; creation is decided under the
    // lock so exactly one genesis exists and the chain is linear.
    let dir = scratch("create-race");
    let barrier = Arc::new(Barrier::new(6));
    let hs: Vec<_> = (0..6u64)
        .map(|t| {
            let (dir, barrier) = (dir.clone(), barrier.clone());
            std::thread::spawn(move || {
                let store = ReceiptStore::open(&dir).unwrap();
                barrier.wait();
                store.append_with(|p| Ok(chained(t + 1, p))).unwrap();
            })
        })
        .collect();
    for h in hs {
        h.join().unwrap();
    }
    let all = ReceiptStore::open(&dir).unwrap().load_all().unwrap();
    assert_eq!(all.len(), 6);
    assert_eq!(all[0].prev_chain_hash_hex, hex::encode([0u8; 32]));
    for w in all.windows(2) {
        assert_eq!(w[1].prev_chain_hash_hex, w[0].chain_hash_hex);
    }
}
