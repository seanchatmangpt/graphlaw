//! CanonArrow egress wire protocol — Phase 1 of ADR-CANONARROW
//! (`docs/jira/v26.10.22/ADR-CANONARROW-canonarrow.md`).
//!
//! This module defines only the guest-side memory wire header and the
//! domain null digest arm. It is not yet wired into `duckdb_eval`
//! (Phase 2/3).

/// Wire header for the CanonArrow dual-witness columnar egress payload
/// (ADR §3.2). 128 bytes total, 64-byte aligned.
///
/// Layout is fixed: the struct is `#[repr(C, align(64))]` and the field
/// order below is the wire order. `total_byte_len` covers payload + 128
/// header + 64-byte trailing padding.
#[repr(C, align(64))]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ArrowEgressHeader {
    /// Wire magic, `b"GLAWARRW"`.
    pub magic: [u8; 8], // b"GLAWARRW"
    /// Wire schema version (`0x0002`).
    pub schema_version: u16, // 0x0002
    /// Flag bits (see `FLAG_*` constants).
    pub flags: u16, // bit0 HasNulls, bit1 BigEndian, bit2 StrictProjectionOrder
    /// Number of record batches in the payload.
    pub batch_count: u32,
    /// Byte offset of the payload region (always 128).
    pub payload_offset: u32, // always 128
    /// Length of the payload region in bytes.
    pub payload_byte_len: u64,
    /// Payload + 128 header + 64-byte trailing padding.
    pub total_byte_len: u64,
    /// BLAKE3 over the triples-side witness.
    pub triples_digest: [u8; 32],
    /// BLAKE3 over the payload region.
    pub columnar_digest: [u8; 32], // BLAKE3 over payload region
    // ADR §3.2 lists `reserved: [u8; 36]`, but the ADR field list cannot
    // produce a 128-byte `#[repr(C, align(64))]` struct: repr(C) natural
    // alignment pads 4 bytes after `payload_offset` (u64 fields must sit at
    // 8-aligned offsets), so fields + padding sum to 104 before `reserved`,
    // and 128 is the next multiple of 64. `reserved` is 24 bytes here so the
    // struct is exactly 128; the *wire* reserved region is offsets 100..128
    // in `to_bytes`/`from_bytes` either way.
    /// Reserved; zero-filled on the wire (offsets 100..128).
    pub reserved: [u8; 24],
}

impl ArrowEgressHeader {
    /// Total header size in bytes.
    pub const SIZE: usize = 128;
    /// Wire magic bytes.
    pub const MAGIC: [u8; 8] = *b"GLAWARRW";
    /// Wire schema version.
    pub const SCHEMA_VERSION: u16 = 0x0002;
    /// Byte offset of the payload region.
    pub const PAYLOAD_OFFSET: u32 = 128;

    /// flags bit0
    pub const FLAG_HAS_NULLS: u16 = 1 << 0;
    /// flags bit1
    pub const FLAG_BIG_ENDIAN: u16 = 1 << 1;
    /// flags bit2
    pub const FLAG_STRICT_PROJECTION_ORDER: u16 = 1 << 2;

    /// Serialize to the 128-byte little-endian wire form.
    ///
    /// Digests and `magic`/`reserved` are byte strings and endian-neutral;
    /// every integer field is written little-endian so the header has one
    /// canonical byte form on every target.
    pub fn to_bytes(&self) -> [u8; Self::SIZE] {
        let mut out = [0u8; Self::SIZE];
        out[0..8].copy_from_slice(&self.magic);
        out[8..10].copy_from_slice(&self.schema_version.to_le_bytes());
        out[10..12].copy_from_slice(&self.flags.to_le_bytes());
        out[12..16].copy_from_slice(&self.batch_count.to_le_bytes());
        out[16..20].copy_from_slice(&self.payload_offset.to_le_bytes());
        out[20..28].copy_from_slice(&self.payload_byte_len.to_le_bytes());
        out[28..36].copy_from_slice(&self.total_byte_len.to_le_bytes());
        out[36..68].copy_from_slice(&self.triples_digest);
        out[68..100].copy_from_slice(&self.columnar_digest);
        // out[100..128] stays zero: reserved zero-fill, per §3.2.
        out
    }

