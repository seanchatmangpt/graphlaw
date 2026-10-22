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
    if n % COARSE_CALIB_EVERY == 0 {
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
}

impl fmt::Display for EvaluatorError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DuckDb(e) => write!(f, "DuckDB execution failure: {e}"),
            Self::ConversionError { column, row, details } => {
                write!(f, "Column conversion error for '{column}' at row {row}: {details}")
            }
            Self::ReceiptError(msg) => write!(f, "Receipt generation error: {msg}"),
            Self::NoCanonicalGraph(query_iri) => write!(
                f,
                "no canonical graph bytes — hand-built CompiledQuery must use compile_turtle (query {query_iri})"
            ),
        }
    }
}

impl std::error::Error for EvaluatorError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            EvaluatorError::DuckDb(error) => Some(error),
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
}

/// Embedded execution context wrapping a sovereign `frozen-duckdb` connection.
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
#[derive(Default)]
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
    Inline { buf: [u8; PARAM_BYTES_INLINE], len: usize },
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
                SqlLiteral::Float(f) => {
                    param_push(&mut buf, &mut len, &f.to_bits().to_le_bytes())
                }
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
/// BLAKE3(h_ast || P_params)` with the param half folded through
/// [`ParamBytes`]. Inline (<= 64B encoded) buffers are bit-identical to the
/// [`hash_param_buffer`] reference by construction (same bytes, one update).
/// Above the inline budget the reference fold is used so the receipt law is
/// never encoding-dependent; the digest branch stays available via
/// [`ParamBytes::fold`] and is measured in `benches/tc_hot_01.rs`.
pub fn fold_params_anchored(h_ast: &[u8; 32], params: &[SqlLiteral]) -> [u8; 32] {
    let encoded = ParamBytes::encode(params);
    let mut hasher = blake3::Hasher::new();
    hasher.update(h_ast);
    if encoded.is_inline() {
        encoded.fold(&mut hasher);
    } else {
        // Digest branch would diverge from the reference receipt encoding —
        // the raw-byte reference fold keeps H_query law-stable.
        hash_param_buffer(&mut hasher, params);
    }
    *hasher.finalize().as_bytes()
}

/// Computes the admitted-statement receipt, in the TC-HOT-01 two-node form:
/// `h_ast = BLAKE3(Canon(G_AST))`, then
/// `H_query = BLAKE3(h_ast || P_params)`
/// (sql-rdf Recommendation, section 1.4, first formula). `canonical_graph`
/// is the output of [`canonical_ast_graph_bytes`]; the parameter buffer is
/// the typed encoding (one type tag byte + little-endian value bytes per
/// parameter) in bind order. One fresh hasher + two updates + finalize —
/// no streaming absorb of `Canon(G_AST)` bytes per receipt.
pub fn admitted_statement_hash(
    canonical_graph: &[u8],
    params: &[SqlLiteral],
) -> [u8; 32] {
    let h_ast = *blake3::hash(canonical_graph).as_bytes();
    let mut hasher = blake3::Hasher::new();
    hasher.update(h_ast.as_slice());
    hash_param_buffer(&mut hasher, params);
    *hasher.finalize().as_bytes()
}

/// C1: true when the SQL text is a stored-data mutation (DML, ATTACH/DETACH,
/// DDL, COPY). Lexical prefix classification, deliberately conservative:
/// anything not matching falls on the no-bump side and relies on the
/// C1-a cacheable bit being false for non-Select statements.
fn mutating_sql(sql: &str) -> bool {
    let trimmed = sql.trim_start();
    let upper: String = trimmed
        .chars()
        .take_while(|c| c.is_ascii_alphabetic())
        .map(|c| c.to_ascii_uppercase())
        .collect();
    matches!(
        upper.as_str(),
        "INSERT" | "UPDATE" | "DELETE" | "ATTACH" | "DETACH" | "CREATE" | "DROP" | "ALTER" | "COPY"
    )
}

impl GraphLawDuckDB {
    /// Creates an in-memory embedded execution instance.
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
        // Lane A02: coarse monotonic start tick (one relaxed atomic load,
        // <10 ns median) instead of an Instant::now() syscall-class read.
        let start_ns = coarse_now_ns();

