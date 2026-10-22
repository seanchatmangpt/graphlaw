//! P28-SIL §3.2 SPSC evidentiary ring (Phase 2 skeleton).
//!
//! Query execution receipts are pushed wait-free into a structurally
//! single-producer/single-consumer ring ([`rtrb`], 4096 slots) and drained by
//! a dedicated thread into an append-only NDJSON ledger whose lines form one
//! linear BLAKE3 `parent -> child` chain, mirroring the conventions of
//! [`crate::receipt_store`]: canonical JSON (fixed key order, no whitespace),
//! content-bound digests, atomic batch writes (single `write_all` + flush,
//! fsync every 16 flushes for a ~1s crash window; a torn tail is detected via
//! the chain, never accepted).
//!
//! Overflow is DROP-NEWEST: the producer never blocks, never takes a lock on
//! the consumer path, and never spins. A dropped push is recorded by the
//! monotonic sequence gap the drainer observes (`DroppedEvidence` ledger
//! lines), so drop accounting is exact without any producer-side I/O.
//!
//! Native-only: no drainer thread on wasm32. Initialization is explicit
//! ([`EvidenceRing::init`] / [`init_global`]) — no lazy default, no
//! surprise-I/O OnceLock: with no explicit init, [`global`] is `None` and the
//! eval seam in `duckdb_eval.rs` is a no-op.
//!
//! SLEEP-CLASS (drainer): the 2 ms empty-cycle park below is SCHEDULING — the
//! drainer genuinely has nothing to do when the ring is empty (P28-SIL §3.2
//! deliberately specifies no condvar so the producer never takes a lock, even
//! indirectly). It is not a coordination sleep.

use std::fmt;
use std::fs::OpenOptions;
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex, OnceLock};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use rtrb::{Consumer, Producer, RingBuffer};

/// Fixed ring capacity: 4096 slots (spec §3.2).
pub const RING_CAPACITY: usize = 4096;
/// Drainer batch bound: drain-to-empty in chunks of at most 256 lines.
const DRAIN_BATCH: usize = 256;
/// Drainer park when the ring is empty (SCHEDULING; see module docs).
const EMPTY_PARK_MS: u64 = 2;
/// fsync cadence: every 16 flushes (~1s crash window per spec).
const FSYNC_EVERY: usize = 16;

/// Exactly 64 B, `#[repr(C)]`, per P28-SIL §3.2. The two pad fields MUST be
/// zero; the verifier treats nonzero pads as a torn-read detector and refuses
/// with a typed [`RingError::TornSlot`].
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RingDescriptor {
    /// Admitted-statement receipt hash H_query = BLAKE3(Canon(G_AST) || P_params).
    pub h_query: [u8; 32],
    /// Rows the query emitted.
    pub rows_emitted: u64,
    /// Execution time in ms, saturating at `u32::MAX`.
    pub execution_ms: u32,
    /// Cache-hit flag (0 = computed; set by the later bench-integration lane).
    pub cached: u8,
    /// MUST be zero (torn-read detector).
    pub pad1: [u8; 3],
    /// Wall-clock push time in ms since the Unix epoch.
    pub timestamp_ms: u64,
    /// MUST be zero (torn-read detector).
    pub pad2: [u8; 8],
}

const _: () = assert!(std::mem::size_of::<RingDescriptor>() == 64);

impl RingDescriptor {
    /// Zero-padded descriptor; the only public constructor keeps pads zero.
    pub fn new(
        h_query: [u8; 32],
        rows_emitted: u64,
        execution_ms: u32,
        cached: u8,
        timestamp_ms: u64,
    ) -> Self {
        Self {
            h_query,
            rows_emitted,
            execution_ms,
            cached,
            timestamp_ms,
            pad1: [0; 3],
            pad2: [0; 8],
        }
    }

    /// Torn-read detector: pads must be zero on every lawful descriptor.
    pub fn pads_zero(&self) -> bool {
        self.pad1 == [0; 3] && self.pad2 == [0; 8]
    }
}

/// One ring slot: producer-assigned monotonic sequence + descriptor (72 B).
#[derive(Debug, Clone, Copy)]
struct EvidenceSlot {
    seq: u64,
    desc: RingDescriptor,
}

