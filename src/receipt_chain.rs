//! Receipt chain: verbatim port of praxis-core's receipt-chain family so
//! `praxis-core` can retire (see ggen `docs/v26_10_10_praxis_retirement_plan.md`).
//!
//! Sources ported verbatim (wire shapes are frozen; hashes are byte-exact):
//! - `praxis-core::law` — [`Andon`], [`Obligation`], [`ReceiptMeta`],
//!   [`build_admission_frame`], [`chain_from_frame`].
//! - `praxis-core::refusal` — [`RefusalScenario`], [`RefusalCategory`].
//! - `praxis-core::receipt_record` — [`ReceiptRecord`], [`ChainRule`],
//!   [`ChainStanding`], [`ChainVerification`], [`ChainRuleMonotonicity`].
//! - `praxis-core::receipt_epoch` — the `epoch` submodule.
//! - the `CoreError` variants the family uses.
//!
//! The 99-byte `OcelCausalFrame::to_hash_bytes` layout, the `DenialPolarity`
//! lane constants/scatter, and the `chain_hash(t+1) =
//! BLAKE3(chain_hash(t) || frame_bytes(t+1))` chain rule are ported verbatim
//! into the [`frame`] submodule from `bcinr-powl-receipt` 26.7.28. The
//! crates.io dependency itself was attempted first but is **infeasible**: its
//! transitive dependency `bcinr-powl` → `wasm4pm-compat` uses
//! `#![feature(...)]` (nightly-only), which fails to compile under graphlaw's
//! pinned stable `1.96.0` toolchain (E0554). The port is proven byte-exact by
//! the FM-CHAIN-009 golden test against ggen's committed TCPS fixture.

use serde::{Deserialize, Serialize};

use self::frame::{DenialPolarity, OcelCausalFrame, OcelCausalReceipt, PackedObjRef};

// ---------------------------------------------------------------------------
// Frame primitives (verbatim port of bcinr-powl-receipt 26.7.28:
// causal_receipt.rs + denial.rs — the exact bytes the chain hashes)
// ---------------------------------------------------------------------------

/// One OCEL causal frame + the denial lane bitfield + the rolling BLAKE3
/// receipt chain (byte-exact port of `bcinr-powl-receipt 26.7.28`).
pub mod frame {
    /// A packed object reference encoding type index (high 8 bits) and object
    /// id (low 24 bits) in a single `u32`.
    #[repr(transparent)]
    #[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Hash)]
    pub struct PackedObjRef(pub u32);

    /// Denial polarity bitfield: 8 single-byte lanes over a `u64` word
    /// (verbatim port of `bcinr-powl-receipt 26.7.28 denial.rs`).
    #[repr(transparent)]
    #[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
    pub struct DenialPolarity(pub u64);

    impl DenialPolarity {
        /// No denial — the step is admitted.
        pub const ADMITTED: Self = Self(0);

        /// Watchdog timer drained before the step completed (lane 7).
        pub const WATCHDOG_DRAINED: Self = Self(0xFF00_0000_0000_0000);

        /// A declared precondition was not satisfied (lane 1).
        pub const PRECONDITION_FAILED: Self = Self(0x0000_0000_0000_FF00);

        /// Service-level agreement deadline exceeded (lane 2).
        pub const SLA_BREACH: Self = Self(0x0000_0000_00FF_0000);

        /// Authorization proof absent or expired (lane 3).
        pub const AUTHORIZATION_DENIED: Self = Self(0x0000_0000_FF00_0000);

        /// Capacity or quota exhausted (lane 4).
        pub const RESOURCE_EXHAUSTED: Self = Self(0x0000_00FF_0000_0000);

        /// Object lifecycle law violated (lane 5).
        pub const OBJECT_LIFECYCLE_VIOLATION: Self = Self(0x0000_FF00_0000_0000);

        /// A process-conformance gate rejected the step (lane 6).
        pub const CONFORMANCE_GATE_FAILED: Self = Self(0x00FF_0000_0000_0000);

        /// Returns `true` when no denial lane is set.
        #[inline]
        #[must_use]
        pub fn is_admitted(self) -> bool {
            self.0 == 0
        }

        /// Compose two words by OR-ing their lanes. Admitted (zero) is the
        /// identity element: `compose(x, ADMITTED) == x`.
        #[inline]
        #[must_use]
        pub fn compose(self, other: Self) -> Self {
            Self(self.0 | other.0)
        }

        /// Branchless lane-to-bit scatter: each active byte lane in the word
        /// produces a distinct bit in the returned `u64`
        /// (verbatim port of `DenialPolarity::to_fired_mask`).
        #[inline]
        #[must_use]
        pub fn to_fired_mask(self) -> u64 {
            let w = self.0;
            let lane = |shift: u32| -> u64 {
                let byte = (w >> shift) & 0xFF;
                (byte | byte.wrapping_neg()) >> 63
            };

            lane(0)       // bit 0
            | (lane(8)  << 1)  // bit 1
            | (lane(16) << 2)  // bit 2
            | (lane(24) << 3)  // bit 3
            | (lane(32) << 4)  // bit 4
            | (lane(40) << 5)  // bit 5
            | (lane(48) << 6)  // bit 6
            | (lane(56) << 7) // bit 7
        }
    }

    /// One OCEL causal frame: a cache-line-sized record of a single
    /// manufacturing step, its denial verdict, its object set, and the rolling
    /// hash of its causal predecessor. Size: 128 bytes, aligned to 64.
    #[derive(Clone)]
    #[repr(C, align(64))]
    pub struct OcelCausalFrame {
        /// Monotonically increasing step identity within a run.
        pub instruction_id: u64,
        /// Scatter of active denial lanes (from
        /// [`DenialPolarity::to_fired_mask`]).
        pub fired_mask: u64,
        /// Denial polarity at the time this step was manufactured.
        pub denial: DenialPolarity,
        /// Up to 8 packed object references participating in this step.
        pub obj_refs: [PackedObjRef; 8],
        /// Wall-clock timestamp in nanoseconds.
        pub ts_ns: u64,
        /// Index into the activity table for this step's activity.
        pub activity_idx: u16,
        /// Classifier byte for the POWL node kind (XOR, SEQ, LOOP, etc.).
        pub node_kind: u8,
        /// Internal padding to maintain 128-byte alignment.
        pub pad: [u8; 5],
        /// BLAKE3 hash of the preceding frame (or genesis zeros for the first
        /// frame).
        pub prior_hash: [u8; 32],
    }

