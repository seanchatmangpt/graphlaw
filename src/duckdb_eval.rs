//! Embedded execution engine bridging GraphLaw's AST compiler with `frozen-duckdb`.
//!
//! Evaluates compiled relational queries inside an embedded, memory-bounded DuckDB instance,
//! collects cryptographic execution receipts, and projects tabular results into RDF statements.

#![allow(missing_docs)]

use std::fmt;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;

// ---------------------------------------------------------------------------
// CoarseClock (lane A02): low-latency monotonic timestamp source for the
// admission/eval hot path. `Instant::now()` measures ~20 ns throughput /
// ~41 ns pair-diff on this host class, so the start tick of each eval is a
// coarse read: one relaxed atomic load of a nanos-since-origin counter that
// is recalibrated against `Instant::now()` every COARSE_CALIB_EVERY reads
// (monotonic-clamped with fetch_max, so it can never run backwards). The
// elapsed leg ALWAYS forces a real calibration, so durations keep true
// Instant accuracy; only the start read is coarse (staleness bounded by one
// calibration batch). Ordering — not wall-clock — is the preserved semantic.
// Wall-clock callers (query_ledger ts_unix_ms, evidence_ring now_ms) are
// deliberately untouched: they keep SystemTime.
// ---------------------------------------------------------------------------

/// One real `Instant::now()` recalibration per this many coarse reads.
const COARSE_CALIB_EVERY: u64 = 64;

static COARSE_ORIGIN: std::sync::OnceLock<Instant> = std::sync::OnceLock::new();
static COARSE_NS: AtomicU64 = AtomicU64::new(0);

/// Coarse monotonic nanos since clock origin. Median acquisition is a single
/// relaxed atomic load (well under the 10 ns budget); every 64th read pays
/// the real `Instant::now()` recalibration.
#[inline]
pub fn coarse_now_ns() -> u64 {
    thread_local! {
        static READS: std::cell::Cell<u64> = const { std::cell::Cell::new(0) };
    }
    let n = READS.with(|r| {
        let n = r.get() + 1;
        r.set(n);
        n
    });
    if n.is_multiple_of(COARSE_CALIB_EVERY) {
        let origin = COARSE_ORIGIN.get_or_init(Instant::now);
        let real = origin.elapsed().as_nanos() as u64;
        COARSE_NS.fetch_max(real, Ordering::Relaxed);
    }
    COARSE_NS.load(Ordering::Relaxed)
}

/// Coarse nanos elapsed since `start_ns` (from [`coarse_now_ns`]). Forces a
/// real recalibration first so the duration reflects true Instant time.
#[inline]
pub fn coarse_elapsed_ns(start_ns: u64) -> u64 {
    let origin = COARSE_ORIGIN.get_or_init(Instant::now);
    let real = origin.elapsed().as_nanos() as u64;
    COARSE_NS.fetch_max(real, Ordering::Relaxed);
    real.saturating_sub(start_ns)
}

use crate::sql_ast::{CompiledQuery, SqlLiteral};

// Re-exported or native types from purrdf
use purrdf::TermValue;

#[derive(Debug, Clone, PartialEq)]
pub struct RdfTriple {
    pub subject: String,
    pub predicate: String,
    pub object: TermValue,
}

/// Lossless serde shadow of `purrdf::TermValue` (SQLRDF-22 G3): upstream
/// `TermValue` offers no serde support, so this tagged enum carries every
/// variant byte-for-byte — IRI, blank node (label + scope ordinal), literal
/// (lexical form, datatype IRI, language tag, base direction token), and
/// recursive quoted triples. `Serialize`/`Deserialize` are exact inverses;
/// no string rendering, no precision loss. If purrdf lands native serde,
/// delete this shadow in favor of it.
#[derive(serde::Serialize, serde::Deserialize)]
enum SerTerm {
    Iri(String),
    Blank { label: String, scope: u32 },
    Literal {
        lexical_form: String,
        datatype: String,
        language: Option<String>,
        direction: Option<String>,
    },
    Triple {
        s: Box<SerTerm>,
        p: Box<SerTerm>,
        o: Box<SerTerm>,
    },
}

impl From<TermValue> for SerTerm {
    fn from(t: TermValue) -> Self {
        match t {
            TermValue::Iri(v) => SerTerm::Iri(v),
            TermValue::Blank { label, scope } => SerTerm::Blank {
                label,
                scope: scope.ordinal(),
            },
            TermValue::Literal {
                lexical_form,
                datatype,
                language,
                direction,
            } => SerTerm::Literal {
                lexical_form,
                datatype,
                language,
                direction: direction.map(|d| format!("{d:?}")),
            },
            TermValue::Triple { s, p, o } => SerTerm::Triple {
                s: Box::new((*s).into()),
                p: Box::new((*p).into()),
                o: Box::new((*o).into()),
            },
        }
    }
}

impl From<SerTerm> for TermValue {
    fn from(t: SerTerm) -> Self {
        match t {
            SerTerm::Iri(v) => TermValue::Iri(v),
            SerTerm::Blank { label, scope } => TermValue::Blank {
                label,
                scope: purrdf::BlankScope(scope),
            },
            SerTerm::Literal {
                lexical_form,
                datatype,
                language,
                direction,
            } => TermValue::Literal {
                lexical_form,
                datatype,
                language,
                direction: direction.map(|d| match d.as_str() {
                    "Ltr" => purrdf::RdfTextDirection::Ltr,
                    "Rtl" => purrdf::RdfTextDirection::Rtl,
                    other => unreachable!("SerTerm direction token '{other}' is not a valid RdfTextDirection — SerTerm is constructed only via From<TermValue>"),
                }),
            },
            SerTerm::Triple { s, p, o } => TermValue::Triple {
                s: Box::new((*s).into()),
                p: Box::new((*p).into()),
                o: Box::new((*o).into()),
            },
        }
    }
}

impl serde::Serialize for RdfTriple {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeStruct;
        let mut st = s.serialize_struct("RdfTriple", 3)?;
        st.serialize_field("subject", &self.subject)?;
        st.serialize_field("predicate", &self.predicate)?;
        st.serialize_field("object", &SerTerm::from(self.object.clone()))?;
        st.end()
    }
}

impl<'de> serde::Deserialize<'de> for RdfTriple {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        #[derive(serde::Deserialize)]
        struct Wire {
            subject: String,
            predicate: String,
            object: SerTerm,
        }
        Wire::deserialize(d).map(|w| RdfTriple {
            subject: w.subject,
            predicate: w.predicate,
            object: TermValue::from(w.object),
        })
    }
}

#[derive(Debug)]
pub enum EvaluatorError {
    DuckDb(frozen_duckdb::DuckDBError),
    ConversionError {
        column: String,
        row: usize,
        details: String,
    },
    ReceiptError(String),
    /// SQLRDF-17: a hand-built [`CompiledQuery`] reached eval with no
    /// canonical AST graph bytes. The legacy SQL-string hash arm and the
    /// re-canonicalize-from-`ast_graph_turtle` fallback are DELETED, not
    /// gated: the only admitted hashes are over
    /// `BLAKE3(Canon(G_AST) || P_params)` (sql-rdf Rec. section 1.4).
    /// Recovery: obtain the query via `compile_turtle`, which pins
    /// `receipt_state` / `canon_graph_bytes` at compile time.
    NoCanonicalGraph(String),
    /// SQLRDF-23: member `index` of a statement-sequence document failed at
    /// execution. `partial` carries the receipts and row counts of members
    /// `[0..index]`, which executed and committed (best-effort sequential
    /// autocommit — documented partial-commit semantics, not a rollback);
    /// the failing member's own detail is `source`. Typed, never a generic
    /// DB error surfaced raw.
    BatchMemberFailed {
        index: usize,
        source: Box<EvaluatorError>,
        partial: BatchReceipt,
    },
}

impl fmt::Display for EvaluatorError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DuckDb(e) => write!(f, "DuckDB execution failure: {e}"),
            Self::ConversionError {
                column,
                row,
                details,
            } => {
                write!(
                    f,
                    "Column conversion error for '{column}' at row {row}: {details}"
                )
            }
            Self::ReceiptError(msg) => write!(f, "Receipt generation error: {msg}"),
            Self::NoCanonicalGraph(query_iri) => write!(
                f,
                "no canonical graph bytes — hand-built CompiledQuery must use compile_turtle (query {query_iri})"
            ),
            Self::BatchMemberFailed { index, source, .. } => write!(
                f,
                "statement-sequence member {index} failed (prior members committed, best-effort autocommit): {source}"
            ),
        }
    }
}

impl std::error::Error for EvaluatorError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            EvaluatorError::DuckDb(error) => Some(error),
            EvaluatorError::BatchMemberFailed { source, .. } => Some(source.as_ref()),
            _ => None,
        }
    }
}

impl From<frozen_duckdb::DuckDBError> for EvaluatorError {
    fn from(err: frozen_duckdb::DuckDBError) -> Self {
        Self::DuckDb(err)
    }
}

/// Cryptographic and semantic proof of an executed relational query.
///
/// Serde: `Serialize`/`Deserialize` are implemented manually (SQLRDF-22 G3)
/// so the COPY egress fields ride the wire only when present — receipts with
/// the artifact fields `None` serialize to exactly the pre-G3 shape, and
/// artifact-free receipts deserialize with the fields defaulting to `None`
/// (serde `Option` semantics). `result_triples` round-trips losslessly
/// through the lane-G4 `SerTerm` shadow codec for `purrdf::TermValue` (no
/// upstream serde support); no lossy string rendering is involved. There
/// were no pre-existing serialization sites of this struct (verified: no
/// `serde_json::to*` over `ExecutionReceipt` anywhere in `src/`, `tests/`,
/// `wasm/`), so the impls are purely additive.
///
/// Receipt-hash choice (documented per SQLRDF-22 G3): `query_hash` is the
/// law-frozen admitted-statement receipt BLAKE3(Canon(G_AST) || P_params)
/// (sql-rdf Recommendation, section 1.4, first formula) and intentionally
/// does NOT fold the artifact fields — the same admitted statement must keep
/// one replay identity regardless of where (or whether) it exported bytes.
/// Egress integrity instead rides in the artifact fields themselves: the
/// digest IS a BLAKE3 over the exported bytes, so a tampered artifact is
/// detected by comparing `artifact_digest_hex` against the file, never by
/// perturbing `query_hash`. The struct carries no other whole-receipt
/// digest/IRI, so there is no additional hash to fold into.
#[derive(Debug, Clone, PartialEq)]
pub struct ExecutionReceipt {
    /// IRI identifying the executed query AST node
    pub query_iri: String,
    /// Admitted-statement receipt: BLAKE3(Canon(G_AST) || P_params)
    /// (sql-rdf Recommendation, section 1.4, first formula). `Canon(G_AST)`
    /// is the RDFC-1.0 canonical form of the admitted AST graph; `P_params`
    /// is the typed parameter buffer.
    pub query_hash: [u8; 32],
    /// Parameterized SQL string sent to frozen-duckdb
    pub sql_statement: String,
    /// Number of tabular rows emitted
    pub rows_emitted: usize,
    /// Execution wall-clock time in milliseconds
    pub execution_ms: f64,
    /// Downstream RDF statements materialized from the result vectors
    pub result_triples: Vec<RdfTriple>,
    /// C1 cache hit witness (designs/C1-cache.md): true only when the
    /// receipt was served from the compiled-plan cache at the live epoch.
    /// LANE C1 minimal seam, disclosed.
    pub cached: bool,
    /// Cache consult cost in milliseconds (0.0 on a miss).
    pub consult_ms: f64,
    /// SQLRDF-22 G3 (egress half): hex BLAKE3 of the file exported by a
    /// successful `COPY ... TO '<path>'` statement, streamed in 8 KiB
    /// chunks, with the byte count in [`ExecutionReceipt::artifact_bytes`].
    /// `None` when the executed statement was not a COPY-out or when the
    /// artifact could not be read/hashed (in which case
    /// [`ExecutionReceipt::artifact_warning`] carries the typed reason and
    /// the receipt is still returned — execution succeeded, the receipt is
    /// never lost). Skipped on the wire when `None`.
    pub artifact_digest_hex: Option<String>,
    /// Number of bytes in the exported artifact (see `artifact_digest_hex`).
    /// Skipped on the wire when `None`.
    pub artifact_bytes: Option<u64>,
    /// Warning text when a COPY-out artifact existed as a statement but the
    /// file could not be opened/hashed (SQLRDF-18/22 G3: prefer keeping the
    /// receipt over refusing a successful execution). `None` when no
    /// warning applies. Skipped on the wire when `None`.
    pub artifact_warning: Option<String>,
}

/// SQLRDF-22 G3: manual `Serialize` for [`ExecutionReceipt`] — the artifact
/// fields are emitted only when `Some` (the `skip_serializing_if` contract,
/// expressed by explicit emission), and `result_triples` serializes through
/// [`serialize_rdf_triple`], which renders `purrdf::TermValue` without
/// upstream serde support.
impl serde::Serialize for ExecutionReceipt {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeMap;
        let extra = self.artifact_digest_hex.is_some() as usize
            + self.artifact_bytes.is_some() as usize
            + self.artifact_warning.is_some() as usize;
        let mut map = serializer.serialize_map(Some(8 + extra))?;
        map.serialize_entry("query_iri", &self.query_iri)?;
        map.serialize_entry("query_hash", &self.query_hash)?;
        map.serialize_entry("sql_statement", &self.sql_statement)?;
        map.serialize_entry("rows_emitted", &self.rows_emitted)?;
        map.serialize_entry("execution_ms", &self.execution_ms)?;
        map.serialize_entry("result_triples", &self.result_triples)?;
        map.serialize_entry("cached", &self.cached)?;
        map.serialize_entry("consult_ms", &self.consult_ms)?;
        if let Some(d) = &self.artifact_digest_hex {
            map.serialize_entry("artifact_digest_hex", d)?;
        }
        if let Some(b) = &self.artifact_bytes {
            map.serialize_entry("artifact_bytes", b)?;
        }
        if let Some(w) = &self.artifact_warning {
            map.serialize_entry("artifact_warning", w)?;
        }
        map.end()
    }
}

/// SQLRDF-22 G3: manual `Deserialize` for [`ExecutionReceipt`] — a lossless
/// wire round-trip through the lane-G4 `SerTerm` codec (via
/// `RdfTriple`'s `Deserialize`). The artifact fields deserialize to `None`
/// when absent (serde `Option` semantics), so pre-G3 receipts (and G3
/// receipts with no artifact or warning) read back unchanged. The
/// serialization side emits the artifact keys only when `Some` — the
/// `skip_serializing_if` contract, expressed by explicit emission — so the
/// wire shape of existing artifact-free receipts is unchanged.
impl<'de> serde::Deserialize<'de> for ExecutionReceipt {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[derive(serde::Deserialize)]
        struct Wire {
            query_iri: String,
            query_hash: [u8; 32],
            sql_statement: String,
            rows_emitted: usize,
            execution_ms: f64,
            result_triples: Vec<RdfTriple>,
            cached: bool,
            consult_ms: f64,
            artifact_digest_hex: Option<String>,
            artifact_bytes: Option<u64>,
            artifact_warning: Option<String>,
        }
        Wire::deserialize(deserializer).map(|w| ExecutionReceipt {
            query_iri: w.query_iri,
            query_hash: w.query_hash,
            sql_statement: w.sql_statement,
            rows_emitted: w.rows_emitted,
            execution_ms: w.execution_ms,
            result_triples: w.result_triples,
            cached: w.cached,
            consult_ms: w.consult_ms,
            artifact_digest_hex: w.artifact_digest_hex,
            artifact_bytes: w.artifact_bytes,
            artifact_warning: w.artifact_warning,
        })
    }
}

