# Machine-Shaped Gate — design (lane M09, K1 side-arm)

## Defect class

Test fixtures hardcode `/Users/<user>/...` absolute paths (K1 side-arm witness:
`tests/chicago_validation.rs` InstallStatement fixture pinned
`/Users/sac/.duckdb/extensions`). "Suite green" is then machine-shaped: green
on the author's box, red on any other checkout, and green-for-the-wrong-reason
on the author's box when the fixture silently reaches outside the repo.

## Law

A test fixture path must be derived from the environment
(`std::env::temp_dir()`, `$HOME`, or an existing fixture file) or explicitly
allowlisted. An unallowlisted `/Users/` literal in `tests/` or `src/` fails
the gate.

## Gate

`scripts/machine_shaped_gate.sh [dirs...]` (default `tests src`):

- Scans `*.rs`, `*.sh`, `*.py`, `*.ttl` under the given roots.
- Exempts comment lines (Rust `//`, `///`, `//!`, `/*`, `*`; shell/TTL `#`).
- Exempts lines carrying `MACHINE-PINNED: <reason>` (same line or the
  immediately preceding line).
- Exit 1 with a file:line listing on any other `/Users/` literal. Fail-closed.

`--self-test` is the anti-vacuity court: plants a `/Users/` literal in a
scratch dir and requires the gate to exit 1; requires the `MACHINE-PINNED`
allowlist and the comment exemption to each suppress their intended class.
A gate that cannot exit 1 on the planted literal is FAIL.

## Current findings (2026-10-22 scan)

Lines that would need `# MACHINE-PINNED:` allowlisting if the gate were
extended beyond its lease (these live in lanes not owned by M09, so M09
reports rather than edits):

- `tests/insa_law_order.rs:32` — ONTOLOGY_PATH into ~/ggen-marketplace pack
- `tests/refusal_crosswalk_differential.rs:33` — SIBLING_ROOT pack root
- `tests/gws12_coldchain_cross_engine.rs:77` — fallback tmp base under ~/smalt
- `src/bin/dl_dualroute.rs:313,388` — `-C /Users/sac/graphlaw` and wasm4pm fixture out-dir
- `src/bin/graphlaw_run.rs:443` — debug log under /Users/sac/graphlaw/.cursor

Non-code occurrences (doc comments, fixture provenance notes in
`tests/fixtures/**` and `tests/pn64_abi_conformance.rs:403,406`) are comment
lines — exempt by design.

## chicago_validation.rs fix

The InstallStatement fixture needs only *a* repository path string to
compile; it now derives
`std::env::temp_dir().join("graphlaw-duckdb-extensions")` instead of the
pinned `/Users/sac/.duckdb/extensions`, matching the env-derived pattern the
`ExtensionRepositoryStatement` fixture already uses (`/tmp/graphlaw-ext-repo`).

## Verification ladder rung used

1. `bash scripts/machine_shaped_gate.sh --self-test` — anti-vacuity
2. `bash scripts/machine_shaped_gate.sh` — repo scan
3. `CARGO_TARGET_DIR=target-m9 cargo test --test chicago_validation` — real
   frozen-duckdb run of the fixed fixture
