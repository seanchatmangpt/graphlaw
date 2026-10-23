# C23 Harness Pruner — Design (L34)

Status: DESIGN (no code in this lane)
Inputs: `~/.claude/rules/*.md` (18 files), `/Users/sac/graphlaw/CLAUDE.md`,
`/Users/sac/graphlaw/.claude/hooks/*.py`, `/Users/sac/graphlaw/.claude/settings.json`.
Operator catalog row: C23 = C (courts) + K (event store) + S (doctrine substrate);
falsifier: "a rule with zero witnessed firings survives a pruner pass, or the pruner
retires an entry that has a witnessed firing in its event log."

## 0. Measured baseline (the problem, witnessed)

### 0.1 Always-loaded prompt stack

18 global rule files at `~/.claude/rules/` (580 lines), plus project CLAUDE.md
stack: `~/.claude/CLAUDE.md` (37 lines), `/Users/sac/CLAUDE.md` (20),
`/Users/sac/graphlaw/CLAUDE.md` (112). Total 749 lines of doctrine injected into
every session, every subagent prompt, every lane. Per-session cost is roughly
749 lines x ~12 tokens/line ~ 9k tokens of standing context; a 20-lane wave pays
it ~20x, and each lane's system prompt pushes real task tokens toward the
context ceiling. The stack is append-only in practice: every incident becomes a
new dated paragraph, never a deletion.

### 0.2 Dated amendments stacked inside single files (monotonic growth)

Witnessed accretion (read from disk, 2026-10-22):

- `same-checkout-fanout.md` — 4 strata: base 2026-09-24; cleanup law
  2026-10-01 (~L21-24); W876 additions 1-3, 2026-10-07 (~L26-35); items 4-6,
  2026-10-16 (~L37-48)
- `testing-chicago-style.md` — 2 strata: base + SDK-parity register
  2026-10-16 (~L13-30)
- `tools.md` — 2 strata: base + Packet-N stdio harnesses (WASM-1,
  2026-10-21, ~L19-21)
- `no-sleep-testing.md` — 66 lines: laws + fences + enforcement + evidence
  (mtime 2026-10-18)
- `operating-doctrine.md` — 61 lines: base + task-normalize + verify-ladder
  strata

Pattern: correction layers are appended as prose, never folded into the base law,
never retracted when superseded. Same shape the catalog names as "generation rate
exceeds execution rate" (`~/.claude/rules/entropic-annealing-doctrine.md` §1).

### 0.3 Mechanism-vs-prose ledger (the actual audit)

Laws that gained a mechanical hook this session flip from PROSE to MECHANISM;
laws with zero witnessed firings and no hook are pruner candidates.

- **P1 MECHANISM** — topology is transport (`topology_guard.py:2-18`).
  Fires at `topology_guard.py:149-175,178-208`: worktree add, stash (L153),
  clone-into-protected (L154-157), repo-copy (L158-162), git-write-in-shadow
  (L172-175), EnterWorktree (L188), Agent isolation (L190), Workflow (L191-192),
  writes (L193-208). Corpus `topology_guard_cases.json`; registered
  `settings.json:13-20`.
- **P2 MECHANISM (new)** — coordinator-owned root manifest (witnessed 3x;
  `manifest_guard.py:1-14`). Denies file-tool edits at
  `manifest_guard.py:19,28-48`; registered `settings.json:21-30`.
- **P3 MECHANISM (new)** — disk headroom before suite/bench (ENOSPC killed
  waves 5x; `disk_guard.py:1-13`). HEAVY regex gate, MIN_FREE_GIB=20 at
  `disk_guard.py:20-26,35-57`; registered `settings.json:31-40`.
- **P4 MECHANISM** — never dispatch generic agents; `ban_generic_agents.py`
  (41 lines); registered `settings.json:4-12`.
- **P5 PROSE** — no fixed sleeps (`no-sleep-testing.md:1-15`). No sleep-gate
  hook in `.claude/hooks/`; smalt's `just sleep-gate` has no counterpart here.
