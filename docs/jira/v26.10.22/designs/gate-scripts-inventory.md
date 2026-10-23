# Gate scripts inventory — scripts/*.sh (v26.10.22)

Filed 2026-10-23 by lane DOC. One entry per `scripts/*.sh` (18 scripts).
`gen_wasm4pm_graphlaw_vectors.py` is out of scope (.py helper). The dl
gates are cargo test courts, not scripts — noted at the end. Standing
re-read from `RECEIPTS.md:1169-1420` (waves W1-W58) plus this session.

Standing key: GREEN / TYPED-RED / SKIP / NRW (never-run-this-wave) /
VERIFIED.

## witness_gate.sh

- Purpose (M02): a build/run witness in `docs/**/*.md` must cite an
  existing receipts/ row or path. Scans blake3 digests (64-hex) and
  `target-lane*` citations. Skips fenced blocks and
  `witness-gate:allow` lines; excludes `docs/reference-generated/`.
- Invocation: `bash scripts/witness_gate.sh` (no args/env documented).
- Writes: none (read-only scan).
- Standing: NRW.

## machine_shaped_gate.sh

- Purpose (K1 side-arm, lane M09): fails closed on `/Users/` literals
  in tests/ or src/ unless (a) comment, or `MACHINE-PINNED: <reason>`
  on the same or preceding line.
- Invocation: `bash scripts/machine_shaped_gate.sh` (usage block; no
  env documented).
- Writes: none.
- Standing: NRW.

## chain_flag_court.sh

- Purpose (M14): non-vacuity court for receipt_chain_sync.sh — pins
  `--check` → exit 0 + VERIFIED and unknown flag → exit 2 + USAGE on
  the live tree.
- Invocation: `bash scripts/chain_flag_court.sh`.
- Writes: none.
- Standing: NRW.

## count_gate.sh

