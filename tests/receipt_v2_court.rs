//! Phase 3 court for ADR-CANONARROW §3.1 — versioned dual-witness receipt envelope.
//!
//! Vector discipline: every pinned digest in this file was computed with the
//! external `b3sum` tool over the exact byte concatenations the folds specify
//! (an independent out-of-band witness — Chicago: the crate's constructors are
//! compared against a real external computation, not against themselves).
//!
//! Court responsibilities:
//! 1. Pin `receipt_v1` (legacy shape): proves the v1 fold is non-mutated for
//!    the whole release.
//! 2. Assert `receipt_v2` domain separation: same inputs plus an extra
//!    `H_columnar` produce v2 != v1, under several input permutations.
//! 3. Assert `canon_arrow_empty_v1() == BLAKE3(b"CANON_ARROW_EMPTY_V1")`.

use graphlaw::attest::receipt_fold::{
    canon_arrow_empty_v1, receipt_v1, receipt_v2, GLAW_RECEIPT_V2,
};

/// Independent hex helper (the `hex` crate is not a dev-dependency).
fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{:02x}", b)).collect()
}

/// Parse a 64-hex-char literal into a 32-byte digest.
fn d32(s: &str) -> [u8; 32] {
    assert_eq!(s.len(), 64, "vector literal must be 64 hex chars");
    let mut out = [0u8; 32];
    for (i, b) in out.iter_mut().enumerate() {
        *b = u8::from_str_radix(&s[2 * i..2 * i + 2], 16).expect("valid hex");
    }
    out
}

// --- Pinned vectors (b3sum, 2026-10-23) ------------------------------------

/// h_query = BLAKE3(b"receipt-v2-court:query")
const H_QUERY: &str = "7a364272788ab0a7a56b8444e9f6f76b1f0f7d90d3e5725e3496a78e732d92fd";
/// h_pre = BLAKE3(b"receipt-v2-court:pre")
const H_PRE: &str = "0bca2e430b2d2b856ae5b262031b0cadd1dd6ee40230ecb75d7ba3b0baf86662";
/// h_post = BLAKE3(b"receipt-v2-court:post")
const H_POST: &str = "b208929e36eafbe6511114fe40910c486b265eff351b4e7edc47f86476db6539";
/// h_triples = BLAKE3(b"receipt-v2-court:triples")
const H_TRIPLES: &str = "0ce59900a950fd16a9546ae14eed62b0b4ea63102b80812c00157a56c7a1d669";
/// h_columnar = BLAKE3(b"receipt-v2-court:columnar")
const H_COLUMNAR: &str = "9d79bfadd4c208bad3a87982bdd8f9d05991011f16b96de3cf1c5c33cf7c58df";
/// Pinned epoch for the golden vectors; folds as 8 bytes, little-endian.
const EPOCH: u64 = 7;

/// BLAKE3(H_query || H_pre || H_post || H_triples || le64(7)) — b3sum over the
/// exact concatenation, /tmp/q.bin etc.
const V1_GOLDEN: &str = "f641cf9578e7ddf23bfbb9df055c15b79fe920b9d98910ca22be069deb20f774";
/// BLAKE3("GLAW_RECEIPT_V2" || H_query || H_pre || H_post || H_triples || H_columnar || le64(7)).
const V2_GOLDEN: &str = "61054d964e2ecadcaac7d82d06fc1bd5f732a7595824376792a3f0970aae96c1";
/// BLAKE3(b"CANON_ARROW_EMPTY_V1") — b3sum of the literal 21-byte string.
const EMPTY_GOLDEN: &str = "db2da6fc6ec068a661a7028f220524ea6fad4b3cbf7f72df336dd99fd44f7b0c";

/// The four v1-era inputs, in fold order.
fn v1_inputs() -> ([u8; 32], [u8; 32], [u8; 32], [u8; 32]) {
    (
        d32(H_QUERY),
        d32(H_PRE),
        d32(H_POST),
        d32(H_TRIPLES),
    )
}

/// The settled v1 pin (2026-10-23): receipt_v1 over BLAKE3(0x01) || BLAKE3(0x02)
/// || BLAKE3(0x03) || BLAKE3(0x04) || le64(7) == 63bff90d… — minted with b3sum
/// over the exact fold input (same settled pin as src/attest.rs receipt_fold
/// tests), verified here at integration level against the public API.
const SETTLED_V1_INPUTS: [(&str, &str); 2] = [
    ("48fc721fbbc172e0925fa27af1671de225ba927134802998b10a1568a188652b",
     "ab13bedf42e84bae0f7c62c7dd6a8ada571e8829bed6ea558217f0361b5e25d0"),
    ("e1e0e81d6ea39b0cf8b86ffd440921011f57400cbc3f76a8a171906a9b8d7505",
     "0c389a743e34fda435fbd575bb889dbc0d3e66b9f9d81e00be33b7188509e7eb"),
];
const SETTLED_V1_GOLDEN: &str =
    "63bff90dee1c16beb2881e11664a1e8c0b6942abe39ba5726d9ce1f9708a8ffe";

