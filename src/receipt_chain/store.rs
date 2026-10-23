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

    /// Append one record as a single JSON line.
    ///
    /// # Errors
    /// Returns [`CoreError::SerializationFailed`] if `record` fails to
    /// serialize, or [`CoreError::Io`] if the file can't be opened/written.
    pub fn append(&self, record: &ReceiptRecord) -> Result<(), CoreError> {
        let line = serde_json::to_string(record)
            .map_err(|e| CoreError::SerializationFailed(e.to_string()))?;
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.path)
            .map_err(|e| CoreError::Io(e.to_string()))?;
        writeln!(file, "{line}").map_err(|e| CoreError::Io(e.to_string()))?;
        Ok(())
    }

    /// Load every record in the ledger, in append order. Returns an empty
    /// `Vec` if the ledger file doesn't exist yet.
    ///
    /// # Errors
    /// Returns [`CoreError::Io`] if an existing ledger can't be read, or
    /// [`CoreError::SerializationFailed`] if a line isn't a valid
    /// [`ReceiptRecord`].
    pub fn load_all(&self) -> Result<Vec<ReceiptRecord>, CoreError> {
        if !self.path.exists() {
            return Ok(Vec::new());
        }
        let content =
            std::fs::read_to_string(&self.path).map_err(|e| CoreError::Io(e.to_string()))?;
        content
            .lines()
            .filter(|l| !l.trim().is_empty())
            .map(|l| {
                serde_json::from_str(l).map_err(|e| CoreError::SerializationFailed(e.to_string()))
            })
            .collect()
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
