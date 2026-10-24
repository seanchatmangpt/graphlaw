# ADR: Crate Splitting of the Law Kernel

- **Status**: Proposed
- **Window**: v26.10.23
- **Scope**: workspace restructuring only; no behavioral change to admission,
  SQL compilation, evaluation, or receipts.

## Context

Lane thrash on the monolith crate is measured, not hypothesized:

- **1,515 transition receipts** in `receipts/` — every lane edit lands on the
  same compile unit.
- **Edit concentration on three files** (v26.10.23 ERRC scan): `src/sql_ast.rs`
  124 writes across 29 edit buckets, `src/duckdb_op*` 50, `src/duckdb_eval.rs`
  47. The write distribution is already crate-shaped; the build graph is not.
- **Lane build roots**: 25 `target-lane*` roots ≈ 120 GiB (~95% of volume),
  because every lane pays the full monolith dependency graph regardless of the
  file it owns.
- **Precedent**: `SPLIT-NOTES.md` records that `src/receipt_chain.rs` was
  already split into frame/record/store modules — the split pattern works, but
  the module split stayed unwired to the build graph. An in-crate module split
  does not reduce what a lane must compile.

Lane check latency and build-root size scale with the crate touched, not with
the monolith. Splitting along the existing edit boundaries makes the lease
boundary and the build boundary the same line.

## Decision

Split three crates out of the root crate, as workspace members:

| New crate | Source | Notes |
|---|---|---|
| `graphlaw-sql-ast` | `src/sql_ast.rs` + `src/sql_ast/` | zero dependencies on eval/op |
| `graphlaw-duckdb-eval` | `src/duckdb_eval.rs` | frozen-duckdb dependency moves here |
| `graphlaw-duckdb-op` | `src/duckdb_op*` | frozen-duckdb dependency moves here |

The root crate keeps `src/law.rs` (admission), `src/attest.rs`, `src/tier.rs`,
`src/dialect.rs`, `src/abi.rs`, and re-exports the split surfaces through its
`lib.rs` so external call sites and the test suite are unchanged.

### The lease-falls-out-of-crate-boundaries property

Once the split lands, a lane's file lease implies its build scope: a lane
leasing `graphlaw-sql-ast` compiles only that crate plus its dependencies; a
lane touching `law.rs` still compiles the monolith but no longer recompiles
sql-ast internals it does not own. `CARGO_TARGET_DIR=target-lane<N>` size then
tracks the leased crate's closure, not the whole tree — this is the mechanism
that retires the 120 GiB lane-root class of ENOSPC failures. The property is
enforceable: `cargo tree -i` from each new crate must not reach another new
crate except through the declared dependency direction below.

## Migration order (dependency-direction-respecting)

1. **`graphlaw-sql-ast` first.** It has zero deps on eval/op, so it can move
   with no downward edges. Wire it, re-export from root `lib.rs`, run the full
   suite before proceeding.
2. **`graphlaw-duckdb-op` next.** Depends on sql-ast types, not on eval.
3. **`graphlaw-duckdb-eval` last.** Depends on both predecessors.

Each step is independently revertable and independently verifiable; no step
changes the dependency direction of an already-moved crate.

## Risks

- **Circular dependencies.** If eval or op reaches back into law/admission
  types held in the root crate, the split deadlocks. Mitigation: check
  `cargo tree -i` at each step before wiring the next; if a cycle appears,
  hoist the shared types into the lowest crate that needs them (sql-ast or a
  small `graphlaw-types`) rather than re-merging.
- **Registry receipt-surface `include_bytes!` paths.** `registry/ARTIFACTS.blake3`
  and `capability-registry.*` paths are resolved relative to `CARGO_MANIFEST_DIR`
  at the call site. Moving a file to a new crate moves the manifest dir; paths
  must be re-anchored (e.g. `env!("CARGO_MANIFEST_DIR")` →
  `concat!(env!("CARGO_WORKSPACE_DIR"), ...)` or an explicit workspace-relative
  constant) and the registry sync flow re-run so the generated artifacts are
  regenerated, never patched.
- **Public API surface.** Downstream (`wasm/`, `crates/graphlaw-eyeron`,
  courts in `tests/`) imports e.g. `graphlaw::sql_ast::*`. The root `lib.rs`
  bridge (`pub use graphlaw_sql_ast as sql_ast;` etc.) keeps every existing
  path valid; the ADR requires zero import-site churn as an acceptance
  condition, so a bridge gap shows up as a compile failure, not a silent break.
- **DuckDB dependency placement.** frozen-duckdb must remain a required
  dependency of the new crates — no feature-gating the SQL surface out of any
  target; graphlaw-wasm still links the whole closure.

## Verification

- **Per step**: `cargo check` at the workspace root, plus
  `cargo check -p graphlaw-sql-ast` (then `-p graphlaw-duckdb-op`, then
  `-p graphlaw-duckdb-eval`) to prove the new crate compiles standalone.
- **Per step**: full host suite unchanged — `cargo test` must show the same
  pass count as the pre-split baseline before the next step begins.
- **End state**: `cargo tree -i` from each new crate confirms the declared
  dependency direction and no cycles; a lane-root size re-measurement confirms
  the lease-falls-out-of-crate-boundaries property (a sql-ast-only lane root
  is materially smaller than a full-suite root).
- **Final**: full ladder per `CLAUDE.md` — `cargo test`,
  `bash scripts/mutation_court.sh` (run solo), wasm boundary build + test.
  A loud SKIP is not a pass.

## See Also

- `SPLIT-NOTES.md` (receipt_chain frame/record/store split precedent)
- `/Users/sac/graphlaw/CLAUDE.md` (lane leases, disk discipline, test ladder)
- `docs/jira/v26.10.23/` (ERRC findings this ADR is filed from)
