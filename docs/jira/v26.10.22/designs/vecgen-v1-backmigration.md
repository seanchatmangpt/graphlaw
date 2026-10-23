# vecgen-v1-backmigration — lane N17 receipt (2026-10-22)

Subject: `/Users/sac/graphlaw` @ `615dbfaf94d407cffe174af02804d0dbfb3f8f1d` (main).

Finding (N3): 13 fixtures across the two generator families
`breed_oracle_gen` (prolog, 4) and `plan_replay_oracle_gen` (gps 2 / htn 2 /
contingent_plan 2 / frames 3) were generator-lagged: the committed bytes had
been hand-conformed to SCHEMA v1 (breed output surface: compound-key goals,
verdict MAPS, conformed `expected.derived_facts_sorted` / verdict text /
`| CONFORMED ...` notes, provenance incl. `graphlaw_version`) while the
generators still emitted the pre-conformance shape.

## Change

Both generators now emit SCHEMA v1 natively:

- provenance: `graphlaw_rev` = live `git rev-parse HEAD`
  (`615dbfaf94d407cffe174af02804d0dbfb3f8f1d`), `graphlaw_version` = root
  `Cargo.toml` `[package] version` (26.10.21) — same pattern as
  `scripts/gen_wasm4pm_graphlaw_vectors.py:32-49` (N3-3 stamp).
- `breed_input`: compound-key goals (`predicate` = full
  `pred:s,o` key, `value` = "true"), fact `value` "true", split rule ids,
  `certainty` 1.0, explicit empty `candidates`/`cases`/`state`.
- `expected`: verdict MAPS and the conformed facts/verdict text/notes
  (committed fixture bytes are the semantic reference, transcribed verbatim),
  emitted natively instead of recomputed-then-hand-conformed.
- Engine replay is retained as a generator-side cross-check: every conformed
  fact must be in the native fixpoint (or, for refusal vectors, absent), and
  the engine goal verdict must agree with the conformed verdict. Generation
  asserts loudly on drift.
- `--verify` upgraded: engine re-evaluation cross-check + parsed-JSON
  compare of disk vs fresh emission (modulo provenance) + provenance
  well-formedness.

## Verification (real output, this session)

- `cargo check` (full crate): exit 0.
- Regen: `cargo run --example breed_oracle_gen` → 4 vectors;
  `cargo run --example plan_replay_oracle_gen` → 9 vectors; exit 0.
- Parsed-JSON compare OLD committed vs NEW generated (provenance stripped):
  13/13 equal (FAILS: []).
- Generator `--verify`: 4 and 9 vectors, both exit 0.
- wasm4pm oracle gate: `cargo test -p wasm4pm-cognition --test graphlaw_vector_oracle`
  (CARGO_TARGET_DIR=target-lane-n17): **4 passed / 0 failed**; 48 vectors
  loaded (47 confirm, 1 twin), all green.
- graphlaw vecgen courts: `cargo test --test corpus_conformance vecgen`:
  8 passed / 0 failed.

## Fixtures regenerated (13, per-family)

- prolog (breed_oracle_gen): prolog_paper_fixture, prolog_recursive_closure_cycle,
  prolog_multi_rule_join, prolog_no_derivation.
- plan_replay (plan_replay_oracle_gen): gps-paper-fixture,
  gps-multistep-frame-persistence, htn-paper-fixture-go-to-dest,
  htn-multistep-travel, contingent-plan-vacuum-paper-fixture,
  contingent-plan-four-worlds-sense-split, frames-chain-reachability,
  frames-diamond-reachability, frames-cycle-refusal.

Files: `/Users/sac/wasm4pm/crates/wasm4pm-cognition/tests/fixtures/graphlaw/<id>.json`.
Old bytes preserved at `/tmp/n17-old-fixtures/` for replay.

## Falsifier / standing

- Falsifier: regenerating must reproduce the committed bytes modulo
  provenance (parsed-JSON compare); a mismatch, or a red gate, fails the lane.
- Standing: ALIVE for the two generator families; other families (strips,
  partial_order_plan, situation_calculus, event_calculus, ...) are NOT
  touched by this lane.
- Note: fixtures now pin the live rev `615dbfaf…`; the wasm4pm ancestry court
  checks the fixed const `82c67c2b…` (ancestor of main) and passed.
