# SPLIT-NOTES — Lane L46 Phase 4 extraction (UNWIRED)

Status: **UNWIRED by directive.** No existing file was modified. The six split
files are parse-validated (rustfmt exit 0 each) but are not referenced by any
`mod` declaration, so `cargo check`/`cargo test` behavior is unchanged. The
switchover lane replaces the corresponding line ranges in
`src/receipt_chain.rs` / `src/registry.rs` with `mod` declarations and deletes
the SPLIT-EXTRACTION banner comments.

## Receipt-chain split (`src/receipt_split/`)

Item map of `src/receipt_chain.rs` (doctests travel with their items; the 21
doctests landed today are inside the sliced ranges and are carried verbatim):

| Source range | Family | Destination |
|---|---|---|
| 1–46 | module doc + use prelude | stays; prelude (44–46) re-embedded verbatim in each split file |
| 47–426 | `pub mod frame` (DenialPolarity, OcelCausalFrame incl. the 99-byte `to_hash_bytes`, OcelCausalReceipt, PackedObjRef) | `frame.rs` |
| 435–871 | CoreError, Obligation, Andon, RefusalCategory, RefusalScenario + lane fns, ReceiptMeta, build_admission_frame, chain_from_frame | `record.rs` (slice A) |
| 872–880 | blank + epoch banner/doc | stays in receipt_chain.rs |
| 881–1596 | `pub mod epoch` | stays (not in L46 ownership) |
| 1597–1989 | RECEIPT_RECORD_VERSION, CHAIN_RULE_V2_FOLD, CHAIN_RULE_BASE, ReceiptRecord, ChainRule, ChainStanding, ChainVerification, decode_hex32, impl ReceiptRecord, ChainRuleMonotonicity, fold_in_v2_epoch | `record.rs` (slice B) |
| 1990–2002 | blank + validator banner/doc | stays |
| 2003–2263 | `pub mod validator` | stays (not in L46 ownership) |
| 2264–2370 (EOF) | `pub mod store` | `store.rs` |

`frame.rs` keeps `pub mod frame { ... }` nested verbatim. Recommended
switchover: wire as `mod frame;` inside receipt_chain with a one-line inner
alias or keep call sites on `receipt_chain::frame::` paths — the nested module
means `receipt_chain::frame::` paths stay stable if the file is wired as
`receipt_chain::frame::frame`? No: the switchover lane should either (a)
unwrap the wrapper (then update the `use self::frame::` prelude lines that
were re-embedded verbatim in frame/record/store.rs), or (b) keep the wrapper
and wire with `pub mod frame { include! }`-free plain `pub use frame::*;`.
Decision is deferred to the switchover lane; both are parse-compatible with
what was extracted.

### Files and provenance (authoritative)

| File | Provenance (src/receipt_chain.rs, 1-based inclusive) |
|---|---|
| `src/receipt_split/frame.rs` | 47–426 |
| `src/receipt_split/record.rs` | 435–871 + 1597–1989 (two slices, joined by one glue blank line — glue is extraction-added, not source text) |
| `src/receipt_split/store.rs` | 2264–2370 (EOF) |

### Not extracted (stays in receipt_chain.rs)

| Range | Why |
|---|---|
| 1–46 module doc + prelude | module doc stays with the parent module |
| 872–880 epoch banner/doc, 881–1596 `pub mod epoch` | epoch not in L46 ownership (frame/record/store only) |
| 1990–2002 validator banner/doc, 2003–2263 `pub mod validator` | validator not in L46 ownership |

### Byte-identity receipts

Post-write re-check: the exact source text of every provenance range is
contained verbatim in its split file.

- frame (47–426): BYTE-IDENTICAL
- record A (435–871): BYTE-IDENTICAL
- record B (1597–1989): BYTE-IDENTICAL
- store (2264–2370): BYTE-IDENTICAL
- definitions (29–911): BYTE-IDENTICAL
- emit_json (913–1636): BYTE-IDENTICAL
- emit_ttl (1638–1964): BYTE-IDENTICAL (initial attempt used end 1965 and
  clipped `#[cfg(test)]`; re-sliced to 1964)

The 99-byte `#[repr(C)]` `OcelCausalFrame` packing and the custom byte
serializers (`to_hash_bytes`, scatter) are inside the frame slice and were
extracted by pure line-range slicing — zero reformatting; the byte-identity
check above is the receipt.

## Registry split (`src/registry_split/`)

### Files and provenance (authoritative)

| File | Provenance (src/registry.rs, 1-based inclusive) |
|---|---|
| `src/registry_split/definitions.rs` | 29–911 |
| `src/registry_split/emit_json.rs` | 913–1636 |
| `src/registry_split/emit_ttl.rs` | 1638–1964 (ends at the blank before `#[cfg(test)]` at 1965; the `mod tests` block stays in registry.rs) |

### Partition rationale

