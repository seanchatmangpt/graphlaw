//! P28-SIL §3.2 evidentiary-ring courts (native-only, feature
//! `evidentiary-ring`). Chicago: real drainer thread, real NDJSON ledger on
//! disk, real table-readback via `verify` re-reading the file. No mocks, no
//! coordination sleeps — every wait is a bounded poll on a real observable
//! (ledger line counts with a deadline).

// LOUD-SKIP GATE (census law): the substantive courts below are gated on the
// `evidentiary-ring` feature (default OFF, Cargo.toml [features]). They live in
// `mod ring_courts` so the file still compiles ungated, and the always-compiled
// companion test at the bottom prints-and-fails a typed SKIP when the gate is
// closed — a default `cargo test` census can never read this target's
// "0 passed" as silent-by-accident.
#[cfg(all(feature = "evidentiary-ring", not(target_arch = "wasm32")))]
mod ring_courts {

use graphlaw::evidence_ring::{
    EvidenceRing, PushOutcome, RingDescriptor, RingError, RING_CAPACITY,
};
use std::os::unix::fs::PermissionsExt;
use std::time::{Duration, Instant};

fn tmp_dir(tag: &str) -> std::path::PathBuf {
    let d = std::env::temp_dir().join(format!(
        "graphlaw-ring-court-{tag}-{}-{}",
        std::process::id(),
        Instant::now().elapsed().as_nanos()
    ));
    std::fs::create_dir_all(&d).expect("create court dir");
    d
}

/// Bounded poll: runs `f` until it returns Some or the deadline expires.
fn poll_until<T>(deadline: Instant, mut f: impl FnMut() -> Option<T>) -> Option<T> {
    loop {
        if let Some(v) = f() {
            return Some(v);
        }
        if Instant::now() >= deadline {
            return None;
        }
        std::thread::yield_now();
    }
}

fn ledger_line_count(ring: &EvidenceRing) -> usize {
    std::fs::read_to_string(ring.ledger_path())
        .map(|t| t.lines().count())
        .unwrap_or(0)
}

fn desc(i: u64) -> RingDescriptor {
    let mut h = [0u8; 32];
    h[..8].copy_from_slice(&i.to_be_bytes());
    RingDescriptor::new(h, i, 1, 0, graphlaw::evidence_ring::now_ms())
}

#[test]
fn ring_descriptor_is_exactly_64_bytes_with_zero_pads() {
    assert_eq!(std::mem::size_of::<RingDescriptor>(), 64);
    assert_eq!(std::mem::align_of::<RingDescriptor>(), 8);
    let d = desc(1);
    assert!(d.pads_zero(), "lawful constructor keeps pads zero");
    assert_eq!(RING_CAPACITY, 4096);
}

#[test]
fn chain_integrity_under_100k_push_drain() {
    let dir = tmp_dir("chain100k");
    let ring = EvidenceRing::init(&dir).expect("ring init");
    const N: u64 = 100_000;
    // Push in chunks bounded below the 4096-slot capacity and poll the real
    // observable (ledger lines on disk) between chunks, so the happy path is
    // genuinely zero-loss instead of racing the drainer's wakeups.
    const CHUNK: u64 = 2048;
    for start in (0..N).step_by(CHUNK as usize) {
        for i in start..(start + CHUNK).min(N) {
            assert_eq!(ring.push(desc(i)), PushOutcome::Buffered, "no drops at {i}");
        }
        let want = (start + CHUNK).min(N) as usize;
        let ok = poll_until(Instant::now() + Duration::from_secs(60), || {
            (ledger_line_count(&ring) >= want).then_some(())
        });
        assert!(ok.is_some(), "ledger did not reach {want} lines in time");
    }
    // Bounded poll on the real observable: every drained line is on disk.
    let ok = poll_until(Instant::now() + Duration::from_secs(60), || {
        (ledger_line_count(&ring) >= N as usize).then_some(())
    });
    assert!(ok.is_some(), "ledger did not reach {N} lines in time");
    ring.close();
    assert_eq!(ring.pushed_total(), N);
    assert_eq!(ring.dropped_total(), 0, "zero-loss happy path");
    let (evidence, dropped_lines, dropped_total) =
        graphlaw::evidence_ring::verify(&dir).expect("ledger verifies");
    assert_eq!(evidence, N as usize);
    assert_eq!(dropped_lines, 0);
    assert_eq!(dropped_total, 0);
    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn torn_pad_entry_is_typed_refused_at_push_and_benign_twin_accepted() {
    let dir = tmp_dir("torn");
    let ring = EvidenceRing::init(&dir).expect("ring init");
    // Pads are private; the only route to a torn descriptor is the public
    // torn_via constructor (with_torn_pad family).
    let torn = desc(0).torn_via(|p1, _| *p1 = [1, 0, 0]);
    assert!(
        !torn.pads_zero(),
        "torn_via must actually tear the pads (court non-vacuity)"
    );
    assert!(
        torn.with_torn_pad(|p1, p2| *p1 != [0; 3] || *p2 != [0; 8]),
        "with_torn_pad inspection must observe the torn pads"
    );
    // ANTI-VACUITY FLIP: the torn entry is typed-refused; its benign twin
    // (identical payload, zero pads) is buffered through the same producer.
    assert_eq!(ring.push(torn), PushOutcome::RefusedTornPad);
    assert_eq!(ring.push(desc(0)), PushOutcome::Buffered);
    // A refusal mints no sequence: the benign twin is seq 1, not seq 2.
    assert_eq!(ring.pushed_total(), 1, "refused torn entry must not mint a sequence");
    let ok = poll_until(Instant::now() + Duration::from_secs(30), || {
        (ledger_line_count(&ring) >= 1).then_some(())
    });
    assert!(ok.is_some(), "benign twin not drained");
    ring.close();
    let (evidence, dropped_lines, dropped_total) =
        graphlaw::evidence_ring::verify(&dir).expect("ledger verifies with benign twin only");
    assert_eq!((evidence, dropped_lines, dropped_total), (1, 0, 0));
    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn tampered_ledger_line_breaks_chain() {
    let dir = tmp_dir("tamper");
    let ring = EvidenceRing::init(&dir).expect("ring init");
    ring.push(desc(1));
    let ok = poll_until(Instant::now() + Duration::from_secs(30), || {
        (ledger_line_count(&ring) >= 1).then_some(())
    });
    assert!(ok.is_some());
    ring.close();
    let path = ring.ledger_path();
    let text = std::fs::read_to_string(&path).expect("ledger read");
    let flipped = text.replacen("\"rows_emitted\":1", "\"rows_emitted\":2", 1);
    assert_ne!(text, flipped, "mutation must change the bytes");
    std::fs::write(&path, flipped).expect("tamper write");
    assert!(matches!(
        graphlaw::evidence_ring::verify(&dir),
        Err(RingError::BrokenChain { .. })
    ));
    std::fs::remove_dir_all(&dir).ok();
}

/// Min-of-rounds max push latency against one ring: `rounds` independent
/// rounds of `per` pushes; each round records its max, the fence binds the
/// minimum round max. Isolates scheduler preemption noise on the measuring
/// thread from real blocking in `push`.
fn min_of_rounds_max(ring: &EvidenceRing, i0: u64, per: u64, rounds: usize) -> Duration {
    let mut round_maxes = [Duration::ZERO; 64];
    let mut i = i0;
    for round in round_maxes.iter_mut().take(rounds) {
        for _ in 0..per {
            let t = Instant::now();
            let _ = ring.push(desc(i));
            *round = (*round).max(t.elapsed());
            i += 1;
        }
    }
    round_maxes[..rounds].iter().copied().min().expect("rounds non-empty")
}

/// One interleaved paired round-set: alternates UNSTALLED and STALLED rounds
/// of EQUAL push count so both legs sample the same load timeline and carry
/// identical preemption exposure. Per-round statistic is the MEDIAN push
/// latency: a scheduler preemption spike lands on a handful of pushes and
/// cannot move a 50k-sample median (witnessed: the MAX-based fence flaked
/// 2/10 under parallel load, one-push spikes to 649µs), while a real
/// never-block violation — push cost scaling with drainer state — inflates
/// the median wholesale. The fence binds min-of-rounds medians.
fn paired_ratio(
    ring: &EvidenceRing,
    i0: u64,
    per: u64,
    rounds: usize,
    pause: &dyn Fn(bool),
) -> (Duration, Duration) {
    fn round_median(ring: &EvidenceRing, per: u64, i: &mut u64) -> Duration {
        let mut samples: Vec<u32> = Vec::with_capacity(per as usize);
        for _ in 0..per {
            let t = Instant::now();
            let _ = ring.push(desc(*i));
            let ns = t.elapsed().as_nanos();
            samples.push(u32::try_from(ns.min(u128::from(u32::MAX))).unwrap_or(u32::MAX));
            *i += 1;
        }
        samples.sort_unstable();
        Duration::from_nanos(u64::from(samples[samples.len() / 2]))
    }
    let mut i = i0;
    let mut base_meds = [Duration::ZERO; 64];
    let mut stalled_meds = [Duration::ZERO; 64];
    for r in 0..rounds {
        pause(false);
        base_meds[r] = round_median(ring, per, &mut i);
        pause(true);
        stalled_meds[r] = round_median(ring, per, &mut i);
    }
    let base = base_meds[..rounds].iter().copied().min().expect("rounds");
    let stalled = stalled_meds[..rounds].iter().copied().min().expect("rounds");
    (base.max(Duration::from_nanos(100)), stalled)
}

#[test]
fn no_blocking_property_1m_pushes_vs_stalled_drainer() {
    let dir = tmp_dir("stall1m");
    let ring = EvidenceRing::init(&dir).expect("ring init");
    const N: u64 = 1_000_000;
    // Paired interleaved rounds (ratio-fence law): equal-length UNSTALLED and
    // STALLED rounds alternate so both legs carry identical preemption
    // exposure under parallel-test load. Unequal rounds flaked ~1-in-3
    // (witnessed: baseline min 292ns vs stalled spike 10-392µs when the
    // baseline window was 4x shorter than the stalled window).
    const ROUNDS: usize = 20;
    const PER: u64 = N / ROUNDS as u64; // 50k per half-round

    let (baseline, max_push) =
        paired_ratio(&ring, 0, PER, ROUNDS, &|p| ring.set_drainer_paused(p));
    // paired_ratio pushes N unstalled + N stalled half-rounds.
    assert_eq!(ring.pushed_total(), 2 * N);
    assert!(
        ring.dropped_total() > 0,
        "ANTI-VACUITY: a stalled drainer with a 4096-slot ring and 1M pushes MUST drop; a zero-drop pass proves nothing"
    );
    // RATIO fence (the no-blocking PROPERTY): push latency must not scale
    // with drainer state. K=10 is generous — blocking push scales
    // unboundedly with the stall, this binds it to the same-binary baseline.
    const K: u32 = 10;
    assert!(
        max_push <= K * baseline,
        "never-block RATIO fence violated: stalled max {max_push:?} > {K}x unstalled baseline {baseline:?}"
    );
    ring.set_drainer_paused(false);
    // close() drains-to-empty before the drainer exits; no O(n^2) re-read
    // poll needed — verify() after close is the exact readback.
    ring.close();
    let (evidence, dropped_lines, dropped_total) =
        graphlaw::evidence_ring::verify(&dir).expect("chain survives drops");
    assert_eq!(
        evidence as u64 + dropped_total,
        ring.pushed_total(),
        "evidence + drop lines must cover every attempted push exactly once"
    );
    assert!(dropped_lines >= 1, "drop lines must be chained in-ledger");
    println!(
        "no-blocking court: stalled max {max_push:?} vs unstalled baseline {baseline:?} (ratio fence K={K}), dropped {}, drop lines {dropped_lines}, evidence {evidence}",
        ring.dropped_total()
    );
    std::fs::remove_dir_all(&dir).ok();
}

/// Joint ring -> query-ledger court: the drainer drains <=256 descriptors per
/// cycle into `QueryLedger::record_batch`, drop-newest overflow becomes
/// persisted ledger gap markers, close() drains-to-empty + final flushes, and
/// the content-addressed chain verifies on a real re-open of the ledger
/// directory. The 1M/stalled-drainer never-block fence is a same-binary
/// RATIO against an unstalled baseline (see the ring-only court).
#[test]
fn joint_ring_to_query_ledger_court() {
    let dir = tmp_dir("joint");
    let ldir = tmp_dir("joint-ledger");
    let ring = EvidenceRing::init_with_ledger(&dir, &ldir).expect("ring+ledger init");
    const N: u64 = 1_000_000;
    // Same paired interleaved rounds as the ring-only court (equal-length
    // UNSTALLED/STALLED half-rounds, identical preemption exposure).
    const ROUNDS: usize = 20;
    const PER: u64 = N / ROUNDS as u64;
    let (baseline, max_push) =
        paired_ratio(&ring, 0, PER, ROUNDS, &|p| ring.set_drainer_paused(p));
    // paired_ratio runs BOTH half-rounds of PER pushes per round: 2N total.
    assert_eq!(ring.pushed_total(), 2 * N);
    assert!(
        ring.dropped_total() > 0,
        "ANTI-VACUITY: stalled drainer + 2M pushes MUST drop; a zero-drop pass proves nothing"
    );
    const K: u32 = 10;
    assert!(
        max_push <= K * baseline,
        "never-block RATIO fence violated: stalled max {max_push:?} > {K}x unstalled baseline {baseline:?}"
    );
    ring.set_drainer_paused(false);
    ring.close(); // drain-to-empty + final flush (Drop obligation honored)

    // QUIESCENCE (no-sleep law: poll a real observable, never a sleep):
    // after close() the drainer thread is joined, but the conservation
    // asserts below are only meaningful once the persisted chain head has
    // stopped advancing — read chain.json's next_seq and require two
    // consecutive identical reads (structural sync witness). A stable chain
    // head with a conservation mismatch is REAL lost accounting, not
    // assert-timing.
    let ledger_path = ldir.join("chain.json");
    let read_next_seq = || -> u64 {
        std::fs::read_to_string(&ledger_path)
            .ok()
            .and_then(|t| {
                t.split("\"next_seq\":")
                    .nth(1)
                    .and_then(|r| r.split(',').next())
                    .and_then(|v| v.trim().parse().ok())
            })
            .unwrap_or(0)
    };
    let mut prev = read_next_seq();
    let quiet = poll_until(Instant::now() + Duration::from_secs(60), || {
        let cur = read_next_seq();
        let stable = cur == prev && cur > 0;
        prev = cur;
        stable.then_some(())
    });
    assert!(
        quiet.is_some(),
        "ledger chain head never reached quiescence (next_seq kept advancing past {prev})"
    );
    let quiescent_next_seq = prev;

    // Real readback: re-open the ledger directory and verify the chain.
    let ledger = graphlaw::query_ledger::QueryLedger::open(&ldir).expect("ledger re-open");
    let state = ledger.verify().expect("joint ledger chain verifies");
    assert_eq!(
        state.next_seq, quiescent_next_seq,
        "chain.json head must match the re-opened ledger at quiescence"
    );
    assert!(
        state.next_seq > 0,
        "drained descriptors must be recorded as ledger batches"
    );

    // Overflow is surfaced, not laundered: the in-memory counter, the NDJSON
    // drop lines, and the persisted ledger gap markers must all agree.
    let gaps = ledger.gaps().expect("gap markers readable");
    assert!(
        !gaps.is_empty(),
        "drop-newest losses must be persisted as ledger gap markers"
    );
    let gap_covered: u64 = gaps.iter().map(|(f, l)| l - f + 1).sum();
    let ndjson = graphlaw::evidence_ring::verify(&dir)
        .expect("NDJSON chain survives the dual write");
    let (nd_ev, nd_drop_lines, nd_dropped) = ndjson;
    assert_eq!(
        gap_covered,
        ring.dropped_total(),
        "ledger gap markers must cover exactly the drops the ring counter reports \
         (three-way conservation: ring counter {}, NDJSON drop coverage {}, ledger gap coverage {}, gap ranges tail {:?})",
        ring.dropped_total(),
        nd_dropped,
        gap_covered,
        &gaps[std::cmp::max(gaps.len() as i64 - 6, 0) as usize..],
    );
    assert_eq!(
        state.next_seq + gap_covered,
        ring.pushed_total(),
        "ledger records + gap markers must cover every attempted push exactly once"
    );
    // Three-way conservation: NDJSON drop-line coverage is the third
    // independent surface and must agree exactly.
    assert_eq!(
        nd_dropped, gap_covered,
        "NDJSON drop coverage must equal ledger gap coverage"
    );
    assert_eq!(
        nd_ev as u64 + nd_dropped,
        ring.pushed_total(),
        "NDJSON evidence + drops must cover every attempted push exactly once"
    );
    assert_eq!(
        ring.last_ledger_error(),
        None,
        "drainer must record into the ledger without error"
    );

    // The NDJSON crash-evidence chain stays coherent under the dual write.
    let (evidence, dropped_lines, dropped_total) =
        graphlaw::evidence_ring::verify(&dir).expect("NDJSON chain survives the dual write");
    assert_eq!(evidence as u64 + dropped_total, ring.pushed_total());
    assert!(dropped_lines >= 1);
    let _ = dropped_lines;
    println!(
        "joint court: stalled max {max_push:?} vs baseline {baseline:?} (K={K}), dropped {}, ledger records {}, gap markers {}",
        ring.dropped_total(),
        state.next_seq,
        gaps.len()
    );
    std::fs::remove_dir_all(&dir).ok();
    std::fs::remove_dir_all(&ldir).ok();
}

/// Release-profile ABSOLUTE leg of the never-block fence. The dev-profile
/// courts carry the stall-independence RATIO (the property); this leg carries
/// the 10µs-class absolute number, which is only honest in an optimized
/// build (`--release`, debug_assertions off).
#[cfg(all(not(debug_assertions), feature = "evidentiary-ring"))]
#[test]
fn no_blocking_release_absolute_bound() {
    let dir = tmp_dir("release-abs");
    let ring = EvidenceRing::init(&dir).expect("ring init");
    ring.set_drainer_paused(true);
    const ROUNDS: usize = 10;
    const PER: u64 = 100_000;
    let max_push = min_of_rounds_max(&ring, 0, PER, ROUNDS);
    assert!(
        ring.dropped_total() > 0,
        "ANTI-VACUITY: stalled drainer MUST drop; a zero-drop pass proves nothing"
    );
    assert!(
        max_push < Duration::from_micros(10),
        "release-profile absolute never-block bound violated: stalled max {max_push:?} >= 10µs"
    );
    println!("release absolute leg: stalled max push {max_push:?} < 10µs");
    std::fs::remove_dir_all(&dir).ok();
}


/// Restart-mid-stream regression (fuzz case 4, SLA fix): a fresh ring
/// re-opened on used ring+ledger directories must NOT silently renumber its
/// records onto the persisted ledger head with gaps=[] and complete=true.
/// The resumed NDJSON coverage watermark reconciles the cold ledger at open
/// (open_with_producer), persisting the discontinuity as gap markers, while
/// the NDJSON chain typed-refuses the renumbered continuation.
#[test]
fn restart_mid_stream_persists_gap_not_silent_renumber() {
    let dir = tmp_dir("restart-gap");
    let ldir = tmp_dir("restart-gap-ledger");
    let r1 = EvidenceRing::init_with_ledger(&dir, &ldir).expect("ring session 1");
    for i in 0..50_000u64 {
        let _ = r1.push(desc(i));
    }
    r1.close(); // graceful session 1: NDJSON + ledger both cover all 50k
    let r2 = EvidenceRing::init_with_ledger(&dir, &ldir).expect("ring session 2");
    for i in 0..1_000u64 {
        let _ = r2.push(desc(9_000_000 + i));
    }
    r2.close();
    let ledger = graphlaw::query_ledger::QueryLedger::open(&ldir).expect("ledger re-open");
    let gaps = ledger.gaps().expect("gap markers readable");
    assert!(
        !gaps.is_empty(),
        "restart discontinuity must be persisted as gap markers, never silent renumbering"
    );
    assert!(
        matches!(
            graphlaw::evidence_ring::verify(&dir),
            Err(RingError::SequenceGap { .. })
        ),
        "NDJSON chain must typed-refuse the renumbered continuation"
    );
    std::fs::remove_dir_all(&dir).ok();
    std::fs::remove_dir_all(&ldir).ok();
}

/// F5 restart court (external producer watermark): a crash with in-flight
/// descriptors is reconciled by `init_with_ledger_restart` — the gap is
/// persisted exactly once with exact arithmetic (`accounted_descriptors` ==
/// producer attempted), aggregation over the gap is `complete: false`, and
/// the resumed sequence CONTINUES the global space (seeded watermark), never
/// silent renumbering. The NDJSON chain still verifies end-to-end.
#[test]
fn restart_court_crash_gap_arithmetic_exact_and_complete_false() {
    let dir = tmp_dir("restart-crash");
    let ldir = tmp_dir("restart-crash-ledger");
    // Session 1: 1000 pushes, chunked below capacity with bounded polls on
    // the real ledger observable — zero drops, graceful close.
    let r1 = EvidenceRing::init_with_ledger(&dir, &ldir).expect("ring session 1");
    const N1: u64 = 1_000;
    for i in 0..N1 {
        if i % 512 == 0 && i > 0 {
            let ok = poll_until(Instant::now() + Duration::from_secs(30), || {
                (ledger_line_count(&r1) >= i as usize).then_some(())
            });
            assert!(ok.is_some(), "session 1 stalled at {i}");
        }
        assert_eq!(r1.push(desc(i)), PushOutcome::Buffered);
    }
    r1.close();
    let ledger1 = graphlaw::query_ledger::QueryLedger::open(&ldir).expect("ledger session 1");
    assert_eq!(ledger1.chain_state().next_seq, N1, "graceful session 1 fully covered");
    assert!(ledger1.gaps().expect("gaps").is_empty());
    drop(ledger1);

    // CRASH: 7 descriptors were in flight (attempted but never drained) and
    // died with the process. The producer's monotonic counter says 1007.
    const ATTEMPTED: u64 = N1 + 7;

    // Session 2: restart-mid-stream with the producer watermark.
    let r2 = EvidenceRing::init_with_ledger_restart(&dir, &ldir, ATTEMPTED)
        .expect("ring session 2 (restart)");
    // The resumed ring continues the GLOBAL sequence space — no renumbering.
    assert_eq!(r2.pushed_total(), ATTEMPTED, "watermark-seeded sequence counter");
    const N2: u64 = 100;
    for i in 0..N2 {
        if i % 512 == 0 && i > 0 {
            let ok = poll_until(Instant::now() + Duration::from_secs(30), || {
                (ledger_line_count(&r2)
                    >= (N1 + (ATTEMPTED - N1) + i) as usize)
                    .then_some(())
            });
            assert!(ok.is_some(), "session 2 stalled at {i}");
        }
        let seq = ATTEMPTED + i + 1;
        let mut h = [0u8; 32];
        h[..8].copy_from_slice(&seq.to_be_bytes());
        assert_eq!(
            r2.push(RingDescriptor::new(h, i, 1, 0, graphlaw::evidence_ring::now_ms())),
            PushOutcome::Buffered
        );
    }
    r2.close();

    // Exact gap arithmetic: ONE marker of exactly the in-flight count.
    let ledger2 = graphlaw::query_ledger::QueryLedger::open(&ldir).expect("ledger session 2");
    let gaps = ledger2.gaps().expect("gap markers readable");
    assert_eq!(gaps, vec![(N1 + 1, ATTEMPTED)], "gap = exactly the crash loss, exactly once");
    let state = ledger2.verify().expect("chain verifies across restart");
    assert_eq!(state.next_seq, N1 + N2, "records continue the ledger sequence space");
    assert_eq!(
        graphlaw::query_ledger::QueryLedger::accounted_descriptors(&ldir).expect("accounted"),
        ATTEMPTED + N2,
        "records + gap markers cover every attempted push exactly once"
    );
    assert_eq!(r2.pushed_total(), ATTEMPTED + N2);
    // Loss is disclosed, never absorbed: periods over the gap are incomplete.
    assert!(
        !ledger2.aggregate(0, u64::MAX).unwrap().complete,
        "period containing the crash gap must aggregate complete:false"
    );
    // The NDJSON crash-evidence chain stays coherent across the restart.
    let (evidence, dropped_lines, dropped_total) =
        graphlaw::evidence_ring::verify(&dir).expect("NDJSON verifies across restart");
    assert_eq!(evidence as u64, N1 + N2);
    assert_eq!(dropped_lines, 1, "one startup drop line discloses the crash gap");
    assert_eq!(dropped_total, ATTEMPTED - N1);
    assert_eq!(r2.last_ledger_error(), None);
    println!(
        "restart court: gap {gaps:?}, records {}, accounted {} == attempted {}",
        state.next_seq,
        ATTEMPTED + N2,
        ATTEMPTED + N2
    );
    std::fs::remove_dir_all(&dir).ok();
    std::fs::remove_dir_all(&ldir).ok();
}

/// RUST-4 court (write-fail injection): the NDJSON write path fails for real
/// (the ledger file is chmod'd read-only, so the drainer's append-open gets
/// EACCES — a real OS failure, no mocks). Law: the drainer must surface a
/// TYPED BrokenChain carrying the failure identity and must NOT advance the
/// on-disk watermark (ledger bytes byte-identical), and the gate must FLIP:
/// the same directory drains cleanly once the file is writable again.
#[test]
fn ndjson_write_failure_surfaces_typed_and_never_advances_watermark() {
    let dir = tmp_dir("writefail");
    // Session 1 (healthy leg of the inline flip): one line on disk, no error.
    let r1 = EvidenceRing::init(&dir).expect("ring session 1");
    r1.push(desc(1));
    let ok = poll_until(Instant::now() + Duration::from_secs(30), || {
        (ledger_line_count(&r1) >= 1).then_some(())
    });
    assert!(ok.is_some(), "session 1 line not drained");
    assert!(r1.last_write_error().is_none(), "healthy leg must be error-free");
    r1.close();
    let path = r1.ledger_path();
    let before = std::fs::read(&path).expect("ledger read");

    // INJECTION: read-only ledger file -> the resumed drainer's append-open
    // fails with a real EACCES.
    let mut perms = std::fs::metadata(&path).expect("stat").permissions();
    perms.set_mode(0o444);
    std::fs::set_permissions(&path, perms.clone()).expect("chmod 0444");
    let r2 = EvidenceRing::init(&dir).expect("session 2 init (dir writable, file not)");
    for i in 1..=64u64 {
        let _ = r2.push(desc(100 + i));
    }
    // Bounded poll on the real observable: the typed write error surfaces.
    let err = poll_until(Instant::now() + Duration::from_secs(30), || {
        r2.last_write_error()
    });
    assert!(
        err.is_some(),
        "write-failure must surface typed within the deadline, never laundered"
    );
    assert!(
        matches!(err.as_ref().unwrap(), RingError::BrokenChain { detail }
            if detail.contains("failed"),
        ),
        "typed refusal must be BrokenChain, got {err:?}"
    );
    r2.close();

    // Watermark law: the on-disk ledger is byte-identical to before the
    // failed session — the failed batch did NOT advance the watermark and was
    // not silently lost (its loss is typed on r2 + covered by the closing
    // drop accounting in the drainer's error state).
    perms.set_mode(0o644);
    std::fs::set_permissions(&path, perms).expect("chmod back");
    let after = std::fs::read(&path).expect("ledger read after failed session");
    assert_eq!(
        before, after,
        "watermark must not advance on a failed write path (no silent loss)"
    );

    // ANTI-VACUITY FLIP: with the file writable again, the same directory
    // drains and verifies — the gate above is not vacuously green. The
    // watermark-seeded restart constructor continues the on-disk sequence
    // space (disk watermark = 1), never renumbers it.
    let ldir = tmp_dir("writefail-ledger");
    let r3 = EvidenceRing::init_with_ledger_restart(&dir, &ldir, 1).expect("ring session 3");
    r3.push(desc(2));
    let ok = poll_until(Instant::now() + Duration::from_secs(30), || {
        (ledger_line_count(&r3) >= 2).then_some(())
    });
    assert!(ok.is_some(), "flip leg: writable ledger must drain");
    r3.close();
    std::fs::remove_dir_all(&ldir).ok();
    assert!(
        r3.last_write_error().is_none(),
        "flip leg: healthy session must be error-free"
    );
    let (evidence, _, _) = graphlaw::evidence_ring::verify(&dir).expect("flip leg verifies");
    assert_eq!(evidence, 2, "both healthy sessions chained on disk");
    std::fs::remove_dir_all(&dir).ok();
}

/// RUST-4 court (concurrent init): two threads race `init_global` through a
/// real start barrier. Init-once semantics admit EXACTLY ONE ledger: the
/// loser gets the typed AlreadyInitialized refusal BEFORE performing any
/// init I/O (no second drainer thread can exist to interleave into the
/// ledger), and the winning ring verifies as a single coherent chain.
#[test]
fn concurrent_init_global_admits_exactly_one_ledger() {
    use std::sync::{Arc, Barrier};
    let dir = tmp_dir("global-race");
    let barrier = Arc::new(Barrier::new(2));
    let mut handles = Vec::new();
    for _ in 0..2 {
        let barrier = Arc::clone(&barrier);
        let dir = dir.clone();
        handles.push(std::thread::spawn(move || {
            barrier.wait(); // structural sync: both racers live before either starts
            EvidenceRing::init_global(&dir)
        }));
    }
    let mut wins = 0usize;
    let mut refusals = 0usize;
    for h in handles {
        match h.join().expect("racer thread") {
            Ok(()) => wins += 1,
            Err(RingError::AlreadyInitialized) => refusals += 1,
            Err(e) => panic!("unexpected typed error from racer: {e}"),
        }
    }
    assert_eq!(wins, 1, "exactly one racer may install the global ring");
    assert_eq!(
        refusals, 1,
        "the loser must be typed-refused, never silently co-installed"
    );
    // Single-identity readback: exactly one ledger exists, drains, verifies.
    let ring = EvidenceRing::global().expect("global ring installed by the winner");
    ring.push(desc(9));
    let ok = poll_until(Instant::now() + Duration::from_secs(30), || {
        (ledger_line_count(ring) >= 1).then_some(())
    });
    assert!(ok.is_some(), "global ring did not drain");
    ring.close();
    let (evidence, dropped_lines, dropped_total) =
        graphlaw::evidence_ring::verify(&dir).expect("single ledger verifies");
    assert_eq!((evidence, dropped_lines, dropped_total), (1, 0, 0));
    std::fs::remove_dir_all(&dir).ok();
}
}

/// ALWAYS-COMPILED loud-skip companion (census law): asserts the gate
/// condition for the feature-gated courts in `mod ring_courts`. When the gate
/// is closed this FAILS LOUDLY with a typed SKIP reason — a loud skip is not a
/// pass, and a gate that cannot exit 1 is the failure.
#[cfg(not(all(feature = "evidentiary-ring", not(target_arch = "wasm32"))))]
#[test]
fn evidentiary_ring_gate_loud_skip_notice() {
    panic!(
        "SKIP [evidentiary-ring-courts] reason: feature 'evidentiary-ring' disabled in this build (default OFF, Cargo.toml) — 9 ring/ledger courts compiled out; run: cargo test --features evidentiary-ring --test evidentiary_ring_court"
    );
}

