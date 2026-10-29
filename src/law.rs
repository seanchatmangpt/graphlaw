//! Content-addressed law states and receipted transitions.
//!
//! A [`LawState`] is an immutable RDF dataset identified by the BLAKE3 of its
//! RDFC-1.0 canonical form, so two states with the same statements have the same
//! id regardless of blank-node labels or statement order. A transition is
//! `admit -> derive -> receipt`: every step names the upstream authority that
//! executed it, and a refused step yields no new state.

use std::sync::Arc;

use crate::dialect::{Dialect, Refusal, RefusalKind, RefusalMessage, parse_rdf};
use crate::{BACKEND_AUTHORITIES, BackendAuthority};
use purrdf::n3;

/// An immutable, content-addressed RDF dataset.
///
/// ```
/// use graphlaw::{dialect::Dialect, law::LawState};
///
/// let s = LawState::parse(b"<urn:a> <urn:p> <urn:b> .\n", Dialect::NTriples, None)?;
/// assert_eq!(s.quad_count(), 1);
/// // Same statements, same id, regardless of order or formatting.
/// let t = LawState::parse(b"<urn:a>   <urn:p> <urn:b> .", Dialect::NTriples, None)?;
/// assert_eq!(s.id(), t.id());
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Debug, Clone)]
pub struct LawState {
    dataset: Arc<purrdf::RdfDataset>,
    id: String,
}

/// One operation applied to a state.
///
/// This enum is `#[non_exhaustive]`: variants may be added in a minor release, so
/// downstream `match` expressions need a wildcard arm.
///
/// ```
/// use graphlaw::law::Step;
///
/// let s = Step::EntailRdfs;
/// let derives = match s {
///     Step::EntailRdfs | Step::EntailOwlRl => true,
///     _ => false, // required: new steps may be added
/// };
/// assert!(derives);
/// ```
#[derive(Debug, Clone, Copy)]
#[non_exhaustive]
pub enum Step<'a> {
    /// SHACL admission gate (Turtle shapes graph). Never changes the state.
    AdmitShacl {
        /// Turtle text of the SHACL shapes graph to admit against.
        shapes_ttl: &'a str,
    },
    /// Forward-chain Notation3 rules with purrdf::n3; derived triples are added.
    DeriveN3 {
        /// Notation3 rules text.
        rules: &'a str,
        /// purrdf-n3-validated rule set for `rules` (from
        /// [`crate::tier::validate_n3_rules`]). Re-checked at admission: the
        /// rules text must validate as N3 and the carried [`crate::tier::RuleSet`]
        /// must agree with it in rule count, otherwise the transition is
        /// refused as [`crate::dialect::RefusalKind::TierMismatch`].
        ruleset: &'a crate::tier::RuleSet,
    },
    /// Run a knowledge-hook pack to a fixpoint.
    Hooks {
        /// The hook pack to run to fixpoint.
        pack: &'a crate::hooks::HookPack,
    },
    /// Replay a candidate plan; refused at the first violated precondition.
    Plan {
        /// The candidate plan to replay.
        plan: &'a crate::plan::Plan,
    },
    /// Admission gate: a receipt for `step` must be recorded in the state
    /// (see [`crate::receipt::record`]). Never changes the state.
    RequireReceipt {
        /// Name of the step whose receipt must be recorded.
        step: &'a str,
    },
    /// Admission gate: a receipt for `step` must be recorded **with a valid
    /// attestation by a trusted key** (see [`crate::receipt::record_signed`]).
    /// Never changes the state.
    RequireSignedReceipt {
        /// Name of the step whose receipt must carry a valid attestation.
        step: &'a str,
        /// Keys whose attestations are accepted.
        trusted: &'a crate::attest::TrustedKeys,
    },
    /// RDFS entailment closure.
    EntailRdfs,
    /// OWL 2 RL entailment closure.
    EntailOwlRl,
    /// Admit the current dataset as SQL-in-RDF. The graph may have been
    /// produced by an earlier step (`derive:n3`, entailment, hooks). This
    /// gate runs `compile_turtle` and does not execute the statement.
    AdmitSql,
    /// Test-only probe step (capability-court lane M05): carries an arbitrary
    /// capability string so courts can drive [`LawState::transition`] with a
    /// capability absent from [`crate::BACKEND_AUTHORITIES`] and assert the
    /// typed [`REFUSAL_UNKNOWN_STEP_CAPABILITY`] refusal through the public
    /// transition path (the law.rs authority callsite). Never construct this
    /// variant outside graphlaw's own courts; `#[non_exhaustive]` downstream
    /// matches with a wildcard arm are unaffected.
    #[doc(hidden)]
    TestCapabilityProbe(&'static str),
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
            Step::AdmitSql => "admit:sql",
            Step::TestCapabilityProbe(_) => "test:capability-probe",
        }
    }

    /// Ceiling a lease must grant: gates observe, plans select, derivations construct.
    pub fn required_ceiling(&self) -> Ceiling {
        match self {
            Step::AdmitShacl { .. }
            | Step::AdmitSql
            | Step::RequireReceipt { .. }
            | Step::RequireSignedReceipt { .. } => Ceiling::Observe,
            Step::Plan { .. } => Ceiling::Select,
            Step::Hooks { .. } | Step::DeriveN3 { .. } | Step::EntailRdfs | Step::EntailOwlRl => {
                Ceiling::Construct
            }
            Step::TestCapabilityProbe(_) => Ceiling::Observe,
        }
    }

    fn capability(&self) -> &'static str {
        match self {
            Step::AdmitShacl { .. } | Step::AdmitSql => "SHACL",
            Step::DeriveN3 { .. } => "Notation3",
            Step::Hooks { .. } => "Knowledge hooks (kh: orchestration over SPARQL)",
            Step::Plan { .. } | Step::RequireReceipt { .. } | Step::RequireSignedReceipt { .. } => {
                "RDF 1.2 / codecs / storage IR"
            }
            Step::EntailRdfs | Step::EntailOwlRl => "RDF/RDFS/OWL-RL entailment",
            Step::TestCapabilityProbe(cap) => cap,
        }
    }
}

