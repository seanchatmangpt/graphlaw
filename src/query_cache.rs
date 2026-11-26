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

use std::collections::{HashMap, VecDeque};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::duckdb_eval::ExecutionReceipt;

/// Maximum live entries; the oldest (FIFO) entry is evicted past this bound.
pub const MAX_CACHE_ENTRIES: usize = 1024;
/// Maximum estimated resident bytes across all entries (FIFO eviction).
pub const MAX_CACHE_BYTES: usize = 64 * 1024 * 1024;

/// One cached compiled-plan result.
#[derive(Debug, Clone)]
pub struct CacheEntry {
    /// Admitted-statement receipt hash the entry is keyed by
    /// (`H_query = BLAKE3(Canon(G_AST) || P_params)`, sql-rdf Rec. §1.4).
    pub h_query: [u8; 32],
    /// The stored execution receipt (as evaluated on a miss).
    pub receipt: ExecutionReceipt,
    /// Grounding bytes: the RDFC-1.0 canonical AST graph (`Canon(G_AST)`)
    /// the entry was stored under. Empty for legacy (non-graph) keys.
    pub grounding: Vec<u8>,
    /// Cache epoch the result was computed against; a store whose epoch is
    /// not the live epoch (checked under the bump lock) is dropped.
    pub epoch: u64,
    /// Store wall-clock time (unix ms).
    pub stored_at: u64,
}

impl CacheEntry {
    /// Estimated resident size in bytes (bound accounting, not exact).
    pub fn approx_bytes(&self) -> usize {
        let r = &self.receipt;
        let triples: usize = r
            .result_triples
            .iter()
            .map(|t| t.subject.len() + t.predicate.len() + 96)
            .sum();
        384 + self.grounding.len()
            + r.query_iri.len()
            + r.sql_statement.len()
            + r.artifact_warning.as_ref().map_or(0, |w| w.len())
            + r.artifact_digest_hex.as_ref().map_or(0, |w| w.len())
            + r.columnar_digest_hex.as_ref().map_or(0, |w| w.len())
            + r.receipt_v2_hex.as_ref().map_or(0, |w| w.len())
            + triples
    }
}

type EntryMap = HashMap<[u8; 32], Arc<CacheEntry>>;

#[derive(Debug, Default)]
struct Inner {
    map: EntryMap,
    /// Insertion order for FIFO eviction.
    order: VecDeque<[u8; 32]>,
    bytes: usize,
}

impl Inner {
    fn remove(&mut self, key: &[u8; 32]) {
        if let Some(old) = self.map.remove(key) {
            self.bytes = self.bytes.saturating_sub(old.approx_bytes());
            self.order.retain(|k| k != key);
        }
    }

    fn clear(&mut self) {
        self.map.clear();
        self.order.clear();
        self.bytes = 0;
    }

    fn insert(&mut self, entry: CacheEntry, max_entries: usize, max_bytes: usize) {
        let size = entry.approx_bytes();
        let key = entry.h_query;
        self.remove(&key);
        if size > max_bytes {
            return; // a single oversize result is never resident
        }
        while !self.order.is_empty()
            && (self.map.len() >= max_entries || self.bytes + size > max_bytes)
        {
            if let Some(oldest) = self.order.pop_front()
                && let Some(old) = self.map.remove(&oldest)
            {
                self.bytes = self.bytes.saturating_sub(old.approx_bytes());
            }
        }
        self.bytes += size;
        self.order.push_back(key);
        self.map.insert(key, Arc::new(entry));
    }

    /// Re-derive order/bytes after an external `with_entries` mutation.
    fn resync(&mut self) {
        let map = &self.map;
        self.order.retain(|k| map.contains_key(k));
        for k in map.keys() {
            if !self.order.contains(k) {
                self.order.push_back(*k);
            }
        }
        self.bytes = self.map.values().map(|e| e.approx_bytes()).sum();
    }
}

/// Epoch-invalidated, bounded, concurrent cache keyed by the
/// admitted-statement hash. Reads take a shared lock; `bump` and `store`
/// serialize on the write lock, and the epoch only changes under it.
#[derive(Debug)]
pub struct QueryCache {
    inner: RwLock<Inner>,
    epoch: AtomicU64,
    max_entries: usize,
    max_bytes: usize,
}

