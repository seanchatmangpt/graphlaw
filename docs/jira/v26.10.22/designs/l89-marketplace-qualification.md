# L89 — ggen-marketplace pack qualification (2026-10-22)

Repo: /Users/sac/ggen-marketplace (own checkout). Subject: L2's
side-table FFI template change in packs/rust-wasi-wasmex-pack.

Gates (real exits):
- python3 scripts/marketplace.py validate → 0 (339 packs, 1927 templates)
- pytest tests/test_pack_lifecycle_court.py → 3 passed
- pytest tests/test_rust_wasi_wasmex_court.py → 1 SKIPPED (pm4pytest has
  no distribution — loud, everywhere). Substitute: Jinja2 render of
  templates/abi/ffi.rs.tmpl with graphlaw's exact binding set proved
  BYTE-IDENTICAL to wasm/src/ffi.rs (b3sum 9dcab4e4…) — every divergence
  in early attempts traced to guessed pin values, confirming the
  template branches are the only variation source.
- from_raw_parts sweep: all remaining sites lawful under the side-table
  law (slot-capacity reconstruction / validated borrow / guest-owned).
- Release convention: feat() commit only; pack version stays 0.1.0.