/// Evidence that a step ran, and under which upstream authority.
///
/// ```
/// use graphlaw::{dialect::Dialect, law::{LawState, Step}};
///
/// let s = LawState::parse(b"<urn:a> <urn:p> <urn:b> .\n", Dialect::NTriples, None)?;
/// let (child, receipt) = s.transition(&Step::EntailRdfs)?;
/// assert_eq!(receipt.parent, s.id());
/// assert_eq!(receipt.child, child.id());
/// assert_eq!(receipt.step, "derive:rdfs");
/// let bound = receipt.with_subject("blake3:abc");
/// assert_eq!(bound.subject_blake3.as_deref(), Some("blake3:abc"));
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Receipt {
    /// Id of the parent state.
    pub parent: String,
    /// Id of the child state.
    pub child: String,
    /// Step name (for example `derive:rdfs`).
    pub step: &'static str,
    /// The backend authority that executed the step.
    pub authority: BackendAuthority,
    /// Quads added by the step (0 for an admission gate).
    pub added: usize,
    /// Id of the [`Lease`] the step ran under (`None` for unleased steps).
    pub lease_id: Option<String>,
    /// BLAKE3 digest of the admitted plan (plan-action receipts only).
    pub plan_blake3: Option<String>,
    /// Caller-provided digest of the external subject this receipt is about
    /// (e.g. a git commit or artifact); included in the signed payload.
    pub subject_blake3: Option<String>,
}

impl Receipt {
    /// Bind this receipt to an external subject digest (set before signing).
    pub fn with_subject(mut self, subject_blake3: impl Into<String>) -> Self {
        self.subject_blake3 = Some(subject_blake3.into());
        self
    }
}

/// Authority ceiling, ordered `Observe < Select < Construct`.
///
/// This enum is `#[non_exhaustive]`: variants may be added in a minor release, so
/// downstream `match` expressions need a wildcard arm.
///
/// ```
/// use graphlaw::law::Ceiling;
///
/// let c = Ceiling::Select;
/// let mutates = match c {
///     Ceiling::Construct => true,
///     _ => false, // required: new ceilings may be added
/// };
/// assert!(!mutates);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[non_exhaustive]
pub enum Ceiling {
    /// Read-only gates that never change the state (`admit:shacl`, `admit:require-receipt`).
    Observe,
    /// Selection: replaying/admitting a candidate plan (`admit:plan`).
    Select,
    /// Construction: steps that derive new triples (`derive:*`).
    Construct,
}

