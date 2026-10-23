use serde::{Deserialize, Serialize};

use super::epoch;
use super::frame::{DenialPolarity, OcelCausalFrame, OcelCausalReceipt, PackedObjRef};

// ---------------------------------------------------------------------------
// Errors (subset of graphlaw-core::error::CoreError used by the family)
// ---------------------------------------------------------------------------

/// Error type for the receipt-chain family (the `CoreError` variants the ported
/// family produces; names match graphlaw-core's `name()` strings exactly).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CoreError {
    /// Payload/epoch serialization failed.
    SerializationFailed(String),
    /// Hex decode failed (wrong length or non-hex characters).
    HexDecodeFailed(String),
    /// An unrecognized or shape-contradicting chain-rule discriminator.
    ReceiptChainRuleInvalid(String),
    /// Filesystem I/O failed.
    Io(String),
    /// A schema string contradicts the presence/absence of a `v2` payload.
    ReceiptSchemaPayloadMismatch(String),
    /// An unrecognized receipt schema identity.
    UnrecognizedReceiptSchema(String),
    /// An explicit ceiling exceeds what the component evidence supports.
    CeilingExceedsMeet {
        /// The requested ceiling.
        requested: crate::receipt_chain::epoch::CeilingLevel,
        /// The ceiling the evidence supports.
        allowed: crate::receipt_chain::epoch::CeilingLevel,
    },
    /// A promotion witness failed validation.
    PromotionRefused {
        /// Why the promotion was refused.
        reason: String,
    },
}

impl CoreError {
    /// Stable machine label, matching graphlaw-core's `CoreError::name()`.
    #[must_use]
    pub fn name(&self) -> &'static str {
        match self {
            CoreError::SerializationFailed(_) => "SerializationFailed",
            CoreError::Io(_) => "Io",
            CoreError::HexDecodeFailed(_) => "HexDecodeFailed",
            CoreError::ReceiptChainRuleInvalid(_) => "ReceiptChainRuleInvalid",
            CoreError::ReceiptSchemaPayloadMismatch(_) => "ReceiptSchemaPayloadMismatch",
            CoreError::UnrecognizedReceiptSchema(_) => "UnrecognizedReceiptSchema",
            CoreError::CeilingExceedsMeet { .. } => "CeilingExceedsMeet",
            CoreError::PromotionRefused { .. } => "PromotionRefused",
        }
    }
}

impl std::fmt::Display for CoreError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CoreError::SerializationFailed(s) => write!(f, "payload serialization failed: {s}"),
            CoreError::Io(s) => write!(f, "io error: {s}"),
            CoreError::HexDecodeFailed(s) => write!(f, "hex decode failed: {s}"),
            CoreError::ReceiptChainRuleInvalid(s) => {
                write!(f, "receipt chain rule invalid: {s}")
            }
            CoreError::ReceiptSchemaPayloadMismatch(s) => {
                write!(f, "receipt schema/payload mismatch: {s}")
            }
            CoreError::UnrecognizedReceiptSchema(s) => {
                write!(f, "unrecognized receipt schema: {s}")
            }
            CoreError::CeilingExceedsMeet { requested, allowed } => {
                write!(f, "ceiling {requested:?} exceeds allowed meet {allowed:?}")
            }
            CoreError::PromotionRefused { reason } => write!(f, "promotion refused: {reason}"),
        }
    }
}

impl std::error::Error for CoreError {}

// ---------------------------------------------------------------------------
// Obligation + Andon (port of graphlaw-core::law)
// ---------------------------------------------------------------------------

/// Precondition or blocking constraint a LawObject must satisfy before admission.
/// Hashable and dispatchable: obligations are first-class values, not closures.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Obligation {
    /// A predicate that must be satisfied.
    Precondition {
        /// Identifier for the predicate being checked.
        predicate_id: String,
        /// Hash of the parameters passed to the predicate.
        params_hash: [u8; 32],
    },
    /// A hard constraint that blocks progress until lifted.
    BlockingConstraint {
        /// Human-readable reason for the block.
        reason: String,
    },
    /// External evidence must be provided.
    EvidenceRequired {
        /// Type or category of evidence needed.
        evidence_type: String,
    },
}

