# GO-NO-GO-v26.10.22

Final go/no-go (v3, supersedes the v2 at this path). Subject: `/Users/sac/graphlaw`,
HEAD `907e663` (fix(security): EH host mirror + ATTACH unit courts,
2026-10-24T01:09:56-07:00), working tree read window 2026-10-24 00:45–01:15 PDT.
Read-only lane: no cargo, no git transitions, no publish executed by this lane.
Warning: the tree moved under this lane — HEAD advanced `16678de` → `679299d`
(00:56:43) → `e94d9bd` → `907e663` (01:09:56) while evidence was being read,
and the security-wave hardening was observed to flip present→absent→present in
the working tree (details §1.1). Every count below is re-read at use time.

## Verdict

**NO-GO.**

Red on four independent, evidence-cited grounds — all narrowed since v2 but
none yet green on disk: (1) the security wave is committed only as of
`907e663` (01:09:56) with uncommitted residue still on the tree, and the
boundary rungs on that exact state have not landed green; (2) the mutation
sweep has no completed verdict — the 21:37 sweep was killed at exit 137 with
18/88 resolved and `end_time: null`, and the last wrapper invocation (mc5,
00:03) failed its post-restore sanity leg (typed exit 4); (3) the last landed
FULL battery output is RED (`battery9.log`, 2 failed) with no post-security-
wave full-battery receipt; (4) the R-1/R-2 pre-tag gates are open — the artifact
re-mint (`58a44095…`) is uncommitted and uncited, the release notes still cite
superseded `332ea218…` as current, and the cargo-deny residual advisories have
no dated operator decision.

## 1.1 Security wave — state at read close (item 1)

| Item | State at close | Evidence (grep counts, live) |
|---|---|---|
| `leading_sql_token` scanner | COMMITTED `679299d` (00:56:43), unit courts in `907e663` | `grep -c leading_sql_token src/duckdb_op.rs` = 14 (WT); `git log --all -S leading_sql_token` = `679299d` |
| FrameOversize, both hosts | COMMITTED (`679299d` run.rs; `907e663` eh) | `grep -c FrameOversize src/bin/graphlaw_run_eh.rs` = 4; `src/bin/graphlaw_run.rs` = 4 |
| Sterile WasiCtx (no inherit_env) | COMMITTED `907e663` | `grep -n "\.inherit_env()" src/bin/graphlaw_run_eh.rs` = only the :295 comment; the call is gone from the builder |
| Fuel/epoch | COMMITTED `907e663` | `grep -c "consume_fuel\|FUEL_PER_REQUEST" src/bin/graphlaw_run_eh.rs` = 8 (also in the `907e663` blob) |
| Uncommitted residue at close | `M src/bin/graphlaw_run_eh.rs`, `M src/duckdb_op.rs` on top of `907e663` | `git status --short` at 01:14 |
| Never-executed on disk evidence | the scanner's own courts (lib + wire court) have no landed green receipt | ps at 01:11 shows the lane running `cargo test --lib duckdb_op` + `graphlaw_run_eh_court` mid-write |

### 1.1 Observed instability (the moving-tree record)

Reads at 00:47 / 00:54 / 01:07 / 01:14 over one tree under concurrent edit:
- 00:47: eh had FrameOversize x4 + "no .inherit_env()" comments (uncommitted).
- 00:54: eh clean at HEAD `16678de` — NO FrameOversize, `.inherit_env()` LIVE
  at :249, no fuel/epoch; the scanner absent from `src/duckdb_op.rs` entirely
  (`grep "token\|leading" src/duckdb_op.rs` = no match at 00:54).
- 01:07: scanner back (`:235`), fuel back, `inherit_env` call gone.
- The lane then committed the wave: `679299d` (00:56:43) → `e94d9bd` →
  `907e663` (01:09:56).