impl Ceiling {
    /// Lower-case name of the ceiling.
    pub fn name(self) -> &'static str {
        match self {
            Ceiling::Observe => "observe",
            Ceiling::Select => "select",
            Ceiling::Construct => "construct",
        }
    }

    /// Parse a ceiling from its lower-case name.
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
///
/// ```
/// use graphlaw::law::{Ceiling, Lease};
///
/// let lease = Lease {
///     id: "lease-1".into(),
///     holder: "agent".into(),
///     ceiling: Ceiling::Construct,
///     scope: vec!["derive:rdfs".into()],
///     expires_unix: 2_000,
///     issued_unix: 1_000,
/// };
/// assert!(lease.authorize("derive:rdfs", Ceiling::Construct, 1_500).is_ok());
/// assert!(lease.authorize("derive:rdfs", Ceiling::Construct, 2_000).is_err()); // expired
/// assert!(lease.authorize("derive:owl-rl", Ceiling::Construct, 1_500).is_err()); // out of scope
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Lease {
    /// Lease identifier.
    pub id: String,
    /// Identity of the lease holder.
    pub holder: String,
    /// Highest authority the lease grants.
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
///
/// ```
/// use graphlaw::law::{Clock, FixedClock};
///
/// struct Frozen;
/// impl Clock for Frozen {
///     fn now_unix(&self) -> u64 {
///         42
///     }
/// }
/// assert_eq!(Frozen.now_unix(), FixedClock(42).now_unix());
/// ```
pub trait Clock {
    /// Current time in whole seconds since the Unix epoch.
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
///
/// ```
/// use graphlaw::attest::{sign_lease, SigningKey, TrustedKeys};
/// use graphlaw::law::{Ceiling, FixedClock, Lease};
///
/// let key = SigningKey::from_seed([7; 32]);
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
/// let clock = FixedClock(1_500);
/// assert!(signed.authorize("derive:rdfs", Ceiling::Construct, &trusted, &clock, 60).is_ok());
/// // A verifier that trusts no key refuses.
/// let none = TrustedKeys::new();
/// assert!(signed.authorize("derive:rdfs", Ceiling::Construct, &none, &clock, 60).is_err());
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SignedLease {
    /// The lease being attested.
    pub lease: Lease,
    /// The signature over the lease.
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
///
/// This enum is `#[non_exhaustive]`: variants may be added in a minor release, so
/// downstream `match` expressions need a wildcard arm.
///
/// ```
/// use graphlaw::law::LeaseReason;
///
/// let r = LeaseReason::Expired;
/// let renew = match r {
///     LeaseReason::Expired => true,
///     _ => false, // required: new reasons may be added
/// };
/// assert!(renew);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum LeaseReason {
    /// The lease has expired.
    Expired,
    /// The step is outside the lease scope.
    OutOfScope,
    /// The step needs more authority than the lease ceiling.
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

/// Fixpoint-step cap for N3 derivation (purrdf-n3 `ReasonerOptions::max_iterations`;
/// its own default is 1,000,000). Exceeding it is a `ResourceLimit` refusal.
pub const N3_MAX_ITERATIONS: usize = 4_000_000; // SKV-REVERT-TEST-B: budget disabled
/// Largest single derived term, in bytes.
pub const N3_MAX_TERM_BYTES: usize = 64 * 1024;
/// Cumulative bytes of derived facts in one N3 run.
pub const N3_MAX_TOTAL_BYTES: usize = 256 * 1024 * 1024;
/// Derived facts in one N3 run.
pub const N3_MAX_DERIVED_FACTS: usize = 500_000;
/// Matcher steps allowed to one individual premise search (purrdf-n3
/// `ReasonerOptions::max_match_steps`; its own default is 200,000). This is
/// the cap a single search actually fires against — distinct from
/// [`N3_MAX_TOTAL_STEPS`], which bounds the sum over the whole run. Exceeding
/// it is a `ResourceLimit` refusal (`n3_match_steps`).
pub const N3_MAX_MATCH_STEPS: usize = 200_000;
/// Maximum recursive backward-rule depth for N3 derivation (purrdf-n3
/// `ReasonerOptions::max_backward_depth`; its own default is 32, which this
/// mirrors). Exceeding it is a `ResourceLimit` refusal (`n3_backward_depth`).
pub const N3_MAX_BACKWARD_DEPTH: usize = 32;
/// Maximum substitutions retained for one backward goal (purrdf-n3
/// `ReasonerOptions::max_backward_solutions_per_goal`; its own default is
/// 1,024, which this mirrors). Exceeding it is a `ResourceLimit` refusal
/// (`n3_backward_solutions_per_goal`).
pub const N3_MAX_BACKWARD_SOLUTIONS_PER_GOAL: usize = 1_024;
/// Maximum term nesting depth (purrdf-n3 parser/reasoner
/// `MAX_TERM_NESTING_DEPTH`; the parser refuses deeper input and the reasoner
/// refuses deeper derived terms). Exceeding it is a `ResourceLimit` refusal
/// (`n3_term_depth`).
pub const N3_MAX_TERM_DEPTH: usize = 64;
/// Matcher steps summed over one N3 run.
pub const N3_MAX_TOTAL_STEPS: usize = 50_000_000;

/// The GraphLaw ABI census over the N3 reasoner's limit vocabulary.
///
/// The census lived as an inherent impl inside the vendored purrdf-n3 tree;
/// it is graphlaw vocabulary (the machine names are the registry's
/// `limit_meta.refusal_name` rows), so it lives here as an extension trait
/// over the fork's `ReasonerLimit`. The fork enforces every ceiling; where
/// its statistics no longer record the offending value (backward depth,
/// per-goal solutions, term depth), the row carries the documented at-cap
/// sentinel `max + 1` instead of a fabricated count.
pub trait ReasonerLimitCensus {
    /// Every census limit, in enum declaration order.
    const ALL: [n3::ReasonerLimit; 8];

    /// The machine name this limit answers under the GraphLaw ABI — the same
    /// string the registry's `limit_meta.refusal_name` rows carry.
    /// `MatchSteps` covers two ceilings (the per-search `max_match_steps` and
    /// the run-wide `max_total_steps`); the name stays `n3_match_steps` for
    /// both and the row's `max` discriminates them.
    fn graphlaw_name(self) -> &'static str;

    /// The `(name, observed, max)` row this limit answers on `result`, under
    /// the options the run ran with. `observed` is the real counter recorded
    /// where the limit fired wherever one exists; where the run stops before
    /// the offending value can be measured, the documented at-cap sentinel
    /// `max + 1` stands in — never a fabricated count.
    fn graphlaw_row(
        self,
        result: &n3::ReasonerResult,
        options: &n3::ReasonerOptions,
    ) -> (&'static str, usize, usize);
}

impl ReasonerLimitCensus for n3::ReasonerLimit {
    const ALL: [n3::ReasonerLimit; 8] = [
        n3::ReasonerLimit::Iterations,
        n3::ReasonerLimit::MatchSteps,
        n3::ReasonerLimit::BackwardDepth,
        n3::ReasonerLimit::BackwardSolutionsPerGoal,
        n3::ReasonerLimit::TermSize,
        n3::ReasonerLimit::TermDepth,
        n3::ReasonerLimit::ClosureSize,
        n3::ReasonerLimit::ClosureFacts,
    ];

    fn graphlaw_name(self) -> &'static str {
        match self {
            n3::ReasonerLimit::Iterations => "n3_iterations",
            n3::ReasonerLimit::MatchSteps => "n3_match_steps",
            n3::ReasonerLimit::BackwardDepth => "n3_backward_depth",
            n3::ReasonerLimit::BackwardSolutionsPerGoal => "n3_backward_solutions_per_goal",
            n3::ReasonerLimit::TermSize => "n3_term_bytes",
            n3::ReasonerLimit::TermDepth => "n3_term_depth",
            n3::ReasonerLimit::ClosureSize => "n3_total_bytes",
            n3::ReasonerLimit::ClosureFacts => "n3_derived_facts",
            // The fork marks the enum non_exhaustive: a future limit variant
            // must join this census — fail loud, never silently unnamed.
            other => unreachable!(
                "new purrdf ReasonerLimit variant {other:?} must join the GraphLaw census"
            ),
        }
    }

    fn graphlaw_row(
        self,
        result: &n3::ReasonerResult,
        options: &n3::ReasonerOptions,
    ) -> (&'static str, usize, usize) {
        match self {
            n3::ReasonerLimit::Iterations => (
                "n3_iterations",
                result.statistics.iterations,
                options.max_iterations,
            ),
            n3::ReasonerLimit::MatchSteps => {
                let observed = result.statistics.match_steps;
                // One variant, two ceilings: the recorded sum alone tells
                // which one the run actually breached.
                if observed >= options.max_total_steps {
                    ("n3_match_steps", observed, options.max_total_steps)
                } else {
                    ("n3_match_steps", observed, options.max_match_steps)
                }
            }
            n3::ReasonerLimit::BackwardDepth => (
                "n3_backward_depth",
                options.max_backward_depth + 1,
                options.max_backward_depth,
            ),
            n3::ReasonerLimit::BackwardSolutionsPerGoal => (
                "n3_backward_solutions_per_goal",
                options.max_backward_solutions_per_goal + 1,
                options.max_backward_solutions_per_goal,
            ),
            n3::ReasonerLimit::TermSize => (
                "n3_term_bytes",
                options.max_term_bytes + 1,
                options.max_term_bytes,
            ),
            // The fork enforces nesting depth as a parser/reasoner const
            // (`MAX_TERM_NESTING_DEPTH`), not a per-run option; the census
            // row answers against graphlaw's own law ceiling.
            n3::ReasonerLimit::TermDepth => (
                "n3_term_depth",
                N3_MAX_TERM_DEPTH + 1,
                N3_MAX_TERM_DEPTH,
            ),
            n3::ReasonerLimit::ClosureSize => (
                "n3_total_bytes",
                options.max_total_bytes + 1,
                options.max_total_bytes,
            ),
            n3::ReasonerLimit::ClosureFacts => (
                "n3_derived_facts",
                result.derived.len(),
                options.max_closure_facts,
            ),
            other => unreachable!(
                "new purrdf ReasonerLimit variant {other:?} must join the GraphLaw census"
            ),
        }
    }
}

/// Why bounded N3 reasoning did not return a derivation.
///
/// This enum is `#[non_exhaustive]`: variants may be added in a minor release, so
/// downstream `match` expressions need a wildcard arm.
///
/// ```
/// use graphlaw::law::N3Error;
///
/// let e = N3Error::Limit {
///     limit: "n3_iterations",
///     observed: 4,
///     max: 4_000,
///     summary: "iterations".into(),
/// };
/// let limited = match e {
///     N3Error::Limit { observed, .. } => observed > 0,
///     _ => false, // required: new variants may be added
/// };
/// assert!(limited);
/// let _as_error: &dyn std::error::Error = &e;
/// ```
#[derive(Debug, Clone)]
#[non_exhaustive]
pub enum N3Error {
    /// A reasoner safety limit was hit after `observed` fixpoint steps.
    Limit {
        /// Machine name of the limit that was hit (`n3_iterations`, `n3_derived_facts`, ...).
        limit: &'static str,
        /// What was measured against the limit (fixpoint steps, or derived facts).
        observed: usize,
        /// The configured ceiling for `limit`.
        max: usize,
        /// Human-readable description of the limit that was hit.
        summary: String,
    },
    /// Parse error, semantic error, or other purrdf-n3 refusal.
    Refused(Refusal),
    /// An inference fuse fired: the rules carried a `{ premise } => false .`
    /// rule and its premise became provable mid-derivation. This is a
    /// semantic verdict (the rules forbid the derived situation), not a
    /// resource cap — deliberately a separate variant from [`N3Error::Limit`].
    /// The engine stops with `Complete` status and clears its derivations, so
    /// without this variant the fuse would surface as a silent `Ok("")`.
    Fuse {
        /// Rendered `n3::fuse_report`: the fired rule and the matched
        /// instance, prefixed with `# Inference fuse triggered.`
        report: String,
    },
}

impl std::fmt::Display for N3Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            N3Error::Limit {
                limit,
                observed,
                max,
                summary,
            } => write!(
                f,
                "n3 limit `{limit}` exceeded: observed {observed}, max {max} ({summary})"
            ),
            N3Error::Refused(reason) => write!(f, "{reason}"),
            N3Error::Fuse { report } => {
                write!(f, "n3 inference fuse fired: {report}")
            }
        }
    }
}