- **P6 PROSE+partial** — Chicago/mocks banned (`testing-chicago-style.md:1-12`).
  `scripts/mutation_court.sh` covers gates, not test-source mock scans.
- **P7 MECHANISM exists** — gate that cannot exit 1 is the failure;
  `scripts/mutation_court.sh`.
- **P8 PROSE** — receipts embed real output; no receipt-schema validator
  hook in `.claude/hooks/`.
- **P9 PROSE** — commit `-F <file>`, never `-m` (`tools.md:9-11`); no hook
  intercepts `git commit -m`.
- **P10 PROSE** — lane leases / CARGO_TARGET_DIR isolation
  (`same-checkout-fanout.md:11-12`); coordinator discipline only.
- **P11 PROSE** — compile-freeze SLA (~10 min fix-or-revert)
  (`same-checkout-fanout.md:31-33`); timer-based, unmechanized.
- **P12 PROSE** — counts re-read at use time
  (`no-overclaiming-conversational.md:8-10`); not hook-enforceable.
- **P13 PROSE stub candidate** — skill disambiguation
  (`skill-disambiguation.md:1-11`); zero witnessed firings this session.
- **P14 PROSE stub candidate** — DMEDI framing (`dmedi-methodology.md:1-12`);
  zero witnessed firings; cited by no hook/court.

Ledger counts: 14 surveyed, 4 MECHANISM (P1-P4), 2 PROSE-with-adjacent-court
(P6-P7), 8 PROSE-only (P5, P8-P14). Pruner scope = the PROSE-only set plus
in-file amendment strata that a hook now covers (e.g. the stash-sweep prose at
`same-checkout-fanout.md` rule 1 is duplicated by `topology_guard.py:51-53,153`
— the prose can shrink to one pointer line once the hook is trusted).

## 1. Witnessed-firing event log (schema + where firings land)

### 1.1 Sink

One append-only NDJSON log per scope, written by every hook at the moment it
emits (or declines to emit) a decision:

- repo scope: `/Users/sac/graphlaw/.claude/hooks/firings.ndjson`
- global scope: `~/.claude/firings.ndjson`

Hooks already parse the full PreToolUse payload (`manifest_guard.py:24`,
`disk_guard.py:31`, `topology_guard.py:180`), so logging is one `append()` call
in each `main()`; no new transport. Rotation: size-capped at 10 MiB, rotated to
`firings.ndjson.1` (one generation). The log is the K (event store) half of C23;
BLAKE3-chained daily digests are out of scope for v1 — NDJSON with UTC
timestamps is sufficient to answer "zero firings in N days".

### 1.2 Record schema

```json
{
  "ts": "2026-10-22T16:27:03Z",
  "source": "manifest_guard.py",          // hook file or "rule:<file>.md"
  "subject": "Cargo.toml",                // target: file path or Bash verb class
  "decision": "deny",                     // deny | allow | error-fail-open
  "reason_code": "MANIFEST-REFUSED",      // typed code, matches docs/refusals.md style
  "sha": "a1b2c3d4e5",                    // blake3 of the rule text or hook script, 10 hex
  "session": "t-..."                      // harness session id if present in payload
}
```

Decisions are logged on BOTH branches (deny and allow-through), because an
allow is a witnessed firing too — the hook ran and evaluated. A rule/hook that
only ever logs allows on paths its regex cannot reach is exactly the vacuity the
non-vacuity court (section 3) must catch. Fail-open paths (`manifest_guard.py:26`,
`disk_guard.py:33`) log `decision: "error-fail-open"` so silent hook breakage
shows up in the ledger as a firing class of its own.

### 1.3 What counts as a "witnessed firing" for a prose rule

Prose rules have no hook to log for them until they are mechanized. Two admission
paths:

1. **Mechanized rule**: firings come from its hook's log lines (`source` field).
2. **Unmechanized prose rule**: a firing is a *cited correction* — a lane report
   or wave receipt that names the rule id as the reason a defect was caught.
   These are appended by the coordinator at integration as
   `{"source": "rule:<file>.md#<heading>", "decision": "cite", ...}`. A prose
   rule with zero `cite` records in the window is the pruner's target class.

## 2. Pruner pass algorithm

One pass = one scripted run (`scripts/rules_pruner.py`, global scope reads
`~/.claude/firings.ndjson` + all repo `firings.ndjson` under a configured root
list). It proposes; it never edits in place — output is a typed retirement plan.

```
for each entry in registry(rules/*.md headings, hooks/*.py, settings.json hooks):
    f = firings(entry, window=N days)
    classify(entry):
        MECHANISM   -> hook in settings.json
        PROSE-CITED -> >=1 cite record in window
        STUB        -> 0 firings AND 0 cites in window

for entry in STUB:
    if entry is the anti-vacuity or topology core (protected list, section 3.3):
        SKIP-PROTECTED (logged, loud)
    elif age(entry) < N days:
        SKIP-YOUNG (logged)
    else:
        RETIRE-PROPOSED(entry, evidence=zero-firing receipt)
```

Laws encoded:

- **Refuse-to-generate ordering (ERRC, `entropic-annealing-doctrine.md` §2)**:
  the pass runs reduce-first — retire/deletion proposals are emitted before any
  consolidation proposal; the pruner has no "add rule" mode at all.
- **Fold amendments, don't append**: for files with >2 dated strata
  (section 0.2), a `FOLD-PROPOSED` rewrites the file to base law + pointer to the
  mechanizing hook, deleting superseded prose. Deletion is permanent entropy
  removal; a fold that only reflows text is refused by the court (section 3).
- **Retirement is receipt-gated, never in-pass**: a proposed retirement lands
  only after the non-vacuity court (section 3) proves the pruner flips on that
  specific entry. Status flips use the standing vocabulary — an entry moves
  PROSE -> STUB -> RETIRE-PROPOSED -> RETIRED, each transition carrying its
  receipt; no silent deletions.
- **Typed refusals** (`docs/refusals.md` convention): `PRUNER-REFUSED(WITNESSED)`
  when a candidate has firings; `PRUNER-REFUSED(PROTECTED)` for core entries;
  `PRUNER-VACUOUS` when the court cannot flip.

Default window: N = 21 days (three weekly waves; the catalog's saturation law
runs ~20-lane waves, so 21 days is ~3 full chances to fire).

## 3. Non-vacuity court for the pruner itself

A pruner that passes while it would retire a live rule (or keep a dead one) is
the exact `admission_vacuous` failure class. The court is a shell/py script,
`scripts/rules_pruner_court.sh`, modeled on `mutation_court.sh` and the smalt
shim-court pattern (baseline -> mutate -> assert-flip -> restore, fail-closed on
restore mismatch):

1. **Survival case (must NOT retire)**: fixture log containing a firing for rule
   R inside the window -> run pruner on a temp copy of the registry -> assert R
   is classified PROSE-CITED/MECHANISM and `RETIRED` count for R is 0, plus a
   `PRUNER-REFUSED(WITNESSED)` line. If R is retired => court FAIL.
2. **Refusal case (must retire)**: fixture registry with a zero-firing stub S
   older than N -> assert S is `RETIRE-PROPOSED` with a zero-firing receipt
   attached. If S survives => court FAIL (this is the catalog falsifier, run
   literally).
3. **Deletion proof**: assert every `FOLD-PROPOSED` diff has net line delta <= 0.
   A "fold" that grows the file is refused (`PRUNER-REFUSED(NET-GROWTH)`).
4. **Restore guard**: sha256 the fixture registry before/after; mismatch exits 1.

Non-vacuity of the court itself: a mutation that flips the pruner's window
constant to 0 days (making everything a stub) must make case 1 FAIL — if the
court still passes with an inverted pruner, exit `PRUNER-COURT-VACUOUS`.

