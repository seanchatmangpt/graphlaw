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
  rust-wasi-wasmex-pack (queries/ + templates/abi/). "Rendered by ggen ...
  never edit this file by hand" is the standing header law
  (`wasm/src/ffi.rs:7-8`, `wasm/src/abi_meta.rs:6-7`). Edit the ontology, re-render.
- **Hand-written safe core** — all behaviour lives in `graphlaw::abi`
  (`src/abi.rs`); the only `unsafe` in the wasm crate is pointer handling in
  the generated shell (`wasm/src/lib.rs:5-9`). The contract is three safe
  functions: `graphlaw::abi::call(request: &[u8]) -> Vec<u8>`,
  `limit_response(name, observed, max)`, `missing_buffer_response()`
  (`wasm/src/ffi.rs:22-27`).

Compile-time asserts in `wasm/src/lib.rs:22-27` pin the generated
`abi_meta` limits and version to `graphlaw::abi`'s constants, so an ontology
edit that drifts from the core fails the native build instead of shipping a
mismatch. Two more contract tests keep the generated tables honest:
`abi_meta_mirrors_registry_ops` / `abi_meta_mirrors_registry_error_codes`
mirror `graphlaw::registry` (`wasm/src/lib.rs:37-52`), and
`shell_routes_through_graphlaw_abi` proves the shell has no local limits or
hook (`wasm/src/lib.rs:55-72`).

## gl_* packed-u64 ABI

All integers are wasm `i32`/`i64`. Protocol (`wasm/src/lib.rs:11-20`,
`wasm/src/ffi.rs:9-17`):

1. `gl_alloc(len: u32) -> *mut u8` — host reserves `len` bytes and writes a
   UTF-8 JSON request there. Returns null (0) when `len` exceeds
   `MAX_REQUEST_BYTES` or when a reservation would exceed the outstanding cap
   (`wasm/src/ffi.rs:82-91`).
2. `gl_call(ptr, len) -> u64` — runs the request and consumes (frees) the
   request buffer (`inputReleasePolicy "call-consumes"`,
   `ontologies/graphlaw-wasm.ttl:38`). The return value packs the response
   location as `packed = (out_ptr << 32) | out_len`
   (`wasm/src/ffi.rs:133-145`; `returnConvention "packed-u64"`,
   `ontologies/graphlaw-wasm.ttl:37`).
3. Host reads `out_len` bytes at `out_ptr` (UTF-8 JSON), then
   `gl_free(ptr: *mut u8, len: u32)`.

A response is always JSON and never empty: the shell substitutes `{}` for an
empty core response (`emptyResponsePolicy "substitute-empty-object"`,
`ontologies/graphlaw-wasm.ttl:36`; `wasm/src/ffi.rs:124-128`), so the host's
free always reclaims a non-zero-length allocation. Null pointer or oversize
length never traps — the call returns a typed error response from the safe
core (`wasm/src/ffi.rs:111-123`).

Every boundary buffer is a `Vec<u8>` of capacity `len.max(1)`, and responses
are shrunk to capacity == length, so `_free`/`_call` reconstruct the exact
allocation with `Vec::from_raw_parts` (`wasm/src/ffi.rs:28-33`).

## Limits (from the wja: graph)

Limits are declared in `ontologies/graphlaw-wasm.ttl:30-32`, rendered into
`wasm/src/abi_meta.rs:21-31`, and asserted equal to the hand-written core in
`src/abi.rs:45-59`:

| limit | value | ttl | abi_meta.rs |
|---|---|---|---|
| `wja:maxRequestBytes` | 16 MiB (16,777,216 B) | ttl:30 | abi_meta.rs:23-24 |
| `wja:maxOutstandingBytes` | 256 MiB (268,435,456 B) | ttl:32 | abi_meta.rs:29-30 |
| `wja:maxJsonDepth` | 64 | ttl:31 | abi_meta.rs:25-26 |

The outstanding-byte cap counts every buffer handed to the host
(`gl_alloc` reservations and `gl_call` responses) not yet freed; once the cap
would be exceeded, `gl_alloc` returns null, so a host that never frees cannot
fill linear memory (`wasm/src/ffi.rs:35-55, 82-91`).

## Compile profile

```sh
cargo build --locked -p graphlaw-wasm --target wasm32-wasip1 --profile wasm
# -> target/wasm32-wasip1/wasm/graphlaw_wasm.wasm
```

`wasm/Cargo.toml:20-28` defines `[profile.wasm]` (inherits release,
`opt-level = "s"`, lto, one codegen unit, strip, `panic = "abort"`), kept
separate so native release builds and tests are unaffected. The module's only
imports are `wasi_snapshot_preview1` (clock, random, stdio) — no JavaScript
glue; `wasm32-unknown-unknown` is not a supported target
(`tests/wasm_abi.rs:56-60` asserts the import set at runtime). The host-side
integration test `tests/wasm_abi.rs` builds the module (into
`target/wasm-abi` by default, or `$GRAPHLAW_WASM` to test a prebuilt module)
and drives all fourteen ops in a real wasmi runtime
(`tests/wasm_abi.rs:5-10`).

## Artifact pin

The shipped artifact is pinned by digest and size:

- digest `fc23a2927187029ade92a4a64abd2de2cd15147be0cd95c70d89504a1aadcb38`,
  6,657,708 bytes — `registry/ARTIFACTS.sha256:6` and
  `priv/graphlaw.wasm.sha256:1` (dual pin: registry pin + sidecar next to the
  tracked `priv/graphlaw.wasm`).
- Reproducibility was witnessed 2026-09-30: two builds, each after
  `cargo clean -p graphlaw-wasm --target wasm32-wasip1 --profile wasm` in the
  same target dir (dependency crates cached), same machine, produced identical
  bytes. Not verified across separate target dirs or machines
  (`registry/ARTIFACTS.sha256:2-3`).
- The wasm line is a **pin, not a live check** — regenerate after any `src/`
  change (`registry/ARTIFACTS.sha256:4`).

**Superseded claim**: 2026-10-07 — lane W637's build claim
`b7664a5e…` (6,657,549 B) was rejected; the pinned artifact
`fc23a292…` (6,657,708 B) is authoritative.

## Intentional 26.10.5 version

`wasm/Cargo.toml:4` reads `version = "26.10.5"` deliberately: the wasm pin
(`fc23a292…`) was minted against that source state, and a version bump would
invalidate the pin — the digest is of the bytes built from 26.10.5 sources.
Regenerate the artifact and re-pin (both files) before/with any version bump.
