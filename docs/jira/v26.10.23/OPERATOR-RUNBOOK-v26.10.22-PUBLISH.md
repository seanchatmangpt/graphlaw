# OPERATOR-RUNBOOK — v26.10.22 publish (S0–S6)

Subject tree: `/Users/sac/graphlaw` (canonical checkout; HEAD `6da1d33`,
dirty worktree = in-flight wasmi retirement, disclosed and load-bearing).
Window: `docs/jira/v26.10.23/`. Authority for the publish posture: operator
publish directive 2026-10-24 (`compliance-final-verdict.md:388`); live
publish-state record: [`PUBLISH-STATUS.md`](PUBLISH-STATUS.md) — this
runbook is its execution half.

**What this doc is**: a runbook, not a verdict. It flips no register, signs
no decision, publishes nothing. The operator executes S1–S6; the lane that
authored it embedded only commands whose output it captured this session
(2026-10-24) and cites only receipts that exist on disk at write time. Every
live value is stamped as-of and carries the falsifier "re-verify at use
time" — if an S0 row has already flipped when you read this, the act it
gates is DONE: keep the step, re-derive its expected values from live state,
and mark it DISCHARGED-as-executed with its witness.

**Lane lease for this file**: this file + one registration row in
`INDEX.md`. No cargo, no git transitions, no publish executed by the author
lane.

## What the machine already proved (do not re-litigate; re-verify pointers)

- **validate-and-pack GREEN** (job 1, run 40084544193, head `29c6d98`,
  window 10:05:43–10:07:25Z): version triple gate (npm `26.10.22` / eyeron
  `0.7.9` / graphlaw `26.10.22`), byte-authority gate on the committed
  artifact (`publish-npm.yml:104-116`), npm 198/198 in CI, deterministic
  pack + drift check (pack mkdir fix in `29c6d98`). Packed tarball from the
  run log: `@seanchatmangpt/graphlaw@26.10.22`, 137 files, 12.6 MB packed /
  58.4 MB unpacked, **shasum `115e5024840955e741c788edcacf3358d2ff277a`**,
  integrity `sha512-SlFgJxGWAxbtM[…HgL4crLlZYS9w==]`, includes
  `wasm/graphlaw.wasm` (58.1 MB).
- **Cross-job blake3 gate passed** — the publish job re-verified the tarball
  blake3 (`publish-npm.yml:198-203`) BEFORE the registry `PUT`; the E404 is
  downstream of every machine gate.
- **The E404 is machine-impossible by design**: OIDC trusted publishing
  carries no token in the runner (`publish-npm.yml:161-170`); npm answers
  the `PUT` with 404 when no Trusted Publisher configuration matches the
  package. Log tail captured in `PUBLISH-STATUS.md` ("The blocker").
- **Tag** `v26.10.22` on origin at `29c6d98`; live tree == tagged tree for
  the pin set (`git show` verified by the final-tree-audit lane).
- **4-surface pin agreement** (re-run by this lane, 2026-10-24, real output):

  ```text
  $ b3sum priv/graphlaw.wasm
  58a4409505f32b5d9aebc8c0ce2e2adeaba05c8c4b55f951a244ab4ad7d2280a  priv/graphlaw.wasm
  $ cat priv/graphlaw.wasm.blake3
  58a4409505f32b5d9aebc8c0ce2e2adeaba05c8c4b55f951a244ab4ad7d2280a  priv/graphlaw.wasm
  $ sed -n '13p' registry/ARTIFACTS.blake3
  wasm 58a4409505f32b5d9aebc8c0ce2e2adeaba05c8c4b55f951a244ab4ad7d2280a 58124612 graphlaw_wasm.wasm
  $ sed -n '3p' js/npm/src/generated/artifact-meta.ts
  export const WASM_BLAKE3 = '58a4409505f32b5d9aebc8c0ce2e2adeaba05c8c4b55f951a244ab4ad7d2280a';
  ```

  Size 58,124,612 B. TAG-PREP §3/§4 (`472d5557…`) is stale as a pin claim —
  the shipped pin is `58a44095…` (see TAG-PREP truth-sync residue, GO-NO-GO
  v5 §A row G1).
- **crates.io dry-pack precedent** (W61): 360 files, 143.9 MiB
  (7.4 MiB compressed); the packaging include-fix landed in `d97da98`.
- **RELEASE_HOLD**: unset (Actions API, PUBLISH-STATUS-era query); chain
  `--check` exit 0 (1,927 receipts, head `31ffac67…` at the terminal-receipt
  read). Re-verify both at use.