/// Typed refusal for the evidentiary ring surface.
///
/// `#[non_exhaustive]`: variants may be added in a minor release, so
/// downstream `match` expressions need a wildcard arm.
#[derive(Debug)]
#[non_exhaustive]
pub enum RingError {
    /// Ledger directory could not be created or the ledger written.
    Io(String),
    /// The ledger does not form one linear BLAKE3 parent -> child chain.
    BrokenChain {
        /// Where the chain breaks.
        detail: String,
    },
    /// A ledger line is not the canonical encoding of a valid event.
    Malformed {
        /// 1-based line number.
        line: usize,
    },
    /// A line carries nonzero pad bytes (torn read).
    TornSlot {
        /// 1-based line number.
        line: usize,
    },
    /// The ledger's sequence coverage is not contiguous under its drop lines.
    SequenceGap {
        /// Where coverage breaks.
        detail: String,
    },
    /// [`EvidenceRing::init_global`] was called twice.
    AlreadyInitialized,
}

impl fmt::Display for RingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RingError::Io(e) => write!(f, "evidentiary ring io: {e}"),
            RingError::BrokenChain { detail } => {
                write!(f, "evidence ledger chain broken: {detail}")
            }
            RingError::Malformed { line } => write!(f, "evidence ledger line {line} malformed"),
            RingError::TornSlot { line } => {
                write!(f, "evidence ledger line {line}: torn slot (nonzero pad)")
            }
            RingError::SequenceGap { detail } => write!(f, "evidence ledger sequence gap: {detail}"),
            RingError::AlreadyInitialized => {
                write!(f, "global evidentiary ring already initialized")
            }
        }
    }
}

impl std::error::Error for RingError {}

impl From<std::io::Error> for RingError {
    fn from(e: std::io::Error) -> Self {
        RingError::Io(e.to_string())
    }
}

/// A drop event, as recorded in `dropped` ledger lines.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DroppedEvidence {
    /// Number of consecutive pushes refused (drop-newest).
    pub dropped_count: u64,
    /// H_query of the first slot received after the gap (the refused
    /// sequence's own payloads are, by drop-newest construction, unavailable;
    /// all-zero when the gap runs to shutdown).
    pub refused_h_query: [u8; 32],
    /// Wall-clock ms at which the drainer observed the gap.
    pub refused_at_ms: u64,
}

/// Wall-clock ms since the Unix epoch (saturating at 0 pre-epoch).
pub fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

/// Genesis parent digest (all-zero hex).
const GENESIS: &str = "0000000000000000000000000000000000000000000000000000000000000000";

/// Canonical NDJSON ledger line: fixed key order, no whitespace.
/// `child = BLAKE3(parent_hex || body_without_child)` — the receipt_store
/// parent -> child convention. Returns (full line with trailing newline, child).
fn encode_line(seq: u64, kind: &str, body: &str, parent: &str) -> (String, String) {
    let body = format!("{{\"seq\":{seq},\"kind\":\"{kind}\",{body}\"parent\":\"{parent}\"");
    let mut h = blake3::Hasher::new();
    h.update(parent.as_bytes());
    h.update(body.as_bytes());
    let child = hex(h.finalize().as_bytes());
    let line = format!("{body},\"child\":\"{child}\"}}\n");
    (line, child)
}

/// Producer handle. [`EvidenceRing::push`] is wait-free on the hot path
/// (rtrb `try_push` + one relaxed atomic fetch_add); on a full ring the
/// newest slot is dropped and later surfaced as a `dropped` ledger line.
pub struct EvidenceRing {
    dir: PathBuf,
    /// When set at init, the drainer ALSO records every drained batch into a
    /// [`crate::query_ledger::QueryLedger`] under this directory (dual write:
    /// the NDJSON chain here stays the crash-evidence surface; the query
    /// ledger is the content-addressed batch surface).
    ledger_dir: Option<PathBuf>,
    /// Single producer by contract (SPSC); this mutex is uncontended on the
    /// hot path and never held across I/O.
    producer: Mutex<Option<Producer<EvidenceSlot>>>,
    /// Taken exactly once by the drainer thread at init.
    consumer: Mutex<Option<Consumer<EvidenceSlot>>>,
    seq_counter: Arc<AtomicU64>,
    dropped: AtomicU64,
    paused_flag: Arc<AtomicBool>,
    shutdown_flag: Arc<AtomicBool>,
    /// Last ledger-side error (drainer threads cannot return errors; surfaced
    /// instead of laundered — [`EvidenceRing::last_ledger_error`]).
    ledger_error: Arc<Mutex<Option<String>>>,
    drainer: Mutex<Option<std::thread::JoinHandle<()>>>,
}