    const _: () = assert!(core::mem::size_of::<OcelCausalFrame>() == 128);

    impl OcelCausalFrame {
        /// Serialise this frame into a fixed-size byte buffer for hashing.
        /// Layout (all integers little-endian):
        /// `[0..8] instruction_id | [8..16] fired_mask | [16..24] denial.0 |
        /// [24..56] obj_refs (8 x u32 LE) | [56..64] ts_ns | [64..66]
        /// activity_idx | [66..67] node_kind | [67..99] prior_hash (verbatim)`.
        /// Total: 99 bytes.
        #[must_use]
        pub fn to_hash_bytes(&self) -> [u8; 99] {
            let mut buf = [0u8; 99];
            let mut pos = 0;

            for i in 0..8 {
                buf[pos + i] = ((self.instruction_id >> (i * 8)) & 0xFF) as u8;
            }
            pos += 8;

            for i in 0..8 {
                buf[pos + i] = ((self.fired_mask >> (i * 8)) & 0xFF) as u8;
            }
            pos += 8;

            for i in 0..8 {
                buf[pos + i] = ((self.denial.0 >> (i * 8)) & 0xFF) as u8;
            }
            pos += 8;

            for r in &self.obj_refs {
                let v = r.0;
                for i in 0..4 {
                    buf[pos + i] = ((v >> (i * 8)) & 0xFF) as u8;
                }
                pos += 4;
            }

            for i in 0..8 {
                buf[pos + i] = ((self.ts_ns >> (i * 8)) & 0xFF) as u8;
            }
            pos += 8;

            buf[pos] = (self.activity_idx & 0xFF) as u8;
            buf[pos + 1] = ((self.activity_idx >> 8) & 0xFF) as u8;
            pos += 2;

            buf[pos] = self.node_kind;
            pos += 1;

            buf[pos..pos + 32].copy_from_slice(&self.prior_hash);

            buf
        }
    }

    /// Rolling BLAKE3 receipt for an ordered sequence of [`OcelCausalFrame`]s:
    /// `chain_hash(t+1) = BLAKE3(chain_hash(t) || frame_bytes(t+1))`.
    /// The genesis hash is BLAKE3 of 32 zero bytes.
    pub struct OcelCausalReceipt {
        /// Current rolling hash (advances with each chain() call).
        pub chain_hash: [u8; 32],
        /// Number of frames chained so far.
        pub frame_count: u64,
        /// Opaque run identifier supplied at genesis.
        pub run_id: [u8; 32],
        /// Replay pointer: index of the last frame that can serve as a replay
        /// root.
        pub replay_ptr: u64,
    }

    impl OcelCausalReceipt {
        /// Create a genesis receipt for the given `run_id`. The initial
        /// `chain_hash` is BLAKE3 of 32 zero bytes.
        #[must_use]
        pub fn genesis(run_id: [u8; 32]) -> Self {
            let chain_hash: [u8; 32] = *blake3::hash(&[0u8; 32]).as_bytes();
            Self {
                chain_hash,
                frame_count: 0,
                run_id,
                replay_ptr: 0,
            }
        }

        /// Advance the chain by one frame:
        /// `BLAKE3(chain_hash || frame.to_hash_bytes())`.
        pub fn chain(&mut self, frame: &OcelCausalFrame) {
            let frame_bytes = frame.to_hash_bytes();
            let mut h = blake3::Hasher::new();
            h.update(&self.chain_hash);
            h.update(&frame_bytes);
            self.chain_hash = *h.finalize().as_bytes();
            self.frame_count += 1;
            self.replay_ptr = self.frame_count - 1;
        }
    }
}

// ---------------------------------------------------------------------------
// Errors (subset of praxis-core::error::CoreError used by the family)
// ---------------------------------------------------------------------------

/// Error type for the receipt-chain family (the `CoreError` variants the ported
/// family produces; names match praxis-core's `name()` strings exactly).
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
    /// Stable machine label, matching praxis-core's `CoreError::name()`.
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
// Obligation + Andon (port of praxis-core::law)
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
// Refusal taxonomy (port of praxis-core::refusal)
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

/// Praxis-native refusal scenarios: obligation-driven halts and the 7
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
// ReceiptMeta + frame construction (port of praxis-core::law)
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
pub fn chain_from_frame(prev_chain_hash: &[u8; 32], frame: &OcelCausalFrame) -> [u8; 32] {
    let mut receipt = OcelCausalReceipt::genesis([0u8; 32]);
    receipt.chain_hash = *prev_chain_hash;
    receipt.chain(frame);
    receipt.chain_hash
}

// ---------------------------------------------------------------------------
// Receipt epoch v2 (port of praxis-core::receipt_epoch)
// ---------------------------------------------------------------------------

/// Receipt schema epoch v2: an itemized admission ledger, a monotonic
/// standing ceiling, a closed artifact-equivalence map, and a
/// precedence-derived Andon level -- layered onto [`ReceiptRecord`]
/// without breaking v1 read compatibility.
pub mod epoch {
    use serde::{Deserialize, Serialize};

    use super::CoreError;

    /// Schema identity for the pre-epoch wire shape: no `v2` payload, andon is
    /// whatever [`super::Andon`] the emitting law object carried.
    pub const SCHEMA_V1: &str = "ggen-receipt/v1";

    /// Schema identity for the epoch-v2 wire shape: `v2` is populated with a real
    /// admission ledger, standing ceiling, equivalence map, and derived Andon.
    pub const SCHEMA_V2: &str = "ggen-receipt/v2";

    pub(crate) fn default_schema() -> String {
        SCHEMA_V1.to_string()
    }

    /// The v2 Andon precedence lattice. Declaration order is significant:
    /// `derive(Ord)` makes `Red < Yellow < Green`, so `.min()` over a set of
    /// levels implements the required "Red beats Yellow beats Green"
    /// aggregation.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
    pub enum AndonLevel {
        /// At least one admission item was refused this generation.
        Red,
        /// No refusals, but at least one admission item was quarantined.
        Yellow,
        /// Every admission item was cleanly admitted.
        Green,
    }

