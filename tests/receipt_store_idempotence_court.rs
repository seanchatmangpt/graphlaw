//! Idempotence courts for `ReceiptStore`: true no-op re-put, typed `.sig`
//! conflict, orphan `.tmp-*` sweep on open, and `verify_from` past a forked retry.
//! Real files on a real temp dir; no doubles.

use std::path::PathBuf;
use std::time::Duration;

use graphlaw::attest::{SigningKey, sign_receipt};
use graphlaw::dialect::Dialect;
use graphlaw::law::{LawState, Receipt, Step};
use graphlaw::receipt_store::{ReceiptStore, StoreError};

fn scratch(tag: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("graphlaw-idem-{tag}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    d
}

fn receipt() -> Receipt {
    let st = LawState::parse(
        b"<urn:a:C> <http://www.w3.org/2000/01/rdf-schema#subClassOf> <urn:a:D> .\n\
          <urn:a:x> <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <urn:a:C> .\n",
        Dialect::NTriples,
        None,
    )
    .unwrap();
    st.transition(&Step::EntailRdfs).unwrap().1
}

fn mtime(p: &std::path::Path) -> std::time::SystemTime {
    std::fs::metadata(p).unwrap().modified().unwrap()
}

#[test]
fn identical_reput_is_a_true_noop() {
    let dir = scratch("noop");
    let store = ReceiptStore::open(&dir).unwrap();
    let r = receipt();
    let att = sign_receipt(&SigningKey::from_seed([3; 32]), &r);
    let d = store.put_signed(&r, "s", &att).unwrap();
    let json = dir.join("s").join(format!("{d}.json"));
    let sig = dir.join("s").join(format!("{d}.sig"));
    let (j0, s0) = (mtime(&json), mtime(&sig));
    // Poll-free: a rewrite would replace the inode; compare inode-independent
    // observables (mtime may tie), plus the absence of any leftover temp files.
    assert_eq!(store.put_signed(&r, "s", &att).unwrap(), d);
    assert_eq!(store.put(&r, "s").unwrap(), d);
    assert_eq!((mtime(&json), mtime(&sig)), (j0, s0));
    let names: Vec<_> = std::fs::read_dir(dir.join("s"))
        .unwrap()
        .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    assert_eq!(names.len(), 2, "{names:?}");
    assert_eq!(store.verify("s").unwrap(), vec![r]);
    std::fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn differing_sig_for_same_digest_is_a_typed_conflict() {
    let dir = scratch("conflict");
    let store = ReceiptStore::open(&dir).unwrap();
    let r = receipt();
    let a = sign_receipt(&SigningKey::from_seed([3; 32]), &r);
    let b = sign_receipt(&SigningKey::from_seed([4; 32]), &r);
    assert_ne!(a.to_json(), b.to_json());
    let d = store.put_signed(&r, "s", &a).unwrap();
    let err = store.put_signed(&r, "s", &b).unwrap_err();
    assert!(matches!(err, StoreError::SignatureConflict { ref file } if *file == format!("{d}.sig")));
    // The original attestation is untouched.
    let kept = std::fs::read_to_string(dir.join("s").join(format!("{d}.sig"))).unwrap();
    assert_eq!(kept, a.to_json());
    std::fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn open_sweeps_orphaned_tmp_files_but_keeps_receipts() {
    let dir = scratch("sweep");
    let store = ReceiptStore::open(&dir).unwrap();
    let r = receipt();
    let d = store.put(&r, "s").unwrap();
    let orphan = dir.join("s").join(format!(".tmp-{d}-json-999999-0"));
    std::fs::write(&orphan, b"partial").unwrap();
    let f = std::fs::File::options().write(true).open(&orphan).unwrap();
    f.set_modified(std::time::SystemTime::now() - Duration::from_secs(3600))
        .unwrap();
    drop(f);
    // A fresh tmp (live writer) survives open's grace window.
    let fresh = dir.join("s").join(format!(".tmp-{d}-json-999998-0"));
    std::fs::write(&fresh, b"inflight").unwrap();

    let reopened = ReceiptStore::open(&dir).unwrap();
    assert!(!orphan.exists(), "stale orphan must be removed on open");
    assert!(fresh.exists(), "young tmp belongs to a live writer");
    assert_eq!(reopened.sweep_orphans(Duration::ZERO).unwrap(), 1);
    assert!(!fresh.exists());
    assert_eq!(reopened.verify("s").unwrap(), vec![r]);
    std::fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn verify_from_head_ignores_an_earlier_forked_retry() {
    let dir = scratch("fork");
    let store = ReceiptStore::open(&dir).unwrap();
    let r1 = receipt();
    let mut r2 = r1.clone();
    r2.parent = r1.child.clone();
    r2.child = "blake3:tip".to_string();
    let mut retry = r1.clone();
    retry.lease_id = Some("retry".to_string()); // same parent, different digest: fork
    store.put(&r1, "s").unwrap();
    store.put(&r2, "s").unwrap();
    store.put(&retry, "s").unwrap();

    assert!(matches!(
        store.verify("s"),
        Err(StoreError::BrokenChain { .. })
    ));
    assert_eq!(store.verify_from("s", &r1.child).unwrap(), vec![r2.clone()]);
    assert_eq!(store.verify_from("s", "blake3:tip").unwrap(), vec![]);
    // A fork on the path from head still refuses.
    assert!(matches!(
        store.verify_from("s", &r1.parent),
        Err(StoreError::BrokenChain { .. })
    ));
    assert!(matches!(
        store.verify_from("s", "blake3:unknown"),
        Err(StoreError::BrokenChain { .. })
    ));
    std::fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn racing_different_attestations_yield_exactly_one_winner_and_a_conflict() {
    let r = receipt();
    for round in 0..20 {
        let dir = scratch(&format!("race{round}"));
        let store = ReceiptStore::open(&dir).unwrap();
        let a = sign_receipt(&SigningKey::from_seed([3; 32]), &r);
        let b = sign_receipt(&SigningKey::from_seed([4; 32]), &r);
        assert_ne!(a.to_json(), b.to_json());
        let barrier = std::sync::Barrier::new(2);
        let (ra, rb) = std::thread::scope(|sc| {
            let ha = sc.spawn(|| {
                barrier.wait();
                store.put_signed(&r, "s", &a)
            });
            let hb = sc.spawn(|| {
                barrier.wait();
                store.put_signed(&r, "s", &b)
            });
            (ha.join().unwrap(), hb.join().unwrap())
        });
        let d = graphlaw::receipt_store::receipt_digest(&r);
        let on_disk = std::fs::read_to_string(dir.join("s").join(format!("{d}.sig"))).unwrap();
        match (&ra, &rb) {
            (Ok(_), Err(StoreError::SignatureConflict { .. })) => assert_eq!(on_disk, a.to_json()),
            (Err(StoreError::SignatureConflict { .. }), Ok(_)) => assert_eq!(on_disk, b.to_json()),
            other => panic!("expected one winner and one SignatureConflict, got {other:?}"),
        }
        std::fs::remove_dir_all(&dir).unwrap();
    }
}
