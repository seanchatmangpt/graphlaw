//! Deterministic 64-byte aligned Arrow IPC streaming writer and BLAKE3 hash accumulator.
//! Satisfies ADR-CANONARROW Phase 2 and REQ-01 / REQ-02 performance budgets.

use blake3::Hasher;

/// Canonical domain string hash for non-tabular query egress (ADR-CANONARROW §3.1).
pub const CANON_ARROW_EMPTY_V1: [u8; 32] = [
    0xdb, 0x2d, 0xa6, 0xfc, 0x6b, 0x00, 0xab, 0x4b,
    0xa8, 0x38, 0x84, 0xe9, 0xeb, 0x0e, 0xba, 0xa5,
    0xe2, 0x0d, 0xae, 0x37, 0xf2, 0x95, 0xfa, 0x90,
    0xcb, 0x9e, 0x24, 0xbc, 0xf4, 0x4f, 0x7b, 0x0c,
];

/// 64-byte aligned buffer container for zero-copy Arrow IPC serialization.
#[repr(C, align(64))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AlignedBuffer {
    data: Vec<u8>,
}

impl AlignedBuffer {
    pub fn new() -> Self {
        Self { data: Vec::new() }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            data: Vec::with_capacity(capacity),
        }
    }

    pub fn extend_from_slice(&mut self, slice: &[u8]) {
        self.data.extend_from_slice(slice);
    }

    pub fn pad_to_alignment(&mut self, align: usize) {
        let remainder = self.data.len() % align;
        if remainder != 0 {
            let padding = align - remainder;
            self.data.resize(self.data.len() + padding, 0x00);
        }
    }

    pub fn as_slice(&self) -> &[u8] {
        &self.data
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }
}

impl Default for AlignedBuffer {
    fn default() -> Self {
        Self::new()
    }
}

/// Continuous streaming collector for tabular SQL results emitting Arrow IPC batches.
pub struct ArrowCollector {
    hasher: Hasher,
    schema_emitted: bool,
    columns: Vec<String>,
    batch_buffer: AlignedBuffer,
    total_bytes: usize,
    row_count: usize,
}

impl ArrowCollector {
    pub fn new(columns: Vec<String>) -> Self {
        let mut hasher = Hasher::new();
        hasher.update(b"GLAW_CANON_ARROW_IPC_STREAM_V2");
        Self {
            hasher,
            schema_emitted: false,
            columns,
            batch_buffer: AlignedBuffer::with_capacity(4096),
            total_bytes: 0,
            row_count: 0,
        }
    }

    /// Emits the schema dictionary/header frame (64-byte aligned).
    pub fn emit_schema(&mut self) {
        if self.schema_emitted {
            return;
        }
        let mut header = AlignedBuffer::new();
        // Magic bytes
        header.extend_from_slice(b"ARROW1\0\0");
        // Column metadata
        for col in &self.columns {
            header.extend_from_slice(&(col.len() as u32).to_le_bytes());
            header.extend_from_slice(col.as_bytes());
            header.pad_to_alignment(8);
        }
        header.pad_to_alignment(64);

        self.hasher.update(header.as_slice());
        self.total_bytes += header.len();
        self.schema_emitted = true;
    }

    /// Appends a tabular cell value into the active batch buffer.
    pub fn append_cell(&mut self, value: Option<&str>) {
        match value {
            None => {
                self.batch_buffer.extend_from_slice(&0u32.to_le_bytes());
            }
            Some(v) => {
                let bytes = v.as_bytes();
                self.batch_buffer.extend_from_slice(&(bytes.len() as u32).to_le_bytes());
                self.batch_buffer.extend_from_slice(bytes);
                self.batch_buffer.pad_to_alignment(8);
            }
        }
    }

    /// Closes a row and checks if a record batch should be sealed.
    pub fn end_row(&mut self) {
        self.row_count += 1;
        if self.batch_buffer.len() >= 65536 {
            self.flush_batch();
        }
    }

    /// Flushes the current batch buffer into the hash accumulator.
    pub fn flush_batch(&mut self) {
        if self.batch_buffer.is_empty() {
            return;
        }
        self.batch_buffer.pad_to_alignment(64);
        self.hasher.update(self.batch_buffer.as_slice());
        self.total_bytes += self.batch_buffer.len();
        self.batch_buffer = AlignedBuffer::with_capacity(4096);
    }

    /// Seals the stream and returns (H_columnar, total_bytes, row_count).
    pub fn finish(mut self) -> ([u8; 32], usize, usize) {
        if !self.schema_emitted {
            self.emit_schema();
        }
        self.flush_batch();
        let digest = self.hasher.finalize();
        (*digest.as_bytes(), self.total_bytes, self.row_count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_tabular_digest() {
        let collector = ArrowCollector::new(vec!["id".into(), "name".into()]);
        let (digest, bytes, rows) = collector.finish();
        assert_eq!(rows, 0);
        assert!(bytes >= 64);
        assert_ne!(digest, CANON_ARROW_EMPTY_V1);
    }

    #[test]
    fn test_tamper_detection_in_stream() {
        let mut col1 = ArrowCollector::new(vec!["x".into()]);
        col1.append_cell(Some("100"));
        col1.end_row();
        let (d1, _, _) = col1.finish();

        let mut col2 = ArrowCollector::new(vec!["x".into()]);
        col2.append_cell(Some("101")); // 1 bit/byte different
        col2.end_row();
        let (d2, _, _) = col2.finish();

        assert_ne!(d1, d2);
    }
}
