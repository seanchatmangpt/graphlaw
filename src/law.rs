//! Content-addressed law states and receipted transitions.
//!
//! A [`LawState`] is an immutable RDF dataset identified by the SHA-256 of its
//! RDFC-1.0 canonical form, so two states with the same statements have the same
//! id regardless of blank-node labels or statement order. A transition is
//! `admit -> derive -> receipt`: every step names the upstream authority that
//! executed it, and a refused step yields no new state.

use std::sync::Arc;

use sha2::{Digest, Sha256};

use crate::dialect::{Dialect, Refusal, RefusalKind, parse_rdf};
use crate::{BACKEND_AUTHORITIES, BackendAuthority};

/// An immutable, content-addressed RDF dataset.
#[derive(Debug, Clone)]
pub struct LawState {
    dataset: Arc<purrdf::RdfDataset>,
    id: String,
}

/// One operation applied to a state.
#[derive(Debug, Clone, Copy)]
pub enum Step<'a> {
    /// SHACL admission gate (Turtle shapes graph). Never changes the state.
    AdmitShacl { shapes_ttl: &'a str },
    /// Forward-chain Notation3 rules with Eyeron; derived triples are added.
    DeriveN3 { rules: &'a str },
    /// Run a knowledge-hook pack to a fixpoint.
    Hooks { pack: &'a crate::hooks::HookPack },
    /// Replay a candidate plan; refused at the first violated precondition.
    Plan { plan: &'a crate::plan::Plan },
    /// Admission gate: a receipt for `step` must be recorded in the state
    /// (see [`crate::receipt::record`]). Never changes the state.
    RequireReceipt { step: &'a str },
    /// Admission gate: a receipt for `step` must be recorded **with a valid
    /// attestation by a trusted key** (see [`crate::receipt::record_signed`]).
    /// Never changes the state.
    RequireSignedReceipt {
        step: &'a str,
        trusted: &'a crate::attest::TrustedKeys,
    },
    /// RDFS entailment closure.
    EntailRdfs,
    /// OWL 2 RL entailment closure.
    EntailOwlRl,
}

impl Step<'_> {
    /// Stable step name recorded in receipts and matched by [`Lease::scope`].
    pub fn name(&self) -> &'static str {
        match self {
            Step::AdmitShacl { .. } => "admit:shacl",
            Step::DeriveN3 { .. } => "derive:n3",
            Step::Hooks { .. } => "derive:hooks",
            Step::Plan { .. } => "admit:plan",
            Step::RequireReceipt { .. } => "admit:require-receipt",
            Step::RequireSignedReceipt { .. } => "admit:require-signed-receipt",
            Step::EntailRdfs => "derive:rdfs",
            Step::EntailOwlRl => "derive:owl-rl",
        }
    }

    /// Ceiling a lease must grant: gates observe, plans select, derivations construct.
    pub fn required_ceiling(&self) -> Ceiling {
        match self {
            Step::AdmitShacl { .. }
            | Step::RequireReceipt { .. }
            | Step::RequireSignedReceipt { .. } => Ceiling::Observe,
            Step::Plan { .. } => Ceiling::Select,
            Step::Hooks { .. } | Step::DeriveN3 { .. } | Step::EntailRdfs | Step::EntailOwlRl => {
                Ceiling::Construct
            }
        }
    }

    fn capability(&self) -> &'static str {
        match self {
            Step::AdmitShacl { .. } => "SHACL",
            Step::DeriveN3 { .. } => "Notation3",
            Step::Hooks { .. } => "Knowledge hooks (kh: orchestration over SPARQL)",
            Step::Plan { .. } | Step::RequireReceipt { .. } | Step::RequireSignedReceipt { .. } => {
                "RDF 1.2 / codecs / storage IR"
            }
            Step::EntailRdfs | Step::EntailOwlRl => "RDF/RDFS/OWL-RL entailment",
        }
    }
}

/// Evidence that a step ran, and under which upstream authority.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Receipt {
    pub parent: String,
    pub child: String,
    pub step: &'static str,
    pub authority: BackendAuthority,
    /// Quads added by the step (0 for an admission gate).
    pub added: usize,
    /// Id of the [`Lease`] the step ran under (`None` for unleased steps).
    pub lease_id: Option<String>,
    /// SHA-256 digest of the admitted plan (plan-action receipts only).
    pub plan_sha256: Option<String>,
    /// Caller-provided digest of the external subject this receipt is about
    /// (e.g. a git commit or artifact); included in the signed payload.
    pub subject_sha256: Option<String>,
}

