//! Differential chain court (lane L79, L76 rank-1 construction).
//!
//! The SAME (pn, ocel) entry stream verified under BOTH engines' fold:
//!
//! 1. insa's fold (`insa-proof/src/sync.rs::step`, ported here byte-for-byte):
//!    `h_i = blake3(h_prev || pn_be || ocel_be || tick_be64)` where tick is
//!    the top byte of the OCEL word. KAT head `05ea1aa0...` pinned with
//!    provenance from `insa-proof/tests/sync_as_replay.rs` (captured from
//!    strata `logchain::chain_head`, mirrored in
//!    ggen-marketplace packs/strata-chain-pack/witness.ttl).
//! 2. graphlaw's fold (`src/receipt_chain.rs`): `build_admission_frame`
//!    99-byte frame + `chain_from_frame` double-mixed fold.
//!
//! Divergence honesty: the two recipes are DIFFERENT (3-input step with tick
//! domain separation vs 99-byte frame with double-mixed prior hash). The
//! court characterizes the divergence — heads must differ — and asserts only
//! shared PROPERTIES: determinism and truncation monotonicity. This is not a
//! unification claim.

use blake3::Hasher;
use graphlaw::receipt_chain::frame::DenialPolarity;
use graphlaw::receipt_chain::{Andon, ReceiptMeta, build_admission_frame, chain_from_frame};

// ---------------------------------------------------------------------------
// Shared fixture (provenance: insa-proof/tests/sync_as_replay.rs `fixture()`)
// ---------------------------------------------------------------------------

const FIXTURE: [(u64, u64); 4] = [
    (0x0001_0000_0000_0001, 0x0A00_0000_0000_0001),
    (0x0002_0000_0000_0002, 0x0B00_0000_0000_0002),
    (0x0003_0000_0000_0003, 0x0C00_0000_0000_0003),
    (0x0004_0000_0000_0004, 0x0D00_0000_0000_0004),
];

/// Cross-repo KAT head over the fixture, captured from strata
/// `logchain::chain_head` (insa `sync_as_replay.rs`, byte-identical to
/// `strata-chain-pack/witness.ttl` `sw:sharedKatHead`).
const INSA_KAT_HEAD_HEX: &str = "05ea1aa096f45968aacb2c996d2b541e4b9fdef24f006e0204e1fa88d33f68a5";

/// graphlaw head over the same fixture via build_admission_frame +
/// chain_from_frame. Captured on first run of this court; pinned below.
const GRAPHLAW_HEAD_HEX: &str = "95163f2a6b24bebdfd19bf77cc60553640b8c9f5f5c54bc415c3b0da77e77598";

fn kat() -> [u8; 32] {
    let mut h = [0u8; 32];
    for (i, b) in INSA_KAT_HEAD_HEX
        .as_bytes()
        .chunks_exact(2)
        .enumerate()
        .take(32)
    {
        h[i] = u8::from_str_radix(std::str::from_utf8(b).unwrap(), 16).unwrap();
    }
    h
}

fn hex(h: &[u8; 32]) -> String {
    h.iter().map(|b| format!("{b:02x}")).collect()
}

// ---------------------------------------------------------------------------
// (a) insa fold port: h_i = blake3(h_prev || pn_be || ocel_be || tick_be64)
// ---------------------------------------------------------------------------

const GENESIS: [u8; 32] = [0u8; 32];

#[inline]
fn ocel_top_byte(w: u64) -> u8 {
    ((w >> 56) & 0xFF) as u8
}

#[inline]
fn insa_step(h: [u8; 32], pn: u64, ocel: u64) -> [u8; 32] {
    let mut hasher = Hasher::new();
    hasher.update(&h);
    hasher.update(&pn.to_be_bytes());
    hasher.update(&ocel.to_be_bytes());
    hasher.update(&u64::from(ocel_top_byte(ocel)).to_be_bytes());
    *hasher.finalize().as_bytes()
}

fn insa_fold(entries: &[(u64, u64)]) -> [u8; 32] {
    let mut h = GENESIS;
    for &(pn, ocel) in entries {
        h = insa_step(h, pn, ocel);
    }
    h
}

// ---------------------------------------------------------------------------
// (b) graphlaw fold over the same entries
// ---------------------------------------------------------------------------

