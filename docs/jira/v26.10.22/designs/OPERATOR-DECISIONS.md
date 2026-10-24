# Operator Decisions — consolidated sign-off surface (L153)

Date: 2026-10-22 · Lane: L153 (docs lane; owns this file only) · Repo: `/Users/sac/graphlaw`
Status: **DECISION REGISTER — nothing decided by this lane.** Every entry: question,
evidence (receipt file:line, verified resolving on disk this session), options with costs,
coordinator recommendation, fill-in decision line. Ordered by blocking impact.

Sign-off pass 2026-10-24 (closes audit row R-25,
`docs/jira/v26.10.23/compliance-final-verdict.md:228-230`): OD-1, OD-3, OD-5
ADMITTED-as-executed with landing commits + receipt loci; OD-2 recorded SUPERSEDED —
no sign-off, its Route-C policy was stood down before execution; OD-4 and OD-6's
own subject remain open. Authority for every 2026-10-24 completion: operator
publish directive 2026-10-24 (`compliance-final-verdict.md:388`).

---

## OD-1. EH route for the wasm artifact (blocks the wasm tier and smalt pairing)

**Question.** Which route unblocks hosting the DuckDB-linked `graphlaw-wasm` module?
The artifact as built (mixed legacy+new EH) is hostable by NO staged runtime.

**Evidence.**
- Tag section, wasmi 2.0.0 has no exceptions knob:
  `designs/gws4-postlink-receipt.md:26-33, 37-40` — resolves.
- libunwind is required; excluding it fails the link with typed undefs; engine module
  carries legacy `try`: `designs/wasi-sdk-reproducibility.md:144-151`.
- Engine artifact exits 95 under wasmi; pre-duckdb generation `8b4f4bc8…`
  (6,655,651 B) serves real packet-4 frames in wasmi; smalt 3/7: `designs/smalt-pairing-receipt.md:47-57, 74-104`.
- node 26 REJECTS the mixed-EH module; artifact unhostable by any staged runtime:
  `eh-runner-decision-memo.md:76-100` (L126 stand-down amendment).
- **A1' is now probe-witnessed** (strengthens A1'): emsdk clang 24 accepts
  `-mllvm -wasm-use-legacy-eh=0` at compile and link; opcode census flips cleanly
  (try 4/delegate 2/rethrow 4 → try_table 12/catch_ref 1/throw_ref 4/exnref 8);
  try_table probe runs exit 0 on node 24.19.0 AND wasmtime 48.0.1; legacy probe
  fails wasmtime (exit 1) — the non-vacuity flip:
  `designs/a1prime-trytable-recipe.md:9-29, 98-110`.
- Positive frame witness for the PURE tier on exactly `8b4f4bc8…`:
  `designs/pure-artifact-witness.md:43-176` (capabilities/parse/law frames, exit 84/90
  negatives, digest-pin positive exit 0).

**Options.**
- **A1'** — emsdk rebuild of duckdb emitting try_table uniformly. Cost: build-side
  change to `frozen-duckdb/scripts/build_wasi_duckdb.sh`; R2b (runtime .a members
  may still carry legacy opcodes) and R2c (rustc `-Z llvm-args` on stable) are
  UNVERIFIABLE-LOCALLY (`a1prime-trytable-recipe.md:120-127`). Upside: probe-level
  flip is witnessed on both candidate hosts.
- **A3** — pin an old legacy-EH wasmtime/wasmer. Cost: EOL tooling freeze.
- **B** — duckdb `-fno-exceptions` rebuild. Cost: unknown risk of breaking duckdb
  C++ internals; may be UNSUPPORTED(BUILD_SEMANTICS) (`eh-runner-decision-memo.md:40-42`).
- **C** — dual-artifact policy. Cost: two pins, two host runtimes. Upside: the ONLY
  route keeping the wasmi court tier on a real module short-term
  (`dual-artifact-policy-design.md:5`).

**Coordinator recommendation.** C now (registry split + host selector + court retier),
with A1' as the engine-tier backend candidate — per
`eh-runner-decision-memo.md:53-62` + `dual-artifact-policy-design.md:5`. A1' is
materially stronger since L144's probes: the compile/link flag is witnessed to flip
opcode encoding and run on both hosts. Route B stays ranked last.

