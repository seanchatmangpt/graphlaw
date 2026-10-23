# N28 — test-host exceptions gate: diagnosis receipt

Lane: N28, checkout `/Users/sac/graphlaw`, 2026-10-22.
Ownership honored: `tests/common/mod.rs` only. **No edit was made** — the
prescribed fix is inapplicable; evidence below.

## Before / after

| gate | before (L110 census) | after (this lane) |
|---|---|---|
| `cargo test -p graphlaw-wasm` | 9/0 expected | **9/0 PASS** (ran, exit 0) |
| `--test gws4_ipc_injection_court` | BLOCKED (exceptions) | **1/2** — 2 fail with typed `BLOCKED-WASI-SDK` (lane env has no `WASI_SDK_PATH`; typed refusal fired as designed) |
| `--test chicago_fibo` | exceptions-gate failures | **PASS** (all green, incl. `court_covers_every_kind`, boundary tests) |
| `--test op_differential` | exceptions-gate failures | **0/16** — all 16 identical typed `BLOCKED-WASI-SDK` (env, not code) |
| `--test wasm_memory_wall` | exceptions-gate failures | same typed `BLOCKED-WASI-SDK` on artifact-backed tests |
| `cargo build -p graphlaw-wasm --target wasm32-wasip1 --profile wasm` | — | **BLOCKED** without `WASI_SDK_PATH` (typed, build.rs:101); with `wasi-sdk-34`: **BUILD-BROKEN** — `rust-lld: undefined symbol: __gxx_wasm_personality_v0` / `_Unwind_GetWasmLPadContext` from `ub_duckdb_variant_cast.cpp.obj` |

df at start: 12 GiB free (DISK gate refused; purge of `target-lane122/130/159/167`
was permission-denied to this lane). Coordinator purged stale lane roots;
df at test time: 60 GiB free. Gate then ran.

## Premise correction (L110/L109)

1. **The test host engine is wasmi 2.0.0, not wasmtime.** `tests/common/mod.rs:20`
   imports `wasmi::{Engine, ...}`; `Engine::default()` at line 114 is wasmi's.
   There is **no wasmtime dependency anywhere** in the workspace (Cargo.lock has
   zero wasmtime entries). `wasmtime::Config::new().wasm_exceptions(true)` names
   an API that does not exist in this tree.
2. **wasmi 2.0.0 has no exceptions switch at all.** `~/.cargo/registry/.../wasmi-2.0.0/src/engine/config.rs`
   — `features: WasmFeatures` is private, `default_features()` (lines 78-96)
   unconditionally omits exceptions, and the full public setter list is:
   mutable_global, sign_extension, saturating_float_to_int, multi_value,
   multi_memory, bulk_memory, reference_types, tail_call, extended_const,
   custom_page_sizes, memory64, wide_arithmetic, simd, relaxed_simd. No
   `wasm_exceptions`, no `wasm_legacy_exceptions`. Validation delegates to
   wasmparser with those fixed features.

## Empirical evidence (real artifact, 45 MB, `target/wasm-abi` as of 20:38)

- Section scan: the artifact carries **TAG section (id 13), payload `01 00 03`**
  (one legacy exception tag), no tag imports/exports.
- wasmi 2.0.0 probe (`/tmp/ehprobe`, offline, cached wasmi 2.0.0):
  - raw artifact → `ERR: exceptions proposal not enabled (at offset 0x13b8d)` —
    **the exact L110 census error, reproduced**.
  - TAG-stripped artifact → `ERR: legacy exceptions support is not enabled (at
    offset 0x669cad)` — the code section contains legacy `try`/`catch`. Stripping
    the section is not a fix; the instructions are in the code.

## Root cause

DuckDB's C++ objects (frozen staticlib, built by wasi-sdk clang) use C++
exceptions → wasm **legacy EH** instructions + tag in the linked module.
Rust side `panic="abort"` ([profile.wasm], present at HEAD) does not remove
them. The EH is irreducible from the artifact side while the frozen DuckDB
archive is EH-compiled, and **no mainstream Rust wasm runtime executes legacy
EH**: wasmi (none at all), wasmtime (new EH only, no legacy). Even a perfectly
linked artifact cannot validate on wasmi — and today, with `panic="abort"` at
HEAD, it does not even link: the personality symbols `__gxx_wasm_personality_v0`
/ `_Unwind_GetWasmLPadContext` are defined in no wasi-sdk archive on this
machine (llvm-nm swept `wasi-sdk-24` and `wasi-sdk-34` eh/noeh/roots); they
come from Rust std's bundled unwind, which `panic="abort"` excludes.

## Standing

- tests/common/mod.rs: UNCHANGED / UNVERIFIED-AS-FIX (nothing to enable).
- Artifact gate: BLOCKED-LINK (typed) in lane env; BUILD-BROKEN with sdk-34.
- Circuit-breaker candidates for the artifact lane (outside N28): build the
  frozen DuckDB archive with `-fno-exceptions` (DuckDB supports it; removes
  tag + try/catch at the source), or host the module on an engine with legacy-EH
  support, or accept `panic=unwind` + a legacy-EH-capable host.
