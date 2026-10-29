# PUBLISH-COMPLETION-RUNBOOK — v26.10.22 completion + v26.10.24 preview

Execution doc for the P16 plan §§2–8. Subject surfaces: npm
`@seanchatmangpt/graphlaw@26.10.22`, crates.io `graphlaw-eyeron 0.7.9` /
`graphlaw 26.10.22`, tag `v26.10.22` (`29c6d98`), CI run `40084544193`.
Window: 2026-10-24. This is the completion-protocol layer between two
existing docs:

- [`PUBLISH-STATUS.md`](PUBLISH-STATUS.md) — the live six-row status table;
  it **governs** (queried-at-use-time locus).
- [`OPERATOR-RUNBOOK-v26.10.22-PUBLISH.md`](OPERATOR-RUNBOOK-v26.10.22-PUBLISH.md)
  — the instance-bound S0–S6 act scripts; this doc's §A.2/§B.2 add the
  retry/verification protocol around S1/S2, not a replacement of them.

Machine-prep receipts this doc rests on (both landed 2026-10-24, real
output embedded):

- `receipts/2026-10-24-npm-attempt-ledger.md` — attempts A0/A1/A2 +
  tarball identity (`shasum 115e5024…`, 137 files, wasm-inside-tarball
  b3sum == pin `58a44095…`).
- `receipts/2026-10-24-crates-scratch-prep-26.10.22.md` — scratch
  `/tmp/gl-pub-26.10.22.1dfLUa` EXACT 4-file delta vs tag, in-scratch gates,
  eyeron dry-run exit 0.

## §1 Stage classes

Every stage row below is typed as exactly one of:

- **machine-prep** — done or doable by a lane without registry credentials
  (verification, receipts, doc fixes). Never includes a registry write.
- **operator-act** — requires credentials/console access this fleet must
  not hold (npmjs.com login, crates.io token). Typed-blocked for lanes by
  design; the runbook's job is to make each act small, ordered, and
  falsifiable.
- **machine-verification** — post-act read-only re-query of the registry
  state (the falsifier leg).
- **receipt** — durable file writes; ordering constraints in §B.4.

## §2 Master stage table (keyed to PUBLISH-STATUS rows)

| Stage | Surface | PUBLISH-STATUS rows | machine-prep | operator-act | machine-verification | receipt |
|---|---|---|---|---|---|---|
| **A** | npm publish | 2, 5, 6 | attempt ledger + tarball identity witness; PUBLISHING.md 5 fixes (landed); this §A protocol | TP config + refire (§A.1–A.2) | §A.3 post-success checks incl. install smoke | ledger A3 flip + PUBLISH-STATUS addendum (§B.4 step 1) |
| **B** | crates.io | 3, 4 | scratch receipt (dry-run green; publishes only FROM the scratch) | §B.2 login + publish eyeron → graphlaw | §B.3 smokes + registry re-query | §B.4 closure (addendum → GO-NO-GO pointer → RECEIPTS-INDEX → chain sync LAST) |
| **C** | v26.10.24 preview | none (new surface) | §C gates (fresh-scratch ZERO-delta, MSRV, TP clock, package --list greps) | none | the §C gates themselves | §C checklist output → receipts/ |
| **D** | closure | all | this runbook | none | §B.4 checks | §B.4 (Stage D is normatively §B.4; carried as its own stage so no receipt lands out of order) |
| **E** | standing watch | all | falsifier cadence (§E) | none | scheduled re-query | falsifier hits → new dated receipts |

## §3 Stage A — npm publish completion

### A.1 The act (operator; npmjs.com console)

1. Log in to npmjs.com as the package owner (`seanchatmangpt`).
2. Package `@seanchatmangpt/graphlaw` → package settings → Trusted
   Publisher. Fields (must match `.github/workflows/publish-npm.yml:161-170`
   exactly — a one-field typo is the A1/A2 E404 class):
   - repository: `seanchatmangpt/graphlaw`
   - workflow filename: `publish-npm.yml`
   - environment: `npm`
3. Save. Note the save time — it feeds the A.2.0 clock gate.

Context carried from the P16 plan (operator-reported npm behavior, NOT
independently verified by this lane): a TP configuration is treated as
fresh for ~2 days; an older config is the suspected cause of a
refire-still-404. Treat it as a gate input, re-derive at use time.

### A.2 Retry-loop protocol

