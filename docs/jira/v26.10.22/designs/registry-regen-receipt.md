# Registry Regen Receipt — Lane L73 (L21 finding C9)

- Date: 2026-10-22
- Subject: `/Users/sac/graphlaw` @ main (post-615dbfa working tree)
- Flow: `cargo run --features abi --bin graphlaw-registry -- --write`
  (the real regen path — `src/bin/graphlaw_registry.rs` write(); the
  registry_artifacts court itself exercises this same `--write` path)

## Before / after digests

| row | before | after |
|---|---|---|
| registry | `blake3:7db561aa315e86f3f21022697276ef3382b808ea1bcef4e4245159b755f819b4` | `blake3:4cd56fd50675c0aa9cc95c0444ed8e378e40a270fc52fe859ffe4aaa83b7091f` |
| receipt-surface | `blake3:78ebb149bb6712049fabbefc8b17cb72dcb6461b66422e1ea04e106e2b63f96e` | `blake3:0cc02ca4100542f68de431eacf4e5d0dac5d51fdd96887b2bf2b2cbebcd3669d` |
| surface | `blake3:da96e1375a76610b5d91aedb9b01d5ac0807b0f850ce4440628c37a3b96b9ee9` | unchanged |

L21 cited drift to `5d31d472…`; at run time the live digest was already
`4cd56fd5…` — the src moved again between L21's observation and this regen.
The regen pinned what the tree hashes today.

## What moved and why

`git diff` of `registry/capability-registry.json` (2103-2109, 2115, 2332, 2408):

1. Per-file digests for `src/duckdb_eval.rs` and `src/sql_ast.rs` (the receipt-minting
   surface) and the derived `receipt_surface_blake3` — moved because L11's edits to
   `src/duckdb_eval.rs` and `src/sql_ast.rs` (L4 seal changes) changed those files' bytes.
2. One doc string under op `duckdb`: "Diagnostic text from the embedded DuckDB
   substrate (op `duckdb`, feature `duckdb`, native-only)" → "(ABI op `duckdb`,
   linked into the wasm module)" — L11's capability-description change in src.
3. `registry_blake3` recomputed over the document minus itself.

`registry/ARTIFACTS.blake3`: only the owned rows (`registry`, `receipt-surface`)
re-pinned. The `wasm` row is byte-identical before/after — asserted from the
diff output: no `wasm` line appears in the before/after diff of
ARTIFACTS.blake3, and the bin never re-pins it (graphlaw_registry.rs:231-239
"the wasm row: never re-pinned by graphlaw-registry"). No `--check` failure.

## Court exits

- `cargo test --test registry_artifacts` — 22 passed, 0 failed (exit 0)
- `cargo test --test registry_dispatch` — 14 passed, 0 failed (exit 0)
- `--write` reported: wrote capability-registry.json/.ttl, repinned ARTIFACTS.blake3, exit 0

## Files touched

- `registry/capability-registry.json` (via flow)
- `registry/capability-registry.ttl` (via flow)
- `registry/ARTIFACTS.blake3` (via flow, owned rows only)
- this receipt
