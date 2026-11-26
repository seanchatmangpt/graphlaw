# release-publish.md

Operator runbook for releasing and publishing graphlaw. Crate version `26.11.25`
(`Cargo.toml:3`), Rust toolchain 1.98.0. The npm sections further down were written against
`26.10.28` line numbers and are not re-verified for 26.11.25; JS/npm is deferred to
v26.11.26. Every step cites file:line.

## v26.11.25 flow (wasm Release asset, Hex without wasm, hold-for-go)

Goal: ship the crate release assets and the Elixir wrapper without bundling the wasm in Hex.

Prerequisites: `gh` authenticated; Hex credentials on the workstation; the current value of
repo variable `RELEASE_HOLD` known.

1. **Hold.** While `RELEASE_HOLD=true` the release workflow skips tag and publish
   (`.github/workflows/release.yml:13-15,63-67`). Keep it set until the operator says go.
2. **Go.** Unset or delete `RELEASE_HOLD`, then merge the version bump to main or re-run the
   workflow (`release.yml:17-20`). The plan step needs no existing `v<version>` tag.
3. **Release asset.** The job builds the wasm (`--profile wasm`, wasi-sdk 34, EH flag) and
   creates GitHub release `v<version>` with `graphlaw.wasm`, registry json/ttl and
   `op-examples.json`, each with a `.blake3` (`release.yml:88-110`).
4. **Hex without wasm.** `graphlaw_ex/mix.exs:99-101` lists `files:` with the pin files
   (`MANIFEST.json`, `.blake3`, `.sha256`) and no `graphlaw.wasm`; the 8 MB Hex limit is the
   reason (`graphlaw_ex/README.md:67-68`). Size check: `mix hex.build` step at
   `.github/workflows/elixir.yml:152-157`.
5. **Consumer side.** `mix graphlaw_ex.fetch` downloads the Release asset for the package
   version and verifies sha256 and BLAKE3; `GRAPHLAW_WASM_PATH=<abs path>` stages a local
   file with no network (air-gapped)
   (`graphlaw_ex/lib/mix/tasks/graphlaw_ex.fetch.ex:7-22`).

Verify: `gh release view v<version>` lists the eight wasm/registry assets;
`mix graphlaw_ex.fetch && mix graphlaw_ex.doctor` succeeds on a clean cache. Not run by this
edit.

## Name resolution (settled 2026-10-24)

The scoped name is the package name and the pipeline proves it:
`js/npm/package.json:2` declares `@seanchatmangpt/graphlaw`, and CI run
40084544193 (tag `v26.10.22`, head `29c6d98`) passed the validate-and-pack
job end to end — deterministic pack of the scoped tarball
(`seanchatmangpt-graphlaw-26.10.22.tgz` shape), the pack-step tarball
assertion, and the cross-job blake3 gate — before the publish job's
registry `PUT` failed E404 on the then-unconfigured Trusted Publisher
(milestone context: `docs/jira/v26.10.28/INDEX.md`). The former "known name
discrepancy" section here described a pre-`29c6d98` state of
`publish-npm.yml`; the scoped-name resolution is proven green in that run
and no naming fix is owed. Configure the Trusted Publisher on
`@seanchatmangpt/graphlaw` (exactly this name — see One-time setup 1).

## One-time setup

1. **npm Trusted Publisher** (`.github/workflows/publish-npm.yml:3-14`; scoped-package header comment).
   On npmjs.com, under the SCOPED package `@seanchatmangpt/graphlaw`, set:
   - repository: `seanchatmangpt/graphlaw`
   - workflow: `publish-npm.yml`
   - environment: `npm`
2. **GitHub environment `npm`**: create an environment named exactly `npm`
   on the repo (optionally with required reviewers)
   (`.github/workflows/publish-npm.yml:13-14`, consumed at
   `.github/workflows/publish-npm.yml:167`).