**A.2.0 TP-clock pre-attempt gate.** Before EVERY refire: if the TP config
save time is older than 2 days, re-save the config first (or confirm its
fields unchanged), then refire. No refire ever runs on a config whose
freshness is unknown.

**A.2.1 Refire command.** Re-run only the failed job — the validate job's
green result is retained:

```bash
gh run rerun 40084544193 -R seanchatmangpt/graphlaw --failed
```

**A.2.2 Bounded watch (≤25 min).** Poll the run's observable state with a
hard deadline; never an unbounded wait:

```bash
deadline=$((SECONDS+1500))
until [ "$(gh run view 40084544193 -R seanchatmangpt/graphlaw \
  --json status --jq .status)" = "completed" ]; do
  [ $SECONDS -gt $deadline ] && { echo "TIMEOUT-25MIN: re-query, do not blind-refire"; break; }
  sleep 30
done
gh run view 40084544193 -R seanchatmangpt/graphlaw --json conclusion,status,attempt,jobs
```

**A.2.3 Classification table.** On a failed attempt, classify BEFORE any
refire:

| Observed (log tail) | Meaning | Action |
|---|---|---|
| `E404` on `PUT …%2fgraphlaw` | no TP config matches (unconfigured, or repo/workflow/environment field mismatch, or expired config per A.2.0) | re-query/re-save the TP config; fix fields; refire ONCE. Two consecutive E404s on a confirmed-fresh config → stop, escalate (the field set or npm-side state is wrong, not stale) |
| `ENEEDAUTH` | OIDC exchange failed (missing `id-token` grant, npm OIDC outage, or npm incident) | verify workflow still has `id-token: write` + `environment: npm`; check npm status page; refire after confirmed, not before |
| `E403` forbidden/OTP | a token-based path was taken, or package settings block the publisher | MUST NOT happen (no token exists by doctrine). Investigate what authenticated; never add `NPM_TOKEN` as the "fix" |
| `EPUBLISHCONFLICT` ("cannot publish over previously published") — **phantom class** | the version is ALREADY on the registry — an earlier "failed" attempt actually succeeded (partial-success phantom) | **re-query before refire**: `npm view @seanchatmangpt/graphlaw versions --json`; if `26.10.22` is present, publish is DONE — go straight to §A.3. Never refire |
| job times out mid-upload with no registry error | unknown terminal state | treat as phantom-suspect: registry re-query first (rule above), classify from registry state, not from the job's exit |

**Re-query-before-refire rule (binding):** never issue a second refire for
the same error class without an intervening registry-state re-query
(`npm view`) between the attempts. The registry, not the job exit, is the
source of truth about whether a publish happened.

### A.3 Post-success verification (machine-verification)

All read-only; run in order; every check's output goes into the A3 receipt:

```bash
# 1. Version + dist-tag
npm view @seanchatmangpt/graphlaw version dist-tags
#   expect: version 26.10.22; dist-tags.latest 26.10.22

# 2. Tarball equality — the registry must serve the CI-validated bytes
npm view @seanchatmangpt/graphlaw@26.10.22 dist.integrity
#   expect: sha512-SlFgJxGWAxbtM[…HgL4crLlZYS9w==] (the CI log's integrity)
npm pack @seanchatmangpt/graphlaw@26.10.22 && shasum -a 1 seanchatmangpt-graphlaw-26.10.22.tgz
#   expect: 115e5024840955e741c788edcacf3358d2ff277a (== CI artifact, see ledger)

# 3. Attestations + registry signatures
npm view @seanchatmangpt/graphlaw@26.10.22 attestations
#   expect: provenance object naming repo seanchatmangpt/graphlaw + the run
npm audit signatures
#   expect: all signatures verified, exit 0

# 4. Install smoke + the pin check (b3sum-of-packaged-artifact == pin)
cd $(mktemp -d) && npm init -y >/dev/null && npm i @seanchatmangpt/graphlaw@26.10.22
b3sum node_modules/@seanchatmangpt/graphlaw/wasm/graphlaw.wasm
#   expect: 58a4409505f32b5d9aebc8c0ce2e2adeaba05c8c4b55f951a244ab4ad7d2280a
#   (== priv/graphlaw.wasm.blake3; ALREADY pre-witnessed on the CI artifact
#   bytes in receipts/2026-10-24-npm-attempt-ledger.md — this re-witnesses it
#   on the registry-served copy)
node -e "import('@seanchatmangpt/graphlaw').then(m => console.log(typeof m.createClient))"
#   expect: function
```