    /// The standing-ceiling lattice. `LegacyObserved` sits strictly between
    /// `Red` and `Yellow`: a chain that has ever passed through a v1
    /// (unitemized) generation is capped there and, under the identity
    /// `recoverable` policy documented on [`recoverable`], can never climb
    /// back to `Green` by mere meet-based inference.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
    pub enum CeilingLevel {
        /// At least one component/admission signal was Red.
        Red,
        /// Bounded by an unitemized v1-epoch ancestor; never promotable to Green
        /// by inference (see [`recoverable`]).
        LegacyObserved,
        /// No Red, but at least one component/admission signal was Yellow.
        Yellow,
        /// Every component/admission signal was Green. The lattice's top
        /// element: `meet(Green, x) == x` for any `x`.
        Green,
    }

    impl From<AndonLevel> for CeilingLevel {
        fn from(level: AndonLevel) -> Self {
            match level {
                AndonLevel::Red => CeilingLevel::Red,
                AndonLevel::Yellow => CeilingLevel::Yellow,
                AndonLevel::Green => CeilingLevel::Green,
            }
        }
    }

    /// Independent component results the standing ceiling is a meet over.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    pub struct ComponentLevels {
        /// `cargo clippy --all-targets -- -D warnings` result.
        pub lint: AndonLevel,
        /// `test-lib` (or the full suite, if run) result.
        pub test: AndonLevel,
        /// `fmt-check` result.
        pub fmt: AndonLevel,
        /// Aggregate of the remaining `just pre-commit` gates not named above.
        pub gate: AndonLevel,
    }

    impl ComponentLevels {
        /// All four components at the same level.
        #[must_use]
        pub fn uniform(level: AndonLevel) -> Self {
            Self {
                lint: level,
                test: level,
                fmt: level,
                gate: level,
            }
        }
    }

    /// `supported(evidence)`: the ceiling this generation's component evidence
    /// alone would support, with no reference to history.
    #[must_use]
    pub fn supported(components: &ComponentLevels) -> CeilingLevel {
        [
            components.lint,
            components.test,
            components.fmt,
            components.gate,
        ]
        .into_iter()
        .map(CeilingLevel::from)
        .min()
        .unwrap_or(CeilingLevel::Green)
    }

    /// `recoverable(ceiling_n)`: how much of the previous generation's ceiling
    /// carries forward into this generation's meet. The current policy is the
    /// identity function: nothing decays and nothing is inferred back up.
    #[must_use]
    pub fn recoverable(prev: CeilingLevel) -> CeilingLevel {
        prev
    }

    /// The ceiling monotonicity rule:
    /// `ceiling_{n+1} = meet(recoverable(ceiling_n), supported(evidence_{n+1}))`.
    #[must_use]
    pub fn compute_ceiling(prev: CeilingLevel, components: &ComponentLevels) -> CeilingLevel {
        recoverable(prev).min(supported(components))
    }

    /// Per-class equivalence status. `Unknown` is a first-class, explicit value.
    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    pub enum EquivalenceStatus {
        /// This generation did not evaluate this artifact class.
        Unknown,
        /// This artifact class was evaluated and found equivalent.
        Equivalent,
        /// This artifact class was evaluated and found divergent, with the
        /// reason recorded rather than dropped.
        Divergent(String),
    }

    /// A closed set over 8 artifact classes, one field per class so the type
    /// itself -- not a runtime check -- guarantees completeness.
    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    #[serde(deny_unknown_fields)]
    pub struct EquivalenceMap {
        /// Source files (ontologies, templates, hand-written Rust).
        pub source: EquivalenceStatus,
        /// Compiled build artifacts.
        pub compiled_binary: EquivalenceStatus,
        /// Documentation (`docs/`, `book/`).
        pub docs: EquivalenceStatus,
        /// Test files and their outcomes.
        pub tests: EquivalenceStatus,
        /// Prior receipts in the chain.
        pub receipts: EquivalenceStatus,
        /// Process/OCEL evidence.
        pub evidence: EquivalenceStatus,
        /// `just pre-commit` gate results.
        pub gates: EquivalenceStatus,
        /// Configuration (`ggen.toml`, `.specify/*.ttl`).
        pub config: EquivalenceStatus,
    }

    impl EquivalenceMap {
        /// Every class explicitly `Unknown`.
        #[must_use]
        pub fn all_unknown() -> Self {
            Self {
                source: EquivalenceStatus::Unknown,
                compiled_binary: EquivalenceStatus::Unknown,
                docs: EquivalenceStatus::Unknown,
                tests: EquivalenceStatus::Unknown,
                receipts: EquivalenceStatus::Unknown,
                evidence: EquivalenceStatus::Unknown,
                gates: EquivalenceStatus::Unknown,
                config: EquivalenceStatus::Unknown,
            }
        }
    }

    /// What was actually observed for one admitted evidence item.
    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    pub enum ObservedOutcome {
        /// The evidence item passed whatever check produced it.
        Pass,
        /// The evidence item failed whatever check produced it.
        Fail,
        /// No pass/fail outcome was observed (e.g. an informational item).
        Unknown,
    }

    /// What the system decided to do with one admitted evidence item.
    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    pub enum AdmissionDecision {
        /// Admitted cleanly.
        Admitted,
        /// Admitted provisionally; downstream promotion is blocked until
        /// resolved.
        Quarantined,
        /// Refused outright.
        Refused,
    }

    impl AdmissionDecision {
        /// This decision's contribution to the derived [`AndonLevel`].
        #[must_use]
        pub fn level(&self) -> AndonLevel {
            match self {
                AdmissionDecision::Admitted => AndonLevel::Green,
                AdmissionDecision::Quarantined => AndonLevel::Yellow,
                AdmissionDecision::Refused => AndonLevel::Red,
            }
        }
    }

    /// One entry in the v2 admission ledger.
    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    pub struct AdmissionItem {
        /// Identifier for the evidence item (e.g. an output path, a gate name).
        pub evidence_id: String,
        /// What was observed.
        pub observed_outcome: ObservedOutcome,
        /// What was decided.
        pub decision: AdmissionDecision,
        /// Why.
        pub reason: String,
        /// Obligation identifiers this item discharged.
        #[serde(default)]
        pub obligations_discharged: Vec<String>,
        /// Obligation identifiers this item created.
        #[serde(default)]
        pub obligations_created: Vec<String>,
    }

    /// The admission ledger: either a genuine v2 itemization (possibly empty)
    /// or [`AdmissionLedger::LegacyUnrecorded`], a sentinel meaning "this
    /// generation predates itemized admission tracking entirely."
    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    pub enum AdmissionLedger {
        /// Predates itemized admission tracking (a v1-epoch ancestor).
        LegacyUnrecorded,
        /// A real, possibly-empty itemization for this generation.
        Recorded(Vec<AdmissionItem>),
    }

    /// Derive the aggregate [`AndonLevel`] from an admission ledger, honoring
    /// strict `Red > Yellow > Green` precedence. `LegacyUnrecorded` never
    /// derives `Green`.
    #[must_use]
    pub fn derive_andon(ledger: &AdmissionLedger) -> AndonLevel {
        match ledger {
            AdmissionLedger::LegacyUnrecorded => AndonLevel::Yellow,
            AdmissionLedger::Recorded(items) => items
                .iter()
                .map(|item| item.decision.level())
                .min()
                .unwrap_or(AndonLevel::Green),
        }
    }

    /// Obligation accounting for one generation, computed strictly after
    /// admission processing (never guessed ahead of it).
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    pub enum ObligationCount {
        /// Obligation state was not itemized this generation (legacy-bounded).
        Unknown,
        /// `required` obligation-identifier occurrences created this generation,
        /// `discharged` occurrences resolved.
        Tracked {
            /// Total obligation-identifier occurrences created.
            required: u32,
            /// Total obligation-identifier occurrences discharged.
            discharged: u32,
        },
    }

    impl ObligationCount {
        /// `required - discharged`, saturating at 0. `None` for `Unknown`.
        #[must_use]
        pub fn remaining(&self) -> Option<u32> {
            match self {
                ObligationCount::Unknown => None,
                ObligationCount::Tracked {
                    required,
                    discharged,
                } => Some(required.saturating_sub(*discharged)),
            }
        }
    }

    /// Compute [`ObligationCount`] from the admission ledger, after processing
    /// every item.
    #[must_use]
    pub fn compute_obligation_count(ledger: &AdmissionLedger) -> ObligationCount {
        match ledger {
            AdmissionLedger::LegacyUnrecorded => ObligationCount::Unknown,
            AdmissionLedger::Recorded(items) => {
                let required: u32 = items
                    .iter()
                    .map(|i| i.obligations_created.len() as u32)
                    .sum();
                let discharged: u32 = items
                    .iter()
                    .map(|i| i.obligations_discharged.len() as u32)
                    .sum();
                ObligationCount::Tracked {
                    required,
                    discharged,
                }
            }
        }
    }

    /// The v2-only payload carried by [`super::ReceiptRecord::v2`].
    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    #[serde(deny_unknown_fields)]
    pub struct ReceiptEpochV2 {
        /// The itemized admission ledger.
        pub admission: AdmissionLedger,
        /// The standing ceiling after this generation (see [`compute_ceiling`]).
        pub standing_ceiling: CeilingLevel,
        /// The closed 8-class artifact-equivalence map.
        pub equivalence: EquivalenceMap,
        /// Obligation accounting, computed after admission processing.
        pub obligation_count: ObligationCount,
        /// The derived Andon level (see [`derive_andon`]).
        pub andon: AndonLevel,
        /// Whether this generation is eligible for promotion.
        pub promotion_eligible: bool,
    }

    impl ReceiptEpochV2 {
        /// The legacy-bounded reading of a v1 receipt: never derived from the
        /// v1 record's own fields, always the same fixed, honest sentinel value.
        #[must_use]
        pub fn legacy_bounded() -> Self {
            let admission = AdmissionLedger::LegacyUnrecorded;
            let andon = derive_andon(&admission);
            let obligation_count = compute_obligation_count(&admission);
            Self {
                admission,
                standing_ceiling: CeilingLevel::LegacyObserved,
                equivalence: EquivalenceMap::all_unknown(),
                obligation_count,
                andon,
                promotion_eligible: false,
            }
        }
    }

    /// A witnessed claim that the standing ceiling should rise from
    /// `from_ceiling` to `to_ceiling` -- the only lawful way to raise
    /// [`recoverable`]'s permanent floor.
    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    pub struct PromotionWitness {
        /// The ceiling this witness claims to promote FROM.
        pub from_ceiling: CeilingLevel,
        /// The ceiling this witness claims to promote TO.
        pub to_ceiling: CeilingLevel,
        /// References to the receipts that constitute the evidence this
        /// promotion is based on. Must be non-empty.
        pub evidence_receipts: Vec<String>,
        /// Identifiers of the obligations this promotion's evidence discharges.
        /// Must be non-empty.
        pub closed_obligations: Vec<String>,
        /// Identity of whoever verified the evidence. Must differ from the
        /// actuator identity performing the promotion.
        pub verifier_identity: String,
        /// Free-text statement of why this promotion is authorized.
        pub authorization_basis: String,
    }

    /// Validate a [`PromotionWitness`] against the actual current ceiling and
    /// the identity performing the promotion.
    ///
    /// # Errors
    /// [`CoreError::PromotionRefused`] naming the specific reason otherwise.
    pub fn validate_promotion(
        current_ceiling: CeilingLevel,
        actuator_identity: &str,
        witness: &PromotionWitness,
    ) -> Result<CeilingLevel, CoreError> {
        if witness.from_ceiling != current_ceiling {
            return Err(CoreError::PromotionRefused {
                reason: format!(
                    "witness.from_ceiling {:?} does not match the actual current ceiling {:?} \
                     -- stale or mismatched witness",
                    witness.from_ceiling, current_ceiling
                ),
            });
        }
        if witness.to_ceiling <= witness.from_ceiling {
            return Err(CoreError::PromotionRefused {
                reason: format!(
                    "to_ceiling {:?} does not exceed from_ceiling {:?} -- not a promotion \
                     (a lateral or downward move is ordinary generation's job, not a witness's)",
                    witness.to_ceiling, witness.from_ceiling
                ),
            });
        }
        if witness.evidence_receipts.is_empty() {
            return Err(CoreError::PromotionRefused {
                reason: "evidence_receipts is empty -- a promotion must cite the receipts its \
                      evidence is based on"
                    .to_string(),
            });
        }
        if witness.closed_obligations.is_empty() {
            return Err(CoreError::PromotionRefused {
                reason: "closed_obligations is empty -- a promotion must name the obligations \
                      whose closure justifies it"
                    .to_string(),
            });
        }
        if witness.verifier_identity.trim().is_empty() {
            return Err(CoreError::PromotionRefused {
                reason: "verifier_identity is empty".to_string(),
            });
        }
        if witness.verifier_identity == actuator_identity {
            return Err(CoreError::PromotionRefused {
                reason: format!(
                    "verifier_identity {:?} matches the actuator identity performing this \
                     promotion -- self-authored promotions are refused, a promotion must be \
                     vouched for by an identity distinct from whoever is applying it",
                    witness.verifier_identity
                ),
            });
        }
        if witness.authorization_basis.trim().is_empty() {
            return Err(CoreError::PromotionRefused {
                reason: "authorization_basis is empty -- a promotion must state what process, \
                      policy, or decision authorizes it"
                    .to_string(),
            });
        }
        Ok(witness.to_ceiling)
    }

    /// Builder for a genuine v2 [`ReceiptEpochV2`]: the only way to construct
    /// one outside of [`ReceiptEpochV2::legacy_bounded`].
    pub struct ReceiptEpochV2Builder {
        prev_ceiling: CeilingLevel,
        components: ComponentLevels,
        admission: Vec<AdmissionItem>,
        equivalence: EquivalenceMap,
        explicit_ceiling: Option<CeilingLevel>,
        /// Set only by [`Self::with_verified_promotion`].
        promoted: bool,
    }

    impl ReceiptEpochV2Builder {
        /// Start a builder chained onto `prev_ceiling` with this generation's
        /// component evidence.
        #[must_use]
        pub fn new(prev_ceiling: CeilingLevel, components: ComponentLevels) -> Self {
            Self {
                prev_ceiling,
                components,
                admission: Vec::new(),
                equivalence: EquivalenceMap::all_unknown(),
                explicit_ceiling: None,
                promoted: false,
            }
        }

        /// Append one admission item.
        #[must_use]
        pub fn admission_item(mut self, item: AdmissionItem) -> Self {
            self.admission.push(item);
            self
        }

        /// Set the full equivalence map (defaults to all-`Unknown`).
        #[must_use]
        pub fn equivalence(mut self, map: EquivalenceMap) -> Self {
            self.equivalence = map;
            self
        }

        /// Force a specific ceiling instead of the computed meet. Refused at
        /// [`Self::build`] time if it exceeds that meet.
        #[must_use]
        pub fn with_explicit_ceiling(mut self, level: CeilingLevel) -> Self {
            self.explicit_ceiling = Some(level);
            self
        }

        /// Apply a validated [`PromotionWitness`]: the one lawful way to set
        /// `standing_ceiling` above `compute_ceiling(prev, components)`.
        ///
        /// # Errors
        /// Returns [`CoreError::PromotionRefused`] if the witness fails any
        /// invariant `validate_promotion` checks.
        pub fn with_verified_promotion(
            mut self,
            witness: &PromotionWitness,
            actuator_identity: &str,
        ) -> Result<Self, CoreError> {
            let new_ceiling = validate_promotion(self.prev_ceiling, actuator_identity, witness)?;
            self.explicit_ceiling = Some(new_ceiling);
            self.promoted = true;
            Ok(self)
        }

        /// Compute the final [`ReceiptEpochV2`], refusing rather than silently
        /// clamping if an explicit ceiling override exceeds what the evidence
        /// supports.
        ///
        /// # Errors
        /// [`CoreError::CeilingExceedsMeet`] on an unwitnessed over-ceiling.
        pub fn build(self) -> Result<ReceiptEpochV2, CoreError> {
            let allowed = compute_ceiling(self.prev_ceiling, &self.components);
            let standing_ceiling = if self.promoted {
                self.explicit_ceiling
            } else {
                Some(match self.explicit_ceiling {
                    Some(requested) if requested > allowed => {
                        return Err(CoreError::CeilingExceedsMeet { requested, allowed });
                    }
                    Some(requested) => requested,
                    None => allowed,
                })
            }
            .expect("promoted=true is only ever set alongside explicit_ceiling");
            let ledger = AdmissionLedger::Recorded(self.admission);
            let andon = derive_andon(&ledger);
            let obligation_count = compute_obligation_count(&ledger);
            let promotion_eligible = andon == AndonLevel::Green;
            Ok(ReceiptEpochV2 {
                admission: ledger,
                standing_ceiling,
                equivalence: self.equivalence,
                obligation_count,
                andon,
                promotion_eligible,
            })
        }
    }

    /// Read the v2 epoch view of a [`super::ReceiptRecord`], dispatching on
    /// its declared `schema`.
    ///
    /// # Errors
    /// A real [`CoreError`] on an unrecognized schema or a schema/payload
    /// contradiction, never a silent default.
    pub fn read_receipt_epoch(record: &super::ReceiptRecord) -> Result<ReceiptEpochV2, CoreError> {
        match record.schema.as_str() {
            SCHEMA_V1 => {
                if record.v2.is_some() {
                    return Err(CoreError::ReceiptSchemaPayloadMismatch(format!(
                        "schema `{SCHEMA_V1}` but a v2 payload is present"
                    )));
                }
                Ok(ReceiptEpochV2::legacy_bounded())
            }
            SCHEMA_V2 => record.v2.clone().ok_or_else(|| {
                CoreError::ReceiptSchemaPayloadMismatch(format!(
                    "schema `{SCHEMA_V2}` but no v2 payload is present"
                ))
            }),
            other => Err(CoreError::UnrecognizedReceiptSchema(other.to_string())),
        }
    }
    // ---------------------------------------------------------------------------
    // ReceiptRecordV1Legacy -- models an old, pre-migration v1-only reader
    // ---------------------------------------------------------------------------

    /// The strict, `deny_unknown_fields` wire shape [`super::ReceiptRecord`]
    /// had before this module existed. Used only to model "an old v1-only
    /// binary" in tests: deserializing a genuine v2 receipt's JSON into this
    /// type must fail (the `schema`/`v2` keys are unknown to it), which is the
    /// concrete, checkable half of "old code refuses new receipts" this module
    /// promises rather than asserts.
    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    #[serde(deny_unknown_fields)]
    pub struct ReceiptRecordV1Legacy {
        /// Schema version; mirrors `RECEIPT_RECORD_VERSION`.
        pub version: u32,
        /// Monotonically increasing step identity within a run.
        pub instruction_id: u64,
        /// Index into the activity table for this step's activity.
        pub activity_idx: u16,
        /// Resolved human-readable label for `activity_idx`, if available.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pub activity: Option<String>,
        /// Classifier byte for the POWL node kind.
        pub node_kind: u8,
        /// Wall-clock timestamp in nanoseconds.
        pub ts_ns: u64,
        /// Optional wall-clock duration in milliseconds.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pub duration_ms: Option<u64>,
        /// BLAKE3 hash of the canonical JSON payload bytes.
        pub payload_hash_hex: String,
        /// The chain hash this record was chained onto.
        pub prev_chain_hash_hex: String,
        /// The resulting chain hash after this record.
        pub chain_hash_hex: String,
        /// The Andon outcome at receipt time.
        pub andon: super::Andon,
        /// Number of obligations attached to the law object at receipt time.
        pub obligation_count: u32,
        /// OCEL object identifiers this receipt governs.
        #[serde(default)]
        pub object_ids: Vec<String>,
        /// Hex-encoded ed25519 signature over `chain_hash_hex`.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pub signature_hex: Option<String>,
    }

    // ---------------------------------------------------------------------------
    // Migration receipt M_1_to_2
    // ---------------------------------------------------------------------------

    /// The fixed migration-law identity for the v1 -> v2 boundary.
    pub const MIGRATION_LAW_1_TO_2: &str = "M_1_to_2";

    /// A single record binding the final v1 chain hash to the first v2 chain
    /// hash: what carries forward, what becomes `Unknown`, and the resulting
    /// (necessarily capped) ceiling.
    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    #[serde(deny_unknown_fields)]
    pub struct MigrationReceipt {
        /// Fixed migration-law identity, always [`MIGRATION_LAW_1_TO_2`].
        pub migration_law: String,
        /// The schema identity being migrated from, always [`SCHEMA_V1`].
        pub from_schema: String,
        /// The schema identity being migrated to, always [`SCHEMA_V2`].
        pub to_schema: String,
        /// The final v1 receipt's chain hash (hex).
        pub final_v1_chain_hash_hex: String,
        /// The first v2 receipt's chain hash (hex), chained onto the field above.
        pub first_v2_chain_hash_hex: String,
        /// What information carries forward across the boundary.
        pub carries_forward: Vec<String>,
        /// What information becomes `Unknown` across the boundary (never
        /// silently dropped -- named explicitly).
        pub becomes_unknown: Vec<String>,
        /// The ceiling after migration -- always [`CeilingLevel::LegacyObserved`]:
        /// necessarily capped, because the v1 history it is built on was never
        /// itemized.
        pub resulting_ceiling: CeilingLevel,
    }

    impl MigrationReceipt {
        /// Construct the migration receipt binding `final_v1_chain_hash_hex` to
        /// `first_v2_chain_hash_hex`. `resulting_ceiling` is always
        /// [`CeilingLevel::LegacyObserved`] -- there is no constructor argument
        /// for it, so a migration receipt can never claim a higher ceiling than
        /// the v1 history actually supports.
        #[must_use]
        pub fn new(
            final_v1_chain_hash_hex: impl Into<String>,
            first_v2_chain_hash_hex: impl Into<String>,
        ) -> Self {
            Self {
                migration_law: MIGRATION_LAW_1_TO_2.to_string(),
                from_schema: SCHEMA_V1.to_string(),
                to_schema: SCHEMA_V2.to_string(),
                final_v1_chain_hash_hex: final_v1_chain_hash_hex.into(),
                first_v2_chain_hash_hex: first_v2_chain_hash_hex.into(),
                carries_forward: vec![
                    "chain_hash_lineage".to_string(),
                    "object_ids".to_string(),
                    "instruction_id_sequence".to_string(),
                ],
                becomes_unknown: vec![
                    "admission".to_string(),
                    "equivalence".to_string(),
                    "obligation_count".to_string(),
                ],
                resulting_ceiling: CeilingLevel::LegacyObserved,
            }
        }
    }
}