impl EvidenceRing {
    /// Explicitly initializes a ring whose drainer thread appends to
    /// `<dir>/evidence.ndjson`. No lazy default exists: nothing touches the
    /// filesystem until this is called.
    pub fn init(dir: impl AsRef<Path>) -> Result<Arc<Self>, RingError> {
        Self::init_inner(dir, None, 0)
    }

    /// Like [`EvidenceRing::init`], but the drainer additionally records every
    /// drained batch into a content-addressed
    /// [`crate::query_ledger::QueryLedger`] under `ledger_dir` (joint
    /// ring -> ledger pipeline; ring sequence gaps become ledger gap markers).
    pub fn init_with_ledger(
        dir: impl AsRef<Path>,
        ledger_dir: impl AsRef<Path>,
    ) -> Result<Arc<Self>, RingError> {
        let ledger_dir = ledger_dir.as_ref().to_path_buf();
        std::fs::create_dir_all(&ledger_dir)?;
        Self::init_inner(dir, Some(ledger_dir), 0)
    }

    /// Like [`EvidenceRing::init_with_ledger`], but resumes an interrupted
    /// ring -> ledger pipeline: the caller supplies the producer's attempted
    /// push count (the watermark its monotonic counter held at crash). The
    /// drainer opens the joint ledger via
    /// [`crate::query_ledger::QueryLedger::open_with_producer`], which
    /// persists the crash loss as gap markers (`complete: false` on periods
    /// containing them), and the NDJSON chain gets one matching startup drop
    /// line — never silent renumbering of the sequence space.
    pub fn init_with_ledger_restart(
        dir: impl AsRef<Path>,
        ledger_dir: impl AsRef<Path>,
        producer_attempted: u64,
    ) -> Result<Arc<Self>, RingError> {
        let ledger_dir = ledger_dir.as_ref().to_path_buf();
        std::fs::create_dir_all(&ledger_dir)?;
        Self::init_inner(dir, Some(ledger_dir), producer_attempted)
    }

    fn init_inner(
        dir: impl AsRef<Path>,
        ledger_dir: Option<PathBuf>,
        watermark: u64,
    ) -> Result<Arc<Self>, RingError> {
        let dir = dir.as_ref();
        std::fs::create_dir_all(dir)?;
        let (producer, consumer) = RingBuffer::new(RING_CAPACITY);
        let ring = Arc::new(Self {
            dir: dir.to_path_buf(),
            ledger_dir,
            producer: Mutex::new(Some(producer)),
            consumer: Mutex::new(Some(consumer)),
            // F5: the sequence counter is seeded from the producer watermark,
            // so a restart-mid-stream continues the global sequence — never
            // silent renumbering. Fresh rings seed at 0.
            seq_counter: Arc::new(AtomicU64::new(watermark)),
            dropped: AtomicU64::new(0),
            paused_flag: Arc::new(AtomicBool::new(false)),
            shutdown_flag: Arc::new(AtomicBool::new(false)),
            ledger_error: Arc::new(Mutex::new(None)),
            drainer: Mutex::new(None),
        });
        let consumer = consumer_taken(&ring.consumer);
        let handle = {
            let dir = ring.dir.clone();
            let ledger_dir = ring.ledger_dir.clone();
            let ledger_error = Arc::clone(&ring.ledger_error);
            let paused = Arc::clone(&ring.paused_flag);
            let shutdown = Arc::clone(&ring.shutdown_flag);
            let seq = Arc::clone(&ring.seq_counter);
            // Drainer: batch-to-empty <= 256, park 2 ms when empty, fsync
            // every 16 flushes.
            std::thread::Builder::new()
                .name("evidence-drainer".into())
                .spawn(move || {
                    drain_loop(
                        consumer, &dir, ledger_dir, &ledger_error, &paused, &shutdown, &seq,
                        watermark,
                    )
                })?
        };
        *ring.drainer.lock().expect("drainer lock poisoned") = Some(handle);
        Ok(ring)
    }