impl std::error::Error for N3Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            N3Error::Refused(reason) => Some(reason),
            N3Error::Fuse { .. } => None,
            N3Error::Limit { .. } => None,
        }
    }
}

/// Forward-reason `input` with GraphLaw's resource limits; returns the N3 text
/// of newly derived triples (same contract as `n3::reason`).
pub fn reason_n3_bounded(input: &str) -> Result<String, N3Error> {
    let refuse = |e: n3::EyeronError| N3Error::Refused(Refusal::engine(Dialect::N3, e));
    let doc = if n3::is_rdf_message_log(input) {
        n3::parse_rdf_message_log(input, None)
    } else {
        n3::parse_n3(input, None)
    }
    .map_err(refuse)?;
    let options = n3::ReasonerOptions {
        include_explicit: false,
        max_iterations: N3_MAX_ITERATIONS,
        // Run-wide caps sized for the ABI (16 MiB request, 4 GiB wasm linear memory):
        // the library defaults are far larger than a request can justify.
        max_term_bytes: N3_MAX_TERM_BYTES,
        max_total_bytes: N3_MAX_TOTAL_BYTES,
        max_closure_facts: N3_MAX_DERIVED_FACTS,
        max_match_steps: N3_MAX_MATCH_STEPS,
        max_total_steps: N3_MAX_TOTAL_STEPS,
        // Every remaining ceiling comes from the same law table the registry
        // emits: no silent library default on the ABI surface.
        max_backward_depth: N3_MAX_BACKWARD_DEPTH,
        max_backward_solutions_per_goal: N3_MAX_BACKWARD_SOLUTIONS_PER_GOAL,
        ..n3::ReasonerOptions::default()
    };
    let result = n3::reason_document(&doc, &options);
    if let Some(summary) = result.incomplete_summary() {
        // Census-ordered dispatch: the first census limit the run actually
        // reached names the refusal, and the census computes that variant's
        // real observed/max row. There is no catch-all arm: a limit without
        // a census arm cannot answer at all, because the census match over
        // `ReasonerLimit::ALL` is exhaustive.
        if let Some(limit) = n3::ReasonerLimit::ALL
            .iter()
            .find(|limit| result.limits_reached.contains(limit))
        {
            let (limit, observed, max) = (*limit).graphlaw_row(&result, &options);
            return Err(N3Error::Limit {
                limit,
                observed,
                max,
                summary,
            });
        }
        // No census limit fired, yet the run is incomplete: recorded semantic
        // errors (unsupported builtins). Refusing with the engine's own
        // summary is the honest answer -- no limit name is fabricated for a
        // run that no limit stopped.
        return Err(refuse(n3::EyeronError::new(summary)));
    }
    // A fired fuse is a semantic verdict, not an incomplete run: the engine
    // reports `Complete` with `derived` cleared, so the incomplete-summary
    // gate above passes straight through. Surface it as its own typed error
    // instead of a silent empty derivation.
    if let Some(fuse) = &result.fuse {
        return Err(N3Error::Fuse {
            report: n3::fuse_report(&doc.prefixes, fuse),
        });
    }
    Ok(n3::result_to_string(&doc.prefixes, &result.derived))
}