impl Receipt {
    /// Bind this receipt to an external subject digest (set before signing).
    pub fn with_subject(mut self, subject_sha256: impl Into<String>) -> Self {
        self.subject_sha256 = Some(subject_sha256.into());
        self
    }
}

/// Authority ceiling, ordered `Observe < Select < Construct`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Ceiling {
    /// Read-only gates that never change the state (`admit:shacl`, `admit:require-receipt`).
    Observe,
    /// Selection: replaying/admitting a candidate plan (`admit:plan`).
    Select,
    /// Construction: steps that derive new triples (`derive:*`).
    Construct,
}

impl Ceiling {
    pub fn name(self) -> &'static str {
        match self {
            Ceiling::Observe => "observe",
            Ceiling::Select => "select",
            Ceiling::Construct => "construct",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "observe" => Some(Ceiling::Observe),
            "select" => Some(Ceiling::Select),
            "construct" => Some(Ceiling::Construct),
            _ => None,
        }
    }
}

/// A time-boxed grant of authority: who may run which steps, up to which ceiling.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Lease {
    pub id: String,
    pub holder: String,
    pub ceiling: Ceiling,
    /// Step names (`Step::name`, e.g. `derive:rdfs`) the lease covers.
    pub scope: Vec<String>,
    /// The lease is expired when `now_unix >= expires_unix`.
    pub expires_unix: u64,
    /// When the issuer says it issued the lease. A verifier whose trusted
    /// clock is more than `max_skew_secs` *behind* this refuses with
    /// [`LeaseReason::ClockSkew`]. `0` means "not stated".
    pub issued_unix: u64,
}

/// Source of trusted time. Expiry is judged against the *verifier's* clock,
/// never against a timestamp the requester supplies.
pub trait Clock {
    fn now_unix(&self) -> u64;
}

/// The host wall clock. A clock before the Unix epoch reads `u64::MAX`, so
/// every lease is refused as expired (fail closed).
#[derive(Debug, Clone, Copy, Default)]
pub struct SystemClock;

impl Clock for SystemClock {
    fn now_unix(&self) -> u64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(u64::MAX, |d| d.as_secs())
    }
}

/// A clock frozen at a chosen instant (tests, replays, auditors pinning a time).
#[derive(Debug, Clone, Copy)]
pub struct FixedClock(pub u64);

impl Clock for FixedClock {
    fn now_unix(&self) -> u64 {
        self.0
    }
}

/// Default tolerated issuer clock lead, in seconds.
pub const DEFAULT_MAX_SKEW_SECS: u64 = 60;

/// A [`Lease`] plus an Ed25519 attestation by the issuing authority key.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SignedLease {
    pub lease: Lease,
    pub attestation: crate::attest::Attestation,
}

impl SignedLease {
    /// Full authorization: signature by a trusted key, then skew, then the
    /// lease's own expiry/scope/ceiling checks at `clock.now_unix()`.
    pub fn authorize(
        &self,
        step: &str,
        required: Ceiling,
        trusted: &crate::attest::TrustedKeys,
        clock: &dyn Clock,
        max_skew_secs: u64,
    ) -> Result<(), LawError> {
        let refuse = |reason| LawError::LeaseRefused {
            lease_id: self.lease.id.clone(),
            step: step.to_string(),
            reason,
        };
        crate::attest::verify_lease(self, trusted).map_err(|e| {
            refuse(match e {
                crate::attest::AttestError::UnknownKey => LeaseReason::UntrustedKey,
                _ => LeaseReason::BadSignature,
            })
        })?;
        let now = clock.now_unix();
        if self.lease.issued_unix > now.saturating_add(max_skew_secs) {
            return Err(refuse(LeaseReason::ClockSkew));
        }
        self.lease.authorize(step, required, now)
    }
}

/// Why a lease refused a step.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LeaseReason {
    Expired,
    OutOfScope,
    Ceiling,
    /// The lease signature does not verify (or the lease was altered).
    BadSignature,
    /// The lease is signed by a key outside the verifier's trusted set.
    UntrustedKey,
    /// The lease claims to be issued in the verifier's future beyond the skew bound.
    ClockSkew,
}

impl LeaseReason {
    /// Stable snake_case code used by the ABI and `Display`.
    pub fn as_str(self) -> &'static str {
        match self {
            LeaseReason::Expired => "expired",
            LeaseReason::OutOfScope => "out_of_scope",
            LeaseReason::Ceiling => "ceiling",
            LeaseReason::BadSignature => "bad_signature",
            LeaseReason::UntrustedKey => "untrusted_key",
            LeaseReason::ClockSkew => "clock_skew",
        }
    }
}

