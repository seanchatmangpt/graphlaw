//! Cold-path query-receipt ledger (PRD §3.2.2).
//!
//! Companion to the hot-path evidence ring: the ring drainer packages drained
//! descriptors into batched canonical-JSON batch files per
//! `docs/jira/v26.10.22/designs/cold-path-ledger.md` (§Format). Conventions are
//! mirrored from [`crate::receipt_store`]: fixed key order, hand-rolled escaper
//! (no serde), content-addressed `<blake3-of-batch-bytes>.json`, subject
//! `query-ledger-v1`, atomic temp+rename.
//!
//! Where C21 receipt chains chain *states*, this ledger chains *batches*:
//! each batch names its parent (the previous batch's content digest); the
//! genesis parent is 64 ASCII zeros. A monotonic `first_seq` makes
//! drain-completeness an arithmetic check, and startup discontinuities are
//! persisted as gap markers — never backfilled. Aggregation over a period
//! reads per-batch `.agg` sidecars (saturating, with a `saturated` flag — no
//! overflow laundering) and spot-re-folds at least one batch per query as the
//! tamper check.
//!
//! Durability bound (disclosed, no WAL): flush at count 1024 or age 2s,
//! whichever first, plus drain-and-flush on [`Drop`]. A crash yields a batch
//! fully valid or absent; loss is bounded by the un-flushed partial batch and
//! disclosed through gap markers, so periods containing them aggregate with
//! `complete: false`.

use std::collections::BTreeMap;
use std::fmt;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

/// Genesis parent digest: 64 ASCII zeros.
pub const GENESIS: &str = "0000000000000000000000000000000000000000000000000000000000000000";

/// Flush batch at this many buffered descriptors.
pub const FLUSH_COUNT: usize = 1024;
/// Flush batch at this age (milliseconds), whichever comes first.
pub const FLUSH_AGE_MS: u64 = 2_000;

/// Hot-path descriptor (64B shape): content-addressed query identity and
/// emission counts. Deliberately excludes SQL text / result triples — cold
/// data; `h_query` is the content address (C1 Layer-2 holds full receipts).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Descriptor {
    /// BLAKE3 of the canonical query text.
    pub h_query: [u8; 32],
    /// Rows emitted (saturating at the producer).
    pub rows_emitted: u32,
    /// bit0 cached, bit1 cacheable-class.
    pub flags: u8,
    /// Producer wall-clock, ms since Unix epoch.
    pub ts_unix_ms: u64,
    /// Producer epoch counter.
    pub epoch: u64,
}

/// Boundary conversion from the 64 B `repr(C)` ring descriptor. Disclosed
/// mapping: `rows_emitted` saturates at `u32::MAX` (ledger field width); the
/// ring carries no producer epoch, so `epoch` records `0`; `flags` = the
/// ring's `cached` byte (bit0 cached; the ring carries no cacheable-class
/// bit); `execution_ms` has no ledger field and is dropped (it survives in
/// the NDJSON evidence chain, which is the full-fidelity surface).
#[cfg(all(feature = "evidentiary-ring", not(target_arch = "wasm32")))]
impl From<crate::evidence_ring::RingDescriptor> for Descriptor {
    fn from(r: crate::evidence_ring::RingDescriptor) -> Self {
        Self {
            h_query: r.h_query,
            rows_emitted: u32::try_from(r.rows_emitted).unwrap_or(u32::MAX),
            flags: r.cached,
            ts_unix_ms: r.timestamp_ms,
            epoch: 0,
        }
    }
}

/// Typed ledger refusal (stable surface; `#[non_exhaustive]` like
/// [`crate::receipt_store::StoreError`]).
#[derive(Debug)]
#[non_exhaustive]
pub enum LedgerError {
    /// An I/O error, rendered as text.
    Io(String),
    /// A batch file's bytes do not hash to its content address.
    DigestMismatch {
        /// File whose bytes are wrong.
        file: String,
        /// Digest the file name promises.
        expected: String,
        /// Digest the bytes actually have.
        actual: String,
    },
    /// A stored batch is not a canonical ledger batch document.
    Malformed {
        /// File the error concerns.
        file: String,
    },
    /// The stored batches do not form one linear parent -> child chain.
    BrokenChain {
        /// Description of where the chain breaks.
        detail: String,
    },
    /// Two batches claim the same parent — the chain forked.
    Fork {
        /// Parent digest claimed by more than one batch.
        parent: String,
    },
    /// More than one batch claims the genesis parent.
    DoubleGenesis {
        /// Digest of the second genesis batch.
        digest: String,
    },
    /// A batch has no `.agg` sidecar.
    MissingAgg {
        /// Batch digest whose sidecar is absent.
        batch: String,
    },
    /// A `.agg` sidecar disagrees with a spot-re-fold of its batch.
    AggMismatch {
        /// Batch digest that failed the re-fold.
        batch: String,
    },
    /// CHAIN index disagrees with the batch files on disk.
    ChainIndex {
        /// Description of the disagreement.
        detail: String,
    },
}