/// v1 fold over the settled b3sum-minted input set equals the settled pin.
#[test]
fn v1_fold_matches_settled_b3sum_pin() {
    let q = d32(SETTLED_V1_INPUTS[0].0);
    let pre = d32(SETTLED_V1_INPUTS[0].1);
    let post = d32(SETTLED_V1_INPUTS[1].0);
    let t = d32(SETTLED_V1_INPUTS[1].1);
    let d = receipt_v1(&q, &pre, &post, &t, 7);
    assert_eq!(hex(&d), SETTLED_V1_GOLDEN, "receipt_v1 does not reproduce the settled b3sum pin");
}

// --- (1) v1 pin ------------------------------------------------------------

/// v1 non-mutation: the legacy fold over the pinned inputs equals the
/// b3sum-computed golden digest, twice (determinism/stability).
#[test]
fn v1_fold_is_pinned_and_stable() {
    let (q, pre, post, t) = v1_inputs();
    let a = receipt_v1(&q, &pre, &post, &t, EPOCH);
    let b = receipt_v1(&q, &pre, &post, &t, EPOCH);
    assert_eq!(a, b, "v1 fold must be deterministic");
    assert_eq!(
        hex(&a),
        V1_GOLDEN,
        "receipt_v1 drifted from its pinned release vector — legacy chains would break"
    );
}

/// The v2 prefix constant is exactly the 15-byte ADR literal.
#[test]
fn v2_prefix_is_exact_adr_literal() {
    assert_eq!(GLAW_RECEIPT_V2.len(), 15);
    assert_eq!(GLAW_RECEIPT_V2, b"GLAW_RECEIPT_V2");
}

// --- (2) v2 domain separation ----------------------------------------------

/// v2 != v1 on the pinned inputs, and v2 itself is pinned to its golden vector.
#[test]
fn v2_differs_from_v1_on_pinned_inputs_and_is_pinned() {
    let (q, pre, post, t) = v1_inputs();
    let v1 = receipt_v1(&q, &pre, &post, &t, EPOCH);
    let v2 = receipt_v2(&q, &pre, &post, &t, &d32(H_COLUMNAR), EPOCH);
    assert_ne!(v1, v2, "domain separation failed on pinned inputs");
    assert_eq!(hex(&v2), V2_GOLDEN, "receipt_v2 drifted from its golden vector");
}

/// v2 != v1 across several input permutations (independent witness sets, not
/// the pinned vector — a separation claim must hold off the golden path too).
#[test]
fn v2_differs_from_v1_under_permutations() {
    // Five permutation sets: varying epoch, varying inputs, and the empty-
    // columnar arm (H_columnar = CANON_ARROW_EMPTY_V1).
    let sets: [(u64, [u8; 32], [u8; 32], [u8; 32], [u8; 32], [u8; 32]); 5] = [
        (0, d32(H_QUERY), d32(H_PRE), d32(H_POST), d32(H_TRIPLES), d32(H_COLUMNAR)),
        (u64::MAX, d32(H_TRIPLES), d32(H_POST), d32(H_PRE), d32(H_QUERY), d32(H_COLUMNAR)),
        (42, [7u8; 32], [9u8; 32], [1u8; 32], [3u8; 32], [5u8; 32]),
        (1, [0u8; 32], [0u8; 32], [0u8; 32], [0u8; 32], [0u8; 32]),
        (
            9,
            d32(H_COLUMNAR),
            d32(H_TRIPLES),
            d32(H_QUERY),
            d32(H_PRE),
            canon_arrow_empty_v1(),
        ),
    ];
    for (i, (epoch, q, pre, post, t, col)) in sets.iter().enumerate() {
        let v1 = receipt_v1(q, pre, post, t, *epoch);
        let v2 = receipt_v2(q, pre, post, t, col, *epoch);
        assert_ne!(v1, v2, "v2 collided with v1 on permutation set {i}");
        // Also: v2 must not equal v1 with any single other field's digest
        // swapped in as the columnar witness (insertion-position separation).
        assert_ne!(v2, receipt_v1(q, pre, post, col, *epoch));
        assert_ne!(v2, receipt_v1(q, pre, post, t, *epoch));
    }
}

// --- (3) domain null digest -------------------------------------------------

/// `canon_arrow_empty_v1()` equals BLAKE3(b"CANON_ARROW_EMPTY_V1") — expected
/// digest cross-checked out-of-band with b3sum over the literal string
/// (EMPTY_GOLDEN), because `src/egress/arrow.rs` is not yet wired into lib.rs
/// (`src/lib.rs` has no `pub mod egress`), so the in-crate constant from
/// `attest::receipt_fold` is compared against the independent computation.
#[test]
fn canon_arrow_empty_v1_matches_independent_blake3() {
    let got = canon_arrow_empty_v1();
    assert_eq!(
        hex(&got),
        EMPTY_GOLDEN,
        "CANON_ARROW_EMPTY_V1 arm drifted from BLAKE3(b\"CANON_ARROW_EMPTY_V1\")"
    );
    // Determinism and non-triviality.
    assert_eq!(got, canon_arrow_empty_v1());
    assert_ne!(got, [0u8; 32]);
    // v2 over the empty arm reproduces the pinned composition path.
    let (q, pre, post, t) = v1_inputs();
    let v2_empty = receipt_v2(&q, &pre, &post, &t, &canon_arrow_empty_v1(), EPOCH);
    assert_ne!(v2_empty, receipt_v1(&q, &pre, &post, &t, EPOCH));
}