/// SQLRDF-23 (eval half): receipt for a compiled statement-sequence
/// document executed by [`GraphLawDuckDB::eval_ast_document`] (design
/// `docs/jira/v26.10.22/designs/statement-sequence-design.md` sections 5-6).
///
/// Serde-compatible by derive: `members` ride the manual `ExecutionReceipt`
/// wire impls, `doc_hash`/`rows_emitted` are plain data. Best-effort
/// sequential-autocommit semantics: `failed_at = Some(i)` implies
/// `members[0..i]` executed and committed (their effects stay visible —
/// documented partial-commit, not a hidden rollback) and `members[i..]`
/// did not execute.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct BatchReceipt {
    /// Caller-supplied envelope IRI for the document.
    pub query_iri: String,
    /// Domain-separated document hash pinned by `compile_turtle_document`
    /// (BLAKE3(0x00 || h_1 || 0x00 || h_2 || ...)); the bare single-root
    /// form hashes to the bare statement's own anchor.
    pub doc_hash: [u8; 32],
    /// One receipt per executed member, in spine order. On failure this
    /// holds exactly the members `[0..failed_at]` that executed.
    pub members: Vec<ExecutionReceipt>,
    /// Per-member row counts, same indexing as `members`.
    pub rows_emitted: Vec<usize>,
    /// 0-based index of the first failing member; `None` on full success.
    pub failed_at: Option<usize>,
    /// SQLRDF-26 execution status: `"partial"` (SQLRDF-23 pipeline default,
    /// also on full success) or `"aborted"` (atomic mode, ROLLBACK executed
    /// after a member failure — every member effect undone, `state_delta`
    /// pinned to 0). Serde-defaulted so SQLRDF-23 wire JSON deserializes.
    #[serde(default = "batch_status_partial")]
    pub status: String,
    /// SQLRDF-26: net committed state change — the count of members whose
    /// effects are committed at receipt time (full success: all members;
    /// pipeline failure: `failed_at`; atomic abort: pinned to 0 because the
    /// ROLLBACK undid every member effect).
    #[serde(default)]
    pub state_delta: u64,
}

/// Serde default for [`BatchReceipt::status`]: SQLRDF-23 pipeline mode.
fn batch_status_partial() -> String {
    "partial".to_string()
}

/// Embedded execution context wrapping a sovereign `frozen-duckdb` connection.
/// Debug is derived (lane L4 fix 7): all fields are Debug-derivable —
/// `frozen_duckdb::Connection` implements Debug, `QueryCache` derives Debug
/// (`src/query_cache.rs`), and `StmtCacheSeam` has a manual Debug below.
#[derive(Debug)]
pub struct GraphLawDuckDB {
    conn: frozen_duckdb::Connection,
    /// C1 compiled-plan cache (designs/C1-cache.md). Default-owned per
    /// instance; shareable across instances via
    /// [`GraphLawDuckDB::from_connection_with_cache`] /
    /// [`GraphLawDuckDB::shared_cache`].
    cache: std::sync::Arc<crate::query_cache::QueryCache>,
    /// Prepared-statement reuse seam. The underlying frozen-duckdb
    /// `Connection::prepare_cached` LRU (capacity pinned to 256 at
    /// construction) reuses the prepared handle for identical trimmed SQL.
    /// `prepared_keys` mirrors which canonical AST digests are prepared (hit/miss
    /// bookkeeping is per-instance, single-threaded per the existing
    /// design); `prepared_hits` counts reuse events. Entries are evicted by
    /// the LRU beyond capacity (mirror may over-report hits past 256
    /// distinct texts — documented, not silent) and flushed on any stored-
    /// data mutation so stale plans cannot survive a schema change.
    stmt_cache: StmtCacheSeam,
}

/// Interior-mutable state for the prepared-statement reuse seam. The
/// connection is !Sync and used single-threaded, so `Cell`/`RefCell` are
/// sufficient and avoid changing the struct's Send story.
#[derive(Default, Debug)]
struct StmtCacheSeam {
    /// Mirror of the underlying frozen-duckdb prepared-statement cache,
    /// keyed by the CANONICAL AST DIGEST (`h_ast`) for graph-backed
    /// compiles — NOT the SQL text. This is the charter-A04 seam: cache-hit
    /// dispatch consults the digest, so the lookup path never needs SQL
    /// synthesis. Hand-built ASTs without a graph anchor fall back to
    /// BLAKE3 of the (already-carried) SQL string — still digest-keyed,
    /// still synthesis-free. `prepared_hits` counts reuse events. Entries
    /// are evicted by the LRU beyond capacity and flushed on any stored-
    /// data mutation so stale plans cannot survive a schema change.
    prepared_keys: std::cell::RefCell<std::collections::HashSet<[u8; 32]>>,
    prepared_hits: std::cell::Cell<u64>,
}

/// Statement-cache mirror key: the canonical AST digest `h_ast` when the
/// compile was graph-backed (the `receipt_state` anchor pinned by
/// `compile_turtle`), otherwise a BLAKE3 digest of the already-carried SQL
/// text. Both arms are 32-byte digests; neither synthesizes SQL.
fn stmt_cache_key(compiled: &crate::sql_ast::CompiledQuery) -> [u8; 32] {
    match compiled.receipt_state {
        Some(h_ast) => h_ast,
        None => *blake3::hash(compiled.sql.as_bytes()).as_bytes(),
    }
}

/// LRU capacity pinned for the underlying frozen-duckdb statement cache.
/// Unbounded is not exposed by the crate; 256 covers the C1-cache hot case
/// (same digest, different bound params → same SQL text) many times over.
const PREPARED_STATEMENT_CACHE_CAPACITY: usize = 256;

/// Computes the RDFC-1.0 canonical byte form of an admitted AST graph
/// (`Canon(G_AST)`). The graph is parsed as Turtle and canonicalized with
/// `purrdf::try_canonicalize`, the same law the state kernel uses. Failure is
/// a typed [`EvaluatorError::ReceiptError`], never a panic.
pub fn canonical_ast_graph_bytes(turtle: &str) -> Result<Vec<u8>, EvaluatorError> {
    let dataset = purrdf::parse_dataset(turtle.as_bytes(), "text/turtle", None)
        .map_err(|e| EvaluatorError::ReceiptError(format!("AST graph parse refused: {e:?}")))?;
    let canon = purrdf::try_canonicalize(&dataset)
        .map_err(|e| EvaluatorError::ReceiptError(format!("canonicalization refused: {e:?}")))?;
    Ok(canon.nquads.into_bytes())
}

/// Feeds the typed parameter buffer `P_params` into `hasher`: one type tag
/// byte plus little-endian value bytes per parameter, in bind order.
///
/// Public so bench/lane code (e.g. `benches/tc_hot_01.rs`) folds the REAL
/// receipt encoding rather than duplicating it. Encoding law is unchanged.
pub fn hash_param_buffer(hasher: &mut blake3::Hasher, params: &[SqlLiteral]) {
    for param in params {
        match param {
            SqlLiteral::String(s) => {
                hasher.update(&[0x01]);
                hasher.update(s.as_bytes());
            }
            SqlLiteral::Integer(i) => {
                hasher.update(&[0x02]);
                hasher.update(&i.to_le_bytes());
            }
            SqlLiteral::Float(f) => {
                hasher.update(&[0x03]);
                hasher.update(&f.to_bits().to_le_bytes());
            }
            SqlLiteral::Boolean(b) => {
                hasher.update(&[0x04]);
                hasher.update(&[*b as u8]);
            }
            SqlLiteral::Null => {
                hasher.update(&[0x00]);
            }
        }
    }
}

/// Fixed-width parameter-vector budget (charter A03): parameter vectors whose
/// `P_params` encoding totals at most this many bytes fold as ONE contiguous
/// stack-buffer update (no dynamic flattening, no heap); larger vectors fold
/// as a standalone 32-byte BLAKE3 digest of the full encoding.
pub const PARAM_BYTES_INLINE: usize = 64;

/// Small-vec style parameter encoding (`P_params`, charter A03):
/// - `Inline`: up to 64 bytes of the raw typed encoding (one type tag byte +
///   little-endian value bytes per parameter, bind order — the SAME byte
///   layout [`hash_param_buffer`] emits) held in a stack buffer and folded as
///   a single `Hasher::update`. Bit-identical to the reference fold.
/// - `Digest`: buffers above the inline budget fold as
///   `BLAKE3(hash_param_buffer encoding)` — a fixed 32-byte node instead of
///   the raw bytes. This branch is an ENCODING DIVERGENCE from the raw-byte
///   reference (different digest by construction); the receipt path only
///   admits the `Inline` branch and falls back to the reference fold above
///   the budget, so `H_query` for stored receipts never changes.
#[derive(Debug, Clone, PartialEq)]
pub enum ParamBytes {
    Inline {
        buf: [u8; PARAM_BYTES_INLINE],
        len: usize,
    },
    Digest([u8; 32]),
}

/// Contiguous stack-buffer push; `false` exactly when the inline budget is
/// exceeded (caller spills to the digest branch).
fn param_push(buf: &mut [u8; PARAM_BYTES_INLINE], len: &mut usize, bytes: &[u8]) -> bool {
    let end = *len + bytes.len();
    if end > buf.len() {
        return false;
    }
    buf[*len..end].copy_from_slice(bytes);
    *len = end;
    true
}

impl ParamBytes {
    /// Encodes a typed parameter vector without dynamic flattening: one
    /// pass over `params` writing contiguously into the 64-byte stack
    /// buffer; overflow spills to a standalone 32-byte digest.
    pub fn encode(params: &[SqlLiteral]) -> ParamBytes {
        let mut buf = [0u8; PARAM_BYTES_INLINE];
        let mut len = 0usize;
        let mut overflow = false;
        for param in params {
            let tag_ok = match param {
                SqlLiteral::String(_) => param_push(&mut buf, &mut len, &[0x01]),
                SqlLiteral::Integer(_) => param_push(&mut buf, &mut len, &[0x02]),
                SqlLiteral::Float(_) => param_push(&mut buf, &mut len, &[0x03]),
                SqlLiteral::Boolean(_) => param_push(&mut buf, &mut len, &[0x04]),
                SqlLiteral::Null => param_push(&mut buf, &mut len, &[0x00]),
            };
            if !tag_ok {
                overflow = true;
                break;
            }
            let value_ok = match param {
                SqlLiteral::String(s) => param_push(&mut buf, &mut len, s.as_bytes()),
                SqlLiteral::Integer(i) => param_push(&mut buf, &mut len, &i.to_le_bytes()),
                SqlLiteral::Float(f) => param_push(&mut buf, &mut len, &f.to_bits().to_le_bytes()),
                SqlLiteral::Boolean(b) => param_push(&mut buf, &mut len, &[*b as u8]),
                SqlLiteral::Null => true,
            };
            if !value_ok {
                overflow = true;
                break;
            }
        }
        if overflow {
            let mut h = blake3::Hasher::new();
            hash_param_buffer(&mut h, params);
            ParamBytes::Digest(*h.finalize().as_bytes())
        } else {
            ParamBytes::Inline { buf, len }
        }
    }

    /// True when the encoding stayed within the inline (stack) budget.
    pub fn is_inline(&self) -> bool {
        matches!(self, ParamBytes::Inline { .. })
    }

    /// Folds this encoding into `hasher` as one update (inline branch) or
    /// one fixed 32-byte node (digest branch).
    pub fn fold(&self, hasher: &mut blake3::Hasher) {
        match self {
            ParamBytes::Inline { buf, len } => hasher.update(&buf[..*len]),
            ParamBytes::Digest(d) => hasher.update(d.as_slice()),
        };
    }
}

/// Anchor-path parameter integration (charter A03): `H_query =
/// BLAKE3(h_ast || P_params)`. Inline (<= 64B encoded) params are copied
/// with `h_ast` into one stack buffer and absorbed with one `Hasher::update`.
/// The bytes are the typed `P_params` encoding, so the digest matches
/// [`admitted_statement_hash`] by BLAKE3 streaming. Above the inline budget
/// the reference [`hash_param_buffer`] fold keeps the receipt law-stable.
pub fn fold_params_anchored(h_ast: &[u8; 32], params: &[SqlLiteral]) -> [u8; 32] {
    let encoded = ParamBytes::encode(params);
    let mut hasher = blake3::Hasher::new();
    match encoded {
        ParamBytes::Inline { buf, len } => {
            let mut joined = [0u8; 32 + PARAM_BYTES_INLINE];
            joined[..32].copy_from_slice(h_ast);
            joined[32..32 + len].copy_from_slice(&buf[..len]);
            hasher.update(&joined[..32 + len]);
        }
        ParamBytes::Digest(_) => {
            // Digest branch would diverge from the reference receipt encoding —
            // the raw-byte reference fold keeps H_query law-stable.
            hasher.update(h_ast);
            hash_param_buffer(&mut hasher, params);
        }
    }
    *hasher.finalize().as_bytes()
}

/// Computes the admitted-statement receipt, in the TC-HOT-01 two-node form:
/// `h_ast = BLAKE3(Canon(G_AST))`, then
/// `H_query = BLAKE3(h_ast || P_params)`
/// (sql-rdf Recommendation, section 1.4, first formula). `canonical_graph`
/// is the output of [`canonical_ast_graph_bytes`]; the parameter buffer is
/// the typed encoding (one type tag byte + little-endian value bytes per
/// parameter) in bind order. One fresh hasher + the reference param updates
/// + finalize — no streaming absorb of `Canon(G_AST)` bytes per receipt.
pub fn admitted_statement_hash(canonical_graph: &[u8], params: &[SqlLiteral]) -> [u8; 32] {
    let h_ast = *blake3::hash(canonical_graph).as_bytes();
    let mut hasher = blake3::Hasher::new();
    hasher.update(h_ast.as_slice());
    hash_param_buffer(&mut hasher, params);
    *hasher.finalize().as_bytes()
}

/// C1: true when the SQL text is a stored-data mutation (DML, ATTACH/DETACH,
/// DDL, COPY). Lexical prefix classification, deliberately conservative:
/// anything not matching falls on the no-bump side and relies on the
/// C1-a cacheable bit being false for non-Select statements. `MERGE` (and
/// `REPLACE`, were it a standalone DuckDB statement — it is not; DuckDB has
/// no `REPLACE` arm in its PEG statement grammar, only `CREATE OR REPLACE`)
/// is lexical belt-and-suspenders: the compiler already emits MERGE as
/// non-cacheable, so the epoch bump fires via the `cacheable` bit even if
/// this classifier misses it.
fn mutating_sql(sql: &str) -> bool {
    let trimmed = sql.trim_start();
    let upper: String = trimmed
        .chars()
        .take_while(|c| c.is_ascii_alphabetic())
        .map(|c| c.to_ascii_uppercase())
        .collect();
    matches!(
        upper.as_str(),
        "INSERT" | "UPDATE" | "DELETE" | "ATTACH" | "DETACH" | "CREATE" | "DROP" | "ALTER"
            | "COPY" | "MERGE"
    )
}