impl fmt::Display for LedgerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LedgerError::Io(e) => write!(f, "query ledger io: {e}"),
            LedgerError::DigestMismatch {
                file,
                expected,
                actual,
            } => write!(
                f,
                "ledger batch `{file}` tampered: address {expected}, content hashes to {actual}"
            ),
            LedgerError::Malformed { file } => write!(f, "malformed ledger batch `{file}`"),
            LedgerError::BrokenChain { detail } => write!(f, "ledger chain broken: {detail}"),
            LedgerError::Fork { parent } => {
                write!(f, "ledger chain forked at parent `{parent}`")
            }
            LedgerError::DoubleGenesis { digest } => {
                write!(f, "double genesis batch `{digest}`")
            }
            LedgerError::MissingAgg { batch } => {
                write!(f, "batch `{batch}` has no .agg sidecar")
            }
            LedgerError::AggMismatch { batch } => {
                write!(f, "batch `{batch}` .agg disagrees with spot-re-fold")
            }
            LedgerError::ChainIndex { detail } => write!(f, "CHAIN index wrong: {detail}"),
        }
    }
}

impl std::error::Error for LedgerError {}

impl From<std::io::Error> for LedgerError {
    fn from(e: std::io::Error) -> Self {
        LedgerError::Io(e.to_string())
    }
}

/// Per-batch aggregation sidecar (ASC 450): saturating per-epoch sums with a
/// `saturated` flag, so overflow is disclosed loss, never a wrapped number.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct BatchAgg {
    /// Batch digest this sidecar summarizes.
    pub batch: String,
    /// rows_emitted summed per epoch, saturating at u64::MAX.
    pub rows_emitted_sum: BTreeMap<u64, u64>,
    /// True when any sum saturated.
    pub saturated: bool,
}

/// Result of a period aggregation.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Aggregate {
    /// rows_emitted summed per epoch over the period, saturating.
    pub rows_emitted_sum: BTreeMap<u64, u64>,
    /// True when any per-epoch sum saturated.
    pub saturated: bool,
    /// False when the period contains a gap marker, a sequence discontinuity
    /// between chain batches, or a missing sidecar — loss is disclosed,
    /// never absorbed.
    pub complete: bool,
    /// Number of batches folded into the sum.
    pub batches: usize,
}

/// Where the ledger's sequence counter stands.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChainState {
    /// Digest of the newest batch (genesis when empty).
    pub head: String,
    /// Digest of the oldest batch.
    pub tail: String,
    /// Number of batches.
    pub batches: u64,
    /// Next descriptor sequence number.
    pub next_seq: u64,
    /// F8 promotion: per-batch `BLAKE3(.agg bytes)` recorded at flush time,
    /// keyed by batch digest. Makes the `.agg` sidecar tamper-EVIDENT via
    /// [`QueryLedger::verify`], not spot-check-only. Batches predating the
    /// promotion (hand-minted chains) carry no entry and are unchecked here.
    pub aggs: BTreeMap<String, String>,
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

fn digest_of(bytes: &[u8]) -> String {
    hex(blake3::hash(bytes).as_bytes())
}

// Hand-rolled escaper, mirroring src/receipt_store.rs conventions verbatim.
fn esc(s: &str) -> String {
    let mut o = String::from("\"");
    for c in s.chars() {
        match c {
            '"' => o.push_str("\\\""),
            '\\' => o.push_str("\\\\"),
            '\n' => o.push_str("\\n"),
            '\r' => o.push_str("\\r"),
            '\t' => o.push_str("\\t"),
            c if (c as u32) < 0x20 => o.push_str(&format!("\\u{:04x}", c as u32)),
            c => o.push(c),
        }
    }
    o.push('"');
    o
}

/// Canonical batch serialization: fixed key order, no whitespace, no serde.
fn encode_batch(parent: &str, first_seq: u64, count: u32, records: &[Descriptor]) -> String {
    let mut out = format!(
        "{{\"parent\":{},\"first_seq\":{},\"count\":{},\"records\":[",
        esc(parent),
        first_seq,
        count
    );
    for (i, r) in records.iter().enumerate() {
        if i > 0 {
            out.push(',');
        }
        out.push_str(&format!(
            "{{\"h_query\":\"{}\",\"rows_emitted\":{},\"flags\":{},\"ts_unix_ms\":{},\"epoch\":{}}}",
            hex(&r.h_query),
            r.rows_emitted,
            r.flags,
            r.ts_unix_ms,
            r.epoch
        ));
    }
    out.push_str("]}");
    out
}