### A.4 Docs flip commit spec (after A.3 is green)

One commit, coordinator-owned git, `git commit -F <file>`:

| File:lines | Change | Receipt line to embed in the commit body |
|---|---|---|
| `README.md:91-97` | replace the publish-state blockquote with: latest published = `26.10.22` (run `40084544193` conclusion success, date) | `npm view @seanchatmangpt/graphlaw version` output |
| `README.md:100` | drop the parenthetical — plain `npm i @seanchatmangpt/graphlaw  # 26.10.22` | same |
| `js/npm/README.md:19-24` | publish-state paragraph → published `26.10.22`; remove the typed-blocked clause | same + `npm view … dist-tags` |
| `js/npm/CHANGELOG.md:10-13` | 26.10.22 entry intro → "published 2026-10-24 via run 40084544193"; remove typed-blocked sentence | run conclusion JSON |
| `js/npm/PUBLISHING.md` | 5 fixes — **already landed 2026-10-24 by the runbook lane**: title/intro scoped to `@seanchatmangpt/graphlaw`; name-claim item corrected (name already claimed, `26.10.14` live); TP console path names the scoped page; verifying section uses `npm view @seanchatmangpt/graphlaw dist-tags`; local fallback rewritten (publish WITHOUT `--provenance` — provenance requires the CI OIDC context — CI-validated tarball by digest only, missing provenance disclosed) | diff of the file |

## §4 Stage B — crates.io publish

### B.1 Machine-prep boundary (receipted; do not re-derive)

Publish ONLY from the verified scratch `/tmp/gl-pub-26.10.22.1dfLUa` —
never from the dirty main worktree. Preceded by:
`receipts/2026-10-24-crates-scratch-prep-26.10.22.md` (EXACT 4-file delta
vs `git show v26.10.22:`, lock byte-identical, no `.git`, versions
`26.10.22`/`0.7.9`, eyeron `--dry-run` exit 0). Before the first publish,
re-run that receipt's §1 tree diff — if a 5th differing file appears, STOP.

### B.2 Operator sequence

```bash
# 1. Login (token never echoed into any receipt; never committed)
cargo login <token>

# 2. Re-query the registry is still at the pre-state (PUBLISH-STATUS rows 3/4)
curl -s -H 'User-Agent: graphlaw-publish' https://crates.io/api/v1/crates/graphlaw-eyeron | jq -r .crate.max_version   # expect 0.7.8
curl -s -H 'User-Agent: graphlaw-publish' https://crates.io/api/v1/crates/graphlaw        | jq -r .crate.max_version   # expect 26.10.21

# 3. Publish eyeron FIRST (dependency order; graphlaw needs eyeron 0.7.9 on the index)
cd /tmp/gl-pub-26.10.22.1dfLUa
CARGO_TARGET_DIR=target-scratch cargo publish --locked -p graphlaw-eyeron

# 4. Bounded poll for index propagation (deadline 10 min; interval 20 s)
#    curl the .crate.max_version above until it prints 0.7.9

# 5. Publish graphlaw — the verify stage is a full real build (frozen-duckdb)
CARGO_TARGET_DIR=target-scratch cargo publish --locked -p graphlaw

# 6. Bounded poll until .crate.max_version prints 26.10.22 (deadline 15 min
#    AFTER the command returns; see the never-abort rule below)
```

**Never-abort rule (stage 20–60 min).** `cargo publish -p graphlaw` spends
most of its wall time in the verify leg (compiles the packaged crate,
frozen-duckdb included) printing NOTHING. Silence for 20–60 minutes is
normal on this repo. Never abort a publish for silence: killing the verify
leg and re-firing is how a phantom conflict class is manufactured. If the
command must be abandoned (machine failure), treat the attempt as
phantom-suspect and apply §A.2.3's re-query rule with
`curl …/crates/graphlaw` before any retry.

### B.3 Smokes (machine-verification, after each version lands)

- **graphlaw-eyeron has no binaries** — a CLI smoke is impossible; use a
  real link smoke from a fresh consumer:

  ```bash
  cd $(mktemp -d) && cargo new eyeron-smoke && cd eyeron-smoke
  cargo add graphlaw-eyeron@0.7.9
  # main.rs: call one real eyeron entry point and print its result
  cargo run --quiet
  #   expect: the entry point's real output, exit 0 — proves the published
  #   crate resolves, compiles standalone, and links
  ```

