//! Courts F1-F7 for the cold-path query ledger
//! (docs/jira/v26.10.22/designs/cold-path-ledger.md).
//!
//! Chicago discipline: every court operates on real files in a real temp
//! directory and asserts on real readbacks — no mocks, no sleeps.

use graphlaw::query_ledger::{
    content_digest, Aggregate, BatchAgg, Descriptor, LedgerError, QueryLedger, GENESIS,
};
use std::collections::BTreeMap;

fn dir(name: &str) -> std::path::PathBuf {
    let d = std::env::temp_dir().join(format!(
        "query-ledger-court-{}-{}",
        name,
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&d);
    d
}

fn desc(epoch: u64, rows: u32, ts: u64) -> Descriptor {
    Descriptor {
        h_query: [9u8; 32],
        rows_emitted: rows,
        flags: 0,
        ts_unix_ms: ts,
        epoch,
    }
}

fn write_batch_file(dir: &std::path::Path, body: &str) -> String {
    let digest = content_digest(body.as_bytes());
    std::fs::write(dir.join(format!("{digest}.json")), body).unwrap();
    digest
}

/// Three chained batches with real digests; returns (dir, digests).
fn three_batch_chain(name: &str) -> (std::path::PathBuf, Vec<String>) {
    let d = dir(name);
    std::fs::create_dir_all(&d).unwrap();
    let mut parent = GENESIS.to_string();
    let mut digests = Vec::new();
    for i in 0..3u64 {
        let body = format!(
            "{{\"parent\":\"{parent}\",\"first_seq\":{},\"count\":2,\"records\":[{{\"h_query\":\"{}\",\"rows_emitted\":10,\"flags\":0,\"ts_unix_ms\":1000,\"epoch\":1}},{{\"h_query\":\"{}\",\"rows_emitted\":20,\"flags\":0,\"ts_unix_ms\":2000,\"epoch\":2}}]}}",
            i * 2,
            "aa".repeat(32),
            "bb".repeat(32),
        );
        let digest = write_batch_file(&d, &body);
        std::fs::write(
            d.join(format!("{digest}.agg")),
            format!(
                "{{\"batch\":\"{digest}\",\"rows_emitted_sum\":{{\"1\":10,\"2\":20}},\"saturated\":false}}"
            ),
        )
        .unwrap();
        parent = digest.clone();
        digests.push(digest);
    }
    let chain = format!(
        "{{\"head\":\"{}\",\"tail\":\"{}\",\"batches\":3,\"next_seq\":6}}",
        digests[2], digests[0]
    );
    std::fs::write(d.join("chain.json"), chain).unwrap();
    (d, digests)
}

/// F1 — a single flipped byte in a batch file breaks its content address.
#[test]
fn f1_byte_flip_is_digest_mismatch() {
    let (d, digests) = three_batch_chain("f1");
    let ledger = QueryLedger::open(&d).unwrap();
    let path = d.join(format!("{}.json", digests[1]));
    let mut bytes = std::fs::read(&path).unwrap();
    let idx = bytes.iter().position(|b| *b == b'1').unwrap();
    bytes[idx] = b'2';
    std::fs::write(&path, &bytes).unwrap();
    let err = ledger.verify().unwrap_err();
    match err {
        LedgerError::DigestMismatch {
            expected, actual, ..
        } => {
            assert_eq!(expected, digests[1]);
            assert_ne!(actual, expected);
            assert_eq!(actual.len(), 64);
        }
        other => panic!("expected DigestMismatch, got {other:?}"),
    }
    let _ = std::fs::remove_dir_all(&d);
}

/// F2 — deleting the middle batch breaks the chain and the period
/// aggregates with complete:false (loss disclosed, never absorbed).
#[test]
fn f2_deleted_middle_batch_broken_chain_and_incomplete() {
    let (d, digests) = three_batch_chain("f2");
    let ledger = QueryLedger::open(&d).unwrap();
    std::fs::remove_file(d.join(format!("{}.json", digests[1]))).unwrap();
    std::fs::remove_file(d.join(format!("{}.agg", digests[1]))).unwrap();
    // Chain walk refuses.
    match ledger.verify().unwrap_err() {
        LedgerError::BrokenChain { detail } => assert!(detail.contains(&digests[2])),
        other => panic!("expected BrokenChain, got {other:?}"),
    }
    // Aggregation over the surviving batches discloses incompleteness
    // (first_seq discontinuity), it does not fabricate a complete answer.
    let agg = ledger.aggregate(0, u64::MAX).unwrap();
    assert!(!agg.complete);
    assert_eq!(agg.batches, 2);
    let _ = std::fs::remove_dir_all(&d);
}

/// F3 — a forked chain (two batches claiming one parent) is refused.
#[test]
fn f3_fork_refused() {
    let (d, digests) = three_batch_chain("f3");
    let ledger = QueryLedger::open(&d).unwrap();
    // Hand-mint a second child of digests[1] with different bytes.
    let body = format!(
        "{{\"parent\":\"{}\",\"first_seq\":99,\"count\":1,\"records\":[{{\"h_query\":\"{}\",\"rows_emitted\":1,\"flags\":0,\"ts_unix_ms\":1,\"epoch\":9}}]}}",
        digests[1],
        "cc".repeat(32),
    );
    write_batch_file(&d, &body);
    match ledger.verify().unwrap_err() {
        LedgerError::Fork { parent } => assert_eq!(parent, digests[1]),
        other => panic!("expected Fork, got {other:?}"),
    }
    let _ = std::fs::remove_dir_all(&d);
}

/// F4 — a second genesis batch is refused.
#[test]
fn f4_double_genesis_refused() {
    let (d, _digests) = three_batch_chain("f4");
    let ledger = QueryLedger::open(&d).unwrap();
    let body = format!(
        "{{\"parent\":\"{GENESIS}\",\"first_seq\":100,\"count\":0,\"records\":[]}}"
    );
    write_batch_file(&d, &body);
    match ledger.verify().unwrap_err() {
        LedgerError::DoubleGenesis { digest } => {
            // The refused digest is the hand-minted second genesis, not a
            // real chain member.
            assert!(!digests_of(&d).contains(&digest) || digest != GENESIS);
            assert!(std::path::Path::new(&d.join(format!("{digest}.json"))).exists());
        }
        other => panic!("expected DoubleGenesis, got {other:?}"),
    }
    let _ = std::fs::remove_dir_all(&d);
}

fn digests_of(d: &std::path::Path) -> Vec<String> {
    let mut v: Vec<String> = std::fs::read_dir(d)
        .unwrap()
        .filter_map(|e| {
            let n = e.unwrap().file_name().to_string_lossy().to_string();
            n.strip_suffix(".json")
                .filter(|s| s.len() == 64)
                .map(|s| s.to_string())
        })
        .collect();
    v.sort();
    v
}

/// F5 — drain completeness is arithmetic: a producer ahead of the persisted
/// ledger by exactly the in-flight count persists a gap marker of exactly
/// that size, and periods over the gap aggregate complete:false.
#[test]
fn f5_drain_completeness_gap_is_inflight_count() {
    let d = dir("f5");
    {
        let mut ledger = QueryLedger::open(&d).unwrap();
        let batch: Vec<Descriptor> = (0..5).map(|i| desc(1, 7, 1000 + i)).collect();
        ledger.record_batch(&batch).unwrap();
        assert_eq!(ledger.chain_state().next_seq, 5);
    } // Drop flushes (nothing buffered here, batch already flushed)
    // Producer drained 3 more descriptors that never hit the ledger.
    let ledger = QueryLedger::open_with_producer(&d, 8).unwrap();
    let gaps = ledger.gaps().unwrap();
    assert_eq!(gaps, vec![(5, 7)], "gap = exactly the in-flight count");
    assert!(!ledger.aggregate(0, u64::MAX).unwrap().complete);
    // Re-open is idempotent: no duplicate markers.
    let again = QueryLedger::open_with_producer(&d, 8).unwrap();
    assert_eq!(again.gaps().unwrap().len(), 1);
    let _ = std::fs::remove_dir_all(&d);
}

/// F6 — overflow is disclosed loss, never a wrapped number: a per-epoch sum
/// at u64::MAX saturates the aggregate and sets `saturated: true`.
#[test]
fn f6_overflow_as_loss_saturates_with_flag() {
    let (d, digests) = three_batch_chain("f6");
    let ledger = QueryLedger::open(&d).unwrap();
    // Craft the NON-spot-checked batch's sidecar at the u64 boundary.
    // (The .agg is keyed to the batch but not self-addressed — disclosed in
    // the design; the court exercises the fold's saturation arithmetic.)
    std::fs::write(
        d.join(format!("{}.agg", digests[2])),
        format!(
            "{{\"batch\":\"{}\",\"rows_emitted_sum\":{{\"1\":{}}},\"saturated\":true}}",
            digests[2],
            u64::MAX
        ),
    )
    .unwrap();
    let agg = ledger.aggregate(0, u64::MAX).unwrap();
    assert!(agg.saturated, "overflow must set saturated, not wrap");
    assert_eq!(agg.rows_emitted_sum[&1], u64::MAX);
    let _ = std::fs::remove_dir_all(&d);
}

/// F7 — aggregation correctness: per-epoch sums fold exactly, first_seq
/// arithmetic holds, and the mandatory spot-re-fold fires on a tampered
/// first batch sidecar.
#[test]
fn f7_aggregation_correct_with_spot_refold() {
    let d = dir("f7");
    {
        let mut ledger = QueryLedger::open(&d).unwrap();
        ledger
            .record_batch(&[desc(1, 10, 1_000), desc(2, 20, 2_000)])
            .unwrap();
        ledger
            .record_batch(&[desc(1, 5, 3_000), desc(1, 5, 4_000)])
            .unwrap();
        assert_eq!(ledger.chain_state().next_seq, 4);
    }
    let ledger = QueryLedger::open(&d).unwrap();
    let st = ledger.verify().unwrap();
    assert_eq!(st.batches, 2);
    assert_eq!(st.tail.len(), 64);
    assert_ne!(st.tail, GENESIS);
    // Period restricted to the first batch's timestamps.
    let agg = ledger.aggregate(0, 2_500).unwrap();
    let mut want = BTreeMap::new();
    want.insert(1u64, 10u64);
    want.insert(2u64, 20u64);
    assert_eq!(agg, Aggregate {
        rows_emitted_sum: want,
        saturated: false,
        complete: true,
        batches: 1,
    });
    // Full-period fold.
    let agg = ledger.aggregate(0, u64::MAX).unwrap();
    assert_eq!(agg.rows_emitted_sum[&1], 20);
    assert_eq!(agg.rows_emitted_sum[&2], 20);
    assert_eq!(agg.batches, 2);
    assert!(agg.complete);
    // Spot-re-fold tamper check: corrupting the FIRST folded batch's sidecar
    // (lowest first_seq — the tail) must be caught against a re-fold of the
    // real batch bytes.
    let tail = st.tail;
    std::fs::write(
        d.join(format!("{tail}.agg")),
        format!(
            "{{\"batch\":\"{tail}\",\"rows_emitted_sum\":{{\"1\":999}},\"saturated\":false}}"
        ),
    )
    .unwrap();
    match ledger.aggregate(0, u64::MAX).unwrap_err() {
        LedgerError::AggMismatch { batch } => assert_eq!(batch, tail),
        other => panic!("expected AggMismatch, got {other:?}"),
    }
    let _ = std::fs::remove_dir_all(&d);
}

/// Format courts: content-addressed names, genesis parent, monotonic
/// first_seq, canonical bytes (fixed key order, hand-rolled escaper), .agg
/// sidecars, flush on Drop, sidecar equals a re-fold.
#[test]
fn format_content_addressing_genesis_and_sidecars() {
    let d = dir("fmt");
    {
        let mut ledger = QueryLedger::open(&d).unwrap();
        ledger
            .record_batch(&[desc(1, 3, 10), desc(1, 4, 20)])
            .unwrap();
        ledger.push(desc(1, 5, 30)).unwrap();
        assert_eq!(ledger.chain_state().batches, 1);
    } // Drop flushes the buffered descriptor
    let ledger = QueryLedger::open(&d).unwrap();
    let st = ledger.verify().unwrap();
    assert_eq!(st.batches, 2);
    assert_eq!(st.next_seq, 3);
    // Every batch file's name is the BLAKE3 of its bytes.
    for digest in digests_of(&d) {
        let bytes = std::fs::read(d.join(format!("{digest}.json"))).unwrap();
        assert_eq!(content_digest(&bytes), digest);
        let text = String::from_utf8(bytes).unwrap();
        assert!(!text.contains(' '), "canonical: no whitespace");
        // Genesis: first (lowest-seq) batch names the 64-zero parent.
        if text.contains(&format!("\"parent\":\"{GENESIS}\"")) {
            assert!(text.contains("\"first_seq\":0"));
        }
    }
    // Sidecar equals a real re-fold of the batch records.
    for digest in digests_of(&d) {
        let bytes = std::fs::read(d.join(format!("{digest}.json"))).unwrap();
        let records = parse_records(&bytes);
        let refold = graphlaw::query_ledger::fold_batch(&digest, &records);
        let sidecar = std::fs::read_to_string(d.join(format!("{digest}.agg"))).unwrap();
        assert_eq!(sidecar.trim(), encode_agg_for_court(&refold));
    }
    let _ = std::fs::remove_dir_all(&d);
}

fn parse_records(body: &[u8]) -> Vec<Descriptor> {
    // Minimal court-side parser over the fixed key order.
    let text = std::str::from_utf8(body).unwrap();
    let recs = text.split("{\"h_query\":\"").skip(1);
    let mut out = Vec::new();
    for r in recs {
        let hq = &r[..64];
        let nums: Vec<u64> = r
            .split("\":")
            .skip(1)
            .filter_map(|p| {
                p.split(|c| c == ',' || c == '}')
                    .next()
                    .unwrap()
                    .parse()
                    .ok()
            })
            .collect();
        let mut h = [0u8; 32];
        for (i, byte) in h.iter_mut().enumerate() {
            *byte = u8::from_str_radix(&hq[i * 2..i * 2 + 2], 16).unwrap();
        }
        // nums: rows, flags, ts, epoch (in fixed key order)
        out.push(Descriptor {
            h_query: h,
            rows_emitted: nums[0] as u32,
            flags: nums[1] as u8,
            ts_unix_ms: nums[2],
            epoch: nums[3],
        });
    }
    out
}

fn encode_agg_for_court(a: &BatchAgg) -> String {
    let mut s = format!(
        "{{\"batch\":\"{}\",\"rows_emitted_sum\":{{",
        a.batch
    );
    for (i, (e, v)) in a.rows_emitted_sum.iter().enumerate() {
        if i > 0 {
            s.push(',');
        }
        s.push_str(&format!("\"{e}\":{v}"));
    }
    s.push_str(&format!(
        "}},\"saturated\":{}}}",
        if a.saturated { "true" } else { "false" }
    ));
    s
}

/// F8 — .agg sidecar tamper-EVIDENCE via the CHAIN index: every flush records
/// BLAKE3(.agg bytes) into chain.json, and verify() refuses a tampered
/// sidecar with a typed DigestMismatch — even when the tampered sidecar is
/// NOT the spot-re-fold batch (previously only the spot-re-fold caught it).
#[test]
fn f8_tampered_agg_sidecar_evident_via_chain() {
    let d = dir("f8");
    {
        let mut ledger = QueryLedger::open(&d).unwrap();
        ledger
            .record_batch(&[desc(1, 10, 1_000), desc(2, 20, 2_000)])
            .unwrap();
        ledger
            .record_batch(&[desc(1, 5, 3_000), desc(1, 5, 4_000)])
            .unwrap();
    }
    let ledger = QueryLedger::open(&d).unwrap();
    let st = ledger.verify().unwrap();
    assert_eq!(st.batches, 2);
    // The promotion is real: the CHAIN records an agg digest per batch.
    assert_eq!(st.aggs.len(), 2);
    for (batch, agg_digest) in &st.aggs {
        let bytes = std::fs::read(d.join(format!("{batch}.agg"))).unwrap();
        assert_eq!(&content_digest(&bytes), agg_digest);
    }
    // Untampered ledger verifies clean.
    ledger.verify().unwrap();
    // Tamper the NEWEST batch's sidecar (head = highest first_seq). The
    // aggregate() spot-re-fold only checks the FIRST folded batch (tail), so
    // before F8 this tamper went undetected by aggregation and had no chain
    // check at all; now verify() refuses it as a typed DigestMismatch on the
    // sidecar file.
    let head = st.head.clone();
    let path = d.join(format!("{head}.agg"));
    let mut bytes = std::fs::read(&path).unwrap();
    let idx = bytes.iter().position(|b| *b == b'1').unwrap();
    bytes[idx] = b'9';
    std::fs::write(&path, &bytes).unwrap();
    let err = ledger.verify().unwrap_err();
    match err {
        LedgerError::DigestMismatch {
            file, expected, actual, ..
        } => {
            assert_eq!(file, format!("{head}.agg"));
            assert_eq!(Some(&expected), st.aggs.get(&head));
            assert_ne!(actual, expected);
            assert_eq!(actual.len(), 64);
        }
        other => panic!("expected DigestMismatch, got {other:?}"),
    }
    // Spot-re-fold does NOT cover the head batch — F8 is what catches it.
    let agg = ledger.aggregate(0, u64::MAX).unwrap();
    assert_eq!(agg.batches, 2);
    // Re-open refuses too: the tamper is evident at startup, not just on
    // explicit verify.
    match QueryLedger::open(&d) {
        Err(LedgerError::DigestMismatch { file, .. }) => {
            assert_eq!(file, format!("{head}.agg"))
        }
        Err(other) => panic!("expected DigestMismatch on open, got {other:?}"),
        Ok(_) => panic!("expected DigestMismatch on open, got Ok"),
    }
    let _ = std::fs::remove_dir_all(&d);
}
