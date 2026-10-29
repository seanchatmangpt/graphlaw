//! Proof-lease court (WP P01): `verify_leased_proof` binds `check_proof`
//! to a caller lease digest — the BLAKE3 of the exact source bytes.
//!
//! Three rungs, all real engines, no doubles:
//!
//! 1. An honest generated proof checks under the lease that covers its own
//!    source bytes, and the digest round-trip (`pe:sourceDigest`) verifies.
//! 2. THE FLIP: a proof that is internally honest for source T is refused
//!    out of hand when the lease covers different bytes — the checker never
//!    runs. The twin leg shows the same pair passes under the lease that
//!    does cover T, so the refusal is the lease binding, not the proof.
//! 3. A tampered proof under the CORRECT lease still dies in the checker
//!    (C4): the lease binds what was checked, the hardened checker judges
//!    how. Both gates are exercised on one surface.
//!
//! Fixture discipline: every helper's result is bound to a local before an
//! assert references it.

use graphlaw::attest::{lease_digest, sign_bytes, verify_bytes, SigningKey, TrustedKeys};
use graphlaw::n3::{
    parse_n3_with_source, proof_to_n3, reason_document, CompletionStatus, ReasonerOptions,
};
use graphlaw::proof_lease::{verify_leased_proof, ProofLeaseError};

const LABEL: &str = "n3_proof_lease_court.n3";

const ORIGINAL_SOURCE: &str = "\
@prefix : <http://example.org/lease-court/> .

:tweety :is :chick .
:chick :is :bird .
:bird :is :animal .
{ ?s :is ?m . ?m :is ?a . } => { ?s :is ?a . } .
";

/// The same program with one fact's object changed — different bytes, same
/// shape, exactly what a splice or a stale replay presents.
const TAMPERED_SOURCE: &str = "\
@prefix : <http://example.org/lease-court/> .

:tweety :is :chick .
:chick :is :finch .
:bird :is :animal .
{ ?s :is ?m . ?m :is ?a . } => { ?s :is ?a . } .
";

fn emitted_proof(source: &str) -> String {
    let document = parse_n3_with_source(source, None, Some(LABEL)).expect("fixture parses");
    let options = ReasonerOptions {
        proof: true,
        ..ReasonerOptions::default()
    };
    let result = reason_document(&document, &options);
    assert_eq!(
        result.status,
        CompletionStatus::Complete,
        "fixture run must reach the fixpoint; limits={:?} errors={:?}",
        result.limits_reached,
        result.errors
    );
    proof_to_n3(&document.prefixes, &result)
}

#[test]
fn honest_proof_checks_under_the_lease_covering_its_own_source() {
    let proof = emitted_proof(ORIGINAL_SOURCE);
    let lease = lease_digest(ORIGINAL_SOURCE.as_bytes());
    let report = verify_leased_proof(&lease, ORIGINAL_SOURCE, &proof, LABEL)
        .expect("the lease covers these exact bytes");
    assert!(
        report.valid(),
        "an honest proof must check under its own lease: {:?}",
        report.failures
    );
    assert!(
        report.verified >= 3,
        "two facts + one rule application must verify; report: {report:?}"
    );
    assert!(
        report.obligations.is_empty(),
        "the digest round-trip must leave no obligations; report: {report:?}"
    );
    assert!(
        proof.contains("pe:sourceDigest"),
        "the writer must anchor fact steps with pe:sourceDigest"
    );
}

#[test]
fn cross_source_replay_is_refused_by_the_lease_before_the_checker_runs() {
    // The tampered pair is internally honest — it checks under its own
    // digest — so the only thing that can refuse it below is the lease.
    let tampered_proof = emitted_proof(TAMPERED_SOURCE);
    let own_lease = lease_digest(TAMPERED_SOURCE.as_bytes());
    let self_check = verify_leased_proof(&own_lease, TAMPERED_SOURCE, &tampered_proof, LABEL)
        .expect("the tampered pair is internally honest under its own lease");
    assert!(
        self_check.valid(),
        "the twin leg must pass for the flip to mean anything: {:?}",
        self_check.failures
    );

    // Replay it against a lease covering the ORIGINAL bytes: refused at the
    // gate, with both digests named.
    let original_lease = lease_digest(ORIGINAL_SOURCE.as_bytes());
    let outcome = verify_leased_proof(&original_lease, TAMPERED_SOURCE, &tampered_proof, LABEL);
    match outcome {
        Err(ProofLeaseError::LeaseDigestMismatch {
            lease_digest,
            source_digest,
        }) => {
            assert_eq!(lease_digest, original_lease);
            assert_eq!(source_digest, own_lease);
            assert_ne!(lease_digest, source_digest, "the fixtures must differ");
        }
        other => panic!("expected a lease digest mismatch, got {other:?}"),
    }
}

#[test]
fn tampered_proof_under_the_correct_lease_still_dies_in_the_checker() {
    let proof = emitted_proof(ORIGINAL_SOURCE);
    // Surgical tamper of the emitted claim, leaving step records intact.
    // Pinned by the assert below: exactly one line, the claim.
    let tampered = proof
        .lines()
        .map(|line| {
            if line.trim_start().starts_with(":tweety :is :bird") {
                line.replacen(":tweety", ":daffy", 1)
            } else {
                line.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join("\n");
    assert_ne!(proof, tampered, "the tamper must change the proof text");
    assert_eq!(
        proof.lines().count() - tampered.lines().count(),
        0,
        "the tamper must not add or drop lines"
    );

    let lease = lease_digest(ORIGINAL_SOURCE.as_bytes());
    let report = verify_leased_proof(&lease, ORIGINAL_SOURCE, &tampered, LABEL)
        .expect("the lease covers these bytes, so the checker runs");
    assert!(
        !report.valid(),
        "a tampered claim must die in the checker; report: {report:?}"
    );
    assert!(
        report
            .failures
            .iter()
            .any(|failure| failure.condition == "C4"),
        "the tampered claim must fail C4; report: {report:?}"
    );
}

#[test]
fn the_lease_digest_is_third_party_attestable() {
    // The binding value is plain BLAKE3 hex, so a caller can sign it and a
    // verifier can confirm which bytes were leased before trusting a
    // checked-proof receipt.
    let lease = lease_digest(ORIGINAL_SOURCE.as_bytes());
    let key = SigningKey::from_seed([11; 32]);
    let mut trusted = TrustedKeys::new();
    trusted.insert(key.verifying_key());
    let attestation = sign_bytes(&key, lease.as_bytes());
    assert!(verify_bytes(lease.as_bytes(), &attestation, &trusted).is_ok());
    // A receipt claiming other bytes does not verify.
    assert!(verify_bytes(b"other-bytes", &attestation, &trusted).is_err());
}
