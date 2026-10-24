# Mutation court runbook (v26.10.22 terminal gate)

Prepared from scripts/mutation_court.sh (source lines cited in-wave).

## Preconditions
1. All src/ work committed: `git status --porcelain -- src/` empty (court refuses exit 6 otherwise; `--force` is forbidden in shared windows).
2. No leaked mutants: `grep -rn 'changed by cargo-mutants' --include='*.rs' src/` empty (exit 5 otherwise).
3. cargo-mutants on PATH; df -h / ≥ 20 GiB.
4. No lane holds a src/ lease; no shared target dir (watchdog kills by argv0 under $CARGO_TARGET_DIR/debug/deps/).

## Command (gate form)
    cd /Users/sac/graphlaw
    bash scripts/mutation_court.sh --check --log /tmp/mutation_court_$(date +%H%M%S).log
    # read the head-of-pipe exit; never a post-pipe $?

Fast pre-flight: `--list` (no tests) and `--leg callsite-revert` (minutes).

## Pass criteria
- callsite-revert leg: law_state court must FAIL under the shim (vacuous pass = exit 7); restore byte-exact.
- Gate grep: zero MISSED mutants in src/registry.rs or src/abi.rs capabilities()/dispatch() (exit 3 = MISSED GATED MUTANT).
- Exit 0 pass; 2 = missed outside gate; 4 = restore/git-status drift (untrustworthy tree — repair from the printed snapshot dir).

## Receipt requirements
stderr must contain `mutation_court: tree restored byte-exact (sha256 verified, git status unchanged)`; embed the real exit code. A summary without them is not a receipt.

## Post-run restore verification
1. git status --porcelain -- src/ still empty.
2. grep -rn 'SHIM-COURT\|changed by cargo-mutants' src/ → nothing.