So the v2-era "uncommitted" state was briefly REVERTED out of the working tree
(~00:48–00:56) and then re-landed as commits. During 00:48–00:56 the tree did
not contain the R3/C0b/C0c hardening at all. A go/no-go on that window would
have read NO-GO on item 1 alone; a go/no-go at close reads item 1 as committed
with residue. Both reads are recorded; only the close read governs.

## 1.3 Mutation court (mc4/mc5 + mutants.out) — item 2

- Latest sweep: `/tmp/mc4_213726.log` (started 21:37:26, last write 00:08):
  "Found 88 mutants", baseline ok (195s build + 349s test), then
  `scripts/mutation_court.sh: line 305: 55268 Killed: 9` — wrapper reports
  `cargo-mutants failed (exit 137)`; it restored mutated src/abi.rs and warned
  `git status (src/) changed during the court: M src/abi.rs` — the leak that
  put the json_depth shim into `b4283a3` (recovered by `16678de`).
- `mutants.out/outcomes.json`: `start_time 2026-10-24T04:55:55Z`, `end_time: null`,
  19 outcomes = baseline Success + 9 CaughtMutant + 9 Unviable → 18/88 mutants
  resolved. `mutants.out/missed.txt` = 0 lines — zero survived among resolved.
  The 4 gate-relevant `dispatch` mutants are NOT among the resolved 18 —
  the gate verdict on them is UNDETERMINED.
- Last wrapper invocation: `/tmp/mc5_run.log` (00:03): callsite-revert leg
  FLIPPED (exit 101, expected), restore byte-exact (sha256 5952cd04…bb07bbe),
  then `post-restore sanity FAILED: court does not pass on restored tree` —
  per `scripts/mutation_court.sh` that is the typed exit 4 path.