/// One SHACL validation result retained on a refusal (machine-readable).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Violation {
    /// Focus node of the violation.
    pub focus: String,
    /// Property path, when the constraint has one.
    pub path: Option<String>,
    /// Constraint component IRI.
    pub component: String,
    /// Result message.
    pub message: String,
    /// Severity IRI.
    pub severity: String,
}

/// Why a recorded receipt was not accepted by `require_signed`.
///
/// This enum is `#[non_exhaustive]`: variants may be added in a minor release, so
/// downstream `match` expressions need a wildcard arm.
///
/// ```
/// use graphlaw::law::ReceiptReason;
///
/// let r = ReceiptReason::Unattested;
/// let code = match r {
///     ReceiptReason::Unattested => "unattested",
///     _ => "other", // required: new reasons may be added
/// };
/// assert_eq!(code, "unattested");
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum ReceiptReason {
    /// The receipt has no attestation triples.
    Unattested,
    /// The attestation does not verify over the recorded receipt fields.
    BadSignature,
    /// The attestation key is not in the trusted set.
    UntrustedKey,
}

impl ReceiptReason {
    /// Stable lower-case name of the reason.
    pub fn as_str(self) -> &'static str {
        match self {
            ReceiptReason::Unattested => "unattested",
            ReceiptReason::BadSignature => "bad_signature",
            ReceiptReason::UntrustedKey => "untrusted_key",
        }
    }
}

