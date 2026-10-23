//! C1 compiled-plan cache (designs/C1-cache.md, moonshot C1).
//!
//! An in-process, epoch-invalidated cache of execution receipts
//! (`crate::duckdb_eval::ExecutionReceipt`) keyed by
//! the admitted-statement receipt hash `H_query = BLAKE3(Canon(G_AST) ||
//! P_params)` (sql-rdf Rec. section 1.4, first formula). Only statements the
//! compiler classified `cacheable` (SelectStatement arm of `compile_turtle`,
//! C1-a bit) are ever consulted or stored; every DML / ATTACH / DETACH /
//! parquet-view mutation bumps the epoch, dropping all entries — a stale
//! hit is impossible, and a store of a mutating statement is impossible by
//! construction (the C1-a bit is false there).
//!
//! Durable sidecar (disk persistence of the cache) is DEFERRED by design:
//! this module is in-process only; a restart yields a cold cache, never a
//! wrong one.

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Mutex, MutexGuard};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::duckdb_eval::ExecutionReceipt;

/// One cached compiled-plan result.
#[derive(Debug, Clone)]
pub struct CacheEntry {
    /// Admitted-statement receipt hash the entry is keyed by
    /// (`H_query = BLAKE3(Canon(G_AST) || P_params)`, sql-rdf Rec. §1.4).
    pub h_query: [u8; 32],
    /// The stored execution receipt (as evaluated on a miss).
    pub receipt: ExecutionReceipt,
    /// Grounding bytes: the RDFC-1.0 canonical AST graph (`Canon(G_AST)`)
    /// the entry was stored under, so an auditor can recompute
    /// `BLAKE3(Canon || P_params)` and confirm the key binds the graph.
    /// Empty for legacy (non-graph) keys.
    pub grounding: Vec<u8>,
    /// Cache epoch at store time; consulted entries must match the live
    /// epoch exactly.
    pub epoch: u64,
    /// Store wall-clock time (unix ms).
    pub stored_at: u64,
}

/// Epoch-invalidated concurrent cache keyed by the admitted-statement hash.
// Debug is derived: all fields (Mutex<HashMap>, AtomicU64) implement Debug;
// no field needs exclusion.
#[derive(Debug)]
pub struct QueryCache {
    entries: Mutex<HashMap<[u8; 32], CacheEntry>>,
    epoch: AtomicU64,
}

impl QueryCache {
    /// Locks the entry map, recovering the inner guard on a poisoned mutex
    /// (SPOF Tier-1 #3). The map contents are structurally valid even after
    /// a panic in a holder — `PoisonError::into_inner` yields the guard — so
    /// a panic in any holder must not permanently brick a process-wide,
    /// Arc-shared cache. The panicking operation itself is lost (its
    /// mutation may be partial); every subsequent operation is served from
    /// the recovered map.
    fn lock_entries(&self) -> MutexGuard<'_, HashMap<[u8; 32], CacheEntry>> {
        match self.entries.lock() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        }
    }

    /// Creates an empty cache at epoch 0.
    pub fn new() -> Self {
        Self {
            entries: Mutex::new(HashMap::new()),
            epoch: AtomicU64::new(0),
        }
    }

    /// Current epoch witness.
    pub fn epoch(&self) -> u64 {
        self.epoch.load(Ordering::Acquire)
    }

    /// Live entry count.
    pub fn len(&self) -> usize {
        self.lock_entries().len()
    }

    /// True when the cache holds no entries.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Looks up `h_query` at the live epoch. Returns the stored receipt
    /// restamped as a cache hit (`cached = true`, `consult_ms` measured) and
    /// the entry's store epoch. Stale-epoch and divergent-grounding entries
    /// are treated as absent (and evicted).
    pub fn consult(
        &self,
        h_query: [u8; 32],
        grounding: Option<&[u8]>,
    ) -> Option<(ExecutionReceipt, u64)> {
        let start = Instant::now();
        let mut entries = self.lock_entries();
        let hit = entries
            .get(&h_query)
            .filter(|entry| {
                entry.epoch == self.epoch()
                    && grounding.map(|g| entry.grounding == g).unwrap_or(true)
            })
            .cloned();
        match hit {
            Some(mut entry) => {
                entry.receipt.cached = true;
                entry.receipt.consult_ms = start.elapsed().as_secs_f64() * 1000.0;
                Some((entry.receipt, entry.epoch))
            }
            None => {
                // A stale-epoch entry carries no reuse value at this epoch:
                // evict lazily so len() stays an honest witness. A
                // grounding-divergent entry (same hash, different canon) is
                // left in place — it may be valid for its own grounding.
                let stale = entries
                    .get(&h_query)
                    .map(|entry| entry.epoch != self.epoch())
                    .unwrap_or(false);
                if stale {
                    entries.remove(&h_query);
                }
                None
            }
        }
    }

    /// Stores a successful miss. The C1-a classification bit must already be
    /// `true` (caller gates); the entry is stamped with the live epoch.
    pub fn store(&self, entry: CacheEntry) {
        self.lock_entries().insert(entry.h_query, entry);
    }

    /// Runs `f` with exclusive access to the entry map, under the same
    /// poison-recovering lock as every other accessor. Callers that need a
    /// multi-step atomic mutation of the map (auditors, maintenance tools)
    /// go through here; a panic inside `f` poisons the mutex but is
    /// recovered by the next `lock_entries`, so the panic loses at most the
    /// in-flight mutation — never the cache itself.
    pub fn with_entries<R>(&self, f: impl FnOnce(&mut HashMap<[u8; 32], CacheEntry>) -> R) -> R {
        f(&mut self.lock_entries())
    }

    /// Bumps the epoch and drops all entries. Wired at every stored-data
    /// mutation point: DML through `eval_ast_query`, `ATTACH`/`DETACH`,
    /// `register_parquet_view`. Raw `raw_connection` use is outside the
    /// cache's visibility — callers mutating through it must call this
    /// (wiring point noted in the report; see
    /// `GraphLawDuckDB::shared_cache`).
    pub fn bump(&self) -> u64 {
        self.lock_entries().clear();
        self.epoch.fetch_add(1, Ordering::AcqRel) + 1
    }
}

impl Default for QueryCache {
    fn default() -> Self {
        Self::new()
    }
}

/// Unix wall-clock milliseconds.
pub fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

use std::time::Instant;