struct Cur<'a> {
    b: &'a [u8],
    i: usize,
}

impl Cur<'_> {
    fn eat(&mut self, lit: &str) -> Option<()> {
        if self.b[self.i..].starts_with(lit.as_bytes()) {
            self.i += lit.len();
            Some(())
        } else {
            None
        }
    }

    fn u(&mut self) -> Option<u64> {
        let start = self.i;
        while self.b.get(self.i).is_some_and(|c| c.is_ascii_digit()) {
            self.i += 1;
        }
        if self.i == start {
            return None;
        }
        std::str::from_utf8(&self.b[start..self.i])
            .ok()?
            .parse()
            .ok()
    }

    fn hex32(&mut self) -> Option<[u8; 32]> {
        let mut out = [0u8; 32];
        for b in &mut out {
            let hi = (self.b.get(self.i).copied().unwrap_or(0) as char).to_digit(16)?;
            let lo = (self.b.get(self.i + 1).copied().unwrap_or(0) as char).to_digit(16)?;
            *b = (hi * 16 + lo) as u8;
            self.i += 2;
        }
        Some(out)
    }
}

fn parse_batch(
    bytes: &[u8],
    file: &str,
) -> Result<(String, u64, u32, Vec<Descriptor>), LedgerError> {
    let malformed = || LedgerError::Malformed {
        file: file.to_string(),
    };
    let mut c = Cur { b: bytes, i: 0 };
    c.eat("{\"parent\":\"").ok_or_else(malformed)?;
    let parent_start = c.i;
    while c.b.get(c.i) != Some(&b'"') {
        c.i += 1;
    }
    let parent = String::from_utf8(c.b[parent_start..c.i].to_vec()).map_err(|_| malformed())?;
    c.i += 1;
    c.eat(",\"first_seq\":").ok_or_else(malformed)?;
    let first_seq = c.u().ok_or_else(malformed)?;
    c.eat(",\"count\":").ok_or_else(malformed)?;
    let count = c.u().ok_or_else(malformed)? as u32;
    c.eat(",\"records\":[").ok_or_else(malformed)?;
    let mut records = Vec::with_capacity(count as usize);
    for idx in 0..count {
        if idx > 0 {
            c.eat(",").ok_or_else(malformed)?;
        }
        c.eat("{\"h_query\":\"").ok_or_else(malformed)?;
        let h = c.hex32().ok_or_else(malformed)?;
        c.eat("\",\"rows_emitted\":").ok_or_else(malformed)?;
        let rows = c.u().ok_or_else(malformed)? as u32;
        c.eat(",\"flags\":").ok_or_else(malformed)?;
        let flags = c.u().ok_or_else(malformed)? as u8;
        c.eat(",\"ts_unix_ms\":").ok_or_else(malformed)?;
        let ts = c.u().ok_or_else(malformed)?;
        c.eat(",\"epoch\":").ok_or_else(malformed)?;
        let epoch = c.u().ok_or_else(malformed)?;
        c.eat("}").ok_or_else(malformed)?;
        records.push(Descriptor {
            h_query: h,
            rows_emitted: rows,
            flags,
            ts_unix_ms: ts,
            epoch,
        });
    }
    c.eat("]}").ok_or_else(malformed)?;
    if records.len() != count as usize {
        return Err(malformed());
    }
    if c.i != bytes.len() {
        return Err(malformed());
    }
    Ok((parent, first_seq, count, records))
}

// ---- .agg sidecar canonical form -------------------------------------------

fn encode_agg(a: &BatchAgg) -> String {
    let mut out = format!("{{\"batch\":{},\"rows_emitted_sum\":{{", esc(&a.batch));
    for (i, (epoch, sum)) in a.rows_emitted_sum.iter().enumerate() {
        if i > 0 {
            out.push(',');
        }
        out.push_str(&format!("\"{epoch}\":{sum}"));
    }
    out.push_str("},\"saturated\":");
    out.push_str(if a.saturated { "true" } else { "false" });
    out.push('}');
    out
}

