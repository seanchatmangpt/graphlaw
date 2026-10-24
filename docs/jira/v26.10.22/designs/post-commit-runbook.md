# Post-Commit Runbook — v26.10.22 (lane N20)

Filed 2026-10-22. Coordinator-executed, in order, after the wave commit lands
and `src/` is quiescent. Every command is sourced verbatim from an on-disk
receipt (citation per step). No step below has been executed by this lane;
this file is a consolidation, not a receipt. Companion:
[integration-gonogo.md](integration-gonogo.md) (P8/P9 gate), [commit-manifest.md](commit-manifest.md).

## Step 0 — Precondition

`git status --porcelain src/ | wc -l` → `0`. Steps 1 and 3 are UNSAFE on a
dirty tree (mutation court refuses typed, exit 6 —
[mutation-court-full-receipt.md:19-49](../designs/mutation-court-full-receipt.md);
G2 refuses `AXIOM DRIFT` on real churn —
[code-partition-census.md:88-89](../designs/code-partition-census.md)).
If non-zero: STOP, the wave commit has not landed.

## Step 1 — Mutation sweep full run (gonogo P8)

- **Command**
  (`mutation-court-full-receipt.md:85` and N1 amendment `:105-106`):
  ```
  CARGO_TARGET_DIR=target-mutants bash scripts/mutation_court.sh --log /tmp/mutation_court.log
  ```
- **Expected**: exit 0 on a clean src/. Scope is 319 mutants
  (`mutation-court-full-receipt.md:54` — src/abi.rs 203 + src/registry.rs 116;
  gated surface 124).
- **Exit discipline**: 0 = court admits; 4 = restore-mismatch (court FAIL —
  treat as defect, not retry); 6 = MUTATION-COURT-DIRTY-TREE (src/ not clean —
  return to Step 0, do not pass --force; the refusal of 2026-10-22 was the
  gate firing as designed, `mutation-court-full-receipt.md:9-14`).
- **Resume a partial run**: append `--case` resumed from the prior `--log`
  (`mutation-court-full-receipt.md:86`).
- **Failure branch**: any non-0/4/6 exit or a leaked mutant edit (EXIT trap
  skipped, e.g. OOM/SIGKILL) → stop the runbook, restore from git, file the
  incident in `RECEIPTS.md` before Step 2.
- **Receipt filed**: append the run verdict to
  `docs/jira/v26.10.22/designs/mutation-court-full-receipt.md` (amendment
  section) and `RECEIPTS.md` (P8 close).

## Step 2 — TC_HOT_FULL quiet reattempt (gonogo P8)