fn graphlaw_fold(entries: &[(u64, u64)]) -> [u8; 32] {
    let mut chain = GENESIS;
    for (i, &(pn, ocel)) in entries.iter().enumerate() {
        // Deterministic mapping of (pn, ocel) -> graphlaw admission inputs:
        // payload commitment = blake3(pn_be || ocel_be); meta fields derived
        // from the entry index and words. Deterministic by construction — no
        // wall clock (ts_ns passed explicitly to build_admission_frame).
        let payload_hash = *blake3::hash(&{
            let mut b = Vec::with_capacity(16);
            b.extend_from_slice(&pn.to_be_bytes());
            b.extend_from_slice(&ocel.to_be_bytes());
            b
        })
        .as_bytes();

        let meta = ReceiptMeta {
            instruction_id: pn,
            activity_idx: i as u16,
            node_kind: 0,
            ts_ns: None,
            denial: DenialPolarity::ADMITTED,
            andon: Andon::Green,
            object_ids: Vec::new(),
            obligation_count: 0,
        };

        let frame = build_admission_frame(&payload_hash, &chain, &meta, ocel);
        chain = chain_from_frame(&chain, &frame);
    }
    chain
}

// ---------------------------------------------------------------------------
// The court
// ---------------------------------------------------------------------------

#[test]
fn insa_port_reproduces_the_kat_head() {
    assert_eq!(
        hex(&insa_fold(&FIXTURE)),
        INSA_KAT_HEAD_HEX,
        "in-test port of insa's 3-input step drifted from the cross-repo KAT"
    );
}

#[test]
fn both_folds_are_deterministic() {
    let insa_a = insa_fold(&FIXTURE);
    let insa_b = insa_fold(&FIXTURE);
    assert_eq!(insa_a, insa_b, "insa fold non-deterministic");
    assert_eq!(hex(&insa_a), INSA_KAT_HEAD_HEX);

    let g_a = graphlaw_fold(&FIXTURE);
    let g_b = graphlaw_fold(&FIXTURE);
    assert_eq!(g_a, g_b, "graphlaw fold non-deterministic");
    assert_eq!(
        hex(&g_a),
        GRAPHLAW_HEAD_HEX,
        "graphlaw head drifted from pin"
    );
}

#[test]
fn truncation_monotone_both_engines() {
    // Drop the last entry: BOTH engines produce a different head than the
    // full-stream head, so a verifier holding the full head refuses.
    let short = &FIXTURE[..3];
    let insa_full = insa_fold(&FIXTURE);
    let insa_short = insa_fold(short);
    assert_ne!(insa_short, insa_full, "insa: truncation not detected");
    assert_ne!(hex(&insa_short), INSA_KAT_HEAD_HEX);

    let g_full = graphlaw_fold(&FIXTURE);
    let g_short = graphlaw_fold(short);
    assert_ne!(g_short, g_full, "graphlaw: truncation not detected");
    assert_ne!(hex(&g_short), GRAPHLAW_HEAD_HEX);

    // Drop nothing: both accept (heads equal their own full-stream heads).
    assert_eq!(insa_fold(&FIXTURE), insa_full);
    assert_eq!(graphlaw_fold(&FIXTURE), g_full);
}

#[test]
fn heads_differ_across_engines() {
    let insa_head = insa_fold(&FIXTURE);
    let g_head = graphlaw_fold(&FIXTURE);
    assert_ne!(
        insa_head, g_head,
        "engines unexpectedly converged — different recipes must not produce \
         the same head over the same entries; investigate before trusting"
    );
    // Divergence honesty: pin both hexes so any drift on either side is loud.
    assert_eq!(hex(&insa_head), INSA_KAT_HEAD_HEX);
    assert_eq!(hex(&g_head), GRAPHLAW_HEAD_HEX);
    eprintln!("insa head     = {INSA_KAT_HEAD_HEX}");
    eprintln!("graphlaw head = {}", hex(&g_head));
}

#[test]
fn kat_parses_exactly_32_bytes() {
    // Guard on the KAT pin helper itself (anti-vacuity: a mis-parsed pin
    // would make every other assertion trivially wrong).
    let k = kat();
    assert_eq!(hex(&k), INSA_KAT_HEAD_HEX);
    assert_ne!(k, [0u8; 32]);
}