impl Lease {
    /// Check a step (by name and required ceiling) against this lease at a
    /// caller-supplied `now_unix`. **Proves nothing about who issued the lease
    /// or what time it is**; production callers use [`SignedLease::authorize`].
    pub fn authorize(&self, step: &str, required: Ceiling, now_unix: u64) -> Result<(), LawError> {
        let reason = if now_unix >= self.expires_unix {
            LeaseReason::Expired
        } else if !self.scope.iter().any(|s| s == step) {
            LeaseReason::OutOfScope
        } else if required > self.ceiling {
            LeaseReason::Ceiling
        } else {
            return Ok(());
        };
        Err(LawError::LeaseRefused {
            lease_id: self.id.clone(),
            step: step.to_string(),
            reason,
        })
    }
}

/// Fixpoint-step cap for N3 derivation (Eyeron `ReasonerOptions::max_iterations`;
/// Eyeron's own default is 1,000,000). Exceeding it is a `ResourceLimit` refusal.
pub const N3_MAX_ITERATIONS: usize = 4_000;

/// Why bounded N3 reasoning did not return a derivation.
#[derive(Debug, Clone)]
pub enum N3Error {
    /// A reasoner safety limit was hit after `observed` fixpoint steps.
    Limit { observed: usize, summary: String },
    /// Parse error, semantic error, or other Eyeron refusal.
    Refused(Refusal),
}

/// Forward-reason `input` with GraphLaw's resource limits; returns the N3 text
/// of newly derived triples (same contract as `eyeron::reason`).
pub fn reason_n3_bounded(input: &str) -> Result<String, N3Error> {
    let refuse = |e: eyeron::EyeronError| N3Error::Refused(Refusal::engine(Dialect::N3, e));
    let doc = if eyeron::is_rdf_message_log(input) {
        eyeron::parse_rdf_message_log(input, None)
    } else {
        eyeron::parse_n3(input, None)
    }
    .map_err(refuse)?;
    let options = eyeron::ReasonerOptions {
        include_explicit: false,
        max_iterations: N3_MAX_ITERATIONS,
        ..eyeron::ReasonerOptions::default()
    };
    let result = eyeron::reason_document(&doc, &options);
    if let Some(summary) = result.incomplete_summary() {
        if !result.limits_reached.is_empty() {
            return Err(N3Error::Limit {
                observed: result.statistics.iterations,
                summary,
            });
        }
        return Err(refuse(eyeron::EyeronError::new(summary)));
    }
    Ok(eyeron::result_to_string(&doc.prefixes, &result.derived))
}

/// One SHACL validation result retained on a refusal (machine-readable).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Violation {
    pub focus: String,
    pub path: Option<String>,
    pub component: String,
    pub message: String,
    pub severity: String,
}

/// Why a recorded receipt was not accepted by `require_signed`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReceiptReason {
    /// The receipt has no attestation triples.
    Unattested,
    /// The attestation does not verify over the recorded receipt fields.
    BadSignature,
    /// The attestation key is not in the trusted set.
    UntrustedKey,
}

impl ReceiptReason {
    pub fn as_str(self) -> &'static str {
        match self {
            ReceiptReason::Unattested => "unattested",
            ReceiptReason::BadSignature => "bad_signature",
            ReceiptReason::UntrustedKey => "untrusted_key",
        }
    }
}

/// A transition was refused; no child state exists.
#[derive(Debug, Clone)]
pub enum LawError {
    /// The SHACL gate found violations. `violations == results.len()`;
    /// `results` carries the machine-readable SHACL report entries.
    NotAdmitted {
        violations: usize,
        results: Vec<Violation>,
    },
    /// A candidate plan failed replay: `action` (index `index`, or `<goal>`
    /// one past the last action) is missing the listed N-Quads lines.
    PlanRefused {
        index: usize,
        action: String,
        missing: Vec<String>,
    },
    /// No recorded receipt for `step` exists in the state.
    ReceiptRequired { step: String },
    /// A receipt for `step` is recorded but carries no valid attestation by a
    /// trusted key.
    ReceiptRefused { step: String, reason: ReceiptReason },
    /// The lease does not authorize the step.
    LeaseRefused {
        lease_id: String,
        step: String,
        reason: LeaseReason,
    },
    /// An upstream engine or routing refused the input.
    Refused(Refusal),
}