/// Halt/override signal: unmet obligations halt progress.
/// Halted state persists until explicitly cleared by a receipt or logged override.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Andon {
    /// All obligations satisfied; proceed.
    Green,
    /// Obligations unmet; halted with timestamp.
    Halted {
        /// Obligations blocking progress.
        unmet: Vec<Obligation>,
        /// Refusal-taxonomy classification of *why* progress halted.
        ///
        /// `#[serde(default)]` so JSON produced before this field existed
        /// still deserializes (empty `refusals`).
        #[serde(default)]
        refusals: Vec<RefusalScenario>,
        /// Timestamp (typically milliseconds since epoch) when halt occurred.
        at: u64,
    },
    /// Obligations were overridden; halt lifted with reason and timestamp.
    Overridden {
        /// Who or what authorized the override.
        by: String,
        /// Reason for override.
        reason: String,
        /// Timestamp when override occurred.
        at: u64,
    },
}

// ---------------------------------------------------------------------------
// Refusal taxonomy (port of graphlaw-core::refusal)
// ---------------------------------------------------------------------------

/// 8-bucket refusal classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RefusalCategory {
    /// Caller/subject identity or ID-format problem.
    Identity,
    /// Resource capacity or quota limit reached.
    Capacity,
    /// Graph/topology (structural) violation.
    Topology,
    /// Temporal or deadline constraint violated.
    Temporal,
    /// Lifecycle / state-machine position violation.
    Lifecycle,
    /// Authorization or credential failure.
    Authorization,
    /// Event-level prerequisite (evidence, precondition) missing.
    Prerequisites,
    /// Reserved for future enforcement.
    Reserved,
}

/// GraphLaw-native refusal scenarios: obligation-driven halts and the 7
/// non-`ADMITTED` `DenialPolarity` lanes.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RefusalScenario {
    /// An [`Obligation::BlockingConstraint`] was in force.
    BlockingConstraint {
        /// The obligation's stated reason.
        reason: String,
    },
    /// An [`Obligation::EvidenceRequired`] was not satisfied.
    MissingEvidence {
        /// The evidence type that was required but absent.
        evidence_type: String,
    },
    /// An [`Obligation::Precondition`] was not satisfied.
    UnsatisfiedPrecondition {
        /// The predicate id that was required but not satisfied.
        predicate_id: String,
    },

    // ── DenialPolarity lanes (bcinr_powl_receipt, non-ADMITTED) ─────────
    /// `DenialPolarity::WATCHDOG_DRAINED`.
    WatchdogDrained,
    /// `DenialPolarity::PRECONDITION_FAILED`.
    PreconditionFailed,
    /// `DenialPolarity::SLA_BREACH`.
    SlaBreach,
    /// `DenialPolarity::AUTHORIZATION_DENIED`.
    AuthorizationDenied,
    /// `DenialPolarity::RESOURCE_EXHAUSTED`.
    ResourceExhausted,
    /// `DenialPolarity::OBJECT_LIFECYCLE_VIOLATION`.
    ObjectLifecycleViolation,
    /// `DenialPolarity::CONFORMANCE_GATE_FAILED`.
    ConformanceGateFailed,
}

impl RefusalScenario {
    /// The 8-bucket category this scenario belongs to. Exhaustive.
    #[must_use]
    pub fn category(&self) -> RefusalCategory {
        match self {
            Self::BlockingConstraint { .. } => RefusalCategory::Lifecycle,
            Self::MissingEvidence { .. } => RefusalCategory::Prerequisites,
            Self::UnsatisfiedPrecondition { .. } => RefusalCategory::Prerequisites,
            Self::WatchdogDrained => RefusalCategory::Temporal,
            Self::PreconditionFailed => RefusalCategory::Prerequisites,
            Self::SlaBreach => RefusalCategory::Temporal,
            Self::AuthorizationDenied => RefusalCategory::Authorization,
            Self::ResourceExhausted => RefusalCategory::Capacity,
            Self::ObjectLifecycleViolation => RefusalCategory::Lifecycle,
            Self::ConformanceGateFailed => RefusalCategory::Topology,
        }
    }
}

/// Convert an unmet [`Obligation`] into the matching [`RefusalScenario`].
impl From<&Obligation> for RefusalScenario {
    fn from(obligation: &Obligation) -> Self {
        match obligation {
            Obligation::BlockingConstraint { reason } => RefusalScenario::BlockingConstraint {
                reason: reason.clone(),
            },
            Obligation::EvidenceRequired { evidence_type } => RefusalScenario::MissingEvidence {
                evidence_type: evidence_type.clone(),
            },
            Obligation::Precondition { predicate_id, .. } => {
                RefusalScenario::UnsatisfiedPrecondition {
                    predicate_id: predicate_id.clone(),
                }
            }
        }
    }
}

