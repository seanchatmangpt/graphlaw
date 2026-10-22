# Code-first refusals & binary-first serialization — P29-RFL (design record)

Full design re-emitted by the design lane under the operator's strengthened
constraints ("no strings or json" on hot paths; client edge owns all
serialization; refusal code space is a ggen build artifact; 80/20
build-stamped admission). This record carries the operative decisions; the
lane's full text is the authority.

## RefusalCode — pure data, 64B, zero heap
`#[repr(C)] Copy` struct: code u16 (ggen-emitted discriminant), dialect u8,
engine u8, kind u8, predicate u16, step_or_index u16, reason u8, limit u16,
observed u32, max u32, offending [u8;32] (BLAKE3 of offending node —
replaces EVERY String payload), pad[2]. No message field. Construction =
one 64B struct literal, <10ns by construction; §3.1's 0.10µs met by
construction, TC-HOT-01 verifies.

## Register migration (receipt-gated)
- Discriminants ARE the register; REFUSAL-01..04 = low u16 values (externally
  receipted codes keep low values; generic codes start at 16). `[REFUSAL-nn]`
  decimals survive as the printed rendering at the edge.
- docs/refusals.md code tables REGENERATED from ggen-emitted
  refusal_codes.bin (prose recovery columns stay human-authored); ABI details
  gains numeric code_id additively; 81 string needles across 6 court files →
  code asserts + ONE rendering court.
- Flip closes the disclosed obligation at docs/refusals.md:36-38 (no
  registry rows, no receipts/ entry) — the receipt records old needle count,
  new code-assert count, digest.

## Binary-first ledger
- 64B frames become the ledger record verbatim; batch = GLQ1 magic + count +
  first_seq + parent[32] + frames + blake3. Hand-rolled escaper/cur parser
  DELETED — digests cover exact hot-path bytes (strictly stronger; byte-flip
  detection positional, not syntactic).
- NDJSON/JSON = cold export renderings (`digest(parse(export))`-independent
  validity law). DroppedEvidence gains code: u16 (typed overflow records).
- Capability registry gains code_space (base,len) per op — the registry IS
  the refusal-code registry, closing the disclosed gap.

## Migration order
N0 ggen code-space pipeline → N1 RefusalCode struct + ABI code_id additive →
N2 refusal sites emit codes+digests (depends on SQLRDF-17 canon-bytes
replacement) → N3 court migration (81 needles) → N4 binary ledger batches
(parallel lane) → N5 registry binary rows + flip receipt → N6 delete
message:String. Critical path N0→N1→N2→N3.

## Honest limits
wasm renderer std-only; Σ_PQC still UNMET; ledger GC/rollup carried
encoding-agnostic. Hot-path cost: <10ns vs 50-150ns today (alloc+format+
drop), verified by the B-chain bench, not asserted.