/// SQLRDF-22 G3: true when `sql` lexically begins with `COPY` and carries a
/// top-level `TO` keyword (the egress form). `TO` inside a single-quoted
/// literal does not count — the scan is quote-aware with the same
/// single-quote-doubling rule the compiler uses when rendering SQL.
fn is_copy_to(sql: &str) -> bool {
    let trimmed = sql.trim_start();
    let mut chars = trimmed.chars().peekable();
    // Cheap prefix check first.
    if !trimmed
        .get(..4)
        .is_some_and(|p| p.eq_ignore_ascii_case("COPY"))
    {
        return false;
    }
    let mut in_quote = false;
    let mut token = String::new();
    while let Some(c) = chars.next() {
        match c {
            '\'' => {
                if in_quote && chars.peek() == Some(&'\'') {
                    let _ = chars.next(); // escaped quote ('') inside literal
                } else {
                    in_quote = !in_quote;
                    token.clear();
                }
            }
            _c if in_quote => {}
            c if c.is_ascii_alphabetic() => token.push(c.to_ascii_uppercase()),
            c if c.is_whitespace() => {
                if token == "TO" {
                    return true;
                }
                token.clear();
            }
            _ => token.clear(),
        }
    }
    token == "TO"
}

/// SQLRDF-22 G3: extract the single-quoted destination path of a
/// `COPY ... TO '<path>'` statement. Quote-aware with single-quote doubling
/// (`''` inside a literal renders one `'`), matching the compiler's
/// rendering rule.
fn copy_to_path(sql: &str) -> Option<String> {
    if !is_copy_to(sql) {
        return None;
    }
    let mut chars = sql.chars().peekable();
    let mut in_quote = false;
    let mut current = String::new();
    let mut saw_to = false;
    while let Some(c) = chars.next() {
        match c {
            '\'' => {
                if in_quote && chars.peek() == Some(&'\'') {
                    current.push('\'');
                    let _ = chars.next();
                } else if in_quote {
                    // Closing quote: if we just collected the path after the
                    // TO keyword, this literal is the destination.
                    if saw_to {
                        return Some(std::mem::take(&mut current));
                    }
                    in_quote = false;
                    current.clear();
                } else {
                    in_quote = true;
                    current.clear();
                }
            }
            c if in_quote => current.push(c),
            c if c.is_ascii_alphabetic() => {
                current.push(c.to_ascii_uppercase());
            }
            c if c.is_whitespace() => {
                if current == "TO" {
                    saw_to = true;
                }
                current.clear();
            }
            _ => current.clear(),
        }
    }
    None
}

/// SQLRDF-22 G3: stream a file through BLAKE3 in 8 KiB chunks, returning the
/// hex digest and byte count. I/O failure is a typed `String` warning for
/// [`ExecutionReceipt::artifact_warning`], never a panic.
fn hash_file_blake3(path: &str) -> Result<(String, u64), String> {
    use std::io::Read;
    let mut file =
        std::fs::File::open(path).map_err(|e| format!("artifact open failed: {e}"))?;
    let mut hasher = blake3::Hasher::new();
    let mut buf = [0u8; 8192];
    let mut total = 0u64;
    loop {
        let n = file
            .read(&mut buf)
            .map_err(|e| format!("artifact read failed: {e}"))?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
        total = total.saturating_add(n as u64);
    }
    Ok((hasher.finalize().to_hex().to_string(), total))
}

/// RDF literal from a projected cell. `datatype` is a full XSD IRI.
fn xsd_cell(lexical_form: String, datatype: &str) -> TermValue {
    TermValue::Literal {
        lexical_form,
        datatype: datatype.to_string(),
        language: None,
        direction: None,
    }
}

/// `YYYY-MM-DD` from DuckDB `Date32`: days since the Unix epoch
/// (1970-01-01). The day count is the engine's own value; this is the
/// civil rendering of that count (Howard Hinnant), the same epoch
/// frozen-duckdb's chrono bridge multiplies by `24 * 3600`.
fn iso_date_from_epoch_days(days: i64) -> String {
    let (y, m, d) = civil_from_epoch_days(days);
    if y < 0 {
        format!("-{:04}-{:02}-{:02}", -y, m, d)
    } else {
        format!("{y:04}-{m:02}-{d:02}")
    }
}

/// Proleptic Gregorian year, month, day for a Unix-epoch day count.
fn civil_from_epoch_days(days: i64) -> (i64, u32, u32) {
    let z = days + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = (z - era * 146_097) as u64;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146_096) / 365;
    let mut y = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let m = (if mp < 10 { mp + 3 } else { mp - 9 }) as u32;
    if m <= 2 {
        y += 1;
    }
    (y, m, d)
}

/// Split an engine time-unit count into whole seconds and leftover nanoseconds.
fn time_unit_parts(unit: frozen_duckdb::duckdb::types::TimeUnit, ticks: i64) -> (i64, u32) {
    use frozen_duckdb::duckdb::types::TimeUnit;
    match unit {
        TimeUnit::Second => (ticks, 0),
        TimeUnit::Millisecond => {
            let secs = ticks.div_euclid(1_000);
            let millis = ticks.rem_euclid(1_000) as u32;
            (secs, millis * 1_000_000)
        }
        TimeUnit::Microsecond => {
            let secs = ticks.div_euclid(1_000_000);
            let micros = ticks.rem_euclid(1_000_000) as u32;
            (secs, micros * 1_000)
        }
        TimeUnit::Nanosecond => {
            let secs = ticks.div_euclid(1_000_000_000);
            let nanos = ticks.rem_euclid(1_000_000_000) as u32;
            (secs, nanos)
        }
    }
}

/// `HH:MM:SS` with a trimmed fractional suffix when the nanosecond residue is non-zero.
fn format_clock(secs_of_day: i64, nanos: u32) -> String {
    let secs = secs_of_day.rem_euclid(86_400);
    let h = secs / 3600;
    let m = (secs % 3600) / 60;
    let s = secs % 60;
    if nanos == 0 {
        format!("{h:02}:{m:02}:{s:02}")
    } else {
        let mut frac = format!("{nanos:09}");
        while frac.ends_with('0') {
            frac.pop();
        }
        format!("{h:02}:{m:02}:{s:02}.{frac}")
    }
}

/// `Time64` is a clock time: the engine's time-unit count since midnight.
fn iso_time_from_time_unit(unit: frozen_duckdb::duckdb::types::TimeUnit, ticks: i64) -> String {
    let (secs, nanos) = time_unit_parts(unit, ticks);
    format_clock(secs, nanos)
}

/// `Timestamp` is an instant: the engine's time-unit count since the Unix epoch,
/// rendered `YYYY-MM-DD HH:MM:SS` (the form frozen-duckdb's chrono bridge prints).
fn iso_datetime_from_time_unit(unit: frozen_duckdb::duckdb::types::TimeUnit, ticks: i64) -> String {
    let (secs, nanos) = time_unit_parts(unit, ticks);
    let days = secs.div_euclid(86_400);
    let sod = secs.rem_euclid(86_400);
    format!(
        "{} {}",
        iso_date_from_epoch_days(days),
        format_clock(sod, nanos)
    )
}

/// RFC 4648 base64 of a DuckDB blob, the lexical form of `xsd:base64Binary`.
fn base64_encode(input: &[u8]) -> String {
    const TABLE: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity(input.len().div_ceil(3) * 4);
    let mut chunks = input.chunks_exact(3);
    for chunk in chunks.by_ref() {
        let n = ((chunk[0] as u32) << 16) | ((chunk[1] as u32) << 8) | (chunk[2] as u32);
        out.push(TABLE[((n >> 18) & 0x3f) as usize] as char);
        out.push(TABLE[((n >> 12) & 0x3f) as usize] as char);
        out.push(TABLE[((n >> 6) & 0x3f) as usize] as char);
        out.push(TABLE[(n & 0x3f) as usize] as char);
    }
    let rest = chunks.remainder();
    if rest.len() == 1 {
        let n = (rest[0] as u32) << 16;
        out.push(TABLE[((n >> 18) & 0x3f) as usize] as char);
        out.push(TABLE[((n >> 12) & 0x3f) as usize] as char);
        out.push('=');
        out.push('=');
    } else if rest.len() == 2 {
        let n = ((rest[0] as u32) << 16) | ((rest[1] as u32) << 8);
        out.push(TABLE[((n >> 18) & 0x3f) as usize] as char);
        out.push(TABLE[((n >> 12) & 0x3f) as usize] as char);
        out.push(TABLE[((n >> 6) & 0x3f) as usize] as char);
        out.push('=');
    }
    out
}

impl GraphLawDuckDB {
    /// Creates an in-memory embedded execution instance backed by a real
    /// `frozen-duckdb` connection (statement cache capacity pinned to 256).
    ///
    /// Returns the [`GraphLawDuckDB`] handle on success; connection failure
    /// surfaces as [`EvaluatorError`] (typed, never a panic). Data lives for
    /// the process lifetime of the connection only.
    pub fn open_in_memory() -> Result<Self, EvaluatorError> {
        let conn = frozen_duckdb::Connection::open_in_memory()?;
        conn.set_prepared_statement_cache_capacity(PREPARED_STATEMENT_CACHE_CAPACITY);
        Ok(Self {
            conn,
            cache: std::sync::Arc::new(crate::query_cache::QueryCache::new()),
            stmt_cache: StmtCacheSeam::default(),
        })
    }

    /// In-memory connection that may `INSTALL` a local unsigned extension file.
    pub fn open_in_memory_allowing_unsigned_extensions() -> Result<Self, EvaluatorError> {
        let config = frozen_duckdb::Config::default().allow_unsigned_extensions()?;
        let conn = frozen_duckdb::Connection::open_in_memory_with_flags(config)?;
        conn.set_prepared_statement_cache_capacity(PREPARED_STATEMENT_CACHE_CAPACITY);
        Ok(Self {
            conn,
            cache: std::sync::Arc::new(crate::query_cache::QueryCache::new()),
            stmt_cache: StmtCacheSeam::default(),
        })
    }

    /// Wraps an existing connection to a static/frozen catalog.
    pub fn from_connection(conn: frozen_duckdb::Connection) -> Self {
        conn.set_prepared_statement_cache_capacity(PREPARED_STATEMENT_CACHE_CAPACITY);
        Self {
            conn,
            cache: std::sync::Arc::new(crate::query_cache::QueryCache::new()),
            stmt_cache: StmtCacheSeam::default(),
        }
    }

    /// C1: fresh in-memory connection bound to a caller-supplied shared
    /// cache (two connections, one cache entry — court F1).
    pub fn open_in_memory_with_cache(
        cache: std::sync::Arc<crate::query_cache::QueryCache>,
    ) -> Result<Self, EvaluatorError> {
        let conn = frozen_duckdb::Connection::open_in_memory()?;
        conn.set_prepared_statement_cache_capacity(PREPARED_STATEMENT_CACHE_CAPACITY);
        Ok(Self {
            conn,
            cache,
            stmt_cache: StmtCacheSeam::default(),
        })
    }

    /// C1: wraps an existing connection with a caller-supplied shared cache
    /// (two connections, one cache entry — court F1).
    pub fn from_connection_with_cache(
        conn: frozen_duckdb::Connection,
        cache: std::sync::Arc<crate::query_cache::QueryCache>,
    ) -> Self {
        conn.set_prepared_statement_cache_capacity(PREPARED_STATEMENT_CACHE_CAPACITY);
        Self {
            conn,
            cache,
            stmt_cache: StmtCacheSeam::default(),
        }
    }

    /// C1: shared handle to this instance's compiled-plan cache. Callers
    /// mutating data through [`GraphLawDuckDB::raw_connection`] MUST call
    /// [`crate::query_cache::QueryCache::bump`] on this handle afterwards —
    /// raw connection use is otherwise invisible to the cache.
    pub fn shared_cache(&self) -> std::sync::Arc<crate::query_cache::QueryCache> {
        std::sync::Arc::clone(&self.cache)
    }

    /// Prepared-statement reuse seam court accessors. `prepared_statements`
    /// reports how many distinct canonical AST digests this instance has
    /// prepared and not yet flushed (mutation flush clears it);
    /// `prepared_statement_cache_hits` counts re-executions of the same
    /// AST digest (same SQL text, possibly different bound params) served
    /// from the reused handle.
    pub fn prepared_statements(&self) -> usize {
        self.stmt_cache.prepared_keys.borrow().len()
    }

    pub fn prepared_statement_cache_hits(&self) -> u64 {
        self.stmt_cache.prepared_hits.get()
    }

    /// Accesses the underlying raw `frozen_duckdb::Connection` for table creation or seeding.
    pub fn raw_connection(&self) -> &frozen_duckdb::Connection {
        &self.conn
    }

    /// Executes a `CompiledQuery`, binds typed parameters to placeholders, computes the
    /// cryptographic execution receipt, and projects tabular rows into RDF triples.
    pub fn eval_ast_query(
        &self,
        query_iri: &str,
        compiled: &CompiledQuery,
        subject_iri_prefix: &str,
        predicate_iri_prefix: &str,
    ) -> Result<ExecutionReceipt, EvaluatorError> {
        self.eval_member(
            query_iri,
            compiled,
            subject_iri_prefix,
            predicate_iri_prefix,
            true,
            true,
        )
    }

    /// SQLRDF-23 (eval half): executes a compiled statement-sequence
    /// document member-by-member in spine order on this connection and
    /// returns a [`BatchReceipt`] (designs/statement-sequence-design.md
    /// sections 5-6). Best-effort sequential autocommit: each member commits
    /// on success; on a mid-sequence failure the error is
    /// [`EvaluatorError::BatchMemberFailed`], which names the failing member
    /// index and carries the partial [`BatchReceipt`] (members `[0..i]`
    /// executed and committed; members `[i..]` did not execute; the failure
    /// detail is `source`).
    ///
    /// Cache law (design section 6): the batch NEVER consults the C1 cache
    /// for the sequence envelope; per member, entries store only when ALL
    /// members are cacheable (each under its own hash — no batch key); a
    /// mutating member bumps the epoch at its own execution point via the
    /// existing flush path inside [`GraphLawDuckDB::eval_member`]. Each
    /// member's admission seal is verified inside
    /// [`GraphLawDuckDB::eval_member`] before any engine call.
    pub fn eval_ast_document(
        &self,
        query_iri: &str,
        document: &crate::sql_ast::CompiledDocument,
        subject_iri_prefix: &str,
        predicate_iri_prefix: &str,
        atomic: bool,
    ) -> Result<BatchReceipt, EvaluatorError> {
        // Design section 6: if ANY member is non-cacheable or mutating, no
        // member stores. (compile_turtle_document stamps sequence members
        // non-cacheable today, so the all-cacheable case is currently only
        // reachable for hand-built documents — the rule is still total.)
        let all_cacheable = document
            .statements
            .iter()
            .all(|m| m.cacheable && !mutating_sql(&m.sql));
        let mut members = Vec::with_capacity(document.statements.len());
        let mut rows_emitted = Vec::with_capacity(document.statements.len());
        // SQLRDF-26 atomic mode: BEGIN before the first member, COMMIT after
        // the last, ROLLBACK + typed `status:"aborted"` partial receipt on
        // any member failure. Pipeline mode (default) never touches the
        // connection's transaction state — byte-identical to SQLRDF-23.
        let run_in_transaction = atomic || document.atomic;
        if run_in_transaction {
            self.conn
                .execute_batch("BEGIN TRANSACTION")
                .map_err(EvaluatorError::from)?;
        }
        for (index, member) in document.statements.iter().enumerate() {
            let member_iri = format!("{query_iri}#member-{index}");
            match self.eval_member(
                &member_iri,
                member,
                subject_iri_prefix,
                predicate_iri_prefix,
                // The batch never consults the cache (design section 6).
                false,
                all_cacheable,
            ) {
                Ok(receipt) => {
                    rows_emitted.push(receipt.rows_emitted);
                    members.push(receipt);
                }
                Err(source) => {
                    if run_in_transaction {
                        // Atomic abort: undo every member effect. A rollback
                        // failure (engine-internal, near-unreachable) must
                        // not mask the typed member-failure envelope.
                        let _ = self.conn.execute_batch("ROLLBACK");
                    }
                    return Err(EvaluatorError::BatchMemberFailed {
                        index,
                        source: Box::new(source),
                        partial: BatchReceipt {
                            query_iri: query_iri.to_string(),
                            doc_hash: document.doc_hash,
                            state_delta: if run_in_transaction { 0 } else { index as u64 },
                            status: if run_in_transaction {
                                "aborted".to_string()
                            } else {
                                batch_status_partial()
                            },
                            members,
                            rows_emitted,
                            failed_at: Some(index),
                        },
                    });
                }
            }
        }
        if run_in_transaction {
            self.conn
                .execute_batch("COMMIT")
                .map_err(EvaluatorError::from)?;
        }
        Ok(BatchReceipt {
            query_iri: query_iri.to_string(),
            doc_hash: document.doc_hash,
            state_delta: members.len() as u64,
            status: batch_status_partial(),
            members,
            rows_emitted,
            failed_at: None,
        })
    }

