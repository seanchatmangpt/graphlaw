# PARKED → RESUMED — Moonshot array v2

**RESUMED 2026-10-26** (goal-hook completion run, task w37vac3n2): the array is
executing again in the background toward the DoD. The park state below remains
accurate for any future re-park; the resume command is idempotent.

Typed state: `PARKED(operator-pivot)`. Nothing is lost; resume is one command.

## Resume command (exact)

```
Workflow({
  scriptPath: "/Users/sac/.claude/projects/-Users-sac-graphlaw/56e0a240-5f0e-4280-a223-1cbeba96c05c/workflows/scripts/moonshot-array-v2.js",
  resumeFromRunId: "wf_fbd828d9-fad"
})
```

The journal replays every completed agent free (results embedded per key); only
in-flight/dead lanes re-run — and implementation lanes checkpoint to
`/tmp/lane-{id}.state` for partial-progress resume.

## State at park

- **Workflow**: `moonshot-array-100` v2 (pipelined), runId `wf_fbd828d9-fad`,
  script on disk (operator-saved) + at the session path above.
- **Journal**: `~/.claude/projects/-Users-sac-graphlaw/56e0a240-5f0e-4280-a223-1cbeba96c05c/subagents/workflows/wf_fbd828d9-fad/journal.jsonl`
  (keys are pure functions of workflow+args+prior results — resume-safe).
- **Phase at park**: W0-Seal lanes re-running under the v2 structure; Array-A
  (11 WPs) queued. Checkpoints present: `/tmp/lane-P01-proof.state`,
  `/tmp/lane-P07-shacl.state`, `/tmp/lane-P08-sieve.state`,
  `/tmp/lane-P19-fuzz.state` (+ any written after).
- **All 20 work-package designs** are embedded in the v2 script
  (mission + acceptance per lane) — the script IS the complete execution spec.
- **Plan** (approved): `/Users/sac/.claude/plans/the-point-is-to-abundant-willow.md`
  (Phase 1 seal + Phase 2 array + dispatch protocol).

## v26.10.26 DoD state at park

- D1 engine findings: fixed upstream + re-vendored (72/72 parity) — earlier wave.
- D2: proof court 5/0, quantifier court 10/0, corpus court green, vendored parity
  receipted; OPEN row = corpus-under-graphlaw-gates witnessed (court exists).
- D3: closed 4/4 (limit identities, registry 200k pin, docs, wasm row re-pin).
- D4: OPEN rows at park = the 3 seal fixes (abi_manifest pin, dualroute vector
  re-pin, run_eh env receipt) + full-suite 0-fail rerun + bench-solo row CLOSED
  (PASS-AS-MEASURED) + mutation sweep CLOSED(TIME-BOXED-PARTIAL 113/204).
- D5: closed 7/7 (version 26.10.26, receipts, changelog, index).

## Standalone recovery notes

- `target/wasm32-wasip1/` may be absent (a prior lane deleted it) — the wasm
  re-mint lane rebuilds it (env: `WASI_SDK_PATH=/Users/sac/wasi-sdks/wasi-sdk-34`,
  `CARGO_TARGET_WASM32_WASIP1_RUSTFLAGS="-C target-feature=+exception-handling"`).
- `/tmp` checkpoint files are ephemeral across reboots — their loss costs only
  partial-progress resume (the journal + tree remain the truth).
- The 6 plan-wave stalls ({P07, P12, P14, P17, P19, P20}) need no recovery: their
  designs are in the v2 script lanes (P07/P19) or absorbed (P20→P06.4, P12→INT,
  P14→W2 lane already dispatched, P17→W3 lane).