- **Precondition gate**
  (`tc-hot-full-receipt.md:143-145`): `uptime` → 1-min loadavg must be < 6
  (L118's budget; 9.13 was typed-ABORT TOO-LOADED). No other cargo lanes live.
- **Command** (`tc-hot-full-receipt.md:18`, identical both prior runs):
  ```
  TC_HOT_FULL=1 TC_HOT_CORPUS_PATH=/Users/sac/graphlaw/target-lane92/tc-hot-25m ./target-lane99/release/deps/tc_hot_02-dbe374288cafca9b
  ```
  If the lane build root was purged at integration (fan-out cleanup law),
  rebuild first: `CARGO_TARGET_DIR=target-lane99 cargo build --release --bench tc_hot_02`
  (`tc-hot-full-receipt.md:15`).
- **Corpus**: persisted at `/Users/sac/graphlaw/target-lane92/tc-hot-25m`
  (8.0 MB, 2 parquet, schema+25M row gates PASS both runs) — do NOT regenerate
  (`tc-hot-full-receipt.md:19-23, 134-139`). Note: this corpus lives under a
  lane build root (`target-lane92/`); if integration purged it, re-generate
  once and re-run the corpus gates before timing.
- **Verdict rules**: PASS requires corpus_assert PASS, timer sanity PASS,
  readback PASS, and variance < 5% (`tc-hot-full-receipt.md:87, 93, 131`).
  Variance ≥ 5% → exit 1, REATTEMPT-QUEUED again (two prior failures at
  7.40% / 15.13% under load — do not launder). Until a quiet-machine PASS,
  the FULL PCE number is NOT sound and must not be cited
  (`tc-hot-full-receipt.md:134-139`).
- **Failure branch**: TOO-LOADED abort → leave the requeue window open, retry
  when quiet. Variance FAIL on a quiet machine → treat as a real defect, not
  a load artifact.
- **Receipt filed**: amend `designs/tc-hot-full-receipt.md` (new reattempt leg
  section, loadavg snapshot + verbatim output).

## Step 3 — Partition-court G2 axiom re-pin (gonogo P9)

- **Axioms drifted this wave** (read-only b3sum check by N20, 2026-10-22,
  vs `courts/code-partition/axioms.blake3` pins):
  - `src/duckdb_eval.rs` — pin `cc199cf6…` actual `11399ac7…`
  - `src/law.rs` — pin `3143efed…` actual `1392d549…`
  - `src/plan.rs` — pin `7287b986…` actual `0cf1ac89…`
  - `src/sql_split/ast.rs` — pin `748a957b…` actual `4541822f…`
  (10 of 14 axioms still match.) G2's law: each re-pin is a court-revision
  event (`code-partition-census.md:88-90`); the court refuses with
  `G2: AXIOM DRIFT <file>` until pins match (`scripts/partition_court.sh:73-78`).
- **Command** (`code-partition-census.md:11`):
  ```
  bash scripts/partition_court.sh; echo "EXIT=$?"
  ```
- **Re-pin order** (court-revision event, in the same commit): update
  `courts/code-partition/axioms.blake3` + `courts/code-partition/manifest.json`
  per-file blake3 for the 4 drifted axioms (b3sum the committed bytes), THEN
  run the court. Expect: G1 PASS, G2 PASS, G3 FAIL with 44 unadmitted leaks
  (expected, honest — the leak list is the codegen roadmap,
  `code-partition-census.md:78`), G4 PASS, anti-vacuity both legs FLIPPED,
  **exit 1** — the known-stable verdict while any unadmitted leak exists.
- **Failure branch**: `G2: AXIOM DRIFT` on a file NOT in the 4-file list → a
  post-commit axiom edit landed after this census; re-run the drift check and
  re-pin that file too. G1/G4/anti-vacuity FAIL → defect, stop.
- **Receipt filed**: amend `designs/code-partition-census.md` (re-pin event
  section: old/new digests, court output, exit).

## Step 4 — Doc-staleness gate final pass (N10 census base)

- **Command** (`m12-staleness-receipt.md:111`, N10 section):
  ```
  bash scripts/doc_staleness_gate.sh
  ```
- **Expected**: exit 1 with **76** findings, ALL in dated jira history
  (`docs/jira/v26.10.22/RECEIPTS.md`, `designs/*`, `SQLRDF-19.md`,
  `mega/tc-mega-fibo-01/README.md`) — zero live-instruction findings in
  lane-owned docs (`m12-staleness-receipt.md:111-118`). Exit 1 here is the
  stable, honest state; it is not a regression.
- **Anti-vacuity spot check** (`m12-staleness-receipt.md:119-121`): scratch a
  file with an unregistered 64-hex digest + a banned string → 76→78, both R2+R3
  fire; remove scratch → back to exactly 76.
- **Failure branch**: count ≠ 76 → diff against the N10 census; any NEW finding
  naming a live doc → fix the doc before landing; any finding vanished →
  verify the doc was intentionally edited, not the gate.
- **Receipt filed**: append the final-pass count to
  `designs/m12-staleness-receipt.md` (N10 section).

## Step 5 — Receipts chain resync

- **Commands** (`chain-resync-receipt.md:74-79`, N14 witness):
  ```
  bash scripts/receipt_chain_sync.sh
  bash scripts/receipt_chain_sync.sh --check
  ```
- **Expected**: sync → `NO-OP` (or chains any receipts/ files new since the
  head `2c316b0261c992cfb0463d135e4af4f6975daf9318368bfaf531098b57749d05`), (historical)
  exit 0; `--check` → `VERIFIED: 8 receipts match CHAIN.blake3`, exit 0.
- **Failure branch**: `TAMPER-DETECTED` (as at
  `chain-resync-receipt.md:11-21`) → STOP; per the L108 finding, the chain
  refuses over an in-place-mutated receipt. Identify the writer defect, fix
  forward (append-only writer, L111 precedent `:57-67`); re-baselining the
  chain is a coordinator decision with provenance preserved in git history —
  never a silent overwrite.
- **Receipt filed**: a new section in `designs/chain-resync-receipt.md`
  (before/after head, census table).

## Step 6 — Artifact re-pin sequence (gonogo P2, L122) — GATED

- **Gate**: `eh-runner-decision-memo.md:4, 74` — status stays
  **DECISION REQUIRED** until the operator picks the EH route;
  `integration-gonogo.md:42` — "coordinator MUST NOT re-pin early"
  (`docs/WASM.md:88-100`, pin currency law). D1–D6 operator decisions:
  `dual-artifact-policy-design.md:128-133` (defaults listed there; D1 default
  D1b, D3 default `8b4f4bc8…`, D6 default re-build-once-and-pin-that).
- **Do NOT run any re-pin in this runbook pass while the gate is closed.**
  The on-disk pins (`332ea218…`, 6,793,617 B) describe the 26.10.21 release,
  not the current tree (`docs/WASM.md:88-93`) — that is the honest state.
- **When the operator opens the gate** (Route C per the memo, D1–D6 defaults):
  1. Engine build (`dual-artifact-policy-design.md:92-93`, L115/L116-witnessed):
     ```
     export WASI_SDK_PATH=<wasi-sdk root>
     CARGO_TARGET_DIR=<lane root> cargo build -p graphlaw-wasm --target wasm32-wasip1 --profile wasm
     ```
  2. Digest and re-pin (`dual-artifact-policy-design.md:94`): `b3sum <artifact>`
     → re-pin the wasm row (`wasm-pure` + `wasm-engine` rows per
     `dual-artifact-policy-design.md:42-46`) in `registry/ARTIFACTS.blake3` and
     `priv/graphlaw.wasm.blake3` **in the same commit** as any source change
     that moved the digest. `graphlaw-registry -- --write` does NOT touch wasm
     rows (`dual-artifact-policy-design.md:97`).
  3. Registry regen for the receipt-surface row (`receipt-surface-pin.md:33-34`,
     gws-16 pattern; gonogo P7): `cargo run --features abi --bin graphlaw-registry -- --write`
     (emitter hint, `integration-gonogo.md:47`), then re-run the L147
     acceptance set going 117/0 (`integration-gonogo.md:47`).
- **Failure branch**: any `--expect-blake3` / digest-court mismatch (exit-84
  path, `dual-artifact-policy-design.md:55, 133`) → stop, the pin and the
  bytes disagree; never patch the pin to match unexplained bytes.
- **Receipt filed**: `designs/registry-regen-receipt.md` amendment +
  `RECEIPTS.md` (P2/P7 close).

## Step 7 — js/npm resync

- **Lane-attribution note (loud)**: the dispatch named "L42/L74 recipes";
  no L42 or L74 receipt exists on disk (grep across `docs/jira/` → 0 hits).
  The js/npm witnesses of record are **L107** (`js-test-receipt.md:1`) and the
  **N9 re-run** (`js-test-receipt.md:94-110`). Commands below cite those.
- **Commands, cwd `js/npm`** (`js-test-receipt.md:100-106`):
  | # | Command | Expected exit |
  |---|---|---|
  | 1 | `npm run build` | 0 — gen → build.mjs (wasm in place, blake3 `332ea218…`) → tsc → assemble |
  | 2 | `npm run test` | 0 — 11 files / 198 tests passed |
  | 3 | `npm run sync-version` | 0 — `already at 26.10.21` |
  | 4 | `npm run gen` | 0 — regenerated types.ts matches the registry |
  | 5 | `GRAPHLAW_BROWSER_TESTS=1 npm run test:browser` | **1 = BLOCKED** on missing Playwright chromium — typed, loud, not a silent skip; recovery `npx playwright install chromium` (`js-test-receipt.md:90-92, 106`) |
- **Drift gates** (`js-test-receipt.md:65-74`): the three
  `git diff --exit-code -- …` gates must each exit 0 (version, generated dir,
  types.ts); plus `b3sum priv/graphlaw.wasm` must equal
  `awk '{print $1}' priv/graphlaw.wasm.blake3`.
- **Ordering note**: step 7 runs AFTER Step 6's pin decision — if the wasm
  artifact changes, `npm run build` picks up the new bytes and the drift gates
  re-verify the pin handoff.
- **Failure branch**: limits-mirror failure (`max_outstanding_alloc_bytes`,
  `js-test-receipt.md:37-39`) → the `js/npm/src/limits.ts` mirror is stale vs
  the registry; fix the mirror, not the test.
- **Receipt filed**: append a new re-run table to
  `designs/js-test-receipt.md`.

## Order rationale

`integration-gonogo.md:68-69`: P3 lands → P7 final regen → commit → P8/P9
post-commit sweep. Steps 1–5 are safe once `src/` is quiescent; Step 6 is
operator-gated and blocks only Step 7's pin-handoff verification, not Steps
1–5.

## See Also

[integration-gonogo.md](integration-gonogo.md) ·
[eh-runner-decision-memo.md](eh-runner-decision-memo.md) ·
[dual-artifact-policy-design.md](dual-artifact-policy-design.md) ·
[m12-staleness-receipt.md](m12-staleness-receipt.md) ·
[chain-resync-receipt.md](chain-resync-receipt.md)
