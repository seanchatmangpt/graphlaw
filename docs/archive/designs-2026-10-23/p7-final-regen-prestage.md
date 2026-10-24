# P7 pre-stage: final registry regen procedure — lane L150 (2026-10-22)

GO/NO-GO row: `designs/integration-gonogo.md:47` (P7, "queued AFTER L125").
Executing lane: L147-class fix, coordinator-owned git, one canonical checkout
`/Users/sac/graphlaw`.

## 1. Exact procedure (in order, no interleaved lanes editing)

1. Quiescence precondition: no lane owns or is editing `src/duckdb_eval.rs` or
   `src/sql_ast.rs` (see §3). Confirm `git status` shows no in-flight edits to
   those two files before starting.
2. Regen (the emitter's own hint, same command as
   `designs/registry-regen-receipt.md:5` and
   `designs/precommit-acceptance-receipt.md:38`):

       cargo run --features abi --bin graphlaw-registry -- --write

   Expected: writes `registry/capability-registry.json` and
   `registry/capability-registry.ttl`, re-pins the three owned rows of
   `registry/ARTIFACTS.blake3` (`registry`, `surface`, `receipt-surface`),
   exit 0. Byte-touches ONLY the owned rows — see §4.
3. Acceptance re-run (all 8 targets, single invocation, isolated target dir —
   L147's command, `designs/precommit-acceptance-receipt.md:5`):

       CARGO_TARGET_DIR=target-lane-l147 cargo test --test chicago_validation --test sql_rdf_dialect --test sql_rdf_laundering_corpus --test registry_artifacts --test trust_attest --test duckdb_statement_kinds --test query_cache_court --test registry_dispatch

   Expected: **117 passed / 0 failed** (9+25+23+22+10+6+8+14; per-target table
   `designs/precommit-acceptance-receipt.md:14-21`, with registry_artifacts
   18 pass + the 4 drift rows now green = 22).

## 2. Pre-verified without P3 (this lane, read-only)

- **Command form**: verified against `designs/registry-regen-receipt.md:5`
  (`cargo run --features abi --bin graphlaw-registry -- --write`)
  and `src/bin/graphlaw_registry.rs:35` (usage string lists owned rows
  `registry`, `surface`, `receipt-surface`; wasm never touched here).
- **Acceptance target list**: the 8 targets above, verbatim from
  `designs/precommit-acceptance-receipt.md:5`. Expected 117/0 = 9+25+23+22
  (registry_artifacts repaired to full 22) +10+6+8+14; arithmetic re-checked
  this session.
- **Registry row semantics** (`src/bin/graphlaw_registry.rs:88-139, 251`):
  - `--write` re-pins exactly three owned rows: `registry` (live
    `registry_blake3` over the document minus itself),
    `surface` (ABI op/dialect surface digest), `receipt-surface`
    (per-file blake3+bytes of the receipt-minting surface).
  - The `wasm` row is parsed out and preserved byte-identical
    (graphlaw_registry.rs:96, 129-139); if absent the bin refuses
    (":131-132: no wasm row; the bin does not pin build artifacts").
    Header law at `registry/ARTIFACTS.blake3:6`: "wasm row: never re-pinned by
    graphlaw-registry; re-pinned by the wasm rebuild flow."
  - Missing-wasm-row refusal path: graphlaw_registry.rs:129-132 — the bin
    errors rather than minting a wasm pin.
- **Live drift state at pre-stage time**: `registry/ARTIFACTS.blake3:10` pins
  receipt-surface `0cc02ca4…`, while `registry/capability-registry.json:2115`
  carries `receipt_surface_blake3: a82d1f1a…` — the two pins disagree with
  each other AND with the live file bytes; `--write` re-pins all of them from
  live computation, so the exact current values need not be predicted.

## 3. P3-dependency, precisely

L125 owns `wasm/src/lib.rs` and `tests/wasm_duckdb_op_wired.rs`
(`designs/integration-gonogo.md:43`, receipt
`designs/wasm-duckdb-op-wired.md`).

**Finding: unblocked-by-surface.** The receipt-surface composition is only two
files — `src/registry.rs:973-976`: `FILES = &[("src/duckdb_eval.rs", …),
("src/sql_ast.rs", …)]` via `include_bytes!`. Neither `wasm/src/lib.rs` nor
`tests/wasm_duckdb_op_wired.rs` is `include_bytes!`-ed anywhere in the
surface document; grep confirms the only two `include_bytes!` of src files in
`src/registry.rs` are duckdb_eval.rs and sql_ast.rs (registry.rs:974-975).
The registry document's only per-file pins are those two
(`registry/capability-registry.json:2105,2110`). Therefore the regen is
**unblocked by surface** — L125's files cannot move any pinned digest.
It is blocked **only by quiescence policy**: gonogo P7's "queued AFTER L125"
is a serialization rule (do not regen while any lane could still touch
`src/duckdb_eval.rs` / `src/sql_ast.rs`), not a surface dependency. If L125
touches neither of those two src files (it does not, per its ownership), the
regen could legally run NOW; the queue-after rule exists so the regen lands
after the last possible surface-moving edit, and so the post-regen 117/0 run
is not invalidated mid-run by a concurrent surface edit.

## 4. Abort/retry branch (post-regen run ≠ 117/0)

1. **Do not rerun the unchanged failing command.** Classify first from real
   cargo output:
   - Failures in `registry_artifacts` naming `DRIFT ARTIFACTS.blake3
     registry|receipt-surface` or a digest mismatch at
     `tests/registry_artifacts.rs:624/675/716/730` → the regen ran against a
     moving tree (surface edited during/between regen and run). New
     hypothesis: tree not quiescent. Action: re-run `--write`, then
     immediately the 8-target run with no interleaving edits; if it repeats,
     `git diff --stat HEAD -- src/duckdb_eval.rs src/sql_ast.rs` and find the
     concurrent editor before any rerun.
   - Failures outside registry_artifacts (chicago_validation, trust_attest,
     etc.): NOT P7's defect class (P7's defect is pins only). Preserve
     output, classify per the ladder, hand to the owning lane; P7 is not
     done until 117/0 but the regen itself may already be correct — do not
     re-pin against a tree whose failures are non-pin defects.
   - `--write` itself exits nonzero (e.g. missing wasm row,
     graphlaw_registry.rs:129-132 refusal): do NOT hand-mint or hand-edit
     `registry/ARTIFACTS.blake3` — generated files are regenerated, never
     patched (CLAUDE.md "Generated artifacts" law). Fix the trigger, rerun
     `--write`.
2. Any retry must state a new hypothesis; a second identical failure with no
   new hypothesis is BLOCKED, not retried.

## 5. Verbatim canonical commands

    cargo run --features abi --bin graphlaw-registry -- --write

    CARGO_TARGET_DIR=target-lane-l147 cargo test --test chicago_validation --test sql_rdf_dialect --test sql_rdf_laundering_corpus --test registry_artifacts --test trust_attest --test duckdb_statement_kinds --test query_cache_court --test registry_dispatch