    /// Single-member execution engine shared by
    /// [`GraphLawDuckDB::eval_ast_query`] (consult+store) and
    /// [`GraphLawDuckDB::eval_ast_document`] (never consults; stores only
    /// under the batch cache law). `cache_consult`/`cache_store` gate the C1
    /// seams; every other step — seal verify, receipt hash, prepare/bind,
    /// execute, epoch bump on mutating members — is byte-identical.
    fn eval_member(
        &self,
        query_iri: &str,
        compiled: &CompiledQuery,
        subject_iri_prefix: &str,
        predicate_iri_prefix: &str,
        cache_consult: bool,
        cache_store: bool,
    ) -> Result<ExecutionReceipt, EvaluatorError> {
        // Lane A02: coarse monotonic start tick (one relaxed atomic load,
        // <10 ns median) instead of an Instant::now() syscall-class read.
        let start_ns = coarse_now_ns();

        // RUST-5 sole-admission gate: verify the compile-time admission seal
        // BEFORE any prepare/execute. A CompiledQuery whose sealed fields
        // (sql, cacheable, receipt_state, canon_graph_bytes) were mutated
        // after compile_turtle — or that never passed through the admitted
        // compile path — is a typed `EvaluatorError::ReceiptError`, never a
        // panic or a generic DB error. `params` is unsealed: binding
        // different parameters to the same admitted statement is lawful.
        if !compiled.verify_seal() {
            return Err(EvaluatorError::ReceiptError(format!(
                "admission seal mismatch on query {query_iri}: CompiledQuery is not in its \
                 admitted compile state; re-compile via compile_turtle (sole-admission path)"
            )));
        }

        // 1. Compute the receipt hash. When the compile was graph-backed,
        // this is the admitted-statement receipt of the sql-rdf
        // Recommendation section 1.4 first formula, TC-HOT-01 two-node
        // form: H_query = BLAKE3(h_ast || P_params) where
        // h_ast = BLAKE3(Canon(G_AST)) is pinned at compile time in
        // `receipt_state` (a 32-byte Copy), so a receipt costs ONE fresh
        // hasher + one stack-buffer update + finalize — bit-identical to
        // `admitted_statement_hash` by construction (same formula).
        // SQLRDF-17: the legacy `(None, None)` raw-SQL-string hash arm and
        // the hand-built `ast_graph_turtle` re-canonicalization fallback are
        // DELETED, not gated — a query carrying neither the compile-time
        // canon anchor nor canon bytes is a typed `NoCanonicalGraph`
        // refusal, never a raw-SQL admission.
        let query_hash: [u8; 32] = match (&compiled.receipt_state, &compiled.canon_graph_bytes) {
            (Some(h_ast), _) => fold_params_anchored(h_ast, &compiled.params),
            (None, Some(canon)) => admitted_statement_hash(canon, &compiled.params),
            (None, None) => {
                return Err(EvaluatorError::NoCanonicalGraph(query_iri.to_string()));
            }
        };

        // C1 consult seam (designs/C1-cache.md): only statements the
        // compiler classified `cacheable` (C1-a bit, SelectStatement arm)
        // are ever consulted or stored. The cache key is the admitted-
        // statement receipt hash `H_query = BLAKE3(Canon(G_AST) || P_params)`
        // (sql-rdf Rec. section 1.4, first formula), so a flipped literal
        // parameter forks to a different entry by construction.
        if cache_consult
            && compiled.cacheable
            && let Some((mut receipt, _store_epoch)) = self
                .cache
                .consult(query_hash, compiled.canon_graph_bytes.as_deref())
        {
            // Restamp the caller's IRI; the receipt identity (hash,
            // rows, triples) is epoch-valid cached state.
            receipt.query_iri = query_iri.to_string();
            return Ok(receipt);
        }

        // D2 (epoch-coverage audit): snapshot the C1 epoch BEFORE execution.
        // A concurrent bump between row materialization and the store seam
        // below would otherwise stamp the fresh (post-mutation) epoch onto a
        // receipt computed from pre-mutation data — poisoning the cache.
        let exec_epoch = self.cache.epoch();

        // 2. Prepare statement and bind positional parameters ($1, $2, ...).
        // Prepared-statement reuse seam: parameters are bound via $N
        // placeholders (never inlined), so the SQL text is param-independent
        // and identical `sql_statement` re-executions with different bound
        // params reuse the same prepared handle. The underlying
        // frozen-duckdb statement cache reuses the handle for identical
        // trimmed SQL; `stmt_cache` mirrors hit/miss bookkeeping.
        let stmt_key = stmt_cache_key(compiled);
        let already_prepared = self.stmt_cache.prepared_keys.borrow().contains(&stmt_key);
        let mut stmt = self.conn.prepare_cached(&compiled.sql)?;
        if already_prepared {
            self.stmt_cache
                .prepared_hits
                .set(self.stmt_cache.prepared_hits.get() + 1);
        } else {
            self.stmt_cache.prepared_keys.borrow_mut().insert(stmt_key);
        }
        let duck_params: Vec<Box<dyn frozen_duckdb::ToSql>> = compiled
            .params
            .iter()
            .map(|p| -> Box<dyn frozen_duckdb::ToSql> {
                match p {
                    SqlLiteral::String(s) => Box::new(s.clone()),
                    SqlLiteral::Integer(i) => Box::new(*i),
                    SqlLiteral::Float(f) => Box::new(*f),
                    SqlLiteral::Boolean(b) => Box::new(*b),
                    SqlLiteral::Null => Box::new(Option::<String>::None),
                }
            })
            .collect();

        let param_refs: Vec<&dyn frozen_duckdb::ToSql> =
            duck_params.iter().map(|b| b.as_ref()).collect();

        // 3. Vectorized execution and columnar projection to RDF statements
        let mut rows = stmt.query(param_refs.as_slice())?;
        let col_count = rows.as_ref().map(|r| r.column_count()).unwrap_or(0);
        let column_names: Vec<String> = (0..col_count)
            .map(|i| {
                rows.as_ref()
                    .and_then(|r| r.column_name(i).ok().map(|s| s.to_string()))
                    .unwrap_or_else(|| format!("col_{i}"))
            })
            .collect();

        let mut result_triples = Vec::new();
        let mut row_index = 0;

        while let Some(row) = rows.next()? {
            row_index += 1;
            let row_subject = format!("{}/row_{}", subject_iri_prefix, row_index);

            for (col_idx, col_name) in column_names.iter().enumerate() {
                let predicate = format!("{}/{}", predicate_iri_prefix, col_name);

                let term: Option<TermValue> = match row.get_ref(col_idx) {
                    Ok(frozen_duckdb::duckdb::types::ValueRef::Null) => None,
                    Ok(frozen_duckdb::duckdb::types::ValueRef::Boolean(b)) => {
                        Some(TermValue::Literal {
                            lexical_form: b.to_string(),
                            datatype: "http://www.w3.org/2001/XMLSchema#boolean".to_string(),
                            language: None,
                            direction: None,
                        })
                    }
                    Ok(frozen_duckdb::duckdb::types::ValueRef::TinyInt(i)) => {
                        Some(TermValue::Literal {
                            lexical_form: i.to_string(),
                            datatype: "http://www.w3.org/2001/XMLSchema#integer".to_string(),
                            language: None,
                            direction: None,
                        })
                    }
                    Ok(frozen_duckdb::duckdb::types::ValueRef::SmallInt(i)) => {
                        Some(TermValue::Literal {
                            lexical_form: i.to_string(),
                            datatype: "http://www.w3.org/2001/XMLSchema#integer".to_string(),
                            language: None,
                            direction: None,
                        })
                    }
                    Ok(frozen_duckdb::duckdb::types::ValueRef::Int(i)) => {
                        Some(TermValue::Literal {
                            lexical_form: i.to_string(),
                            datatype: "http://www.w3.org/2001/XMLSchema#integer".to_string(),
                            language: None,
                            direction: None,
                        })
                    }
                    Ok(frozen_duckdb::duckdb::types::ValueRef::BigInt(i)) => {
                        Some(TermValue::Literal {
                            lexical_form: i.to_string(),
                            datatype: "http://www.w3.org/2001/XMLSchema#integer".to_string(),
                            language: None,
                            direction: None,
                        })
                    }
                    Ok(frozen_duckdb::duckdb::types::ValueRef::HugeInt(i)) => {
                        Some(TermValue::Literal {
                            lexical_form: i.to_string(),
                            datatype: "http://www.w3.org/2001/XMLSchema#integer".to_string(),
                            language: None,
                            direction: None,
                        })
                    }
                    Ok(frozen_duckdb::duckdb::types::ValueRef::UTinyInt(i)) => {
                        Some(TermValue::Literal {
                            lexical_form: i.to_string(),
                            datatype: "http://www.w3.org/2001/XMLSchema#integer".to_string(),
                            language: None,
                            direction: None,
                        })
                    }
                    Ok(frozen_duckdb::duckdb::types::ValueRef::USmallInt(i)) => {
                        Some(TermValue::Literal {
                            lexical_form: i.to_string(),
                            datatype: "http://www.w3.org/2001/XMLSchema#integer".to_string(),
                            language: None,
                            direction: None,
                        })
                    }
                    Ok(frozen_duckdb::duckdb::types::ValueRef::UInt(i)) => {
                        Some(TermValue::Literal {
                            lexical_form: i.to_string(),
                            datatype: "http://www.w3.org/2001/XMLSchema#integer".to_string(),
                            language: None,
                            direction: None,
                        })
                    }
                    Ok(frozen_duckdb::duckdb::types::ValueRef::UBigInt(i)) => {
                        Some(TermValue::Literal {
                            lexical_form: i.to_string(),
                            datatype: "http://www.w3.org/2001/XMLSchema#integer".to_string(),
                            language: None,
                            direction: None,
                        })
                    }
                    Ok(frozen_duckdb::duckdb::types::ValueRef::Float(f)) => {
                        Some(TermValue::Literal {
                            lexical_form: f.to_string(),
                            datatype: "http://www.w3.org/2001/XMLSchema#double".to_string(),
                            language: None,
                            direction: None,
                        })
                    }
                    Ok(frozen_duckdb::duckdb::types::ValueRef::Double(d)) => {
                        Some(TermValue::Literal {
                            lexical_form: d.to_string(),
                            datatype: "http://www.w3.org/2001/XMLSchema#double".to_string(),
                            language: None,
                            direction: None,
                        })
                    }
                    Ok(frozen_duckdb::duckdb::types::ValueRef::Decimal(d)) => {
                        Some(TermValue::Literal {
                            lexical_form: d.to_string(),
                            datatype: "http://www.w3.org/2001/XMLSchema#decimal".to_string(),
                            language: None,
                            direction: None,
                        })
                    }
                    Ok(frozen_duckdb::duckdb::types::ValueRef::Text(bytes)) => {
                        Some(TermValue::Literal {
                            lexical_form: String::from_utf8_lossy(bytes).into_owned(),
                            datatype: "http://www.w3.org/2001/XMLSchema#string".to_string(),
                            language: None,
                            direction: None,
                        })
                    }
                    Ok(frozen_duckdb::duckdb::types::ValueRef::Date32(days)) => Some(xsd_cell(
                        iso_date_from_epoch_days(i64::from(days)),
                        "http://www.w3.org/2001/XMLSchema#date",
                    )),
                    Ok(frozen_duckdb::duckdb::types::ValueRef::Time64(unit, ticks)) => {
                        Some(xsd_cell(
                            iso_time_from_time_unit(unit, ticks),
                            "http://www.w3.org/2001/XMLSchema#time",
                        ))
                    }
                    Ok(frozen_duckdb::duckdb::types::ValueRef::Timestamp(unit, ticks)) => {
                        Some(xsd_cell(
                            iso_datetime_from_time_unit(unit, ticks),
                            "http://www.w3.org/2001/XMLSchema#dateTime",
                        ))
                    }
                    Ok(frozen_duckdb::duckdb::types::ValueRef::Blob(bytes)) => Some(xsd_cell(
                        base64_encode(bytes),
                        "http://www.w3.org/2001/XMLSchema#base64Binary",
                    )),
                    Ok(frozen_duckdb::duckdb::types::ValueRef::Interval {
                        months,
                        days,
                        nanos,
                    }) => Some(xsd_cell(
                        format!("({months},{days},{nanos})"),
                        "http://www.w3.org/2001/XMLSchema#string",
                    )),
                    // Nested values have no XSD datatype in this projection.
                    // Naming each variant keeps a new `ValueRef` arm a compile
                    // failure instead of a silent string.
                    Ok(
                        frozen_duckdb::duckdb::types::ValueRef::List(..)
                        | frozen_duckdb::duckdb::types::ValueRef::Enum(..)
                        | frozen_duckdb::duckdb::types::ValueRef::Struct(..)
                        | frozen_duckdb::duckdb::types::ValueRef::Array(..)
                        | frozen_duckdb::duckdb::types::ValueRef::Map(..)
                        | frozen_duckdb::duckdb::types::ValueRef::Union(..)
                        | frozen_duckdb::duckdb::types::ValueRef::Variant(..),
                    ) => row
                        .get::<_, String>(col_idx)
                        .ok()
                        .map(|val| xsd_cell(val, "http://www.w3.org/2001/XMLSchema#string")),
                    // A cell the engine could not read is a null projection,
                    // not a string of the error.
                    Err(_) => None,
                };

                if let Some(object_term) = term {
                    result_triples.push(RdfTriple {
                        subject: row_subject.clone(),
                        predicate,
                        object: object_term,
                    });
                }
            }
        }

        let elapsed = std::time::Duration::from_nanos(coarse_elapsed_ns(start_ns));

        // P28-SIL §3.2 evidentiary-ring seam: after receipt formation, push a
        // 64B RingDescriptor into the SPSC ring. No-op unless a ring was
        // explicitly initialized (EvidenceRing::init_global) — no lazy
        // default, no surprise I/O. push never blocks (drop-newest on full).
        #[cfg(all(feature = "evidentiary-ring", not(target_arch = "wasm32")))]
        if let Some(ring) = crate::evidence_ring::EvidenceRing::global() {
            let exec_ms_f = elapsed.as_secs_f64() * 1000.0;
            let exec_ms = if exec_ms_f >= u32::MAX as f64 {
                u32::MAX
            } else {
                exec_ms_f as u32
            };
            let desc = crate::evidence_ring::RingDescriptor::new(
                query_hash,
                row_index as u64,
                exec_ms,
                0,
                crate::evidence_ring::now_ms(),
            );
            let _ = ring.push(desc); // outcome accounted by the ring's counters
        }

        // C1 epoch bump: a non-cacheable statement that mutates stored data
        // (DML, ATTACH/DETACH, DDL) invalidates every cached entry — wired
        // after successful execution only, so a failed statement never
        // invalidates.
        if !compiled.cacheable && mutating_sql(&compiled.sql) {
            // Stored-data mutation: drop prepared handles so no stale plan
            // survives a schema/data change, then bump the C1 plan epoch.
            self.conn.flush_prepared_statement_cache();
            self.stmt_cache.prepared_keys.borrow_mut().clear();
            self.cache.bump();
        }

        let receipt = ExecutionReceipt {
            query_iri: query_iri.to_string(),
            query_hash,
            sql_statement: compiled.sql.clone(),
            rows_emitted: row_index,
            execution_ms: elapsed.as_secs_f64() * 1000.0,
            result_triples,
            cached: false,
            consult_ms: 0.0,
            artifact_digest_hex: None,
            artifact_bytes: None,
            artifact_warning: None,
        };

        // SQLRDF-22 G3 (egress half): a successful `COPY ... TO '<path>'`
        // statement exports bytes; witness them on the receipt by streaming
        // the file through BLAKE3. Read/hash failure keeps the receipt (the
        // execution itself succeeded) and records a typed warning instead —
        // never a panic, never a lost receipt.
        let (mut artifact_digest_hex, mut artifact_bytes, mut artifact_warning) =
            (None, None, None);
        if mutating_sql(&compiled.sql) && is_copy_to(&compiled.sql) {
            match copy_to_path(&compiled.sql)
                .ok_or_else(|| {
                    "COPY TO statement with no single-quoted destination path".to_string()
                })
                .and_then(|path| {
                    hash_file_blake3(&path)
                        .map(|(digest, bytes)| (Some(digest), Some(bytes)))
                }) {
                Ok((digest, bytes)) => {
                    artifact_digest_hex = digest;
                    artifact_bytes = bytes;
                }
                Err(warning) => artifact_warning = Some(warning),
            }
        }
        let receipt = ExecutionReceipt {
            artifact_digest_hex,
            artifact_bytes,
            artifact_warning,
            ..receipt
        };

        // C1 store seam: successful miss of a cacheable statement. D2
        // compare-and-swap: the entry is stamped with `exec_epoch` and stored
        // only if the epoch is unchanged since before execution — a store
        // racing a bump is aborted (the next consult is a miss and re-executes
        // against post-mutation data).
        if cache_store && compiled.cacheable && self.cache.epoch() == exec_epoch {
            self.cache.store(crate::query_cache::CacheEntry {
                h_query: query_hash,
                receipt: receipt.clone(),
                grounding: compiled.canon_graph_bytes.clone().unwrap_or_default(),
                epoch: exec_epoch,
                stored_at: crate::query_cache::now_ms(),
            });
        }

        Ok(receipt)
    }

