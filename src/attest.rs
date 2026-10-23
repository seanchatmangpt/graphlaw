//! Offline-verifiable Ed25519 attestations over receipts and leases.
//!
//! A plain [`Receipt`] is only a record: anyone able to write triples can write
//! one. An [`Attestation`] binds a canonical payload (sorted-key, no-whitespace
//! JSON) to an Ed25519 key. Signing is deterministic (RFC 8032; no RNG) and
//! verification needs only the payload, the attestation and a caller-supplied
//! [`TrustedKeys`] set: no network, no clock.
//!
//! Attestation fields (all lowercase hex): `payload_blake3` is the BLAKE3 of
//! the canonical payload, `key_id` is the BLAKE3 of the 32-byte public key,
//! `signature` is the 64-byte Ed25519 signature over the payload bytes.

use std::collections::BTreeMap;
use std::fmt;

use ed25519_dalek::{
    Signature, Signer, SigningKey as DalekSigning, VerifyingKey as DalekVerifying,
};

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
///
/// ```
/// use graphlaw::attest::{hex_decode, hex_encode};
///
/// assert_eq!(hex_encode(&[0x0f, 0xa0]), "0fa0");
/// assert_eq!(hex_decode(&hex_encode(&[0xde, 0xad])), Some(vec![0xde, 0xad]));
/// ```
pub fn hex_encode(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

/// Hex decoding (either case); `None` on odd length or a non-hex digit.
///
/// ```
/// use graphlaw::attest::hex_decode;
///
/// assert_eq!(hex_decode("0Fa0"), Some(vec![0x0f, 0xa0]));
/// assert_eq!(hex_decode("abc"), None); // odd length
/// assert_eq!(hex_decode("zz"), None); // not hex
/// ```
pub fn hex_decode(s: &str) -> Option<Vec<u8>> {
    if !s.len().is_multiple_of(2) || !s.is_ascii() {
        return None;
    }
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16).ok())
        .collect()
}

fn blake3_hex(bytes: &[u8]) -> String {
    hex_encode(blake3::hash(bytes).as_bytes())
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
    ///
    /// ```
    /// use graphlaw::attest::SigningKey;
    ///
    /// let key = SigningKey::from_seed([7; 32]);
    /// // Ed25519 signing is deterministic: same seed, same public half.
    /// assert_eq!(
    ///     key.verifying_key().to_hex(),
    ///     SigningKey::from_seed([7; 32]).verifying_key().to_hex()
    /// );
    /// // Debug output never shows the seed bytes.
    /// assert!(!format!("{key:?}").contains("070707"));
    /// ```
    pub fn from_seed(seed: [u8; 32]) -> Self {
        SigningKey(DalekSigning::from_bytes(&seed))
    }

    /// Build a signing key from a 64-character hex seed.
    ///
    /// ```
    /// use graphlaw::attest::SigningKey;
    ///
    /// let hex = "ab".repeat(32);
    /// let key = SigningKey::from_seed_hex(&hex)?;
    /// // Same key as building from the equivalent raw seed bytes.
    /// let raw: [u8; 32] = hex.as_str().as_bytes().chunks(2)
    ///     .map(|c| u8::from_str_radix(std::str::from_utf8(c).unwrap(), 16).unwrap())
    ///     .collect::<Vec<u8>>()
    ///     .try_into()
    ///     .unwrap();
    /// assert_eq!(key.verifying_key(), SigningKey::from_seed(raw).verifying_key());
    /// assert!(SigningKey::from_seed_hex("zz").is_err()); // not hex
    /// assert!(SigningKey::from_seed_hex("ab").is_err()); // too short
    /// # Ok::<(), graphlaw::attest::AttestError>(())
    /// ```
    pub fn from_seed_hex(hex: &str) -> Result<Self, AttestError> {
        let bytes = hex_decode(hex).ok_or_else(|| malformed("seed is not hex"))?;
        let seed: [u8; 32] = bytes
            .try_into()
            .map_err(|_| malformed("seed must be 32 bytes"))?;
        Ok(Self::from_seed(seed))
    }

    /// The public half of this signing key.
    ///
    /// ```
    /// use graphlaw::attest::SigningKey;
    ///
    /// let key = SigningKey::from_seed([1; 32]);
    /// assert_eq!(
    ///     key.verifying_key().to_hex(),
    ///     SigningKey::from_seed([1; 32]).verifying_key().to_hex()
    /// );
    /// ```
    pub fn verifying_key(&self) -> VerifyingKey {
        VerifyingKey(self.0.verifying_key())
    }
}

