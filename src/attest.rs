//! Offline-verifiable Ed25519 attestations over receipts and leases.
//!
//! A plain [`Receipt`] is only a record: anyone able to write triples can write
//! one. An [`Attestation`] binds a canonical payload (sorted-key, no-whitespace
//! JSON) to an Ed25519 key. Signing is deterministic (RFC 8032; no RNG) and
//! verification needs only the payload, the attestation and a caller-supplied
//! [`TrustedKeys`] set: no network, no clock.
//!
//! Attestation fields (all lowercase hex): `payload_sha256` is the SHA-256 of
//! the canonical payload, `key_id` is the SHA-256 of the 32-byte public key,
//! `signature` is the 64-byte Ed25519 signature over the payload bytes.

use std::collections::BTreeMap;
use std::fmt;

use ed25519_dalek::{
    Signature, Signer, SigningKey as DalekSigning, VerifyingKey as DalekVerifying,
};
use sha2::{Digest, Sha256};

use crate::law::{Lease, Receipt, SignedLease};

/// Why an attestation was refused.
///
/// This enum is `#[non_exhaustive]`: variants may be added in a minor release, so
/// downstream `match` expressions need a wildcard arm.
///
/// ```
/// use graphlaw::attest::AttestError;
///
/// let e = AttestError::UnknownKey;
/// let trusted_later = match e {
///     AttestError::UnknownKey => true,
///     _ => false, // required: new variants may be added
/// };
/// assert!(trusted_later);
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum AttestError {
    /// The signature does not verify, or the payload no longer matches the
    /// digest the attestation was issued over.
    BadSignature,
    /// The attestation's `key_id` is not in the trusted key set.
    UnknownKey,
    /// A key, signature or attestation document is not well-formed.
    Malformed(String),
}

impl fmt::Display for AttestError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AttestError::BadSignature => write!(f, "bad signature"),
            AttestError::UnknownKey => write!(f, "attestation key is not trusted"),
            AttestError::Malformed(m) => write!(f, "malformed attestation: {m}"),
        }
    }
}

impl std::error::Error for AttestError {}

/// Lowercase hex encoding.
pub fn hex_encode(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

/// Hex decoding (either case); `None` on odd length or a non-hex digit.
pub fn hex_decode(s: &str) -> Option<Vec<u8>> {
    if !s.len().is_multiple_of(2) || !s.is_ascii() {
        return None;
    }
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16).ok())
        .collect()
}

fn sha256_hex(bytes: &[u8]) -> String {
    hex_encode(&Sha256::digest(bytes))
}

fn malformed(m: impl Into<String>) -> AttestError {
    AttestError::Malformed(m.into())
}

/// An Ed25519 signing key (32-byte seed). Debug output never shows the seed.
pub struct SigningKey(DalekSigning);

impl fmt::Debug for SigningKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "SigningKey(key_id={})", self.verifying_key().key_id())
    }
}

impl SigningKey {
    /// Build a signing key from a caller-supplied 32-byte seed (no RNG is used).
    pub fn from_seed(seed: [u8; 32]) -> Self {
        SigningKey(DalekSigning::from_bytes(&seed))
    }

    /// Build a signing key from a 64-character hex seed.
    pub fn from_seed_hex(hex: &str) -> Result<Self, AttestError> {
        let bytes = hex_decode(hex).ok_or_else(|| malformed("seed is not hex"))?;
        let seed: [u8; 32] = bytes
            .try_into()
            .map_err(|_| malformed("seed must be 32 bytes"))?;
        Ok(Self::from_seed(seed))
    }

    /// The public half of this signing key.
    pub fn verifying_key(&self) -> VerifyingKey {
        VerifyingKey(self.0.verifying_key())
    }
}

/// An Ed25519 public key.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VerifyingKey(DalekVerifying);

impl VerifyingKey {
    /// Parse a verifying key from its 32 raw bytes.
    pub fn from_bytes(bytes: &[u8; 32]) -> Result<Self, AttestError> {
        DalekVerifying::from_bytes(bytes)
            .map(VerifyingKey)
            .map_err(|_| malformed("not a valid Ed25519 public key"))
    }

    /// Parse a verifying key from 64 hex characters.
    pub fn from_hex(hex: &str) -> Result<Self, AttestError> {
        let bytes = hex_decode(hex).ok_or_else(|| malformed("public key is not hex"))?;
        let arr: [u8; 32] = bytes
            .try_into()
            .map_err(|_| malformed("public key must be 32 bytes"))?;
        Self::from_bytes(&arr)
    }

    /// Lower-case hex of the 32 raw key bytes.
    pub fn to_hex(&self) -> String {
        hex_encode(self.0.as_bytes())
    }

    /// SHA-256 of the public key bytes, lowercase hex.
    pub fn key_id(&self) -> String {
        sha256_hex(self.0.as_bytes())
    }
}