// ---------------------------------------------------------------------------
// ReceiptRecord (port of praxis-core::receipt_record)
// ---------------------------------------------------------------------------

/// Current schema version for [`ReceiptRecord`].
pub const RECEIPT_RECORD_VERSION: u32 = 1;

/// Chain-rule discriminator value for the v2-fold rule (the F1 fix): the
/// base admission-frame chain hash, then `schema` and the full `v2` payload
/// folded in by `fold_in_v2_epoch` (a strict no-op when `v2` is `None`).
pub const CHAIN_RULE_V2_FOLD: &str = "praxis-chain/v2-fold";

/// Chain-rule discriminator value for the pre-F1 base rule: the admission
/// frame chain hash alone, with no `v2` fold.
pub const CHAIN_RULE_BASE: &str = "praxis-chain/base";

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
    /// emission path uses, then folds [`Self::v2`] in via
    /// [`fold_in_v2_epoch`].
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

// ---------------------------------------------------------------------------
// ReceiptValidator (port of praxis-core::receipt_validator)
// ---------------------------------------------------------------------------

/// Staged, replayable verification of a receipt ledger (port of
/// `praxis-core::receipt_validator`). Runs every stage and reports all
/// outcomes rather than short-circuiting.
///
/// Stages, in order: `schema`, `chain_recompute` (tamper detection),
/// `chain_linkage`, `monotonic`. The praxis `token_replay` stage is
/// intentionally absent: it depends on `replay_adapter`/POWL lifecycle
/// machinery that is not part of this port.
pub mod validator {
    use std::time::{SystemTime, UNIX_EPOCH};

