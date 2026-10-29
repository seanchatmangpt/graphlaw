# Publishing — @seanchatmangpt/graphlaw

Operator runbook for npm releases of the scoped package
`@seanchatmangpt/graphlaw` (calver, kept in lockstep with the root
`Cargo.toml` version by `scripts/sync-version.mjs`).

Doctrine (mirrors `publish-npm.yml` / `release.yml`, operator decision
2026-10-10): **CI never holds registry credentials.** No `NPM_TOKEN` secret
exists or may be added; publication authenticates via GitHub OIDC against npm
Trusted Publishing.

## One-time npm setup

1. **The package name is already claimed.** `@seanchatmangpt/graphlaw`
   exists on the npm registry (latest published version: `26.10.14`,
   published 2026-10-17) and is owned by the `seanchatmangpt` npm account —
   there is no name to claim. Never publish ad hoc from a local machine to
   "test" the name: a publish is immutable and goes through CI (Trusted
   Publishing) or, only in the typed outage path, the digest-verified
   local fallback below.
2. **Configure the Trusted Publisher.** On npmjs.com, open the scoped
   package page for `@seanchatmangpt/graphlaw` and set Trusted Publisher to:
   - repository: `seanchatmangpt/graphlaw`
   - workflow filename: `publish-npm.yml`
   - environment: `npm`
   The configuration is **not validated until first use** — a typo in any of
   these fields surfaces only as a failed OIDC exchange at publish time.
3. **Create the GitHub environment `npm`.** Repository Settings →
   Environments → `npm`. It holds **no secrets** — the publish job needs only
   `id-token: write` for OIDC. Optional reviewers/protection rules apply to
   every publish.
4. **Repository must be public.** Provenance attestation is only visible for
   public repositories; a private repo publishes without visible provenance.

## Release flow

1. Bump the version in the root `Cargo.toml` (`[package] version`).
2. Sync the npm package:
   `node scripts/sync-version.mjs` (updates `js/npm/package.json`; commit
   both files).
3. Commit, then tag `v<version>` (e.g. `v26.10.13`) and push the tag.
4. `release.yml` runs automatically: it creates the GitHub release for the
   version declared in `Cargo.toml` (tagging is automatic there too — the
   manual tag drives `publish-npm.yml`, which triggers on `v*` tags).
5. `publish-npm.yml` then: verifies the tag matches BOTH `js/npm/package.json`
   and root `Cargo.toml` (fails closed), rebuilds the wasm artifact and
   checks its blake3 against `priv/graphlaw.wasm.blake3`, installs/builds/
   tests the package, packs twice and diffs the file lists (tarball drift
   check), uploads the tarball, then the `publish` job re-verifies the
   blake3 and publishes with OIDC (`--tag latest`, `--access public`).
6. `RELEASE_HOLD` gate: while the repository variable `RELEASE_HOLD` is
   `true`, nothing is validated or published (both jobs' `if:` conditions).
   Set it to `false`/unset to release.

## Same-day re-release rule

npm calver versions are immutable once published. If a same-day fix is
needed after `26.10.13` is live, publish a prerelease-bumped version, e.g.
`26.10.13-1`, not a rewritten `26.10.13`. Note the tag check requires the
tag to match the package version, so the re-release gets its own tag
`v26.10.13-1` and its own `Cargo.toml` bump.

## Verifying

- `npm view @seanchatmangpt/graphlaw dist-tags` — `latest` should point at
  the new version.
- Provenance: the attestation is visible on the npmjs.com package page
  (Provenance section, linking the public repo and workflow run).
- `npm audit signatures` — verifies registry signatures.

## Local publish fallback

Only if OIDC is blocked. This path forges no CI credentials — it uses your
personal npm account, not any stored token:

1. `cd js/npm && npm login` (WebAuthn/hardware-key 2FA).
2. `npm publish` **without** `--provenance`. Provenance attestations
   require the CI OIDC context (GitHub Actions `id-token: write` against a
   configured Trusted Publisher) — a local run cannot mint one, and passing
   `--provenance` locally fails. The local publish therefore ships with no
   provenance attestation.
3. Because of that, only publish the **CI-validated tarball by digest**: the
   tarball packed by `publish-npm.yml` on a green validate-and-pack run,
   verified byte-for-byte against the run's logged shasum before upload.
   Do not publish from a locally-modified tree.
4. Disclose it: a fallback-published version carries no provenance — note
   the outage and the run id in the release notes.
5. Same immutability rules apply — a local publish claims the version just
   as permanently as CI would.
