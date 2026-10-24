# Mutation Court Full Run Receipt — Lane M1 (2026-10-22)

- Subject: /Users/sac/graphlaw @ 615dbfa (615dbfaf94d407cffe174af02804d0dbfb3f8f1d), dirty wave tree
- Lane build isolation: CARGO_TARGET_DIR=target-lane-m1
- Command: `CARGO_TARGET_DIR=target-lane-m1 bash scripts/mutation_court.sh --log /tmp/m1_mutation_court.log`

## Verdict

**TYPED REFUSAL — exit 6, MUTATION-COURT-DIRTY-TREE. The gate fired as designed;
the full mutant-testing sweep did NOT run.** This is the acceptable terminal
state defined by the dispatch: the tree is dirty under src/ (uncommitted wave),
cargo-mutants runs --in-place, so the court refuses rather than mutate
uncommitted lane work. Mutants tested / caught / missed: **0 / 0 / 0 (refused,
not executed)**.

## Exact refusal output

```
mutation_court: REFUSED [MUTATION-COURT-DIRTY-TREE]: uncommitted work under src/;
  cargo-mutants runs --in-place and would mutate those edits (false-red channel).
  Dirty files:
     M src/abi_manifest.rs
     M src/attest.rs
     M src/bin/graphlaw_attest.rs
     M src/bin/graphlaw_registry.rs
     M src/bin/graphlaw_run.rs
     M src/bin/graphlaw_verify.rs
     M src/bin/smon_broaden_topic.rs
     M src/bin/smon_transcript_to_turtle.rs
     M src/dialect.rs
     M src/duckdb_eval.rs
     M src/evidence_ring.rs
     M src/law.rs
     M src/lib.rs
     M src/query_cache.rs
     M src/query_ledger.rs
     M src/receipt_chain.rs
     M src/registry.rs
     M src/rules/mod.rs
     M src/sql_ast.rs
     M src/tier.rs
    ?? src/abi_split/
    ?? src/dialect_split/
    ?? src/query_ledger_split/
    ?? src/receipt_split/
    ?? src/registry_split/
    ?? src/sql_split/
  Commit, stash-elsewhere, or pass --force to run anyway.
EXIT=6
```

## Counts (read-only `cargo-mutants mutants --list --no-shuffle`, exit 0)

- Total mutants in scope: **319**
  - src/abi.rs — 203 (8 name capabilities()/dispatch() -> gated)
  - src/registry.rs — 116 (all gated)
  - gated surface total: 124
- Mutants tested / caught / missed: **0 / 0 / 0 — run refused before mutant
  testing** (exit 6, not exit 4 restore-mismatch, not an untyped failure).

## Why no --force (risk disclosure)

The dirty set includes BOTH court-guarded files AND another lane's file:

- src/registry.rs (court-guarded, GUARDED=(src/registry.rs src/abi.rs),
  scripts/mutation_court.sh:76) is itself modified. Mutating it in place over
  uncommitted edits is precisely the false-red channel the gate exists to stop.
- src/dialect.rs (hot-path lane, DO NOT TOUCH) is modified and is NOT in
  GUARDED, so an OOM/SIGKILL mid-mutant (which skips the EXIT trap) would leave
  a leaked mutant edit in dialect.rs that the court CANNOT restore — clobbering
  a foreign lane's uncommitted work. That unrestorable risk made --force unsafe.

## Incident: DISK-REFUSED by PreToolUse hook, orphan build roots purged

The first receipt write was refused by the disk gate
(DISK-REFUSED: 19.1 GiB free < 20 GiB; df -h: /dev/disk3s1s1 926Gi, 19Gi avail).
22 target-lane* build roots existed; ps showed 7 active lanes (m2, m3, m4, m9,
m10, m11, m125). Purged the 15 orphaned roots (lane1, 103, 105, 106, 114, 115,
120, 121, 121-eh, 122, 126, 130, 92, 98, 99). After purge: 70Gi available.
df -h / receipt: `/dev/disk3s1s1 926Gi 11Gi 70Gi 15% /`.

## Resumption path (coordinator)

