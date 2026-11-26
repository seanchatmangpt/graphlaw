//! Append-only JSONL persistence for [`ReceiptRecord`]s (port of
//! `graphlaw-core::receipt_store`). One JSON object per line, opened in
//! append+create mode, no in-place rewrites.

use std::fs::OpenOptions;
use std::io::Write as _;
use std::path::{Path, PathBuf};

use super::CoreError;
use super::ReceiptRecord;

/// Default receipts directory when no configured path is available.
pub const DEFAULT_RECEIPTS_DIR: &str = "receipts";

/// Ledger file name within the receipts directory.
pub const LEDGER_FILE_NAME: &str = "receipts.jsonl";

/// Genesis chain hash (32 zero bytes) used when the ledger has no entries
/// yet.
pub const GENESIS_CHAIN_HASH: [u8; 32] = [0u8; 32];

/// Append-only JSONL receipt ledger.
pub struct ReceiptStore {
    path: PathBuf,
}

impl ReceiptStore {
    /// Open (or prepare to create) a store at `<dir>/receipts.jsonl`,
    /// creating `dir` if it doesn't exist yet. Does not create the ledger
    /// file itself until the first [`ReceiptStore::append`].
    ///
    /// # Errors
    /// Returns [`CoreError::Io`] if `dir` cannot be created.
    pub fn open(dir: impl AsRef<Path>) -> Result<Self, CoreError> {
        let dir = dir.as_ref();
        std::fs::create_dir_all(dir).map_err(|e| CoreError::Io(e.to_string()))?;
        Ok(Self {
            path: dir.join(LEDGER_FILE_NAME),
        })
    }

    /// Open the default store ([`DEFAULT_RECEIPTS_DIR`]`/receipts.jsonl`).
    ///
    /// # Errors
    /// See [`ReceiptStore::open`].
    pub fn open_default() -> Result<Self, CoreError> {
        Self::open(DEFAULT_RECEIPTS_DIR)
    }

    /// The ledger file path.
    #[must_use]
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Append one record as a single JSON line, durably.
    ///
    /// Takes an exclusive advisory lock on the ledger for the whole
    /// recover-write-fsync sequence, repairs a torn trailing line (see
    /// [`ReceiptStore::append_with`]), writes the record, and fsyncs the file
    /// (and the parent directory when the ledger was just created).
    ///
    /// # Errors
    /// Returns [`CoreError::SerializationFailed`] if `record` fails to
    /// serialize, or [`CoreError::Io`] if the file can't be locked, written,
    /// synced, or carries mid-file corruption (`RECEIPT-STORE-CORRUPT`).
    pub fn append(&self, record: &ReceiptRecord) -> Result<(), CoreError> {
        self.append_with(|_prev| Ok(record.clone()))
    }

    /// Build and append the next record atomically with respect to other
    /// appenders: `build` receives the chain hash of the current last record
    /// (or [`GENESIS_CHAIN_HASH`]) while the exclusive lock is held, so
    /// concurrent appenders cannot fork the chain.
    ///
    /// Torn-tail law: a trailing fragment with no final newline is the
    /// residue of a crash mid-append. If it parses as a record only the
    /// newline is restored; otherwise it is truncated, but only after every
    /// preceding line has been verified to parse. An unparsable
    /// newline-terminated line is never repaired: the call is refused with
    /// `RECEIPT-STORE-CORRUPT`.
    ///
    /// # Errors
    /// See [`ReceiptStore::append`]; plus any error returned by `build`.
    pub fn append_with(
        &self,
        build: impl FnOnce([u8; 32]) -> Result<ReceiptRecord, CoreError>,
    ) -> Result<(), CoreError> {
        let io = |e: std::io::Error| CoreError::Io(e.to_string());
        // Creation is decided atomically by the filesystem (`create_new`), not
        // by a racy `exists()` probe before the lock.
        let (mut file, created) = match OpenOptions::new()
            .create_new(true)
            .read(true)
            .write(true)
            .open(&self.path)
        {
            Ok(f) => (f, true),
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => (
                OpenOptions::new()
                    .read(true)
                    .write(true)
                    .truncate(false)
                    .open(&self.path)
                    .map_err(io)?,
                false,
            ),
            Err(e) => return Err(io(e)),
        };
        file.lock().map_err(io)?;
        let result = self.append_guarded(&mut file, created, build);
        let unlock = file.unlock().map_err(io);
        result?;
        unlock?;
        Ok(())
    }

    /// Runs while the exclusive lock is held: makes the directory entry
    /// durable when this ledger is new (or still empty), then appends.
    fn append_guarded(
        &self,
        file: &mut std::fs::File,
        created: bool,
        build: impl FnOnce([u8; 32]) -> Result<ReceiptRecord, CoreError>,
    ) -> Result<(), CoreError> {
        let io = |e: std::io::Error| CoreError::Io(e.to_string());
        if created || file.metadata().map_err(io)?.len() == 0 {
            if let Some(parent) = self.path.parent() {
                let parent = if parent.as_os_str().is_empty() {
                    Path::new(".")
                } else {
                    parent
                };
                std::fs::File::open(parent)
                    .and_then(|d| d.sync_all())
                    .map_err(io)?;
            }
        }
        self.append_locked(file, build)
    }

