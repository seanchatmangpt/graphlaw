# PUBLISH-STATUS

Publish state of v26.10.22, recorded 2026-10-24 by a read-only verification
lane over `/Users/sac/graphlaw` (HEAD `4df19b5` at read open; write lease =
this file only). Every value below is a live query result from this session;
commands and outputs are embedded per receipt discipline.

## Status table

| # | Surface | Check | Observed (live, 2026-10-24) | Expected | Verdict |
|---|---|---|---|---|---|
| 1 | git tag | `v26.10.22` on origin | present, commit `29c6d985…` | present | GREEN |
| 2 | npm | latest | `26.10.14` (`2026-10-17`) | `26.10.14` until TP set | GREEN (blocked) |
| 3 | crates.io | `graphlaw-eyeron` max | `0.7.8` | `0.7.8` | GREEN (local `0.7.9` publishable) |
| 4 | crates.io | `graphlaw` max | `26.10.21` | `26.10.21` | GREEN (local `26.10.22` publishable) |
| 5 | CI | run `40084544193` job 1 validate-and-pack | `success` | green before publish | GREEN |
| 6 | CI | run `40084544193` npm publish | `failure` — npm `E404` | publish completes | RED |

## What shipped by machine

Everything through the validate-and-pack green run. Run `40084544193`
(`publish-npm`, push event on tag `v26.10.22`, head `29c6d98` — exactly the
tagged commit; run window 10:05:43–10:07:25Z):

- **Version triple gate**: npm `26.10.22` / eyeron `0.7.9` / graphlaw
  `26.10.22` (local `crates/graphlaw-eyeron/Cargo.toml:3` and `Cargo.toml:3`
  agree with the packed tarball).
- **Byte-authority gate** on the committed wasm artifact (CI-side wasm rebuild
  is not stageable; js.yml retains the rebuild gate for main pushes).
- **npm 198/198 in CI**, deterministic pack + drift check (pack destination
  mkdir fix in `29c6d98`).
- **Packed artifact, from the run log**: `@seanchatmangpt/graphlaw@26.10.22`,
  137 files, 12.6 MB packed / 58.4 MB unpacked,
  shasum `115e5024840955e741c788edcacf3358d2ff277a`, integrity
  `sha512-SlFgJxGWAxbtM[...]HgL4crLlZYS9w==`, includes
  `wasm/graphlaw.wasm` (58.1 MB).
- Tag `v26.10.22` pushed to origin (row 1).

The publish job then failed at the registry boundary — machine-side, nothing
further can ship without the two operator acts below.

## The blocker (run 40084544193)

Job 2 log tail (10:07:23Z):

```text
npm error code E404
npm error 404 Not Found - PUT https://registry.npmjs.org/@seanchatmangpt%2fgraphlaw - Not found
npm error 404  The requested resource '@seanchatmangpt/graphlaw@26.10.22' could not be found or you do not have permission to access it.
npm error 404 Note that you can also install from a tarball, folder, http url, or git url.
##[error]Process completed with exit code 1.
```

The workflow runs OIDC trusted publishing with no token (`id-token: write`,
`environment: npm`, `.github/workflows/publish-npm.yml:165-170`). npm answers
the `PUT` with **404 when no Trusted Publisher configuration matches the
package** — i.e. the Trusted Publisher for `@seanchatmangpt/graphlaw` is not
configured on npmjs.com. This is machine-impossible by design (no credential
in the runner), not a workflow defect: job 1 on the same run is green.

## Operator acts (the only remaining acts; both manual)

```bash
# Act 1 — npm: configure Trusted Publisher (npmjs.com console action, not CLI).
#   npmjs.com → package @seanchatmangpt/graphlaw → package settings →
#   Trusted Publisher:
#     repository  : seanchatmangpt/graphlaw
#     workflow    : publish-npm.yml
#     environment : npm
#   Then rerun the publish:
gh run rerun 40084544193 -R seanchatmangpt/graphlaw

# Act 2 — crates.io: token login, then publish in dependency order.
cargo login <token>
cargo publish --locked -p graphlaw-eyeron   # 0.7.9 (crates.io max today: 0.7.8)
cargo publish --locked -p graphlaw          # 26.10.22 (crates.io max today: 26.10.21)
```

Act 2 order matters: graphlaw depends on eyeron (`Cargo.toml:68`
`eyeron = { package = "graphlaw-eyeron", version = "0.7.9", ... }`).

## Post-act verification commands

```bash
# After Act 1 (npm):
npm view @seanchatmangpt/graphlaw version dist-tags
#   expect: version 26.10.22; dist-tags.latest 26.10.22

# After Act 2 (crates.io) — jq path is .crate.max_version (verified live):
curl -s -H 'User-Agent: graphlaw-status' \
  https://crates.io/api/v1/crates/graphlaw-eyeron \
  | jq -r .crate.max_version        # expect 0.7.9
curl -s -H 'User-Agent: graphlaw-status' \
  https://crates.io/api/v1/crates/graphlaw \
  | jq -r .crate.max_version        # expect 26.10.22

# Rerun verdict:
gh run view 40084544193 -R seanchatmangpt/graphlaw --json conclusion,jobs \
  --jq '{conclusion, jobs: [.jobs[] | {name, conclusion}]}'
#   expect: conclusion "success", both jobs green
```

## Receipt fields

- **Identity**: read-only lane over `/Users/sac/graphlaw` HEAD `4df19b5`
  (branch `main`), read window 2026-10-24; subject = publish state of
  tag `v26.10.22` across origin / npm / crates.io / Actions.
- **Authority**: write lease = `docs/jira/v26.10.23/PUBLISH-STATUS.md` only;
  no publish, no cargo, no git transitions executed by this lane.
- **Consequence**: one file created (this one); zero code edits.
- **Replay** (each command re-runs read-only):

```bash
gh api repos/seanchatmangpt/graphlaw/git/ref/tags/v26.10.22
npm view @seanchatmangpt/graphlaw version dist-tags time.modified
curl -s -H 'User-Agent: ...' \
  https://crates.io/api/v1/crates/graphlaw-eyeron | jq -r .crate.max_version
curl -s -H 'User-Agent: ...' \
  https://crates.io/api/v1/crates/graphlaw | jq -r .crate.max_version
gh api repos/seanchatmangpt/graphlaw/actions/runs/40084544193 \
  --jq '{conclusion, head_sha}'
gh run view 40084544193 -R seanchatmangpt/graphlaw --log-failed | tail -20
```

- **Falsifiers**: (a) `npm view` returning `26.10.22` flips row 2 to published
  and discharges Act 1; (b) crates.io `.crate.max_version` returning `0.7.9` /
  `26.10.22` flips rows 3/4 and discharges Act 2; (c) run `40084544193`
  (or its rerun) reaching `conclusion: success` flips row 6. Any observed value
  differing from the table above at read time invalidates this file.
- **Standing**: rows 1/3/4/5 GREEN as of 2026-10-24; row 2 GREEN-as-expected
  (unpublished); row 6 RED — publish typed-blocked pending the two operator
  acts; re-verify every row at use time before citing.

## See Also

- `/Users/sac/graphlaw/docs/jira/v26.10.23/GO-NO-GO-v26.10.22.md` (v4 terminal state)
- `/Users/sac/graphlaw/docs/jira/v26.10.23/TAG-PREP.md`
- `/Users/sac/graphlaw/.github/workflows/publish-npm.yml`
- `https://github.com/seanchatmangpt/graphlaw/actions/runs/40084544193`
