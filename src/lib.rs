//! GraphLaw is a standards-composition boundary, not a standards implementation.
//!
//! Since v26.9.28, GraphLaw delegates executable RDF semantics to maintained
//! upstream Rust libraries:
//!
//! - PurRDF owns RDF 1.2, SPARQL 1.1/1.2, SHACL, ShEx 2.1, Datalog,
//!   RDF/RDFS/OWL-RL entailment, one admission law
//!   ([`LawState`]: parse → shape-validate → entail → step, each transition
//!   content-addressed and receipted), and RDF event ingestion.
//! - purrdf::n3 (upstream purrdf-n3, vendored) owns Notation3 parsing and reasoning.
//!
//! GraphLaw intentionally contains no fallback parser, rule evaluator, triplestore,
//! SHACL/ShEx validator, SPARQL evaluator, or OWL ruleset of its own. Missing
//! capabilities are refused or added upstream instead of being reimplemented here.
//!
//! The SQL-in-RDF surface composes the same law into a pipeline: a SQL
//! statement written as an RDF graph admits only if it validates against the
//! SHACL shapes in `ontologies/sql-ast.ttl`, compiles to parameterized
//! DuckDB SQL via [`sql_ast::compile_turtle`], executes on an embedded
//! [`duckdb_eval::GraphLawDuckDB`] connection, and returns an
//! [`duckdb_eval::ExecutionReceipt`] whose hash is
//! `BLAKE3(Canon(G_AST) || P_params)` (sql-rdf Rec. section 1.4).
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
//!
//! The same shape end-to-end for the SQL surface — RDF in, SHACL admission,
//! parameterized SQL, receipt out:
//!
//! ```
//! use graphlaw::duckdb_eval::GraphLawDuckDB;
//! use graphlaw::sql_ast::compile_turtle;
//!
//! let db = GraphLawDuckDB::open_in_memory()?;
//! db.raw_connection().execute_batch(
//!     "CREATE TABLE t(c INTEGER); INSERT INTO t VALUES (42);",
//! )?;
//! let doc = concat!(
//!     "@prefix sql: <http://graphlaw.org/sql#> .\n",
//!     "@prefix ex: <http://example.org/queries#> .\n",
//!     "ex:S a sql:SelectStatement ;\n",
//!     "    sql:statementIndex 1 ;\n",
//!     "    sql:fromTable [ sql:tableName \"t\" ] ;\n",
//!     "    sql:selectList ( [ sql:columnName \"c\" ] ) .\n",
//! );
//! let compiled = compile_turtle(doc)?;
//! let receipt = db.eval_ast_query("ex:S", &compiled, "https://e/row/", "https://e/p/")?;
//! assert_eq!(receipt.rows_emitted, 1);
//! // The receipt hash binds the canonical AST graph and the bound params.
//! assert_ne!(receipt.query_hash, [0u8; 32]);
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![cfg_attr(docsrs, feature(doc_cfg))]

#[cfg(feature = "abi")]
#[cfg_attr(docsrs, doc(cfg(feature = "abi")))]
pub mod abi;
#[cfg(feature = "abi")]
#[cfg_attr(docsrs, doc(cfg(feature = "abi")))]
pub mod abi_manifest;
pub mod attest;
pub mod dialect;
/// Lease-bound proof verification: `check_proof` behind a caller lease digest (WP P01).
pub mod proof_lease;
/// CanonArrow dual-witness columnar egress (ADR-CANONARROW).
pub mod egress;
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

/// PurRDF's byte-deterministic, five-table Parquet columnar codec.
pub use purrdf_columnar;

/// Unstable re-export of PurRDF's RDF API. Not covered by GraphLaw's stability promise.
pub mod rdf {
    pub use purrdf::{
        CanonHash, RdfDataset, RdfDatasetBuilder, RdfDiagnostic, RdfLiteral, SerializeGraph,
        SparqlEngine, SparqlRequest, SparqlResult, TermId, TermValue, parse_dataset,
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

/// Unstable re-export of PurRDF's GeoSPARQL API. Not covered by GraphLaw's stability promise.
pub mod geo {
    pub use purrdf::geo::*;
}

/// Unstable re-export of PurRDF's full-text search API. Not covered by GraphLaw's stability promise.
pub mod text {
    pub use purrdf::text::*;
}

/// Unstable re-export of PurRDF's retrieval and ranking composition API.
pub mod retrieval {
    pub use purrdf::retrieval::*;
}

/// Unstable re-export of PurRDF's GTS container engine API.
pub mod gts {
    pub use purrdf::gts::*;
}

/// Unstable re-export of PurRDF's SARIF validation reporting boundary.
pub mod validate {
    pub use purrdf::validate::*;
}

/// Unstable re-export of Notation3 API (`purrdf::n3::*`). Not covered by GraphLaw's stability promise.
pub mod n3 {
    pub use purrdf::n3::*;
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
        revision: "3.0.1",
    },
    BackendAuthority {
        capability: "SPARQL 1.1/1.2",
        authority: "purrdf::sparql",
        revision: "3.0.1",
    },
    BackendAuthority {
        capability: "SHACL",
        authority: "purrdf::shapes",
        revision: "3.0.1",
    },
    BackendAuthority {
        capability: "ShEx 2.1",
        authority: "purrdf::shex",
        revision: "3.0.1",
    },
    BackendAuthority {
        capability: "Datalog",
        authority: "purrdf::datalog",
        revision: "3.0.1",
    },
    BackendAuthority {
        capability: "RDF/RDFS/OWL-RL entailment",
        authority: "purrdf::entail",
        revision: "3.0.1",
    },
    BackendAuthority {
        capability: "Knowledge hooks (kh: orchestration over SPARQL)",
        authority: "purrdf::sparql",
        revision: "3.0.1",
    },
    BackendAuthority {
        capability: "Notation3",
        authority: "purrdf::n3",
        revision: "3.0.1",
    },
    BackendAuthority {
        capability: "Vectorized columnar execution, Parquet scan, and SIMD aggregations",
        authority: "frozen-duckdb",
        revision: "2.0.2",
    },
];

/// This release has no legacy semantic fallback path.
pub const LEGACY_FALLBACK_ENABLED: bool = false;

/// SQL evaluation against frozen-duckdb. frozen-duckdb's wasm static archive is
/// rebuilt with `-fwasm-exceptions -mllvm -wasm-use-legacy-eh=0` (uniform W3C
/// exception handling) and is hosted by wasmtime 48 `wasm_exceptions`; DuckDB is
/// not a feature — the same graphlaw crate links it on every target.
pub mod duckdb_eval;
/// ABI op `duckdb`.
pub mod duckdb_op;
/// P28-SIL §3.2 SPSC evidentiary ring + NDJSON receipt ledger (native-only,
/// feature `evidentiary-ring`, default OFF until TC-HOT-01/02 bench it).
#[cfg(all(feature = "evidentiary-ring", not(target_arch = "wasm32")))]
#[cfg_attr(docsrs, doc(cfg(feature = "evidentiary-ring")))]
pub mod evidence_ring;
/// Compiled-plan cache for SQL evaluation. The frozen-duckdb wasm static
/// archive uses uniform W3C exception handling (legacy EH disabled), hosted by
/// wasmtime 48 `wasm_exceptions` — no wasmi EH rejection.
pub mod query_cache;
/// SQL builtins; they call `duckdb_eval`.
pub mod rules;
pub mod sql_ast;
pub mod sql_statement_kinds;