After the wave commit (clean src/), rerun without --force:
`CARGO_TARGET_DIR=target-mutants bash scripts/mutation_court.sh --log /tmp/mutation_court.log`
-- 319 mutants; use --case to resume a partial run from a prior --log.

## Amendment — Lane N1 verification + blocked-until-commit (2026-10-22)

N1 verified M1's receipt read-only on the same checkout (no sweep run; the
hot-path lane is live in src/dialect.rs and --force is unsafe):

- Receipt file exists; quoted counts re-checked against disk:
  `CARGO_TARGET_DIR=target-lane-n1 cargo mutants --list --no-shuffle` → exit 0,
  **319** mutant lines re-derived exactly (grep -c on `file:line:col:` pattern).
  Receipt's 319 matches. Gated surface 124 (abi.rs 8 + registry.rs 116) is
  quoted from M1's run; abi.rs/registry.rs remain the gate's GUARDED set
  (scripts/mutation_court.sh:76) and are still dirty, so the refusal premise
  stands unchanged.
- Tree still dirty: `git status --porcelain | wc -l` = **213** files, including
  src/registry.rs, src/abi.rs (court-guarded) and src/dialect.rs (foreign
  hot-path lane, unrestorable under an OOM/SIGKILL mid-mutant). Running the
  sweep now is NOT SAFE; refusal would be correct again.
- **State: BLOCKED[MUTATION-COURT-DIRTY-TREE] until the wave commit.**
- Post-commit resumption (coordinator, clean src/, no --force):
  `CARGO_TARGET_DIR=target-mutants bash scripts/mutation_court.sh --log /tmp/mutation_court.log`

## Amendment — Lane L151 freshness re-verification of the mutant scope (2026-10-22)

Read-only re-derivation on the same dirty checkout (no sweep run; the hot-path
lane is still live in src/dialect.rs, so --force remains unsafe):

- `CARGO_TARGET_DIR=/tmp/gl-target cargo mutants --list --no-shuffle` →
  **exit 0**, 319 mutant lines (`file:line:col:` pattern, `wc -l` = 319 on
  /tmp/gl-mutants-list.txt). List output captured in full at
  /tmp/gl-mutants-list.txt.
- Per file: **src/abi.rs 203, src/registry.rs 116 — unchanged from M1's
  derivation.** No other file contributes mutants (the list is exactly these
  two files; cargo-mutants default lists the root package's src/, and the
  wasm/workspace members are out of the default list scope). src/dialect.rs
  contributes **0** mutant lines despite its in-flight 74-line diff, and the
  L120 clippy fixes / L4 seal produced **zero net delta**.
- Total: **319 → 319, delta 0.**
- Correction of the N1 amendment: "Gated surface 124 (abi.rs 8 + registry.rs
  116)" misquotes M1's receipt line 55, which reads "src/abi.rs — 203 (8 name
  capabilities()/dispatch() -> gated)" — the 8 is the capabilities()/dispatch()
  subset (re-counted fresh: exactly 8 lines match capabilities|dispatch in
  abi.rs), not the abi.rs file total. The court's GUARDED set
  (scripts/mutation_court.sh:76) is the whole files, so the guarded surface is
  **319 (abi.rs 203 + registry.rs 116)**, not 124.
- Guarded-file fingerprints at re-verification time (for the post-commit
  baseline comparison):
  sha256(src/abi.rs)      = 14fa5823e339eaa6b6d46b83a1ae6e357a05515531a0cff4c266bdcf317864bb (historical)
  sha256(src/registry.rs) = 23af3f1b918d92103a92c895ef7a1c4aa40f739c17973cfbad09771281d17398 (historical)
- **State: still BLOCKED[MUTATION-COURT-DIRTY-TREE]** (`git status --porcelain
  -- src/` non-empty; src/abi.rs, src/registry.rs, src/dialect.rs all
  modified). Post-commit expectation: the --list count must still be **319**
  (abi.rs 203 + registry.rs 116); any post-commit drift in that split invalidates
  M1's resumption path (`--case` resumes against a 319-mutant universe) and
  requires a fresh --log before the sweep:
  `CARGO_TARGET_DIR=target-mutants bash scripts/mutation_court.sh --log /tmp/mutation_court.log`
