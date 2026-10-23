# WASM integration reference

GraphLaw ships one self-contained WASI module — `graphlaw-wasm` — that exposes
the JSON ABI (`ABI_VERSION 1`) over linear memory with three exports:
`gl_alloc`, `gl_free`, `gl_call`. Re-read from `wasm/src/{lib,ffi,abi_meta}.rs`,
`ontologies/graphlaw-wasm.ttl`, and `tests/wasm_abi.rs`. See also
[ABI reference](abi-reference.md) and [Refusals](refusals.md).

## Architecture

The crate has two halves:

- **Generated shell** — `wasm/src/ffi.rs` and `wasm/src/abi_meta.rs` are
  rendered by `ggen sync` from `ontologies/graphlaw-wasm.ttl` through the
  rust-wasi-wasmex-pack (queries/ + templates/abi/). The sync pin is the
  committed `wasm/ggen.toml` (ffi rule only, output_dir `src`; `abi_meta.rs`
  is rendered by Lane 1's config, not this one). Regenerate `ffi.rs` with
  `cd wasm && ggen sync run` (ggen >= 26.10.11). Determinism receipt
  (2026-10-22): two consecutive syncs plus a delete-and-regen all produce
  b3sum `9dcab4e4099ede0e605b2f619c8791e8f19bf656f3c6a8bd552eeade6d9e46a9` (historical lane receipt, 2026-10-22, not a pin; re-verify with `b3sum wasm/src/ffi.rs`)
  for `wasm/src/ffi.rs` at graph_hash_hex
  `8821e277608452cd44baac350c850e4d96759b17a64bec9b08f2b644b0c07ae2` (historical graph hash, not a pin).
  "Rendered by ggen ...
  never edit this file by hand" is the standing header law
  (`wasm/src/ffi.rs:7-8`, `wasm/src/abi_meta.rs:6-7`). Edit the ontology, re-render.
- **Hand-written safe core** — all behaviour lives in `graphlaw::abi`
  (`src/abi.rs`); the only `unsafe` in the wasm crate is pointer handling in
  the generated shell (`wasm/src/lib.rs:8-10`). The contract is three safe
  functions: `graphlaw::abi::call(request: &[u8]) -> Vec<u8>`,
  `limit_response(name, observed, max)`, `missing_buffer_response()`
  (`wasm/src/ffi.rs:22-26`).

For streaming analytical evaluations and large Parquet scans, the runtime
architecture is detailed in [WASM ABI Op-15 Design](wasm-duckdb-op15-design.md)
and [WASM + DuckDB Integration Wave](wasm-duckdb-integration.md). The module
now links frozen-duckdb itself: the archive is built new-EH, the module
carries the Tag section, and the hosting runtime is `graphlaw-run-eh`
(wasmtime 48, `wasm_exceptions`). Memory accounting operates under a
deterministic 32 MiB DuckDB ceiling within the 64 MiB WASM arena, avoiding
linear memory fragmentation.

Compile-time asserts in `wasm/src/lib.rs:29-36` pin the generated
`abi_meta` limits and version to `graphlaw::abi`'s constants, so an ontology
edit that drifts from the core fails the native build instead of shipping a
mismatch. Two more contract tests keep the generated tables honest:
`abi_meta_mirrors_registry_ops` / `abi_meta_mirrors_registry_error_codes`
mirror `graphlaw::registry` (`wasm/src/lib.rs:45-56`), and
`shell_routes_through_graphlaw_abi` proves the shell has no local limits or
hook (`wasm/src/lib.rs:61-75`).

## gl_* packed-u64 ABI

All integers are wasm `i32`/`i64`. Protocol (`wasm/src/lib.rs:8-15`,
`wasm/src/ffi.rs:11-18`):

1. `gl_alloc(len: u32) -> *mut u8` — host reserves `len` bytes and writes a
   UTF-8 JSON request there. Returns null (0) when `len` exceeds
   `MAX_REQUEST_BYTES` or when a reservation would exceed the outstanding cap
   (`wasm/src/ffi.rs:82-91`).
2. `gl_call(ptr, len) -> u64` — runs the request and consumes (frees) the
   request buffer (`inputReleasePolicy "call-consumes"`,
   `ontologies/graphlaw-wasm.ttl:38`). The return value packs the response
   location as `packed = (out_ptr << 32) | out_len`
   (`wasm/src/ffi.rs:133-142`; `returnConvention "packed-u64"`,
   `ontologies/graphlaw-wasm.ttl:37`).
3. Host reads `out_len` bytes at `out_ptr` (UTF-8 JSON), then
   `gl_free(ptr: *mut u8, len: u32)`.

A response is always JSON and never empty: the shell substitutes `{}` for an
empty core response (`emptyResponsePolicy "substitute-empty-object"`,
`ontologies/graphlaw-wasm.ttl:36`; `wasm/src/ffi.rs:125-128`), so the host's
free always reclaims a non-zero-length allocation. Null pointer or oversize
length never traps — the call returns a typed error response from the safe
core (`wasm/src/ffi.rs:111-123`).

## Side-table contract (allocation discipline)

Every buffer crossing the boundary is registered in a bounds-checked side
table (`LIVE`, `wasm/src/ffi.rs:100-140`) at publish time — key = raw pointer,
value = (real `Vec::capacity()`, initialized `length`, host-visible `declared`
length, outstanding-byte `budget`). The guest never supplies layout facts.
Every guest `(ptr, len)` pair is looked up by handle before any
reconstruction; a buffer is rebuilt (`Vec::from_raw_parts`) only from the
recorded capacity, and only when the host length equals the declared length
(`wasm/src/ffi.rs:159-181`). Unknown pointers, mismatched lengths, and a
second free never rebuild a `Vec`: they yield typed error responses from the
safe core (`missing_buffer_response` / `unknown_buffer_response` /
`limit_response`) or are no-ops — never a trap, never undefined behavior.

This contract lives in the pack template
(`ggen-marketplace` `rust-wasi-wasmex-pack` `templates/abi/ffi.rs.tmpl`),
not in the rendered file: a `ggen sync` re-render cannot regenerate the
legacy host-trusting `Vec::from_raw_parts(ptr, len, len.max(1))`
reconstruction, which is UB when the allocator's capacity is larger. The
template's tests (rendered into `wasm/src/ffi.rs` under
`#[cfg(all(test, not(target_arch = "wasm32")))]`) pin the behavior: mismatched
lengths leave the entry live, unknown pointers and double frees are no-ops,
and calls consume the request exactly once.

