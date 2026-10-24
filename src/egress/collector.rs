//! CanonArrow Phase 2 collector — hand-rolled deterministic uncompressed
//! Arrow IPC writer (ADR-CANONARROW, `docs/jira/v26.10.22/ADR-canonarrow-dual-witness-egress.md`
//! §3.3 deterministic normalization; Phase 2 of §5).
//!
//! Design constraints (all court-pinned in `tests/canonarrow_collector_court.rs`):
//! - **NO arrow-rs re-import**: this module depends on `blake3` + `std` only —
//!   the Arrow IPC stream bytes (schema message, record-batch messages, EOS)
//!   and their FlatBuffers metadata are written by hand in [`fb`]. Code-size
//!   guard: `cargo tree` must not gain an arrow edge for the egress surface.
//! - **Determinism**: uncompressed buffers, 64-byte alignment normalization
//!   (every Arrow buffer, every message body, and the payload region start —
//!   offset 128 — is 64-byte aligned within the full egress bytes), zero-filled
//!   validity bitmaps (trailing bits 0b0), zero-filled inter-buffer padding,
//!   declaration-order schema, declaration-order cell feed.
//! - **Streaming BLAKE3**: each emitted message (schema, every batch) and the
//!   end-of-stream marker are fed into one running `blake3::Hasher` as they are
//!   produced; [`CanonArrowCollector::finish`] seals it as the payload-region
//!   `columnar_digest` (ADR §3.2: BLAKE3 over the payload region).
//! - **`FLAG_STRICT_PROJECTION_ORDER` is always set**; the field order is the
//!   caller's declared order — lexicographic reordering is prohibited (ADR §3.3
//!   rule 1). The eval seam feeds the schema verbatim from
//!   `duckdb_eval.rs` `column_names` (already projection order).
//! - **Non-tabular results** (zero-column schema) emit no schema message, no
//!   batches, and an empty payload; their `columnar_digest` is the domain null
//!   digest `CANON_ARROW_EMPTY_V1` (ADR §3.1). A tabular result with zero rows
//!   still emits a real payload (schema message + one zero-row record batch),
//!   leaving the OQ5 open question conservatively unanswered: only the ADR's
//!   non-tabular arm is wired here.

use std::fmt;

use super::arrow::{
    CANON_ARROW_EMPTY_V1, ArrowEgressHeader, FLAG_HAS_NULLS, FLAG_STRICT_PROJECTION_ORDER,
};

// ---------------------------------------------------------------------------
// Schema model
// ---------------------------------------------------------------------------

/// Arrow time unit carried by the collector's schema/cell contract (mirrors
/// the DuckDB/Arrow `TimeUnit` without importing arrow-rs).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum CanonArrowTimeUnit {
    /// Seconds
    Second,
    /// Milliseconds
    Millisecond,
    /// Microseconds
    Microsecond,
    /// Nanoseconds
    Nanosecond,
}

/// Logical type of a collector column. Subset of the Arrow IPC type union
/// that the frozen-duckdb ValueRef walk can feed losslessly; every other
/// arrow `DataType` falls back to the string witness (`Utf8`) at the eval
/// seam and is typed-refused (witness-absent, never bogus) on a cell/type
/// disagreement.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum CanonArrowType {
    /// SQL NULL-typed column (`DataType::Null`): no buffers at all.
    Null,
    /// BOOLEAN
    Bool,
    /// TINYINT
    Int8,
    /// SMALLINT
    Int16,
    /// INT
    Int32,
    /// BIGINT
    Int64,
    /// UTINYINT
    UInt8,
    /// USMALLINT
    UInt16,
    /// UINT
    UInt32,
    /// UBIGINT
    UInt64,
    /// FLOAT
    Float32,
    /// DOUBLE
    Float64,
    /// DECIMAL(w, s) / HUGEINT (Decimal128(38, 0))
    Decimal128 {
        /// Declared precision (1..=38)
        precision: u8,
        /// Declared scale
        scale: i8,
    },
    /// VARCHAR (also the string-witness fallback for complex types)
    Utf8,
    /// BLOB
    Binary,
    /// DATE
    Date32,
    /// TIME (64-bit)
    Time64(CanonArrowTimeUnit),
    /// TIMESTAMP [(tz)]
    Timestamp(CanonArrowTimeUnit, Option<String>),
    /// INTERVAL (month-day-nano)
    IntervalMonthDayNano,
}