/// Map a single `DenialPolarity` lane constant to the [`RefusalScenario`] that
/// fires it, or `None` for `ADMITTED` (or any non-single-lane word).
#[must_use]
pub fn scenario_for_denial_lane(lane: DenialPolarity) -> Option<RefusalScenario> {
    if lane == DenialPolarity::ADMITTED {
        None
    } else if lane == DenialPolarity::WATCHDOG_DRAINED {
        Some(RefusalScenario::WatchdogDrained)
    } else if lane == DenialPolarity::PRECONDITION_FAILED {
        Some(RefusalScenario::PreconditionFailed)
    } else if lane == DenialPolarity::SLA_BREACH {
        Some(RefusalScenario::SlaBreach)
    } else if lane == DenialPolarity::AUTHORIZATION_DENIED {
        Some(RefusalScenario::AuthorizationDenied)
    } else if lane == DenialPolarity::RESOURCE_EXHAUSTED {
        Some(RefusalScenario::ResourceExhausted)
    } else if lane == DenialPolarity::OBJECT_LIFECYCLE_VIOLATION {
        Some(RefusalScenario::ObjectLifecycleViolation)
    } else if lane == DenialPolarity::CONFORMANCE_GATE_FAILED {
        Some(RefusalScenario::ConformanceGateFailed)
    } else {
        None
    }
}

/// Inverse of [`scenario_for_denial_lane`]: which `DenialPolarity` lane a
/// scenario composes into a receipt's denial word.
#[must_use]
pub fn denial_lane(scenario: &RefusalScenario) -> DenialPolarity {
    match scenario {
        RefusalScenario::BlockingConstraint { .. } => DenialPolarity::OBJECT_LIFECYCLE_VIOLATION,
        RefusalScenario::MissingEvidence { .. } => DenialPolarity::PRECONDITION_FAILED,
        RefusalScenario::UnsatisfiedPrecondition { .. } => DenialPolarity::PRECONDITION_FAILED,
        RefusalScenario::WatchdogDrained => DenialPolarity::WATCHDOG_DRAINED,
        RefusalScenario::PreconditionFailed => DenialPolarity::PRECONDITION_FAILED,
        RefusalScenario::SlaBreach => DenialPolarity::SLA_BREACH,
        RefusalScenario::AuthorizationDenied => DenialPolarity::AUTHORIZATION_DENIED,
        RefusalScenario::ResourceExhausted => DenialPolarity::RESOURCE_EXHAUSTED,
        RefusalScenario::ObjectLifecycleViolation => DenialPolarity::OBJECT_LIFECYCLE_VIOLATION,
        RefusalScenario::ConformanceGateFailed => DenialPolarity::CONFORMANCE_GATE_FAILED,
    }
}

/// Fold a set of refusal scenarios into a single composed `DenialPolarity`
/// word (OR of each scenario's [`denial_lane`]). Empty input composes to
/// `ADMITTED` (the identity element of `compose`).
#[must_use]
pub fn compose_denials<'a>(
    scenarios: impl IntoIterator<Item = &'a RefusalScenario>,
) -> DenialPolarity {
    scenarios
        .into_iter()
        .map(denial_lane)
        .fold(DenialPolarity::ADMITTED, DenialPolarity::compose)
}

// ---------------------------------------------------------------------------
// ReceiptMeta + frame construction (port of graphlaw-core::law)
// ---------------------------------------------------------------------------

/// Parameters that bind a receipt to its position in an OCEL run.
#[derive(Debug, Clone)]
pub struct ReceiptMeta {
    /// Monotonically increasing step identity within a run.
    pub instruction_id: u64,
    /// Index into the activity table for this step's activity.
    pub activity_idx: u16,
    /// Classifier byte for the POWL node kind (XOR, SEQ, LOOP, etc.).
    pub node_kind: u8,
    /// Wall-clock timestamp in nanoseconds. `None` uses `SystemTime::now()`.
    pub ts_ns: Option<u64>,
    /// The honest denial polarity for this receipt; written straight into the
    /// frame's `denial`/`fired_mask`. Defaults to `DenialPolarity::ADMITTED`.
    pub denial: DenialPolarity,
    /// Halt/override status.
    pub andon: Andon,
    /// Associated object IDs.
    pub object_ids: Vec<String>,
    /// Number of obligations.
    pub obligation_count: u32,
}

