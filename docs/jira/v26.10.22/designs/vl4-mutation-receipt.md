# V-L4 Mutation Court Receipt — 2026-10-22

- **Lane**: V-L4 (/Users/sac/graphlaw, canonical checkout)
- **Command**: `CARGO_TARGET_DIR=target-lane-vl4 bash scripts/mutation_court.sh --log /tmp/vl4_mutation.log`
- **Exit**: 6
- **Verdict**: TYPED-REFUSAL (not run) — `MUTATION-COURT-DIRTY-TREE`
- **Mutants tested/caught**: 0 / 0 — sweep refused before mutant testing.

## Embedded real output (abridged; full stderr in task transcript)

```
mutation_court: REFUSED [MUTATION-COURT-DIRTY-TREE]: uncommitted work under src/;
  cargo-mutants runs --in-place and would mutate those edits (false-red channel).
  Dirty files:
     M src/abi.rs
     M src/abi_manifest.rs
     M src/attest.rs
     M src/law.rs
     M src/sql_ast.rs
     ... (25 modified files under src/) ...
     ?? src/abi/
     ?? src/dialect/
     ?? src/query_ledger/
     ?? src/receipt_chain/
     ?? src/registry/
     ?? src/sql_ast/
  Commit, stash-elsewhere, or pass --force to run anyway.
EXIT=6
```

(25 modified files + 6 untracked dirs under src/; full list in refusal output.
Log file /tmp/vl4_mutation.log was NOT created — refusal precedes logging.)

## Standing

BLOCKED for this lane pending tree switchover commit. A loud typed refusal is
the correct behavior for an in-place mutator over a dirty src/ tree; not a pass.