    use serde::{Deserialize, Serialize};

    use super::{ChainRuleMonotonicity, ChainVerification, RECEIPT_RECORD_VERSION, ReceiptRecord};

    /// Injectable wall-clock so "timestamp not in the future" checks are
    /// deterministic in tests.
    pub trait Clock: Sync {
        /// Current time in nanoseconds since the UNIX epoch.
        fn now_ns(&self) -> u64;
    }

    /// Real wall-clock via `SystemTime::now()`.
    #[derive(Debug, Default, Clone, Copy)]
    pub struct SystemClock;

    impl Clock for SystemClock {
        fn now_ns(&self) -> u64 {
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos() as u64
        }
    }

    /// A fixed clock for deterministic tests: always reports the same `now_ns()`.
    #[derive(Debug, Clone, Copy)]
    pub struct FixedClock(pub u64);

    impl Clock for FixedClock {
        fn now_ns(&self) -> u64 {
            self.0
        }
    }

    /// The outcome of a single validation stage.
    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    pub enum CheckOutcome {
        /// The stage found no issues.
        Pass,
        /// The stage found a problem, described in the message.
        Fail(String),
        /// The stage was not applicable (e.g. no records to check).
        Skip(String),
    }

    impl CheckOutcome {
        /// `true` iff this outcome is [`CheckOutcome::Pass`].
        #[must_use]
        pub fn is_pass(&self) -> bool {
            matches!(self, CheckOutcome::Pass)
        }
    }