impl Default for ReceiptMeta {
    /// `DenialPolarity` has no `Default` impl of its own (it's an external
    /// bitfield newtype with named constants, not a derived enum), so this
    /// is written by hand rather than derived.
    fn default() -> Self {
        Self {
            instruction_id: 0,
            activity_idx: 0,
            node_kind: 0,
            ts_ns: None,
            denial: DenialPolarity::ADMITTED,
            andon: Andon::Green,
            object_ids: Vec::new(),
            obligation_count: 0,
        }
    }
}

/// Build the [`OcelCausalFrame`] for one admission receipt.
///
/// This is the **single construction site** for the frame, shared by the live
/// emission path and the persisted-replay path
/// ([`ReceiptRecord::recompute_chain_hash`]), so the two can never drift from
/// each other.
///
/// Packs the 32-byte `payload_hash` into the frame's 8 `obj_refs` slots as 8
/// little-endian u32 words, using the `PackedObjRef` tuple constructor
/// directly (not `PackedObjRef::new`, which packs a type index into the high
/// 8 bits and truncates the id to 24 bits) so the full 256-bit hash survives
/// intact as a payload commitment.
///
/// ```
/// use graphlaw::receipt_chain::{build_admission_frame, ReceiptMeta,
///     chain_from_frame, frame::DenialPolarity};
///
/// let meta = ReceiptMeta {
///     instruction_id: 0,
///     activity_idx: 0,
///     node_kind: 0,
///     ts_ns: Some(42),
///     denial: DenialPolarity::ADMITTED,
///     ..ReceiptMeta::default()
/// };
/// let f = build_admission_frame(&[0xABu8; 32], &[0u8; 32], &meta, 42);
/// assert_eq!(f.ts_ns, 42);
/// assert_eq!(f.fired_mask, 0);
///
/// // Frame hash binds the payload commitment; real value from this path:
/// let h = chain_from_frame(&[0u8; 32], &f);
/// assert_eq!(
///     h.iter().map(|x| format!("{:02x}", x)).collect::<String>(),
///     "387676fe1a45fca52eeb3cbe5d9f0ec48c79a2eab43c0a9e0b89b50428024567"
/// );
/// ```
pub fn build_admission_frame(
    payload_hash: &[u8; 32],
    prev_chain_hash: &[u8; 32],
    meta: &ReceiptMeta,
    ts_ns: u64,
) -> OcelCausalFrame {
    let meta_json =
        serde_json::to_vec(&(&meta.andon, &meta.object_ids, &meta.obligation_count)).unwrap();
    let mut combined = Vec::with_capacity(32 + meta_json.len());
    combined.extend_from_slice(payload_hash);
    combined.extend_from_slice(&meta_json);
    let mixed_hash = *blake3::hash(&combined).as_bytes();

    let mut obj_refs = [PackedObjRef::default(); 8];
    for (i, word) in mixed_hash.chunks_exact(4).enumerate() {
        let w = u32::from_le_bytes([word[0], word[1], word[2], word[3]]);
        obj_refs[i] = PackedObjRef(w);
    }

    OcelCausalFrame {
        instruction_id: meta.instruction_id,
        fired_mask: meta.denial.to_fired_mask(),
        denial: meta.denial,
        obj_refs,
        ts_ns,
        activity_idx: meta.activity_idx,
        node_kind: meta.node_kind,
        pad: [0u8; 5],
        prior_hash: *prev_chain_hash,
    }
}