    /// Deserialize from the 128-byte wire form.
    ///
    /// Typed refusal on a bad magic / schema version / payload offset /
    /// non-zero reserved region, per the repo's typed-refusal convention
    /// (`docs/refusals.md` style); returns the canonical error string so
    /// callers can map it to a stable code without a custom error type in
    /// this phase.
    pub fn from_bytes(bytes: &[u8; Self::SIZE]) -> Result<Self, ArrowEgressHeaderError> {
        if bytes[0..8] != Self::MAGIC {
            return Err(ArrowEgressHeaderError::BadMagic);
        }
        let schema_version = u16::from_le_bytes(bytes[8..10].try_into().unwrap());
        if schema_version != Self::SCHEMA_VERSION {
            return Err(ArrowEgressHeaderError::BadSchemaVersion(schema_version));
        }
        let flags = u16::from_le_bytes(bytes[10..12].try_into().unwrap());
        if flags & !Self::FLAGS_MASK != 0 {
            return Err(ArrowEgressHeaderError::BadFlags(flags));
        }
        let batch_count = u32::from_le_bytes(bytes[12..16].try_into().unwrap());
        let payload_offset = u32::from_le_bytes(bytes[16..20].try_into().unwrap());
        if payload_offset != Self::PAYLOAD_OFFSET {
            return Err(ArrowEgressHeaderError::BadPayloadOffset(payload_offset));
        }
        let payload_byte_len = u64::from_le_bytes(bytes[20..28].try_into().unwrap());
        let total_byte_len = u64::from_le_bytes(bytes[28..36].try_into().unwrap());
        if total_byte_len
            != payload_byte_len as u64 + Self::SIZE as u64 + 64
        {
            return Err(ArrowEgressHeaderError::BadTotalLen {
                payload_byte_len,
                total_byte_len,
            });
        }
        let mut triples_digest = [0u8; 32];
        triples_digest.copy_from_slice(&bytes[36..68]);
        let mut columnar_digest = [0u8; 32];
        columnar_digest.copy_from_slice(&bytes[68..100]);
        if bytes[100..128].iter().any(|&b| b != 0) {
            return Err(ArrowEgressHeaderError::ReservedNotZero);
        }
        Ok(Self {
            magic: Self::MAGIC,
            schema_version,
            flags,
            batch_count,
            payload_offset,
            payload_byte_len,
            total_byte_len,
            triples_digest,
            columnar_digest,
            reserved: [0u8; 24],
        })
    }
}

impl ArrowEgressHeader {
    /// Mask of the defined flag bits (bit0..bit2).
    pub const FLAGS_MASK: u16 =
        Self::FLAG_HAS_NULLS | Self::FLAG_BIG_ENDIAN | Self::FLAG_STRICT_PROJECTION_ORDER;
}

/// Typed errors for header admission (stable discriminants; codes to be
/// appended to `docs/refusals.md` when the egress path is wired in).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ArrowEgressHeaderError {
    /// Wire magic did not match `GLAWARRW`.
    BadMagic,
    /// Unknown schema version.
    BadSchemaVersion(u16),
    /// Undefined flag bits set.
    BadFlags(u16),
    /// Payload offset was not 128.
    BadPayloadOffset(u32),
    /// `total_byte_len` inconsistent with `payload_byte_len`.
    BadTotalLen {
        /// Payload region length.
        payload_byte_len: u64,
        /// Declared total length.
        total_byte_len: u64,
    },
    /// Reserved region was not zero-filled.
    ReservedNotZero,
}

/// Domain null digest: `H_columnar_empty = BLAKE3(b"CANON_ARROW_EMPTY_V1")`
/// for non-tabular results (ADR §3.1).
///
/// `blake3::hash` is not `const fn`, so this is a `OnceLock` rather than a
/// `const`/`static` — the value is computed exactly once per process and
/// never recomputed.
pub fn canon_arrow_empty_v1() -> &'static [u8; 32] {
    static DIGEST: std::sync::OnceLock<[u8; 32]> = std::sync::OnceLock::new();
    DIGEST.get_or_init(|| *blake3::hash(b"CANON_ARROW_EMPTY_V1").as_bytes())
}