    /// One stage's name and outcome.
    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    pub struct StageResult {
        /// Stage identifier (`"schema"`, `"chain_recompute"`, `"chain_linkage"`,
        /// `"monotonic"`).
        pub stage: String,
        /// The stage's outcome.
        pub outcome: CheckOutcome,
    }

    impl StageResult {
        fn new(stage: &'static str, outcome: CheckOutcome) -> Self {
            Self {
                stage: stage.to_string(),
                outcome,
            }
        }
    }

    /// The full validation result: every stage's outcome plus an overall verdict.
    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    pub struct Verdict {
        /// `true` iff every stage passed (or was skipped).
        pub ok: bool,
        /// Per-stage results, in pipeline order.
        pub stages: Vec<StageResult>,
        /// Number of records validated.
        pub records_checked: usize,
    }

    /// Runs the staged validation pipeline over a receipt ledger.
    pub struct ReceiptValidator;

    impl ReceiptValidator {
        /// Validate `records` against every stage, using `clock` for the
        /// `monotonic` stage's "not in the future" check.
        #[must_use]
        pub fn validate(records: &[ReceiptRecord], clock: &dyn Clock) -> Verdict {
            let stages = vec![
                Self::check_schema(records),
                Self::check_chain_recompute(records),
                Self::check_chain_linkage(records),
                Self::check_monotonic(records, clock),
            ];
            let ok = stages
                .iter()
                .all(|s| s.outcome.is_pass() || matches!(s.outcome, CheckOutcome::Skip(_)));
            Verdict {
                ok,
                stages,
                records_checked: records.len(),
            }
        }