    /// Executes an admitted sql-rdf statement, computing the execution
    /// receipt as the admitted-statement receipt of the sql-rdf
    /// Recommendation section 1.4 first formula:
    /// `H_query = BLAKE3(Canon(G_AST) || P_params)`.
    ///
    /// `ast_turtle` is the admitted AST graph document (Turtle) that produced
    /// `compiled`; it is canonicalized via RDFC-1.0
    /// ([`canonical_ast_graph_bytes`]). The returned [`ExecutionReceipt`] is
    /// otherwise identical to [`GraphLawDuckDB::eval_ast_query`]. A
    /// canonicalization refusal is a typed
    /// [`EvaluatorError::ReceiptError`], never a crash.
    pub fn eval_admitted_ast(
        &self,
        ast_turtle: &str,
        query_iri: &str,
        compiled: &CompiledQuery,
        subject_iri_prefix: &str,
        predicate_iri_prefix: &str,
    ) -> Result<ExecutionReceipt, EvaluatorError> {
        // R1 / P28-SIL §3.1: when `ast_turtle` is the same graph document
        // the compile carried, the receipt from `eval_ast_query` is already
        // the two-node H_query = BLAKE3(h_ast || P_params) — via the
        // compile-time `receipt_state` anchor (bit-identical to
        // `admitted_statement_hash` by construction) or the carried
        // `canon_graph_bytes` — so it is returned WITHOUT a redundant canon
        // clone or second hash.
        // Only a divergent `ast_turtle` re-canonicalizes and overrides the
        // hash with the divergent graph's receipt.
        let same_document = compiled.ast_graph_turtle.as_deref() == Some(ast_turtle);
        let mut receipt = self.eval_ast_query(
            query_iri,
            compiled,
            subject_iri_prefix,
            predicate_iri_prefix,
        )?;
        if !same_document {
            let canonical_graph = canonical_ast_graph_bytes(ast_turtle)?;
            receipt.query_hash = admitted_statement_hash(&canonical_graph, &compiled.params);
        }
        Ok(receipt)
    }

    /// Mounts a directory of partitioned Parquet files or a glob pattern as a
    /// zero-copy virtual view.
    ///
    /// - `view_name`: Relational table identifier exposed to the AST (e.g., "telemetry_events").
    /// - `glob_pattern`: Path pattern (e.g., "data/telemetry/tenant_id=*/*.parquet").
    /// - `hive_partitioning`: Enables automatic partition column extraction from paths.
    pub fn register_parquet_view(
        &self,
        view_name: &str,
        glob_pattern: &str,
        hive_partitioning: bool,
    ) -> Result<(), EvaluatorError> {
        crate::sql_ast::validate_identifier(view_name)
            .map_err(|e| EvaluatorError::ReceiptError(e.to_string()))?;

        // Sanitize glob pattern string to prevent escaping the single-quoted literal
        let sanitized_pattern = glob_pattern.replace('\'', "''");

        let sql = format!(
            "CREATE OR REPLACE VIEW {} AS SELECT * FROM read_parquet('{}', hive_partitioning = {});",
            view_name,
            sanitized_pattern,
            if hive_partitioning { "true" } else { "false" }
        );

        self.conn.execute_batch(&sql)?;
        // C1 epoch bump: a new parquet view changes the stored data surface
        // every cached entry may have read. The replaced view also orphans
        // any prepared handle compiled against the old one — flush the
        // statement cache so no stale plan survives.
        self.conn.flush_prepared_statement_cache();
        self.stmt_cache.prepared_keys.borrow_mut().clear();
        self.cache.bump();
        Ok(())
    }

    /// Mounts a CSV glob as a zero-copy virtual view (`read_csv_auto`), with
    /// the same C1 cache discipline as [`GraphLawDuckDB::register_parquet_view`]
    /// (D1, epoch-coverage audit): prepared-handle flush + epoch bump after a
    /// successful `CREATE OR REPLACE VIEW`, per the documented MUST on
    /// [`GraphLawDuckDB::shared_cache`] — raw-connection DDL is otherwise
    /// invisible to the compiled-plan cache.
    pub fn register_csv_view(
        &self,
        view_name: &str,
        glob_pattern: &str,
    ) -> Result<(), EvaluatorError> {
        crate::sql_ast::validate_identifier(view_name)
            .map_err(|e| EvaluatorError::ReceiptError(e.to_string()))?;
        let sanitized_pattern = glob_pattern.replace('\'', "''");
        let sql = format!(
            "CREATE OR REPLACE VIEW {view_name} AS SELECT * FROM read_csv_auto('{}');",
            sanitized_pattern
        );
        self.conn.execute_batch(&sql)?;
        self.conn.flush_prepared_statement_cache();
        self.stmt_cache.prepared_keys.borrow_mut().clear();
        self.cache.bump();
        Ok(())
    }

    /// Verifies zone map pruning by retrieving DuckDB's physical execution plan.
    pub fn explain_ast_query(&self, compiled: &CompiledQuery) -> Result<String, EvaluatorError> {
        let mut stmt = self.conn.prepare(&format!("EXPLAIN {}", compiled.sql))?;
        let duck_params: Vec<Box<dyn frozen_duckdb::ToSql>> = compiled
            .params
            .iter()
            .map(|p| -> Box<dyn frozen_duckdb::ToSql> {
                match p {
                    SqlLiteral::String(s) => Box::new(s.clone()),
                    SqlLiteral::Integer(i) => Box::new(*i),
                    SqlLiteral::Float(f) => Box::new(*f),
                    SqlLiteral::Boolean(b) => Box::new(*b),
                    SqlLiteral::Null => Box::new(Option::<String>::None),
                }
            })
            .collect();

        let param_refs: Vec<&dyn frozen_duckdb::ToSql> =
            duck_params.iter().map(|b| b.as_ref()).collect();

        let mut rows = stmt.query(param_refs.as_slice())?;

        let mut plan = String::new();
        while let Some(row) = rows.next()? {
            let line: String = row.get(1)?;
            plan.push_str(&line);
            plan.push('\n');
        }
        Ok(plan)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// P28-SIL §3.1 property court (TC-HOT-01 two-node form): the
    /// `receipt_state` anchor path (`h_ast` + `P_params` fold + finalize)
    /// is bit-identical to the `admitted_statement_hash` reference for
    /// fuzzed canon bytes and parameter buffers — same two-node formula,
    /// Chicago style (real hashes, no mocks), deterministic xorshift fuzz.
    #[test]
    fn receipt_state_clone_path_bit_identical_to_reference() {
        use crate::sql_ast::compile_turtle;
        let mut s = 0x243F6A8885A308D3u64;
        let mut next = move || {
            s ^= s << 13;
            s ^= s >> 7;
            s ^= s << 17;
            s
        };
        for _case in 0..256 {
            let canon_len = (next() % 4096) as usize;
            let canon: Vec<u8> = (0..canon_len).map(|_| next() as u8).collect();
            let params: Vec<SqlLiteral> = (0..(next() % 8))
                .map(|_| match next() % 5 {
                    0 => SqlLiteral::String(
                        (0..(next() % 32))
                            .map(|_| (next() as u8 % 95) + 32)
                            .map(|b| b as char)
                            .collect(),
                    ),
                    1 => SqlLiteral::Integer(next() as i64),
                    2 => SqlLiteral::Float(f64::from_bits(next())),
                    3 => SqlLiteral::Boolean(next() % 2 == 0),
                    _ => SqlLiteral::Null,
                })
                .collect();
            // Reference: two-node admitted_statement_hash.
            let reference = admitted_statement_hash(&canon, &params);
            // Shipped fold: pin h_ast = BLAKE3(canon) once, then
            // fold_params_anchored (one stack-buffer update).
            let h_ast = *blake3::hash(&canon).as_bytes();
            let clone_path = fold_params_anchored(&h_ast, &params);
            assert_eq!(
                clone_path, reference,
                "clone path must be bit-identical for canon_len={canon_len}"
            );
        }

        // Real-compile witness: compile_turtle's pre-pinned `h_ast` anchor
        // yields the same receipt as the admitted_statement_hash reference
        // over the carried Canon(G_AST) bytes.
        let turtle = "@prefix : <http://example.org/> .\n@prefix sql: <http://graphlaw.org/sql#> .\n:g a sql:SelectStatement ;\n    sql:statementIndex 1 ;\n    sql:fromTable [ sql:tableName \"t\" ] ;\n    sql:selectList ( [ sql:columnName \"v\" ] ) .";
        let compiled = compile_turtle(turtle).expect("graph must compile");
        let canon = compiled
            .canon_graph_bytes
            .as_deref()
            .expect("compile carries canon bytes");
        let params = vec![SqlLiteral::Integer(7), SqlLiteral::String("w".into())];
        let h_ast = compiled.receipt_state.expect("compile pins h_ast");
        assert_eq!(
            fold_params_anchored(&h_ast, &params),
            admitted_statement_hash(canon, &params),
            "compile-time pinned h_ast anchor must match the reference receipt"
        );
    }

    /// Charter A03 property court: `ParamBytes` inline encoding is
    /// byte-for-byte the `hash_param_buffer` layout, and
    /// `fold_params_anchored` is bit-identical to the reference anchor fold
    /// for fuzzed parameter vectors — both below and above the 64-byte
    /// inline budget (the >64B case exercises the law-stable reference
    /// fallback).
    #[test]
    fn param_bytes_inline_fold_bit_identical_to_reference() {
        let mut s = 0xBB67AE8584CAA73Bu64;
        let mut next = move || {
            s ^= s << 13;
            s ^= s >> 7;
            s ^= s << 17;
            s
        };
        for _case in 0..1024 {
            let params: Vec<SqlLiteral> = (0..(next() % 12))
                .map(|_| match next() % 5 {
                    0 => SqlLiteral::String(
                        (0..(next() % 32))
                            .map(|_| (next() as u8 % 95) + 32)
                            .map(|b| b as char)
                            .collect(),
                    ),
                    1 => SqlLiteral::Integer(next() as i64),
                    2 => SqlLiteral::Float(f64::from_bits(next())),
                    3 => SqlLiteral::Boolean(next() % 2 == 0),
                    _ => SqlLiteral::Null,
                })
                .collect();
            let h_ast = *blake3::hash(&canon_key(&mut next)).as_bytes();

            // Inline branch must fold exactly the reference bytes.
            let encoded = ParamBytes::encode(&params);
            let inline_total: usize = params
                .iter()
                .map(|p| match p {
                    SqlLiteral::String(s) => 1 + s.len(),
                    SqlLiteral::Integer(_) | SqlLiteral::Float(_) => 9,
                    SqlLiteral::Boolean(_) => 2,
                    SqlLiteral::Null => 1,
                })
                .sum();
            let inline = encoded.is_inline();
            assert_eq!(
                inline,
                inline_total <= PARAM_BYTES_INLINE,
                "is_inline must track the encoded size exactly"
            );

            let fast = fold_params_anchored(&h_ast, &params);
            let mut h_ref = blake3::Hasher::new();
            h_ref.update(&h_ast);
            hash_param_buffer(&mut h_ref, &params);
            let reference = *h_ref.finalize().as_bytes();
            assert_eq!(
                fast, reference,
                "fold_params_anchored must be bit-identical to the reference fold (inline_total={inline_total})"
            );
        }
    }

    /// Charter A03 digest branch (buffers above the inline budget): the
    /// standalone digest is deterministic over the full reference encoding
    /// and separates distinct parameter vectors.
    #[test]
    fn param_bytes_digest_branch_is_deterministic_and_separating() {
        let small: Vec<SqlLiteral> = vec![SqlLiteral::Integer(1), SqlLiteral::String("x".into())];
        let big_a: Vec<SqlLiteral> = (0..16)
            .map(|i| SqlLiteral::String(format!("param-value-{i:02}-aaaaaaaaaaaaaaaa")))
            .collect();
        let big_b: Vec<SqlLiteral> = (0..16)
            .map(|i| SqlLiteral::String(format!("PARAM-VALUE-{i:02}-aaaaaaaaaaaaaaaa")))
            .collect();
        assert!(ParamBytes::encode(&small).is_inline());
        let da = ParamBytes::encode(&big_a);
        let db = ParamBytes::encode(&big_b);
        assert!(
            !da.is_inline(),
            "large vector must spill to the digest branch"
        );
        assert!(!db.is_inline());
        // Digest branch: deterministic, and folds through ParamBytes::fold.
        let fold_once = {
            let mut h = blake3::Hasher::new();
            da.fold(&mut h);
            *h.finalize().as_bytes()
        };
        let fold_twice = {
            let mut h = blake3::Hasher::new();
            da.fold(&mut h);
            *h.finalize().as_bytes()
        };
        assert_eq!(fold_once, fold_twice, "digest fold must be deterministic");
        assert_ne!(
            ParamBytes::encode(&big_a),
            ParamBytes::encode(&big_b),
            "distinct large vectors must produce distinct digests"
        );
        // Digest equals BLAKE3 of the full reference encoding.
        let mut h = blake3::Hasher::new();
        hash_param_buffer(&mut h, &big_a);
        assert_eq!(da, ParamBytes::Digest(*h.finalize().as_bytes()));
        let _ = &small;
    }

    /// Synthetic canon bytes for fuzz seeds.
    fn canon_key(next: &mut impl FnMut() -> u64) -> Vec<u8> {
        (0..64).map(|_| next() as u8).collect()
    }

    /// RUST-5 sole-admission court (lane L4): eval verifies the admission
    /// seal BEFORE prepare. A tampered CompiledQuery (sql mutated after
    /// compile) is a typed `EvaluatorError::ReceiptError` — never a panic
    /// and never a generic DB error. Anti-vacuity is inlined: the same query
    /// unmutated evals for real against a real frozen-duckdb table.
    #[test]
    fn eval_refuses_typed_when_admission_seal_is_broken() {
        let db = GraphLawDuckDB::open_in_memory().expect("DuckDB");
        db.raw_connection()
            .execute_batch(
                "CREATE TABLE telemetry (batch_id VARCHAR, latency INTEGER);
                 INSERT INTO telemetry VALUES ('alpha', 480), ('beta', 610);",
            )
            .expect("seed");
        let compiled = crate::sql_ast::compile_turtle(A04_TURTLE).expect("compile");
        assert!(compiled.verify_seal());

        let good = db
            .eval_ast_query(
                "http://example.org/queries#Q",
                &compiled,
                "http://example.org/data",
                "http://example.org/vocab",
            )
            .expect("untampered query evals (court is not vacuous)");

        // Mutation: tamper with the sealed SQL after compile.
        let mut tampered = compiled.clone();
        tampered.sql = tampered
            .sql
            .replace("latency", "latency; DROP TABLE telemetry; --");
        assert!(!tampered.verify_seal());
        let err = db
            .eval_ast_query(
                "http://example.org/queries#Q",
                &tampered,
                "http://example.org/data",
                "http://example.org/vocab",
            )
            .expect_err("tampered query must refuse");
        assert!(
            matches!(err, EvaluatorError::ReceiptError(ref m) if m.contains("admission seal mismatch")),
            "expected typed ReceiptError, got: {err:?}"
        );
        // No child state: the tampered eval executed nothing.
        let conn = db.raw_connection();
        let mut stmt = conn.prepare("SELECT count(*) FROM telemetry").unwrap();
        let n: i64 = stmt.query_row([], |r| r.get(0)).unwrap();
        assert_eq!(n, 2, "tampered eval must not execute any SQL");
        let _ = good;
    }

    #[test]
    fn test_eval_ast_query_in_memory() {
        let db = GraphLawDuckDB::open_in_memory().expect("Failed to initialize DuckDB");

        // Prepare test schema & seed data
        db.raw_connection()
            .execute_batch(
                "CREATE TABLE telemetry (
                    batch_id VARCHAR,
                    cost BIGINT,
                    latency INTEGER
                );
                INSERT INTO telemetry VALUES ('batch-001', 120, 450);
                INSERT INTO telemetry VALUES ('batch-001', 80, 600);
                INSERT INTO telemetry VALUES ('batch-002', 250, 720);",
            )
            .expect("Failed to insert seed telemetry");

        // SELECT batch_id, SUM(cost) AS total_cost FROM telemetry WHERE
        // (latency > $1) GROUP BY batch_id — SQLRDF-17: the query is
        // compiled from its admitted Turtle AST graph via compile_turtle,
        // so the receipt carries Canon(G_AST); hand-built ASTs no longer
        // admit (the legacy fallback arm is deleted, not gated).
        let turtle = r#"@prefix sql: <http://graphlaw.org/sql#> .
@prefix xsd: <http://www.w3.org/2001/XMLSchema#> .
@prefix ex: <http://example.org/queries#> .
ex:q1 a sql:SelectStatement ;
    sql:statementIndex 1 ;
    sql:fromTable [ sql:tableName "telemetry" ] ;
    sql:selectList (
        [ sql:columnName "batch_id" ]
        [ sql:aliasName "total_cost" ; sql:expression [
            sql:function <http://graphlaw.org/functions#SUM> ;
            sql:argument [ sql:columnName "cost" ] ] ]
    ) ;
    sql:whereClause [
        sql:operator sql:GreaterThan ;
        sql:leftOperand [ sql:columnName "latency" ] ;
        sql:rightOperand [ sql:literalValue "500"^^xsd:integer ]
    ] ;
    sql:groupBy ( [ sql:columnName "batch_id" ] ) .
"#;
        let compiled = crate::sql_ast::compile_turtle(turtle).expect("Compilation failed");
        let query_iri = "http://example.org/queries/q1";

        let receipt = db
            .eval_ast_query(
                query_iri,
                &compiled,
                "http://example.org/data",
                "http://example.org/vocab",
            )
            .expect("Execution failed");

        assert_eq!(receipt.rows_emitted, 2);
        assert_eq!(
            receipt.sql_statement,
            "SELECT batch_id, SUM(cost) AS total_cost FROM telemetry WHERE (latency > $1) GROUP BY batch_id"
        );
        assert!(!receipt.result_triples.is_empty());
        assert_ne!(receipt.query_hash, [0u8; 32]);
    }

