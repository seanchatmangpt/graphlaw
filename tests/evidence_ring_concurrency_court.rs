//! Evidentiary-ring concurrent-producer court (native-only, feature
//! `evidentiary-ring`). N real threads x M pushes into one real ring with a
//! real drainer and NDJSON ledger; readback through `verify` plus a direct
//! strictly-increasing-seq scan of the ledger bytes. No mocks, no sleeps.
//!
//! Anti-vacuity: in `EvidenceRing::push` (src/evidence_ring.rs) move
//! `let seq = self.seq_counter.fetch_add(1, ..) + 1;` back ABOVE
//! `self.producer.lock()`. Threads can then enqueue seq k+1 before k; the
//! drainer sees a regression/gap and `verify` returns SequenceGap (or drop
//! lines appear), so this court fails. N*M < RING_CAPACITY so a correct
//! build can never legitimately drop.
#[cfg(all(feature = "evidentiary-ring", not(target_arch = "wasm32")))]
mod concurrency_courts {
    use graphlaw::evidence_ring::{
        EvidenceRing, FixedClock, PushOutcome, RING_CAPACITY, RingDescriptor, verify,
    };
    use std::sync::Arc;

    const THREADS: u64 = 8;
    const PER_THREAD: u64 = 400;

    fn desc(i: u64) -> RingDescriptor {
        let mut h = [0u8; 32];
        h[..8].copy_from_slice(&i.to_be_bytes());
        RingDescriptor::new(h, i, 1, 0, 1_000)
    }

    #[test]
    fn concurrent_pushes_keep_chain_and_sequence_intact() {
        assert!(((THREADS * PER_THREAD) as usize) < RING_CAPACITY);
        let dir = std::env::temp_dir().join(format!(
            "graphlaw-ring-concurrency-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("court dir");
        let ring = EvidenceRing::init_with_clock(&dir, Arc::new(FixedClock(42))).expect("init");

        let barrier = Arc::new(std::sync::Barrier::new(THREADS as usize));
        let handles: Vec<_> = (0..THREADS)
            .map(|t| {
                let ring = Arc::clone(&ring);
                let barrier = Arc::clone(&barrier);
                std::thread::spawn(move || {
                    barrier.wait();
                    for i in 0..PER_THREAD {
                        let o = ring.push(desc(t * PER_THREAD + i + 1));
                        assert!(matches!(o, PushOutcome::Buffered), "no drops expected: {o:?}");
                    }
                })
            })
            .collect();
        for h in handles {
            h.join().expect("producer thread");
        }
        ring.close();

        let total = THREADS * PER_THREAD;
        assert_eq!(
            verify(&dir).expect("concurrent ledger verifies"),
            (total as usize, 0, 0),
            "every push is one evidence line, zero drop lines"
        );
        let text = std::fs::read_to_string(dir.join("evidence.ndjson")).expect("ledger");
        let mut expect = 1u64;
        for line in text.lines() {
            let rest = line.strip_prefix("{\"seq\":").expect("seq-first line");
            let n: u64 = rest
                .split(',')
                .next()
                .and_then(|s| s.parse().ok())
                .expect("numeric seq");
            assert_eq!(n, expect, "strictly increasing, gapless seq");
            expect += 1;
        }
        assert_eq!(expect - 1, total);
        std::fs::remove_dir_all(&dir).ok();
    }
}

#[cfg(not(all(feature = "evidentiary-ring", not(target_arch = "wasm32"))))]
#[test]
fn concurrency_court_loud_skip() {
    eprintln!("SKIP: evidence_ring_concurrency_court needs feature evidentiary-ring on native");
}
