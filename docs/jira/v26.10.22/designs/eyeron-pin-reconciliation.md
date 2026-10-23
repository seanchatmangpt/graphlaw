# Eyeron Pin Reconciliation (L59)

Date: 2026-10-22. Lane L59, repo `/Users/sac/graphlaw`.
Trigger: L29 observed `crates/graphlaw-eyeron/Cargo.toml:3` version `"0.7.8"`
vs `src/lib.rs:189-190` authority revision
`"0.7.7 (eyereasoner/eyeron@d6568f657c19805b64223acf28d74234156bb837, vendored)"`.

## Finding

The two version strings are **different registers, deliberately distinct, and
already reconciled by a court** — this is DOC-ONLY, and the documentation
already exists.

- The `BACKEND_AUTHORITIES` revision names the **upstream** version and commit
  (`0.7.7 @ d6568f65`), by design. `crates/graphlaw-eyeron/UPSTREAM.md`
  states this explicitly: "This crate's own version is 0.7.8: upstream 0.7.7
  plus the GraphLaw changes below. The `BACKEND_AUTHORITIES` revision in
  `graphlaw` records the upstream version and commit."
- The vendored crate version `0.7.8` is the upstream 0.7.7 snapshot **plus
  real functional changes** (not a version-bump-only), landed in commits
  `ae8372c` (Bound reasoner, ABI and wasm resource use) and `fe87783` (Finish:
  wasm stack overflow, typed limits, review fixes, manifest), both
  2026-09-30, then version-bumped in `9531691` (Prepare crates.io release:
  graphlaw-eyeron 0.7.8, fmt, semver gate, 2026-09-30). Diff stat of the
  functional changes over the vendored `src/`:

  ```
  $ git diff --stat 5d4791c HEAD -- crates/graphlaw-eyeron/src   # EXIT=0
  crates/graphlaw-eyeron/src/main.rs     | 108 +++++++++++----
  crates/graphlaw-eyeron/src/reasoner.rs | 238 +++++++++++++++++++++++++++----
  crates/graphlaw-eyeron/src/sudoku.rs   |  33 +++--
  crates/graphlaw-eyeron/src/w3c_rdf.rs  |  31 +++--
  4 files changed, 343 insertions(+), 67 deletions(-)
  ```

  These are the resource-safety caps recorded in UPSTREAM.md: run-wide
  `ReasonerOptions` caps (`max_total_bytes`, `max_closure_facts`,
  `max_total_steps`), `ReasonerLimit::ClosureSize`/`ClosureFacts` (enum now
  `#[non_exhaustive]`), bounded `log:includes`/`list:append`, `string:format`
  precision, exact BigInt sizes, tabled backward goals, WASI clock gate.

## Evidence (commands + exits)

1. Version-bump commit identification —
   `git log --follow --oneline -- crates/graphlaw-eyeron/Cargo.toml` (EXIT=0):
   `9531691` is the only commit touching the manifest after the vendoring
   `5d4791c` (Vendor Eyeron (MIT) as graphlaw-eyeron so GraphLaw can be
   published, 2026-09-28). Commit `9531691` message: "graphlaw 26.9.29 uses
   new Eyeron ReasonerOptions/ReasonerLimit items that crates.io
   graphlaw-eyeron 0.7.7 lacks, so the vendored crate is bumped to 0.7.8 ...
   the upstream version stays 0.7.7 in the BACKEND_AUTHORITIES revision,
   authority_manifest now reads it from UPSTREAM.md."
2. Not a version-bump-only — `git diff --stat 5d4791c HEAD --
   crates/graphlaw-eyeron/src` (EXIT=0; 343 insertions across reasoner.rs,
   main.rs, sudoku.rs, w3c_rdf.rs, per the stat above). The functional
   changes landed in `ae8372c` + `fe87783`; `9531691` changed only
   `Cargo.toml`, `UPSTREAM.md`, and unrelated workspace files (verified via
   `git show 9531691 --stat`, EXIT=0: 10 files, none of the vendored `src/`).
3. Vendored source is frozen since the last functional commit —
   `git diff fe87783 HEAD -- crates/graphlaw-eyeron/src` (EXIT=0, **empty
   diff**): the working tree `src/` is byte-identical to `fe87783`, i.e. it
   still is the d6568f65 snapshot + the two documented deltas. The
   intervening `984a230` / `0a38b68` are an extract/revert pair (±16600 lines,
   net zero on `crates/graphlaw-eyeron/src`, verified via `git show --stat`).
4. Court enforcement — `tests/authority_manifest.rs::eyeron_revision_matches_the_vendored_crate`
   (lines 63-94) asserts `BACKEND_AUTHORITIES`'s eyeron revision string's
   version and commit appear in `crates/graphlaw-eyeron/UPSTREAM.md` and that
   the manifest is `graphlaw-eyeron`. The "0.7.7 vs 0.7.8" split is therefore
   an enforced invariant, not drift.

## Recommendation

**DOC-ONLY (record and leave).** No file changes by the coordinator.

- Reverting the vendored crate to "0.7.7" (PIN-DOWN) is wrong: it would
  collide with the crates.io 0.7.7 release that lacks the resource-safety
  caps (the stated reason for the bump in `9531691`), and would violate the
  court-checked UPSTREAM.md record.
- UPGRADE-AUTHORITY is also wrong: the authority revision deliberately names
  the **upstream** provenance (0.7.7 @ d6568f65), which is still accurate;
  bumping it to 0.7.8 would misattribute the vendored-crate version as
  upstream provenance and break the court's UPSTREAM.md match (`crate version 0.7.7`).
- The reconciliation is already documented (`UPSTREAM.md`) and
  court-enforced (`tests/authority_manifest.rs`).

## Observation (not acted on — outside lane ownership)

Untracked directories `crates/graphlaw-eyeron/src/parser_split/` and
`crates/graphlaw-eyeron/src/reasoner_split/` exist in the working tree (git
status, EXIT=0). They are not referenced by this reconciliation; flagging for
the coordinator.
