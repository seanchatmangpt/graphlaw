# PUBLISH-DRYRUN (2026-11-17)

Goal step: dry-run the npm publish for `@seanchatmangpt/graphlaw`. Run locally in this container at HEAD `dd6a80e`; **nothing was published, tagged or pushed**. Output below is real, trimmed to the lines that matter.

## What was and was not exercised
- Exercised: version triple, clean install, wasm pin check, build, test suite, publint, tarball contents, and `npm publish --dry-run`.
- NOT exercised: npm Trusted Publisher / OIDC exchange (GL28-12, operator action, still unconfigured per repo text), the GitHub `npm` environment, `RELEASE_HOLD`, `release.yml`, crates.io (none by policy), Hex (`mix` not installed here; undecided), PyPI (no path). A green dry run therefore does not mean the real publish will succeed.

## Evidence
| step | command | result |
|---|---|---|
| version triple | Cargo.toml / js/npm/package.json / python/pyproject.toml | all `26.10.28` |
| remote tag | `git ls-remote --tags origin \| grep -c v26.10.28` | 2 lines (tag + peeled): tag `v26.10.28` already exists on origin |
| clean install | `cd js/npm && npm ci` | `added 110 packages`, exit 0 |
| build | `npm run build` | exit 0; `build: wasm/graphlaw.wasm in place (blake3 c92caeb6d56792587f2939769678e02b77c868f2919ef8566060e02f3c6c1b9c)`; dist-nodejs, dist-web, dist-bundler assembled |
| wasm pin | `cat priv/graphlaw.wasm.blake3` | `c92caeb6d56792587f2939769678e02b77c868f2919ef8566060e02f3c6c1b9c` — identical to the build digest |
| tests | `npm test` | exit 0; `Test Files 10 passed \| 1 skipped (11)`; `Tests 189 passed \| 9 skipped (198)` |
| lint | `npx publint` | `All good!` |
| dry run | `npm publish --dry-run --ignore-scripts --access public` | exit 0; `+ @seanchatmangpt/graphlaw@26.10.28`; 137 files; package 14.9 MB; unpacked 63.2 MB; shasum `4a2efc3cdb03fc41845a204e327551aae4c093a5`; `wasm/graphlaw.wasm` 62.9 MB inside; "requires you to be logged in ... (dry-run)" warning is expected without credentials |

`--ignore-scripts` skipped `prepublishOnly` (build + test) only because both were run explicitly just before.

## Reading the result
- The package is publishable as built: pin matches, tests pass, tarball is well-formed.
- The 9 skipped tests are the opt-in browser suite (`GRAPHLAW_BROWSER_TESTS=1`); browser coverage was not witnessed.
- **26.10.28 cannot be the next publish by tag push:** tag `v26.10.28` already exists on origin. npm versions are immutable, and registry state (reported `latest` 26.10.14) was not re-checked. Publishing needs the operator steps in RELEASE-PUBLISH-STANDING.md; a new version needs a coordinated bump of Cargo.toml, js/npm/package.json, python/pyproject.toml and the wasm digest pin (ARTIFACTS.blake3 / priv/graphlaw.wasm.blake3) in one commit.
- Shasum differs from any earlier CI pack only if inputs differ; no earlier tarball was available to diff.

## Working tree after the run
`git status --short` shows only `?? docs/jira/v26.11.17/`. `js/npm/node_modules`, `dist-*` and `wasm/graphlaw.wasm` are gitignored.

## Real publish: not done, needs the operator
1. Confirm `gh variable list` shows RELEASE_HOLD unset; configure the npm Trusted Publisher (`seanchatmangpt/graphlaw`, `publish-npm.yml`, env `npm`); create GitHub environment `npm`.
2. Decide the version (26.10.28 tag already exists), bump all sites together, merge, tag, let publish-npm.yml run.
3. `npm view @seanchatmangpt/graphlaw version` afterwards. No publish receipt exists until that is done; do not claim one.
