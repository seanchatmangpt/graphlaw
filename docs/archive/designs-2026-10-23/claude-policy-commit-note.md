# .claude/ Commit Policy — GO/NO-GO Operator Decision 3

**Status**: RECOMMENDATION (operator decides; coordinator owns git)
**Subject**: `/Users/sac/graphlaw/.claude/` tree, as of 2026-10-22
**Scope**: this note recommends only; it makes no commits and no `.claude/` edits.

## 1. Inventory

Receipt: `ls -la .claude/ .claude/hooks/ .claude/agents/`,
`git status --porcelain -- .claude/ .gitignore`, `git ls-files .claude/`,
2026-10-22.

| Path | On disk | Tracked | Status |
|---|---|---|---|
| `.claude/settings.json` | yes | no | untracked (`??`) |
| `.claude/hooks/ban_generic_agents.py` | yes | yes | clean |
| `.claude/hooks/topology_guard.py` | yes | yes | clean |
| `.claude/hooks/topology_guard_cases.json` | yes | yes | clean |
| `.claude/hooks/manifest_guard.py` | yes | no | untracked (`??`) |
| `.claude/hooks/disk_guard.py` | yes | no | untracked (`??`) |
| `.claude/hooks/__pycache__/` | yes | no | untracked (`??`) |
| `.claude/agents/attest-keeper.md` | yes | yes | clean |
| `.claude/agents/court-runner.md` | yes | yes | modified (`M`) |
| `.claude/agents/milestone-doc.md` | yes | yes | clean |
| `.claude/agents/ontology-author.md` | yes | yes | clean |
| `.claude/agents/sql-compiler.md` | yes | yes | modified (`M`) |
| `.claude/agents/wasm-op.md` | yes | yes | modified (`M`) |
| `.claude/scheduled_tasks.lock` | yes | no | untracked, already gitignored |

Verbatim `git status --porcelain -- .claude/`:

```
 M .claude/agents/court-runner.md
 M .claude/agents/sql-compiler.md
 M .claude/agents/wasm-op.md
?? .claude/hooks/__pycache__/
?? .claude/hooks/disk_guard.py
?? .claude/hooks/manifest_guard.py
?? .claude/settings.json
```

`.gitignore` already carries `.claude/scheduled_tasks.lock` (verified on disk,
under `# session-local runtime state`). `.gitignore` does not carry
`__pycache__` — it is currently an untracked-not-ignored path that will
pollute every future `git status`.

## 2. Per-artifact policy analysis

The deciding question: does a contributor cloning the repo get the same
guardrails? The hooks are wired in `settings.json` and fire for any Claude
session in this checkout — they are repo law, not operator preference.

### Commit — repo law

| Artifact | What it IS | Repo-policy value |
|---|---|---|
| `settings.json` | The wiring: 4 PreToolUse matchers (Agent → ban_generic_agents, EnterWorktree → topology_guard, Edit/Write/MultiEdit/NotebookEdit → manifest_guard, Bash → disk_guard), all via `$CLAUDE_PROJECT_DIR`. Without it the hook .py files are inert scripts. | A fresh clone gets zero guardrails without it; CLAUDE.md's agent rules stay prose. Commit. |
| `ban_generic_agents.py` | Enforces the CLAUDE.md law "never dispatch generic agents" mechanically (exit 2, `GENERIC_AGENT_REFUSED`, names the allowed set). | Turns a prose law into a typed refusal. Commit (keep tracked). |
| `topology_guard.py` + `topology_guard_cases.json` | Refuses worktree creation, clones into `~/wt`, EnterWorktree, isolation=worktree — the one-canonical-checkout law. 19-case corpus on disk. | Already tracked; keeps the fan-out law mechanical for contributors. Keep tracked. |
| `manifest_guard.py` | Root `Cargo.toml`/`deny.toml` are coordinator-owned. Concurrent lane manifest edits reverted each other 3x in one session (hook docstring: "witnessed 3x, v26.10.22"). Deliberate escape hatch: coordinator applies manifest transitions via Bash, which the hook does not intercept. | Prevents the concurrent-lane manifest-contention class mechanically. Commit. |
| `disk_guard.py` | Refuses suite/bench-class cargo work while `/` has < 20 GiB free; cheap ladder rungs (`cargo test --test <t>`, `cargo check`) pass through. ENOSPC has killed all-lane waves. | Prevents ENOSPC wave kills mechanically. Commit, after hardening (§4). |
| `agents/*.md` (all 6) | The dispatch surface the ban names. Committing them is what makes the generic-agent ban meaningful for a contributor. | Commit, including the 3 modified (court-runner, sql-compiler, wasm-op — lane edits of already-tracked files). |

### Gitignore — session/derived state, not repo law