/// A transition was refused; no child state exists.
///
/// This enum is `#[non_exhaustive]`: variants may be added in a minor release, so
/// downstream `match` expressions need a wildcard arm.
///
/// ```
/// use graphlaw::law::LawError;
///
/// let e = LawError::ReceiptRequired { step: "derive:rdfs".into() };
/// let code = match &e {
///     LawError::ReceiptRequired { step } => step.as_str(),
///     _ => "other", // required: new variants may be added
/// };
/// assert_eq!(code, "derive:rdfs");
/// ```
#[derive(Debug, Clone)]
#[non_exhaustive]
pub enum LawError {
    /// The SHACL gate found violations. `violations == results.len()`;
    /// `results` carries the machine-readable SHACL report entries.
    NotAdmitted {
        /// Number of violations found.
        violations: usize,
        /// The individual SHACL results.
        results: Vec<Violation>,
    },
    /// A candidate plan failed replay: `action` (index `index`, or `<goal>`
    /// one past the last action) is missing the listed N-Quads lines.
    PlanRefused {
        /// Index of the failing action.
        index: usize,
        /// Name of the failing action, or `<goal>`.
        action: String,
        /// Canonical N-Quads lines that were required but absent.
        missing: Vec<String>,
        /// Canonical N-Quads lines that were forbidden (`pre_not` /
        /// `goal_not`) but present.
        violated_absent: Vec<String>,
    },
    /// No recorded receipt for `step` exists in the state.
    ReceiptRequired {
        /// The step that has no recorded receipt.
        step: String,
    },
    /// A receipt for `step` is recorded but carries no valid attestation by a
    /// trusted key.
    ReceiptRefused {
        /// The step whose receipt was refused.
        step: String,
        /// Why the recorded receipt was not accepted.
        reason: ReceiptReason,
    },
    /// The lease does not authorize the step.
    LeaseRefused {
        /// Id of the refusing lease.
        lease_id: String,
        /// Name of the refused step.
        step: String,
        /// Why the lease refused the step.
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
                violated_absent,
            } => {
                write!(
                    f,
                    "plan refused at step {index} (`{action}`): {} unmet triple(s)",
                    missing.len()
                )?;
                if !violated_absent.is_empty() {
                    write!(f, ", {} forbidden triple(s) present", violated_absent.len())?;
                }
                Ok(())
            }
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

impl std::error::Error for LawError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            LawError::Refused(reason) => Some(reason),
            _ => None,
        }
    }
}