- In flight? No. `ps` at 01:11: no cargo-mutants / mutation_court process
  (the live cargo procs are the security lane's boundary rungs: `cargo test
  --lib duckdb_op`, `cargo test --test graphlaw_run_eh_court`, `cargo build
  --locked -p`, `cargo check --features runner-eh`). No mc6 log exists.
- Verdict: **FAIL/INCOMPLETE, not PENDING** — no process is running, so this
  is not "in flight"; the gate has no green verdict on the current source.

## 1.4 Artifact pin chain (item 3)

Converged DURING this lane to `58a44095…`, re-mint in flight at open:

```text
b3sum priv/graphlaw.wasm
  = 58a4409505f32b5d9aebc8c0ce2e2adeaba05c8c4b55f951a244ab4ad7d2280a (58,124,612 B)
priv/graphlaw.wasm.blake3        = 58a44095… (byte-identical)
registry/ARTIFACTS.blake3:13     = wasm 58a44095… 58124612 graphlaw_wasm.wasm
js/npm/src/generated/artifact-meta.ts:3 = '58a44095…'
```

- Internally consistent at close (4/4 surfaces, live b3sum this lane).
- **Uncommitted**: all four files are `M` vs `907e663` (HEAD still pins
  `472d5557…`); the re-pin is working-tree only.
- **Stale citations**: `docs/jira/v26.10.22/RELEASE-NOTES-26.10.22.md:38,43,71`
  still cite `332ea218…2c30c` as the current pin; TAG-PREP §3/§4 and the
  proposed tag message pin `472d5557…`. None mention `58a44095…`.
- The R-1 correspondence rungs on `58a44095…` (host-contract
  `cargo test -p graphlaw-wasm` + one anti-vacuity inversion on the
  wrap-enabled artifact, embedded in RECEIPTS.md) are not witnessed on disk.
  (The M fixtures/wasm-shim/glaw_wrap_shim.o suggests an inversion leg is
  being prepared, not landed.)

## 1.5 Battery (item 4)

- Last landed FULL battery: `/tmp/battery9.log` (2026-10-23 17:56:50):
  `dvp_settlement_court → test result: FAILED. 2 passed; 2 failed`
  (dvp_02 binder error, dvp_03 REFUSAL-01 multi-root).
- Post-fix green: `receipts/2026-10-23-banking-4of4-witness.md` —
  `CARGO_TARGET_DIR=target-lane-wit cargo test --test dvp_settlement_court`:
  `test result: ok. 4 passed; 0 failed` — after the INSERT pairing fix landed.
- No post-security-wave FULL battery receipt exists in /tmp or receipts/.
- Verdict: **PENDING, last landed evidence RED** (court-level green witness
  exists; the full battery re-run is owed on the committed wave).

## 1.6 eyeron 0.7.9 (item 5) — CLOSED

- `crates/graphlaw-eyeron/Cargo.toml:3` = `0.7.9`; `Cargo.toml:68` =
  `eyeron = { package = "graphlaw-eyeron", version = "0.7.9", path = ... }`.
  Both committed in `6b57840`. crates.io max_version 0.7.8 (TAG-PREP §2 live
  query) → 0.7.9 publishable, dependency order eyeron → graphlaw.

## 1.7 Compliance verdict + R-1/R-2 (item 6)

- Verdict file exists: `docs/jira/v26.10.23/compliance-final-verdict.md`
  (00:26): not-an-AI-system determination recorded; consolidated verdict
  release-ready conditional on discharging R-1 + R-2; §3 honestly records
  three hardening items OPEN-at-00:26 (frame bound, sterile env, fuel) —
  all three of those rows are now committed by `907e663`, so the verdict's
  §3 table itself is stale-as-of-commit and the rows flip receipt-gated.
- **R-1 OPEN** (worse than at v2): the new artifact `58a44095…` is uncommitted
  and cited nowhere; the release notes still cite superseded `332ea218…`;
  the two correspondence rungs are not witnessed on the new pin.
- **R-2 OPEN**: `RELEASE-NOTES-26.10.22.md:98-100` still asks for the
  operator decision; no dated accept/diagnose line exists anywhere on disk.

## Remaining items (discharge order)

1. Finish the wave landing: commit or revert the uncommitted residue on
   `907e663` (`M src/bin/graphlaw_run_eh.rs`, `M src/duckdb_op.rs`), then land
   the re-pin set (`priv/graphlaw.wasm` + `.blake3`, `registry/ARTIFACTS.blake3`,
   `js/npm/src/generated/artifact-meta.ts`, `docs/jira/v26.10.22/RECEIPTS.md`,
   `docs/reference/runner-exit-codes.md`) as one commit.
2. Boundary rungs first, then expand, on the committed state: `cargo check`;
   `cargo test --test law_state` (the mc5 sanity failure); `cargo test --lib
   duckdb_op`; `cargo test --test graphlaw_run_eh_court` (already running in
   the security lane); then `cargo test` full; then the sql-rdf acceptance set.
3. Mutation court to verdict: complete the 88/88 sweep with the tree committed
   and solo (mc4 was killed at exit 137; mc5's sanity leg needs a new
   hypothesis — the restored tree was red at 00:03, pre-wave); zero
   gate-relevant misses required. Serialize against all lanes (run solo).
4. Full battery re-run on the committed wave; dvp court must stay 4/0.
5. R-1: witness the two correspondence rungs on `58a44095…` with embedded
   output in RECEIPTS.md; truth-sync the release notes (replace `332ea218…`
   at :38/:43/:71) and TAG-PREP §3/§4 (replace `472d5557…`).
6. R-2: dated operator accept/diagnose line per residual deny advisory.
7. Register the verdict docs in `docs/jira/v26.10.23/INDEX.md` (missing on
   disk; ls exit 1 at 01:14).

## Operator command list (ONLY after items 1-6 discharge and re-verification)

```bash
# 1. tag + push -> npm OIDC auto-publish watch
git tag -a v26.10.22 -F <tag-msg-file>   # tag message re-derived: it pins 472d5557… today
git push origin v26.10.22
#    watch publish-npm.yml validate -> publish; cross-job blake3 equality gate
#    (.github/workflows/publish-npm.yml:196-203); confirm dist-tag + the
#    artifact-meta digest inside the published tarball.

# 2. crates.io, eyeron first (dependency order), then graphlaw
cargo publish -p graphlaw-eyeron   # 0.7.9; fails loud if dirty
cargo publish -p graphlaw
```

## Receipt fields

- **Identity**: read-only go/no-go lane over HEAD `907e66324e025a717649e3e19adfdfc6c13f7759`
  + working tree, read window 2026-10-24 00:45–01:15 PDT, under concurrent
  wave-landing edits (HEAD advanced 3 commits during the window).
- **Authority**: write lease = this file only (overwritten v2 → v3); no cargo,
  no git transitions, no publish executed by this lane.
- **Consequence**: one file overwritten (this one); zero code edits.
- **Replay** (each command re-run at use time, counts re-read):
  `b3sum priv/graphlaw.wasm && cat priv/graphlaw.wasm.blake3`;
  `grep -c leading_sql_token src/duckdb_op.rs`; `grep -c FrameOversize
  src/bin/graphlaw_run_eh.rs src/bin/graphlaw_run.rs`;
  `grep -n "\.inherit_env()" src/bin/graphlaw_run_eh.rs` (comment-only);
  `grep -c "consume_fuel\|FUEL_PER_REQUEST" src/bin/graphlaw_run_eh.rs`;
  `git log --all -S leading_sql_token --oneline`;
  `tail /tmp/mc4_213726.log /tmp/mc5_run.log`;
  `python3 -c "import json;o=json.load(open('mutants.out/outcomes.json'));print(o['caught'],o['missed'],o['unviable'],o['end_time'])"`;
  `wc -l mutants.out/missed.txt`; `tail -25 /tmp/battery9.log`;
  `head -12 receipts/2026-10-23-banking-4of4-witness.md`.
- **Falsifiers**: (a) a landed green receipt for the full host suite + battery
  on the committed wave → item-2/4 legs flip; (b) outcomes.json gaining
  `end_time` with missed=0 over 88/88 → the mutation leg flips to
  complete/clean; (c) the re-pin commit landing with all four surfaces +
  rungs witnessed → R-1 flips; (d) a dated deny-decision line → R-2 flips.
  Note the instability falsifier: any future read of a tree whose HEAD
  predates the wave commits must re-derive item 1 from scratch — this file's
  item-1 state is valid only for HEAD ≥ `907e663` with clean src residue.
- **Standing**: NO-GO as of HEAD `907e663`; every closure is receipt-gated
  and re-verified at use time.

## See Also

- `/Users/sac/graphlaw/docs/jira/v26.10.23/compliance-final-verdict.md`
- `/Users/sac/graphlaw/docs/jira/v26.10.23/compliance-gap-closure-plan.md`
- `/Users/sac/graphlaw/docs/jira/v26.10.23/TAG-PREP.md`
- `/Users/sac/graphlaw/docs/jira/v26.10.22/RELEASE-NOTES-26.10.22.md`
- `/Users/sac/graphlaw/receipts/2026-10-23-banking-4of4-witness.md`
- `/Users/sac/graphlaw/docs/jira/v26.10.22/designs/mutation-court-full-receipt.md`

---
## v4 TERMINAL STATE (2026-10-24, coordinator)

**Published-by-machine:** everything through the validate-and-pack green
run (40084544193 validate ✓: version triple, byte authority, npm 198/198
in CI, deterministic pack + drift check). Tag v26.10.22 at 669c817+.
**Typed-blocked on operator credentials (machine-impossible by design):**
1. npm OIDC publish → 404: Trusted Publisher for
   `@seanchatmangpt/graphlaw` is not configured on npmjs.com. Operator
   console action: npmjs.com → add Trusted Publisher (repo
   seanchatmangpt/graphlaw, workflow publish-npm.yml, environment npm),
   then `gh run rerun 40084544193`.
2. crates.io publish → 403 auth failed: no cargo token in the session.
   Operator: `cargo login <token>` then
   `cargo publish --locked -p graphlaw-eyeron && cargo publish --locked -p graphlaw`
   (0.7.9 then 26.10.22, in that order).
The workflow fixes landed en route (ABI-surface gate port, byte-authority
gate, scoped-tarball name + pack order + mkdir fixes, staging attempts
documented). The publish completes with those two operator acts.

---
## v5 CLOSEOUT GO/NO-GO (2026-10-24, closeout lane)

Closeout read over `/Users/sac/graphlaw` at HEAD `6da1d33` (dirty working
tree = in-flight wasmi retirement, disclosed and load-bearing; no cargo, no
git transitions, no publish executed by this lane). Every count and pointer
below was re-read from disk this session. This section appends only; v3's
four grounds, v4's terminal state, and PUBLISH-STATUS.md's live rows are not
rewritten by it.

## A. Ground ledger (v3 grounds, closeout state)

| # | Ground (v3 §1.x) | State at closeout | Evidence (re-read 2026-10-24) |
|---|---|---|---|
| G1 | Security wave + re-pin + pin-currency docs (v3 item 1/5) | **DISCHARGED** | Wave committed (`679299d` scanner, `907e663` EH mirror + ATTACH courts; v3 §1.1). Re-pin landed: all four surfaces agree live — `priv/graphlaw.wasm.blake3` = `registry/ARTIFACTS.blake3:13` = `js/npm/src/generated/artifact-meta.ts:3` = live `b3sum` = `58a44095…d2280a`, 58,124,612 B (re-run this session). Courts receipted: `wasm_frame_bound_court` 8/0, `attach_bypass_court` 3/0, `wasm_env_isolation_court` 3/0, `graphlaw_run_eh_court` 7/0 (`docs/reference/wasm-gap-register.md` 03:20 flip log, G4/SEC1/INC1 CLOSED). Fuel calibrated 10M red → 1B red → 100B green (`receipts/2026-10-24-fuel-calibration.md`). Pin-currency truth-sync landed (README / docs/WASM.md / SECURITY.md / ARCHITECTURE / js/npm README; historical pins demoted to marked sentences; the GWS-2 reproducible-SDK-rebuild marker stays PENDING — pin identity is not reproducibility). |
| G2 | Mutation-court verdict (v3 item 3) | **REMAINING** — owner: coordinator, solo window | Latest sweep died ENOSPC at build phase (2026-10-24 11:10:03Z; outcomes.json ABSENT, caught/missed empty, no process — not PENDING, INCOMPLETE). Expected receipt path: solo window with `git status --porcelain -- src/` empty; `bash scripts/mutation_court.sh` to verdict (exit ∈ {0,2}, zero gate-relevant misses, `outcomes.json` end_time set); durable `receipts/` file with kill counts. The 11 known `src/abi.rs` survivors (`SQLRDF-28.md:16-27`) should now KILL — the kill court `tests/abi_response_builders_court.rs` (10/0 witnessed, `b499fb7`) is committed in the tagged tree. Gate-relevant survivor (registry.rs / abi.rs dispatch / capabilities) → STOP-and-decide, never silent-accept. |
| G3 | Post-security-wave FULL battery receipt (v3 item 4) | **REMAINING** — owner: coordinator | Last landed FULL battery is RED (battery9: dvp 2/2 failed, fixed since by the INSERT pairing fix — banking 4/0 receipt exists on historical `f3570f15`); no battery receipt on the shipped pin `58a44095` exists. Expected receipt path: battery re-run with `GRAPHLAW_WASM=priv/graphlaw.wasm` in every invocation (no new artifact identity minted); dvp court must stay 4/0; per-item durable receipts + one consolidated pointer row flipping terminal-receipt §2 TRANSCRIPT → DURABLE. |
| G4 | Pre-tag gates R-1/R-2 (v3 item 5/6) | **SPLIT** | **R-2 DISCHARGED** — dated per-advisory operator decision: `docs/jira/v26.10.23/compliance-final-verdict.md:374-388` (3 advisories dispositioned, expiry 2026-11-23) + `deny.toml:10-20` dated ignores. **R-1 rungs REMAINING** — host-contract `cargo test -p graphlaw-wasm` (no SDK needed; host-target build) + one anti-vacuity inversion on the wrap-enabled artifact (admit exit 0 → mutated /tmp copy exit 84 with got == b3sum(mutated) → restore proven), embedded in RECEIPTS.md. Same rungs close SQLRDF-27's owed leg — one execution, two ledgers. |

## A.1 Terminal-addendum disposition (v4)

v4's TERMINAL STATE recorded the publish pipeline machine-green through
validate-and-pack (run 40084544193) with the two operator credential acts
outstanding. That state is NOT restated here: the live publish-state record
is [`PUBLISH-STATUS.md`](PUBLISH-STATUS.md) (status table + falsifiers keyed
to its six rows). Where v4 prose and PUBLISH-STATUS disagree, PUBLISH-STATUS
governs — it is the queried-at-use-time locus.

## B. Posture verdict block (fill-in)

Closeout standing: every machine gate is green or typed-blocked-on-operator
except G2/G3/R-1, which posture P carries as non-blocking follow-up receipts.
The operator picks the posture; the lane files evidence, never the decision.

- **POSTURE**: ______ (P = finish v26.10.22 from tag `29c6d98`: publish once
  the credential acts land, gate receipts G2/G3/R-1 land as immediate
  follow-ups; G = G2/G3/R-1 receipts land BEFORE any publish)
- **DATE**: __________
- **Vacating falsifier**: any S0 pre-flight value in
  [`PUBLISH-STATUS.md`](PUBLISH-STATUS.md) flipping before the operator acts
  (npm dist-tag, crates.io max_version, run 40084544193 conclusion, tag ref,
  4-surface pin) re-opens this section — the verdict is valid only for the
  state it was read on. A G2/G3/R-1 receipt landing under posture P does not
  flip this block; it discharges its ledger row in §A.

## C. Receipt fields

- **Identity**: closeout read over `/Users/sac/graphlaw` HEAD `6da1d33`
  (dirty worktree: wasmi retirement in flight, disclosed), read window
  2026-10-24; subject = v3's four go/no-go grounds at closeout.
- **Authority**: write lease = this file, APPEND ONLY (v3/v4 bodies
  untouched); no cargo, no git transitions, no publish executed by this
  lane.
- **Consequence**: one append to this file; the §A table, §A.1 pointer, §B
  fill-in block, §C, §D. No code, no registry, no receipts/ writes.
- **Replay** (re-read at use time):
  `b3sum priv/graphlaw.wasm && cat priv/graphlaw.wasm.blake3`;
  `sed -n '13p' registry/ARTIFACTS.blake3`; `sed -n '3p'
  js/npm/src/generated/artifact-meta.ts`;
  `grep -c "FrameOversize" src/bin/graphlaw_run_eh.rs`;
  `sed -n '103,160p' docs/reference/wasm-gap-register.md`;
  `ls receipts/2026-10-24-fuel-calibration.md`;
  `sed -n '374,388p' docs/jira/v26.10.23/compliance-final-verdict.md`;
  `python3 -c "import json;print(json.load(open('mutants.out/outcomes.json')).get('end_time'))"`
  (expect absent/error until G2 closes);
  `docs/jira/v26.10.23/PUBLISH-STATUS.md` status table re-read.
- **Falsifiers**: (a) outcomes.json gaining `end_time` with zero gated
  misses → G2 flips DISCHARGED; (b) a durable battery receipt on pin
  `58a44095` → G3 flips; (c) R-1's two rungs with embedded output in
  RECEIPTS.md → the R-1 half of G4 flips; (d) any live value contradicting
  a §A evidence cell invalidates that cell — re-derive from disk.
- **Standing**: closeout read as of 2026-10-24, HEAD `6da1d33`; G1
  DISCHARGED on observed evidence, G2/G3/R-1 REMAINING receipt-gated;
  publish standing lives in PUBLISH-STATUS.md and moves with its own
  falsifiers.

## D. See Also

- `/Users/sac/graphlaw/docs/jira/v26.10.23/PUBLISH-STATUS.md`
- `/Users/sac/graphlaw/docs/jira/v26.10.23/OPERATOR-RUNBOOK-v26.10.22-PUBLISH.md`
- `/Users/sac/graphlaw/docs/jira/v26.10.22/designs/OPERATOR-DECISIONS.md`
- `/Users/sac/graphlaw/docs/jira/v26.10.23/compliance-final-verdict.md`
- `/Users/sac/graphlaw/docs/reference/wasm-gap-register.md` (03:20 flip log)
- `/Users/sac/graphlaw/receipts/2026-10-24-fuel-calibration.md`
- `/Users/sac/graphlaw/docs/jira/v26.10.23/ADR-wasmi-retirement.md`

---
## A.2 Battery-ledger addendum (v6, 2026-10-24)

Battery-receipts execution lane; APPEND ONLY (v3 grounds, v4 terminal state,
v5 §A/§A.1/§B/§C/§D bodies untouched; no cargo, no git transitions by this
lane). Every number below is spliced or recomputed from the four b3sum-digested
battery logs at
`receipts/2026-10-24-battery-full.md` (b3sum
`7fa6e1489e6a4a5da358fd471189d3c2a442650ba2141e5d5289e15a11fccbaf`).
Headline: **G3 goes REMAINING → SPLIT on the G4 precedent.**

### G3 SPLIT

| Half | Scope | State | Evidence (b3sum-digested logs, 2026-10-24) |
|---|---|---|---|
| DISCHARGED | L1 sqlrdf (8 targets) | **DISCHARGED** | 102 passed / 0 failed (`/tmp/bat-L1.log` b3sum `7dec1d1f…29b3`), incl. the sql-rdf statement-kind census **36/0** (`sql_rdf_dialect` — the honest census; "69/0" is witnessed nowhere in the logs) |
| DISCHARGED | L2 chicago (5 targets) | **DISCHARGED** | 154/0 (`/tmp/bat-L2.log` b3sum `5d533716…2029`) |
| DISCHARGED | L3 meta (15 targets) | **DISCHARGED** | 114/0 (`/tmp/bat-L3.log` b3sum `5f69751d…192d`), incl. `receipt_v2_court` 6/0, `receipt_v2_emission_court` 8/0, `trust_attest` 12/0 (Ed25519 attestation surface; Sigma_PQC §1.4 second formula stays UNMET and is quoted, not instantiated) |
| DISCHARGED | dvp gating leg | **DISCHARGED** | `dvp_settlement_court` **4/0** (v5 G3's condition "dvp court must stay 4/0" holds; battery9's 2/2 RED closed on the shipped pin `58a44095…` — `receipts/2026-10-24-battery-banking.md`) |
| REMAINING | op_differential | **CLASSIFIED-RED** | 15 passed / 1 failed — `sniff_dialect_agrees_with_parse_dialect_for_every_serialization` panicked at `tests/op_differential.rs:181:9`: native sniffs ntriples (`{"dialect":"NTriples",…}`) vs wasm sniffs Turtle on the same bytes — G8-class sniff differential, **fresh witness for `docs/jira/v26.10.23/design-sniff-fixes.md`**; the sniffer is not runtime-deterministic on identical input today |
| REMAINING | 4 fail-fast-unexecuted courts | **SKIP-UNEXECUTED** | `op_examples`, `op_forward_compat`, `rehypothecation_gate_court`, `ubo_sanctions_court` — compiled in-leg (10 test targets, `target-battery-finance` build dir), never executed behind the `op_differential` failure. Count note: the dispatch said "5 fail-fast-unexecuted"; the log + build dir give **4 unexecuted**, and op_differential itself RED makes **5 courts without a green verdict** — both recorded so the ledger does not drift |
| REMAINING | L5/L6 legs | **NOT RUN** | No logs, no receipts; their receipts flip the remaining legs |

Capture-masking disclosure: the battery driver's shell wrapper echoed success
unconditionally, so every leg's observed shell exit code is 0 regardless of
cargo's outcome — the captured logs at the digests above are the evidence of
record, not the leg exit codes.

### G2 restated

**REMAINING (unchanged), owner: coordinator, solo window** — with the config
precondition: `abi_response_builders_court` is missing from the mutation
kill suite (E07 lands the fix); the G2 sweep must run on a tree with that fix
committed. The battery legs did not run that court; no battery receipt moves
G2.

### frame_bound currency note

The v5 §A G1 row cites `wasm_frame_bound_court` **8/0**. The court on disk
carries **4** `#[test]` functions today — the 8/0 figure is
historical-at-write-time (pre wasmi-retirement split, `1e8ec32`), not
re-witnessed by this battery. Cite 4/0-current basis until a fresh receipt
witnesses it on the current HEAD.

### Falsifiers (a)–(d)

- (a) A green rerun of `cargo test --locked --test op_differential`
  (16 passed; 0 failed, sniff-parity) on the shipped pin flips the
  op_differential row CLASSIFIED-RED → DISCHARGED; a rerun failing on a
  different test extends the classification instead.
- (b) L5/L6 leg receipts with embedded `test result:` lines flip the
  remaining legs from NOT RUN.
- (c) Any court in the spliced tables returning a non-`ok` line on a rerun of
  the recorded HEAD + pin `58a44095…` falsifies that row's PASS.
- (d) A sniff fix landing WITHOUT (a) green is not a fix; the
  design-sniff-fixes.md list shrinks only via `tests/sniff_court.rs` + the
  differential going green.

### §C receipt fields (this addendum)

- **Identity**: append-only read of the 2026-10-24 battery over HEAD
  `966cec3` → `a00770b4998a55ed4ee397e3e2a1f5a880c51db0` (moving tree,
  disclosed), logs captured 11:15–11:35 local on the shipped artifact
  identity `58a44095…` (58,124,612 B).
- **Authority**: write lease = `receipts/2026-10-24-battery-*.md` (5 files,
  this lane) + these ledger appends; no git mutations; no chain sync (the
  coordinator runs it at integration).
- **Consequence**: this append + 5 durable receipts + pointer rows in
  `docs/jira/v26.10.22/RECEIPTS.md` and `RECEIPTS-INDEX.md`; zero code edits;
  no register status hand-edited anywhere.
- **Replay**: `b3sum /tmp/bat-L1.log /tmp/bat-L2.log /tmp/bat-L3.log
  /tmp/bat-L4.log` must reproduce the digests in
  `receipts/2026-10-24-battery-full.md`; reruns named in that receipt's §C.
- **Standing**: posture P unchanged; G3 SPLIT as recorded; the operator
  decision block (§B) is not moved by this addendum.

Receipts of record (this lane, 5): `receipts/2026-10-24-battery-dialect-census.md`
(b3sum `91cee82a…a444c`), `receipts/2026-10-24-battery-receipt-v2.md`
(b3sum `7ce1dfc7…685f`), `receipts/2026-10-24-battery-trust-attest.md`
(b3sum `dfdb8091…67fc`), `receipts/2026-10-24-battery-banking.md`
(b3sum `99cce2e4…c59d`), `receipts/2026-10-24-battery-full.md`
(b3sum `7fa6e148…cbaf`). Two sibling battery receipts
(`battery-pytest-oracle`, `battery-wasm-eh`) appeared on disk mid-lane under
the same `battery-*` prefix; they are another lane's writes and are recorded
here as observed-on-disk only, no claims made.