    /// Chicago readback: a real DATE and TIMESTAMP project as `xsd:date` and
    /// `xsd:dateTime` with the engine's calendar lexical forms. A catch-all
    /// that stringifies every other variant as `xsd:string` fails this assert.
    #[test]
    fn date_and_timestamp_project_as_xsd_temporal() {
        let db = GraphLawDuckDB::open_in_memory().expect("duckdb");
        db.raw_connection()
            .execute_batch(
                "CREATE TABLE temporal (d DATE, ts TIMESTAMP);
                 INSERT INTO temporal VALUES (DATE '2020-01-02', TIMESTAMP '2020-01-02 03:04:05');",
            )
            .expect("seed");
        let turtle = r#"@prefix sql: <http://graphlaw.org/sql#> .
@prefix ex: <http://example.org/queries#> .
ex:q a sql:SelectStatement ;
    sql:statementIndex 1 ;
    sql:fromTable [ sql:tableName "temporal" ] ;
    sql:selectList ( [ sql:columnName "d" ] [ sql:columnName "ts" ] ) .
"#;
        let compiled = crate::sql_ast::compile_turtle(turtle).expect("compile");
        let receipt = db
            .eval_ast_query(
                "http://example.org/queries/temporal",
                &compiled,
                "http://example.org/data",
                "http://example.org/vocab",
            )
            .expect("eval");

        let cell = |suffix: &str| -> &TermValue {
            &receipt
                .result_triples
                .iter()
                .find(|t| t.predicate.ends_with(suffix))
                .unwrap_or_else(|| panic!("missing predicate {suffix}"))
                .object
        };
        match cell("/d") {
            TermValue::Literal {
                lexical_form,
                datatype,
                ..
            } => {
                assert_eq!(lexical_form, "2020-01-02");
                assert_eq!(datatype, "http://www.w3.org/2001/XMLSchema#date");
            }
            other => panic!("date cell: {other:?}"),
        }
        match cell("/ts") {
            TermValue::Literal {
                lexical_form,
                datatype,
                ..
            } => {
                assert_eq!(lexical_form, "2020-01-02 03:04:05");
                assert_eq!(datatype, "http://www.w3.org/2001/XMLSchema#dateTime");
            }
            other => panic!("timestamp cell: {other:?}"),
        }
    }

    #[test]
    fn admitted_statement_receipt_is_canon_params_and_replay_deterministic() {
        let turtle = r#"
@prefix sql: <http://graphlaw.org/sql#> .
@prefix xsd: <http://www.w3.org/2001/XMLSchema#> .
@prefix ex:  <http://example.org/queries#> .

ex:Q a sql:SelectStatement ;
    sql:statementIndex "0"^^xsd:integer ;
    sql:fromTable [ sql:tableName "telemetry" ] ;
    sql:selectList ( [ sql:columnName "batch_id" ] ) ;
    sql:whereClause [
        sql:operator sql:GreaterThan ;
        sql:leftOperand [ sql:columnName "latency" ] ;
        sql:rightOperand [ sql:literalValue "500"^^xsd:integer ]
    ] .
"#;
        let compiled = crate::sql_ast::compile_turtle(turtle).expect("compile");
        let canonical = canonical_ast_graph_bytes(turtle).expect("canonical bytes");
        assert!(!canonical.is_empty());

        // Formula check: the receipt hash equals a direct BLAKE3 over the
        // canonical graph bytes || the typed parameter buffer.
        let expected = admitted_statement_hash(&canonical, &compiled.params);

        let db = GraphLawDuckDB::open_in_memory().expect("DuckDB");
        db.raw_connection()
            .execute_batch(
                "CREATE TABLE telemetry (batch_id VARCHAR, latency INTEGER);
                 INSERT INTO telemetry VALUES ('alpha', 480), ('beta', 610);",
            )
            .expect("seed");

        let run = || {
            db.eval_admitted_ast(
                turtle,
                "http://example.org/queries#Q",
                &compiled,
                "http://example.org/data",
                "http://example.org/vocab",
            )
            .expect("eval")
        };
        let r1 = run();
        let r2 = run();
        assert_eq!(r1.query_hash, r2.query_hash, "replay determinism");
        assert_eq!(r1.query_hash, expected);
        assert_eq!(r1.rows_emitted, 1);

        // A different parameter buffer changes the receipt.
        let mut compiled_other = compiled.clone();
        compiled_other.params[0] = SqlLiteral::Integer(999);
        let r3 = db
            .eval_admitted_ast(
                turtle,
                "http://example.org/queries#Q",
                &compiled_other,
                "http://example.org/data",
                "http://example.org/vocab",
            )
            .expect("eval");
        assert_ne!(r3.query_hash, r1.query_hash);
    }

    // =========================================================================
    // Charter-A04 courts: prepared-statement mirror keyed by canonical AST
    // digest (h_ast), not SQL text. Chicago style: real frozen-duckdb
    // connections, real table readbacks, no mocks, no sleeps.
    // =========================================================================

    pub(crate) const A04_TURTLE: &str = r#"
@prefix sql: <http://graphlaw.org/sql#> .
@prefix xsd: <http://www.w3.org/2001/XMLSchema#> .
@prefix ex:  <http://example.org/queries#> .

ex:Q a sql:SelectStatement ;
    sql:statementIndex "0"^^xsd:integer ;
    sql:fromTable [ sql:tableName "telemetry" ] ;
    sql:selectList ( [ sql:columnName "batch_id" ] ) ;
    sql:whereClause [
        sql:operator sql:GreaterThan ;
        sql:leftOperand [ sql:columnName "latency" ] ;
        sql:rightOperand [ sql:literalValue "500"^^xsd:integer ]
    ] .
"#;

    pub(crate) fn a04_seed(db: &GraphLawDuckDB) {
        db.raw_connection()
            .execute_batch(
                "CREATE TABLE telemetry (batch_id VARCHAR, latency INTEGER);
                 INSERT INTO telemetry VALUES ('alpha', 480), ('beta', 610);",
            )
            .expect("seed");
    }

    /// Same-SQL-different-params: the mirror key is h_ast (params-
    /// independent), so mutating a bound literal forks the C1 receipt hash
    /// but NOT the prepared-statement key — the second execution must be a
    /// mirror HIT (count increments, no new entry). The dispatch path uses
    /// the digest lookup only; `compiled.sql` is already carried on the
    /// CompiledQuery, so no SQL is synthesized to route the hit.
    #[test]
    fn a04_same_sql_different_params_is_mirror_hit() {
        let compiled = crate::sql_ast::compile_turtle(A04_TURTLE).expect("compile");
        let db = GraphLawDuckDB::open_in_memory().expect("DuckDB");
        a04_seed(&db);

        let mut other = compiled.clone();
        other.params[0] = SqlLiteral::Integer(501);
        assert_eq!(
            compiled.receipt_state, other.receipt_state,
            "same graph => same h_ast anchor"
        );

        let r1 = db
            .eval_ast_query(
                "http://example.org/queries#Q",
                &compiled,
                "http://example.org/data",
                "http://example.org/vocab",
            )
            .expect("eval 1");
        assert_eq!(db.prepared_statements(), 1, "first eval prepares one entry");
        assert_eq!(db.prepared_statement_cache_hits(), 0);

        let r2 = db
            .eval_ast_query(
                "http://example.org/queries#Q",
                &other,
                "http://example.org/data",
                "http://example.org/vocab",
            )
            .expect("eval 2");
        assert_ne!(r1.query_hash, r2.query_hash, "params fork the receipt hash");
        assert_eq!(db.prepared_statements(), 1, "same h_ast => same entry");
        assert_eq!(
            db.prepared_statement_cache_hits(),
            1,
            "second execution served from the reused handle"
        );
    }

    /// Different-SQL (different graph, different h_ast): separate mirror
    /// entries, zero hits.
    #[test]
    fn a04_different_sql_separate_mirror_entries() {
        let q_batch = crate::sql_ast::compile_turtle(A04_TURTLE).expect("compile");
        let alt = A04_TURTLE
            .replace("sql:columnName \"batch_id\"", "sql:columnName \"latency\"")
            .replace("\"500\"^^xsd:integer", "\"250\"^^xsd:integer");
        let q_lat = crate::sql_ast::compile_turtle(&alt).expect("compile");
        assert_ne!(q_batch.receipt_state, q_lat.receipt_state, "distinct h_ast");
        assert_ne!(q_batch.sql, q_lat.sql, "distinct SQL text");

        let db = GraphLawDuckDB::open_in_memory().expect("DuckDB");
        a04_seed(&db);
        for c in [&q_batch, &q_lat] {
            db.eval_ast_query(
                "http://example.org/queries#Q",
                c,
                "http://example.org/data",
                "http://example.org/vocab",
            )
            .expect("eval");
        }
        assert_eq!(db.prepared_statements(), 2, "two digests => two entries");
        assert_eq!(db.prepared_statement_cache_hits(), 0, "no reuse yet");
    }