fn parse_agg(bytes: &[u8], batch: &str) -> Result<BatchAgg, LedgerError> {
    let mut a = BatchAgg {
        batch: batch.to_string(),
        rows_emitted_sum: BTreeMap::new(),
        saturated: false,
    };
    let malformed = || LedgerError::Malformed {
        file: format!("{batch}.agg"),
    };
    let mut c = Cur { b: bytes, i: 0 };
    c.eat("{\"batch\":\"").ok_or_else(malformed)?;
    while c.b.get(c.i) != Some(&b'"') {
        c.i += 1;
    }
    c.i += 1;
    c.eat(",\"rows_emitted_sum\":{").ok_or_else(malformed)?;
    if c.eat("}").is_none() {
        loop {
            c.eat("\"").ok_or_else(malformed)?;
            let epoch = c.u().ok_or_else(malformed)?;
            c.eat("\"").ok_or_else(malformed)?;
            c.eat(":").ok_or_else(malformed)?;
            let sum = c.u().ok_or_else(malformed)?;
            a.rows_emitted_sum.insert(epoch, sum);
            if c.eat(",").is_none() {
                break;
            }
        }
        c.eat("}").ok_or_else(malformed)?;
    }
    if c.eat(",\"saturated\":true").is_some() {
        a.saturated = true;
    } else {
        c.eat(",\"saturated\":false").ok_or_else(malformed)?;
    }
    c.eat("}").ok_or_else(malformed)?;
    if c.i != bytes.len() {
        return Err(malformed());
    }
    Ok(a)
}

/// Saturating fold of one batch's records into a [`BatchAgg`]. Overflow pins
/// the sum at u64::MAX and sets `saturated` — disclosed loss, never wrapped.
pub fn fold_batch(batch: &str, records: &[Descriptor]) -> BatchAgg {
    let mut agg = BatchAgg {
        batch: batch.to_string(),
        rows_emitted_sum: BTreeMap::new(),
        saturated: false,
    };
    for r in records {
        let e = agg.rows_emitted_sum.entry(r.epoch).or_insert(0u64);
        let (sum, over) = e.overflowing_add(u64::from(r.rows_emitted));
        if over {
            *e = u64::MAX;
            agg.saturated = true;
        } else {
            *e = sum;
        }
    }
    agg
}

/// Content address of a batch file: BLAKE3 of its bytes, hex-encoded. Public
/// so courts can mint/verify content-addressed names without a direct
/// blake3 dependency.
pub fn content_digest(bytes: &[u8]) -> String {
    digest_of(bytes)
}

/// Now in ms since Unix epoch.
pub fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

/// The cold-path ledger over a directory.
pub struct QueryLedger {
    dir: PathBuf,
    parent: String,
    next_seq: u64,
    batches: u64,
    head: String,
    tail: String,
    buf: Vec<Descriptor>,
    buf_first_seq: u64,
    buf_opened_ms: u64,
    /// F8: `BLAKE3(.agg bytes)` per persisted batch, written into the CHAIN
    /// index at flush time.
    aggs: BTreeMap<String, String>,
    /// Last flush error (flush on Drop cannot return; surfaced here).
    pub last_flush_error: Option<LedgerError>,
}

impl QueryLedger {
    /// Open (or create) a ledger directory. Loads the CHAIN index and
    /// cross-checks it against the batch files (full walk is
    /// [`QueryLedger::verify`]).
    pub fn open(dir: impl AsRef<Path>) -> Result<Self, LedgerError> {
        let dir = dir.as_ref();
        fs::create_dir_all(dir)?;
        let state = verify_dir(dir)?;
        Ok(QueryLedger {
            dir: dir.to_path_buf(),
            parent: state.head.clone(),
            next_seq: state.next_seq,
            batches: state.batches,
            head: state.head.clone(),
            tail: state.tail,
            buf: Vec::new(),
            buf_first_seq: state.next_seq,
            buf_opened_ms: now_ms(),
            aggs: state.aggs.clone(),
            last_flush_error: None,
        })
    }

    /// Open and reconcile against the producer's sequence counter. If the
    /// producer is ahead of the persisted ledger (startup discontinuity —
    /// descriptors drained before the last flush were lost with the process),
    /// the loss is persisted as a gap marker and never backfilled.
    pub fn open_with_producer(
        dir: impl AsRef<Path>,
        producer_next_seq: u64,
    ) -> Result<Self, LedgerError> {
        let ledger = Self::open(dir)?;
        let persisted = ledger.next_seq;
        if producer_next_seq > persisted {
            ledger.record_gap(persisted, producer_next_seq - 1)?;
        }
        Ok(ledger)
    }