- **graphlaw**, two rungs (pick per time budget; both are real-crate,
  no-mock):
  - *Install rung*: `cargo install --locked graphlaw` (builds
    frozen-duckdb; minutes), then `graphlaw-verify --help` (exit 0) and —
    stronger — `graphlaw-verify` against one real receipt JSON produced by
    the repo's own courts.
  - *Lighter assert rung* (still compiles the dep, skips the bin install):
    a scratch consumer crate that depends on `graphlaw 26.10.22` and
    asserts (a) `graphlaw::BACKEND_AUTHORITIES` is non-empty
    (`src/lib.rs:190`) and (b) the typed refusal for an unknown step
    capability fires (`src/law.rs:783-796`, `UnknownStepCapability`) — the
    refusal text names `BACKEND_AUTHORITIES`, so a passing assert proves
    the published lib's admission surface is alive.

### B.4 CRITICAL closure — order is load-bearing

1. **PUBLISH-STATUS post-act addendum — APPEND ONLY.** New "Acted
   2026-10-24" section at the end; the six original rows are never
   rewritten. Each flip carries its live re-query output (rows 2/6 from
   §A.3, rows 3/4 from §B.2 step 6, row 5 unchanged). APPEND, because the
   v5 closeout (GO-NO-GO §C) declares the file's read-history and a
   rewrite would orphan it.
2. **GO-NO-GO §A.1 pointer.** v5 §A.1 already delegates publish standing to
   PUBLISH-STATUS; append one dated line under §A.1: "publish completion
   recorded — see PUBLISH-STATUS addendum 2026-10-24" (GO-NO-GO is
   append-only per its own v5 lease).
3. **RECEIPTS-INDEX.** Register in `docs/jira/v26.10.22/RECEIPTS.md` (and
   the milestone `INDEX.md` rows this lane already created): the attempt
   ledger, the scratch-prep receipt, this runbook, and the addendum — with
   one-line standing each.
4. **Chain sync LAST.** Only after every file above is final, sync
   `receipts/CHAIN.blake3` so the chain head covers the final bytes of
   everything cited. Syncing earlier = a chain over documents that then
   changed = a receipt chain that proves the wrong tree.

## §5 Stage C — v26.10.24 preview gates (machine-prep only)

No operator act. Run before any `.24` tag exists:

1. **Fresh-scratch ZERO-delta gate.** The `.24` scratch (staged from its
   future tag the same way) must diff against that tag with **zero** delta.
   The `.22` scratch needed 4 prep fixes (license, docs.rs pin, LICENSE
   restore, include drop) because the tag predated them — the `.24` tag
   must already contain every packaging fix, so nothing post-tag is ever
   shipped. A non-empty diff = tag not publish-ready.
2. **MSRV gate.** Re-read `rust-version` from the `.24` candidate's root
   `Cargo.toml` at gate time (`.22` declares `1.96`; the `.24` line's
   declared floor governs the gate) and build with a toolchain pinned to
   exactly that floor — the docs.rs/native build must succeed on the
   declared MSRV, not just on the maintainer's toolchain.
3. **TP clock check.** npm Trusted Publisher must name the workflow file +
   environment the `.24` tag will actually run. Any workflow rename or
   environment change between `.22` and `.24` requires a TP re-save BEFORE
   the tag is pushed (and the A.2.0 freshness clock applies to the
   publish, i.e. within 2 days of the save).
4. **`cargo package --list` greps** (both crates, on the fresh scratch):
   - eyeron: MUST NOT contain `LICENSE-P28-SIL.md` (include drop is
     intentional); MUST contain `LICENSE.md`, `README.md`, `UPSTREAM.md`.
   - root: MUST contain `LICENSE.md` AND `LICENSE-P28-SIL.md` (the pointer
     line must not dangle); MUST contain `priv/graphlaw.wasm` +
     `priv/graphlaw.wasm.blake3`; MUST NOT contain `.git`, `target/`,
     `receipts/`, or any `*.wasm` other than the pinned artifact.
   - grep both lists for `^\.git` and `target/` — zero matches.

## §6 Stage D — closure

Normatively specified once, at **§B.4** (addendum → GO-NO-GO pointer →
RECEIPTS-INDEX → chain sync last). It is carried as a distinct stage in
§2 so that the closure ordering is visible in the master table: no receipt
lands after the chain sync; nothing cites a file whose final bytes the
chain does not cover.

