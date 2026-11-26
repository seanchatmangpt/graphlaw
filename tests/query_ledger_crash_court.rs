//! Crash court for the cold-path query ledger: chain.json is the single
//! atomic commit point. A crash after the batch/agg files but before
//! chain.json must recover (quarantine, never delete) without bricking.
//! Real files, real flushes; the crash is a real on-disk state reconstructed
//! from real flush output.

use graphlaw::query_ledger::{Descriptor, QueryLedger};
use std::path::{Path, PathBuf};

fn dir(name: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("ql-crash-{}-{}", name, std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    d
}

fn desc(epoch: u64, ts: u64) -> Descriptor {
    Descriptor {
        h_query: [3u8; 32],
        rows_emitted: 7,
        flags: 0,
        ts_unix_ms: ts,
        epoch,
    }
}

fn names(d: &Path) -> Vec<String> {
    let mut v: Vec<String> = std::fs::read_dir(d)
        .unwrap()
        .map(|e| e.unwrap().file_name().to_string_lossy().to_string())
        .collect();
    v.sort();
    v
}

/// Build a ledger with two committed batches; return (dir, chain.json bytes
/// after batch 1, head after batch 1).
fn crashed_after_batch_files(name: &str) -> (PathBuf, String) {
    let d = dir(name);
    let mut l = QueryLedger::open(&d).unwrap();
    l.record_batch(&[desc(1, 100), desc(1, 101)]).unwrap();
    let chain1 = std::fs::read(d.join("chain.json")).unwrap();
    let head1 = l.chain_state().head;
    l.record_batch(&[desc(2, 200)]).unwrap();
    drop(l);
    // Crash image: batch-2 files present, chain.json still at batch 1.
    std::fs::write(d.join("chain.json"), chain1).unwrap();
    (d, head1)
}

#[test]
fn crash_before_chain_commit_recovers_and_quarantines() {
    let (d, head1) = crashed_after_batch_files("recover");
    let before = names(&d);
    let l = QueryLedger::open(&d).expect("reopen must not brick");
    let st = l.chain_state();
    assert_eq!(st.head, head1, "committed head is chain.json's head");
    assert_eq!(st.batches, 1);
    assert_eq!(st.next_seq, 2);
    l.verify().expect("verifies after recovery");
    let q = names(&d.join("quarantine"));
    assert_eq!(q.len(), 2, "orphan .json and .agg quarantined: {q:?}");
    // Nothing deleted: every pre-crash file is either still present or quarantined.
    let after = names(&d);
    for n in before {
        assert!(after.contains(&n) || q.contains(&n), "{n} was deleted");
    }
}

#[test]
fn recovered_ledger_accepts_new_batches_and_survives_reopen() {
    let (d, _) = crashed_after_batch_files("continue");
    {
        let mut l = QueryLedger::open(&d).unwrap();
        l.record_batch(&[desc(3, 300)]).unwrap();
    }
    let l = QueryLedger::open(&d).unwrap();
    let st = l.verify().unwrap();
    assert_eq!(st.batches, 2);
    assert_eq!(st.next_seq, 3);
}

#[test]
fn crash_on_first_flush_without_chain_json_recovers() {
    let d = dir("first");
    {
        let mut l = QueryLedger::open(&d).unwrap();
        l.record_batch(&[desc(1, 1)]).unwrap();
    }
    std::fs::remove_file(d.join("chain.json")).unwrap();
    let l = QueryLedger::open(&d).expect("genesis crash recovers");
    assert_eq!(l.chain_state().batches, 0);
    assert_eq!(names(&d.join("quarantine")).len(), 2);
}

#[test]
fn clean_ledger_has_no_quarantine_and_stray_tmp_is_harmless() {
    let d = dir("clean");
    {
        let mut l = QueryLedger::open(&d).unwrap();
        l.record_batch(&[desc(1, 1)]).unwrap();
    }
    std::fs::write(d.join("chain.tmp"), b"torn").unwrap();
    let l = QueryLedger::open(&d).unwrap();
    l.verify().unwrap();
    assert!(!d.join("quarantine").exists());
}