- **`abi_response_builders_court.rs` (10 kill tests, `b499fb7`) is in the
  tagged tree** — a mutation re-sweep should kill the 11 `src/abi.rs`
  survivors (`SQLRDF-28.md:16-27`).

## Global laws (apply to every step)

1. **Tag immutable** — never re-tag, never force-push; merge only, fix
   forward (CLAUDE.md). A fix that cannot ride tag `v26.10.22` is a new
   release decision, not a fix forward.
2. **One canonical checkout** (CLAUDE.md:108) — S2 stages from
   `git archive`, never a clone or worktree.
3. **Mutation court runs solo** (CLAUDE.md:94-96) — S5 serializes against
   ALL lanes; its inversions false-red concurrent compiles.
4. **Disk ≥ 20 GiB** (CLAUDE.md:88) — the 2026-10-24 11:10:03Z mutation
   sweep died ENOSPC; pre-flight gate 6 catches it.
5. **No sleeps** — bounded polls on observables only.
6. **A loud SKIP is not a pass**; a gate that cannot exit 1 is the failure.

## S0 — Pre-flight (all steps; re-run the whole table before acting)

| # | Gate | Command | Expected (as-of 2026-10-24) | Typed failure → action |
|---|---|---|---|---|
| 1 | RELEASE_HOLD unset | `gh variable list -R seanchatmangpt/graphlaw` | no RELEASE_HOLD row | HOLD=true → STOP (coordinator release-lease act) |
| 2 | Receipt chain | `bash scripts/receipt_chain_sync.sh --check` | VERIFIED, exit 0 | ≠0 → STOP (coordinator resync) |
| 3 | Tag on origin | `gh api repos/seanchatmangpt/graphlaw/git/ref/tags/v26.10.22` | `29c6d98…` | missing/different → STOP (re-derive from GO-NO-GO v5) |
| 4 | Registry pre-state | see live query below | npm `26.10.14` / eyeron `0.7.8` / graphlaw `26.10.21` | already published → that act is DONE; skip it |
| 5 | Run state | `gh run view 40084544193 -R seanchatmangpt/graphlaw --json conclusion,jobs` | validate `success` / publish `failure` | rerun in flight → wait on observables, never double-fire |
| 6 | Disk | `df -h /` | ≥ 20 GiB free (97 GiB observed 2026-10-24) | <20 → STOP S2/S5; coordinator frees lane roots |
| 7 | Worktree disclosed | `git rev-parse --short HEAD; git status --porcelain \| wc -l` | `6da1d33`+, dirty | dirty is NOT an S1 blocker — it is the reason S2 uses scratch |
| 8 | Pin 4-surface | the four pin commands in "What the machine already proved" | `58a44095…2280a` ×4, 58,124,612 B | mismatch → STOP (release identity broken) |
| 9 | src/ clean (S5 only) | `git status --porcelain -- src/` | empty | dirt → S5 refuses typed (exit 6) |

Gate-4 live query, real output embedded (this lane, 2026-10-24):

```text
$ npm view @seanchatmangpt/graphlaw version dist-tags.latest
version = '26.10.14'
dist-tags.latest = '26.10.14'
$ curl -s -H 'User-Agent: graphlaw-status' https://crates.io/api/v1/crates/graphlaw-eyeron | jq -r .crate.max_version
0.7.8
$ curl -s -H 'User-Agent: graphlaw-status' https://crates.io/api/v1/crates/graphlaw | jq -r .crate.max_version
26.10.21
```

## S1 — Act 1: npm Trusted Publisher + re-fire (10–25 min; publish-critical)

Zero repo mutation. Unblocks the only red run.

**(a) Actions.** Console action — there is no CLI for Trusted Publisher:
npmjs.com → package `@seanchatmangpt/graphlaw` → Settings → Trusted
Publisher, exactly:

- repository: `seanchatmangpt/graphlaw`
- workflow: `publish-npm.yml`
- environment: `npm`

(The GitHub `npm` environment already exists — the publish job reached its
registry `PUT`, so `publish-npm.yml:167` resolved. The TP config is not
validated on save and expires ~2 days unused: configure, then re-fire
immediately.)

Then:

```bash
gh run rerun 40084544193 -R seanchatmangpt/graphlaw --failed
```

`--failed` re-runs only the publish job; the banked validate green and its
outputs are reused as needs-outputs.

**(b) Expected.** Publish job green; run conclusion `success`.