/// An Ed25519 public key.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VerifyingKey(DalekVerifying);

impl VerifyingKey {
    /// Parse a verifying key from its 32 raw bytes.
    ///
    /// ```
    /// use graphlaw::attest::{hex_decode, SigningKey, VerifyingKey};
    ///
    /// let vk = SigningKey::from_seed([3; 32]).verifying_key();
    /// let raw: [u8; 32] = hex_decode(&vk.to_hex()).unwrap().try_into().unwrap();
    /// assert_eq!(VerifyingKey::from_bytes(&raw)?.to_hex(), vk.to_hex());
    /// # Ok::<(), graphlaw::attest::AttestError>(())
    /// ```
    pub fn from_bytes(bytes: &[u8; 32]) -> Result<Self, AttestError> {
        DalekVerifying::from_bytes(bytes)
            .map(VerifyingKey)
            .map_err(|_| malformed("not a valid Ed25519 public key"))
    }

    /// Parse a verifying key from 64 hex characters.
    ///
    /// ```
    /// use graphlaw::attest::{SigningKey, VerifyingKey};
    ///
    /// let vk = SigningKey::from_seed([3; 32]).verifying_key();
    /// assert_eq!(VerifyingKey::from_hex(&vk.to_hex())?.to_hex(), vk.to_hex());
    /// assert!(VerifyingKey::from_hex("zz").is_err());
    /// # Ok::<(), graphlaw::attest::AttestError>(())
    /// ```
    pub fn from_hex(hex: &str) -> Result<Self, AttestError> {
        let bytes = hex_decode(hex).ok_or_else(|| malformed("public key is not hex"))?;
        let arr: [u8; 32] = bytes
            .try_into()
            .map_err(|_| malformed("public key must be 32 bytes"))?;
        Self::from_bytes(&arr)
    }

    /// Lower-case hex of the 32 raw key bytes.
    ///
    /// ```
    /// use graphlaw::attest::SigningKey;
    ///
    /// let hex = SigningKey::from_seed([4; 32]).verifying_key().to_hex();
    /// assert_eq!(hex.len(), 64);
    /// assert!(hex.bytes().all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase()));
    /// ```
    pub fn to_hex(&self) -> String {
        hex_encode(self.0.as_bytes())
    }

    /// BLAKE3 of the public key bytes, lowercase hex.
    ///
    /// ```
    /// use graphlaw::attest::SigningKey;
    ///
    /// let vk = SigningKey::from_seed([5; 32]).verifying_key();
    /// assert_eq!(vk.key_id().len(), 64);
    /// assert_eq!(vk.key_id(), SigningKey::from_seed([5; 32]).verifying_key().key_id());
    /// ```
    pub fn key_id(&self) -> String {
        blake3_hex(self.0.as_bytes())
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
    ///
    /// ```
    /// use graphlaw::attest::{SigningKey, TrustedKeys};
    ///
    /// let mut trusted = TrustedKeys::new();
    /// assert!(trusted.is_empty());
    /// trusted.insert(SigningKey::from_seed([6; 32]).verifying_key());
    /// assert_eq!(trusted.len(), 1);
    /// ```
    pub fn insert(&mut self, key: VerifyingKey) {
        self.0.insert(key.key_id(), key);
    }

    /// Build from hex public keys.
    ///
    /// ```
    /// use graphlaw::attest::{SigningKey, TrustedKeys};
    ///
    /// let hex = SigningKey::from_seed([6; 32]).verifying_key().to_hex();
    /// let trusted = TrustedKeys::from_hex([hex])?;
    /// assert_eq!(trusted.len(), 1);
    /// assert!(TrustedKeys::from_hex(["zz"]).is_err());
    /// # Ok::<(), graphlaw::attest::AttestError>(())
    /// ```
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
    ///
    /// ```
    /// use graphlaw::attest::{SigningKey, TrustedKeys};
    ///
    /// let key = SigningKey::from_seed([6; 32]);
    /// let mut trusted = TrustedKeys::new();
    /// trusted.insert(key.verifying_key());
    /// assert!(trusted.get(&key.verifying_key().key_id()).is_some());
    /// assert!(trusted.get("deadbeef").is_none());
    /// ```
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
    /// Lowercase hex BLAKE3 of the canonical payload.
    pub payload_blake3: String,
    /// Identifier of the signing key.
    pub key_id: String,
    /// Lowercase hex Ed25519 signature over the payload bytes.
    pub signature: String,
}

impl Attestation {
    /// Canonical JSON: `{"key_id":..,"payload_blake3":..,"signature":..}`.
    ///
    /// ```
    /// use graphlaw::attest::{sign_bytes, Attestation, SigningKey};
    ///
    /// let att = sign_bytes(&SigningKey::from_seed([9; 32]), b"payload");
    /// let text = att.to_json();
    /// assert!(text.starts_with("{\"key_id\":\""));
    /// assert_eq!(Attestation::from_json(&text)?, att);
    /// # Ok::<(), graphlaw::attest::AttestError>(())
    /// ```
    pub fn to_json(&self) -> String {
        format!(
            "{{\"key_id\":\"{}\",\"payload_blake3\":\"{}\",\"signature\":\"{}\"}}",
            self.key_id, self.payload_blake3, self.signature
        )
    }

