//! Evidentiary-ring startup-recovery courts (native-only, feature
//! `evidentiary-ring`). Chicago: real drainer thread, real NDJSON ledger on
//! disk, real bytes torn/forked by hand, readback through `verify`. No mocks,
//! no sleeps. Gated like `evidentiary_ring_court.rs`; the always-compiled
//! companion at the bottom is the loud skip.
#[cfg(all(feature = "evidentiary-ring", not(target_arch = "wasm32")))]
mod recovery_courts {
    use graphlaw::evidence_ring::{
        EvidenceRing, FixedClock, RING_CAPACITY, RingDescriptor, RingError, verify,
    };
    use std::path::{Path, PathBuf};
    use std::sync::Arc;

    fn tmp_dir(tag: &str) -> PathBuf {
        let d = std::env::temp_dir().join(format!(
            "graphlaw-ring-recovery-{tag}-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&d);
        std::fs::create_dir_all(&d).expect("create court dir");
        d
    }

    fn desc(i: u64) -> RingDescriptor {
        let mut h = [0u8; 32];
        h[..8].copy_from_slice(&i.to_be_bytes());
        RingDescriptor::new(h, i, 1, 0, 1_000)
    }

    /// Session that writes `n` evidence lines, then closes gracefully.
    fn seed_ledger(dir: &Path, n: u64) -> Vec<u8> {
        let ring = EvidenceRing::init_with_clock(dir, Arc::new(FixedClock(42))).expect("init");
        for i in 1..=n {
            ring.push(desc(i));
        }
        ring.close();
        std::fs::read(dir.join("evidence.ndjson")).expect("ledger bytes")
    }

    fn last_line(bytes: &[u8]) -> &[u8] {
        let body = &bytes[..bytes.len() - 1];
        let start = body.iter().rposition(|b| *b == b'\n').map_or(0, |i| i + 1);
        &bytes[start..]
    }

    #[test]
    fn torn_tail_recovers_to_last_valid_record() {
        let dir = tmp_dir("torn");
        let good = seed_ledger(&dir, 5);
        assert_eq!(verify(&dir).expect("seed verifies").0, 5);
        // Torn append: half of a real line, no terminating newline.
        let tail = last_line(&good);
        let mut torn = good.clone();
        torn.extend_from_slice(&tail[..tail.len() / 2]);
        std::fs::write(dir.join("evidence.ndjson"), &torn).expect("tear");
        assert!(
            matches!(verify(&dir), Err(RingError::Malformed { line: 6 })),
            "verify never accepts a torn tail"
        );
        let ring = EvidenceRing::init_with_clock(&dir, Arc::new(FixedClock(42)))
            .expect("torn tail is recoverable, not fatal");
        // Truncation happens inside init, before the drainer can write.
        assert_eq!(
            std::fs::read(ring.ledger_path()).expect("read"),
            good,
            "torn bytes truncated back to the last valid terminated record"
        );
        ring.close();
        assert_eq!(verify(&dir).expect("recovered ledger verifies"), (5, 0, 0));
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn push_after_torn_tail_recovery_continues_chain_and_survives_restart() {
        let dir = tmp_dir("torn-push");
        let good = seed_ledger(&dir, 5);
        let tail = last_line(&good);
        let mut torn = good.clone();
        torn.extend_from_slice(&tail[..tail.len() / 2]);
        std::fs::write(dir.join("evidence.ndjson"), &torn).expect("tear");
        let ring = EvidenceRing::init_with_clock(&dir, Arc::new(FixedClock(42)))
            .expect("torn tail recoverable");
        for i in 6..=8 {
            ring.push(desc(i));
        }
        ring.close();
        assert_eq!(
            verify(&dir).expect("chain continues past recovery, no SequenceGap"),
            (8, 0, 0)
        );
        let ring2 = EvidenceRing::init_with_clock(&dir, Arc::new(FixedClock(42)))
            .expect("second restart init succeeds");
        ring2.push(desc(9));
        ring2.close();
        assert_eq!(verify(&dir).expect("verify after 2nd restart").0, 9);
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn clean_restart_push_continues_sequence() {
        let dir = tmp_dir("clean-restart-push");
        seed_ledger(&dir, 4);
        let ring = EvidenceRing::init_with_clock(&dir, Arc::new(FixedClock(42))).expect("init");
        ring.push(desc(5));
        ring.close();
        assert_eq!(verify(&dir).expect("verify"), (5, 0, 0));
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn forked_tail_refuses_and_leaves_bytes_untouched() {
        let dir = tmp_dir("fork");
        let good = seed_ledger(&dir, 5);
        // Fork: a complete, well-formed, newline-terminated record whose parent
        // is not the chain head (line 2 replayed at the tail).
        let second = good
            .split(|b| *b == b'\n')
            .nth(1)
            .expect("line 2")
            .to_vec();
        let mut forked = good.clone();
        forked.extend_from_slice(&second);
        forked.push(b'\n');
        std::fs::write(dir.join("evidence.ndjson"), &forked).expect("fork");
        let err = EvidenceRing::init_with_clock(&dir, Arc::new(FixedClock(42)))
            .err()
            .expect("forked tail must refuse");
        assert!(matches!(err, RingError::BrokenChain { .. }), "{err}");
        assert_eq!(
            std::fs::read(dir.join("evidence.ndjson")).expect("read"),
            forked,
            "refusal never rewrites the ledger"
        );
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn corrupt_terminated_tail_refuses_never_genesis() {
        let dir = tmp_dir("corrupt");
        let good = seed_ledger(&dir, 3);
        let mut garbage = good.clone();
        garbage.extend_from_slice(b"not a record\n");
        std::fs::write(dir.join("evidence.ndjson"), &garbage).expect("garbage");
        let err = EvidenceRing::init_with_clock(&dir, Arc::new(FixedClock(42)))
            .err()
            .expect("garbage terminated line must refuse");
        assert!(matches!(err, RingError::Malformed { line: 4 }), "{err}");

        // Hash-invalid (tampered body) terminated final record also refuses.
        let text = String::from_utf8(good).expect("utf8");
        let tampered = text.replacen("\"rows_emitted\":3", "\"rows_emitted\":4", 1);
        assert_ne!(text, tampered);
        std::fs::write(dir.join("evidence.ndjson"), &tampered).expect("tamper");
        let err = EvidenceRing::init_with_clock(&dir, Arc::new(FixedClock(42)))
            .err()
            .expect("tampered tail must refuse");
        assert!(matches!(err, RingError::BrokenChain { .. }), "{err}");
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn injected_clock_makes_drop_lines_replay_deterministic() {
        let run = |tag: &str| -> Vec<u8> {
            let dir = tmp_dir(tag);
            let ring = EvidenceRing::init_with_clock(&dir, Arc::new(FixedClock(7))).expect("init");
            ring.set_drainer_paused(true);
            for i in 1..=(RING_CAPACITY as u64 + 10) {
                ring.push(desc(i));
            }
            ring.set_drainer_paused(false);
            ring.close();
            let bytes = std::fs::read(ring.ledger_path()).expect("read");
            let (_, dropped_lines, dropped_total) = verify(&dir).expect("verifies");
            assert_eq!((dropped_lines, dropped_total), (1, 10));
            std::fs::remove_dir_all(&dir).ok();
            bytes
        };
        let a = run("replay-a");
        let b = run("replay-b");
        assert_eq!(a, b, "same inputs + injected clock => byte-identical ledger");
        assert!(
            String::from_utf8(a).expect("utf8").contains("\"refused_at_ms\":7"),
            "drop line carries the injected clock, not wall time"
        );
    }
}

#[cfg(not(all(feature = "evidentiary-ring", not(target_arch = "wasm32"))))]
#[test]
fn evidence_ring_recovery_gate_loud_skip_notice() {
    eprintln!(
        "SKIP [evidence-ring-recovery-courts] reason: feature 'evidentiary-ring' disabled in this build; run: cargo test --features evidentiary-ring --test evidence_ring_recovery_court"
    );
}
