# golden fixtures (Gate 2 differential)

- `data.ttl`, `query.rq`: inputs.
- `native_canonical.json`, `native_sparql.json`, `id.blake3`: NATIVE-MINTED by
  `cargo run --example golden_mint -- <golden_dir>` in `/Users/sac/graphlaw`
  (native `graphlaw::abi::call_json`). Never hand-edit; re-mint. `id.blake3`
  holds the `blake3:`-prefixed canonical id of `data.ttl` plus a newline.
  The differential court compares the pinned wasm engine against these.
- `canonical.nq`, `bindings.json`: hand-derived from the spec (reference text,
  not asserted by the differential court; superseded by the native goldens).
- `MANIFEST.json`: file inventory with origin. Per-file blake3 fields are null
  (UNVERIFIED) until derived with a hash tool.