    /// Installs a ring as the process-global ring consumed by the eval seam
    /// in `duckdb_eval.rs`. Explicit, never lazy; refuses a second install.
    pub fn init_global(dir: impl AsRef<Path>) -> Result<(), RingError> {
        let ring = Self::init(dir)?;
        GLOBAL.set(ring).map_err(|_| RingError::AlreadyInitialized)
    }

    /// Like [`EvidenceRing::init_global`], but the global ring's drainer also
    /// records into a content-addressed query ledger under `ledger_dir` (joint
    /// ring -> ledger pipeline). Required for the `capabilities` response to
    /// surface a nonzero `evidentiary_ledger_gaps` (abi.rs reads the global
    /// ring's ledger gap markers).
    pub fn init_global_with_ledger(
        dir: impl AsRef<Path>,
        ledger_dir: impl AsRef<Path>,
    ) -> Result<(), RingError> {
        let ring = Self::init_with_ledger(dir, ledger_dir)?;
        GLOBAL.set(ring).map_err(|_| RingError::AlreadyInitialized)
    }

    /// The process-global ring, if one was explicitly initialized.
    pub fn global() -> Option<&'static Arc<Self>> {
        GLOBAL.get()
    }

    /// Path of the ledger file.
    pub fn ledger_path(&self) -> PathBuf {
        self.dir.join("evidence.ndjson")
    }

    /// Directory of the joint content-addressed query ledger, when this ring
    /// was initialized with [`EvidenceRing::init_with_ledger`].
    pub fn ledger_dir(&self) -> Option<&Path> {
        self.ledger_dir.as_deref()
    }

    /// Last error the drainer hit while recording into the joint query
    /// ledger, if any (never laundered; `None` = healthy).
    pub fn last_ledger_error(&self) -> Option<String> {
        self.ledger_error
            .lock()
            .expect("ledger error lock poisoned")
            .clone()
    }

    /// Total pushes attempted (buffered + dropped).
    pub fn pushed_total(&self) -> u64 {
        self.seq_counter.load(Ordering::Relaxed)
    }


    /// Total pushes refused drop-newest (mirrored by `dropped` ledger lines).
    pub fn dropped_total(&self) -> u64 {
        self.dropped.load(Ordering::Relaxed)
    }

    /// Operational backpressure valve: when set, the drainer stops draining
    /// (courts the never-block property against a stalled drainer).
    pub fn set_drainer_paused(&self, paused: bool) {
        self.paused_flag.store(paused, Ordering::Relaxed);
    }

    /// Pushes one descriptor. NEVER blocks and NEVER spins; returns
    /// [`PushOutcome::DroppedNewest`] when the ring is full (drop-newest).
    pub fn push(&self, desc: RingDescriptor) -> PushOutcome {
        let seq = self.seq_counter.fetch_add(1, Ordering::Relaxed) + 1;
        let mut guard = self.producer.lock().expect("producer lock poisoned");
        let Some(producer) = guard.as_mut() else {
            drop(guard);
            self.dropped.fetch_add(1, Ordering::Relaxed);
            return PushOutcome::DroppedNewest;
        };
        match producer.push(EvidenceSlot { seq, desc }) {
            Ok(()) => PushOutcome::Buffered,
            Err(_full) => {
                drop(guard);
                self.dropped.fetch_add(1, Ordering::Relaxed);
                PushOutcome::DroppedNewest
            }
        }
    }

    /// Stops the drainer after it drains the remaining slots and finalizes
    /// the ledger (a trailing drop line covers pushes refused after the last
    /// drained slot, so the ledger covers every attempted push). Idempotent.
    pub fn close(&self) {
        if !self.shutdown_flag.swap(true, Ordering::Relaxed) {
            if let Some(h) = self.drainer.lock().expect("drainer lock poisoned").take() {
                let _ = h.join();
            }
        }
    }
}

impl Drop for EvidenceRing {
    /// Drop obligation: drain-to-empty + final flush. [`EvidenceRing::close`]
    /// is idempotent, so an explicit close followed by drop is a no-op second
    /// pass. (The process-global ring in [`GLOBAL`] is a static and never
    /// drops.)
    fn drop(&mut self) {
        self.close();
    }
}

/// Outcome of [`EvidenceRing::push`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PushOutcome {
    /// Slot buffered for the drainer.
    Buffered,
    /// Ring full: newest slot refused (drop-newest); the sequence gap is
    /// recorded by the drainer as [`DroppedEvidence`].
    DroppedNewest,
}