    /// Producer watermark reconciliation readback: `W = next_seq + gap_covered`
    /// — the number of descriptors the ledger has accounted for (recorded in
    /// batches or disclosed via gap markers). The restart path in
    /// [`crate::evidence_ring`] seeds the producer sequence from the caller's
    /// attempted count, never from a silent reset; this helper is the exact
    /// accounting a court asserts against.
    pub fn accounted_descriptors(dir: impl AsRef<Path>) -> Result<u64, LedgerError> {
        let ledger = QueryLedger::open(dir)?;
        let covered: u64 = ledger.gaps()?.iter().map(|(f, l)| l - f + 1).sum();
        Ok(ledger.chain_state().next_seq + covered)
    }

    /// Persist a gap marker for descriptors `first..=last` that will never
    /// arrive.
    pub fn record_gap(&self, first: u64, last: u64) -> Result<(), LedgerError> {
        if last < first {
            return Ok(());
        }
        let gaps = self.dir.join("gaps");
        fs::create_dir_all(&gaps)?;
        let file = gaps.join(format!("gap-{first}.json"));
        let body = format!(
            "{{\"missing_first_seq\":{first},\"missing_last_seq\":{last}}}"
        );
        atomic_write(&file, body.as_bytes())
    }

    /// Buffer one descriptor; flushes at [`FLUSH_COUNT`] or [`FLUSH_AGE_MS`].
    pub fn push(&mut self, d: Descriptor) -> Result<(), LedgerError> {
        self.buf.push(d);
        if self.buf.len() >= FLUSH_COUNT
            || now_ms().saturating_sub(self.buf_opened_ms) >= FLUSH_AGE_MS
        {
            self.flush()?;
        }
        Ok(())
    }

    /// Record a drained batch: buffer every descriptor, then force a flush.
    /// This is the entry point the ring drainer calls.
    pub fn record_batch(&mut self, batch: &[Descriptor]) -> Result<(), LedgerError> {
        for d in batch {
            self.buf.push(*d);
        }
        self.flush()
    }

    /// Write the buffered descriptors as one content-addressed batch file plus
    /// its `.agg` sidecar, then advance the CHAIN index atomically.
    pub fn flush(&mut self) -> Result<(), LedgerError> {
        if self.buf.is_empty() {
            return Ok(());
        }
        let records = std::mem::take(&mut self.buf);
        let first_seq = self.buf_first_seq;
        let count = records.len() as u32;
        let parent = self.parent.clone();
        let body = encode_batch(&parent, first_seq, count, &records);
        let digest = digest_of(body.as_bytes());
        let path = self.dir.join(format!("{digest}.json"));
        if !path.exists() {
            atomic_write(&path, body.as_bytes())?;
        }
        // .agg sidecar, keyed to the batch digest (not self-addressed — but
        // F8 promotes it to tamper-EVIDENT: its BLAKE3 is written into the
        // CHAIN index below, and verify() checks it, so the spot-re-fold is
        // no longer the only tamper check).
        let agg = fold_batch(&digest, &records);
        let agg_path = self.dir.join(format!("{digest}.agg"));
        if !agg_path.exists() {
            atomic_write(&agg_path, encode_agg(&agg).as_bytes())?;
        }
        // Digest the bytes actually on disk (idempotent re-flush keeps the
        // recorded digest faithful to the persisted sidecar).
        let agg_digest = digest_of(&fs::read(&agg_path)?);
        self.aggs.insert(digest.clone(), agg_digest);
        // Advance the chain.
        self.parent = digest.clone();
        self.head = digest;
        if self.tail == GENESIS {
            self.tail = self.head.clone();
        }
        self.batches += 1;
        self.next_seq = first_seq + u64::from(count);
        self.buf_first_seq = self.next_seq;
        self.buf_opened_ms = now_ms();
        self.write_chain()
    }

    fn write_chain(&self) -> Result<(), LedgerError> {
        let mut body = format!(
            "{{\"head\":{},\"tail\":{},\"batches\":{},\"next_seq\":{},\"aggs\":{{",
            esc(&self.head),
            esc(&self.tail),
            self.batches,
            self.next_seq
        );
        for (i, (batch, agg)) in self.aggs.iter().enumerate() {
            if i > 0 {
                body.push(',');
            }
            body.push_str(&format!("{},{}", esc(batch), esc(agg)));
        }
        body.push_str("}}");
        atomic_write(&self.dir.join("chain.json"), body.as_bytes())
    }

    /// Current chain state.
    pub fn chain_state(&self) -> ChainState {
        ChainState {
            head: self.head.clone(),
            tail: self.tail.clone(),
            batches: self.batches,
            next_seq: self.next_seq,
            aggs: self.aggs.clone(),
        }
    }

    /// Full chain verification: content addresses, chain linkage, fork /
    /// double-genesis refusal, and CHAIN agreement.
    pub fn verify(&self) -> Result<ChainState, LedgerError> {
        verify_dir(&self.dir)
    }