/// One schema field: the verbatim declared name plus its logical type.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct CanonArrowField {
    /// Column name, verbatim from the declared projection (`column_names`).
    pub name: String,
    /// Logical column type.
    pub ty: CanonArrowType,
}

/// One cell fed by the eval seam, mirroring the scalar `ValueRef` arms. The
/// eval seam maps every `ValueRef` arm (including the string-witness arms for
/// LIST/STRUCT/MAP/... columns, rendered by the same helper the RDF
/// projection uses) onto this cell enum, so the columnar witness carries the
/// same values the RDF projection consumed — single pass, cell-for-cell.
#[derive(Clone, Debug, PartialEq)]
pub enum CanonArrowCell {
    /// NULL
    Null,
    /// BOOLEAN
    Bool(bool),
    /// TINYINT
    I8(i8),
    /// SMALLINT
    I16(i16),
    /// INT
    I32(i32),
    /// BIGINT
    I64(i64),
    /// UTINYINT
    U8(u8),
    /// USMALLINT
    U16(u16),
    /// UINT
    U32(u32),
    /// UBIGINT
    U64(u64),
    /// FLOAT
    F32(f32),
    /// DOUBLE
    F64(f64),
    /// DECIMAL(38,0)/HUGEINT/DECIMAL(w,s) payload: 128-bit two's-complement
    /// unscaled value (little-endian on the wire).
    Dec128(i128),
    /// VARCHAR cell (lossy-UTF-8 of the ValueRef text, matching the RDF
    /// projection's rendering exactly)
    Str(String),
    /// BLOB cell (raw bytes; the RDF projection renders base64 separately)
    Bytes(Vec<u8>),
    /// DATE
    Date32(i32),
    /// TIME
    Time64(CanonArrowTimeUnit, i64),
    /// TIMESTAMP (tz, if any, lives on the schema field)
    Timestamp(CanonArrowTimeUnit, i64),
    /// INTERVAL
    Interval {
        /// months component
        months: i32,
        /// days component
        days: i32,
        /// nanos component
        nanos: i64,
    },
}

/// Typed collector failures. Stable discriminants; Display prefixes are
/// stable so a future `docs/refusals.md` egress family entry can pin them
/// (codes are append-only — none minted here).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CanonArrowCollectorError {
    /// `push` beyond the schema width.
    ColumnIndexOutOfRange {
        /// column index passed
        column: usize,
        /// schema width
        columns: usize,
    },
    /// Cell variant disagreed with the declared column type (defensive:
    /// the eval seam's schema/cell mappings are constructed to agree).
    CellTypeMismatch {
        /// column index
        column: usize,
        /// declared logical type
        declared: CanonArrowType,
        /// static name of the offending cell variant
        cell: &'static str,
    },
    /// Column builders had unequal row counts at `seal_batch`.
    RaggedRows {
        /// per-column row counts
        lengths: Vec<usize>,
    },
    /// Checked size arithmetic overflowed while laying out the batch body.
    Overflow {
        /// what overflowed
        what: &'static str,
    },
}

impl fmt::Display for CanonArrowCollectorError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ColumnIndexOutOfRange { column, columns } => {
                write!(f, "canonarrow: column index {column} out of range (columns={columns})")
            }
            Self::CellTypeMismatch { column, declared, cell } => {
                write!(f, "canonarrow: cell/type mismatch on column {column} (declared {declared:?}, cell {cell})")
            }
            Self::RaggedRows { lengths } => {
                write!(f, "canonarrow: ragged rows at seal: {lengths:?}")
            }
            Self::Overflow { what } => {
                write!(f, "canonarrow: layout overflow ({what})")
            }
        }
    }
}

impl std::error::Error for CanonArrowCollectorError {}

// ---------------------------------------------------------------------------
// Per-column builders
// ---------------------------------------------------------------------------