/// The set of public keys a verifier is willing to accept, keyed by `key_id`.
#[derive(Debug, Clone, Default)]
pub struct TrustedKeys(BTreeMap<String, VerifyingKey>);

impl TrustedKeys {
    /// An empty trust set.
    pub fn new() -> Self {
        Self::default()
    }

    /// Trust `key`.
    pub fn insert(&mut self, key: VerifyingKey) {
        self.0.insert(key.key_id(), key);
    }

    /// Build from hex public keys.
    pub fn from_hex<I, S>(keys: I) -> Result<Self, AttestError>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        let mut t = Self::new();
        for k in keys {
            t.insert(VerifyingKey::from_hex(k.as_ref())?);
        }
        Ok(t)
    }

    /// Look up a trusted key by its key id.
    pub fn get(&self, key_id: &str) -> Option<&VerifyingKey> {
        self.0.get(key_id)
    }

    /// True when no key is trusted.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Number of trusted keys.
    pub fn len(&self) -> usize {
        self.0.len()
    }
}

/// Detached signature over a canonical payload.
///
/// ```
/// use graphlaw::{attest::{sign_receipt, verify_receipt, SigningKey, TrustedKeys}, dialect::Dialect, law::{LawState, Step}};
///
/// let key = SigningKey::from_seed([9; 32]);
/// let mut trusted = TrustedKeys::new();
/// trusted.insert(key.verifying_key());
/// let s = LawState::parse(b"<urn:a> <urn:p> <urn:b> .\n", Dialect::NTriples, None)?;
/// let (_c, receipt) = s.transition(&Step::EntailRdfs)?;
/// let att = sign_receipt(&key, &receipt);
/// assert!(verify_receipt(&receipt, &att, &trusted).is_ok());
/// // Any change to the receipt breaks the signature.
/// let mut forged = receipt.clone();
/// forged.added += 1;
/// assert!(verify_receipt(&forged, &att, &trusted).is_err());
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Attestation {
    /// Lowercase hex SHA-256 of the canonical payload.
    pub payload_sha256: String,
    /// Identifier of the signing key.
    pub key_id: String,
    /// Lowercase hex Ed25519 signature over the payload bytes.
    pub signature: String,
}

impl Attestation {
    /// Canonical JSON: `{"key_id":..,"payload_sha256":..,"signature":..}`.
    pub fn to_json(&self) -> String {
        format!(
            "{{\"key_id\":\"{}\",\"payload_sha256\":\"{}\",\"signature\":\"{}\"}}",
            self.key_id, self.payload_sha256, self.signature
        )
    }

    /// Strict inverse of [`Attestation::to_json`] (the exact canonical form).
    pub fn from_json(text: &str) -> Result<Self, AttestError> {
        let rest = text
            .strip_prefix("{\"key_id\":\"")
            .ok_or_else(|| malformed("not a canonical attestation"))?;
        let (key_id, rest) = rest
            .split_once("\",\"payload_sha256\":\"")
            .ok_or_else(|| malformed("missing payload_sha256"))?;
        let (payload_sha256, rest) = rest
            .split_once("\",\"signature\":\"")
            .ok_or_else(|| malformed("missing signature"))?;
        let signature = rest
            .strip_suffix("\"}")
            .ok_or_else(|| malformed("trailing content"))?;
        let a = Attestation {
            payload_sha256: payload_sha256.to_string(),
            key_id: key_id.to_string(),
            signature: signature.to_string(),
        };
        a.check_shape()?;
        Ok(a)
    }

    fn check_shape(&self) -> Result<(), AttestError> {
        let hexlen = |s: &str, n: usize| s.len() == n && s.bytes().all(|b| b.is_ascii_hexdigit());
        if hexlen(&self.payload_sha256, 64)
            && hexlen(&self.key_id, 64)
            && hexlen(&self.signature, 128)
        {
            Ok(())
        } else {
            Err(malformed(
                "fields must be lowercase-or-uppercase hex of fixed length",
            ))
        }
    }
}

/// Sign arbitrary payload bytes.
pub fn sign_bytes(key: &SigningKey, payload: &[u8]) -> Attestation {
    let sig = key.0.sign(payload);
    Attestation {
        payload_sha256: sha256_hex(payload),
        key_id: key.verifying_key().key_id(),
        signature: hex_encode(&sig.to_bytes()),
    }
}

/// Verify `att` over `payload` against `trusted`. Order: shape, payload
/// digest (`BadSignature`), key trust (`UnknownKey`), signature (`BadSignature`).
pub fn verify_bytes(
    payload: &[u8],
    att: &Attestation,
    trusted: &TrustedKeys,
) -> Result<(), AttestError> {
    att.check_shape()?;
    if !att
        .payload_sha256
        .eq_ignore_ascii_case(&sha256_hex(payload))
    {
        return Err(AttestError::BadSignature);
    }
    let key = trusted
        .get(&att.key_id.to_ascii_lowercase())
        .ok_or(AttestError::UnknownKey)?;
    let raw = hex_decode(&att.signature).ok_or_else(|| malformed("signature is not hex"))?;
    let sig = Signature::from_slice(&raw).map_err(|_| malformed("signature length"))?;
    key.0
        .verify_strict(payload, &sig)
        .map_err(|_| AttestError::BadSignature)
}

