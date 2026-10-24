# ATTEST-API

Design for the `graphlaw-attest` verification CLI and the self-contained
replay-JSON wire shape, raised from the v26.10.22 ERRC return (SRC-RAISE
lane) and grounded in `src/attest.rs::receipt_fold` and
`src/egress/arrow.rs` as they exist on disk. Nothing in this document is
claimed as landed; status vocabulary follows the repo's
UNVERIFIED/DESIGN default.

## Dependency classes

- **DEP-0**: exists on disk today, cited file:line.
- **DEP-1**: additive library/CLI work on existing primitives.
- **DEP-2**: hard Phase-2 dependency — see
  [Phase-2 dependency](#phase-2-dependency).

## Existing surface (DEP-0)

- `src/attest.rs:1-12` — module contract: canonical payload = sorted-key,
  no-whitespace JSON; signing is deterministic (RFC 8032); verification
  needs payload + attestation + `TrustedKeys`; no network, no clock.
- `src/attest.rs:443-463` — `verify_bytes`: order is shape → payload
  digest (`BadSignature`) → key trust (`UnknownKey`) → signature
  (`BadSignature`).
- `src/attest.rs:337-402` — `Attestation::to_json`/`from_json`: canonical
  form `{"key_id":..,"payload_blake3":..,"signature":..}`, strict inverse,
  fixed-length hex shape check.
- `src/attest.rs:526-562` — `receipt_payload`: sorted keys, absent options
  as `null`; the canonical receipt payload discipline the replay JSON
  mirrors.
- `src/attest.rs:658-791` — `receipt_fold`: `receipt_v1` =
  `BLAKE3(H_query ∥ H_pre ∥ H_post ∥ H_triples ∥ epoch_le)` (frozen, pinned
  golden `63bff90d…08a8ffe` at line 749); `receipt_v2` =
  `BLAKE3("GLAW_RECEIPT_V2" ∥ H_query ∥ H_pre ∥ H_post ∥ H_triples ∥
  H_columnar ∥ epoch_le)`, 15-byte domain prefix (line 671), pinned goldens
  at lines 749, 776, 784.
- `src/egress/arrow.rs:14-44` — `ArrowEgressHeader`: 128 bytes,
  `#[repr(C, align(64))]`, magic `GLAWARRW`, schema `0x0002`,
  `triples_digest` at wire bytes 36..68, `columnar_digest` at 68..100,
  reserved 100..128 zero-filled.
- `src/egress/arrow.rs:147-166` — typed header errors: `BadMagic`,
  `BadSchemaVersion`, `BadFlags`, `BadPayloadOffset`, `BadTotalLen`,
  `ReservedNotZero`.
- `src/egress/arrow.rs:174-177` — `canon_arrow_empty_v1()` =
  `BLAKE3(b"CANON_ARROW_EMPTY_V1")` = `db2da6fc…44f7b0c`, the non-tabular
  `H_columnar` arm.
- `src/bin/graphlaw_attest.rs` — existing `sign`/`verify`/`pubkey`
  subcommands over arbitrary files; exit codes 0 ok / 1 refused /
  2 usage-or-IO. The design below extends this bin additively.
- `src/bin/graphlaw_verify.rs` — precedent for the offline third-party
  output contract: `ADMITTED` / `REFUSED: <code>: <detail>`, exit 0/1/2.

## CLI surface

New subcommands on `graphlaw-attest` (DEP-1; existing modes untouched):

```text
graphlaw-attest seal   <evidence.arrows> --key-seed-hex <hex> [--out <replay.json>]
graphlaw-attest verify <replay.json> [--evidence <file>] --trusted-key <hex> [...]
graphlaw-attest chain  <replay.json> [--next <replay.json> ...]
```

- `seal` parses a `.arrows` artifact, folds the receipt digest, signs the
  replay JSON, writes `--out` (default stdout).
- `verify` re-derives every witness from the evidence file when given
  (or from the replay JSON alone when not) and prints `VERIFIED` or
  `REFUSED: <code>: <detail>`; exit 0/1/2, matching
  `graphlaw-attest`/`graphlaw-verify` today.
- `chain` checks the `chain` field linkage across one or more replay
  documents (parent-child digest chaining, same discipline as
  `receipts/CHAIN.blake3`).

## Replay-JSON schema

One self-contained document, canonical field order, all digests lowercase
hex, mirroring the `attest.rs` canonical-JSON discipline (sorted keys, no
whitespace, absent options `null`). `verify_bytes` re-derives the payload
bytes from exactly this field order.

```json
{
  "v": 2,
  "receipt": "<receipt_v2, 64 hex>",
  "folds": {
    "h_query":    "<64 hex>",
    "h_pre":      "<64 hex>",
    "h_post":     "<64 hex>",
    "h_triples":  "<64 hex>",
    "h_columnar": "<64 hex>",
    "epoch":      "<u64, 16 hex, little-endian value rendered as hex>"
  },
  "egress_header": "<128-byte wire header, 256 lowercase hex>",
  "attestation": {
    "key_id":         "<64 hex>",
    "payload_blake3": "<64 hex>",
    "signature":      "<128 hex>"
  },
  "chain": "<64 hex, parent replay digest; 64 zeros for a chain head>"
}
```

Canonical serialization order is exactly: `v`, `receipt`, `folds`
(inside `folds`: `epoch`, `h_columnar`, `h_post`, `h_pre`, `h_query`,
`h_triples` — sorted, as `receipt_payload` sorts), `egress_header`,
`attestation` (sorted: `key_id`, `payload_blake3`, `signature`), `chain`.
The canonical payload signed by `sign_bytes` is this document with the
`attestation` field removed and `chain` included; `payload_blake3` is
`BLAKE3` of those bytes. A re-serialization that differs by one byte is a
refusal, same as `Attestation::from_json`'s strict inverse
(`src/attest.rs:367-387`).

Field derivations (DEP-0 primitives):

- `folds.h_query`, `h_pre`, `h_post` — from the receipt fold inputs;
  provenance is the collector (DEP-2 until the collector lands).
- `folds.h_triples` — equals `ArrowEgressHeader.triples_digest`
  (wire bytes 36..68).
- `folds.h_columnar` — equals `ArrowEgressHeader.columnar_digest`
  (wire bytes 68..100), or `canon_arrow_empty_v1()` for non-tabular
  results.
- `receipt` — `receipt_fold::receipt_v2(&h_query, &h_pre, &h_post,
  &h_triples, &h_columnar, epoch)` (`src/attest.rs:704-721`).
- `egress_header` — the full 128-byte `to_bytes()` form, hex, so `verify`
  can re-run `from_bytes` and re-derive both witness digests without the
  payload.
- `epoch` — the fold's `u64`, folded little-endian per
  `src/attest.rs:696,719`.

## Error taxonomy

`verify` fails closed, naming WHICH witness broke. Two layers, mirroring
`AttestError` (`src/attest.rs:39-47`) plus witness-specific variants; new
codes are appended to `docs/refusals.md`, append-only, never renumbered.

Structural / cryptographic layer (reuse semantics of `AttestError`):

- `Malformed(<detail>)` — not canonical JSON, wrong field order, wrong hex
  length, header `from_bytes` typed refusal (`BadMagic`,
  `BadSchemaVersion`, `BadPayloadOffset`, `BadTotalLen`,
  `ReservedNotZero`).
- `BadSignature` — the canonical payload digest in the attestation does
  not match the re-serialized payload, or the Ed25519 signature does not
  verify (`src/attest.rs:449-462` order preserved).
- `UnknownKey` — `attestation.key_id` not in the `--trusted-key` set.

Witness layer (which fold disagrees with re-derivation):

- `TriplesMismatch` — recomputed `BLAKE3` over the triples-side witness
  does not equal `folds.h_triples` (or the header's `triples_digest`).
- `ColumnarMismatch` — recomputed digest over the columnar payload region
  does not equal `folds.h_columnar` (or the header's `columnar_digest`).
- `EpochMismatch` — the `epoch` recovered from the evidence does not equal
  `folds.epoch`.
- `ChainMismatch` — a `--next` document's `chain` field does not equal the
  BLAKE3 digest of the parent replay document.

Verification order: Malformed → UnknownKey → BadSignature → witness
digests (`TriplesMismatch`/`ColumnarMismatch`/`EpochMismatch`) →
`ChainMismatch`. An untrusted key refuses before any witness work (same
premise as `verify_bytes`: trust before digests).

## Two-failure killer demo

The demo that proves the dual-witness design is not decorative: each
witness catches exactly the corruption the other misses. Both flips run
against one sealed document plus its evidence file; `verify` must refuse
with the named variant, not a generic error.

1. **Flip an Arrow payload byte** (one byte inside the payload region,
   offsets 128..total-64): `h_columnar` recomputed from the payload
   region no longer equals the header's `columnar_digest` →
   `REFUSED: ColumnarMismatch`. The triples digest is untouched — only
   the columnar witness sees this.
2. **Flip a triple** (change one term in the triples-side witness): the
   recomputed `triples_digest` no longer equals `folds.h_triples` →
   `REFUSED: TriplesMismatch`. The payload region is untouched — only
   the triples witness sees this.

Anti-vacuity: the demo is non-vacuous only if `verify` on the unmutated
seal prints `VERIFIED` immediately before each flip (shim-court shape:
baseline → mutate → flip observed → restore byte-exact). A demo where the
baseline does not verify, or where a flip surfaces as `Malformed` instead
of the named variant, is a FAIL.

Standing: the mutation mechanics and the `verify` refusal arms are DEP-1
(synthetic headers suffice); the demo as an ALIVE receipt over real query
evidence is DEP-2.

## Merkle prove-row raise

Additive raise, flat digest unchanged (ERRC-RAISE item): give `verify`
per-row proof capability without touching the existing folds.

- Tree: BLAKE3 binary Merkle tree over the columnar payload region cut
  into fixed 64-byte chunks (final partial chunk zero-padded to 64;
  empty payload is the single leaf `canon_arrow_empty_v1()`).
- Leaves: `BLAKE3(chunk || le32(chunk_index))` — index binding prevents
  sibling-swap reuse.
- Internal nodes: `BLAKE3(left || right)`, odd node promoted unchanged at
  each level.
- Root: folded — `columnar_digest' = BLAKE3("GLAW_MERKLE_V1" ||
  columnar_digest || merkle_root)`. The header's flat
  `columnar_digest` (BLAKE3 over the payload region) stays exactly as it
  is on the wire today (`src/egress/arrow.rs:34`); the Merkle root lives
  only in the replay JSON as a new optional field `folds.merkle_root`
  (absent = `null`, flat-digest-only verification, v2 documents remain
  valid forever). A `merkle_root` present in the replay JSON must satisfy
  `columnar_digest' == folds.h_columnar` or the verify refuses
  `MerkleMismatch` (appended to the refusal register).
- `prove-row <replay.json> --row <i>` (DEP-1): emits
  `{row_index, chunk_index, chunk_bytes_hex, inclusion_path: [64-hex
  siblings]}`. `verify-row` recomputes leaf-up and checks the root and
  the folded digest; refuses `Malformed` / `MerkleMismatch`.
- New domain constants: `GLAW_MERKLE_V1` (root fold), and the leaf/domain
  separation above — minted with pinned goldens in the
  `receipt_fold`-style test block, same discipline as the v1/v2 goldens.

## Phase-2 dependency (hard)

`seal` has no real subject today. No real `.arrows` evidence exists to
seal until the CanonArrow collector lands (ADR-CANONARROW Phase 2:
wiring the egress path into `duckdb_eval`; `src/egress/arrow.rs:5-6`
states the module "is not yet wired into `duckdb_eval` (Phase 2/3)").

Consequences, stated explicitly:

- `seal`/`verify` design, parsers, refusal arms, and the two-failure demo
  can be built and anti-vacuity-mutated now (DEP-1) against synthetic
  headers — `ArrowEgressHeader::to_bytes` is a DEP-0 constructor.
- An ALIVE seal of a real query result, and therefore an ALIVE killer
  demo over production evidence, is BLOCKED-BY the Phase-2 collector
  (DEP-2). No receipt may claim it before that landing.

## See Also

- `docs/jira/v26.10.22/ADR-canonarrow-dual-witness-egress.md`
- `docs/refusals.md` (append-only refusal register)
- `src/attest.rs`, `src/egress/arrow.rs`, `src/bin/graphlaw_attest.rs`,
  `src/bin/graphlaw_verify.rs`