/// Chain `frame` onto `prev_chain_hash` using bcinr-powl-receipt's causal
/// chain rule, returning the resulting chain hash.
///
/// NOTE: `prev_chain_hash` is intentionally mixed in twice here: once as
/// `frame.prior_hash` (hashed as part of the 99-byte frame body via
/// `to_hash_bytes()`), and again as the receipt's own seeded `chain_hash`
/// before `chain()` prepends it a second time (`chain_hash(t+1) =
/// BLAKE3(chain_hash(t) || frame_bytes(t+1))`). This double-mixing predates
/// the refactor and is kept unchanged for chain compatibility with any
/// receipts already computed by this code path.
///
/// ```
/// use graphlaw::receipt_chain::frame::{DenialPolarity, OcelCausalFrame, PackedObjRef};
/// use graphlaw::receipt_chain::chain_from_frame;
///
/// let frame = |prior: [u8; 32]| OcelCausalFrame {
///     instruction_id: 1,
///     fired_mask: 0,
///     denial: DenialPolarity::ADMITTED,
///     obj_refs: [PackedObjRef(1); 8],
///     ts_ns: 0,
///     activity_idx: 3,
///     node_kind: 9,
///     pad: [0; 5],
///     prior_hash: prior,
/// };
///
/// // Link 1 off the all-zero genesis: real value from this code path.
/// let h1 = chain_from_frame(&[0u8; 32], &frame([0u8; 32]));
/// assert_eq!(
///     h1.iter().map(|x| format!("{:02x}", x)).collect::<String>(),
///     "15abdafad661fb353049079245043fcaa0d42161609a5b5c3ef21788cc3de01e"
/// );
///
/// // Determinism: same inputs, same hash.
/// assert_eq!(chain_from_frame(&[0u8; 32], &frame([0u8; 32])), h1);
/// ```
pub fn chain_from_frame(prev_chain_hash: &[u8; 32], frame: &OcelCausalFrame) -> [u8; 32] {
    let mut receipt = OcelCausalReceipt::genesis([0u8; 32]);
    receipt.chain_hash = *prev_chain_hash;
    receipt.chain(frame);
    receipt.chain_hash
}

// ---------------------------------------------------------------------------
// ReceiptRecord (port of graphlaw-core::receipt_record)
// ---------------------------------------------------------------------------

/// Current schema version for [`ReceiptRecord`].
pub const RECEIPT_RECORD_VERSION: u32 = 1;

/// Chain-rule discriminator value for the v2-fold rule (the F1 fix): the
/// base admission-frame chain hash, then `schema` and the full `v2` payload
/// folded in by `fold_in_v2_epoch` (a strict no-op when `v2` is `None`).
pub const CHAIN_RULE_V2_FOLD: &str = "graphlaw-chain/v2-fold";

/// Chain-rule discriminator value for the pre-F1 base rule: the admission
/// frame chain hash alone, with no `v2` fold.
pub const CHAIN_RULE_BASE: &str = "graphlaw-chain/base";

/// A persisted snapshot of one receipt: enough to append to a JSONL
/// ledger, re-verify its chain hash later without the original law object,
/// and replay its lifecycle through the POWL token model.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReceiptRecord {
    /// Schema version; see [`RECEIPT_RECORD_VERSION`].
    pub version: u32,
    /// Monotonically increasing step identity within a run.
    pub instruction_id: u64,
    /// Index into the activity table for this step's activity.
    pub activity_idx: u16,
    /// Resolved human-readable label for `activity_idx`, if available. Not
    /// part of the chain-hash computation — purely descriptive.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub activity: Option<String>,
    /// Classifier byte for the POWL node kind (XOR, SEQ, LOOP, etc.).
    pub node_kind: u8,
    /// Wall-clock timestamp in nanoseconds.
    pub ts_ns: u64,
    /// Optional wall-clock duration of the admission this receipt seals, in
    /// milliseconds. Descriptive only — not part of the chain-hash
    /// computation.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub duration_ms: Option<u64>,
    /// Provenance tag distinguishing how this receipt's write was authorized
    /// (`None` for the ordinary reviewed path, `Some("unattended-dispatch")`
    /// for an unattended-fired write). Descriptive only, deliberately
    /// excluded from chain-hash computation.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub origin: Option<String>,
    /// BLAKE3 hash of the canonical JSON payload bytes, as 64 lowercase hex characters.
    pub payload_hash_hex: String,
    /// The chain hash this record was chained onto, as 64 lowercase hex characters.
    pub prev_chain_hash_hex: String,
    /// The resulting chain hash after this record, as 64 lowercase hex characters.
    pub chain_hash_hex: String,
    /// The Andon outcome at receipt time (`Green`/`Halted`/`Overridden`).
    pub andon: Andon,
    /// Number of obligations attached to the law object at receipt time.
    pub obligation_count: u32,
    /// OCEL object identifiers this receipt governs (E2O links).
    #[serde(default)]
    pub object_ids: Vec<String>,
    /// Hex-encoded ed25519 signature over [`Self::chain_hash_hex`], present only when the record has been signed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub signature_hex: Option<String>,
    /// Schema identity: [`epoch::SCHEMA_V1`] (the default) or
    /// [`epoch::SCHEMA_V2`]. Folded into the chain hash together with `v2`
    /// when `v2` is `Some`.
    #[serde(default = "epoch::default_schema")]
    pub schema: String,
    /// The v2 epoch payload (see [`epoch::ReceiptEpochV2`]). `None` on every
    /// v1 record; folded into the chain hash by
    /// [`Self::recompute_chain_hash`] whenever present.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub v2: Option<epoch::ReceiptEpochV2>,
    /// Chain-rule discriminator: which rule produced [`Self::chain_hash_hex`]
    /// ([`CHAIN_RULE_V2_FOLD`] or [`CHAIN_RULE_BASE`]; see [`ChainRule`]).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub chain_rule: Option<String>,
}