    fn append_locked(
        &self,
        file: &mut std::fs::File,
        build: impl FnOnce([u8; 32]) -> Result<ReceiptRecord, CoreError>,
    ) -> Result<(), CoreError> {
        use std::io::{Read as _, Seek as _, SeekFrom};
        let io = |e: std::io::Error| CoreError::Io(e.to_string());
        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes).map_err(io)?;

        let scan = scan_ledger(&bytes)
            .map_err(|why| CoreError::Io(format!("RECEIPT-STORE-CORRUPT: {why}")))?;
        let complete_len = scan.complete_len;
        let mut records = scan.records;
        let mut prefix: &[u8] = b"";
        match scan.tail {
            Tail::Empty => {}
            Tail::CompleteRecord => prefix = b"\n",
            Tail::Torn => {
                file.set_len(complete_len as u64).map_err(io)?;
                file.sync_all().map_err(io)?;
            }
        }

        let prev = match records.pop() {
            Some(r) => r.chain_hash()?,
            None => GENESIS_CHAIN_HASH,
        };
        let record = build(prev)?;
        let line = serde_json::to_string(&record)
            .map_err(|e| CoreError::SerializationFailed(e.to_string()))?;
        if prefix.is_empty() {
            file.seek(SeekFrom::Start(complete_len as u64)).map_err(io)?;
        } else {
            file.seek(SeekFrom::End(0)).map_err(io)?;
        }
        let mut out = Vec::with_capacity(prefix.len() + line.len() + 1);
        out.extend_from_slice(prefix);
        out.extend_from_slice(line.as_bytes());
        out.push(b'\n');
        file.write_all(&out).map_err(io)?;
        file.sync_all().map_err(io)?;
        Ok(())
    }

    /// Load every record in the ledger, in append order. Returns an empty
    /// `Vec` if the ledger file doesn't exist yet.
    ///
    /// Takes a shared advisory lock so a concurrent [`ReceiptStore::append`]
    /// is never observed mid-write. Torn-tail law (same as
    /// [`ReceiptStore::append_with`]): an unterminated trailing fragment is
    /// crash residue and is ignored (a complete record merely missing its
    /// newline is included); an unparsable newline-terminated line is
    /// mid-file corruption and is refused.
    ///
    /// # Errors
    /// Returns [`CoreError::Io`] if an existing ledger can't be opened,
    /// locked or read, or [`CoreError::SerializationFailed`] if a
    /// newline-terminated line isn't a valid [`ReceiptRecord`].
    pub fn load_all(&self) -> Result<Vec<ReceiptRecord>, CoreError> {
        use std::io::Read as _;
        let io = |e: std::io::Error| CoreError::Io(e.to_string());
        let mut file = match std::fs::File::open(&self.path) {
            Ok(f) => f,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
            Err(e) => return Err(io(e)),
        };
        file.lock_shared().map_err(io)?;
        let mut bytes = Vec::new();
        let read = file.read_to_end(&mut bytes).map_err(io);
        let unlock = file.unlock().map_err(io);
        read?;
        unlock?;
        let scan = scan_ledger(&bytes).map_err(CoreError::SerializationFailed)?;
        Ok(scan.records)
    }

    /// The `chain_hash` of the last record, or [`GENESIS_CHAIN_HASH`] if
    /// the ledger is empty (or doesn't exist yet).
    ///
    /// # Errors
    /// See [`ReceiptStore::load_all`], plus [`CoreError::HexDecodeFailed`]
    /// if the last record's `chain_hash_hex` is malformed.
    pub fn last_chain_hash(&self) -> Result<[u8; 32], CoreError> {
        let records = self.load_all()?;
        match records.last() {
            Some(r) => r.chain_hash(),
            None => Ok(GENESIS_CHAIN_HASH),
        }
    }
}

/// How the bytes after the last newline look.
enum Tail {
    /// The ledger ends in a newline (or is empty).
    Empty,
    /// A complete record that only lost its newline (already in `records`).
    CompleteRecord,
    /// An unparsable fragment: crash residue, not in `records`.
    Torn,
}

struct Scan {
    records: Vec<ReceiptRecord>,
    /// Byte length of the newline-terminated prefix.
    complete_len: usize,
    tail: Tail,
}

/// Parse a ledger image. Every newline-terminated non-blank line must parse
/// (else `Err(why)`); the unterminated tail is classified, never refused.
fn scan_ledger(bytes: &[u8]) -> Result<Scan, String> {
    let complete_len = bytes.iter().rposition(|b| *b == b'\n').map_or(0, |i| i + 1);
    let mut records = Vec::new();
    for (n, line) in bytes[..complete_len].split(|b| *b == b'\n').enumerate() {
        if line.iter().all(u8::is_ascii_whitespace) {
            continue;
        }
        let rec: ReceiptRecord = serde_json::from_slice(line)
            .map_err(|e| format!("line {} unparsable: {e}", n + 1))?;
        records.push(rec);
    }
    let tail_bytes = &bytes[complete_len..];
    let tail = if tail_bytes.is_empty() {
        Tail::Empty
    } else {
        match serde_json::from_slice::<ReceiptRecord>(tail_bytes) {
            Ok(rec) => {
                records.push(rec);
                Tail::CompleteRecord
            }
            Err(_) => Tail::Torn,
        }
    };
    Ok(Scan {
        records,
        complete_len,
        tail,
    })
}
