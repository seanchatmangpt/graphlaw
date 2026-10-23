# SQLRDF-08 — REFUSAL-04 on the wasm build

- **Id:** SQLRDF-08
- **Title:** REFUSAL-04 on the wasm build
- **Status:** Withdrawn

## Why this ticket is withdrawn

The required behavior below was written when DuckDB was treated as a Cargo feature that the wasm crate turned off. That assumption is rejected. `graphlaw-wasm` is the one unit. It links frozen-duckdb. A `sql` document admits through `compile_turtle` inside the module, the same path as the other dialects.

`REFUSAL-04` (`CapabilityRefusal`) fires when builtin SQL eval is given a reader that is not document-backed (`src/rules/builtins.rs`, `docs/refusals.md`). It does not fire because the caller is inside wasm.

Historical note: frozen-duckdb 2.0.0 always emitted `dylib=duckdb`, so wasm-ld failed with `unable to find library -lduckdb`. That was a link failure, not a capability refusal. 2.0.2 emits `static=duckdb` for wasm32-wasip1. The link is witnessed (2026-10-22). `WASI_SDK_PATH` must be a wasi-sdk root (`bin/clang`, `share/cmake/wasi-sdk-p1.cmake`); without it, frozen-duckdb-sys `build.rs` fails typed (`WASI_SDK_PATH required`). The recorded pin `332ea218…` (6,793,617 B, `registry/ARTIFACTS.blake3`) was not re-linked in this update.

## Conformance target

Section 5 of `docs/reference/sql-rdf-rec.md`. The trigger is the non-document-backed builtin reader.

## Acceptance

Do not restore `default-features = false` on `wasm/Cargo.toml`, a `duckdb` feature, or a `#[cfg(target_arch = "wasm32")]` strip of the SQL surface. A court that expects every `sql` document on wasm to return `REFUSAL-04` is testing the withdrawn requirement.