static GLOBAL: OnceLock<Arc<EvidenceRing>> = OnceLock::new();

fn consumer_taken(cell: &Mutex<Option<Consumer<EvidenceSlot>>>) -> Consumer<EvidenceSlot> {
    cell.lock()
        .expect("consumer lock poisoned")
        .take()
        .expect("consumer taken exactly once")
}

fn drop_line(
    seq: u64,
    dropped_count: u64,
    refused_h_query: &[u8; 32],
    parent: &str,
) -> (String, String) {
    encode_line(
        seq,
        "dropped",
        &format!(
            "\"dropped_count\":{},\"refused_h_query\":\"{}\",\"refused_at_ms\":{},",
            dropped_count,
            hex(refused_h_query),
            now_ms()
        ),
        parent,
    )
}

fn drain_loop(
    mut consumer: Consumer<EvidenceSlot>,
    dir: &Path,
    ledger_dir: Option<PathBuf>,
    ledger_error: &Mutex<Option<String>>,
    paused: &AtomicBool,
    shutdown: &AtomicBool,
    seq: &AtomicU64,
    producer_attempted: u64,
) {
    let path = dir.join("evidence.ndjson");
    let file = match OpenOptions::new().create(true).append(true).open(&path) {
        Ok(f) => f,
        Err(_) => return, // ledger unavailable: in-memory counters still record
    };
    let mut out = BufWriter::new(file);
    let mut parent = String::from(GENESIS);
    // Restart watermark: the highest sequence the NDJSON ledger shows as
    // covered (evidence + drop lines). On a restart-mid-stream this is the
    // previous session's true producer watermark; a new drainer reconciles
    // the cold ledger against it at open so lost drain coverage is persisted
    // as a gap marker (never backfilled, never silently renumbered).
    let mut restart_seq: u64 = 0;
    // Resume: an existing ledger's last child becomes the chain head.
    if let Ok(existing) = std::fs::read_to_string(&path) {
        if let Some(last) = existing.lines().next_back() {
            if let Some(child) = last
                .rsplit("\"child\":\"")
                .next()
                .and_then(|r| r.split('"').next())
            {
                parent = child.to_string();
            }
            restart_seq = last
                .split("\"seq\":")
                .nth(1)
                .and_then(|r| r.split(',').next())
                .and_then(|v| v.parse().ok())
                .unwrap_or(0);
        }
    }
    // Joint pipeline: the content-addressed query ledger is owned exclusively
    // by this thread (no locks on the ledger from producer or court threads).
    let mut ledger = ledger_dir.map(|d| (d, None::<crate::query_ledger::QueryLedger>));
    let mut batch: Vec<crate::query_ledger::Descriptor> = Vec::with_capacity(DRAIN_BATCH);
    let open_ledger = |slot: &mut Option<crate::query_ledger::QueryLedger>,
                           d: &Path,
                           err: &Mutex<Option<String>>,
                           producer_next_seq: u64| {
        // Reconcile at open against the true producer watermark: the higher
        // of the resumed NDJSON coverage watermark (the previous session's
        // last covered sequence) and the caller-supplied producer attempted
        // count (`init_with_ledger_restart`). NOT the live seq counter:
        // sequences minted before open are still buffered in the ring and
        // will drain — declaring them lost double-covers them (witnessed:
        // joint-court delta 911 between gap coverage and dropped_total when
        // the live counter was used).
        match crate::query_ledger::QueryLedger::open_with_producer(d, producer_next_seq) {
            Ok(l) => *slot = Some(l),
            Err(e) => {
                *err.lock().expect("ledger error lock poisoned") = Some(e.to_string());
            }
        }
    };
    // F5 (single-cover law): the cold ledger reconciles against the NDJSON
    // coverage watermark only. The caller-supplied producer attempted count
    // (`init_with_ledger_restart`) exceeds it by exactly the in-flight crash
    // loss and is disclosed ONCE — one ledger gap marker + one NDJSON startup
    // drop line, both in the same ring space the in-drain drop path uses.
    // Feeding it to open_with_producer as well double-covers the same loss in
    // a second coordinate space (ledger count-space vs ring seq-space).
    let mut expected_seq: u64 = restart_seq;
    let mut flushes = 0usize;
    let mut buf = String::new();
    if let Some((d, slot)) = ledger.as_mut() {
        open_ledger(slot, d, ledger_error, restart_seq);
        if producer_attempted > restart_seq {
            if let Some(l) = slot.as_mut() {
                if let Err(e) = l.record_gap(restart_seq + 1, producer_attempted) {
                    *ledger_error.lock().expect("ledger error lock poisoned") =
                        Some(e.to_string());
                }
            }
        }
    }
    if producer_attempted > restart_seq {
        // Startup drop line: crash loss between sessions, disclosed before
        // the first resumed slot so NDJSON coverage stays exact.
        let count = producer_attempted - restart_seq;
        let (line, child) = drop_line(restart_seq + 1, count, &[0u8; 32], &parent);
        buf.push_str(&line);
        parent = child;
        expected_seq = producer_attempted;
        let _ = out.write_all(buf.as_bytes());
        let _ = out.flush();
        buf.clear();
    }

    loop {
        // Shutdown always drains-to-empty: the pause valve never holds slots
        // hostage past close (Drop obligation).
        let stopping = shutdown.load(Ordering::Relaxed);
        if paused.load(Ordering::Relaxed) && !stopping {
            let _ = out.flush();
            std::thread::sleep(Duration::from_millis(EMPTY_PARK_MS));
            continue;
        }
        let mut drained = 0usize;
        buf.clear();
        batch.clear();
        while drained < DRAIN_BATCH {
            match consumer.pop() {
                Ok(slot) => {
                    if slot.seq > expected_seq + 1 {
                        // Drop-newest gap: NDJSON drop line + ledger gap
                        // marker. The pending batch flushes first so the
                        // ledger never reorders records around the gap.
                        if let Some((_, Some(l))) = ledger.as_mut() {
                            if let Err(e) = l.flush() {
                                *ledger_error.lock().expect("ledger error lock poisoned") =
                                    Some(e.to_string());
                            }
                        }
                        let count = slot.seq - expected_seq - 1;
                        if let Some((_, Some(l))) = ledger.as_mut() {
                            if let Err(e) =
                                l.record_gap(expected_seq + 1, slot.seq - 1)
                            {
                                *ledger_error.lock().expect("ledger error lock poisoned") =
                                    Some(e.to_string());
                            }
                        }
                        let (line, child) =
                            drop_line(expected_seq + 1, count, &slot.desc.h_query, &parent);
                        buf.push_str(&line);
                        parent = child;
                    }
                    expected_seq = slot.seq;
                    batch.push(crate::query_ledger::Descriptor::from(slot.desc));
                    let (line, child) = encode_line(
                        slot.seq,
                        "evidence",
                        &format!(
                            "\"h_query\":\"{}\",\"rows_emitted\":{},\"execution_ms\":{},\"cached\":{},\"timestamp_ms\":{},\"pads\":{},",
                            hex(&slot.desc.h_query),
                            slot.desc.rows_emitted,
                            slot.desc.execution_ms,
                            slot.desc.cached,
                            slot.desc.timestamp_ms,
                            u8::from(!slot.desc.pads_zero())
                        ),
                        &parent,
                    );
                    buf.push_str(&line);
                    parent = child;
                    drained += 1;
                }
                Err(_) => break, // ring empty
            }
        }
        if drained > 0 {
            if let Some((_, Some(l))) = ledger.as_mut() {
                if let Err(e) = l.record_batch(&batch) {
                    *ledger_error.lock().expect("ledger error lock poisoned") =
                        Some(e.to_string());
                }
            }
            if out.write_all(buf.as_bytes()).is_ok() {
                let _ = out.flush();
                flushes += 1;
                if flushes % FSYNC_EVERY == 0 {
                    let _ = out.get_ref().sync_data();
                }
            }
        } else if stopping {
            break;
        } else {
            std::thread::sleep(Duration::from_millis(EMPTY_PARK_MS));
        }
    }

    // Finalize: trailing drops (refused after the last drained slot) become
    // one closing drop line so coverage over attempted pushes is exact; the
    // query ledger gets the matching gap marker and a final flush.
    let final_seq = seq.load(Ordering::Relaxed);
    if final_seq > expected_seq {
        if let Some((_, Some(l))) = ledger.as_mut() {
            if let Err(e) = l.record_gap(expected_seq + 1, final_seq) {
                *ledger_error.lock().expect("ledger error lock poisoned") = Some(e.to_string());
            }
        }
        let count = final_seq - expected_seq;
        let (line, _child) = drop_line(expected_seq + 1, count, &[0u8; 32], &parent);
        let _ = out.write_all(line.as_bytes());
    }
    let _ = out.flush();
    let _ = out.get_ref().sync_data();
    if let Some((_, Some(l))) = ledger.as_mut() {
        if let Err(e) = l.flush() {
            *ledger_error.lock().expect("ledger error lock poisoned") = Some(e.to_string());
        }
    }
}