    /// Read all persisted gap markers.
    pub fn gaps(&self) -> Result<Vec<(u64, u64)>, LedgerError> {
        let malformed = || LedgerError::Malformed { file: String::from("gaps") };
        let mut out = Vec::new();
        let entries = match fs::read_dir(self.dir.join("gaps")) {
            Ok(e) => e,
            Err(_) => return Ok(out),
        };
        for e in entries {
            let e = e?;
            let name = e.file_name();
            let name = name.to_string_lossy();
            let Some(rest) = name.strip_prefix("gap-").and_then(|s| s.strip_suffix(".json"))
            else {
                continue;
            };
            let _first: u64 = rest.parse().map_err(|_| LedgerError::Malformed {
                file: name.to_string(),
            })?;
            let bytes = fs::read(e.path())?;
            let mut c = Cur { b: &bytes, i: 0 };
            c.eat("{\"missing_first_seq\":").ok_or_else(malformed)?;
            let f = c.u().ok_or_else(malformed)?;
            c.eat(",\"missing_last_seq\":").ok_or_else(malformed)?;
            let l = c.u().ok_or_else(malformed)?;
            c.eat("}").ok_or_else(malformed)?;
            if c.i != bytes.len() {
                return Err(LedgerError::Malformed {
                    file: name.to_string(),
                });
            }
            out.push((f, l));
        }
        out.sort();
        Ok(out)
    }

    /// Aggregate rows_emitted per epoch over a wall-clock period. Folds the
    /// `.agg` sidecar of every batch overlapping the period and spot-re-folds
    /// at least one batch (the tamper check). `complete: false` over gaps.
    pub fn aggregate(&self, from_ms: u64, to_ms: u64) -> Result<Aggregate, LedgerError> {
        let mut batches = load_batches(&self.dir)?;
        batches.sort_by_key(|b| b.first_seq);

        let mut out = Aggregate {
            complete: true,
            ..Aggregate::default()
        };
        let mut folded: Vec<&LoadedBatch> = Vec::new();
        for b in &batches {
            let overlaps = b
                .records
                .iter()
                .any(|r| r.ts_unix_ms >= from_ms && r.ts_unix_ms <= to_ms);
            if !overlaps {
                continue;
            }
            let agg_path = self.dir.join(format!("{}.agg", b.digest));
            if !agg_path.exists() {
                return Err(LedgerError::MissingAgg {
                    batch: b.digest.clone(),
                });
            }
            folded.push(b);
        }
        // Sequence continuity inside the folded window.
        for w in folded.windows(2) {
            let expected = w[0].first_seq + w[0].records.len() as u64;
            if w[1].first_seq != expected {
                out.complete = false;
            }
        }
        // Persisted gap markers overlapping the folded seq range disclose loss.
        let gaps = self.gaps()?;
        if !gaps.is_empty() {
            match (
                folded.first().map(|b| b.first_seq),
                folded.last().map(|b| b.first_seq + b.records.len() as u64),
            ) {
                (Some(lo), Some(hi)) => {
                    // hi is exclusive chain coverage; a gap starting exactly
                    // at hi still discloses loss adjacent to the period.
                    if gaps.iter().any(|(f, l)| *l >= lo && *f <= hi) {
                        out.complete = false;
                    }
                }
                _ => out.complete = false,
            }
        }
        // Fold sidecars with saturating adds; spot-re-fold the first batch.
        let mut spot_checked = false;
        for b in &folded {
            let bytes = fs::read(self.dir.join(format!("{}.agg", b.digest)))?;
            let agg = parse_agg(&bytes, &b.digest)?;
            if !spot_checked {
                let refold = fold_batch(&b.digest, &b.records);
                if refold != agg {
                    return Err(LedgerError::AggMismatch {
                        batch: b.digest.clone(),
                    });
                }
                spot_checked = true;
            }
            for (epoch, sum) in agg.rows_emitted_sum {
                let e = out.rows_emitted_sum.entry(epoch).or_insert(0u64);
                let (s, over) = e.overflowing_add(sum);
                if over {
                    *e = u64::MAX;
                    out.saturated = true;
                } else {
                    *e = s;
                }
            }
            out.saturated |= agg.saturated;
            out.batches += 1;
        }
        Ok(out)
    }
}

impl Drop for QueryLedger {
    fn drop(&mut self) {
        // Drain-and-flush obligation; errors surfaced via last_flush_error.
        if let Err(e) = self.flush() {
            self.last_flush_error = Some(e);
        }
    }
}

// ---- on-disk verification ---------------------------------------------------

struct LoadedBatch {
    digest: String,
    parent: String,
    first_seq: u64,
    records: Vec<Descriptor>,
}