    /// Strict inverse of [`Attestation::to_json`] (the exact canonical form).
    ///
    /// ```
    /// use graphlaw::attest::{sign_bytes, Attestation, SigningKey};
    ///
    /// let att = sign_bytes(&SigningKey::from_seed([9; 32]), b"payload");
    /// assert_eq!(Attestation::from_json(&att.to_json())?, att);
    /// // Anything that is not the exact canonical shape is refused.
    /// assert!(Attestation::from_json("{\"key_id\":\"x\"}").is_err());
    /// # Ok::<(), graphlaw::attest::AttestError>(())
    /// ```
    pub fn from_json(text: &str) -> Result<Self, AttestError> {
        let rest = text
            .strip_prefix("{\"key_id\":\"")
            .ok_or_else(|| malformed("not a canonical attestation"))?;
        let (key_id, rest) = rest
            .split_once("\",\"payload_blake3\":\"")
            .ok_or_else(|| malformed("missing payload_blake3"))?;
        let (payload_blake3, rest) = rest
            .split_once("\",\"signature\":\"")
            .ok_or_else(|| malformed("missing signature"))?;
        let signature = rest
            .strip_suffix("\"}")
            .ok_or_else(|| malformed("trailing content"))?;
        let a = Attestation {
            payload_blake3: payload_blake3.to_string(),
            key_id: key_id.to_string(),
            signature: signature.to_string(),
        };
        a.check_shape()?;
        Ok(a)
    }