### 3.3 Protected list (never pruned regardless of firings)

`topology_guard.py` and its corpus (`topology_guard_cases.json`), the
anti-vacuity/mutation-court doctrine lines, the receipt-law lines, and the
coordinator/git-ownership lines. Rationale: these are load-bearing for lane
safety with low firing frequency by construction (they refuse rare, destructive
acts); zero firings in 21 days is expected, not evidence of dead weight. The
protected list is data (a checked-in file), reviewed at milestone boundaries —
it can change, but not by a pruner pass.

## 4. OOD-split admission per C22

A pruner edit (window constant, classifier change, protected-list change,
retirement heuristics) is a harness self-edit, so it inherits the C22 court:

- **In-family run**: the court of section 3 on the graphlaw fixture registry
  must pass. Necessary, not sufficient.
- **Out-of-family run (required)**: replay the same pruner + court against a
  registry the edit was not written against — concretely, the smalt-shaped
  fixture (sleep-gate allowlist entries + KD-nn divergence rows) checked into
  `tests/fixtures/rules_pruner/ood_smalt_registry/`. Admission predicate: the
  edit passes >=1 out-of-family subject. A window/classifier change that passes
  at home and misclassifies the smalt fixture (retires a row with witnessed
  shim-court firings, or keeps a genuine stub) is refused with
  `PRUNER-REFUSED(OOD-FAIL)`.
- The admission receipt names both subjects with their SHAs; a single-subject
  receipt is not an admission (`DFCM-26923-01` falsifier, cited verbatim in the
  court's docstring).

## 5. Staged rollout

- **S0** — add firing-log append to the 4 existing hooks (`topology_guard.py`,
  `manifest_guard.py`, `disk_guard.py`, `ban_generic_agents.py`); log-only, no
  behavior change. Gate: one wave of logs accumulates; existing hook behavior
  unchanged.
- **S1** — registry check-in: machine-readable inventory of rules/headings with
  ids, ages, protected flags. Gate: registry parses; every rule file maps to
  >=1 id.
- **S2** — pruner script + court, run manually, proposals only. Gate: court
  passes; one dry-run produces a typed plan against real logs.
- **S3** — C22 OOD fixture + admission predicate wired into the court. Gate:
  OOD replay receipt exists.
- **S4** — first real retirement pass: P12/P13/P14 from the section 0.3 ledger
  (P12 pending cite-evidence) + fold of `same-checkout-fanout.md` stash prose
  into a topology_guard pointer. Gate: every retirement carries a zero-firing
  receipt; court re-run post-retirement still passes (survival case intact).
- **S5** — standing: pruner runs at each milestone boundary; results appended
  to `docs/jira/<v>/INDEX.md`. Gate: two consecutive waves with zero retirement
  candidates = saturation for this surface.

Rollback: retirements are file deletions in a git-owned tree; revert restores
without state migration. The firing log is append-only and never rolled back.

## Receipts

None yet — this is a design lane. Commands executed while grounding this doc
(real output, 2026-10-22):

```
$ wc -l ~/.claude/rules/*.md | tail -1
     580 total
$ wc -l ~/.claude/CLAUDE.md /Users/sac/CLAUDE.md /Users/sac/graphlaw/CLAUDE.md
      37 /Users/sac/.claude/CLAUDE.md
      20 /Users/sac/CLAUDE.md
     112 /Users/sac/graphlaw/CLAUDE.md
     169 total
$ ls /Users/sac/graphlaw/.claude/hooks/*.py | xargs -n1 basename
ban_generic_agents.py
disk_guard.py
manifest_guard.py
topology_guard.py
```

## See Also

`docs/jira/v26.10.22/designs/spof-scan.md` (adjacent SPOF lane) ·
`~/.claude/rules/entropic-annealing-doctrine.md` §2 (ERRC ordering) ·
operator catalog C22 (OOD-split) / C23 (this design)
