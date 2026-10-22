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
//!
//! ```
//! use graphlaw::dialect::Dialect;
//! use graphlaw::law::{LawState, Step};
//!
//! // Parse with PurRDF, derive with PurRDF's RDFS entailment, get a receipt.
//! let state = LawState::parse(
//!     b"<https://e/Cat> <http://www.w3.org/2000/01/rdf-schema#subClassOf> <https://e/Animal> .\n\
//!       <https://e/tom> <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <https://e/Cat> .\n",
//!     Dialect::NTriples,
//!     None,
//! )?;
//! let (child, receipt) = state.transition(&Step::EntailRdfs)?;
//! assert_eq!(receipt.step, "derive:rdfs");
//! assert!(child.quad_count() > state.quad_count());
//! assert_ne!(child.id(), state.id());
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![cfg_attr(docsrs, feature(doc_cfg))]

#[cfg(all(target_os = "wasi", not(graphlaw_wasi_clock_patched)))]
compile_error!(
    "graphlaw from crates.io cannot run SPARQL on WASI: upstream reads the clock through \
     JavaScript on every wasm32 target, which traps outside the browser. The \
     `wasi-patched-deps` feature does not apply the fix. Use the prebuilt \
     graphlaw.wasm attached to each GitHub release, or build inside the graphlaw repository \
     workspace (`cargo build -p graphlaw-wasm --target wasm32-wasip1 --profile wasm`), which \
     vendors the clock gate and lets build.rs set graphlaw_wasi_clock_patched."
);

/// True when `build.rs` saw the vendored `purrdf-sparql-eval` clock gate
/// (`target_os = "unknown"`). False on a crates.io build, which does not
/// ship `vendor/` and therefore cannot compile for WASI.
pub const WASI_CLOCK_PATCH_APPLIED: bool = cfg!(graphlaw_wasi_clock_patched);

#[cfg(feature = "abi")]
#[cfg_attr(docsrs, doc(cfg(feature = "abi")))]
pub mod abi;
#[cfg(feature = "abi")]
#[cfg_attr(docsrs, doc(cfg(feature = "abi")))]
pub mod abi_manifest;
pub mod attest;
pub mod dialect;
pub mod hooks;
pub mod law;
pub mod plan;
pub mod policy;
#[cfg(feature = "abi")]
#[cfg_attr(docsrs, doc(cfg(feature = "abi")))]
pub mod qualification;
pub mod query_ledger;
pub mod receipt;
pub mod receipt_chain;
pub mod receipt_store;
pub use receipt_chain::{ReceiptRecord, validator::ReceiptValidator};
#[cfg(feature = "abi")]
#[cfg_attr(docsrs, doc(cfg(feature = "abi")))]
pub mod registry;
pub mod shacl_guard;
#[cfg(feature = "pack-tools")]
#[cfg_attr(docsrs, doc(cfg(feature = "pack-tools")))]
pub mod smon;
pub mod tier;

/// Unstable escape hatch: the whole PurRDF crate. Not covered by GraphLaw's own
/// API stability; prefer the curated modules below.
pub use purrdf;

/// Unstable escape hatch: the whole Eyeron crate. Not covered by GraphLaw's own
/// API stability; prefer the curated modules below.
pub use eyeron;

/// Unstable re-export of PurRDF's RDF API. Not covered by GraphLaw's stability promise.
pub mod rdf {
    pub use purrdf::{
        CanonHash, RdfDataset, RdfDatasetBuilder, RdfDiagnostic, RdfLiteral, SerializeGraph,
        SparqlEngine, SparqlRequest, SparqlResult, TermId, TermValue, canonicalize, parse_dataset,
        parse_dataset_with, serialize_dataset, try_canonicalize,
    };
}

/// Unstable re-export of PurRDF's SPARQL API (`purrdf::sparql::*`). Not covered by GraphLaw's stability promise.
pub mod sparql {
    pub use purrdf::sparql::*;
}

/// Unstable re-export of PurRDF's SHACL API. Not covered by GraphLaw's stability promise.
pub mod shacl {
    pub use purrdf::shapes::*;
}

/// Unstable re-export of PurRDF's ShEx API. Not covered by GraphLaw's stability promise.
pub mod shex {
    pub use purrdf::shex::*;
}

/// Unstable re-export of PurRDF's Datalog API. Not covered by GraphLaw's stability promise.
pub mod datalog {
    pub use purrdf::datalog::*;
}

/// Unstable re-export of PurRDF's entailment API. Not covered by GraphLaw's stability promise.
pub mod entailment {
    pub use purrdf::entail::*;
}

/// Unstable re-export of PurRDF's event API. Not covered by GraphLaw's stability promise.
pub mod events {
    pub use purrdf::events::*;
}

/// Unstable re-export of Eyeron's Notation3 API. Not covered by GraphLaw's stability promise.
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
    BackendAuthority {
        capability: "Vectorized columnar execution, Parquet scan, and SIMD aggregations",
        authority: "frozen-duckdb",
        revision: "2.0.0",
    },
];

/// This release has no legacy semantic fallback path.
pub const LEGACY_FALLBACK_ENABLED: bool = false;

pub mod sql_ast;
pub mod sql_statement_kinds;
/// P28-SIL §3.2 SPSC evidentiary ring + NDJSON receipt ledger (native-only,
/// feature `evidentiary-ring`, default OFF until TC-HOT-01/02 bench it).
#[cfg(all(feature = "evidentiary-ring", not(target_arch = "wasm32")))]
#[cfg_attr(docsrs, doc(cfg(feature = "evidentiary-ring")))]
pub mod evidence_ring;
/// Compiled-plan cache for SQL evaluation. Linked into the wasm module with the rest of the kernel.
pub mod query_cache;
/// SQL evaluation against frozen-duckdb. Linked into the wasm module with the rest of the kernel.
pub mod duckdb_eval;
/// ABI op `duckdb`: cursor-streamed analytical eval inside the wasm module.
pub mod duckdb_op;
pub mod rules;

