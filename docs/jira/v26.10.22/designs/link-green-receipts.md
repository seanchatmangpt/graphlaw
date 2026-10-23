# Link-green receipt set (coordinator-filed from lane reports, 2026-10-22)

> Note: these receipts' env (`WASI_SDK_PATH`) used the engine cache; the current recipe is a wasi-sdk root, documented in `docs/WASM.md`.

Lanes dispatched during the fleet wave report back through the
coordinator; where a lane's receipt was not filed by the lane itself,
this record carries it. Each entry is the lane's own reported command +
exit, transcribed verbatim-class (numbers unaltered).

The `WASI_SDK_PATH` values below name the DuckDB engine cache
(`~/.frozen-duckdb/cache/v2.0.0-dev-wasm32-wasip1`), which is not a wasi-sdk
root. They stay as the witnessed env. The current runbook is `docs/WASM.md`.

## L1 — wasm link GREEN (four-hop root-cause chain)

Final gates: `WASI_SDK_PATH=~/.frozen-duckdb/cache/v2.0.0-dev-wasm32-wasip1
cargo build -p graphlaw-wasm --target wasm32-wasip1 --profile wasm` →
exit 0 (1m03s); artifact 45,162,380 B, `\0asm` v1, exports
gl_call/gl_alloc/gl_free, full DuckDB linked. `cargo test -p
graphlaw-wasm` → 9/0 (incl. sql_document_admitted,
sql_document_admits_in_the_wasm_unit). Original red verbatim:
`rust-lld: error: unable to find library -lduckdb`.

Four hops, each observed:
1. frozen-duckdb-sys emitted `dylib=duckdb` unconditionally (host-only)
   → wasi branch; converged upstream as crates.io 2.0.2.
2. graphlaw on crates.io froze checkout edits → needed
   `[patch.crates-io]` + lock re-point; superseded by the upstream 2.0.2
   lock (patch entries since removed — 2.0.2 works without them).
3. `archive_is_wasm32`'s 256 KiB probe window sat behind a 10.6 MB ar
   symbol table and falsely rejected a GENUINE wasm32 archive (first
   `\0asm` offset 10596838). Fix: widened probe in
   frozen-duckdb-builder/src/lib.rs. CAVEAT ON THE CENSUS: `file(1)`
   cannot distinguish wasm from host archives ("current ar archive"
   either way) — the "host archive" census reading was wrong; the real
   blocker was the validator clamp.
4. `clang --target=wasm32-wasip1` had no wasi libc (`errno.h` not
   found) → sysroot-aware wrapper over emsdk clang 24 +
   emscripten sysroot; libc++/libc++abi/libunwind (wasmexcept) staged.

Archive: DuckDB source pin /Users/sac/rel4r/duckdb-src @ b6013f7aa2,
compiled `-fwasm-exceptions -DISABLE_THREADS=ON
-DUCKDB_CAPABILITIES=none` (staticlib + core_functions + icu), 87.6 MB
libduckdb.a. Pitfalls recorded: sccache cannot handle the wasm triple
(disable launcher); unity builds break under the /tmp symlink
(configure under /private/tmp).

Handoffs: (a) WASI_SDK_PATH reproducibility script required before the
priv/graphlaw.wasm re-pin (L109); (b) gws4 inline build copy repoint
(L105, landed); (c) CI needs the SDK setup (L113).

## L105 — gws4 repoint (single build path)

tests/gws4_ipc_injection_court.rs now imports `mod common;` and reads
`common::wasm_path()`; inline cargo invocation deleted. At landing the
link was still red: gws4_a/gws4_c typed BLOCKED-LINK (shared helper
message witnessed verbatim), gws4_b 1 passed (native, no wasm build).

## L112 — js limits mirror fix (198/0)

js/npm/src/limits.ts max_outstanding_alloc_bytes 268435456 → 16777216.
Authority: src/abi.rs:73 const → wasm/src/lib.rs:33 assert → registry
line 287; js was the only stale surface. Gates: npm run test 198 passed /
0 failed; npm run build exit 0 (332ea218 in place); quickstart snippet
node exit 0.

## L102 — conformance vectors regen (12/0 + 8/0)

ASSETS.blake3 stale for the two L71-changed ontologies (witnessed at
tests/corpus_conformance.rs:173). Regen via UPDATE_ASSET_MANIFEST=1
cargo test --test corpus_conformance
asset_manifest_pins_every_byte → exactly 2 lines changed (sql-ast.ttl
14914→15824 B; ast_governance.ttl 3092→4315 B). gen_w3c_manifest.sh →
712 entries unchanged, +13 lines (census notice the committed file
predated). Gates: corpus_conformance 12/0+1 ignored; w3c_conformance
8/0; w3c_manifest 1/0 (typed census SKIP).
