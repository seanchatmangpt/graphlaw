//! Proof-lease hardening court: byte caps before hashing, canonical lease
//! digest, typed refusals, and abi response paths free of panics.

use graphlaw::attest::lease_digest;
use graphlaw::proof_lease::{
    verify_leased_proof, ProofLeaseError, MAX_LEASED_PROOF_BYTES, MAX_LEASED_SOURCE_BYTES,
};

const SOURCE: &str = "@prefix : <http://example.org/> .\n:Socrates a :Human .\n";
const PROOF: &str = "\
@prefix : <http://example.org/> .
@prefix pe: <https://eyereasoner.github.io/pe#> .

:Socrates a :Human .

{ :Socrates a :Human } pe:fact \"h.n3\" .
";

#[test]
fn canonical_lease_admits() {
    let d = lease_digest(SOURCE.as_bytes());
    let report = verify_leased_proof(&d, SOURCE, PROOF, "h.n3");
    assert!(report.is_ok(), "{report:?}");
}

#[test]
fn uppercase_and_padded_digests_are_refused_not_normalized() {
    let d = lease_digest(SOURCE.as_bytes());
    let upper = d.to_ascii_uppercase();
    let padded = format!(" {d}\n");
    for bad in [upper, padded, d[..63].to_string(), format!("{d}0"), String::new()] {
        let got = verify_leased_proof(&bad, SOURCE, PROOF, "h.n3");
        assert!(matches!(got, Err(ProofLeaseError::MalformedLeaseDigest { len }) if len == bad.len()));
    }
    let nonhex = "g".repeat(64);
    assert!(matches!(
        verify_leased_proof(&nonhex, SOURCE, PROOF, "h.n3"),
        Err(ProofLeaseError::MalformedLeaseDigest { len: 64 })
    ));
}

#[test]
fn oversize_source_refused_before_hashing() {
    let big = "a".repeat(MAX_LEASED_SOURCE_BYTES + 1);
    // A malformed digest would otherwise win; the cap must be judged first.
    let got = verify_leased_proof("x", &big, PROOF, "h.n3");
    assert!(matches!(
        got,
        Err(ProofLeaseError::SourceTooLarge { observed, max })
            if observed == MAX_LEASED_SOURCE_BYTES + 1 && max == MAX_LEASED_SOURCE_BYTES
    ));
}

#[test]
fn oversize_proof_refused() {
    let big = "a".repeat(MAX_LEASED_PROOF_BYTES + 1);
    let d = lease_digest(SOURCE.as_bytes());
    assert!(matches!(
        verify_leased_proof(&d, SOURCE, &big, "h.n3"),
        Err(ProofLeaseError::ProofTooLarge { .. })
    ));
}

#[test]
fn mismatch_and_parse_stay_typed() {
    let other = lease_digest(b"other");
    assert!(matches!(
        verify_leased_proof(&other, SOURCE, PROOF, "h.n3"),
        Err(ProofLeaseError::LeaseDigestMismatch { .. })
    ));
    let bad_src = "@prefix : <http://example.org/ .\n";
    let d = lease_digest(bad_src.as_bytes());
    assert!(matches!(
        verify_leased_proof(&d, bad_src, PROOF, "h.n3"),
        Err(ProofLeaseError::Parse(_))
    ));
}

#[test]
fn abi_call_on_hostile_requests_returns_typed_json() {
    for req in [&b"\xff\xfe"[..], b"", b"{", b"[]", b"{\"op\":1}"] {
        let v: serde_json::Value =
            serde_json::from_slice(&graphlaw::abi::call(req)).expect("typed JSON response");
        assert_eq!(v["ok"], false);
    }
}

#[test]
fn abi_source_has_no_request_path_expect() {
    let src = include_str!("../src/abi.rs");
    let prod = src
        .split("#[cfg(test)]")
        .next()
        .expect("split always yields a first segment");
    assert!(!prod.contains(".expect("), "abi.rs must not .expect( on request/response paths");
    assert!(!prod.contains(".unwrap()"), "abi.rs must not .unwrap()");
}