        fn check_schema(records: &[ReceiptRecord]) -> StageResult {
            for (i, record) in records.iter().enumerate() {
                if record.version != RECEIPT_RECORD_VERSION {
                    return StageResult::new(
                        "schema",
                        CheckOutcome::Fail(format!(
                            "record {i}: unsupported schema version {} (expected {})",
                            record.version, RECEIPT_RECORD_VERSION
                        )),
                    );
                }
                if let Err(e) = record.payload_hash() {
                    return StageResult::new(
                        "schema",
                        CheckOutcome::Fail(format!("record {i}: {e}")),
                    );
                }
                if let Err(e) = record.prev_chain_hash() {
                    return StageResult::new(
                        "schema",
                        CheckOutcome::Fail(format!("record {i}: {e}")),
                    );
                }
                if let Err(e) = record.chain_hash() {
                    return StageResult::new(
                        "schema",
                        CheckOutcome::Fail(format!("record {i}: {e}")),
                    );
                }
            }
            StageResult::new("schema", CheckOutcome::Pass)
        }

        fn check_chain_recompute(records: &[ReceiptRecord]) -> StageResult {
            // Rule-aware (FM-CHAIN-009): each record is checked under the
            // chain rule that sealed it, and a legacy base-rule record after
            // a declared one is a refused downgrade.
            let mut monotonic = ChainRuleMonotonicity::new();
            for (i, record) in records.iter().enumerate() {
                let Ok(claimed) = record.chain_hash() else {
                    continue; // already reported by `schema`
                };
                match record.verify_chain_against(claimed) {
                    Ok(ChainVerification::Verified(standing)) => {
                        if let Err(e) = monotonic.observe(i, record, standing) {
                            return StageResult::new(
                                "chain_recompute",
                                CheckOutcome::Fail(format!("record {i}: {e}")),
                            );
                        }
                    }
                    Ok(ChainVerification::Mismatch { .. }) => {
                        return StageResult::new(
                            "chain_recompute",
                            CheckOutcome::Fail(format!(
                                "record {i}: recomputed chain hash does not match stored chain_hash_hex (tamper detected)"
                            )),
                        );
                    }
                    Err(e) => {
                        return StageResult::new(
                            "chain_recompute",
                            CheckOutcome::Fail(format!("record {i}: {e}")),
                        );
                    }
                }
            }
            StageResult::new("chain_recompute", CheckOutcome::Pass)
        }

        fn check_chain_linkage(records: &[ReceiptRecord]) -> StageResult {
            if !records.is_empty() {
                let genesis_hex = hex::encode(super::store::GENESIS_CHAIN_HASH);
                if records[0].prev_chain_hash_hex != genesis_hex {
                    return StageResult::new(
                        "chain_linkage",
                        CheckOutcome::Fail(format!(
                            "record 0: prev_chain_hash_hex ({}) does not match genesis anchor ({})",
                            records[0].prev_chain_hash_hex, genesis_hex
                        )),
                    );
                }
            }
            for i in 1..records.len() {
                let expected_prev = &records[i - 1].chain_hash_hex;
                let actual_prev = &records[i].prev_chain_hash_hex;
                if expected_prev != actual_prev {
                    return StageResult::new(
                        "chain_linkage",
                        CheckOutcome::Fail(format!(
                            "record {i}: prev_chain_hash_hex ({actual_prev}) != record {}'s chain_hash_hex ({expected_prev})",
                            i - 1
                        )),
                    );
                }
            }
            StageResult::new("chain_linkage", CheckOutcome::Pass)
        }