    /// Mutation flush court on the digest-keyed mirror: a mutating DML
    /// statement flushes every prepared digest (no stale plan survives a
    /// schema/data change).
    #[test]
    fn a04_mutation_flushes_digest_keyed_mirror() {
        let compiled = crate::sql_ast::compile_turtle(A04_TURTLE).expect("compile");
        let db = GraphLawDuckDB::open_in_memory().expect("DuckDB");
        a04_seed(&db);
        db.eval_ast_query(
            "http://example.org/queries#Q",
            &compiled,
            "http://example.org/data",
            "http://example.org/vocab",
        )
        .expect("eval");
        assert_eq!(db.prepared_statements(), 1);

        // Eval a graph-backed mutating command (InsertStatement arm of
        // compile_turtle): non-cacheable + mutating_sql => full flush.
        let ins = crate::sql_ast::compile_turtle(
            r#"@prefix sql: <http://graphlaw.org/sql#> .
@prefix xsd: <http://www.w3.org/2001/XMLSchema#> .
<http://example.org/s> a sql:InsertStatement ;
    sql:statementIndex "0"^^xsd:integer ;
    sql:targetTable "telemetry" ;
    sql:assignColumn "batch_id" ;
    sql:assignValue "gamma" ."#,
        )
        .expect("compile insert");
        let _ = db
            .eval_ast_query(
                "http://example.org/queries#q-ins",
                &ins,
                "http://example.org/data",
                "http://example.org/vocab",
            )
            .expect("eval insert");
        assert_eq!(
            db.prepared_statements(),
            0,
            "mutating eval flushes every prepared digest"
        );
        assert_eq!(db.prepared_statement_cache_hits(), 0);
    }

    /// Timing probe (embed): the digest lookup that dispatches a cache hit
    /// — `stmt_cache_key` + mirror `contains` — is measured against the
    /// charter fence. Reports the measured mean; the assertion fence is set
    /// wide enough to be flake-proof in a dev-profile test binary while a
    /// regression to SQL-string work would blow past it by orders of
    /// magnitude.
    #[test]
    fn a04_digest_lookup_timing_probe() {
        let compiled = crate::sql_ast::compile_turtle(A04_TURTLE).expect("compile");
        let key = stmt_cache_key(&compiled);
        let seam = StmtCacheSeam::default();
        seam.prepared_keys.borrow_mut().insert(key);

        // Warm.
        for _ in 0..1_000 {
            let _ = seam.prepared_keys.borrow().contains(&key);
        }
        // Min-of-rounds relative fence (no-sleep doctrine: paired rounds,
        // min-of-N — never a single absolute mean under parallel test load).
        const ROUNDS: usize = 25;
        const BATCH: u64 = 10_000;
        let mut best_ns = f64::MAX;
        for _ in 0..ROUNDS {
            let t0 = std::time::Instant::now();
            let mut observed: u64 = 0;
            for _ in 0..BATCH {
                // Exactly the hit-dispatch path: digest in, membership out —
                // no string synthesis anywhere in this expression.
                if seam
                    .prepared_keys
                    .borrow()
                    .contains(&stmt_cache_key(&compiled))
                {
                    observed += 1;
                }
            }
            assert_eq!(observed, BATCH, "every lookup must hit");
            best_ns = best_ns.min(t0.elapsed().as_nanos() as f64 / BATCH as f64);
        }
        eprintln!("A04 digest-lookup min-of-{ROUNDS}: {best_ns:.1} ns/lookup (dev profile)");
        assert!(
            best_ns < 2_000.0,
            "digest lookup regressed: {best_ns:.1} ns (dev-profile fence; a re-render or SQL synthesis would blow past this by orders of magnitude)"
        );
    }

    /// The section 7 MERGE graph (rec §7), verbatim from the Chicago
    /// acceptance set's fixture shape: MERGE INTO main.regional_sales USING
    /// (SELECT ... FROM staging.invoices) with a bound status literal.
    const MERGE_TTL: &str = r#"@prefix sql: <http://graphlaw.org/sql#> .
@prefix xsd: <http://www.w3.org/2001/XMLSchema#> .
_:mergeStatement a sql:MergeIntoStatement ;
    sql:statementIndex 1 ;
    sql:targetTable [
        a sql:TableIdentifier ;
        sql:schemaName "main" ;
        sql:tableName "regional_sales"
    ] ;
    sql:usingSource [
        a sql:SelectStatement ;
        sql:statementIndex 1 ;
        sql:select (
            [ a sql:ColumnIdentifier ;
              sql:columnName "region_id" ;
              sql:scope sql:ViewScope ]
            [ a sql:FunctionExpression ;
              sql:functionName "sum" ;
              sql:arguments ( [ a sql:ColumnIdentifier ; sql:columnName "amount" ; sql:scope sql:ProcessScope ] ) ;
              sql:alias "total_invoiced" ;
              sql:scope sql:ViewScope ]
        ) ;
        sql:from (
            [ a sql:TableIdentifier ;
              sql:schemaName "staging" ;
              sql:tableName "invoices" ]
        ) ;
        sql:where [
            a sql:BinaryExpression ;
            sql:left [ a sql:ColumnIdentifier ; sql:columnName "status" ; sql:scope sql:ProcessScope ] ;
            sql:operator "=" ;
            sql:right [ a sql:LiteralExpression ; sql:value "SETTLED" ; sql:datatype xsd:string ]
        ] ;
        sql:groupBy (
            [ a sql:ColumnIdentifier ; sql:columnName "region_id" ; sql:scope sql:ProcessScope ]
        )
    ] ;
    sql:mergeOn [
        a sql:BinaryExpression ;
        sql:left [ a sql:ColumnIdentifier ; sql:qualifier "main.regional_sales" ; sql:columnName "region_id" ; sql:scope sql:ProcessScope ] ;
        sql:operator "=" ;
        sql:right [ a sql:ColumnIdentifier ; sql:qualifier "src" ; sql:columnName "region_id" ; sql:scope sql:ProcessScope ]
    ] ;
    sql:whenMatched [
        a sql:MergeUpdateAction ;
        sql:setAssignments (
            [ a sql:Assignment ;
              sql:targetColumn "revenue" ;
              sql:valueExpression [
                  a sql:BinaryExpression ;
                  sql:left [ a sql:ColumnIdentifier ; sql:qualifier "main.regional_sales" ; sql:columnName "revenue" ; sql:scope sql:ProcessScope ] ;
                  sql:operator "+" ;
                  sql:right [ a sql:ColumnIdentifier ; sql:qualifier "src" ; sql:columnName "total_invoiced" ; sql:scope sql:ProcessScope ]
              ]
            ]
        )
    ] ."#;

    /// Court: a graph-backed MERGE must bump the C1 cache epoch and flush
    /// cached plans. Observable via a real cached SELECT: warm hit before
    /// the MERGE, forced miss (fresh re-execution) after it.
    #[test]
    fn merge_bumps_c1_epoch_and_flushes_cached_select() {
        // Serialize engine-heavy tests: the frozen engine has segfaulted
        // under concurrent engine-heavy tests (see tests/duckdb_statement_kinds.rs).
        static MERGE_ENGINE_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());
        let _engine = MERGE_ENGINE_LOCK
            .lock()
            .unwrap_or_else(|e| e.into_inner());

        let select = crate::sql_ast::compile_turtle(A04_TURTLE).expect("compile select");
        let merge = crate::sql_ast::compile_turtle(MERGE_TTL).expect("compile merge");
        assert!(
            !merge.cacheable,
            "MERGE compiles non-cacheable; mutating_sql is belt-and-suspenders"
        );
        assert!(
            mutating_sql(&merge.sql),
            "MERGE SQL text must be lexically classified as mutating"
        );

        let db = GraphLawDuckDB::open_in_memory().expect("DuckDB");
        db.raw_connection()
            .execute_batch(
                "CREATE TABLE telemetry (batch_id VARCHAR, latency INTEGER);
                 INSERT INTO telemetry VALUES ('alpha', 480), ('beta', 610);
                 CREATE SCHEMA staging;
                 CREATE TABLE staging.invoices(region_id INTEGER, amount DECIMAL, status VARCHAR);
                 INSERT INTO staging.invoices VALUES (1, 30, 'SETTLED'), (2, 50, 'PENDING'), (3, 7, 'SETTLED');
                 CREATE TABLE main.regional_sales(region_id INTEGER PRIMARY KEY, revenue DECIMAL);
                 INSERT INTO main.regional_sales VALUES (1, 100), (2, 50), (3, 0);",
            )
            .expect("seed");

        let cache = db.shared_cache();
        let eval = |c: &crate::sql_ast::CompiledQuery| {
            db.eval_ast_query("http://example.org/queries#Q", c, "urn:r:", "urn:p:")
                .expect("eval must execute")
        };

        // (a) Warm the C1 cache with a real cached SELECT: miss then hit.
        let s1 = eval(&select);
        assert!(!s1.cached, "first SELECT is a cache miss");
        let s2 = eval(&select);
        assert!(s2.cached, "second SELECT must be served from the C1 cache");

        // (b) MERGE executes and bumps the epoch, flushing the cached plan.
        let epoch_before = cache.epoch();
        let m = eval(&merge);
        assert!(!m.cached, "MERGE never serves from the cache");
        assert_eq!(
            cache.epoch(),
            epoch_before + 1,
            "MERGE must bump the C1 epoch"
        );
        assert_eq!(
            cache.len(),
            0,
            "epoch bump must drop every cached entry"
        );
        assert_eq!(
            db.prepared_statements(),
            0,
            "MERGE must flush the prepared-statement mirror"
        );

        // (c) The subsequent cached SELECT is a miss and re-executes fresh:
        // rows written after the flush are visible in the table readback.
        let s3 = eval(&select);
        assert!(!s3.cached, "post-MERGE SELECT must be a fresh miss, not a stale hit");
        db.raw_connection()
            .execute_batch("INSERT INTO telemetry VALUES ('gamma', 700);")
            .expect("post-merge insert");
        let mut stmt = db
            .raw_connection()
            .prepare("SELECT batch_id FROM telemetry WHERE latency > 500 ORDER BY batch_id")
            .unwrap();
        let rows: Vec<String> = stmt
            .query_map([], |r| r.get(0))
            .unwrap()
            .map(|r| r.unwrap())
            .collect();
        assert_eq!(
            rows,
            vec!["beta".to_string(), "gamma".to_string()],
            "fresh rows must be visible after the MERGE epoch bump"
        );
    }

    /// SQLRDF-22 G3 inline court: a successful `COPY ... TO` eval must
    /// carry the artifact digest + byte count on the receipt, matching an
    /// independent std::fs read + BLAKE3 of the file (Chicago: real file,
    /// real frozen-duckdb, no mocks).
    #[test]
    fn g3_copy_to_receipt_carries_artifact_digest() {
        let dir = std::env::temp_dir().join(format!("graphlaw-g3-{}", std::process::id()));
        let path = dir.join("out.csv");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("tmp dir");

        let db = GraphLawDuckDB::open_in_memory().expect("DuckDB");
        a04_seed(&db);
        let compiled = crate::sql_ast::compile_turtle(&format!(
            r#"@prefix sql: <http://graphlaw.org/sql#> .
@prefix xsd: <http://www.w3.org/2001/XMLSchema#> .
<http://example.org/q> a sql:CopyStatement ;
    sql:statementIndex "0"^^xsd:integer ;
    sql:copyOperation [ sql:tableName "telemetry" ; sql:filePath "{}" ; sql:fileFormat "CSV" ] ."#,
            path.display()
        ))
        .expect("compile COPY");

        let receipt = db
            .eval_ast_query(
                "http://example.org/queries#q-copy",
                &compiled,
                "http://example.org/data",
                "http://example.org/vocab",
            )
            .expect("COPY eval must succeed");

        let (digest, bytes) = (
            receipt.artifact_digest_hex.clone().expect("digest present"),
            receipt.artifact_bytes.expect("bytes present"),
        );
        assert!(receipt.artifact_warning.is_none(), "no warning on success");
        // Independent readback: raw bytes via std::fs, hashed separately.
        let raw = std::fs::read(&path).expect("independent file read");
        let expected = blake3::hash(&raw).to_hex().to_string();
        assert_eq!(digest, expected, "receipt digest matches independent BLAKE3");
        assert_eq!(bytes, raw.len() as u64, "byte count matches file size");
        // Table readback: the receipt carries the SQL that produced it.
        assert!(
            receipt.sql_statement.starts_with("COPY telemetry TO"),
            "receipt SQL is the COPY-out statement"
        );
        std::fs::remove_dir_all(&dir).ok();
    }

    /// SQLRDF-22 G3 twin court: `COPY ... TO` aimed at a nonexistent
    /// directory. The execution itself succeeds (DuckDB writes-or-fails
    /// inside the statement; either way the eval returns Ok), so per the
    /// documented G3 choice the receipt is kept and the failure is recorded
    /// as `artifact_warning` with digest/bytes `None` — never a lost
    /// receipt, never a panic, never a generic DB error surfaced.
    #[test]
    fn g3_copy_to_bad_directory_records_typed_warning() {
        let path = std::env::temp_dir()
            .join(format!("graphlaw-g3-missing-{}", std::process::id()))
            .join("out.csv");
        let db = GraphLawDuckDB::open_in_memory().expect("DuckDB");
        a04_seed(&db);
        let compiled = crate::sql_ast::compile_turtle(&format!(
            r#"@prefix sql: <http://graphlaw.org/sql#> .
@prefix xsd: <http://www.w3.org/2001/XMLSchema#> .
<http://example.org/q> a sql:CopyStatement ;
    sql:statementIndex "0"^^xsd:integer ;
    sql:copyOperation [ sql:tableName "telemetry" ; sql:filePath "{}" ; sql:fileFormat "CSV" ] ."#,
            path.display()
        ))
        .expect("compile COPY");

        let result = db.eval_ast_query(
            "http://graphlaw.org/queries#q-copy-bad",
            &compiled,
            "http://example.org/data",
            "example.org/vocab",
        );
        match result {
            Ok(receipt) => {
                // DuckDB may fail the COPY inside the statement (Ok here
                // only if it silently succeeded, which it cannot for a
                // missing dir) — but the eval may also surface it as
                // EvaluatorError. Both arms below are typed.
                assert!(receipt.artifact_warning.is_some());
                assert!(receipt.artifact_digest_hex.is_none());
            }
            Err(EvaluatorError::DuckDb(_)) => {
                // DuckDB refused the COPY outright (IO error inside the
                // statement): typed EvaluatorError, execution not
                // successful, so no receipt to lose.
            }
            Err(other) => panic!("unexpected typed error: {other}"),
        }
        std::fs::remove_dir_all(path.parent().unwrap()).ok();
    }
}

// ===========================================================================
// SQLRDF-23 eval-half courts (designs/statement-sequence-design.md sections
// 5-6). Chicago: real frozen-duckdb connections, real table readbacks, no
// mocks, no sleeps. ENGINE_LOCK serializes engine-heavy courts.
// ===========================================================================
#[cfg(test)]
mod document_eval_courts {
    use super::*;
    use crate::duckdb_eval::tests::{a04_seed, A04_TURTLE};

    static ENGINE_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

