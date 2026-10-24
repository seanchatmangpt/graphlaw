# ADR-CANONARROW (Revised; absorbed into the v26.10.22 release boundary — the 26.10.23 moniker is deferred): Canonical Arrow IPC Dual-Witness Columnar Egress

Operator-approved specification, 2026-10-23. Baseline measurements: N-Triples
text rendering 9.246 ms / 10k rows; DuckDB raw row materialization 1.354 ms;
raw-bytes digest proxy ~992 µs; serialization penalty ~6.83× (Lane 4, v26.10.22).

## Decision — CanonArrow Dual-Witness Egress

1. **Native in-WASM Arrow IPC streams**: DuckDB columnar execution vectors write
   directly into an uncompressed, memory-aligned Arrow IPC Stream (RecordBatch)
   inside guest linear memory.
2. **Versioned dual-witness envelope (`receipt_v2`)**: `receipt_v1` verifiers
   untouched; `receipt_v2` seals `triples_digest` + `columnar_digest`; CHAIN
   re-baseline via the documented quiesced-regen procedure.
3. **Projection-order preservation**: canonical field order matches the admitted
   AST's declared projection order; determinism from deterministic AST,
   uncompressed buffers, 64-byte alignment, zero-filled validity bitmasks.
4. **Guest view-pinning**: host holds an immutable pinned read-only slice across
   receipt verification — no re-entry that can `memory.grow`.

```
R_receipt_v2 = BLAKE3( "GLAW_RECEIPT_V2" ∥ H_query ∥ H_pre ∥ H_post
                       ∥ H_triples ∥ H_columnar ∥ epoch )
```

## 3.1 Cryptographic invariants

- `receipt_v1 = BLAKE3(H_query ∥ H_pre ∥ H_post ∥ H_triples ∥ epoch)` (legacy, unchanged).
- `receipt_v2` prepends domain-separation constant `b"GLAW_RECEIPT_V2"` (15 B)
  and inserts `H_columnar` before `epoch` — cannot collide with v1 under any
  permutation.
- Domain null digest: `H_columnar_empty = BLAKE3(b"CANON_ARROW_EMPTY_V1")` for
  non-tabular results.
- Chain re-baseline: quiesced-regen of `CHAIN.blake3` per release runbook.

## 3.2 Memory wire protocol

```rust
#[repr(C, align(64))]
pub struct ArrowEgressHeader {
    pub magic: [u8; 8],            // b"GLAWARRW"
    pub schema_version: u16,       // 0x0002
    pub flags: u16,                // bit0 HasNulls, bit1 BigEndian, bit2 StrictProjectionOrder
    pub batch_count: u32,
    pub payload_offset: u32,       // always 128
    pub payload_byte_len: u64,
    pub total_byte_len: u64,       // payload + 128 header + 64-byte padding
    pub triples_digest: [u8; 32],
    pub columnar_digest: [u8; 32], // BLAKE3 over payload region
    pub reserved: [u8; 24],        // zero-filled to 128
}
// AMENDED 2026-10-23 (implementation witness): the spec's original
// `reserved: [u8; 36]` cannot form a 128-byte repr(C, align(64)) struct —
// repr(C) pads 4 bytes after `payload_offset` so the u64 fields sit at
// 8-aligned offsets (104 bytes before `reserved`); 128 is the next multiple
// of 64, so `reserved` is 24 bytes. Wire reserved region = offsets 100..128
// under either reading.
```

## 3.3 CanonArrow deterministic normalization

1. Declared projection order (lexicographic reordering prohibited).
2. Zero compression — no LZ4_FRAME/ZSTD on the witness path.
3. Deterministic null bitmaps — trailing bits zero-filled, unset bits `0b0`.
4. 64-byte alignment normalization, zero padding.
5. Host view-pinning for the whole receipt calculation.

## 4 Requirements

| ID | Requirement | Verification |
|---|---|---|
| REQ-01 | 10k-row materialize+egress ≤ 1.50 ms (from 8.46 ms) | bench |
| REQ-02 | in-WASM BLAKE3 over 10k-row Arrow buffer ≤ 1.20 ms | bench in wasm32 |
| REQ-03 | dual-witness integrity: both digests match vectors under receipt_v2 | courts |
| REQ-04 | zero-copy host extraction via `payload_offset` pinned view | host court |
| REQ-05 | memory plateau Δ ≤ 2 pages/cycle over 10k-cycle run | memory-wall court |

## 5 Implementation sequence

- **Phase 1**: audit frozen DuckDB C headers for `duckdb_query_arrow`;
  fallback = `duckdb_fetch_chunk` guest-side assembly; define
  `ArrowEgressHeader` + `CANON_ARROW_EMPTY_V1` arm in `src/egress/arrow.rs`.
- **Phase 2**: 64-byte-aligned collector, projection-order schema, zero
  compression, streaming BLAKE3 → `columnar_digest`.
- **Phase 3**: `receipt_v2` constructor (v1 verifiers preserved), dual-witness
  test harness, FIBO 4-domain battery under receipt_v2, quiesced CHAIN regen.

## SQLRDF-25 closure sequencing (Little's Law rationale)

Closing the five dialect gaps eliminates downstream reshaping hops (WIP L = λW
collapse) and keeps the entire transformation inside the BLAKE3-sealed boundary —
post-boundary reshaping by external tools breaks the evidentiary chain
(FRE 902(14) framing). Order: Rows 4+5 (composite literals + JSON `->`/`->>`,
shared token sites) → Rows 1+2 (star EXCLUDE/REPLACE + `COLUMNS('regex')`,
EMPTY_PROJECTION refusal stays intact) → Row 3 (PIVOT/UNPIVOT SHACL shapes +
column-set readback courts) → full battery: `sql_rdf_dialect`,
`chicago_validation`, mutation courts, chain re-baseline, pinned release.

Row ledger: `docs/jira/v26.10.22/SQLRDF-25-remaining-gaps.md`.