| Artifact | What it IS | Policy |
|---|---|---|
| `hooks/__pycache__/` | CPython bytecode cache derived from committed .py sources — regenerable, machine-specific, status noise. | Ignore. |
| `scheduled_tasks.lock` | Session scheduling state, per-checkout. | Already ignored; no action. |

## 3. Recommendation

1. Commit `settings.json`, `manifest_guard.py`, `disk_guard.py`, and all 6
   `agents/*.md` (including the 3 modified). Keep
   `ban_generic_agents.py`, `topology_guard.py`, `topology_guard_cases.json`
   tracked.
2. Add to `.gitignore`:

```
# Python bytecode cache from .claude/hooks/*.py
__pycache__/
```

   A bare `__pycache__/` pattern ignores it at any depth; a depth-scoped
   alternative is `.claude/hooks/__pycache__/`.
3. Do not commit `scheduled_tasks.lock`; its existing ignore entry stays.

## 4. Hardening before these become repo law

1. `disk_guard.py` `MIN_FREE_GIB = 20` is a hardcoded constant with no
   in-file provenance. As repo law it should carry a comment citing the
   ENOSPC receipts (`designs/mutation-court-full-receipt.md:73-76`,
   `designs/wasm4pm-fixtures-drift.md:48`) and the ~180 GiB lane-root purge,
   and optionally read an env override (`GRAPHLAW_MIN_FREE_GIB`) so hosts
   with different disk sizes tune without forking the hook.
2. Fail-open defaults audited, not changed. All guard hooks exit 0 on
   unreadable stdin JSON (ban_generic_agents.py:23, manifest_guard.py:26,
   disk_guard.py:32-33). Fail-open on malformed input is correct — a broken
   guard must not brick all tool use — but only ban_generic and manifest_guard
   say so in a comment. State it as deliberate policy in each docstring so a
   contributor does not "fix" it to fail-closed. Docstring-only change.
3. `manifest_guard.py` basename matching (line 34) is coarse: it also matches
   `target*/.../Cargo.toml` build outputs. Document this as deliberate
   (build outputs are regenerable; the false-positive cost is a
   scripted-edit escalation, not data loss). No behavior change.
4. `disk_guard.py` HEAVY regex already carves out `cargo test --test <t>`
   (line 21, `test(?!\s+--test\b)`) — matches the repo test ladder. Designed
   behavior; cite, no change.

## 5. Witnessed firings justifying each hook

On-disk receipts only; lane-reported in-session firings are marked as such.

| Hook | Witness | Receipt |
|---|---|---|
| `disk_guard.py` | DISK-REFUSED(19.1 GiB free < 20 GiB) refused a mutation-court run; stale lane build roots purged (~180 GiB freed) | `docs/jira/v26.10.22/designs/mutation-court-full-receipt.md:73-76` |
| `disk_guard.py` | DISK-REFUSED(11.3 GiB free < 20 GiB) refused a suite-class run in the wasm4pm fixtures lane | `docs/jira/v26.10.22/designs/wasm4pm-fixtures-drift.md:48` |
| `manifest_guard.py` | 3 concurrent-lane root-manifest contentions in one session, each reverting the other's state (optional flag, features, license line) | In-session; corroborated by the hook docstring ("witnessed 3x, v26.10.22") and `docs/refusals.md:97` (row marked DESIGN-STAGE at reconciliation time — the hook has since landed on disk) |
| `ban_generic_agents.py` | Refused a generic-agent dispatch in lane L7 | Lane-reported, in-session; no on-disk receipt. The ban's authority is CLAUDE.md's "Never dispatch the built-in generic agents"; the hook is its mechanical form. |
| `topology_guard.py` | PreToolUse refusal of worktree creation, witnessed firing 2026-09-24; 19-case corpus | `topology_guard_cases.json` on disk; composition catalog primitive T (standing ALIVE) |

## 6. Falsifier

This recommendation is refuted if committing `settings.json` changes the
operator's own session behavior in a way the operator rejects — it should
not: the hooks fire from the on-disk file today regardless of tracking, so
committing changes visibility (hook diffs become reviewed changes in
`git status`), not behavior. Contributor-clone risk is bounded: if python3
is absent the hooks fail to spawn and the session proceeds without
guardrails, i.e. exactly the pre-commit state of a fresh clone, not a
regression.

## 7. Summary

Commit the guards, the wiring, and the agent definitions — they are repo law,
and a fresh clone should be law-bound identically. Ignore `__pycache__/`;
leave `scheduled_tasks.lock` ignored. Four hardening items (one constant +
env override, fail-open documentation, manifest basename note, ladder
carve-out citation) precede the commit. Each hook is justified by a witnessed
firing: two on-disk DISK-REFUSED receipts, one 3x manifest-contention witness
corroborated by the refusals register, one lane-reported ban firing (L7), and
the topology guard's 2026-09-24 witnessed firing with its 19-case corpus.