pub(crate) fn json_str(s: &str) -> String {
    let mut o = String::from("\"");
    for c in s.chars() {
        match c {
            '"' => o.push_str("\\\""),
            '\\' => o.push_str("\\\\"),
            '\n' => o.push_str("\\n"),
            '\r' => o.push_str("\\r"),
            '\t' => o.push_str("\\t"),
            c if (c as u32) < 0x20 => o.push_str(&format!("\\u{:04x}", c as u32)),
            c => o.push(c),
        }
    }
    o.push('"');
    o
}

fn opt(s: Option<&str>) -> String {
    s.map_or_else(|| "null".to_string(), json_str)
}

/// The signed content of a receipt, as borrowed strings (so the same payload can
/// be rebuilt from a stored [`Receipt`] or from triples inside a state).
#[derive(Debug, Clone, Copy)]
pub struct ReceiptFields<'a> {
    /// Parent state id.
    pub parent: &'a str,
    /// Child state id.
    pub child: &'a str,
    /// Step name.
    pub step: &'a str,
    /// Backend authority capability.
    pub authority: &'a str,
    /// Backend authority revision.
    pub revision: &'a str,
    /// Quads added by the step.
    pub added: u64,
    /// Lease id, when the step ran under a lease.
    pub lease_id: Option<&'a str>,
    /// Plan digest, for plan steps.
    pub plan_sha256: Option<&'a str>,
    /// Subject digest, when the receipt is bound to a subject.
    pub subject_sha256: Option<&'a str>,
}

impl<'a> From<&'a Receipt> for ReceiptFields<'a> {
    fn from(r: &'a Receipt) -> Self {
        ReceiptFields {
            parent: &r.parent,
            child: &r.child,
            step: r.step,
            authority: r.authority.authority,
            revision: r.authority.revision,
            added: r.added as u64,
            lease_id: r.lease_id.as_deref(),
            plan_sha256: r.plan_sha256.as_deref(),
            subject_sha256: r.subject_sha256.as_deref(),
        }
    }
}

/// Canonical receipt payload: keys sorted, no whitespace, absent options `null`.
pub fn receipt_payload(f: &ReceiptFields<'_>) -> String {
    format!(
        "{{\"added\":{},\"authority\":{},\"child\":{},\"lease_id\":{},\"parent\":{},\"plan_sha256\":{},\"revision\":{},\"step\":{},\"subject_sha256\":{}}}",
        f.added,
        json_str(f.authority),
        json_str(f.child),
        opt(f.lease_id),
        json_str(f.parent),
        opt(f.plan_sha256),
        json_str(f.revision),
        json_str(f.step),
        opt(f.subject_sha256),
    )
}

/// Sign a receipt.
pub fn sign_receipt(key: &SigningKey, receipt: &Receipt) -> Attestation {
    sign_bytes(key, receipt_payload(&receipt.into()).as_bytes())
}

/// Verify a receipt's attestation against `trusted`.
pub fn verify_receipt(
    receipt: &Receipt,
    att: &Attestation,
    trusted: &TrustedKeys,
) -> Result<(), AttestError> {
    verify_bytes(receipt_payload(&receipt.into()).as_bytes(), att, trusted)
}

/// Canonical lease payload (sorted keys, no whitespace; `scope` order preserved).
pub fn lease_payload(l: &Lease) -> String {
    let scope = l
        .scope
        .iter()
        .map(|s| json_str(s))
        .collect::<Vec<_>>()
        .join(",");
    format!(
        "{{\"ceiling\":{},\"expires_unix\":{},\"holder\":{},\"id\":{},\"issued_unix\":{},\"scope\":[{scope}]}}",
        json_str(l.ceiling.name()),
        l.expires_unix,
        json_str(&l.holder),
        json_str(&l.id),
        l.issued_unix,
    )
}

/// Issue a signed lease.
pub fn sign_lease(key: &SigningKey, lease: Lease) -> SignedLease {
    let attestation = sign_bytes(key, lease_payload(&lease).as_bytes());
    SignedLease { lease, attestation }
}

/// Verify a signed lease's signature (not its expiry: expiry needs a clock).
pub fn verify_lease(signed: &SignedLease, trusted: &TrustedKeys) -> Result<(), AttestError> {
    verify_bytes(
        lease_payload(&signed.lease).as_bytes(),
        &signed.attestation,
        trusted,
    )
}