/// Convenience alias matching the ADR's `CANON_ARROW_EMPTY_V1` name.
pub fn CANON_ARROW_EMPTY_V1() -> &'static [u8; 32] {
    canon_arrow_empty_v1()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn header_size_is_128() {
        assert_eq!(
            std::mem::size_of::<ArrowEgressHeader>(),
            ArrowEgressHeader::SIZE
        );
        assert_eq!(std::mem::size_of::<ArrowEgressHeader>(), 128);
    }

    #[test]
    fn header_align_is_64() {
        assert_eq!(std::mem::align_of::<ArrowEgressHeader>(), 64);
    }

    #[test]
    fn serialize_deserialize_round_trip() {
        let h = ArrowEgressHeader {
            magic: ArrowEgressHeader::MAGIC,
            schema_version: ArrowEgressHeader::SCHEMA_VERSION,
            flags: ArrowEgressHeader::FLAG_HAS_NULLS
                | ArrowEgressHeader::FLAG_STRICT_PROJECTION_ORDER,
            batch_count: 3,
            payload_offset: ArrowEgressHeader::PAYLOAD_OFFSET,
            payload_byte_len: 0xDEAD_BEEF_1234_5678,
            total_byte_len: 0xDEAD_BEEF_1234_5678 + 128 + 64,
            triples_digest: [0x11; 32],
            columnar_digest: [0x22; 32],
            reserved: [0u8; 24],
        };
        let bytes = h.to_bytes();
        assert_eq!(bytes.len(), 128);
        let back = ArrowEgressHeader::from_bytes(&bytes).expect("round trip");
        assert_eq!(back, h);
    }

    #[test]
    fn reserved_is_zero_in_wire_form() {
        let h = ArrowEgressHeader {
            magic: ArrowEgressHeader::MAGIC,
            schema_version: ArrowEgressHeader::SCHEMA_VERSION,
            flags: 0,
            batch_count: 0,
            payload_offset: 128,
            payload_byte_len: 0,
            total_byte_len: 128 + 64,
            triples_digest: [0; 32],
            columnar_digest: [0; 32],
            reserved: [0; 24],
        };
        let bytes = h.to_bytes();
        assert!(bytes[100..128].iter().all(|&b| b == 0));
    }

    #[test]
    fn from_bytes_refuses_bad_magic_and_tampered_reserved() {
        let h = ArrowEgressHeader {
            magic: ArrowEgressHeader::MAGIC,
            schema_version: ArrowEgressHeader::SCHEMA_VERSION,
            flags: 0,
            batch_count: 1,
            payload_offset: 128,
            payload_byte_len: 16,
            total_byte_len: 16 + 128 + 64,
            triples_digest: [0; 32],
            columnar_digest: [0; 32],
            reserved: [0; 24],
        };
        let mut bytes = h.to_bytes();
        bytes[0] = b'X';
        assert_eq!(
            ArrowEgressHeader::from_bytes(&bytes),
            Err(ArrowEgressHeaderError::BadMagic)
        );

        let bytes = h.to_bytes();
        let mut bytes = bytes;
        bytes[127] = 1;
        assert_eq!(
            ArrowEgressHeader::from_bytes(&bytes),
            Err(ArrowEgressHeaderError::ReservedNotZero)
        );
    }

    #[test]
    fn canon_arrow_empty_v1_is_blake3_of_domain_string() {
        let expected = *blake3::hash(b"CANON_ARROW_EMPTY_V1").as_bytes();
        assert_eq!(canon_arrow_empty_v1(), &expected);
        // Stable across calls (OnceLock identity).
        assert_eq!(canon_arrow_empty_v1(), canon_arrow_empty_v1());
        assert_eq!(CANON_ARROW_EMPTY_V1(), &expected);
    }
}