        // 1. Compute the receipt hash. When the compile was graph-backed,
        // this is the admitted-statement receipt of the sql-rdf
        // Recommendation section 1.4 first formula, TC-HOT-01 two-node
        // form: H_query = BLAKE3(h_ast || P_params) where
        // h_ast = BLAKE3(Canon(G_AST)) is pinned at compile time in
        // `receipt_state` (a 32-byte Copy), so a receipt costs ONE fresh
        // hasher + two updates + finalize — bit-identical to
        // `admitted_statement_hash` by construction (same formula).
        // SQLRDF-17: the legacy `(None, None)` raw-SQL-string hash arm and
        // the hand-built `ast_graph_turtle` re-canonicalization fallback are
        // DELETED, not gated — a query carrying neither the compile-time
        // canon anchor nor canon bytes is a typed `NoCanonicalGraph`
        // refusal, never a raw-SQL admission.
        let query_hash: [u8; 32] = match (&compiled.receipt_state, &compiled.canon_graph_bytes) {
            (Some(h_ast), _) => {
                // Charter A03: fixed-width param ingestion — inline (<= 64B)
                // buffers fold as one contiguous stack-buffer update,
                // bit-identical to the reference fold by construction.
                let mut h_ast_arr = [0u8; 32];
                h_ast_arr.copy_from_slice(h_ast);
                fold_params_anchored(&h_ast_arr, &compiled.params)
            }
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
        if compiled.cacheable {
            if let Some((mut receipt, _store_epoch)) = self
                .cache
                .consult(query_hash, compiled.canon_graph_bytes.as_deref())
            {
                // Restamp the caller's IRI; the receipt identity (hash,
                // rows, triples) is epoch-valid cached state.
                receipt.query_iri = query_iri.to_string();
                return Ok(receipt);
            }
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
            self.stmt_cache
                .prepared_keys
                .borrow_mut()
                .insert(stmt_key);
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
                    Ok(frozen_duckdb::duckdb::types::ValueRef::Boolean(b)) => Some(TermValue::Literal {
                        lexical_form: b.to_string(),
                        datatype: "http://www.w3.org/2001/XMLSchema#boolean".to_string(),
                        language: None,
                        direction: None,
                    }),
                    Ok(frozen_duckdb::duckdb::types::ValueRef::TinyInt(i)) => Some(TermValue::Literal {
                        lexical_form: i.to_string(),
                        datatype: "http://www.w3.org/2001/XMLSchema#integer".to_string(),
                        language: None,
                        direction: None,
                    }),
                    Ok(frozen_duckdb::duckdb::types::ValueRef::SmallInt(i)) => Some(TermValue::Literal {
                        lexical_form: i.to_string(),
                        datatype: "http://www.w3.org/2001/XMLSchema#integer".to_string(),
                        language: None,
                        direction: None,
                    }),
                    Ok(frozen_duckdb::duckdb::types::ValueRef::Int(i)) => Some(TermValue::Literal {
                        lexical_form: i.to_string(),
                        datatype: "http://www.w3.org/2001/XMLSchema#integer".to_string(),
                        language: None,
                        direction: None,
                    }),
                    Ok(frozen_duckdb::duckdb::types::ValueRef::BigInt(i)) => Some(TermValue::Literal {
                        lexical_form: i.to_string(),
                        datatype: "http://www.w3.org/2001/XMLSchema#integer".to_string(),
                        language: None,
                        direction: None,
                    }),
                    Ok(frozen_duckdb::duckdb::types::ValueRef::HugeInt(i)) => Some(TermValue::Literal {
                        lexical_form: i.to_string(),
                        datatype: "http://www.w3.org/2001/XMLSchema#integer".to_string(),
                        language: None,
                        direction: None,
                    }),
                    Ok(frozen_duckdb::duckdb::types::ValueRef::UTinyInt(i)) => Some(TermValue::Literal {
                        lexical_form: i.to_string(),
                        datatype: "http://www.w3.org/2001/XMLSchema#integer".to_string(),
                        language: None,
                        direction: None,
                    }),
                    Ok(frozen_duckdb::duckdb::types::ValueRef::USmallInt(i)) => Some(TermValue::Literal {
                        lexical_form: i.to_string(),
                        datatype: "http://www.w3.org/2001/XMLSchema#integer".to_string(),
                        language: None,
                        direction: None,
                    }),
                    Ok(frozen_duckdb::duckdb::types::ValueRef::UInt(i)) => Some(TermValue::Literal {
                        lexical_form: i.to_string(),
                        datatype: "http://www.w3.org/2001/XMLSchema#integer".to_string(),
                        language: None,
                        direction: None,
                    }),
                    Ok(frozen_duckdb::duckdb::types::ValueRef::UBigInt(i)) => Some(TermValue::Literal {
                        lexical_form: i.to_string(),
                        datatype: "http://www.w3.org/2001/XMLSchema#integer".to_string(),
                        language: None,
                        direction: None,
                    }),
                    Ok(frozen_duckdb::duckdb::types::ValueRef::Float(f)) => Some(TermValue::Literal {
                        lexical_form: f.to_string(),
                        datatype: "http://www.w3.org/2001/XMLSchema#double".to_string(),
                        language: None,
                        direction: None,
                    }),
                    Ok(frozen_duckdb::duckdb::types::ValueRef::Double(d)) => Some(TermValue::Literal {
                        lexical_form: d.to_string(),
                        datatype: "http://www.w3.org/2001/XMLSchema#double".to_string(),
                        language: None,
                        direction: None,
                    }),
                    Ok(frozen_duckdb::duckdb::types::ValueRef::Decimal(d)) => Some(TermValue::Literal {
                        lexical_form: d.to_string(),
                        datatype: "http://www.w3.org/2001/XMLSchema#decimal".to_string(),
                        language: None,
                        direction: None,
                    }),
                    Ok(frozen_duckdb::duckdb::types::ValueRef::Text(bytes)) => Some(TermValue::Literal {
                        lexical_form: String::from_utf8_lossy(bytes).into_owned(),
                        datatype: "http://www.w3.org/2001/XMLSchema#string".to_string(),
                        language: None,
                        direction: None,
                    }),
                    _ => row.get::<_, String>(col_idx).ok().map(|val| TermValue::Literal {
                        lexical_form: val,
                        datatype: "http://www.w3.org/2001/XMLSchema#string".to_string(),
                        language: None,
                        direction: None,
                    }),
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
        };

        // C1 store seam: successful miss of a cacheable statement. D2
        // compare-and-swap: the entry is stamped with `exec_epoch` and stored
        // only if the epoch is unchanged since before execution — a store
        // racing a bump is aborted (the next consult is a miss and re-executes
        // against post-mutation data).
        if compiled.cacheable && self.cache.epoch() == exec_epoch {
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
    pub fn register_csv_view(&self, view_name: &str, glob_pattern: &str) -> Result<(), EvaluatorError> {
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
    use crate::sql_ast::{
        AggregateFunction, Operator, SelectStatement, SqlExpr, TableRef,
    };

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
                    0 => SqlLiteral::String((0..(next() % 32)).map(|_| (next() as u8 % 95) + 32).map(|b| b as char).collect()),
                    1 => SqlLiteral::Integer(next() as i64),
                    2 => SqlLiteral::Float(f64::from_bits(next())),
                    3 => SqlLiteral::Boolean(next() % 2 == 0),
                    _ => SqlLiteral::Null,
                })
                .collect();
            // Reference: two-node admitted_statement_hash.
            let reference = admitted_statement_hash(&canon, &params);
            // Anchor path: pin h_ast = BLAKE3(canon) once, then per receipt
            // one fresh hasher folding h_ast || P_params.
            let h_ast = *blake3::hash(&canon).as_bytes();
            let clone_path = {
                let mut h = blake3::Hasher::new();
                h.update(&h_ast);
                hash_param_buffer(&mut h, &params);
                *h.finalize().as_bytes()
            };
            assert_eq!(clone_path, reference, "clone path must be bit-identical for canon_len={canon_len}");
        }