impl std::fmt::Display for LawError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LawError::NotAdmitted { violations, .. } => {
                write!(f, "SHACL admission refused: {violations} violation(s)")
            }
            LawError::PlanRefused {
                index,
                action,
                missing,
            } => write!(
                f,
                "plan refused at step {index} (`{action}`): {} unmet triple(s)",
                missing.len()
            ),
            LawError::ReceiptRequired { step } => {
                write!(f, "receipt required: no recorded receipt for step `{step}`")
            }
            LawError::LeaseRefused {
                lease_id,
                step,
                reason,
            } => write!(
                f,
                "lease `{lease_id}` refused step `{step}`: {}",
                reason.as_str()
            ),
            LawError::ReceiptRefused { step, reason } => {
                write!(f, "receipt for step `{step}` refused: {}", reason.as_str())
            }
            LawError::Refused(r) => write!(f, "{r}"),
        }
    }
}

impl std::error::Error for LawError {}

impl From<Refusal> for LawError {
    fn from(r: Refusal) -> Self {
        LawError::Refused(r)
    }
}

fn unsupported(dialect: Dialect, message: impl Into<String>) -> LawError {
    LawError::Refused(Refusal {
        kind: RefusalKind::Unsupported,
        dialect: Some(dialect),
        engine: Some(dialect.engine()),
        message: message.into(),
    })
}

fn blank_labels(nquads: &str) -> std::collections::BTreeSet<&str> {
    nquads
        .split_whitespace()
        .filter(|t| t.starts_with("_:"))
        .collect()
}

impl LawState {
    /// Wrap a dataset, computing its canonical identity.
    pub fn from_dataset(dataset: Arc<purrdf::RdfDataset>) -> Result<Self, Refusal> {
        let canon = purrdf::try_canonicalize(&dataset).map_err(|e| Refusal {
            kind: RefusalKind::EngineRejected,
            dialect: None,
            engine: Some(crate::dialect::Engine::PurRdf),
            message: format!("canonicalization refused: {e:?}"),
        })?;
        let digest = Sha256::digest(canon.nquads.as_bytes());
        let id = format!(
            "sha256:{}",
            digest
                .iter()
                .map(|b| format!("{b:02x}"))
                .collect::<String>()
        );
        Ok(LawState { dataset, id })
    }

    /// Parse an RDF document of a known dialect.
    pub fn parse(bytes: &[u8], dialect: Dialect, base: Option<&str>) -> Result<Self, Refusal> {
        Self::from_dataset(parse_rdf(bytes, dialect, base)?)
    }

    /// `sha256:` of the RDFC-1.0 canonical N-Quads.
    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn dataset(&self) -> &Arc<purrdf::RdfDataset> {
        &self.dataset
    }

    pub fn quad_count(&self) -> usize {
        self.dataset.quad_count()
    }

    fn nquads(&self) -> Result<String, Refusal> {
        let bytes = purrdf::serialize_dataset(
            self.dataset.as_ref(),
            "application/n-quads",
            purrdf::SerializeGraph::Dataset,
        )
        .map_err(|e| Refusal::engine(Dialect::NQuads, e))?;
        String::from_utf8(bytes).map_err(|e| Refusal::engine(Dialect::NQuads, e))
    }

    /// Apply `step` only if the [`SignedLease`] verifies against `trusted`
    /// keys and authorizes the step at the verifier's `clock` (issuer clock
    /// lead bounded by `max_skew_secs`); the receipt carries the lease id.
    pub fn transition_authorized(
        &self,
        signed: &SignedLease,
        trusted: &crate::attest::TrustedKeys,
        clock: &dyn Clock,
        max_skew_secs: u64,
        step: &Step<'_>,
    ) -> Result<(LawState, Receipt), LawError> {
        signed.authorize(
            step.name(),
            step.required_ceiling(),
            trusted,
            clock,
            max_skew_secs,
        )?;
        let (child, mut receipt) = self.transition(step)?;
        receipt.lease_id = Some(signed.lease.id.clone());
        Ok((child, receipt))
    }

    /// **Unverified**: apply `step` if an *unsigned* `lease` authorizes it at a
    /// caller-supplied `now_unix`. Anyone can construct such a lease and pick
    /// the time, so this proves nothing about authority; it exists for
    /// trusted in-process callers and tests. Use
    /// [`LawState::transition_authorized`] for admission.
    pub fn transition_leased_unverified(
        &self,
        lease: &Lease,
        step: &Step<'_>,
        now_unix: u64,
    ) -> Result<(LawState, Receipt), LawError> {
        lease.authorize(step.name(), step.required_ceiling(), now_unix)?;
        let (child, mut receipt) = self.transition(step)?;
        receipt.lease_id = Some(lease.id.clone());
        Ok((child, receipt))
    }