fn read_chain_state(dir: &Path) -> Result<ChainState, LedgerError> {
    let path = dir.join("chain.json");
    if !path.exists() {
        return Ok(ChainState {
            head: GENESIS.to_string(),
            tail: GENESIS.to_string(),
            batches: 0,
            next_seq: 0,
            aggs: BTreeMap::new(),
        });
    }
    let bytes = fs::read(&path)?;
    let malformed = || LedgerError::Malformed {
        file: "chain.json".into(),
    };
    let mut c = Cur { b: &bytes, i: 0 };
    c.eat("{\"head\":\"").ok_or_else(malformed)?;
    let head = take_hex64(&mut c)?;
    c.eat("\",\"tail\":\"").ok_or_else(malformed)?;
    let tail = take_hex64(&mut c)?;
    c.eat("\",\"batches\":").ok_or_else(malformed)?;
    let batches = c.u().ok_or_else(malformed)?;
    c.eat(",\"next_seq\":").ok_or_else(malformed)?;
    let next_seq = c.u().ok_or_else(malformed)?;
    // F8: optional `,"aggs":{"<batch>":"<agg-digest>",...}` — absent in
    // legacy chains (hand-minted courts), whose sidecars stay spot-check-only.
    let mut aggs = BTreeMap::new();
    if c.eat(",\"aggs\":{").is_some() {
        if c.eat("}").is_none() {
            loop {
                c.eat("\"").ok_or_else(malformed)?;
                let batch = take_hex64(&mut c)?;
                c.eat("\",\"").ok_or_else(malformed)?;
                let agg = take_hex64(&mut c)?;
                c.eat("\"").ok_or_else(malformed)?;
                if aggs.insert(batch, agg).is_some() {
                    return Err(LedgerError::ChainIndex {
                        detail: "duplicate agg digest entry in CHAIN".into(),
                    });
                }
                if c.eat(",").is_none() {
                    break;
                }
            }
            c.eat("}").ok_or_else(malformed)?;
        }
    }
    c.eat("}").ok_or_else(malformed)?;
    if c.i != bytes.len() {
        return Err(LedgerError::Malformed {
            file: "chain.json".into(),
        });
    }
    Ok(ChainState {
        head,
        tail,
        batches,
        next_seq,
        aggs,
    })
}

fn take_hex64(c: &mut Cur<'_>) -> Result<String, LedgerError> {
    let malformed = || LedgerError::Malformed {
        file: "chain.json".into(),
    };
    let raw = c.b.get(c.i..c.i + 64).ok_or_else(malformed)?;
    c.i += 64;
    if raw.iter().copied().all(|b| b.is_ascii_hexdigit()) {
        // hex digits are ASCII by construction
        Ok(std::str::from_utf8(raw).map_err(|_| malformed())?.to_string())
    } else {
        Err(malformed())
    }
}

fn list_batch_files(dir: &Path) -> Result<Vec<(String, PathBuf)>, LedgerError> {
    let mut out = Vec::new();
    for e in fs::read_dir(dir)? {
        let e = e?;
        let name = e.file_name();
        let name = name.to_string_lossy();
        let Some(stem) = name.strip_suffix(".json") else {
            continue;
        };
        if stem == "chain" || stem.len() != 64 || !stem.bytes().all(|b| b.is_ascii_hexdigit()) {
            continue;
        }
        out.push((stem.to_string(), e.path()));
    }
    Ok(out)
}

fn load_batches(dir: &Path) -> Result<Vec<LoadedBatch>, LedgerError> {
    let mut out = Vec::new();
    for (digest, path) in list_batch_files(dir)? {
        let bytes = fs::read(&path)?;
        let actual = digest_of(&bytes);
        if actual != digest {
            return Err(LedgerError::DigestMismatch {
                file: format!("{digest}.json"),
                expected: digest,
                actual,
            });
        }
        let (parent, first_seq, _count, records) =
            parse_batch(&bytes, &format!("{digest}.json"))?;
        out.push(LoadedBatch {
            digest,
            parent,
            first_seq,
            records,
        });
    }
    Ok(out)
}