- Purpose: stale "N passed; 0 failed" detector. Scans docs/** for
  count rows tied to an allowlist court, re-runs each in a lane target
  dir, compares. COUNT-STALE / COUNT-UNPAIRED findings; fence-filtered
  (quoted output is SKIPPED-FENCED, never a claim).
- Invocation: `bash scripts/count_gate.sh` (v26.10.22 hardening:
  unpaired rows, broadened regex, fence filter).
- Writes: none durable (lane target dir for re-runs).
- Standing: NRW.

## exit_capture_check.sh

- Purpose (M11): self-test of the Exit-capture law in
  `.claude/agents/court-runner.md`. Runs `false | true` and proves the
  documented zsh pattern detects the real head exit; exits 1 if the
  pattern fails to detect it.
- Invocation: `bash scripts/exit_capture_check.sh`.
- Writes: none.
- Standing: NRW.

## fleet_cleanup.sh

- Purpose: advisory disk reclaim for lane build roots. Deletes
  repo-local `target-*` dirs (never ./target) older than a grace
  window; prints per-dir freed bytes and df before/after. Always
  exits 0.
- Invocation: `fleet_cleanup.sh [--dry-run] [--grace-minutes N]
  [--repo PATH]`; default grace 45 min.
- Writes: deletes stale target-* dirs only.
- Standing: NRW.

## partition_court.sh (+selfchecks)

- Purpose: Chicago court over the three-partition code census
  (`courts/code-partition/manifest.json`). Read-only over src/,
  crates/graphlaw-eyeron/src/, wasm/src/. Exit 1 violation, exit 2
  BLOCKED (inputs missing — never a silent pass).
- Invocation: `bash scripts/partition_court.sh`; anti-vacuity via
  `python3 scripts/partition_court_selfcheck_g{1,2}.py`.
- Writes: self-check scratch under /tmp only.
- Standing: NRW (+selfchecks).

## doc_staleness_gate.sh

- Purpose (L35): CI-level staleness court for prose. Exit 0 pass,
  1 findings, 2 internal error. Loud, never silent.
- Invocation: `doc_staleness_gate.sh [ROOT]` (default repo root;
  non-repo roots scan standalone for flip proofs).
- Writes: none.
- Standing: NRW.

## mutation_court.sh

- Purpose: native mutation court over the ABI registry surface
  (src/registry.rs; capabilities()/dispatch() of src/abi.rs). 319
  mutants (203 + 116; gated surface 124).
- Invocation: `mutation_court.sh [--list] [--check]
  [--leg callsite-revert] [--log F] [--case]`; isolated
  `CARGO_TARGET_DIR`; typed exit 6 on dirty src/ tree.
- Writes: patches src/ in place, EXIT-trap restore fail-closed; log
  via `--log`.
- Standing: TYPED-RED→BLOCKED (W14): blocked on clean tree/commit;
  EXCLUDED-HOT-PATH in commit-manifest.md:63. Open row, not a pass.

## shape_mutation_court.sh

- Purpose (TC-MUT-01): shape-flip anti-vacuity over
  `ontologies/sql-ast.ttl`. Snapshot sha256 → mutate in place → run
  sql-rdf courts → byte-exact restore via EXIT trap. Operator-enum
  mutations are E2 coverage by design (code-side backstop in
  sql_ast.rs), not expected to flip courts.
- Invocation: `bash scripts/shape_mutation_court.sh`.
- Writes: mutates the shape file in place; restores byte-exact.
- Standing: NRW.

## spc_court.sh

- Purpose: §5 statistical process control over the receipt hot path;
  Cpk ≥ 1.33. Run AFTER mutation_court.sh (test ladder). Anti-vacuity
  via injected outlier that must flip the verdict.
- Invocation: `scripts/spc_court.sh` (SMOKE, 1M iter); `SPC_MODE=FULL`
  (10M, TC_HOT_FULL=1); env `SPC_INJECT_OUTLIER=1`.
- Writes: appends JSON receipt (append-only, L111 fix).
- Standing: GREEN — PASS, `receipts/spc-2026-10-23T035422.json` (W57).

## wasm_audit.sh

- Purpose (W04): wasm artifact size ceiling + header court for the
  artifact matching the CURRENT wasm/Cargo.toml VERSION. Exit 0 pass,
  1 breach, 3 missing-subject. Measures a /tmp copy; every absence
  loud.
- Invocation: `bash scripts/wasm_audit.sh`; anti-vacuity env
  `WASM_AUDIT_FATFIXTURE=<path>` must exit 1.
- Writes: none (measures a /tmp copy).
- Standing: GREEN.

## receipt_chain_sync.sh

- Purpose (L52): BLAKE3 sidecar chain over receipts/. One b3sum per
  file + chain head; TAMPER-DETECTED exit 1 on any digest change
  (receipts immutable). Modes: default sync, `--check`, `--force`.
- Invocation: `receipt_chain_sync.sh [--check] [--force]`.
- Writes: `receipts/CHAIN.blake3`.
- Standing: VERIFIED — 840 receipts, head c2d79ff2… (W57, re-baseline
  #2 with note appended).

## receipt_replay_court.sh

- Purpose (K7): chain proves byte-immutability, not truth. Re-executes
  the embedded `cargo test --test X` from a receipt sample; compares
  embedded vs observed counts. Selfcheck: a fabricated receipt must
  be flagged (VACUOUS GATE otherwise).
- Invocation: `receipt_replay_court.sh [check|selfcheck]`; default
  all. Exit 1 = divergence or vacuous.
- Writes: none.
- Standing: NRW; W57 witnessed receipt-replay 6/6 after the
  head-parser fix.

## gen_w3c_manifest.sh

- Purpose: re-runnable generator for tests/w3c_manifest.rs from the
  vendored W3C RDF 1.1 syntax suites (manifest.ttl per dialect).
  Syntax tests only; RDF/XML positives admitted as parse-acceptance.
- Invocation: `bash scripts/gen_w3c_manifest.sh`.
- Writes: tests/w3c_manifest.rs (checked-in generated module).
- Standing: GREEN — no-drift.

## wasm_unknown_unknown_smoke.sh

- Purpose (W03 falsifier, G13): artifact exists + wasm magic; if a
  wasm runtime is present, ZERO wasi_snapshot_preview1 imports
  (fail-closed exit 1); loud exit 3 skip with no tooling.
- Invocation: `bash scripts/wasm_unknown_unknown_smoke.sh`.
- Writes: none.
- Standing: TYPED SKIP — exit 3 (no runtime tooling on host), loud,
  not a pass.

## Integration-commit gate sequence membership

Per `designs/post-commit-runbook.md` (CONSOLIDATION, coordinator
executes after the wave commit), in sequence:

1. mutation_court.sh — Step 1, gonogo P8; typed-EXCLUDED-HOT-PATH
   from bucket assignment (commit-manifest.md:63).
2. partition_court.sh — G2 AXIOM DRIFT check.
3. doc_staleness_gate.sh.
4. receipt_chain_sync.sh (+ `--check`).

Not in the runbook sequence (standalone / CI / lane-run): witness_gate,
machine_shaped_gate, chain_flag_court, count_gate, exit_capture_check,
fleet_cleanup, receipt_replay_court, shape_mutation_court, spc_court,
wasm_audit, gen_w3c_manifest, wasm_unknown_unknown_smoke.

## dl gates

No `scripts/*dl*.sh` exists — the absence is stated loudly. The
dl-class gates are cargo test courts; the single court is
`tests/dl_dualroute_vector_pin.rs`, invoked as:

```
cargo test --test dl_dualroute_vector_pin
```

re-pinned this wave with 4 real emitted digests (`dl_dualroute.rs`
is a bucket-2 rider in commit-manifest.md:39). Witnessed this session
(2026-10-23, real run on the live tree):

```
test dl_dualroute_vector_bytes_are_pinned ... ok
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured;
0 filtered out; finished in 27.82s
```

The court runs the real generator binary
(`src/bin/dl_dualroute.rs`) and asserts the emitted vector bytes are
byte-pinned; the four OK lines above are dual-route agreement per
vector (`a-paper-baader-el`, `b-chain3-conj`, `c-exists-pair`,
`d-exists-conj-interlock`) written into the wasm4pm-cognition
fixtures tree.

## Totals

18 `scripts/*.sh` inventoried (17 gates/courts/generators + the
partition selfcheck pair counted once with partition_court). 4 in the
integration sequence; 13 standalone; 1 typed skip; 1 typed-red
(mutation court, clean-tree-blocked). 19 files under scripts/ counting
the .py vector generator, which is out of scope.
