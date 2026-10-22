# SQLRDF-19 — CI feature-matrix gaps — untested build cells

- **Id:** SQLRDF-19
- **Title:** CI feature-matrix gaps — untested build cells
- **Status:** Open (follow-up)
- **Standing:** **BLOCKED-BY: v26.10.22 integration commit** — no CI change
  lands before the coordinator integrates the v26.10.22 sql-rdf wave (same
  gate as SQLRDF-16/-17/-18).
- **Source:** feature-matrix audit of PR CI vs release CI, 2026-10-22.

## Current behavior (verified on disk, 2026-10-22)

PR CI ([`.github/workflows/ci.yml`](../../../.github/workflows/ci.yml)) runs
exactly **one** native feature combination:

- `cargo clippy --all-targets --features pack-tools,abi,wasi-patched-deps`
  (ci.yml:29) and
  `cargo test --locked --all-targets --features pack-tools,abi,wasi-patched-deps`
  (ci.yml:37) — i.e. `default(abi,duckdb) + pack-tools + wasi-patched-deps`.
- The only other native-feature jobs are single-feature additions on top of
  default (`--features abi` for the registry check, ci.yml:33) and the wasm
  court (`--features abi,wasi-patched-deps`, ci.yml:82) — still never
  varying the default set downward.
- `--all-features` (which adds `evidentiary-ring` + `runner`) appears **only**
  in the release workflow
  ([`.github/workflows/release.yml:85`](../../../.github/workflows/release.yml)),
  and only when the release plan says there is work to do. Between releases,
  these features are never compiled or tested.

Untested cells (feature combinations never built in PR CI):

1. **`runner`** — `src/bin/graphlaw_run.rs`
   (`Cargo.toml:107-109`, `required-features = ["runner"]`) is a shipped bin
   with **zero test coverage anywhere in the tree**: no test file references
   `graphlaw_run`/`graphlaw-run` (grep over `tests/` returns only the bin
   source itself), and it is compiled only in the release job's
   `--all-features` build. A regression that breaks the bin compiles/behaves
   only surfaces at release time.
2. **`evidentiary-ring` without `duckdb`** — the capabilities-counter path
   (`src/abi.rs:477` `#[cfg(all(feature = "evidentiary-ring",
   not(target_arch = "wasm32")))]` vs the `not(...)` fallback at
   `src/abi.rs:506`) compiles but has zero coverage: the ring court
   (`tests/evidentiary_ring_court.rs:6`) gates on `feature = "evidentiary-ring"`
   and runs in CI only under the release-only `--all-features` build, which
   always also has duckdb on.
3. **`evidentiary-ring` + `duckdb` seam push path** — the
   `duckdb_eval` seam push (`src/duckdb_eval.rs:513`, inside the
   evidentiary-ring cfg) is exercised by the ring court only under
   `--all-features`, which is release-only. See also
   `tests/evidentiary_ring_court.rs` and `src/query_ledger.rs:63`.
4. **`--no-default-features` minimal-lib cells** — never built in CI at all;
   the lib's cfg surface (`src/lib.rs:194`,
   `#[cfg(all(feature = "evidentiary-ring", not(target_arch = "wasm32")))]`,
   plus `duckdb`-gated modules) has no minimal-config compile gate, so a
   `cfg`-gate typo breaks downstream minimal builds only.
5. **`duckdb`-without-`abi`, `abi`-without-`duckdb`, `pack-tools`-only** —
   the pairwise/off-default cells of the declared feature set
   (`Cargo.toml:61-87`: `default = ["abi","duckdb"]`, `pack-tools`,
   `duckdb`, `abi`, `wasi-patched-deps`, `runner`, `evidentiary-ring`) are
   never compiled in PR CI.

## Required behavior

1. **Matrix expansion.** Either `cargo hack check --each-feature
   --no-private-dev-deps` (plus `--feature-powerset --depth 2` for the ring
   × duckdb seam cell) as one CI job, or an explicit matrix job listing the
   cells: `--no-default-features`, `--features duckdb`,
   `--features abi`, `--features pack-tools`, `--features
   evidentiary-ring`, `--features evidentiary-ring,duckdb`,
   `--all-features`. Every cell must `cargo check` (and, for cells with
   courts — `evidentiary-ring`, `evidentiary-ring,duckdb`,
   `--all-features` — `cargo test --all-targets`).
2. **Runner smoke test.** At least one court that builds the `runner`
   feature and drives the real `graphlaw-run` bin over a real wasm artifact
   (`priv/graphlaw.wasm` + `priv/graphlaw.wasm.blake3` sidecar) end to end:
   spawn the bin, send one packet-4 frame, assert on the real response frame
   and exit status. Chicago: real subprocess, real artifact, assert on final
   state — no mocks.
3. Release-only is not coverage: cells exercised only by the release
   workflow's `--all-features` step (release.yml:85) must move into or be
   mirrored by a PR CI job.

## Falsifier / Acceptance

- **Falsifier:** each listed cell compiles + tests green in PR CI. If a cell
  is missing from the CI config, or present but skipped/loud-SKIP, the cell
  is still untested and the ticket stays Open.
- **Acceptance:** the CI matrix job green on a PR; the runner smoke court
  green (`cargo test --test <runner_court>` target name chosen at closure)
  with embedded real output in [RECEIPTS.md](RECEIPTS.md). A receipt with no
  embedded output is not a receipt. Anti-vacuity: removing a cell from the
  matrix must make the CI run fail loudly (a matrix that cannot go red is
  FAIL).
