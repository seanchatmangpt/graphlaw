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
            Step::EntailRdfs => "derive:rdfs",
            Step::EntailOwlRl => "derive:owl-rl",
        }
    }

    /// Ceiling a lease must grant: gates observe, plans select, derivations construct.
    pub fn required_ceiling(&self) -> Ceiling {
        match self {
            Step::AdmitShacl { .. } | Step::RequireReceipt { .. } => Ceiling::Observe,
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
            Step::Plan { .. } | Step::RequireReceipt { .. } => "RDF 1.2 / codecs / storage IR",
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
}

/// Why a lease refused a step.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LeaseReason {
    Expired,
    OutOfScope,
    Ceiling,
}

impl Lease {
    /// Check a step (by name and required ceiling) against this lease.
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

/// A transition was refused; no child state exists.
#[derive(Debug, Clone)]
pub enum LawError {
    /// The SHACL gate found violations.
    NotAdmitted { violations: usize },
    /// A candidate plan failed replay: `action` (index `index`, or `<goal>`
    /// one past the last action) is missing the listed N-Quads lines.
    PlanRefused {
        index: usize,
        action: String,
        missing: Vec<String>,
    },
    /// No recorded receipt for `step` exists in the state.
    ReceiptRequired { step: String },
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
            LawError::NotAdmitted { violations } => {
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
                match reason {
                    LeaseReason::Expired => "expired",
                    LeaseReason::OutOfScope => "out_of_scope",
                    LeaseReason::Ceiling => "ceiling",
                }
            ),
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

    /// Apply `step` only if `lease` authorizes it at `now_unix`; the receipt
    /// carries the lease id. Refused (typed) when expired, out of scope, or over ceiling.
    pub fn transition_leased(
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
                let derived = eyeron::reason(&doc).map_err(|e| Refusal::engine(Dialect::N3, e))?;
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
        };
        Ok((child, receipt))
    }
}