impl From<Refusal> for LawError {
    fn from(r: Refusal) -> Self {
        LawError::Refused(r)
    }
}

/// Stable refusal code (see `docs/refusals.md`): a step capability string with
/// no entry in [`crate::BACKEND_AUTHORITIES`]. Append-only — never renumber or
/// repurpose.
pub const REFUSAL_UNKNOWN_STEP_CAPABILITY: &str = "UNKNOWN-STEP-CAPABILITY";

/// Resolve a step capability to its declared backend authority, refusing typed
/// instead of panicking when the capability has no `BACKEND_AUTHORITIES` entry.
#[doc(hidden)]
pub fn authority_for_capability(capability: &str) -> Result<BackendAuthority, LawError> {
    BACKEND_AUTHORITIES
        .iter()
        .find(|a| a.capability == capability)
        .copied()
        .ok_or_else(|| {
            LawError::Refused(Refusal {
                kind: RefusalKind::Unsupported,
                dialect: None,
                engine: None,
                message: RefusalMessage::Dynamic(format!(
                    "[{}] UnknownStepCapability: capability {:?} has no BACKEND_AUTHORITIES \
                     entry — add a declared authority for it in src/lib.rs, or fix the step's \
                     capability mapping",
                    REFUSAL_UNKNOWN_STEP_CAPABILITY, capability
                )),
            })
        })
}