        fn check_monotonic(records: &[ReceiptRecord], clock: &dyn Clock) -> StageResult {
            let now = clock.now_ns();
            let mut prev_instruction: Option<u64> = None;
            let mut prev_ts: Option<u64> = None;

            for (i, record) in records.iter().enumerate() {
                if record.ts_ns > now {
                    return StageResult::new(
                        "monotonic",
                        CheckOutcome::Fail(format!(
                            "record {i}: ts_ns ({}) is in the future (now={now})",
                            record.ts_ns
                        )),
                    );
                }
                if let Some(prev) = prev_instruction
                    && record.instruction_id <= prev
                {
                    return StageResult::new(
                        "monotonic",
                        CheckOutcome::Fail(format!(
                            "record {i}: instruction_id ({}) not strictly increasing after {prev}",
                            record.instruction_id
                        )),
                    );
                }
                if let Some(prev) = prev_ts
                    && record.ts_ns < prev
                {
                    return StageResult::new(
                        "monotonic",
                        CheckOutcome::Fail(format!(
                            "record {i}: ts_ns ({}) decreased from previous record's {prev}",
                            record.ts_ns
                        )),
                    );
                }
                prev_instruction = Some(record.instruction_id);
                prev_ts = Some(record.ts_ns);
            }
            StageResult::new("monotonic", CheckOutcome::Pass)
        }
    }
}

// ---------------------------------------------------------------------------
// ReceiptStore (port of praxis-core::receipt_store — append-only JSONL)
// ---------------------------------------------------------------------------

/// Append-only JSONL persistence for [`ReceiptRecord`]s (port of
/// `praxis-core::receipt_store`). One JSON object per line, opened in
/// append+create mode, no in-place rewrites.
pub mod store {
    use std::fs::OpenOptions;
    use std::io::Write as _;
    use std::path::{Path, PathBuf};

    use super::CoreError;
    use super::ReceiptRecord;

    /// Default receipts directory when no configured path is available.
    pub const DEFAULT_RECEIPTS_DIR: &str = "receipts";

    /// Ledger file name within the receipts directory.
    pub const LEDGER_FILE_NAME: &str = "receipts.jsonl";

    /// Genesis chain hash (32 zero bytes) used when the ledger has no entries
    /// yet.
    pub const GENESIS_CHAIN_HASH: [u8; 32] = [0u8; 32];

    /// Append-only JSONL receipt ledger.
    pub struct ReceiptStore {
        path: PathBuf,
    }

    impl ReceiptStore {
        /// Open (or prepare to create) a store at `<dir>/receipts.jsonl`,
        /// creating `dir` if it doesn't exist yet. Does not create the ledger
        /// file itself until the first [`ReceiptStore::append`].
        ///
        /// # Errors
        /// Returns [`CoreError::Io`] if `dir` cannot be created.
        pub fn open(dir: impl AsRef<Path>) -> Result<Self, CoreError> {
            let dir = dir.as_ref();
            std::fs::create_dir_all(dir).map_err(|e| CoreError::Io(e.to_string()))?;
            Ok(Self {
                path: dir.join(LEDGER_FILE_NAME),
            })
        }

        /// Open the default store ([`DEFAULT_RECEIPTS_DIR`]`/receipts.jsonl`).
        ///
        /// # Errors
        /// See [`ReceiptStore::open`].
        pub fn open_default() -> Result<Self, CoreError> {
            Self::open(DEFAULT_RECEIPTS_DIR)
        }

        /// The ledger file path.
        #[must_use]
        pub fn path(&self) -> &Path {
            &self.path
        }

        /// Append one record as a single JSON line.
        ///
        /// # Errors
        /// Returns [`CoreError::SerializationFailed`] if `record` fails to
        /// serialize, or [`CoreError::Io`] if the file can't be opened/written.
        pub fn append(&self, record: &ReceiptRecord) -> Result<(), CoreError> {
            let line = serde_json::to_string(record)
                .map_err(|e| CoreError::SerializationFailed(e.to_string()))?;
            let mut file = OpenOptions::new()
                .create(true)
                .append(true)
                .open(&self.path)
                .map_err(|e| CoreError::Io(e.to_string()))?;
            writeln!(file, "{line}").map_err(|e| CoreError::Io(e.to_string()))?;
            Ok(())
        }

        /// Load every record in the ledger, in append order. Returns an empty
        /// `Vec` if the ledger file doesn't exist yet.
        ///
        /// # Errors
        /// Returns [`CoreError::Io`] if an existing ledger can't be read, or
        /// [`CoreError::SerializationFailed`] if a line isn't a valid
        /// [`ReceiptRecord`].
        pub fn load_all(&self) -> Result<Vec<ReceiptRecord>, CoreError> {
            if !self.path.exists() {
                return Ok(Vec::new());
            }
            let content =
                std::fs::read_to_string(&self.path).map_err(|e| CoreError::Io(e.to_string()))?;
            content
                .lines()
                .filter(|l| !l.trim().is_empty())
                .map(|l| {
                    serde_json::from_str(l)
                        .map_err(|e| CoreError::SerializationFailed(e.to_string()))
                })
                .collect()
        }

        /// The `chain_hash` of the last record, or [`GENESIS_CHAIN_HASH`] if
        /// the ledger is empty (or doesn't exist yet).
        ///
        /// # Errors
        /// See [`ReceiptStore::load_all`], plus [`CoreError::HexDecodeFailed`]
        /// if the last record's `chain_hash_hex` is malformed.
        pub fn last_chain_hash(&self) -> Result<[u8; 32], CoreError> {
            let records = self.load_all()?;
            match records.last() {
                Some(r) => r.chain_hash(),
                None => Ok(GENESIS_CHAIN_HASH),
            }
        }
    }
}
