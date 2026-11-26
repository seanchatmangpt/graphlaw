//! Concurrent `put_signed` of one digest: no `.json` without its `.sig`,
//! no temp-name collisions. Real filesystem directory.

use std::sync::{Arc, Barrier, atomic::{AtomicBool, Ordering}};
use std::path::PathBuf;

use graphlaw::attest::{SigningKey, sign_receipt};
use graphlaw::dialect::Dialect;
use graphlaw::law::{LawState, Step};
use graphlaw::receipt_store::ReceiptStore;

fn scratch(tag: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("graphlaw-atomicity-{tag}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    d
}

#[test]
fn concurrent_put_signed_never_exposes_json_without_sig() {
    let dir = scratch("sig");
    let store = Arc::new(ReceiptStore::open(&dir).unwrap());
    let st = LawState::parse(
        b"<urn:a:C> <http://www.w3.org/2000/01/rdf-schema#subClassOf> <urn:a:D> .\n\
          <urn:a:x> <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <urn:a:C> .\n",
        Dialect::NTriples,
        None,
    )
    .unwrap();
    let (_, receipt) = st.transition(&Step::EntailRdfs).unwrap();
    let key = SigningKey::from_seed([3; 32]);
    let att = sign_receipt(&key, &receipt);
    const N: usize = 16;
    let subject = "subj-1";
    let barrier = Arc::new(Barrier::new(N + 1));
    let done = Arc::new(AtomicBool::new(false));
    let sd = dir.join(subject);

    let mut hs = Vec::new();
    for _ in 0..N {
        let (store, receipt, att, barrier) =
            (store.clone(), receipt.clone(), att.clone(), barrier.clone());
        hs.push(std::thread::spawn(move || {
            barrier.wait();
            for _ in 0..20 {
                store.put_signed(&receipt, subject, &att).unwrap();
            }
        }));
    }
    let watcher = {
        let (done, barrier, sd) = (done.clone(), barrier.clone(), sd.clone());
        std::thread::spawn(move || {
            barrier.wait();
            let mut violations = 0usize;
            while !done.load(Ordering::SeqCst) {
                if let Ok(rd) = std::fs::read_dir(&sd) {
                    let names: Vec<String> = rd
                        .filter_map(|e| e.ok())
                        .map(|e| e.file_name().to_string_lossy().into_owned())
                        .collect();
                    for n in &names {
                        if let Some(d) = n.strip_suffix(".json") {
                            // .sig is published first and never removed.
                            if !sd.join(format!("{d}.sig")).exists() {
                                violations += 1;
                            }
                        }
                    }
                }
            }
            violations
        })
    };
    for h in hs {
        h.join().unwrap();
    }
    done.store(true, Ordering::SeqCst);
    assert_eq!(watcher.join().unwrap(), 0, ".json observed without .sig");

    let trusted = {
        let mut t = graphlaw::attest::TrustedKeys::new();
        t.insert(key.verifying_key());
        t
    };
    assert_eq!(store.verify_attested(subject, &trusted).unwrap().len(), 1);
    let leftovers: Vec<_> = std::fs::read_dir(&sd)
        .unwrap()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_name().to_string_lossy().starts_with(".tmp-"))
        .collect();
    assert!(leftovers.is_empty(), "temp files left behind: {leftovers:?}");
    let _ = std::fs::remove_dir_all(&dir);
}