**(c) Verify** (allow ~1 min propagation):

```bash
gh run view 40084544193 -R seanchatmangpt/graphlaw --json conclusion,jobs \
  --jq '{conclusion, jobs: [.jobs[] | {name, conclusion}]}'
#   → conclusion "success", both jobs success
npm view @seanchatmangpt/graphlaw version dist-tags.latest
#   → 26.10.22 / 26.10.22
```

**(d) Typed failures.**

| Symptom | Meaning | Action |
|---|---|---|
| E404 again | TP not yet visible on npm's side | wait ~60 s (bounded), rerun once; on a 3rd E404 re-check the three TP fields byte-exact; still E404 → **STOP** (npm-side investigation; the machine part is green) |
| publish job "waiting" | environment has required reviewers | approve the deployment (or drop the reviewer rule) — config act, allowed |
| any NEW failure class | not E404 → a workflow change would be needed | **STOP** — no workflow change can ride tag `v26.10.22` (immutable); escalate to coordinator (new tag = new release decision) |

STOP criteria: STOP on any new failure class; fix-forward is lawful only for
the config/rerun actions themselves.

## S2 — Act 2: crates.io login + publish eyeron → graphlaw FROM THE TAG (60–120 min; publish-critical)

**(a) Actions.**

```bash
cargo login <token>        # crates.io API token with publish scope; writes
                           # ~/.cargo/credentials.toml. CI never holds creds
                           # (release.yml:8-12; the CI publish step is a typed refusal).

# Scratch staging per the one-canonical-checkout law — never publish from the
# dirty worktree (it would ship a manifest differing from the tag):
SCRATCH=$(mktemp -d /tmp/gl-publish-v26.10.22.XXXXXX)
git archive v26.10.22 | tar -x -C "$SCRATCH"
cd "$SCRATCH"

# In-scratch gates:
grep -n '^version' Cargo.toml crates/graphlaw-eyeron/Cargo.toml
#   → 26.10.22 / 0.7.9
test ! -d .git && echo "archive ok (no .git)"
ls vendor/ >/dev/null && echo "vendor present"

cargo publish --locked -p graphlaw-eyeron     # fast
cargo publish --locked -p graphlaw            # verify stage cold-builds
                                              # frozen-duckdb: 20–60 min. Keep
                                              # the terminal open; do not abort
                                              # on silence.
```

**(b) Expected.** `Packaging graphlaw-eyeron v0.7.9` → `Uploaded
graphlaw-eyeron v0.7.9`; then `Packaging graphlaw v26.10.22` → `Verifying…`
→ `Uploaded graphlaw v26.10.22`.

**(c) Verify** (~60 s index propagation between acts):

```bash
curl -s -H 'User-Agent: graphlaw-status' https://crates.io/api/v1/crates/graphlaw-eyeron | jq -r .crate.max_version   # 0.7.9
curl -s -H 'User-Agent: graphlaw-status' https://crates.io/api/v1/crates/graphlaw | jq -r .crate.max_version           # 26.10.22
rm -rf "$SCRATCH"
```

**(d) Typed failures.**

| Symptom | Meaning | Action |
|---|---|---|
| `no matching version for graphlaw-eyeron 0.7.9` during the graphlaw publish | index propagation lag | retry once after ~60 s; second miss → STOP |
| 403 from registry | token lacks publish scope | fix the credential; never re-fire blind |
| `failed to verify package` in the verify stage | classify: environmental vs structural | environmental (toolchain) → fix env, re-run. Structural include-list gap (the packaged crate ships no `/tests`; first-ever verify on this crate — W61 used `--no-verify`) → `cargo publish --locked --no-verify -p graphlaw` is a disclosed-precedent operator fallback (W61); any OTHER structural miss → **STOP** (include change = new version = new tag decision) |
| dirty/manifest complaint | scratch was wrong | **STOP** (re-stage the archive) |

Cleanup: `rm -rf "$SCRATCH"` on BOTH success and failure paths.

## S3 — OD-4 letter (10 min; NON-blocking)

