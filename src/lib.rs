//! GraphLaw is a standards-composition boundary, not a standards implementation.
//!
//! Since v26.9.28, GraphLaw delegates executable RDF semantics to maintained
//! upstream Rust libraries:
//!
//! - PurRDF owns RDF 1.2, SPARQL 1.1/1.2, SHACL, ShEx 2.1, Datalog,
//!   RDF/RDFS/OWL-RL entailment, and RDF event ingestion.
//! - Eyeron owns Notation3 parsing and reasoning.
//!
//! GraphLaw intentionally contains no fallback parser, rule evaluator, triplestore,
//! SHACL/ShEx validator, SPARQL evaluator, or OWL ruleset of its own. Missing
//! capabilities are refused or added upstream instead of being reimplemented here.

#![forbid(unsafe_code)]

#[cfg(feature = "abi")]
pub mod abi;
pub mod dialect;
pub mod hooks;
pub mod law;
#[cfg(feature = "pack-tools")]
pub mod smon;

/// Unstable escape hatch: the whole PurRDF crate. Not covered by GraphLaw's own
/// API stability; prefer the curated modules below.
pub use purrdf;

/// Unstable escape hatch: the whole Eyeron crate. Not covered by GraphLaw's own
/// API stability; prefer the curated modules below.
pub use eyeron;

/// RDF 1.2 data model, codecs, builders, and related primitives.
pub mod rdf {
    pub use purrdf::{
        CanonHash, RdfDataset, RdfDatasetBuilder, RdfDiagnostic, RdfLiteral, SerializeGraph,
        SparqlEngine, SparqlRequest, SparqlResult, TermId, TermValue, canonicalize, parse_dataset,
        parse_dataset_with, serialize_dataset, try_canonicalize,
    };
}

/// SPARQL parser, algebra, evaluator, prepared execution, and result formats.
pub mod sparql {
    pub use purrdf::sparql::*;
}

/// SHACL parsing, validation, reports, rules, and schema projections.
pub mod shacl {
    pub use purrdf::shapes::*;
}

/// ShEx 2.1 parsing, structure checking, and validation.
pub mod shex {
    pub use purrdf::shex::*;
}

/// Deterministic semi-naive Datalog, stratification, chase, and proofs.
pub mod datalog {
    pub use purrdf::datalog::*;
}

/// RDF, RDFS, OWL-RL, OWL-Direct, and RIF entailment.
pub mod entailment {
    pub use purrdf::entail::*;
}

/// Streaming RDF event model.
pub mod events {
    pub use purrdf::events::*;
}

/// Notation3 parsing, forward/backward reasoning, built-ins, limits, and proofs.
pub mod n3 {
    pub use eyeron::{
        CompletionStatus, EyeronError, PreparedReasoner, RdfFormat, ReasonerError, ReasonerLimit,
        ReasonerOptions, ReasonerResult, ReasonerStatistics, Result, parse_n3, parse_rdf12,
        proof_to_n3, reason, reason_document, result_to_string, triples_to_n3, triples_to_trig,
    };
}

/// The canonical immutable, content-addressed GraphLaw law state.
pub use law::LawState;

/// One externally-owned semantic capability.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BackendAuthority {
    /// Capability family.
    pub capability: &'static str,
    /// Crate/repository that owns the executable semantics.
    pub authority: &'static str,
    /// Pinned release or revision.
    pub revision: &'static str,
}

/// Executable semantic authority map for this GraphLaw release.
pub const BACKEND_AUTHORITIES: &[BackendAuthority] = &[
    BackendAuthority {
        capability: "RDF 1.2 / codecs / storage IR",
        authority: "purrdf",
        revision: "2.0.2",
    },
    BackendAuthority {
        capability: "SPARQL 1.1/1.2",
        authority: "purrdf::sparql",
        revision: "2.0.2",
    },
    BackendAuthority {
        capability: "SHACL",
        authority: "purrdf::shapes",
        revision: "2.0.2",
    },
    BackendAuthority {
        capability: "ShEx 2.1",
        authority: "purrdf::shex",
        revision: "2.0.2",
    },
    BackendAuthority {
        capability: "Datalog",
        authority: "purrdf::datalog",
        revision: "2.0.2",
    },
    BackendAuthority {
        capability: "RDF/RDFS/OWL-RL entailment",
        authority: "purrdf::entail",
        revision: "2.0.2",
    },
    BackendAuthority {
        capability: "Knowledge hooks (kh: orchestration over SPARQL)",
        authority: "purrdf::sparql",
        revision: "2.0.2",
    },
    BackendAuthority {
        capability: "Notation3",
        authority: "eyeron",
        revision: "0.7.7 (eyereasoner/eyeron@d6568f657c19805b64223acf28d74234156bb837, vendored)",
    },
];

/// This release has no legacy semantic fallback path.
pub const LEGACY_FALLBACK_ENABLED: bool = false;