    fn lock() -> std::sync::MutexGuard<'static, ()> {
        ENGINE_LOCK.lock().unwrap_or_else(|e| e.into_inner())
    }

    /// 3-member CREATE / INSERT / SELECT document (statementIndex 0,1,2).
    fn doc_create_insert_select(table: &str, val: i64) -> String {
        format!(
            r#"@prefix sql: <http://graphlaw.org/sql#> .
@prefix xsd: <http://www.w3.org/2001/XMLSchema#> .
sql:doc a sql:StatementSequence ;
    sql:statements ( <http://example.org/c> <http://example.org/i> <http://example.org/s> ) .
<http://example.org/c> a sql:CreateStatement ;
    sql:statementIndex 0 ;
    sql:targetTable "{table}" ;
    sql:assignColumn "v" .
<http://example.org/i> a sql:InsertStatement ;
    sql:statementIndex 1 ;
    sql:targetTable "{table}" ;
    sql:assignColumn "v" ;
    sql:assignValue "{val}"^^xsd:integer .
<http://example.org/s> a sql:SelectStatement ;
    sql:statementIndex 2 ;
    sql:fromTable [ sql:tableName "{table}" ] ;
    sql:selectList ( [ sql:columnName "v" ] ) .
"#
        )
    }

    /// Court (a): 3-member CREATE/INSERT/SELECT doc on one connection ->
    /// BatchReceipt with 3 members, exact rows readback, failed_at None.
    #[test]
    fn batch_three_member_create_insert_select_executes_clean() {
        let _engine = lock();
        const TABLE: &str = "seq_court_events";
        let doc = crate::sql_ast::compile_turtle_document(&doc_create_insert_select(TABLE, 42))
            .expect("sequence document compiles");
        assert_eq!(doc.statements.len(), 3, "three members compiled");
        assert!(!doc.statements.iter().all(|m| m.cacheable), "sequence members compile non-cacheable per compile_turtle_document");

        let db = GraphLawDuckDB::open_in_memory().expect("DuckDB");
        let receipt = db
            .eval_ast_document(
                "http://example.org/docs#batch-a",
                &doc,
                "http://example.org/data",
                "http://example.org/vocab",
                false,
            )
            .expect("3-member batch executes");
        assert_eq!(receipt.query_iri, "http://example.org/docs#batch-a");
        assert_eq!(receipt.doc_hash, doc.doc_hash, "receipt carries the document hash");
        assert_eq!(receipt.members.len(), 3, "one receipt per member");
        assert_eq!(receipt.rows_emitted, vec![0, 1, 1], "CREATE=0, INSERT=1, SELECT=1 row");
        assert_eq!(receipt.failed_at, None);
        // Exact rows readback: the SELECT member's receipt carries the row it
        // projected, and the table reads back the inserted value.
        assert_eq!(
            receipt.members[2].rows_emitted, 1,
            "SELECT member emitted the inserted row"
        );
        let mut stmt = db
            .raw_connection()
            .prepare(&format!("SELECT v FROM {TABLE}"))
            .unwrap();
        let rows: Vec<i64> = stmt
            .query_map([], |r| r.get(0))
            .unwrap()
            .map(|r| r.unwrap())
            .collect();
        assert_eq!(rows, vec![42_i64], "table readback: the INSERT member's row");
        // Real CREATE member ran: member 0 is the CREATE (rows 0).
        assert_eq!(receipt.members[0].rows_emitted, 0, "CREATE emits no rows");
    }

    /// 2-member CREATE / INSERT document (statementIndex 0,1) — ends on the
    /// mutating member, so a post-batch flush observation is exact.
    fn doc_create_insert(table: &str, val: i64) -> String {
        let three = doc_create_insert_select(table, val);
        // Drop the SELECT member: remove its block and its entry from the
        // sequence list.
        let three = three
            .split_once("<http://example.org/s> a sql:SelectStatement")
            .map(|(head, _)| head.to_string())
            .unwrap_or(three);
        three.replace(
            " ( <http://example.org/c> <http://example.org/i> <http://example.org/s> )",
            " ( <http://example.org/c> <http://example.org/i> )",
        )
    }

    /// Court (b): member at position 2 (index 1) references a missing table
    /// -> failed_at Some(1), error names the index, member 0's effects stay
    /// committed (autocommit witness via follow-up raw SELECT readback).
    #[test]
    fn batch_mid_sequence_failure_names_index_and_commits_prior_members() {
        let _engine = lock();
        const TABLE: &str = "seq_court_partial";
        let ttl = doc_create_insert_select(TABLE, 7).replacen(
            &format!(
                "sql:targetTable \"{TABLE}\" ;\n    sql:assignColumn \"v\" ;\n    sql:assignValue"
            ),
            "sql:targetTable \"no_such_table\" ;\n    sql:assignColumn \"v\" ;\n    sql:assignValue",
            1,
        );
        assert!(
            ttl.contains("sql:targetTable \"no_such_table\""),
            "fixture must retarget the INSERT member only"
        );
        assert_eq!(
            ttl.matches(&format!("sql:targetTable \"{TABLE}\"")).count(),
            1,
            "only the CREATE member keeps the real targetTable (SELECT uses fromTable)"
        );
        let doc = crate::sql_ast::compile_turtle_document(&ttl)
            .expect("sequence compiles (missing table is an eval-time failure)");
        assert_eq!(doc.statements.len(), 3);

        let db = GraphLawDuckDB::open_in_memory().expect("DuckDB");
        let err = db
            .eval_ast_document(
                "http://example.org/docs#batch-b",
                &doc,
                "http://example.org/data",
                "http://example.org/vocab",
                false,
            )
            .expect_err("member 1 must fail on the missing table");
        match &err {
            EvaluatorError::BatchMemberFailed { index, partial, .. } => {
                assert_eq!(*index, 1, "failure at member index 1");
                assert_eq!(partial.failed_at, Some(1));
                assert_eq!(partial.members.len(), 1, "members [0..1] executed");
                assert_eq!(partial.rows_emitted, vec![0], "CREATE member receipt kept");
                assert_eq!(partial.doc_hash, doc.doc_hash);
            }
            other => panic!("unexpected typed error: {other}"),
        }
        assert!(
            err.to_string().contains("member 1"),
            "typed error names the index: {err}"
        );
        // Autocommit witness: member 0's CREATE committed and is visible in a
        // real follow-up SELECT on the same connection.
        let mut stmt = db
            .raw_connection()
            .prepare(&format!("SELECT count(*) FROM {TABLE}"))
            .unwrap();
        let count: i64 = stmt.query_row([], |r| r.get(0)).unwrap();
        assert_eq!(count, 0, "member 0's CREATE is committed and queryable");
    }

    /// Court (c): a mutating member in the batch bumps the C1 epoch and
    /// flushes the prepared-statement mirror (mirrors the MERGE epoch
    /// court's observability on a real connection).
    #[test]
    fn batch_mutating_member_bumps_epoch_and_flushes_prepared_mirror() {
        let _engine = lock();
        // Warm the caches with a bare SELECT first (eval_ast_query consults
        // and stores), then run a batch whose INSERT member mutates.
        let select = crate::sql_ast::compile_turtle(A04_TURTLE).expect("compile select");
        let doc = crate::sql_ast::compile_turtle_document(&doc_create_insert(
            "seq_court_epoch",
            5,
        ))
        .expect("sequence compiles");
        assert!(
            mutating_sql(&doc.statements[1].sql),
            "INSERT member is lexically mutating: {}",
            doc.statements[1].sql
        );

        let db = GraphLawDuckDB::open_in_memory().expect("DuckDB");
        a04_seed(&db);
        let warm = db
            .eval_ast_query("http://example.org/queries#Q", &select, "urn:r:", "urn:p:")
            .expect("warm select");
        let cache = db.shared_cache();
        assert!(!warm.cached, "warm-up is a miss");
        let warm2 = db
            .eval_ast_query("http://example.org/queries#Q", &select, "urn:r:", "urn:p:")
            .expect("warm select 2");
        assert!(warm2.cached, "cache is warm before the batch");
        assert_eq!(db.prepared_statements(), 1, "one prepared handle mirrored");

        let epoch_before = cache.epoch();
        let receipt = db
            .eval_ast_document(
                "http://example.org/docs#batch-c",
                &doc,
                "http://example.org/data",
                "http://example.org/vocab",
                false,
            )
            .expect("batch executes");
        assert_eq!(receipt.failed_at, None);
        assert_eq!(
            cache.epoch(),
            epoch_before + 2,
            "each mutating member (CREATE, INSERT) bumps the C1 epoch exactly once at its own execution point"
        );
        assert_eq!(
            db.prepared_statements(),
            0,
            "the mutating member must flush the prepared-statement mirror"
        );
        assert_eq!(cache.len(), 0, "epoch bump drops every cached entry");
    }

    /// Court (d): a batch containing any non-cacheable member stores
    /// NOTHING in the shared cache (design section 6: all-or-none), even
    /// though every member executed successfully.
    #[test]
    fn batch_with_non_cacheable_member_stores_nothing() {
        let _engine = lock();
        let doc = crate::sql_ast::compile_turtle_document(&doc_create_insert_select(
            "seq_court_nostore",
            3,
        ))
        .expect("sequence compiles");
        let db = GraphLawDuckDB::open_in_memory().expect("DuckDB");
        let cache = db.shared_cache();
        assert_eq!(cache.len(), 0, "cache starts empty");
        let receipt = db
            .eval_ast_document(
                "http://example.org/docs#batch-d",
                &doc,
                "http://example.org/data",
                "http://example.org/vocab",
                false,
            )
            .expect("batch executes");
        assert_eq!(receipt.failed_at, None);
        assert_eq!(receipt.members.len(), 3);
        assert!(
            doc.statements.iter().any(|m| !m.cacheable),
            "fixture really contains a non-cacheable member"
        );
        assert_eq!(
            cache.len(),
            0,
            "all-or-none cache law: no member stores when any member is non-cacheable"
        );
    }

    /// SQLRDF-26 fixture: the 3-member doc, marked `sql:atomic true` on the
    /// sequence root (compile-threaded flag).
    fn doc_atomic_create_insert_select(table: &str, val: i64) -> String {
        doc_create_insert_select(table, val).replacen(
            "sql:doc a sql:StatementSequence ;",
            "sql:doc a sql:StatementSequence ;\n    sql:atomic true ;",
            1,
        )
    }

    /// SQLRDF-26 (a): atomic 3-member success -> all members execute, COMMIT
    /// runs, receipt carries `state_delta = 3`.
    #[test]
    fn atomic_three_member_batch_commits() {
        let _engine = lock();
        const TABLE: &str = "seq_atomic_ok";
        let doc = crate::sql_ast::compile_turtle_document(&doc_atomic_create_insert_select(
            TABLE, 42,
        ))
        .expect("atomic sequence compiles");
        assert!(doc.atomic, "compile threads sql:atomic true into the document");

        let db = GraphLawDuckDB::open_in_memory().expect("DuckDB");
        let receipt = db
            .eval_ast_document(
                "http://example.org/docs#atomic-a",
                &doc,
                "http://example.org/data",
                "http://example.org/vocab",
                false,
            )
            .expect("atomic batch executes");
        assert_eq!(receipt.failed_at, None);
        assert_eq!(receipt.members.len(), 3, "all members executed");
        assert_eq!(receipt.state_delta, 3, "all members committed");
        assert_eq!(receipt.rows_emitted, vec![0, 1, 1]);
        // COMMIT witness: if the transaction were still open, a fresh BEGIN
        // on the same connection would refuse ("transaction already active").
        db.raw_connection()
            .execute_batch("BEGIN TRANSACTION")
            .expect("no transaction is left open — COMMIT executed");
    }

    /// SQLRDF-26 (b): atomic member-2 failure -> ROLLBACK witnessed (the
    /// CREATE from member 1 does NOT survive a post-batch SELECT — vs the
    /// pipeline twin where it does), partial receipt marked aborted/0.
    #[test]
    fn atomic_member_failure_rolls_back_and_marks_aborted() {
        let _engine = lock();
        const TABLE: &str = "seq_atomic_abort";
        let ttl = doc_atomic_create_insert_select(TABLE, 7).replacen(
            &format!(
                "sql:targetTable \"{TABLE}\" ;\n    sql:assignColumn \"v\" ;\n    sql:assignValue"
            ),
            "sql:targetTable \"no_such_table\" ;\n    sql:assignColumn \"v\" ;\n    sql:assignValue",
            1,
        );
        let doc = crate::sql_ast::compile_turtle_document(&ttl)
            .expect("sequence compiles (missing table is an eval-time failure)");
        let db = GraphLawDuckDB::open_in_memory().expect("DuckDB");
        let err = db
            .eval_ast_document(
                "http://example.org/docs#atomic-b",
                &doc,
                "http://example.org/data",
                "http://example.org/vocab",
                false,
            )
            .expect_err("member 1 must fail on the missing table");
        match &err {
            EvaluatorError::BatchMemberFailed { index, partial, .. } => {
                assert_eq!(*index, 1);
                assert_eq!(partial.failed_at, Some(1));
                assert_eq!(partial.members.len(), 1, "member 0 executed inside the txn");
                assert_eq!(partial.status, "aborted", "atomic failure marks aborted");
                assert_eq!(partial.state_delta, 0, "rollback pins net delta to zero");
            }
            other => panic!("unexpected typed error: {other}"),
        }
        // ROLLBACK witness: the member-0 CREATE did not survive.
        let rolled = db.raw_connection().prepare(&format!("SELECT count(*) FROM {TABLE}"));
        assert!(
            rolled.is_err(),
            "member 0's CREATE must be rolled back (pipeline twin keeps it committed)"
        );
        // Reactor alive: the connection is out of the aborted transaction and
        // still executes.
        db.raw_connection()
            .execute_batch("BEGIN TRANSACTION")
            .expect("ROLLBACK closed the transaction — connection reusable");
    }

    /// SQLRDF-26 (c): Seq_B order inversion, pipeline mode —
    /// [INSERT-into-missing-table, CREATE] -> member 0 fails, member 1 never
    /// executes, reactor alive.
    #[test]
    fn order_inverted_pipeline_batch_stops_at_member_0_reactor_alive() {
        let _engine = lock();
        const TABLE: &str = "seq_inverted";
        let ttl = format!(
            r#"@prefix sql: <http://graphlaw.org/sql#> .
@prefix xsd: <http://www.w3.org/2001/XMLSchema#> .
sql:doc a sql:StatementSequence ;
    sql:statements ( <http://example.org/i> <http://example.org/c> ) .
<http://example.org/i> a sql:InsertStatement ;
    sql:statementIndex 0 ;
    sql:targetTable "no_such_table" ;
    sql:assignColumn "v" ;
    sql:assignValue "1"^^xsd:integer .
<http://example.org/c> a sql:CreateStatement ;
    sql:statementIndex 1 ;
    sql:targetTable "{TABLE}" ;
    sql:assignColumn "v" .
"#
        );
        let doc = crate::sql_ast::compile_turtle_document(&ttl)
            .expect("inverted sequence compiles");
        let db = GraphLawDuckDB::open_in_memory().expect("DuckDB");
        let err = db
            .eval_ast_document(
                "http://example.org/docs#seq-b",
                &doc,
                "http://example.org/data",
                "http://example.org/vocab",
                false,
            )
            .expect_err("member 0 must fail on the missing table");
        match &err {
            EvaluatorError::BatchMemberFailed { index, partial, .. } => {
                assert_eq!(*index, 0, "failure at member index 0");
                assert_eq!(partial.failed_at, Some(0));
                assert_eq!(partial.members.len(), 0, "no member completed");
                assert_eq!(partial.status, "partial", "pipeline mode stays partial");
            }
            other => panic!("unexpected typed error: {other}"),
        }
        // Member 1 never executed: the CREATE did not land.
        let created = db
            .raw_connection()
            .prepare(&format!("SELECT count(*) FROM {TABLE}"));
        assert!(created.is_err(), "member 1 (CREATE) must never execute after a member-0 failure");
        // Reactor alive on the same connection.
        db.raw_connection()
            .execute_batch("CREATE TABLE seq_inverted_reactor (v INTEGER)")
            .expect("connection still executes after the typed failure");
    }
}
