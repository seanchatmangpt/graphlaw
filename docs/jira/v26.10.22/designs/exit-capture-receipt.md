# M11 — Exit-capture law (court-runner) receipt

- Id: M11
- Title: Exit-capture law for pipe-fronted court runs (zsh has no PIPESTATUS)
- Status: DONE
- Conformance target: `.claude/agents/court-runner.md` Exit-capture section; `scripts/exit_capture_check.sh` self-test passes
- Defect class (witnessed 4x: W01, W04, AA4, W01-retry): lanes echo `EXIT=$?`
  after a pipe; zsh `$?` is the tail's exit, reporting false green when the
  head (cargo) failed.

## Current behavior (before)

`.claude/agents/court-runner.md` had no exit-capture law; lanes freely wrote
`cargo test | tail -20; EXIT=$?` and reported tail's exit.

## Required behavior

- Agent law: never trust `$?` after a pipe; use `set -o pipefail` +
  `${pipestatus[1]}` (zsh) or capture cargo's own result line.
- Runtime guard: `scripts/exit_capture_check.sh` runs `false | true` and
  asserts the wrapper detects exit 1 (anti-vacuity: proves the pattern works).

## Acceptance

`bash scripts/exit_capture_check.sh` — real output embedded below.

```
WITNESS: $? after 'false | true' is 0 (tail's exit) — the defect is real
LAWFUL: zsh pipestatus[1] for 'false | true' = 1 (zsh rc=0)
PASS: exit-capture pattern detects head exit 1
PASS: healthy 'true | true' reads 0 (no false positive)
exit_capture_check: PASS
script-exit=0
```