**DECISION (operator):** ADMITTED-as-executed — route = wasmtime 48 +
`wasm_exceptions(true)` hosting the SINGLE DuckDB-linked artifact, executed
through the A1' try_table archive law (`-fwasm-exceptions
-mllvm -wasm-use-legacy-eh=0`); Route C (dual-artifact) STOOD DOWN.
Signed off 2026-10-24 under the operator publish directive 2026-10-24
(authorizing act: `docs/jira/v26.10.23/compliance-final-verdict.md:388`).
Sign-off record: operator decision dated 2026-10-23,
`docs/archive/designs-2026-10-23/eh-runner-decision-memo.md:134-160`;
landing commits `16678de`→`679299d`→`e94d9bd`→`907e663` (EH-closure wave;
`GO-NO-GO-v26.10.22.md:7-8,33-37`); 4-surface re-pin to `58a44095…`
(58,124,612 B) committed ahead of tag v26.10.22 at `669c817+`
(`registry/ARTIFACTS.blake3:13`, `priv/graphlaw.wasm.blake3`,
`js/npm/src/generated/artifact-meta.ts:3`; `GO-NO-GO-v26.10.22.md:82-89`);
receipts `receipts/2026-10-23-eh-closure-wave.md` + GO-NO-GO §v4
(published-by-machine through validate-and-pack green, CI run 40084544193).

---

## OD-2. Dual-artifact sub-decisions D1–D6 (execute OD-1=C)

**Question.** Six parameters of the dual-artifact policy
(`dual-artifact-policy-design.md:126-133`).

**Evidence.**
- D3 is now EVIDENCE-BACKED: `designs/pure-artifact-witness.md:147-160` — the beam4pm
  artifact IS the `8b4f4bc8…` bytes (BLAKE3 exact), full frame witness passed against
  exactly those bytes; `332ea218…` (6,793,617 B) is a different byte stream, never
  frame-witnessed.
- D1a boundary: frozen-duckdb-sys `build.rs` fails typed `WASI_SDK_PATH required`
  without the env — `src/abi_manifest.rs:137` (via `dual-artifact-policy-design.md:101`).
- D5 mechanism witnessed: `--expect-blake3` exit-84 path,
  `smalt-pairing-receipt.md:18` (gate 2b) and `pure-artifact-witness.md:113-143`.

**Options (defaults per §6 of the design).**
- **D1** PURE v1 pin source — D1b (pin `8b4f4bc8…`, pin-source=beam4pm@<sha>,
  recipe-external, immediate) vs D1a (feature-off rebuild knob; build change, then
  re-pin). Cost of D1b: pin points at foreign bytes until D1a. Default: D1b.
- **D2** engine-tier EH host pre-A1': wasmtime `legacy_exceptions` (witnessed FAILING
  at default config, L109/L126) or A3 (pin old runtime, EOL-freeze). Default: none —
  ENGINE tier stays PIN-ONLY until A1' witnesses a host.
- **D3** PURE tier digest — `8b4f4bc8…` (frame-witnessed) vs `332ea218…` (never
  frame-witnessed). **Now evidence-backed; default is no longer a guess.**
- **D6** ENGINE canonical digest — `940af4c7…` vs `500333a4…`. Default: rebuild once
  at pin time, pin THAT digest.
- **D4** graphlaw-run `--host` flag — out of scope until A1' witnesses a host. Default: out of scope.
- **D5** tier banner — reuse exit-84 (witnessed). Default: reuse.

**Coordinator recommendation.** Accept all defaults: D1b, D2=none (PIN-ONLY), D3=
`8b4f4bc8…`, D6=rebuild-and-pin, D4=out of scope, D5=exit-84 reuse. This makes the
policy execute with zero un-witnessed assertions; D1a tightening and D4 follow A1'
outcomes.

**DECISION (operator):** NO SIGN-OFF — SUPERSEDED, not executed. The OD-1
decision STOOD DOWN Route C (`docs/archive/designs-2026-10-23/eh-runner-decision-memo.md:134-160`),
so D1–D6 lapsed as parameters of a policy that never landed; no letter was
picked and none is admitted here. The pin state OD-2 would have governed
was resolved by the EH-closure re-pin instead: single registry row
`wasm 58a44095…` 58,124,612 B (`registry/ARTIFACTS.blake3:12-13`; the
`wasm-pure`/`wasm-engine` row split never landed). Disposition recorded
2026-10-24; authority: operator publish directive 2026-10-24
(`compliance-final-verdict.md:388`). Disposition record, not an admission.

---

## OD-3. Registry re-pin timing vs dual-artifact adoption

**Question.** Re-pin `registry/ARTIFACTS.blake3` / capability-registry now, or only
after the dual-artifact split lands?

**Evidence.**
- The regen flow is real and courted: `cargo run --features abi --bin
  graphlaw-registry -- --write`, registry_artifacts 22/0, registry_dispatch 14/0 —
  `designs/registry-regen-receipt.md:5-8, 40-44`.
- graphlaw-registry never touches the wasm row: `registry/ARTIFACTS.blake3:6`
  header law; enforced at `graphlaw_registry.rs:231-239` (cited in
  `registry-regen-receipt.md`).
- The current single `wasm 332ea218…` row collides generations (W9;
  `smalt-pairing-receipt.md:100-104`).
- The dual-artifact split itself edits ARTIFACTS.blake3 (adds `wasm-pure`/
  `wasm-engine` rows, `dual-artifact-policy-design.md:39-46`), and the W09 law
  requires the pin to move in the same commit as the source change that moves the digest.

**Options.**
- **(a) Re-pin now, split later**: registry rows are fresh against today's tree; the
  split then edits the file again. Cost: two consecutive churn commits to
  ARTIFACTS.blake3.
- **(b) Split first, single regen**: fold the row split (OD-2 defaults) and the owned-row
  re-pin into one commit. Cost: until then, registry rows stay at the L73 regen state;
  acceptable — the regen receipt is current as of 2026-10-22 and `surface` is unchanged.

**Coordinator recommendation.** **(b)** — one ARTIFACTS.blake3 commit carrying both the
OD-2 row split and the owned-row re-pin, satisfying W09 same-commit law and halving
pin churn. If the operator wants the registry green before any split work starts,
fall back to (a); the flow is receipted either way.

**DECISION (operator):** ADMITTED-as-executed (resolved by execution; no
(a)/(b) letter was picked): the re-pin landed 2026-10-23 WITHOUT any
dual-artifact split — the option-(a)-shaped outcome. Single `wasm 58a44095…`
row moved with the EH-closure wave (commits `e94d9bd`→`907e663`; 4-surface
re-pin ahead of tag `669c817+`). Receipt locus: `registry/ARTIFACTS.blake3:7,12-13`
(26.10.22 pin marked superseded 2026-10-23) + `GO-NO-GO-v26.10.22.md:82-89`.
Signed off 2026-10-24 under the operator publish directive 2026-10-24
(`compliance-final-verdict.md:388`).

---

## OD-4. Cpk / charter CTQ production bar (SPC gate is red)

**Question.** What scope of perf work closes the red SPC gate, given the operator's
production bar is the full charter per operator dispatch?

**Evidence.**
- Charter Cpk 1.44 REFUTED: measured Cpk(Cpu) = −0.05, P(X>USL=250ns)=42.9% —
  `designs/cpk-correction.md:10-12`, `CHARTER-DFLSS.md:80-87`.
- SPC court honest-FAIL both legs: cpk −0.0018/−0.0025 < 1.33, p99 375 ns > 250 ns
  USL — `CHARTER-DFLSS.md:89-94`; gate `bash scripts/spc_court.sh`.
- Refusal latency CTQ UNVERIFIED/NOT-MET (ungated);
  throughput NOT-MET at claimed scale (12M printed, 25M TC_HOT_FULL unexecuted) —
  `CHARTER-DFLSS.md:16-37`; close condition already stated: quiet-machine
  re-measurement, no concurrent lanes (`CHARTER-DFLSS.md:98-99`).

**Options.**
- **(a) Full charter closure**: quiet-machine 10M-iteration capability re-measure,
  land the `RefusalMessage::Static` fast path, execute the 25M TC_HOT_FULL run,
  gate all three CTQs. Cost: dedicated quiet-machine windows; largest scope.
- **(b) SPC-gate closure only**: quiet-machine re-measure to restore Cpk ≥ 1.33 on the
  fold path (or operator re-baselines USL against the measured 250–300 ns intrinsic
  fold cost). Cost: leaves refusal-latency and throughput CTQs open.
- **(c) Re-baseline the bar**: accept measured intrinsic fold cost 250–300 ns as the
  new USL basis; rewrite charter §5.2/§1.2 to measured reality. Cost: a charter edit,
  requires operator authority (this lane cannot re-baseline its own bar).

**Coordinator recommendation.** **(a)** if the 26.10.22 milestone is to close
charter-green; otherwise **(b)** now with (c) recorded honestly as charter-text drift.
Any of the three requires quiet-machine measurement — no gate can flip under fan-out
contention (all five runs measured Cpk ≤ 0 under contention,
`cpk-correction.md:12`).

**DECISION (operator):** (a)/(b)/(c): ______  date: __________

### Decision memo filed 2026-10-24 (closeout docs lane — evidence only, no decision authored here)

This memo discharges the docs lane's evidence duty for OD-4. The DECISION
line above stays BLANK: the note at the head of this file governs — nothing
is decided by a docs lane. The operator fills the line (and summary row
below); runbook step S3
(`docs/jira/v26.10.23/OPERATOR-RUNBOOK-v26.10.22-PUBLISH.md`) carries the
act.

**Evidence table** (every row re-read on disk 2026-10-24):

| # | Observation | Value | Locus |
|---|---|---|---|
| 1 | Latest SPC receipt (solo rerun, SMOKE 1M iters) | Cpk **0.632** — honest FAIL (exit 1); mean 128.2 ns, **sd 64.24 ns**, p99 167 ns (inside USL 250 ns); 283 Shewhart rule-1 samples | `receipts/2026-10-24-spc-solo-rerun.md` + machine JSON `receipts/spc-2026-10-24T032511.json` |
| 2 | Quiet-machine witness in the standing set | Cpk **1.7573 PASS** @ sd **23.3 ns** (03:54 quiet run) — the same gate, same fold path | `receipts/spc-2026-10-23T035422.json` |
| 3 | Gate bistability | PASS at sd 23.3 ns vs FAIL at sd ≥ 48.8 ns; threshold: sd ≤ **30.5 ns** (derived from the Cpk arithmetic at the observed mean) | rows 1-2 arithmetic |
| 4 | sd root cause, primary | **Timer quantization**: 41.667 ns tick clock (14.318 MHz); every observed quantile is a tick multiple; 2×`Instant::now` contributes ~20-40 ns of the measured number; single-sample noise floor ~12.5 ns | receipts 1-2 quantile tables |
| 5 | Secondary | IRQ shoulder: 283 rule-1 samples, rule-2 never fires — isolated spikes, not drift | receipt 1 |
| 6 | Fuel-boundary coherence | p99 167 ns is deep inside the 250 ns USL; the 10M-era exit-98 red is resolved by the 100B calibration (`receipts/2026-10-24-fuel-calibration.md`) — SPC standing is not a correctness gate | `receipts/2026-10-24-fuel-calibration.md` |
| 7 | FULL mode | **Never receipted at FULL** (every standing receipt is SMOKE); SD rung 9's "FULL" asks stay open | register row 4 |

**Root-cause line**: the gate is bistable on sd alone; sd is dominated by
timer-tick quantization (41.667 ns) plus an IRQ shoulder, so the FAIL is a
measurement-method property of a loaded/SMOKE run, not a change in the
measured code — the fold path is verified allocation-free (cause-4
hypothesis falsified by inspection).

**Options** (unchanged from the register, now informed by rows 1-7):

- **(a) Full charter closure** — quiet-machine FULL-mode 10M-iteration
  capability re-measure + the `RefusalMessage::Static` fast path + the 25M
  TC_HOT_FULL run, all three CTQs gated. Consequence: largest scope,
  dedicated quiet windows; closes the charter honestly. Consequence of NOT
  (a): charter CTQs stay open past this release (posture-P follow-up).
- **(b) SPC-gate closure only** — quiet-machine re-measure until Cpk ≥ 1.33
  on the fold path. Consequence: cheapest honest green IF quiet-machine sd
  reliably lands ≤ 30.5 ns (the W57 witness says it can); leaves
  refusal-latency + throughput CTQs open as registered residuals.
- **(c) Re-baseline the bar** — accept the measured intrinsic cost as the
  new USL basis (rewrite charter §5.2/§1.2). Consequence: needs operator
  authority over the charter text; converts the FAIL into an honest
  re-baseline instead of a chase against a 41.667 ns tick.

**Falsifier for the choice**: two consecutive quiet FULL-mode FAILs at sd
45-70 ns with p99 ≤ 167 ns → (b) is unreachable as written → sign (c)
(re-baseline per the F1a amortized-window fix) — or fund (a). A single
noisy-window FAIL falsifies nothing; the gate is load-bistable by rows 1-3.

**Consequences of leaving it OPEN (posture P)**: the SPC gate stays an
honest red on the latest receipt; release standing is unaffected because
the gate is registered as a non-blocking follow-up (GO-NO-GO v5 §A row G2/
G3 class), but OD-4 remains an open operator debt in this register and the
runbook's S3 step stays owed.

---

## OD-5. Absolute paths in wasm/ggen.toml (commit as-is vs tempath-ize)

**Question.** `wasm/ggen.toml:17-18, 26-27` hard-code `/Users/sac/...` ontology,
pack, query, and template paths. Commit as-is or normalize?

**Evidence.**
- The four absolute-path lines resolve today and the ggen v2 receipt chain records
  them as closure inputs with per-input digests:
  `wasm/.ggen-v2/receipt-log.jsonl:1` (closure map, actuator ggen@26.10.11).
- Counter-example in-tree: `ggen/sql-dialect/ggen.toml` uses fully relative paths
  (`ggen/sql-dialect/ggen.toml:9, 17-34`) — the normalizable shape already exists
  in this repo.
- `ggen/sql-dialect/extract_statements.py:71` and generated
  `ggen/sql-dialect/sql_statement_kinds.rs:66` embed `/Users/sac/.duckdb/extensions`
  inside a PEG example string — generated files are regenerated, not patched (repo law).

**Options.**
- **(a) Commit as-is**: zero work; digests in the receipt chain stay valid. Cost: the
  repo is not fresh-clone reproducible for `wasm/src/ffi.rs` regeneration; violates
  the one-canonical-checkout portability intent; the closure map leaks a local
  username.
- **(b) Tempath-ize (relative/env-var paths)**: rewrite the four lines relative to
  `wasm/` (pack files via a `GGEN_MARKETPLACE` env or vendored pack path). Cost: must
  re-run `ggen sync` and append a new receipt-log entry (the chain re-folds; digests
  change), plus verify `wasm/src/ffi.rs` output is byte-identical. Risk: if ggen.toml
  supports no path indirection, this is UNSUPPORTED(generator-config) and (a) becomes
  the honest fallback.
- **(c) Vendor the pack**: copy rust-wasi-wasmex-pack into `ggen/` at a pinned digest,
  point ggen.toml at it relatively. Cost: pack duplication; but matches the P-pin
  primitive (SHA-pinned pack by reference, composition catalog P).

**Coordinator recommendation.** **(b) with (c) as the tightening step**: try
relative paths first, witness byte-identical ffi.rs output via a fresh ggen sync + new
receipt; if path indirection is UNSUPPORTED, fall back to (a) and register the
divergence. Do NOT hand-patch the generated `sql_statement_kinds.rs:66` — that fix
belongs in `extract_statements.py`'s example-string sanitization at next regen.

**DECISION (operator):** ADMITTED-as-executed (resolved by execution; no
(a)/(b)/(c) letter was picked): option (b) tempath-ize landed — all
`wasm/ggen.toml` paths are relative to wasm/ (L160 normalization
2026-10-22, `wasm/ggen.toml:13-21`), CLEARED in the commit manifest
(`designs/commit-manifest.md:103,119`; landing batch 5 "wasm link-green +
js/npm + CI", `designs/commit-manifest.md:42`). Determinism receipt
embedded at `wasm/ggen.toml:5-7` (b3sum `9dcab4e4…`). Signed off
2026-10-24 under the operator publish directive 2026-10-24
(`compliance-final-verdict.md:388`).

---

## OD-6. Commit .claude/ as repo law

**Question.** The hooks and settings are working-tree-only (`?? .claude/settings.json`,
`?? .claude/hooks/disk_guard.py`, `?? .claude/hooks/manifest_guard.py` per git status);
commit them as repo policy or keep local?

**Evidence.**
- Already tracked today (precedent): `.claude/agents/*.md`, `.claude/hooks/
  ban_generic_agents.py`, `topology_guard.py`, `topology_guard_cases.json`
  (git ls-files output, this session).
- Untracked new guards: settings.json wiring all four guards into PreToolUse;
  disk_guard.py, manifest_guard.py. `__pycache__/` is untracked and NOT gitignored
  (git check-ignore rc=1; only `.claude/scheduled_tasks.lock` is ignored, `.gitignore:16`).
- CLAUDE.md (committed) already declares agent policy as repo law
  (`/Users/sac/graphlaw/CLAUDE.md` "Agents" section); the untracked hooks enforce
  what the committed text claims.

**Options.**
- **(a) Commit settings.json + disk_guard.py + manifest_guard.py; gitignore __pycache__**:
  policy becomes law for every clone. Cost: hooks run for any clone; a broken hook
  freezes all agent edits (mitigated: hooks are typed-refusal scripts, court them like
  any code).
  - **(b) Keep local**: zero exposure. Cost: committed CLAUDE.md claims rules the
    working tree enforces but a fresh clone does not — a doc/behavior divergence.

**Coordinator recommendation.** **(a)** — the hooks ARE the enforcement half of
already-committed policy text; committing them closes a doc/behavior divergence, and
`__pycache__/` joins `.gitignore` alongside `scheduled_tasks.lock`.

**ANOMALY RECORD 2026-10-24**: the bracketed line below was filled with
OD-1 text in commit `0d11bc3`; it signed nothing for OD-6. It is retained
verbatim as evidence and superseded by the re-presented block under it.

> **[ANOMALOUS LINE — OD-1 TEXT, SIGNS NOTHING FOR OD-6]** **DECISION
> (operator):** (a) wasmtime 48 + `wasm_exceptions` single DuckDB-linked
> artifact (archive law `-mllvm -wasm-use-legacy-eh=0`); Route C stood
> down. Date: 2026-10-23. Witness: eh-runner-decision-memo DECISION block;
> artifact 46,015,267 B / Tag x1, wasmtime CLI validation exit 0.

> **DECISION (operator):** ______ (option (a) recorded-as-executed — see
> the execution-state witness below) date: __________
> signature: __________

**Execution state (witnessed, independent of the blank signature)**:
option (a) is de-facto EXECUTED on this tree — `.claude/settings.json`,
`.claude/hooks/disk_guard.py`, `.claude/hooks/manifest_guard.py` are all
tracked (`git ls-files .claude/`), the `.claude/` commit landed at
`dab6a73`, and `__pycache__/` is ignored (`__pycache__/` at `.gitignore:39`).
The re-presented block above exists so the signature debt can be discharged
without retroactively validating the corrupted line.

---

## Summary table

| ID | Decision | Blocks | Rec | Sign-off (2026-10-24) |
|---|---|---|---|---|
| OD-1 | EH route pick | wasm tier, smalt 3/7 | C now + A1' engine backend | ADMITTED-as-executed (wasmtime 48 single artifact; Route C stood down) |
| OD-2 | D1–D6 defaults | executes OD-1 | all defaults | NO SIGN-OFF — SUPERSEDED with the Route C stand-down |
| OD-3 | registry re-pin timing | ARTIFACTS.blake3 churn | (b) single split+re-pin commit | ADMITTED-as-executed (re-pin landed without any split) |
| OD-4 | Cpk/CTQ production bar | charter closure | (a) full closure; (b) minimum | memo filed 2026-10-24 (evidence table + root cause + falsifier, above); DECISION line BLANK — decision pending |
| OD-5 | ggen.toml absolute paths | portability | (b), fallback (a) registered | ADMITTED-as-executed (tempath-ize landed, L160) |
| OD-6 | .claude/ commit | repo-policy parity | (a) commit + gitignore __pycache__ | anomaly recorded 2026-10-24; re-presented with BLANK decision/date/signature; signature pending |

## See Also

`eh-runner-decision-memo.md` · `dual-artifact-policy-design.md` ·
`a1prime-trytable-recipe.md` · `pure-artifact-witness.md` ·
`registry-regen-receipt.md` · `cpk-correction.md` · `../../CHARTER-DFLSS.md` ·
`registry/ARTIFACTS.blake3` · `wasm/ggen.toml`