**(a) Action.** Read `receipts/2026-10-24-spc-solo-rerun.md` (Cpk 0.632
honest FAIL; p99 167 ns in spec; mean 128.2, sd 64.24; 283 rule-1 samples)
and the evidence memo already filed for you at
`docs/jira/v26.10.22/designs/OPERATOR-DECISIONS.md` (OD-4 section: evidence
table incl. the W57 quiet-machine PASS `receipts/spc-2026-10-23T035422.json`
Cpk 1.7573 @ sd 23.3 ns; root cause = timer quantization + IRQ shoulder;
options (a)/(b)/(c) with the falsifier "two consecutive quiet FULL FAILs at
sd 45-70 ns w/ p99 ≤ 167 ns → (b) unreachable → (c)"). Fill the OD-4
DECISION line (the `(a)/(b)/(c): ______` + `date:` line — the signing
surface) and the summary-table OD-4 row.

**(b) Expected.** DECISION line non-blank; summary row updated.

**(c) Verify.** `sed -n '198p'` of OPERATOR-DECISIONS.md no longer blank;
summary row reflects the letter.

**(d) Typed failure.** The lane that filed the memo NEVER authors the
decision (register head note: "nothing decided by this lane"). An empty
letter at use time = the act is still owed.

## S4 — OD-6 signature repair (10 min; NON-blocking)

**(a) Action.** OD-6's decision line was corrupted (filled with OD-1 text in
`0d11bc3`); the anomaly is recorded and the block re-presented at
`docs/jira/v26.10.22/designs/OPERATOR-DECISIONS.md` (OD-6 section) with
BLANK decision/date/signature. Option (a) is de-facto executed —
`.claude/settings.json`, `.claude/hooks/disk_guard.py`,
`.claude/hooks/manifest_guard.py` are tracked (`git ls-files .claude/`; the
`.claude/` commit landed at `dab6a73`) and `__pycache__/` is ignored
(`.gitignore:39`). Sign the re-presented block (option (a) recorded-as-executed)
and flip the summary row.

**(b) Expected.** Decision line signed; summary row no longer "signature
pending".

**(c) Verify.** The corrupted OD-1 text is bracketed by the ANOMALY RECORD
banner; `git ls-files .claude/` lists the three files; summary row updated.

**(d) Typed failure.** None machine-side; the docs lane never authors the
signature.

## S5 — Mutation sweep to verdict + kill-or-accept (3–4.5 h; NON-blocking, SOLO)

**(a) Precondition + run.** `git status --porcelain -- src/` empty (it is);
`df -h /` ≥ 20 GiB (97 GiB observed 2026-10-24; the 11:10:03Z sweep died
ENOSPC — disclose in the receipt); solo against ALL lanes
(CLAUDE.md:94-96).

```bash
CARGO_TARGET_DIR=target-lane-mut9 bash scripts/mutation_court.sh --check --log /tmp/mc_v26.10.22.log
# then the sweep itself, per scripts/mutation_court.sh usage
```

Contract (`scripts/mutation_court.sh:4-36`): exit 0/2 = pass or no
gate-relevant miss; 3 = gate-relevant MISSED; 4 = restore mismatch;
6 = dirty-src refusal; 7 = vacuous leg.

**(b) Expected.** 88 mutants (all `src/abi.rs`; registry.rs has zero
mutants — disclose that the registry clause of the gate is vacuous under
this scope); sweep completes; `mutants.out/outcomes.json` end_time set. The
11 known survivors should now KILL —
`tests/abi_response_builders_court.rs` (10/0 witnessed, `b499fb7`) exists
precisely for them.

**(c) Verify + receipt.** `wc -l mutants.out/missed.txt` (expect 0);
`outcomes.json` end_time non-null; write the durable `receipts/` file with
kill counts (closes SQLRDF-28, residual rows 2-3, R-7).

**(d) Typed failures.**

- The 11 reproduce (surprise — the kill court is in the tree): per-survivor
  operator call — kill (new test) or accept with operator sign-off — recorded
  in a RECEIPTS.md append; gate-relevant survivors (registry.rs, abi.rs
  dispatch/capabilities) → **STOP-and-decide**, never silent-accept.
- ENOSPC (os error 28) → STOP, free space, re-run (typed failure, twice
  witnessed).
- Never SIGKILL the sweep (TERM + bounded poll; leaked mutated files cause
  the mc5-class exit-4).

## S6 — Post-act verification + receipt follow-ups (15 min + owed)

**(a) Re-query every PUBLISH-STATUS row**: tag ref (row 1); npm
version/dist-tags (row 2); crates.io max_version ×2 (rows 3-4); run 40084544193
conclusion (row 6). GitHub release assets: `release.yml` runs on the next
push to MAIN, not the tag (`release-publish.md:57-62`) — a green run with
`work=no` is NORMAL (existing-tag skip), not a skipped gate. Release-asset
sidecars digest the CI rebuild: record and compare against pin `58a44095…`
(TAG-PREP §5.3 watch item; N9 caveat).