/// The chain rules a [`ReceiptRecord`] can be sealed under.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChainRule {
    /// Pre-F1 rule: admission-frame chain hash only; `v2` not covered.
    Base,
    /// Post-F1 rule: admission-frame chain hash with `schema` + `v2` folded in.
    V2Fold,
}

impl ChainRule {
    /// The wire discriminator for this rule.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            ChainRule::Base => CHAIN_RULE_BASE,
            ChainRule::V2Fold => CHAIN_RULE_V2_FOLD,
        }
    }

    /// Parse a wire discriminator; an unknown value is refused, never
    /// defaulted.
    ///
    /// # Errors
    /// [`CoreError::ReceiptChainRuleInvalid`] for any unrecognized string.
    pub fn parse(s: &str) -> Result<Self, CoreError> {
        match s {
            CHAIN_RULE_V2_FOLD => Ok(ChainRule::V2Fold),
            CHAIN_RULE_BASE => Ok(ChainRule::Base),
            other => Err(CoreError::ReceiptChainRuleInvalid(format!(
                "unrecognized chain rule `{other}`"
            ))),
        }
    }
}

/// What a successful chain verification proves about a record.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChainStanding {
    /// Every hash-relevant field, including `schema` + `v2` when present, is
    /// bound into the chain hash.
    FullyBound,
    /// A pre-F1 record (no chain-rule declaration, `v2` present) whose chain
    /// hash recomputes only under [`ChainRule::Base`]. Its `v2` payload was
    /// never covered by the chain hash and cannot be proven untampered.
    LegacyV2Unbound,
}

impl ChainStanding {
    /// Stable machine label for reports.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            ChainStanding::FullyBound => "fully-bound",
            ChainStanding::LegacyV2Unbound => "legacy-v2-unbound",
        }
    }
}

/// Outcome of [`ReceiptRecord::verify_chain`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChainVerification {
    /// The stored chain hash recomputes under a lawful rule for this record.
    Verified(ChainStanding),
    /// The stored chain hash matches no lawful rule for this record.
    Mismatch {
        /// The rule the mismatch is reported against.
        rule: ChainRule,
        /// The chain hash that rule produces from the record's fields.
        recomputed: [u8; 32],
    },
}

/// Decode a 64-lowercase-hex-character string into 32 raw bytes.
fn decode_hex32(field: &str, s: &str) -> Result<[u8; 32], CoreError> {
    let bytes = hex::decode(s).map_err(|e| CoreError::HexDecodeFailed(format!("{field}: {e}")))?;
    bytes.try_into().map_err(|v: Vec<u8>| {
        CoreError::HexDecodeFailed(format!("{field}: expected 32 bytes, got {}", v.len()))
    })
}

impl ReceiptRecord {
    /// Decode [`Self::payload_hash_hex`] into raw bytes.
    ///
    /// # Errors
    /// Malformed hex.
    pub fn payload_hash(&self) -> Result<[u8; 32], CoreError> {
        decode_hex32("payload_hash_hex", &self.payload_hash_hex)
    }

    /// Decode [`Self::prev_chain_hash_hex`] into raw bytes.
    ///
    /// # Errors
    /// Malformed hex.
    pub fn prev_chain_hash(&self) -> Result<[u8; 32], CoreError> {
        decode_hex32("prev_chain_hash_hex", &self.prev_chain_hash_hex)
    }

    /// Decode [`Self::chain_hash_hex`] into raw bytes.
    ///
    /// # Errors
    /// Malformed hex.
    pub fn chain_hash(&self) -> Result<[u8; 32], CoreError> {
        decode_hex32("chain_hash_hex", &self.chain_hash_hex)
    }

