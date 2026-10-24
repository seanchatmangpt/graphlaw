# Design: Pre-Dispatch Drift Gate

- **Id**: DESIGN-PDDG-01
- **Status**: PROPOSED
- **Class**: ERRC-Reduce (~zero new machinery: one PreToolUse hook reusing the existing attest-hook pattern)
- **Repo**: /Users/sac/graphlaw
- **Scope**: `.claude/hooks/` (new hook), `.claude/settings.json` (one PreToolUse registration). No `src/` changes, no manifest changes.

## 1. Context

Dispatching agent lanes onto a drifted shared tree manufactures entropy at
generation rate: every dispatched lane inherits the drift, compiles against it,
and reports findings against a subject that is not the admitted tree.

Measured thrash:

- **Broken-tree freeze class (E0271/E0369/E0063)**: lanes dispatched onto a
  tree whose `src/` did not compile froze the shared compile; every
  concurrent lane's build inherited the break and all lanes stalled until the
  owner fixed or reverted. One broken edit cost a full wave's latency.
- **Mid-mutation-court dispatch (witnessed)**: a lane was dispatched at
  13:04 while `scripts/mutation_court.sh` had `src/abi.rs:48` inverted in
  place. Mutation court mutates the tree solo (see CLAUDE.md); a dispatch
  during its window admits a subject that is by construction a mutation.
- **Z22 post-kill freeze**: after a kill/prune lane terminated, its in-flight
  `src/` state was still on disk; a subsequent dispatch admitted the
  half-removed surface and froze on the residue.

Common cause: dispatch is currently admitted on agent assertion, not on
observed tree state. The dispatch hop is the one remaining unreceipted edge
in the fan-out loop.

## 2. Mechanism

A new PreToolUse hook, `.claude/hooks/pre_dispatch_drift_gate.py`, registered
in `.claude/settings.json` against the Agent dispatch tool (same wiring
pattern as `.claude/hooks/ban_generic_agents.py` and
`.claude/hooks/graphlaw_receipt_attest.py`; infra exists, no new runtime).

On every Agent-tool dispatch the hook runs three checks on the shared tree
(`/Users/sac/graphlaw`, resolved from the hook's own location, never cwd):

1. **Green compile**: `cargo check` on the shared tree exits 0.
   Incremental: a fingerprint of `(mtimes of src/**, Cargo.toml, last
   green-check digest)` is cached at `.claude/hooks/.pddg_check_cache`;
   the check is skipped only when no input changed since the last green run.
2. **Clean `src/`**: `git status --porcelain -- src/` is empty. Untracked or
   modified `src/` files mean a lane's in-flight edit is live.
3. **No mutation markers**: `grep -rl "changed by cargo-mutants" src/`
   returns no matches (covers the mid-mutation-court window even when the
   mutated file mtime predates the cache).

**Refusal (typed, fail-closed)**: any failed/skipped check refuses the
dispatch with a stable code and the failing evidence:

```
PDDG-REFUSED(COMPILE) | PDDG-REFUSED(DIRTY_SRC) | PDDG-REFUSED(MUTATION_MARKER)
```

with payload: check name, first failing path(s), and the one-line remediation
("fix or revert the shared src/ before dispatch; see CLAUDE.md compile-freeze SLA").

Fail-closed semantics: hook crash, timeout (>30 s budget, see §4), unreadable
git state, or unknown tool name all refuse. A gate that cannot exit 1 is the
failure; there is no silent-pass arm.

## 3. Exemption path

An operator who genuinely needs to dispatch onto a drifted tree (e.g. a
repair lane dispatched to fix the very break) passes `--force-dispatch`
in the dispatch prompt with an explicit reason string. The hook admits the
dispatch but writes a receipt to `receipts/pddg-forced-<UTC timestamp>.json`
carrying: tool input digest, reason, the failed check name, and `git
rev-parse HEAD`. The exemption is receipted, never silent; forced dispatches
are auditable and count toward wave-lead indicators.

## 4. Performance budget

The gate adds latency to every dispatch; budget is **<30 s end to end**:

- Cache hit (no `src/`/manifest change since last green check): <1 s
  (fingerprint stat + git status + grep).
- Cache miss: one `cargo check` on a warm `target/` — measured ~8-20 s on
  this tree. If it exceeds 30 s the hook refuses `PDDG-REFUSED(TIMEOUT)`
  rather than admitting unobserved; the operator runs `cargo check`
  manually (warming the cache) and retries.
- The gate never runs a cold full build in the dispatch path.

## 5. Anti-vacuity self-test

A gate that never refuses is vacuous. Self-test `bash
scripts/pddg_selftest.sh` (solo — it touches the tree, like mutation court):

1. Baseline: dispatch-shaped hook invocation on a green tree → admit.
2. Plant: append `fn _pddg_vacuity() { let x: u8 = "s"; }` to a `src/`
   file → dispatch → must refuse `PDDG-REFUSED(COMPILE)`.
3. Plant: dirty an untouched `src/` file (touch + one-char change) → must
   refuse `PDDG-REFUSED(DIRTY_SRC)`.
4. Plant: insert a `// changed by cargo-mutants` comment → must refuse
   `PDDG-REFUSED(MUTATION_MARKER)`.
5. Restore: revert all plants, verify `git status --porcelain -- src/`
   byte-identical to baseline (sha256), fail-closed on restore mismatch.

A self-test where any planted dispatch is admitted is FAIL. The exemption
court (step 6) verifies `--force-dispatch` writes the receipt file; a forced
dispatch with no receipt on disk is FAIL.

## 6. Acceptance

- `bash scripts/pddg_selftest.sh` exits 0 with all five courts PASS and the
  restore hash embedded in output.
- `cargo check` remains green after hook installation (hook adds no Rust).
- `.claude/settings.json` diff is exactly one PreToolUse registration.
- Receipts land under `receipts/pddg-*` only on the forced path.

## See Also

- `docs/jira/v26.10.22/` — freeze-class witnesses
- `.claude/hooks/graphlaw_receipt_attest.py` — hook wiring precedent
- `CLAUDE.md` — mutation-court-solo and compile-freeze SLA laws