**(b) Gate receipts that FOLLOW the publish (posture P, non-blocking)**:
R-1 rungs (`cargo test -p graphlaw-wasm` host-contract + one anti-vacuity
inversion on the wrap-enabled artifact — admit exit 0 → mutated `/tmp` copy
exit 84 with got == b3sum(mutated) → restore proven) with embedded output in
RECEIPTS.md; R-21 transcript→durable receipts; R-5 FULL-mode SPC rung; R-22
closes with the real publish itself.

**(c) Expected.** Registry surfaces green; receipts appended.

**(d) Typed failure.** Any re-query disagreeing with a PUBLISH-STATUS row →
the falsifier fires; re-derive from live state before citing.

## Cross-reference map (no act dropped or invented)

| Runbook act | Source sections (cite these) |
|---|---|
| S1 npm act | PUBLISH-STATUS "Operator acts" Act 1 + falsifier (a); GO-NO-GO v4 items 1-2; release-publish.md "Name resolution" + "One-time setup" 1-2; TAG-PREP §5.1 |
| S2 crates act | PUBLISH-STATUS "Operator acts" Act 2 + falsifier (b); terminal receipt §5.3; RELEASE-NOTES checklist item 1; GO-NO-GO §1.6; TAG-PREP §2/§5.2; release-publish.md "Per-release order" |
| S3 OD-4 | OPERATOR-DECISIONS.md OD-4 memo (evidence table, root cause, options, falsifier) + summary row; `receipts/2026-10-24-spc-solo-rerun.md`; `receipts/spc-2026-10-23T035422.json`; residual-unknowns row 4; compliance R-5 |
| S4 OD-6 repair | OPERATOR-DECISIONS.md OD-6 anomaly record + re-presented block + summary row; compliance R-25; live tracking state (`git ls-files .claude/`) |
| S5 mutation | terminal receipt §4 + §5.5; SQLRDF-28 (11 survivors + kill court); residual rows 2-3; compliance R-7; `scripts/mutation_court.sh:4-36` |
| S6 follow-ups | compliance §5 list (R-1 rungs, R-5, R-7, R-21, R-22); residual rows 5/6/9, N9-N11; TAG-PREP §5.3; GO-NO-GO v5 §A G2/G3/G4 |
| NOT an act | R-2 DISCHARGED (`compliance-final-verdict.md:374-388` + `deny.toml:10-20`, expiry 2026-11-23) — pre-flight INFO only; OD-1/2/3/5 ADMITTED/SUPERSEDED (OPERATOR-DECISIONS summary); OD-6 action executed, signature only |
| Explicitly out of scope | re-tagging; workflow edits riding v26.10.22; docs/how-to edits from this runbook; any lane-authored decision text |

## Time budget

- Publish-critical path S0 → S1 → S2: **~1.25–2.5 h**.
- S3/S4: 20 min (parallel window, non-blocking).
- S5: 3–4.5 h solo window (schedule separately; disk + solo preconditions).
- S6: 15 min after S1/S2.

## Receipt fields

- **Identity**: `/Users/sac/graphlaw` HEAD `6da1d33` (dirty worktree:
  in-flight wasmi retirement, disclosed); authored 2026-10-24 by the
  closeout docs lane; subject = the v26.10.22 publish acts S0–S6.
- **Authority**: write lease = this file + one INDEX.md registration row; no
  cargo, no git transitions, no publish executed by the author lane; the
  operator directive (`compliance-final-verdict.md:388`) is the release
  authority.
- **Consequence**: one new file; one registration row; flips no register,
  signs no decision, runs no publish.
- **Replay**: every command in S0–S6 is re-runnable at use time; the
  embedded outputs (pin 4-surface block, gate-4 registry queries, disk) were
  captured live on 2026-10-24 and are stamped as-of — re-verify at use.
- **Falsifiers** (keyed to PUBLISH-STATUS rows 1-6): (a) `npm view` →
  `26.10.22` discharges S1 (mark DISCHARGED-as-executed with the receipt);
  (b) crates.io max_version → `0.7.9`/`26.10.22` discharges S2; (c) run
  conclusion `success` discharges the S1 re-fire; (d) tag ref ≠ `29c6d98`
  or pin ≠ `58a44095…` invalidates this runbook's identity block — STOP and
  re-derive from GO-NO-GO v5.
- **Standing**: runbook ALIVE as of 2026-10-24; each step is owed until its
  receipt exists; the doc flips nothing and says so.