    /// Rebuild the [`ReceiptMeta`] this record was chained with (denial
    /// always resolves to `ADMITTED`: a receipt only ever exists for an
    /// object that reached the admitted stage).
    fn receipt_meta(&self) -> ReceiptMeta {
        ReceiptMeta {
            instruction_id: self.instruction_id,
            activity_idx: self.activity_idx,
            node_kind: self.node_kind,
            ts_ns: Some(self.ts_ns),
            andon: self.andon.clone(),
            object_ids: self.object_ids.clone(),
            obligation_count: self.obligation_count,
            ..Default::default()
        }
    }

    /// Recompute `chain_hash` from this record's own fields, using the exact
    /// same [`build_admission_frame`]/[`chain_from_frame`] construction the
    /// emission path uses, then folds [`Self::v2`] in via the private
    /// `fold_in_v2_epoch` step.
    ///
    /// Uses the record's declared [`Self::chain_rule`], or
    /// [`ChainRule::V2Fold`] when undeclared.
    ///
    /// # Errors
    /// Malformed hex fields, an unrecognized or shape-contradicting
    /// [`Self::chain_rule`], or a `v2` payload that fails to serialize.
    pub fn recompute_chain_hash(&self) -> Result<[u8; 32], CoreError> {
        let rule = self.declared_chain_rule()?.unwrap_or(ChainRule::V2Fold);
        self.recompute_chain_hash_under(rule)
    }

    /// The declared chain rule, if any, validated against this record's
    /// shape.
    ///
    /// # Errors
    /// [`CoreError::ReceiptChainRuleInvalid`] for an unrecognized
    /// discriminator, or for [`ChainRule::Base`] declared on a record that
    /// carries a `v2` payload.
    pub fn declared_chain_rule(&self) -> Result<Option<ChainRule>, CoreError> {
        let Some(raw) = self.chain_rule.as_deref() else {
            return Ok(None);
        };
        let rule = ChainRule::parse(raw)?;
        if rule == ChainRule::Base && self.v2.is_some() {
            return Err(CoreError::ReceiptChainRuleInvalid(format!(
                "`{CHAIN_RULE_BASE}` declared on a record carrying a v2 payload: the base \
                 rule would leave the payload outside the chain hash"
            )));
        }
        Ok(Some(rule))
    }

    /// Recompute the chain hash under an explicit `rule`, ignoring
    /// [`Self::chain_rule`].
    ///
    /// # Errors
    /// Malformed hex fields, or a `v2` payload that fails to serialize.
    pub fn recompute_chain_hash_under(&self, rule: ChainRule) -> Result<[u8; 32], CoreError> {
        let payload_hash = self.payload_hash()?;
        let prev_chain_hash = self.prev_chain_hash()?;
        let meta = self.receipt_meta();
        let frame = build_admission_frame(&payload_hash, &prev_chain_hash, &meta, self.ts_ns);
        let base = chain_from_frame(&prev_chain_hash, &frame);
        match rule {
            ChainRule::Base => Ok(base),
            ChainRule::V2Fold => fold_in_v2_epoch(base, &self.schema, self.v2.as_ref()),
        }
    }

    /// Rule-aware chain verification of this record's stored
    /// [`Self::chain_hash_hex`].
    ///
    /// - Declared rule: verified under exactly that rule, no fallback.
    /// - Undeclared: [`ChainRule::V2Fold`] first ([`ChainStanding::FullyBound`]
    ///   on match). Only if that fails and a `v2` payload is present is
    ///   [`ChainRule::Base`] tried; a match there is the capped
    ///   [`ChainStanding::LegacyV2Unbound`], never `FullyBound`.
    ///
    /// # Errors
    /// Malformed hex fields, an invalid [`Self::chain_rule`], or a `v2`
    /// payload that fails to serialize.
    pub fn verify_chain(&self) -> Result<ChainVerification, CoreError> {
        self.verify_chain_against(self.chain_hash()?)
    }

