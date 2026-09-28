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
    /// RDFS entailment closure.
    EntailRdfs,
    /// OWL 2 RL entailment closure.
    EntailOwlRl,
}

impl Step<'_> {
    fn name(&self) -> &'static str {
        match self {
            Step::AdmitShacl { .. } => "admit:shacl",
            Step::DeriveN3 { .. } => "derive:n3",
            Step::EntailRdfs => "derive:rdfs",
            Step::EntailOwlRl => "derive:owl-rl",
        }
    }

    fn capability(&self) -> &'static str {
        match self {
            Step::AdmitShacl { .. } => "SHACL",
            Step::DeriveN3 { .. } => "Notation3",
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
}

/// A transition was refused; no child state exists.
#[derive(Debug, Clone)]
pub enum LawError {
    /// The SHACL gate found violations.
    NotAdmitted { violations: usize },
    /// An upstream engine or routing refused the input.
    Refused(Refusal),
}

impl std::fmt::Display for LawError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LawError::NotAdmitted { violations } => {
                write!(f, "SHACL admission refused: {violations} violation(s)")
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
        };
        Ok((child, receipt))
    }
}