        // Real-compile witness: compile_turtle's pre-pinned `h_ast` anchor
        // yields the same receipt as the admitted_statement_hash reference
        // over the carried Canon(G_AST) bytes.
        let turtle = "@prefix : <http://example.org/> .\n@prefix sql: <http://graphlaw.org/sql#> .\n:g a sql:SelectStatement ;\n    sql:statementIndex 1 ;\n    sql:fromTable [ sql:tableName \"t\" ] ;\n    sql:selectList ( [ sql:columnName \"v\" ] ) .";
        let compiled = compile_turtle(turtle).expect("graph must compile");
        let canon = compiled.canon_graph_bytes.as_deref().expect("compile carries canon bytes");
        let params = vec![SqlLiteral::Integer(7), SqlLiteral::String("w".into())];
        let mut h = blake3::Hasher::new();
        h.update(&compiled.receipt_state.expect("compile pins h_ast"));
        hash_param_buffer(&mut h, &params);
        assert_eq!(
            *h.finalize().as_bytes(),
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
                        (0..(next() % 32)).map(|_| (next() as u8 % 95) + 32).map(|b| b as char).collect(),
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
            assert_eq!(inline, inline_total <= PARAM_BYTES_INLINE,
                "is_inline must track the encoded size exactly");

            let fast = fold_params_anchored(&h_ast, &params);
            let mut h_ref = blake3::Hasher::new();
            h_ref.update(&h_ast);
            hash_param_buffer(&mut h_ref, &params);
            let reference = *h_ref.finalize().as_bytes();
            assert_eq!(fast, reference,
                "fold_params_anchored must be bit-identical to the reference fold (inline_total={inline_total})");
        }
    }

    /// Charter A03 digest branch (buffers above the inline budget): the
    /// standalone digest is deterministic over the full reference encoding
    /// and separates distinct parameter vectors.
    #[test]
    fn param_bytes_digest_branch_is_deterministic_and_separating() {
        let small: Vec<SqlLiteral> = vec![
            SqlLiteral::Integer(1),
            SqlLiteral::String("x".into()),
        ];
        let big_a: Vec<SqlLiteral> = (0..16)
            .map(|i| SqlLiteral::String(format!("param-value-{i:02}-aaaaaaaaaaaaaaaa")))
            .collect();
        let big_b: Vec<SqlLiteral> = (0..16)
            .map(|i| SqlLiteral::String(format!("PARAM-VALUE-{i:02}-aaaaaaaaaaaaaaaa")))
            .collect();
        assert!(ParamBytes::encode(&small).is_inline());
        let da = ParamBytes::encode(&big_a);
        let db = ParamBytes::encode(&big_b);
        assert!(!da.is_inline(), "large vector must spill to the digest branch");
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
                &query_iri,
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

    const A04_TURTLE: &str = r#"
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

    fn a04_seed(db: &GraphLawDuckDB) {
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
                if seam.prepared_keys.borrow().contains(&stmt_cache_key(&compiled)) {
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
}