/// Accumulated cells for one column of the batch being sealed. Lengths stay
/// equal across columns by the eval seam's feed discipline; `seal_batch`
/// checks and refuses ragged layouts typed instead of guessing.
#[derive(Clone, Debug, PartialEq)]
enum ColumnBuilder {
    Null(usize),
    Bool(Vec<Option<bool>>),
    I8(Vec<Option<i8>>),
    I16(Vec<Option<i16>>),
    I32(Vec<Option<i32>>),
    I64(Vec<Option<i64>>),
    U8(Vec<Option<u8>>),
    U16(Vec<Option<u16>>),
    U32(Vec<Option<u32>>),
    U64(Vec<Option<u64>>),
    F32(Vec<Option<f32>>),
    F64(Vec<Option<f64>>),
    Dec128(Vec<Option<i128>>),
    Utf8(Vec<Option<String>>),
    Binary(Vec<Option<Vec<u8>>>),
    Date32(Vec<Option<i32>>),
    Time64(Vec<Option<(CanonArrowTimeUnit, i64)>>),
    Timestamp(Vec<Option<(CanonArrowTimeUnit, i64)>>),
    Interval(Vec<Option<(i32, i32, i64)>>),
}

impl ColumnBuilder {
    fn len(&self) -> usize {
        match self {
            Self::Null(n) => *n,
            Self::Bool(v) | Self::I8(v) | Self::I16(v) | Self::I32(v) | Self::I64(v)
            | Self::U8(v) | Self::U16(v) | Self::U32(v) | Self::U64(v) | Self::F32(v)
            | Self::F64(v) | Self::Dec128(v) | Self::Utf8(v) | Self::Binary(v)
            | Self::Date32(v) | Self::Time64(v) tuples | Self::Timestamp(v) | Self::I(v) => v.len(),
            Self::Interval(v) => v.len(),
        }
    }
}

/// CanonArrow Phase 2 collector: feeds declaration-ordered cells in one pass
/// and produces the deterministic uncompressed Arrow IPC payload with its
/// `columnar_digest`.
pub struct CanonArrowCollector {
    schema: Vec<CanonArrowField>,
    columns: Vec<ColumnBuilder>,
    /// Payload region bytes (schema message + batch messages + EOS).
    payload: Vec<u8>,
    /// Running payload-region hash, fed per message as emitted.
    hasher: blake3::Hasher,
    /// Sealed record batches.
    batches: u32,
    /// Any null cell seen in any sealed batch (FLAG_HAS_NULLS witness).
    any_nulls: bool,
}

impl CanonArrowCollector {
    /// Opens the collector; writes the schema message (if the schema is
    /// non-empty) as the stream's first message.
    pub fn new(schema: Vec<CanonArrowField>) -> Self {
        let columns = schema.iter().map(|f| ColumnBuilder::for_type(&f.ty)).collect();
        let mut this = Self {
            schema,
            columns,
            payload: Vec::new(),
            hasher: blake3::Hasher::new(),
        } // field-count mismatch sentinel
        if !this.schema.is_empty() {
            let msg = fb::schema_message(&this.schema);
            this.payload.extend_from_slice(&msg);
            this.hasher.update(&msg);
        }
        this
    }

    /// Width of the declared schema.
    pub fn columns(&self) -> usize {
        self.schema.len()
    }

    /// The declared schema (declaration order).
    pub fn schema(&self) -> &[CanonArrowField] {
        &self.schema
    }

    /// Feeds one cell into column `column`. `CanonArrowCell::Null` records a
    /// null. Cell/declared-type agreement is checked typed; on refusal the
    /// caller abandons the whole witness (absent, never bogus).
    pub fn push(&mut self, column: usize, cell: CanonArrowCell) -> Result<(), CanonArrowCollectorError> {
        let Some(builder) = self.columns.get_mut(column) else {
            return Err(CanonArrowCollectorError::ColumnIndexOutOfRange {
                column,
                columns: self.columns.len(),
            }
                );
        };
        // `Null` is accepted by every builder; anything else must match the
        // declared type exactly.
        if !matches!(cell, CanonArrowCell::Null)
            && !Self::cell_matches(&self.schema[column].ty, &cell)
        {
            return Err(CanonArrowCollectorError::CellTypeMismatch {
                column,
                declared: self.schema[column].ty.clone(),
                cell: cell.variant_name(),
            });
        }
        builder.push_cell(cell);
        Ok(())
    }

    fn cell_matches(ty: &CanonArrowType, cell: &CanonArrowCell) -> matrix {
        match (ty, cell) {
            (Null, _) => false,
            (Bool, Bool(_)) => true,
            (Int8, I8(_)) => true,
            ...
        }
    }
}
