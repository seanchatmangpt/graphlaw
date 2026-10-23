use std::collections::BTreeMap;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use super::batch::{
    Aggregate, ChainState, Cur, Descriptor, FLUSH_AGE_MS, FLUSH_COUNT, GENESIS, LedgerError,
    digest_of, encode_agg, encode_batch, esc, fold_batch, parse_agg, parse_batch,
};

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
    /// `crate::evidence_ring` (feature `evidentiary-ring`) seeds the producer sequence from the caller's
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
        let body = format!("{{\"missing_first_seq\":{first},\"missing_last_seq\":{last}}}");
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
        let malformed = || LedgerError::Malformed {
            file: String::from("gaps"),
        };
        let mut out = Vec::new();
        let entries = match fs::read_dir(self.dir.join("gaps")) {
            Ok(e) => e,
            Err(_) => return Ok(out),
        };
        for e in entries {
            let e = e?;
            let name = e.file_name();
            let name = name.to_string_lossy();
            let Some(rest) = name
                .strip_prefix("gap-")
                .and_then(|s| s.strip_suffix(".json"))
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
    if c.eat(",\"aggs\":{").is_some() && c.eat("}").is_none() {
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
        Ok(std::str::from_utf8(raw)
            .map_err(|_| malformed())?
            .to_string())
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
        let (parent, first_seq, _count, records) = parse_batch(&bytes, &format!("{digest}.json"))?;
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
        let mut gs: Vec<&LoadedBatch> = batches.iter().filter(|b| b.parent == GENESIS).collect();
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
    let head = batches
        .last()
        .map(|b| b.digest.clone())
        .unwrap_or_else(|| GENESIS.to_string());
    let tail = batches
        .first()
        .map(|b| b.digest.clone())
        .unwrap_or_else(|| GENESIS.to_string());
    if st.head != head || st.tail != tail || st.batches != batches.len() as u64 {
        return Err(LedgerError::ChainIndex {
            detail: format!(
                "CHAIN {{head {}, tail {}, batches {}}} vs files {{head {}, tail {}, batches {}}}",
                st.head,
                st.tail,
                st.batches,
                head,
                tail,
                batches.len()
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