    /// Apply `step`, returning the child state and a receipt, or a refusal.
    pub fn transition(&self, step: &Step<'_>) -> Result<(LawState, Receipt), LawError> {
        let child = match step {
            Step::AdmitShacl { shapes_ttl } => {
                let shapes = purrdf::shapes::engine::parse_shapes(shapes_ttl, None)
                    .map_err(|e| Refusal::engine(Dialect::Turtle, e))?;
                let report = purrdf::shapes::engine::validate_dataset(&self.dataset, &shapes)
                    .map_err(|e| Refusal::engine(Dialect::Turtle, e))?;
                if !report.conforms {
                    return Err(LawError::NotAdmitted {
                        violations: report.results.len(),
                        results: report
                            .results
                            .iter()
                            .map(|r| Violation {
                                focus: r.focus_node.to_string(),
                                path: r.result_path.as_ref().map(ToString::to_string),
                                component: r.source_constraint_component.to_string(),
                                message: r.message.clone().unwrap_or_default(),
                                severity: format!("{:?}", r.severity),
                            })
                            .collect(),
                    });
                }
                self.clone()
            }
            Step::Hooks { pack } => pack.materialize(self)?.state,
            Step::Plan { plan } => plan.admit(self)?.state,
            Step::RequireReceipt { step } => {
                crate::receipt::require(self, step)?;
                self.clone()
            }
            Step::RequireSignedReceipt { step, trusted } => {
                crate::receipt::require_signed(self, step, trusted)?;
                self.clone()
            }
            Step::EntailRdfs | Step::EntailOwlRl => {
                let plan = if matches!(step, Step::EntailRdfs) {
                    purrdf::entail::Materialization::Rdfs
                } else {
                    purrdf::entail::Materialization::OwlRl
                };
                let (closure, _report) = purrdf::entail::materialize(self.dataset.as_ref(), plan)
                    .map_err(|e| Refusal::engine(Dialect::Turtle, e))?;
                LawState::from_dataset(closure)?
            }
            Step::DeriveN3 { rules } => {
                let base = self.nquads()?;
                if self.dataset.quads().any(|q| q.g.is_some()) {
                    return Err(unsupported(
                        Dialect::N3,
                        "N3 derivation needs a default-graph-only state",
                    ));
                }
                let doc = format!("{base}\n{rules}");
                let derived = reason_n3_bounded(&doc).map_err(|e| match e {
                    N3Error::Refused(r) => r,
                    N3Error::Limit { observed, summary } => Refusal {
                        kind: RefusalKind::ResourceLimit,
                        dialect: Some(Dialect::N3),
                        engine: Some(Dialect::N3.engine()),
                        message: format!(
                            "resource limit `n3_iterations` exceeded: {observed} > {N3_MAX_ITERATIONS} ({summary})"
                        ),
                    },
                })?;
                let derived_ds = purrdf::parse_dataset(derived.as_bytes(), "text/turtle", None)
                    .map_err(|e| Refusal::engine(Dialect::Turtle, e))?;
                let derived_nq = String::from_utf8(
                    purrdf::serialize_dataset(
                        derived_ds.as_ref(),
                        "application/n-quads",
                        purrdf::SerializeGraph::Dataset,
                    )
                    .map_err(|e| Refusal::engine(Dialect::NQuads, e))?,
                )
                .map_err(|e| Refusal::engine(Dialect::NQuads, e))?;
                if blank_labels(&base)
                    .intersection(&blank_labels(&derived_nq))
                    .next()
                    .is_some()
                {
                    return Err(unsupported(
                        Dialect::N3,
                        "blank-node labels collide between the state and derived triples",
                    ));
                }
                let merged = format!("{base}{derived_nq}");
                LawState::parse(merged.as_bytes(), Dialect::NQuads, None)?
            }
        };
        let authority = *BACKEND_AUTHORITIES
            .iter()
            .find(|a| a.capability == step.capability())
            .expect("every step maps to a declared authority");
        let receipt = Receipt {
            parent: self.id.clone(),
            child: child.id.clone(),
            step: step.name(),
            authority,
            added: child.quad_count().saturating_sub(self.quad_count()),
            lease_id: None,
            plan_sha256: None,
            subject_sha256: None,
        };
        Ok((child, receipt))
    }
}