## Limits (from the wja: graph)

Limits are declared in `ontologies/graphlaw-wasm.ttl:30-32`, rendered into
`wasm/src/abi_meta.rs:17-26`, and asserted equal to the hand-written core in
`src/abi.rs:45-59`:

| limit | value | ttl | abi_meta.rs |
|---|---|---|---|
| `wja:maxRequestBytes` | 16 MiB (16,777,216 B) | ttl:30 | wasm/src/abi_meta.rs:20 |
| `wja:maxOutstandingBytes` | 16 MiB (16,777,216 B) | ttl:32 | wasm/src/abi_meta.rs:29 |
| `wja:maxJsonDepth` | 64 | ttl:31 | wasm/src/abi_meta.rs:23 |

The outstanding-byte cap counts every buffer handed to the host
(`gl_alloc` reservations and `gl_call` responses) not yet freed; once the cap
would be exceeded, `gl_alloc` returns null, so a host that never frees cannot
fill linear memory (`wasm/src/ffi.rs:43-61, 82-91`).

## Compile profile

Build prerequisite: `WASI_SDK_PATH` must point at a wasi-sdk root
(`bin/clang` and `share/cmake/wasi-sdk-p1.cmake`). Locally the unpacked
engine cache `~/.frozen-duckdb/cache/v2.0.0-dev-wasm32-wasip1` doubles as
the SDK root; CI installs wasi-sdk at `/opt/wasi-sdk`. frozen-duckdb-sys
links `libduckdb.a` from that tree when the variable is set; without it the
build fails typed at its `build.rs`.

**Archive law (fail-closed):** the frozen-duckdb wasm static archive must be
built uniformly new-EH — every compile line carries
`-fwasm-exceptions -mllvm -wasm-use-legacy-eh=0`. clang 24 otherwise emits
legacy-EH `try` instructions, which wasmtime 48 rejects at validation
(legacy EH was dropped) and node 26 rejects as mixed-EH. The archive is
produced by `/Users/sac/frozen-duckdb/scripts/build_wasi_duckdb.sh`, which
encodes the flag on both the wrapper compile and the CMake CXX flags lines.
Any future archive rebuild that drops the flag yields a module that fails
wasmtime validation — that failure is the law firing, not a tooling bug.

```sh
WASI_SDK_PATH="$HOME/.frozen-duckdb/cache/v2.0.0-dev-wasm32-wasip1" \
  CARGO_TARGET_WASM32_WASIP1_RUSTFLAGS="-C target-feature=+exception-handling" \
  cargo build --locked -p graphlaw-wasm --target wasm32-wasip1 --profile wasm
# -> target/wasm32-wasip1/wasm/graphlaw_wasm.wasm (~45 MB class, new-EH)
```