- **definitions.rs** — the static tables: REGISTRY_SCHEMA, GAC_NAMESPACE,
  REGISTRY_BASE, FieldDefault/Field (+impl, req/opt), Variant, Op,
  RefusalCodeRow, LawStepRow, DialectRow, ENGINES, REFUSAL_KINDS, REGIMES,
  LEASE_CEILINGS, LEASE_REASONS, RECEIPT_REASONS, POLICY_REFUSAL_KINDS,
  RDF_DIALECTS, OTHER_DIALECTS, KINDS_*/CODES_LIMIT/DATA/NOT_TAGGED,
  OPS (323–719), REFUSAL_CODES (720–817), LAW_STEPS (818–895),
  NATIVE_ONLY_OPS, op_names, rdf_dialect_names, other_dialect_names.
- **emit_json.rs** — JSON emission and the canonical/hash surface:
  json_string_array, field_json, fields_json, dialects_json,
  surface_document, receipt_surface_document, blake3_of, base_document,
  LIMIT_META (incl. `type ModelRow` at 1190), limit_meta_json, MODELS,
  models_json, model_enums_json, static DOCUMENT OnceLock + document(),
  surface_blake3, receipt_surface_blake3, registry_blake3, registry_value,
  escape_json, sorted_entries, write_json, canonical, registry_json_pretty.
- **emit_ttl.rs** — TTL emission: Triple, Graph, node, lit, text_of, int_of,
  bool_of, items, impl Graph, graph(doc), registry_turtle. It reads
  `document()`/`registry_value()` from emit_json; on switchover it sees them
  via the parent module — no emission-order change.

### Emission key order (as extracted; the split reorders nothing)

`registry_value()` / `registry_blake3()` are built in emit_json.rs with
`document()` (source 1498–1510) inserting, in this exact order:

```
schema, graphlaw_version, abi_version, receipt_surface, authorities,
capability, authority, engines, refusal_kinds, rdf_dialects, other_dialects,
regimes, lease_ceilings, lease_reasons, receipt_reasons,
policy_refusal_kinds, limits, limit_meta, models
```

(`registry_blake3` itself is inserted last by `document()`; hashing removes
that key and canonicalizes with byte-sorted keys, so the *hash* is
order-insensitive but the *insertion order* above is the extracted surface.)

## Hash-invariance court spec (tests/registry_hash_invariance.rs does not exist)

`ls tests | grep -i registry` returns registry_artifacts.rs,
registry_dispatch.rs, registry_limits.rs only. Spec for the switchover lane
to add — the court must assert the root `registry_blake3` invariant across
the split:

1. **Baseline capture (pre-switchover)**: on the monolith, record
   `registry_blake3()`, `surface_blake3()`, `receipt_surface_blake3()`,
   `canonical(&registry_value())`, and `registry_turtle()` outputs as the
   pinned expectations (golden bytes or literals).
2. **Post-switchover assertion**: the same public functions, now routed
   through `registry::{definitions, emit_json, emit_ttl}` (re-exported at
   `graphlaw::registry::` with unchanged paths), reproduce the pinned
   expectations exactly.
3. **Committed-artifact cross-check**: `registry_blake3()` must equal the
   `registry_blake3` key inside the committed
   `registry/capability-registry.json`, and `registry_turtle()` must equal
   the committed `registry/capability-registry.ttl` (regenerated only by the
   registry sync flow, never hand-edited).
4. **Anti-vacuity**: the court must flip if either split file's emission
   order or table content changes — a mutation that reorders
   `document()`'s insertions or edits any OPS row must fail the court.

Minimal sketch (compilable form is the switchover lane's call):

```rust
#[test]
fn root_registry_blake3_invariant_across_split() {
    let doc = graphlaw::registry::registry_value();
    let stated = doc.get("registry_blake3").and_then(|v| v.as_str()).expect("key present");
    assert_eq!(stated, graphlaw::registry::registry_blake3());
    let committed = std::fs::read_to_string("registry/capability-registry.json")
        .expect("committed registry JSON");
    let committed_digest = committed
        .lines()
        .find_map(|l| {
            let l = l.trim_start();
            l.strip_prefix("\"registry_blake3\": \"")
                .map(|r| r.trim_end_matches([',', '"', ' ', '\t']).to_string())
        })
        .expect("registry_blake3 key in committed artifact");
    assert_eq!(graphlaw::registry::registry_blake3().as_str(), committed_digest.as_str());
}
```

## Verify receipts (this session, actual output)

- Write phase: `wrote 6 of 6` (python3), then record.rs/emit_ttl re-slices
  printed `ok`.
- rustfmt `--emit stdout --edition 2024`, quoted exits — final state:
  frame.rs exit=0, record.rs exit=0 (after re-slice; first attempt exit=1 at
  a clipped item boundary), store.rs exit=0, definitions.rs exit=0,
  emit_json.rs exit=0, emit_ttl.rs exit=0 (after re-slice; first attempt
  exit=1 clipping `#[cfg(test)]`).
- Byte-identity: all seven provenance ranges BYTE-IDENTICAL (listed above).
- `git status --porcelain src/receipt_split src/registry_split`: untracked
  additions only.
- No existing file modified this session. The working-tree diff vs HEAD on
  receipt_chain.rs/registry.rs pre-dates this lane (earlier lanes' landed
  work today).

## Falsifier

If post-switchover `registry_blake3()` differs from the pre-switchover value
captured on the monolith, the split reordered something and must be re-sliced.
The byte-identity receipts are the pre-assertion; the hash court above is the
post-assertion. A court that cannot exit 1 is the failure.