3. **cargo login** (local machine only). CI never holds registry credentials
   (`.github/workflows/release.yml:8-10`, re-asserted at
   `.github/workflows/release.yml:112-116`, the "No crates.io publication
   (policy witness)" step, which only echoes the policy and publishes nothing). On your workstation: `cargo login` once; publishing is
   local-only by operator decision 2026-10-10.

## Per-release order

1. **Version triple match.** `js/npm/package.json` version, root
   `Cargo.toml` `package.version`, and the tag `v<X.Y.Z>` must all be equal.
   `release.yml` derives the version from Cargo.toml
   (`.github/workflows/release.yml:57-59`); `publish-npm.yml` fail-closes
   the triple on tag push (`.github/workflows/publish-npm.yml:78-103`);
   on `workflow_dispatch` it still requires package==crate
   (`.github/workflows/publish-npm.yml:94-100`).
2. **RELEASE_HOLD absent check.** While repo variable `RELEASE_HOLD=true`,
   release.yml's plan step forces `work=no`
   (`.github/workflows/release.yml:63-67`) and
   publish-npm's jobs skip at the job level
   (`.github/workflows/publish-npm.yml:49,164`). Verify
   `gh variable list` shows RELEASE_HOLD unset or not `true` before tagging.
3. **Tag push.** `git tag v26.10.28 && git push origin v26.10.28` (substitute
   the version being released). The tag
   push triggers publish-npm (`publish-npm.yml:22-25`, `v*` tags);
   release.yml triggers on push to main + dispatch (`release.yml:17-20`). Note release.yml's
   tag/release steps run on a `push` to main, not on the tag itself —
   the tag push alone does not run release.yml; merging/bumping on main
   does.
4. **release.yml verify work=yes.** Open the "Release" run and check the
   plan step's outputs (`work=yes` requires that no `refs/tags/v<version>` exists on origin,
   `.github/workflows/release.yml:52-71`). A green run with
   `work=no` is normal — nothing to release, not a skipped gate
   (`.github/workflows/release.yml:30-33`). Confirm the "Tag and create the
   GitHub release" step created release `v<version>` with graphlaw.wasm,
   capability-registry.json/.ttl, op-examples.json, each with `.blake3`
   sidecars (`.github/workflows/release.yml:90-110`).
5. **publish-npm validate+publish.** The `validate` job gates on the version
   triple, committed-artifact-bytes == wasm digest pin, npm ci/build/test, and a
   tarball drift check (`.github/workflows/publish-npm.yml:41-150`); the `publish`
   job re-verifies the tarball blake3 across jobs
   (`.github/workflows/publish-npm.yml:198-210`) and publishes via OIDC —
   no NPM_TOKEN secret exists or may be added
   (`.github/workflows/publish-npm.yml:3-6`); plain `npm publish`, never
   `--provenance` (`.github/workflows/publish-npm.yml:211-215`).
6. **crates.io: none.**
   graphlaw does not publish to crates.io — the purrdf dependency is the
   rev-pinned [seanchatmangpt/purrdf](https://github.com/seanchatmangpt/purrdf) fork, and a git-only dependency
   cannot publish. Distribution is this repository and the GitHub release assets
   (`.github/workflows/release.yml`).
7. **Post-publish verification.** `cargo search graphlaw` (or open
   https://crates.io/crates/graphlaw and
   https://www.npmjs.com/package/@seanchatmangpt/graphlaw); confirm the
   GitHub release assets' blake3 sidecars against a local `b3sum` of the
   downloaded artifacts; `npm view @seanchatmangpt/graphlaw version`.
   For the npm provenance to be visible the repo must be public
   (`.github/workflows/publish-npm.yml:17-20`).

## Failure modes

- **Hold silently skips green.** With `RELEASE_HOLD=true`, release.yml's
  plan step sets `tag=yes` and leaves `work=no`, exiting green
  (`.github/workflows/release.yml:63-67`): the run is green but
  nothing tagged or published. publish-npm's jobs skip entirely at job
  level (`.github/workflows/publish-npm.yml:49,164`). A green run is not
  evidence of a release while the hold is set — check the hold first.
- **Tag ≠ version fails closed.** publish-npm's versions step exits 1 if
  the tag mismatches package.json or Cargo.toml
  (`.github/workflows/publish-npm.yml:84-93`), and on dispatch mismatches
  package vs crate (`.github/workflows/publish-npm.yml:94-100`). Re-tag:
  delete and push a corrected tag (never rebase/force-push); the tag name
  is the failing input, not the tree.
- **wasm digest pin gate.** The validate job does NOT rebuild the wasm (the
  build is not byte-reproducible across hosts); it checks that the committed
  `priv/graphlaw.wasm` bytes equal the digest in `priv/graphlaw.wasm.blake3`
  (`.github/workflows/publish-npm.yml:104-119`). Red here means the committed
  artifact drifted from its pin — regenerate the pin via the normal flow
  (`docs/WASM.md`), do not hand-edit it.
- **npm 11.5.1 only in the publish job.** Trusted Publishing needs npm
  >= 11.5.1; Node 22 bundles 10.x, so only the publish job upgrades npm
  (`.github/workflows/publish-npm.yml:179-182`); the validate job runs on
  Node 22's bundled npm (`.github/workflows/publish-npm.yml:64-67`). Do not
  "fix" the validate job by moving the upgrade there — it is only required
  for OIDC publish, not validate.

## See Also

- [run-the-wasm-module.md](run-the-wasm-module.md)
- [../WASM.md](../WASM.md)
- [../../SECURITY.md](../../SECURITY.md)
