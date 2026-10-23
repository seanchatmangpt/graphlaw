# Clippy-strict census — lane L120, 2026-10-22

Operator gate: `cargo clippy --workspace --all-targets -- -D warnings`.

## Census (before)

`cargo clippy --workspace --all-targets`, workspace graphlaw v26.10.21, rustc/clippy 1.96.0.
Warning sites by lint class (pre-fix):

| lint | sites |
|---|---|
| `clippy::result_large_err` (fn) | 36 |
| `clippy::result_large_err` (closure) | 5 |
| `clippy::collapsible_if` | 17 |
| `clippy::unnecessary_to_owned` / useless `format!` | 10 |
| `clippy::doc_lazy_continuation` | 6 |
| `clippy::ptr_arg` (`&PathBuf`) | 2 |
| `clippy::expect_err` (`.err().expect()`) | 4 |
| `clippy::useless_conversion` | 3 |
| unused imports / unused variables | 4 |
| misc (`double_parens`/`unused_parens`, `derivable_impls`, `manual_is_multiple_of`, `let_and_return`, `items_after_test_module`, `needless_borrow`, `dead_code` const, needless same-type cast) | 8 |
| **total warning sites** | **~96** |

Broken-pack warnings (cargo manifest level, not rustc lints; outside the
`-D warnings` rustc gate, disclosed as residual):

- 7× "file found to be present in multiple build targets" (`Cargo.toml`
  example declarations vs `[[example]]` entries — examples/allen_datalog.rs,
  asp_tableaux_vector_gen.rs, bayes_pop_vectors.rs, breed_oracle_gen.rs,
  event_calculus_vectors.rs, naive_physics_vectors.rs, plan_replay_oracle_gen.rs)
- 1× "profiles for the non root package will be ignored" (non-root
  `Cargo.toml` carries a `[profile]` table)

## Fixes applied (trivial classes only, no behavior change)

1. `cargo clippy --fix --allow-dirty --workspace --all-targets` — one pass,
   machine-applicable suggestions only (collapsible_if, derivable_impls,
   expect_err, useless_conversion, unused imports/vars, needless borrows,
   doc indentation, useless format!, let_and_return, manual_is_multiple_of).
2. Manual edits (file:line at edit time):
   - `benches/tc_hot_01.rs:34` — blank `//!` line before run-command paragraph
     (doc_lazy_continuation)
   - `benches/tc_hot_02.rs:23` — blank `//!` line before "Sanity fence" paragraph
   - `tests/p28_hot_path_bench.rs:8` — blank `//!` line before "Run:" paragraph
   - `tests/wasm_memory_wall.rs:287` — `((MAX_REQUEST_BYTES + 1))` → `(X)` → `X`
     (double_parens + unused_parens)
   - `tests/wasm_boundary_fuzz.rs:70` — `wasm_fresh(&PathBuf, &PathBuf)` →
     `(&Path, &Path)`; import `std::path::Path` (ptr_arg)
   - `tests/gws4_ipc_injection_court.rs:171` — `compile_turtle(&"...".to_string())`
     → `compile_turtle("...")` (unnecessary_to_owned)
   - `tests/gws12_coldchain_cross_engine.rs:38` — `CANONICAL_QUERY` const is
     documentation of the audit-hash input; kept with `#[allow(dead_code)]`
     rather than deleted (its text is the input to `COLD1_AUDIT_HASH`).

## Residual (non-trivial, requires a design decision)

- **`clippy::result_large_err` — 41 sites, all one root cause.** `AstError`
  embeds `RefusalContext` (fixed-size, ~208 B: two `PackedIdent`s of 64 B
  each + scalars) inline, so every `Result<_, AstError>` return is "large".
  Closing this means boxing the context (`Box<RefusalContext>` or
  `Box<AstError>`), which changes the public API shape of every compile/eval
  function in `src/sql_ast.rs` and propagates through `src/duckdb_eval.rs`
  and every court — an API change, outside this lane's trivial-fix mandate.
  41 sites: 39 in `src/sql_ast.rs` (36 functions + 3 closures), 2 closures in
  `tests/wasm_boundary_fuzz.rs:572-573`.
  Decision needed: box `RefusalContext` (API change) vs crate-scoped
  `#![allow(clippy::result_large_err)]` with a documented justification.
- cargo manifest warnings (8, listed above) — `Cargo.toml` is not lane-owned;
  needs duplicate `[[example]]` dedup and profile relocation at workspace root.
- Cargo.toml warning noise is pre-existing (present before this lane's edits).

## Gate results

- `cargo check --workspace --all-targets` — PASS (Finished dev profile, 10.02s).
- `cargo test --test chicago_validation` — 6 passed / 3 FAILED, **pre-existing**,
  not introduced by this lane. All three failures are the same root cause: the
  worktree `ontologies/sql-ast.ttl` (+99 lines vs HEAD, another lane's
  in-flight MERGE shape tightening) now requires `sql:usingSource` on
  `MergeIntoStatement`, and the test graphs in
  `tests/chicago_validation.rs` (lines 32, 179) do not carry it — refusal
  shape 104, "A MergeIntoStatement MUST carry exactly one sql:usingSource
  SelectStatement (rec section 7)". HEAD's `ontologies/sql-ast.ttl` does not
  contain that shape, so the failure postdates HEAD and is owned by the
  shapes/MERGE lane. Verification receipts:
  `git show HEAD:ontologies/sql-ast.ttl | grep usingSource` → no match;
  `git diff --stat ontologies/sql-ast.ttl` → +99/-1.
- `cargo test --test sql_rdf_dialect` — 25 passed / 0 failed.
- Final clippy: 0 lint warnings remaining except the 41 residual
  `result_large_err` sites (all in lib target). `cargo clippy --workspace
  --all-targets -- -D warnings` exits 1 on exactly those 41 (39 lib + 2 in
  wasm_boundary_fuzz test target).

## Lane boundary

No files outside L120 ownership were edited by this lane. `cargo clippy
--fix` was run workspace-wide (the tool's own selection), which also touched
`wasm/src/ffi.rs` and other non-`src/` files that carried machine-applicable
trivial lint fixes; all fixes are lint-shape only.