The `+exception-handling` RUSTFLAGS target feature is required so the
module's own code matches the archive's EH model; the artifact hosts under
`graphlaw-run-eh` (wasmtime 48, `wasm_exceptions`). wasmi 2.0.0 rejects the
DuckDB-linked artifact and is used only for legacy pre-duckdb pin courts.

`wasm/Cargo.toml:25-31` defines `[profile.wasm]` (inherits release,
`opt-level = "s"`, lto, one codegen unit, strip, `panic = "abort"`), kept
separate so native release builds and tests are unaffected. The `wasm32-wasip1`
module imports only `wasi_snapshot_preview1` (clock, random, stdio) — no
JavaScript glue; the lib facade also builds clean under
`wasm32-unknown-unknown` with **zero WASI imports** (clock routed through a
wasm-bindgen `Date.now()` extern in `graphlaw-eyeron`; verified by
`scripts/wasm_unknown_unknown_smoke.sh`, and the import set asserted at
runtime by `tests/wasm_abi.rs:56-64`). The host-side
integration test `tests/wasm_abi.rs` builds the module (into
`target/wasm-abi` by default, or `$GRAPHLAW_WASM` to test a prebuilt module)
and drives all fourteen ops in a real wasmi runtime
(`tests/wasm_abi.rs:1-4`) — historically a legacy-pin court; since the
2026-10-23 re-pin the pinned `priv/graphlaw.wasm` is itself the new-EH
DuckDB-linked artifact, and `graphlaw-run-eh` hosts it under wasmtime 48.

## Artifact pin

The shipped artifact is pinned by digest and size:

- The current pin is the new-EH DuckDB-linked artifact:
  `fa1558306690493249b176ec312a4f01e97798937e05cd49f2946d61a7e6d107`
  (BLAKE3, verified with `b3sum priv/graphlaw.wasm`), 57,781,384 bytes,
  Tag(13) section count 1, wasm_audit PASS — re-pinned 2026-10-23 in
  `registry/ARTIFACTS.blake3` (`wasm` row) and `priv/graphlaw.wasm.blake3`
  (dual pin: registry pin + sidecar next to the tracked `priv/graphlaw.wasm`);
  `graphlaw-registry --check` exit 0. Receipt:
  `receipts/2026-10-23-eh-closure-wave.md`. Crate version is 26.10.22.
  See [WASM.md](WASM.md). The legacy pre-DuckDB pin
  `f149433c6e1a84ec95e961ac92b16b65b28e15d6fd8546beb1779d3b54a02e7e`
  (7,104,147 bytes, tag section count 0, `pin-source=26.10.22`) is
  historical. Earlier workspace pins:
  `74ec3c8d6cdb3db1ff1869ac8b2454c2cf94d08f2a211b0b7d2af2c36b08a7a5`
  (7,098,370 B, 26.10.21);
  `332ea2188fafb465666e71407e8f940c5b60a15184e6ad6567bb4f723fe2c30c`
  (6,793,617 B); (the 26.10.13-era pin was
  `cbc80f07346f025d4e1cd001f21d15474928cb055d4b1a9c0ac7c31c5938dc6a`,
  6,654,496 B.)
- Reproducibility was witnessed 2026-09-30: two builds, each after
  `cargo clean -p graphlaw-wasm --target wasm32-wasip1 --profile wasm` in the
  same target dir (dependency crates cached), same machine, produced identical
  bytes. Not verified across separate target dirs or machines.
- The wasm line is a **pin, not a live check** — regenerate after any `src/`
  change.

**Superseded claims**: 2026-10-07 — lane W637's build claim
`b7664a5e…` (6,657,549 B) was rejected. 2026-10-10 — the earlier pin
`fc23a292…` (6,657,708 B) is superseded; `2802cede…` (6,659,724 B), the
26.10.5-era pin `02a54c48…` (6,648,473 B), the 26.10.13-era pin
`cbc80f07…` (6,654,496 B), `332ea218…` (6,793,617 B) and `f149433c…`
(7,104,147 B, tag section count 0) are superseded by the current BLAKE3
pin `fa155830…` (57,781,384 B, DuckDB-linked). Crate version is 26.10.22.
The v26.9.29 release digest `b4cf5dd1…` is a different artifact.

## Pinned 26.10.22 source state

`wasm/Cargo.toml` reads `version = "26.10.22"`. The on-disk pin is
`fa155830…` (57,781,384 B), the new-EH DuckDB-linked mint, re-pinned
2026-10-23 in `registry/ARTIFACTS.blake3` and `priv/graphlaw.wasm.blake3`
in one commit (`receipts/2026-10-23-eh-closure-wave.md`; legacy
`f149433c…` is historical). Never hand-write a digest.