impl QueryCache {
    /// Poison-recovering shared lock (SPOF Tier-1 #3): contents stay
    /// structurally valid after a holder panic.
    fn read(&self) -> RwLockReadGuard<'_, Inner> {
        self.inner.read().unwrap_or_else(|p| p.into_inner())
    }

    fn write(&self) -> RwLockWriteGuard<'_, Inner> {
        self.inner.write().unwrap_or_else(|p| p.into_inner())
    }

    /// Creates an empty cache at epoch 0 with the default bounds.
    pub fn new() -> Self {
        Self::with_bounds(MAX_CACHE_ENTRIES, MAX_CACHE_BYTES)
    }

    /// Creates an empty cache with explicit entry/byte bounds.
    pub fn with_bounds(max_entries: usize, max_bytes: usize) -> Self {
        Self {
            inner: RwLock::new(Inner::default()),
            epoch: AtomicU64::new(0),
            max_entries: max_entries.max(1),
            max_bytes,
        }
    }

    /// Current epoch witness.
    pub fn epoch(&self) -> u64 {
        self.epoch.load(Ordering::Acquire)
    }

    /// Live entry count.
    pub fn len(&self) -> usize {
        self.read().map.len()
    }

    /// True when the cache holds no entries.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Estimated resident bytes.
    pub fn approx_bytes(&self) -> usize {
        self.read().bytes
    }

    /// Looks up `h_query` at the live epoch. Returns the stored receipt
    /// restamped as a cache hit and the entry's store epoch. Stale-epoch and
    /// divergent-grounding entries are misses (stale ones evicted). The
    /// lookup holds only a shared lock and clones an `Arc`; the receipt copy
    /// for restamping happens after the lock is released.
    pub fn consult(
        &self,
        h_query: [u8; 32],
        grounding: Option<&[u8]>,
    ) -> Option<(ExecutionReceipt, u64)> {
        let start = Instant::now();
        let found = {
            let inner = self.read();
            let live = self.epoch();
            inner.map.get(&h_query).map(|e| (Arc::clone(e), live))
        };
        let Some((entry, live)) = found else {
            return None;
        };
        if entry.epoch != live {
            // Stale: evict lazily, re-checking under the write lock.
            let mut inner = self.write();
            let still_stale = inner
                .map
                .get(&h_query)
                .is_some_and(|e| e.epoch != self.epoch());
            if still_stale {
                inner.remove(&h_query);
            }
            return None;
        }
        if !grounding.map(|g| entry.grounding == g).unwrap_or(true) {
            return None; // divergent canon: left in place
        }
        let mut receipt = entry.receipt.clone();
        receipt.cached = true;
        receipt.consult_ms = start.elapsed().as_secs_f64() * 1000.0;
        Some((receipt, entry.epoch))
    }

    /// Compare-and-store: under the same write lock `bump` uses, the entry's
    /// computed-against epoch must equal the live epoch, otherwise the stale
    /// result is dropped. Returns whether the entry was stored. Evicts FIFO
    /// past the entry/byte bounds.
    pub fn store(&self, entry: CacheEntry) -> bool {
        let mut inner = self.write();
        if entry.epoch != self.epoch() {
            return false;
        }
        inner.insert(entry, self.max_entries, self.max_bytes);
        true
    }

    /// Runs `f` with exclusive access to the entry map, under the same
    /// poison-recovering lock as every other accessor. Order/byte accounting
    /// is re-derived afterwards.
    pub fn with_entries<R>(&self, f: impl FnOnce(&mut HashMap<[u8; 32], Arc<CacheEntry>>) -> R) -> R {
        let mut inner = self.write();
        let r = f(&mut inner.map);
        inner.resync();
        r
    }

    /// Bumps the epoch and drops all entries, atomically with respect to
    /// `store` (both hold the write lock). Wired at every stored-data
    /// mutation point; raw `raw_connection` mutation must call this.
    pub fn bump(&self) -> u64 {
        let mut inner = self.write();
        inner.clear();
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
