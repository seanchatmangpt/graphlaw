//! Lease-bound proof verification (WP P01).
//!
//! `check_proof` re-performs a proof document's recorded inferences against
//! a source program — but nothing binds the check to the exact source bytes
//! a caller meant to check. [`verify_leased_proof`] closes that: the source
//! bytes are content-addressed (BLAKE3, the same digest the N3 parser mints
//! into every `SourceRef` at parse time and a fact step records as
//! `pe:sourceDigest`), and the check only runs when the caller's lease
//! digest matches. The checker's own hardened policy (P01 H1–H3) then
//! governs the proof document itself.
//!
//! No clocks, no sleeps: the lease binding is a content identity, judged at
//! the hop where the check runs.

use crate::n3::proof_check::Report;
use crate::n3::proof_check_n3::check_proof;

/// Why [`verify_leased_proof`] refused to check a proof.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProofLeaseError {
    /// The source bytes are not the bytes the lease covers. The checker
    /// never runs: the lease binding is the gate.
    LeaseDigestMismatch {
        /// What the caller's lease covers.
        lease_digest: String,
        /// The BLAKE3 of the source bytes as presented.
        source_digest: String,
    },
    /// The source program or the proof document is not readable N3.
    Parse(String),
}

impl std::fmt::Display for ProofLeaseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProofLeaseError::LeaseDigestMismatch { lease_digest, source_digest } => write!(
                f,
                "proof lease digest mismatch: the lease covers {lease_digest}, the presented source hashes to {source_digest}"
            ),
            ProofLeaseError::Parse(detail) => write!(f, "unreadable N3: {detail}"),
        }
    }
}

impl std::error::Error for ProofLeaseError {}

/// Verify a proof against the exact source bytes a caller lease covers,
/// under the hardened checker policy (P01).
///
/// The source is content-addressed first and held against `lease_digest`;
/// on any mismatch the checker does not run at all and the typed refusal
/// [`ProofLeaseError::LeaseDigestMismatch`] carries both digests. Only a
/// matching lease reaches `check_proof`, whose [`Report`] decides the
/// proof document itself.
///
/// # Examples
///
/// ```
/// use graphlaw::attest::lease_digest;
/// use graphlaw::proof_lease::{verify_leased_proof, ProofLeaseError};
///
/// const SOURCE: &str = "\
/// @prefix : <http://example.org/> .
/// :Socrates a :Human .
/// ";
/// let source_digest = lease_digest(SOURCE.as_bytes());
///
/// // A proof step for a fact the lease-covered source gives.
/// let proof = "\
/// @prefix : <http://example.org/> .
/// @prefix pe: <https://eyereasoner.github.io/pe#> .
///
/// :Socrates a :Human .
///
/// { :Socrates a :Human } pe:fact \"leased.n3\" .
/// ";
/// let report = verify_leased_proof(&source_digest, SOURCE, proof, "leased.n3").unwrap();
/// assert!(report.valid(), "{:?}", report.failures);
///
/// // The same proof against a lease for OTHER bytes is refused before the
/// // checker runs.
/// let other = lease_digest(b"@prefix : <http://example.org/> .\n:Plato a :Man .\n");
/// assert_eq!(
///     verify_leased_proof(&other, SOURCE, proof, "leased.n3"),
///     Err(ProofLeaseError::LeaseDigestMismatch {
///         lease_digest: other,
///         source_digest,
///     })
/// );
/// ```
pub fn verify_leased_proof(
    lease_digest: &str,
    source: &str,
    proof: &str,
    label: &str,
) -> Result<Report, ProofLeaseError> {
    let source_digest = lease_digest_source(source);
    if source_digest != lease_digest.trim().to_ascii_lowercase() {
        return Err(ProofLeaseError::LeaseDigestMismatch {
            lease_digest: lease_digest.to_string(),
            source_digest,
        });
    }
    check_proof(source, proof, label).map_err(|error| ProofLeaseError::Parse(error.to_string()))
}

/// The BLAKE3 hex of `source`'s exact bytes — the lease binding value.
pub fn lease_digest_source(source: &str) -> String {
    crate::attest::lease_digest(source.as_bytes())
}

#[cfg(test)]
mod tests {
    use super::*;

    const SOURCE: &str = "\
@prefix : <http://example.org/> .
:Socrates a :Human .
";

    #[test]
    fn mismatch_refuses_before_the_checker_runs() {
        let proof = "\
@prefix : <http://example.org/> .
@prefix pe: <https://eyereasoner.github.io/pe#> .

:Socrates a :Human .

{ :Socrates a :Human } pe:fact \"leased.n3\" .
";
        let other = crate::attest::lease_digest(b"not the leased bytes");
        let outcome = verify_leased_proof(&other, SOURCE, proof, "leased.n3");
        match outcome {
            Err(ProofLeaseError::LeaseDigestMismatch { lease_digest, source_digest }) => {
                assert_eq!(lease_digest, other);
                assert_eq!(source_digest, crate::attest::lease_digest(SOURCE.as_bytes()));
            }
            other => panic!("expected a lease digest mismatch, got {other:?}"),
        }
    }
}