    fn check_shape(&self) -> Result<(), AttestError> {
        let hexlen = |s: &str, n: usize| s.len() == n && s.bytes().all(|b| b.is_ascii_hexdigit());
        if hexlen(&self.payload_blake3, 64)
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
///
/// ```
/// use graphlaw::attest::{sign_bytes, verify_bytes, SigningKey, TrustedKeys};
///
/// let key = SigningKey::from_seed([9; 32]);
/// let mut trusted = TrustedKeys::new();
/// trusted.insert(key.verifying_key());
/// let att = sign_bytes(&key, b"law payload");
/// assert!(verify_bytes(b"law payload", &att, &trusted).is_ok());
/// // Any other payload fails verification.
/// assert!(verify_bytes(b"OTHER", &att, &trusted).is_err());
/// ```
pub fn sign_bytes(key: &SigningKey, payload: &[u8]) -> Attestation {
    let sig = key.0.sign(payload);
    Attestation {
        payload_blake3: blake3_hex(payload),
        key_id: key.verifying_key().key_id(),
        signature: hex_encode(&sig.to_bytes()),
    }
}

/// Verify `att` over `payload` against `trusted`. Order: shape, payload
/// digest (`BadSignature`), key trust (`UnknownKey`), signature (`BadSignature`).
///
/// ```
/// use graphlaw::attest::{sign_bytes, verify_bytes, SigningKey, TrustedKeys};
///
/// let key = SigningKey::from_seed([9; 32]);
/// let mut trusted = TrustedKeys::new();
/// trusted.insert(key.verifying_key());
/// let att = sign_bytes(&key, b"payload");
/// assert!(verify_bytes(b"payload", &att, &trusted).is_ok());
/// // A key outside the trust set is refused even with a valid signature.
/// assert_eq!(
///     verify_bytes(b"payload", &att, &TrustedKeys::new()),
///     Err(graphlaw::attest::AttestError::UnknownKey)
/// );
/// ```
pub fn verify_bytes(
    payload: &[u8],
    att: &Attestation,
    trusted: &TrustedKeys,
) -> Result<(), AttestError> {
    att.check_shape()?;
    if !att
        .payload_blake3
        .eq_ignore_ascii_case(&blake3_hex(payload))
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
    pub plan_blake3: Option<&'a str>,
    /// Subject digest, when the receipt is bound to a subject.
    pub subject_blake3: Option<&'a str>,
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
            plan_blake3: r.plan_blake3.as_deref(),
            subject_blake3: r.subject_blake3.as_deref(),
        }
    }
}

/// Canonical receipt payload: keys sorted, no whitespace, absent options `null`.
///
/// ```
/// use graphlaw::attest::{receipt_payload, ReceiptFields};
///
/// let f = ReceiptFields {
///     parent: "p",
///     child: "c",
///     step: "derive:rdfs",
///     authority: "duckdb",
///     revision: "v1",
///     added: 2,
///     lease_id: None,
///     plan_blake3: None,
///     subject_blake3: Some("blake3:abc"),
/// };
/// assert_eq!(
///     receipt_payload(&f),
///     "{\"added\":2,\"authority\":\"duckdb\",\"child\":\"c\",\"lease_id\":null,\
///      \"parent\":\"p\",\"plan_blake3\":null,\"revision\":\"v1\",\
///      \"step\":\"derive:rdfs\",\"subject_blake3\":\"blake3:abc\"}"
/// );
/// ```
pub fn receipt_payload(f: &ReceiptFields<'_>) -> String {
    format!(
        "{{\"added\":{},\"authority\":{},\"child\":{},\"lease_id\":{},\"parent\":{},\"plan_blake3\":{},\"revision\":{},\"step\":{},\"subject_blake3\":{}}}",
        f.added,
        json_str(f.authority),
        json_str(f.child),
        opt(f.lease_id),
        json_str(f.parent),
        opt(f.plan_blake3),
        json_str(f.revision),
        json_str(f.step),
        opt(f.subject_blake3),
    )
}

/// Sign a receipt.
///
/// Documented reason for no standalone doctest: exercising this needs a real
/// [`crate::law::Receipt`] from a [`crate::law::LawState`] transition, which the
/// struct-level example on [`Attestation`] already runs end to end — it calls
/// [`sign_receipt`] and [`verify_receipt`] together and asserts the forged-receipt
/// refusal.
pub fn sign_receipt(key: &SigningKey, receipt: &Receipt) -> Attestation {
    sign_bytes(key, receipt_payload(&receipt.into()).as_bytes())
}

/// Verify a receipt's attestation against `trusted`. See [`Attestation`]'s
/// struct-level example, which exercises this against [`sign_receipt`],
/// including the tampered-receipt refusal.
pub fn verify_receipt(
    receipt: &Receipt,
    att: &Attestation,
    trusted: &TrustedKeys,
) -> Result<(), AttestError> {
    verify_bytes(receipt_payload(&receipt.into()).as_bytes(), att, trusted)
}

/// Canonical lease payload (sorted keys, no whitespace; `scope` order preserved).
///
/// ```
/// use graphlaw::attest::lease_payload;
/// use graphlaw::law::{Ceiling, Lease};
///
/// let lease = Lease {
///     id: "l1".into(),
///     holder: "agent".into(),
///     ceiling: Ceiling::Construct,
///     scope: vec!["derive:rdfs".into()],
///     expires_unix: 2_000,
///     issued_unix: 1_000,
/// };
/// assert_eq!(
///     lease_payload(&lease),
///     "{\"ceiling\":\"construct\",\"expires_unix\":2000,\"holder\":\"agent\",\
///      \"id\":\"l1\",\"issued_unix\":1000,\"scope\":[\"derive:rdfs\"]}"
/// );
/// ```
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
///
/// ```
/// use graphlaw::attest::{sign_lease, verify_lease, SigningKey, TrustedKeys};
/// use graphlaw::law::{Ceiling, Lease};
///
/// let key = SigningKey::from_seed([8; 32]);
/// let mut trusted = TrustedKeys::new();
/// trusted.insert(key.verifying_key());
/// let signed = sign_lease(&key, Lease {
///     id: "l".into(),
///     holder: "h".into(),
///     ceiling: Ceiling::Construct,
///     scope: vec!["derive:rdfs".into()],
///     expires_unix: 2_000,
///     issued_unix: 1_000,
/// });
/// assert!(verify_lease(&signed, &trusted).is_ok());
/// // An untrusted issuer key is refused.
/// assert!(verify_lease(&signed, &TrustedKeys::new()).is_err());
/// ```
pub fn verify_lease(signed: &SignedLease, trusted: &TrustedKeys) -> Result<(), AttestError> {
    verify_bytes(
        lease_payload(&signed.lease).as_bytes(),
        &signed.attestation,
        trusted,
    )
}