fn decode_u64_field(body: &str, key: &str) -> Option<u64> {
    let marker = format!("\"{key}\":");
    let i = body.find(&marker)? + marker.len();
    let rest = &body[i..];
    let end = rest.find(',').unwrap_or(rest.len());
    rest[..end].parse().ok()
}

fn decode_hex_field<'a>(body: &'a str, key: &str) -> Option<&'a str> {
    let marker = format!("\"{key}\":\"");
    let i = body.find(&marker)? + marker.len();
    let rest = &body[i..];
    let end = rest.find('"')?;
    rest.get(..end)
}

/// Re-reads the ledger under `dir`, recomputes every child digest, checks the
/// linear parent -> child chain, the pad-zero detector, and that evidence +
/// drop lines cover every attempted sequence exactly once. Returns
/// `(evidence_lines, dropped_lines, dropped_total)`.
pub fn verify(dir: impl AsRef<Path>) -> Result<(usize, usize, u64), RingError> {
    let path = dir.as_ref().join("evidence.ndjson");
    let text = std::fs::read_to_string(&path).map_err(|e| RingError::Io(e.to_string()))?;
    let mut parent = String::from(GENESIS);
    let mut expected_seq: u64 = 0;
    let mut evidence = 0usize;
    let mut dropped_lines = 0usize;
    let mut dropped_total = 0u64;
    for (idx, line) in text.lines().enumerate() {
        let line_no = idx + 1;
        let (body, child) = match line.strip_suffix("\"}").and_then(|b| b.rsplit_once(",\"child\":\"")) {
            Some((b, c)) if !c.contains('"') => (b, c),
            _ => return Err(RingError::Malformed { line: line_no }),
        };
        let mut h = blake3::Hasher::new();
        h.update(parent.as_bytes());
        h.update(body.as_bytes());
        if hex(h.finalize().as_bytes()) != child {
            return Err(RingError::BrokenChain {
                detail: format!("line {line_no}: child digest mismatch"),
            });
        }
        let seq = decode_u64_field(body, "seq").ok_or(RingError::Malformed { line: line_no })?;
        let kind = decode_hex_field(body, "kind").ok_or(RingError::Malformed { line: line_no })?;
        if kind == "evidence" {
            let hq = decode_hex_field(body, "h_query").ok_or(RingError::Malformed { line: line_no })?;
            if hq.len() != 64 {
                return Err(RingError::Malformed { line: line_no });
            }
            if seq != expected_seq + 1 {
                return Err(RingError::SequenceGap {
                    detail: format!("line {line_no}: evidence seq {seq}, expected {}", expected_seq + 1),
                });
            }
            evidence += 1;
            let pads = decode_u64_field(body, "pads").ok_or(RingError::Malformed { line: line_no })?;
            if pads != 0 {
                return Err(RingError::TornSlot { line: line_no });
            }
        } else if kind == "dropped" {
            let count =
                decode_u64_field(body, "dropped_count").ok_or(RingError::Malformed { line: line_no })?;
            let hq = decode_hex_field(body, "refused_h_query")
                .ok_or(RingError::Malformed { line: line_no })?;
            if count == 0 || hq.len() != 64 {
                return Err(RingError::Malformed { line: line_no });
            }
            if seq != expected_seq + 1 {
                return Err(RingError::SequenceGap {
                    detail: format!("line {line_no}: drop seq {seq}, expected {}", expected_seq + 1),
                });
            }
            expected_seq += count - 1;
            dropped_lines += 1;
            dropped_total += count;
        } else {
            return Err(RingError::Malformed { line: line_no });
        }
        expected_seq += 1;
        parent = child.to_string();
    }
    Ok((evidence, dropped_lines, dropped_total))
}