    /// [`Self::verify_chain`] against an already-decoded stored chain hash.
    ///
    /// # Errors
    /// Malformed payload/prev hex fields, an invalid [`Self::chain_rule`],
    /// or a `v2` payload that fails to serialize.
    pub fn verify_chain_against(&self, stored: [u8; 32]) -> Result<ChainVerification, CoreError> {
        if let Some(rule) = self.declared_chain_rule()? {
            let recomputed = self.recompute_chain_hash_under(rule)?;
            return Ok(if recomputed == stored {
                ChainVerification::Verified(ChainStanding::FullyBound)
            } else {
                ChainVerification::Mismatch { rule, recomputed }
            });
        }
        let base = self.recompute_chain_hash_under(ChainRule::Base)?;
        let fold = fold_in_v2_epoch(base, &self.schema, self.v2.as_ref())?;
        if fold == stored {
            return Ok(ChainVerification::Verified(ChainStanding::FullyBound));
        }
        if self.v2.is_some() && base == stored {
            return Ok(ChainVerification::Verified(ChainStanding::LegacyV2Unbound));
        }
        Ok(ChainVerification::Mismatch {
            rule: ChainRule::V2Fold,
            recomputed: fold,
        })
    }

    /// Recompute the chain hash under the rule [`Self::verify_chain`] would
    /// govern this record by. This is the recompute every *verifier* of
    /// stored records must use; emission paths use the strict
    /// [`Self::recompute_chain_hash`].
    ///
    /// # Errors
    /// Same as [`Self::verify_chain`].
    pub fn recompute_chain_hash_lawful(&self) -> Result<[u8; 32], CoreError> {
        match self.verify_chain()? {
            ChainVerification::Verified(_) => self.chain_hash(),
            ChainVerification::Mismatch { recomputed, .. } => Ok(recomputed),
        }
    }
}

/// Chain-rule monotonicity over an ordered ledger (downgrade guard).
///
/// A record that verifies only as [`ChainStanding::LegacyV2Unbound`] is
/// lawful only in the pre-F1 prefix of a chain. Feed records in ledger order
/// via [`Self::observe`].
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ChainRuleMonotonicity {
    first_declared: Option<usize>,
}

impl ChainRuleMonotonicity {
    /// A fresh tracker (no record observed yet).
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Index of the first record that declared a chain rule, if any.
    #[must_use]
    pub fn first_declared(&self) -> Option<usize> {
        self.first_declared
    }

    /// Observe record `idx` (ledger order) with its verified `standing`.
    ///
    /// # Errors
    /// [`CoreError::ReceiptChainRuleInvalid`] when a
    /// [`ChainStanding::LegacyV2Unbound`] record follows a record that
    /// declared a chain rule.
    pub fn observe(
        &mut self,
        idx: usize,
        record: &ReceiptRecord,
        standing: ChainStanding,
    ) -> Result<(), CoreError> {
        self.observe_declared(idx, record.chain_rule.is_some(), standing)
    }

    /// [`Self::observe`] for callers that hold only whether record `idx`
    /// declared a chain rule.
    ///
    /// # Errors
    /// Same as [`Self::observe`].
    pub fn observe_declared(
        &mut self,
        idx: usize,
        declared: bool,
        standing: ChainStanding,
    ) -> Result<(), CoreError> {
        if standing == ChainStanding::LegacyV2Unbound
            && let Some(first) = self.first_declared
        {
            return Err(CoreError::ReceiptChainRuleInvalid(format!(
                "chain-rule downgrade: record {idx} verifies only as `{}` after record \
                 {first} declared a chain rule",
                ChainStanding::LegacyV2Unbound.as_str()
            )));
        }
        if declared && self.first_declared.is_none() {
            self.first_declared = Some(idx);
        }
        Ok(())
    }
}

/// Fold a record's v2 epoch payload (if present) into `base`, so a tampered
/// `standing_ceiling`/`admission`/`equivalence`/`promotion_eligible` changes
/// the resulting [`ReceiptRecord::chain_hash_hex`] instead of leaving it.
///
/// `v2: None` returns `base` completely unchanged — a strict no-op.
fn fold_in_v2_epoch(
    base: [u8; 32],
    schema: &str,
    v2: Option<&epoch::ReceiptEpochV2>,
) -> Result<[u8; 32], CoreError> {
    let Some(epoch) = v2 else {
        return Ok(base);
    };
    let epoch_bytes = serde_json::to_vec(epoch)
        .map_err(|e| CoreError::SerializationFailed(format!("v2 epoch: {e}")))?;
    let mut combined = Vec::with_capacity(32 + schema.len() + epoch_bytes.len());
    combined.extend_from_slice(&base);
    combined.extend_from_slice(schema.as_bytes());
    combined.extend_from_slice(&epoch_bytes);
    Ok(*blake3::hash(&combined).as_bytes())
}