fn verify_dir(dir: &Path) -> Result<ChainState, LedgerError> {
    let mut batches = load_batches(dir)?;
    if batches.is_empty() {
        let st = read_chain_state(dir)?;
        if st.batches != 0 || st.head != GENESIS {
            return Err(LedgerError::ChainIndex {
                detail: "CHAIN claims batches but no batch files exist".into(),
            });
        }
        return Ok(st);
    }
    // Double genesis: at most one batch may claim GENESIS.
    let genesis_count = batches.iter().filter(|b| b.parent == GENESIS).count();
    if genesis_count > 1 {
        let mut gs: Vec<&LoadedBatch> =
            batches.iter().filter(|b| b.parent == GENESIS).collect();
        gs.sort_by_key(|b| b.first_seq);
        return Err(LedgerError::DoubleGenesis {
            digest: gs[1].digest.clone(),
        });
    }
    // Fork: at most one batch per parent.
    let mut by_parent: BTreeMap<&str, ()> = BTreeMap::new();
    for b in &batches {
        if by_parent.insert(b.parent.as_str(), ()).is_some() {
            return Err(LedgerError::Fork {
                parent: b.parent.clone(),
            });
        }
    }
    // Linear chain: sort by first_seq and require parent linkage.
    batches.sort_by_key(|b| b.first_seq);
    let mut expected_parent = GENESIS.to_string();
    for b in &batches {
        if b.parent != expected_parent {
            return Err(LedgerError::BrokenChain {
                detail: format!(
                    "batch {} (first_seq {}) names parent {}, expected {}",
                    b.digest, b.first_seq, b.parent, expected_parent
                ),
            });
        }
        expected_parent = b.digest.clone();
    }
    // CHAIN agreement.
    let st = read_chain_state(dir)?;
    let head = batches.last().map(|b| b.digest.clone()).unwrap_or_else(|| GENESIS.to_string());
    let tail = batches.first().map(|b| b.digest.clone()).unwrap_or_else(|| GENESIS.to_string());
    if st.head != head || st.tail != tail || st.batches != batches.len() as u64 {
        return Err(LedgerError::ChainIndex {
            detail: format!(
                "CHAIN {{head {}, tail {}, batches {}}} vs files {{head {}, tail {}, batches {}}}",
                st.head, st.tail, st.batches, head, tail, batches.len()
            ),
        });
    }
    // F8: every recorded .agg digest must match the sidecar bytes on disk —
    // the sidecar is tamper-EVIDENT against the CHAIN, not spot-check-only.
    for (batch, agg_digest) in &st.aggs {
        if !batches.iter().any(|b| &b.digest == batch) {
            return Err(LedgerError::ChainIndex {
                detail: format!("CHAIN records an agg digest for unknown batch {batch}"),
            });
        }
        let agg_path = dir.join(format!("{batch}.agg"));
        if !agg_path.exists() {
            return Err(LedgerError::MissingAgg {
                batch: batch.clone(),
            });
        }
        let actual = digest_of(&fs::read(&agg_path)?);
        if &actual != agg_digest {
            return Err(LedgerError::DigestMismatch {
                file: format!("{batch}.agg"),
                expected: agg_digest.clone(),
                actual,
            });
        }
    }
    Ok(ChainState {
        head,
        tail,
        batches: batches.len() as u64,
        next_seq: st.next_seq,
        aggs: st.aggs,
    })
}

fn atomic_write(path: &Path, bytes: &[u8]) -> Result<(), LedgerError> {
    let tmp = path.with_extension("tmp");
    {
        let mut f = fs::File::create(&tmp)?;
        f.write_all(bytes)?;
        f.sync_all()?;
    }
    fs::rename(&tmp, path)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn desc(epoch: u64, rows: u32, ts: u64) -> Descriptor {
        Descriptor {
            h_query: [7u8; 32],
            rows_emitted: rows,
            flags: 0,
            ts_unix_ms: ts,
            epoch,
        }
    }

    #[test]
    fn roundtrip_batch_bytes() {
        let body = encode_batch(GENESIS, 0, 2, &[desc(1, 5, 100), desc(2, 9, 200)]);
        let (parent, first_seq, count, records) = parse_batch(body.as_bytes(), "t.json").unwrap();
        assert_eq!(parent, GENESIS);
        assert_eq!((first_seq, count), (0, 2));
        assert_eq!(records, vec![desc(1, 5, 100), desc(2, 9, 200)]);
        assert_eq!(digest_of(body.as_bytes()).len(), 64);
    }

    #[test]
    fn fold_accumulates_exactly() {
        // Per-batch folds of u32 rows into u64 can never overflow; saturation
        // is only reachable when folding multiple sidecars (court F6).
        let a = fold_batch("b", &[desc(1, u32::MAX, 0), desc(1, u32::MAX, 0)]);
        assert!(!a.saturated);
        assert_eq!(a.rows_emitted_sum[&1], 2 * u64::from(u32::MAX));
    }

    #[test]
    fn escaper_matches_receipt_store_shape() {
        assert_eq!(esc("a\"b\\c\n"), "\"a\\\"b\\\\c\\n\"");
    }
}
