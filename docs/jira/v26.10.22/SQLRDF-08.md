# SQLRDF-08 — REFUSAL-04 on the wasm build

- **Id:** SQLRDF-08
- **Title:** REFUSAL-04 on the wasm build
- **Status:** Open

## Conformance target

Section 5 of the pasted sql-rdf Recommendation, the conformance text for v26.10.22. `REFUSAL-04` is `CapabilityRefusal`: a statement that needs an engine feature disabled in the sandbox, including calling DuckDB from WASM.

## Current behavior

[`wasm/Cargo.toml`](../../../wasm/Cargo.toml) line 19 depends on `graphlaw` with `default-features = false` and features `abi` and `wasi-patched-deps` only. The `duckdb` feature is off.

[`src/lib.rs`](../../../src/lib.rs) lines 191–199 compile `duckdb_eval` and `duckdb_op` only under `feature = "duckdb"`. [`src/abi.rs`](../../../src/abi.rs) lines 467–469 register op `duckdb` only under that feature. [`src/rules/mod.rs`](../../../src/rules/mod.rs) likewise gates `builtins`.

`compile_turtle` stays linked on that build. [`src/tier.rs`](../../../src/tier.rs) `admit_sql_rdf` (lines 126–132) compiles a `sql` document to `Admitted::Query` with no DuckDB call. A `sql` document can therefore be admitted on the wasm crate. Nothing in that path returns `REFUSAL-04`.

[`docs/refusals.md`](../../refusals.md) records `DuckDb` for the native substrate. It has no `REFUSAL-04` row. Host contract tests in [`wasm/src/lib.rs`](../../../wasm/src/lib.rs) (`abi_meta_mirrors_registry_ops`, `abi_meta_mirrors_registry_error_codes`, `shell_routes_through_graphlaw_abi`) mirror the registry and the ABI shell.

## Required behavior

A `sql` document presented to `graphlaw-wasm` returns `REFUSAL-04` and does not execute SQL. The wasm crate stays `default-features = false` with no `duckdb` feature. The code is stable in [`docs/refusals.md`](../../refusals.md).

## Acceptance

`cargo build -p graphlaw-wasm --target wasm32-wasip1 --profile wasm`, plus a refusal assertion on that build. `cargo test -p graphlaw-wasm` remains the host contract around [`wasm/src/lib.rs`](../../../wasm/src/lib.rs).
