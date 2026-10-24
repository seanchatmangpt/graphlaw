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
