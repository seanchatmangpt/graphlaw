# BINDINGS-STANDING (v26.11.17)

Lane 12. Static inspection at HEAD dd6a80e, 2026-11-17. Nothing executed; no network checks of npm/PyPI. Standing: UNVERIFIED by court.

## Version alignment (target 26.10.28)
| Surface | Version | Source |
|---|---|---|
| Rust workspace | 26.10.28 | Cargo.toml |
| wasm crate | 26.10.28 | wasm/Cargo.toml |
| npm @seanchatmangpt/graphlaw | 26.10.28 | js/npm/package.json |
| python graphlaw | 26.10.28 | python/pyproject.toml |
| TS reference doc | 26.10.28 | docs/reference/typescript-api.md:4 |
| JS how-to | 26.10.28 | docs/how-to/use-from-javascript.md:16 |
| ABI_VERSION | 1 | wasm/src/abi_meta.rs |

Aligned. `js-drift` (js.yml) gates package.json vs workspace via sync-version.mjs.

## Layout
- The npm package is `js/npm/` (js/ holds only README.md, HANDWRITTEN.md, npm/).
- `python/` and `docs/python/` are TRACKED (28 files, commit dd6a80e). The v26.11.16 migration-plan "untracked" note is superseded.

## Publish state (repo text stamped 2026-11-09; live registry NOT re-checked)
- npm `latest` = 26.10.14 (2026-10-17). 26.10.22 was packed and validated (run 40084544193) but its publish was blocked on the Trusted Publisher config (GL28-12). 26.10.28 not published. Source is 2 releases ahead.
- publish-npm.yml: OIDC only, environment `npm`, tag `v*` or dispatch, gated by `vars.RELEASE_HOLD != 'true'`. No NPM_TOKEN by doctrine.
- PyPI: no workflow, not published; pyproject has `dependencies=[]` and wraps the packet-4 runner and verify CLI (external binary).

## Test inventory
- JS vitest (js/npm/test): about 47 literal cases in 11 files (artifact 4, blake3 9, client.types 6, corpus 1, errors 5, forward-compat 2, generated-types 5, init 6, limits 5, wire-memory-grow 3, ops.examples) plus opt-in browser test (`GRAPHLAW_BROWSER_TESTS=1`). Loop-generated cases not counted.
- Python pytest: 11 defs (test_wrapper 9, test_capability_gate 2).
- js.yml: js-build (wasm build, BLAKE3 pin, ABI compare), js-test-node (20, 22), js-test-browser (self-skips exit 0 without Chromium, so green may mean zero browser tests ran), js-package-lint, js-drift. All skip under RELEASE_HOLD=true.
- ci.yml (~line 308) sets up Python 3.12 + pytest; whether it runs python/tests is unconfirmed.

## Gaps / actions
1. Operator: finish npm Trusted Publisher config, unset RELEASE_HOLD, tag, publish.
2. Decide Python distribution (PyPI or repo-only).
3. Confirm ci.yml runs python/tests.
4. Browser tests self-skip: record whether any CI run executed them.
5. Run `cd js/npm && npm ci && npm run typecheck && npm test` and pytest for real counts.
6. Re-stamp "as of 2026-11-09" doc dates after publish; typescript-api.md hardcodes 26.10.28.
