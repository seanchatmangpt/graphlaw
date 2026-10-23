//! Receipt chain: verbatim port of graphlaw-core's receipt-chain family so
//! `graphlaw-core` can retire (see ggen `docs/v26_10_10_graphlaw_retirement_plan.md`).
//!
//! Sources ported verbatim (wire shapes are frozen; hashes are byte-exact):
//! - `graphlaw-core::law` — [`Andon`], [`Obligation`], [`ReceiptMeta`],
//!   [`build_admission_frame`], [`chain_from_frame`].
//! - `graphlaw-core::refusal` — [`RefusalScenario`], [`RefusalCategory`].
//! - `graphlaw-core::receipt_record` — [`ReceiptRecord`], [`ChainRule`],
//!   [`ChainStanding`], [`ChainVerification`], [`ChainRuleMonotonicity`].
//! - `graphlaw-core::receipt_epoch` — the `epoch` submodule.
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
//!
//! # Doctest coverage note
//!
//! Behavior-bearing items carry compiling doctests (see [`frame`],
//! [`build_admission_frame`], [`chain_from_frame`]). The following pub items
//! intentionally do not, with the reason recorded here rather than repeated
//! per-item:
//!
//! - **Data-only structs/enums** ([`CoreError`], [`RefusalCategory`],
//!   [`RefusalScenario`], [`ReceiptMeta`], [`ReceiptRecord`],
//!   [`ChainStanding`], [`ChainVerification`], and every `epoch` type):
//!   plain field carriers with no behavior to demonstrate; their invariants
//!   are exercised by the integration courts (`tests/trust_attest.rs`,
//!   `tests/chicago_validation.rs`), not by doctests.
//! - **`store` items** ([`store::ReceiptStore`]): doctests would write to the
//!   ambient working directory; filesystem round-trips are covered by real
//!   I/O courts in `tests/`, which is the Chicago-school home for them.
//! - **`validator` items**: covered end-to-end by
//!   `cargo test --test trust_attest` over real ledgers.
//!
//! [`store::ReceiptStore`]: store::ReceiptStore

pub mod frame;
mod record;
pub mod store;

pub use record::*;

// ---------------------------------------------------------------------------
// Receipt epoch v2 (port of graphlaw-core::receipt_epoch)
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
        /// Monotonic generation counter recorded by the producer, read back
        /// verbatim on the wire. Receipts that predate generation tracking
        /// deserialize as `0` via the serde default — never synthesized from
        /// timestamps (same fail-closed discipline as the `LegacyUnrecorded`
        /// → `Yellow` precedent in [`derive_andon`]).
        ///
        /// Serialization skips `0` so every pre-generation wire payload (and
        /// therefore every pinned chain-hash golden) stays byte-identical.
        #[serde(default, skip_serializing_if = "generation_is_legacy")]
        pub generation: u64,
    }

    /// Serde `skip_serializing_if` predicate: true at the legacy `0`
    /// generation sentinel, so pre-generation payloads serialize
    /// byte-identically to before the field existed.
    fn generation_is_legacy(generation: &u64) -> bool {
        *generation == 0
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
        /// Monotonic generation counter for this epoch, read back verbatim.
        /// Legacy receipts that predate generation tracking deserialize as
        /// `0` via the serde default — never synthesized from timestamps.
        /// This is the same fail-closed migration shape as the admission
        /// ledger: missing evidence reads as an explicit sentinel
        /// (`LegacyUnrecorded` → `Yellow`, generation absent → `0`), not as
        /// invented evidence.
        ///
        /// Serialization skips `0` so pre-generation v2 payloads (and their
        /// pinned fold-rule chain hashes) stay byte-identical.
        #[serde(default, skip_serializing_if = "generation_is_legacy")]
        pub generation: u64,
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
                generation: 0,
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
        /// Monotonic generation counter; defaults to `0` (legacy-equivalent)
        /// unless set via [`Self::with_generation`].
        generation: u64,
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
                generation: 0,
            }
        }

        /// Record the monotonic generation counter for the epoch being
        /// built. The value is stored verbatim; the default of `0` is the
        /// legacy-equivalent sentinel, never a timestamp-derived guess.
        #[must_use]
        pub fn with_generation(mut self, generation: u64) -> Self {
            self.generation = generation;
            self
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
                generation: self.generation,
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
// ReceiptValidator (port of graphlaw-core::receipt_validator)
// ---------------------------------------------------------------------------

/// Staged, replayable verification of a receipt ledger (port of
/// `graphlaw-core::receipt_validator`). Runs every stage and reports all
/// outcomes rather than short-circuiting.
///
/// Stages, in order: `schema`, `chain_recompute` (tamper detection),
/// `chain_linkage`, `monotonic`. The graphlaw `token_replay` stage is
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
