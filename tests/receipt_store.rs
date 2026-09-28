//! Out-of-subject receipt store: real files on disk, no git, no test doubles.

use std::fs;
use std::path::PathBuf;

use graphlaw::dialect::Dialect;
use graphlaw::law::LawState;
use graphlaw::plan::{Action, Plan};
use graphlaw::receipt_store::{ReceiptStore, StoreError};

fn t(o: &str) -> String {
    format!("<urn:p:robot> <urn:p:at> <urn:p:{o}> .\n")
}

fn mv(name: &str, from: &str, to: &str) -> Action {
    Action {
        name: name.into(),
        pre: t(from),
        add: t(to),
        del: t(from),
    }
}

fn receipts() -> Vec<graphlaw::law::Receipt> {
    let s = LawState::parse(t("a").as_bytes(), Dialect::NTriples, None).unwrap();
    let plan = Plan {
        actions: vec![
            mv("a-b", "a", "b"),
            mv("b-c", "b", "c"),
            mv("c-d", "c", "d"),
        ],
        goal: t("d"),
    };
    plan.admit(&s).unwrap().receipts
}

fn scratch(name: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("graphlaw-store-{name}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&d);
    d
}

const SUBJECT: &str = "0123456789abcdef0123456789abcdef01234567";

#[test]
fn put_then_reopen_recovers_the_same_chain() {
    let dir = scratch("chain");
    let rs = receipts();
    let store = ReceiptStore::open(&dir).unwrap();
    // insert out of order: chain order is recovered from parent->child links
    for r in [&rs[2], &rs[0], &rs[1]] {
        store.put(r, SUBJECT).unwrap();
    }
    drop(store);
    let reopened = ReceiptStore::open(&dir).unwrap();
    assert_eq!(reopened.list(SUBJECT).unwrap().len(), 3);
    assert_eq!(reopened.verify(SUBJECT).unwrap(), rs);
    assert!(reopened.list("other-subject").unwrap().is_empty());
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn one_tampered_byte_is_refused() {
    let dir = scratch("tamper");
    let rs = receipts();
    let store = ReceiptStore::open(&dir).unwrap();
    let digests: Vec<_> = rs.iter().map(|r| store.put(r, SUBJECT).unwrap()).collect();
    let file = dir.join(SUBJECT).join(format!("{}.json", digests[1]));
    let mut bytes = fs::read(&file).unwrap();
    // flip the `added` digit: still well-formed canonical JSON, wrong content
    let pos = bytes.windows(8).position(|w| w == b"\"added\":").unwrap() + 8;
    bytes[pos] = if bytes[pos] == b'9' {
        b'8'
    } else {
        bytes[pos] + 1
    };
    fs::write(&file, &bytes).unwrap();
    match store.verify(SUBJECT) {
        Err(StoreError::DigestMismatch { file, .. }) => assert!(file.contains(&digests[1])),
        other => panic!("expected DigestMismatch, got {other:?}"),
    }
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn missing_receipt_breaks_the_chain() {
    let dir = scratch("gap");
    let rs = receipts();
    let store = ReceiptStore::open(&dir).unwrap();
    store.put(&rs[0], SUBJECT).unwrap();
    store.put(&rs[2], SUBJECT).unwrap();
    assert!(matches!(
        store.verify(SUBJECT),
        Err(StoreError::BrokenChain { .. })
    ));
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn subject_keys_cannot_escape_the_store() {
    let dir = scratch("subject");
    let store = ReceiptStore::open(&dir).unwrap();
    for bad in ["", "..", "a/b", "../x"] {
        assert!(matches!(
            store.put(&receipts()[0], bad),
            Err(StoreError::InvalidSubject(_))
        ));
    }
    let _ = fs::remove_dir_all(&dir);
}