## §7 Stage E — standing watch

After completion, the standing claims are:

- npm: `26.10.22` is `latest`; provenance attestation visible.
- crates.io: max_version `0.7.9` / `26.10.22`.
- run `40084544193`: `conclusion: success`.

Re-query cadence: at every future doc cite (receipt-cited counts are
re-read from the receipt at use time — a recalled count is stale by
default). Falsifier hits (a registry value moving WITHOUT a receipted
transition — e.g. an unexpected version appearing) are typed incidents:
file a dated receipt, do not edit prior receipts.

## §8 Typed-failure table (all stages)

| Code / signal | Stage | Meaning | Recovery |
|---|---|---|---|
| `E404` on npm `PUT` | A | TP unconfigured/mismatched/expired | §A.2.3 row 1; re-save config; refire once on fresh config |
| `ENEEDAUTH` | A | OIDC exchange failed | §A.2.3 row 2; verify workflow grants; npm status; refire confirmed |
| `E403` | A | token path taken / settings block | §A.2.3 row 3; investigate; NEVER add `NPM_TOKEN` |
| `EPUBLISHCONFLICT` (phantom) | A, B | version already published by an earlier attempt | re-query registry FIRST (npm view / crates.io API); if present → verification stage, never refire |
| 25-min watch timeout | A | run still in progress or wedged | re-query run JSON; classify from job step states; do not blind-refire |
| "the lock file needs to be updated" | B | `--locked` vs manifest mismatch | STOP — a lock/manifest drift means the scratch is no longer tag-faithful; re-run the scratch receipt §1 diff |
| verify-leg silence > 60 min | B | normal-build vs wedged (ambiguous) | check `target-scratch/package/` mtime progress; if advancing, keep waiting; if frozen >15 min more, treat as abandoned → phantom-suspect protocol |
| crates.io 5xx / rate limit | B | registry-side | bounded backoff, re-query, retry publish only if max_version still absent |
| ENOSPC | A, B, C | disk full (killed the 11:10Z mutation sweep once) | `df -h /` ≥ 20 GiB pre-flight gate; abort typed, clean, re-enter |
| fresh-scratch diff non-empty | C | tag not publish-ready | file the delta as a tag-prep defect; no publish from a mutated scratch |

## Receipt fields (this doc)

- **Identity**: machine-prep lane over `/Users/sac/graphlaw` (branch
  `main`), 2026-10-24; subject = the completion protocol for
  `v26.10.22` + the `.24` preview gates. Live evidence captured this
  session: run 40084544193 attempt-2 JSON + failed-log tail, artifact
  download shasum/b3sum, scratch tree-diff + dry-run (embedded in the two
  receipts above).
- **Authority**: write lease = this file + the two receipts +
  `js/npm/PUBLISHING.md` (5 fixes) + one `INDEX.md` registration row. No
  publish, no rerun, no git transitions executed by this lane.
- **Consequence**: 4 new files + 1 edit + 1 registration; zero code edits.
- **Replay**: §A.2.2/§A.3/§B.2 commands all re-run read-only (the publish
  commands themselves are marked as operator acts); §C gates re-run on a
  fresh scratch.
- **Falsifiers**: (a) `gh run view 40084544193 --json conclusion` =
  `success` discharges Stage A; (b) crates.io `max_version` `0.7.9`/`26.10.22`
  discharges Stage B; (c) any §2 cell contradicting PUBLISH-STATUS at read
  time invalidates that cell — PUBLISH-STATUS governs.
- **Standing**: protocol-current as of 2026-10-24 (A2 latest attempt,
  E404, TP unconfirmed); every live value re-verified at use time.

## See Also

- [`PUBLISH-STATUS.md`](PUBLISH-STATUS.md) ·
  [`OPERATOR-RUNBOOK-v26.10.22-PUBLISH.md`](OPERATOR-RUNBOOK-v26.10.22-PUBLISH.md) ·
  [`GO-NO-GO-v26.10.22.md`](GO-NO-GO-v26.10.22.md) (§A.1)
- `/Users/sac/graphlaw/receipts/2026-10-24-npm-attempt-ledger.md` ·
  `/Users/sac/graphlaw/receipts/2026-10-24-crates-scratch-prep-26.10.22.md`
- `/Users/sac/graphlaw/js/npm/PUBLISHING.md`