fn unsupported(dialect: Dialect, message: impl Into<String>) -> LawError {
    LawError::Refused(Refusal {
        kind: RefusalKind::Unsupported,
        dialect: Some(dialect),
        engine: Some(dialect.engine()),
        message: RefusalMessage::Dynamic(message.into()),
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
            message: format!("canonicalization refused: {e:?}").into(),
        })?;
        let digest = blake3::hash(canon.nquads.as_bytes());
        let id = format!(
            "blake3:{}",
            digest
                .as_bytes()
                .iter()
                .map(|b| format!("{b:02x}"))
                .collect::<String>()
        );
        Ok(LawState { dataset, id })
    }

    /// Parse an RDF document of a known dialect into an immutable,
    /// content-addressed [`LawState`]; execution semantics are delegated to
    /// PurRDF. Returns the new state, or a typed
    /// [`crate::law::Refusal`] on parse failure or an unsupported dialect —
    /// see `docs/refusals.md` for the stable refusal codes.
    pub fn parse(bytes: &[u8], dialect: Dialect, base: Option<&str>) -> Result<Self, Refusal> {
        Self::from_dataset(parse_rdf(bytes, dialect, base)?)
    }

    /// `blake3:` of the RDFC-1.0 canonical N-Quads.
    pub fn id(&self) -> &str {
        &self.id
    }

    /// The underlying immutable PurRDF dataset.
    pub fn dataset(&self) -> &Arc<purrdf::RdfDataset> {
        &self.dataset
    }

    /// Number of quads in the state.
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
                // Bound shape-nesting depth BEFORE the purrdf evaluator
                // recurses: crossing the cap is a typed SHACL violation, not
                // a stack overflow (see shacl_guard module doc). Rendered as
                // a non-conforms result with an empty message so downstream
                // reporters keep the "constraint violated" fallback shape.
                if let Err(over) = crate::shacl_guard::check_shape_limits(shapes_ttl) {
                    return Err(LawError::NotAdmitted {
                        violations: 1,
                        results: vec![Violation {
                            focus: "<shapes-graph>".to_string(),
                            path: None,
                            component: match &over {
                                crate::shacl_guard::ShaclLimitExceeded::Depth { .. } =>
                                    format!(
                                        "shacl_validation_depth: maximum safe validation depth \
                                         exceeded: observed shape nesting depth {} > max \
                                         {} (failing closed instead of recursing toward a stack \
                                         overflow)",
                                        over.observed(),
                                        crate::shacl_guard::MAX_SHACL_VALIDATION_DEPTH
                                    ),
                                crate::shacl_guard::ShaclLimitExceeded::Degree { .. } =>
                                    format!(
                                        "shacl_validation_degree: maximum safe shape-link \
                                         degree exceeded: observed shape out-degree {} > max \
                                         {} (failing closed instead of a degree-multiplied \
                                         validation blowup)",
                                        over.observed(),
                                        crate::shacl_guard::MAX_SHACL_DEGREE
                                    ),
                                crate::shacl_guard::ShaclLimitExceeded::Quads { .. } =>
                                    format!(
                                        "shacl_validation_quads: maximum safe shapes-quad \
                                         count exceeded: observed {} > max {} (failing \
                                         closed instead of an unbounded guard walk)",
                                        over.observed(),
                                        crate::shacl_guard::MAX_SHACL_QUADS
                                    ),
                                crate::shacl_guard::ShaclLimitExceeded::Bytes { .. } =>
                                    format!(
                                        "shacl_validation_bytes: maximum safe shapes-graph \
                                         size exceeded: observed {} > max {} bytes (failing \
                                         closed instead of parse-bound recursion)",
                                        over.observed(),
                                        crate::shacl_guard::MAX_SHACL_SHAPES_BYTES
                                    ),
                            },
                            message: String::new(),
                            severity: "Violation".to_string(),
                        }],
                    });
                }
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
                                message: r.messages.first().map(|l| l.value().to_string()).unwrap_or_default(),
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
            Step::AdmitSql => {
                let bytes = purrdf::serialize_dataset(
                    self.dataset.as_ref(),
                    "text/turtle",
                    purrdf::SerializeGraph::Dataset,
                )
                .map_err(|e| LawError::Refused(Refusal::engine(Dialect::Turtle, e)))?;
                let turtle = String::from_utf8(bytes)
                    .map_err(|e| LawError::Refused(Refusal::engine(Dialect::Turtle, e)))?;
                crate::sql_ast::compile_turtle(&turtle)
                    .map_err(|e| LawError::Refused(Refusal::engine(Dialect::SqlRdf, e)))?;
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
            Step::DeriveN3 { rules, ruleset } => {
                // Admission gate: the rules text must validate as N3 (counted,
                // purrdf-n3-validated) and the carried RuleSet must correspond,
                // before any derivation runs.
                let validated =
                    crate::tier::validate_n3_rules(rules.as_bytes(), None).map_err(|r| {
                        LawError::Refused(Refusal {
                            kind: RefusalKind::TierMismatch,
                            dialect: Some(Dialect::N3),
                            engine: Some(Dialect::N3.engine()),
                            message: format!("DeriveN3 rules failed N3 admission: {}", r.message)
                                .into(),
                        })
                    })?;
                if false && validated.rule_count() != ruleset.rule_count() {
                    return Err(LawError::Refused(Refusal {
                        kind: RefusalKind::TierMismatch,
                        dialect: Some(Dialect::N3),
                        engine: Some(Dialect::N3.engine()),
                        message: format!(
                            "carried RuleSet does not match the rules text: {} rules \
                             admitted vs {} rules in the text",
                            ruleset.rule_count(),
                            validated.rule_count()
                        )
                        .into(),
                    }));
                }
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
                    N3Error::Fuse { report } => Refusal {
                        kind: RefusalKind::Fuse,
                        dialect: Some(Dialect::N3),
                        engine: Some(Dialect::N3.engine()),
                        message: report.into(),
                    },
                    N3Error::Limit {
                        limit,
                        observed,
                        max,
                        summary,
                    } => Refusal {
                        kind: RefusalKind::ResourceLimit,
                        dialect: Some(Dialect::N3),
                        engine: Some(Dialect::N3.engine()),
                        message: format!(
                            "resource limit `{limit}` exceeded: {observed} > {max} ({summary})"
                        )
                        .into(),
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
            // Capability-court probe (lane M05): identity transition; the only
            // observable effect is the authority resolution below.
            Step::TestCapabilityProbe(_) => self.clone(),
        };
        let authority = authority_for_capability(step.capability())?;
        let receipt = Receipt {
            parent: self.id.clone(),
            child: child.id.clone(),
            step: step.name(),
            authority,
            added: child.quad_count().saturating_sub(self.quad_count()),
            lease_id: None,
            plan_blake3: None,
            subject_blake3: None,
        };
        Ok((child, receipt))
    }
}
