# release-publish.md

Operator runbook for releasing and publishing graphlaw. Mechanism verified
against `.github/workflows/release.yml` and `.github/workflows/publish-npm.yml`
at the current tree; every step cites file:line.

## Known name discrepancy (fix before first publish)

- `js/npm/package.json:2` declares the package as the scoped
  `@seanchatmangpt/graphlaw`, but the `publish-npm.yml` header
  (`.github/workflows/publish-npm.yml:9`) instructs configuring a Trusted
  Publisher for the unscoped name `graphlaw`. npm Trusted Publishers are
  configured per exact package name; configuring `graphlaw` will not admit
  OIDC publishes of `@seanchatmangpt/graphlaw`. Configure the Trusted
  Publisher on `@seanchatmangpt/graphlaw`.
- Secondary: the pack step's tarball assertion
  (`.github/workflows/publish-npm.yml:131-133`) expects `graphlaw-$VERSION.tgz`,
  but `npm pack` of a scoped package produces
  `seanchatmangpt-graphlaw-$VERSION.tgz` (scope becomes a directory-like
  prefix with `/` flattened to `-`). With the current scoped name this
  assertion fails the validate job every run. Either unscope the package or
  fix the expected tarball name before relying on this pipeline.

## One-time setup

1. **npm Trusted Publisher** (`.github/workflows/publish-npm.yml:3-14`).
   On npmjs.com, under the SCOPED package `@seanchatmangpt/graphlaw`
   (see discrepancy above), set:
   - repository: `seanchatmangpt/graphlaw`
   - workflow: `publish-npm.yml`
   - environment: `npm`
2. **GitHub environment `npm`**: create an environment named exactly `npm`
   on the repo (optionally with required reviewers)
   (`.github/workflows/publish-npm.yml:13-14`, consumed at
   `.github/workflows/publish-npm.yml:161`).
3. **cargo login** (local machine only). CI never holds registry credentials
   (`.github/workflows/release.yml:8-12`, re-asserted at
   `.github/workflows/release.yml:181-185` where the CI publish step is a
   typed refusal). On your workstation: `cargo login` once; publishing is
   local-only by operator decision 2026-10-10.

## Per-release order

1. **Version triple match.** `js/npm/package.json` version, root
   `Cargo.toml` `package.version`, and the tag `v<X.Y.Z>` must all be equal.
   `release.yml` derives the version from Cargo.toml
   (`.github/workflows/release.yml:60-68`); `publish-npm.yml` fail-closes
   the triple on tag push (`.github/workflows/publish-npm.yml:77-100`);
   on `workflow_dispatch` it still requires package==crate
   (`.github/workflows/publish-npm.yml:94-99`).
2. **RELEASE_HOLD absent check.** While repo variable `RELEASE_HOLD=true`,
   release.yml's plan step forces `work=no`
   (`.github/workflows/release.yml:74-77`) and
   publish-npm's jobs skip at the job level
   (`.github/workflows/publish-npm.yml:48,158`). Verify
   `gh variable list` shows RELEASE_HOLD unset or not `true` before tagging.
3. **Tag push.** `git tag v26.10.22 && git push origin v26.10.22`. The tag
   push triggers both workflows (`release.yml:19-22` push to main +
   dispatch; `publish-npm.yml:21-24` on `v*` tags). Note release.yml's
   tag/release steps run on a `push` to main, not on the tag itself —
   the tag push alone does not run release.yml; merging/bumping on main
   does.
4. **release.yml verify work=yes.** Open the "Release" run and check the
   plan step's outputs (`work=yes` requires any of tag/eyeron/graphlaw
   unpublished, `.github/workflows/release.yml:68-78`). A green run with
   `work=no` is normal — nothing to release, not a skipped gate
   (`.github/workflows/release.yml:32-35`). Confirm the "Tag and create the
   GitHub release" step created release `v26.10.22` with graphlaw.wasm,
   capability-registry.json/.ttl, op-examples.json, each with `.blake3`
   sidecars (`.github/workflows/release.yml:164-179`).
5. **publish-npm validate+publish.** The `validate` job gates on the version
   triple, the wasm digest pin, npm ci/build/test, and a double-pack drift
   check (`.github/workflows/publish-npm.yml:41-152`); the `publish` job
   re-verifies the tarball blake3 across jobs
   (`.github/workflows/publish-npm.yml:192-203`) and publishes via OIDC —
   no NPM_TOKEN secret exists or may be added
   (`.github/workflows/publish-npm.yml:3-6`); plain `npm publish`, never
   `--provenance` (`.github/workflows/publish-npm.yml:205-209`).
6. **crates.io: eyeron first, then graphlaw, locally.**
   `cargo publish -p graphlaw-eyeron && cargo publish -p graphlaw`
   from the logged-in machine (`.github/workflows/release.yml:8-12`).
   graphlaw depends on graphlaw-eyeron, so eyeron must land first.
7. **Post-publish verification.** `cargo search graphlaw` (or open
   https://crates.io/crates/graphlaw and
   https://www.npmjs.com/package/@seanchatmangpt/graphlaw); confirm the
   GitHub release assets' blake3 sidecars against a local `b3sum` of the
   downloaded artifacts; `npm view @seanchatmangpt/graphlaw version`.
   For the npm provenance to be visible the repo must be public
   (`.github/workflows/publish-npm.yml:16-19`).

## Failure modes

- **Hold silently skips green.** With `RELEASE_HOLD=true`, release.yml's
  plan step rewrites tag/eyeron/graphlaw to `yes` and exits green with
  work=no (`.github/workflows/release.yml:74-77`): the run is green but
  nothing tagged or published. publish-npm's jobs skip entirely at job
  level (`.github/workflows/publish-npm.yml:48,158`). A green run is not
  evidence of a release while the hold is set — check the hold first.
- **Tag ≠ version fails closed.** publish-npm's versions step exits 1 if
  the tag mismatches package.json or Cargo.toml
  (`.github/workflows/publish-npm.yml:85-92`), and on dispatch mismatches
  package vs crate (`.github/workflows/publish-npm.yml:96-99`). Re-tag:
  delete and push a corrected tag (never rebase/force-push); the tag name
  is the failing input, not the tree.
- **wasm digest pin gate.** The validate job rebuilds the wasm artifact and
  refuses on digest drift vs `priv/graphlaw.wasm.blake3`
  (`.github/workflows/publish-npm.yml:103-114`). Red here means the built
  artifact differs from the pinned digest — regenerate the pin via the
  normal flow, do not hand-edit it.
- **npm 11.5.1 only in the publish job.** Trusted Publishing needs npm
  >= 11.5.1; Node 22 bundles 10.x, so only the publish job upgrades npm
  (`.github/workflows/publish-npm.yml:173-176`); the validate job runs on
  Node 22's bundled npm (`.github/workflows/publish-npm.yml:63-66`). Do not
  "fix" the validate job by moving the upgrade there — it is only required
  for OIDC publish, not validate.
