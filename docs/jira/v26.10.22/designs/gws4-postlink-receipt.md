# gws4 post-link receipt — L116

> Note: this receipt's env (`WASI_SDK_PATH`) used the engine cache; the current recipe is a wasi-sdk root, documented in `docs/WASM.md`.

Date: 2026-10-22. Lane L116, repo `/Users/sac/graphlaw`, branch `main` @ 615dbfa.
Target dir: `target-lane116`. Env: `WASI_SDK_PATH=~/.frozen-duckdb/cache/v2.0.0-dev-wasm32-wasip1`.
That path is the DuckDB engine cache, not a wasi-sdk root. Do not reuse it as `WASI_SDK_PATH`. The runbook is `docs/WASM.md`.

## Verdict: FAIL (1 passed / 2 failed) — court still red, root cause narrowed

Command:
```
WASI_SDK_PATH=~/.frozen-duckdb/cache/v2.0.0-dev-wasm32-wasip1 \
CARGO_TARGET_DIR=target-lane116 cargo test --test gws4_ipc_injection_court
```
Exit: 1. `test result: FAILED. 1 passed; 2 failed; 0 ignored; 0 measured; finished in 208.91s`

Per-subtest:
- `gws4_a_corpus_injected_as_linear_memory_buffer_loads_into_duckdb_eval` — FAIL, >60 s (panic `tests/gws4_ipc_injection_court.rs:233:62` "valid wasm: ... exceptions proposal not enabled")
- `gws4_b_chunk_streaming_design_meets_16mib_bound_and_merge_equals_whole` — PASS, 0.34 s (isolated rerun, cached link)
- `gws4_c_fail_closed_when_corpus_exceeds_bounds` — FAIL, >60 s (same panic site)
- Total 208.91 s: gws4_a + gws4_c each spend most of it in the real link inside `Host::new` (tests/common/mod.rs builds `-p graphlaw-wasm --target wasm32-wasip1 --profile wasm`), then fail at module validation.

## The link no longer BLOCKs — it produces an unloadable module

The prior BLOCKED-LINK is gone: `cargo build -p graphlaw-wasm --target wasm32-wasip1 --profile wasm`
succeeds (artifact 45,162,452 B, sha256
`500333a4b876e0d97741ccc96eb5a00c43959714a8d29b172e7dbaff101d2053`, (historical)
`target-lane116/wasm-abi/wasm32-wasip1/wasm/graphlaw_wasm.wasm`).
But the linked module contains a **wasm Tag section** (section id 13, size 3,
offset 80778; entry `01 00 03` = one tag, attribute 0 (`__cpp_exception`),
typeidx 3) and wasmi's validator rejects it:
`exceptions proposal not enabled` (wasmparser 0.228.0, validator/core.rs:958).

## Root cause chain (verified, each hop executed)

1. wasmi `Engine::default()` (tests/gws4_ipc_injection_court.rs:232) →
   `Config::default_features()` omits `WasmFeatures::EXCEPTIONS`
   (wasmi-2.0.0 `src/engine/config.rs:78-95`) and exposes **no setter** to
   enable it (`pub fn` surface: mutable_global .. enforced_limits only).
2. wasmparser 0.228 rejects any tag section unless `features.exceptions()`.
3. The tag comes from the **wasi-sdk `libunwind.a`**
   (`~/.frozen-duckdb/cache/v2.0.0-dev-wasm32-wasip1/lib/wasm32-wasip1/libunwind.a`,
   1 object with a tag section — verified by parsing all members). wasm-ld pulls
   it in when frozen-duckdb-sys links the C++ runtime (build.rs requires
   WASI_SDK_PATH "to link libc++ into {target}").
4. Negative controls, all parsed member-by-member: staged `libduckdb.a` (246 objs),
   `libicu_extension.a` (31), `libcore_functions_extension.a` (25),
   `libduckdb_wasi_compat.a` (1), SDK `libc++.a` (57), `libc++abi.a` (18), and all
   rustc `wasm32-wasip1` std rlibs — **zero tag sections**. Only libunwind.a has one.

## Why this is not fixable inside the court as-is

- wasmi 2.0.0 cannot execute wasm EH even if validation were bypassed.
- `panic = "abort"` is already set ([profile.wasm], Cargo.toml:26); the tag is
  not Rust-side.

## Repair routes (owner decision, not landed by this lane)

- **R1**: link without libunwind's EH object (e.g. `-Wl,--no-demangle`-class knob
  does not exist; the honest knob is excluding `libunwind.a` from
  frozen-duckdb-sys's link line, or a wasi-sdk variant built with
  `-fno-exceptions` unwinder) — module then validates under wasmi defaults.
- **R2**: swap the court host engine to one that supports wasm EH
  (wasmtime with `Config::wasm_exceptions(true)`) for this court only.
- **R3**: wasmi upstream/fork with the exceptions feature.

## Cross-repo gate (L106), status only

`/Users/sac/wasm4pm/tests/graphlaw_vector_oracle.rs` does not exist on disk
(`ls: No such file or directory`); naive-physics fixtures not landed — L106 gate
NOT RUN, recorded BLOCKED(FILE_MISSING). Disk cost avoided accordingly.
