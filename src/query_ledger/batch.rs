use std::collections::BTreeMap;
use std::fmt;

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

pub(super) fn digest_of(bytes: &[u8]) -> String {
    hex(blake3::hash(bytes).as_bytes())
}

// Hand-rolled escaper, mirroring src/receipt_store.rs conventions verbatim.
pub(super) fn esc(s: &str) -> String {
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
pub(super) fn encode_batch(
    parent: &str,
    first_seq: u64,
    count: u32,
    records: &[Descriptor],
) -> String {
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

pub(super) struct Cur<'a> {
    pub(super) b: &'a [u8],
    pub(super) i: usize,
}

impl Cur<'_> {
    pub(super) fn eat(&mut self, lit: &str) -> Option<()> {
        if self.b[self.i..].starts_with(lit.as_bytes()) {
            self.i += lit.len();
            Some(())
        } else {
            None
        }
    }

    pub(super) fn u(&mut self) -> Option<u64> {
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

    pub(super) fn hex32(&mut self) -> Option<[u8; 32]> {
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

pub(super) fn parse_batch(
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

pub(super) fn encode_agg(a: &BatchAgg) -> String {
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

pub(super) fn parse_agg(bytes: &[u8], batch: &str) -> Result<BatchAgg, LedgerError> {
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
