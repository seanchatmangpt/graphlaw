# JS Test Receipt — L107 (2026-10-22, lane L107, /Users/sac/graphlaw)

First local witnessing this session of the js.yml surfaces. All runs are real
executions in `js/npm/`; no code was modified (read-only lane).

## 1. `npm run build` — PASS (exit 0)

```
> npm run gen && node scripts/build.mjs && tsc -p tsconfig.build.json && node scripts/assemble.mjs

generated src/generated/types.ts from /Users/sac/graphlaw/registry/capability-registry.json
build: wasm/graphlaw.wasm in place (blake3 332ea2188fafb465666e71407e8f940c5b60a15184e6ad6567bb4f723fe2c30c)
assemble: dist-nodejs ready
assemble: dist-web ready (runtime -> runtime.web)
assemble: dist-bundler ready (runtime -> runtime.web)
EXIT=0
```

## 2. Unit suite — FAIL (exit 1): `npm run test` (vitest run, 11 files)

```
 Test Files  1 failed | 10 passed (11)
      Tests  1 failed | 197 passed (198)
   Duration  3.28s
VITEST_EXIT=1
```

Failure (pre-existing; this lane changed nothing):

```
FAIL  test/limits.test.ts > limits mirror the registry > LIMITS keys and values equal
registry/capability-registry.json `limits`
AssertionError: max_outstanding_alloc_bytes: expected 268435456 to be 16777216
 ❯ test/limits.test.ts:71:55
```

Root cause: `js/npm/src/limits.ts:41` has `max_outstanding_alloc_bytes: 268435456`
but `registry/capability-registry.json:287` says `16777216`. The js mirror is
stale vs the registry. Every other limit matches.

## 3. Browser suite — FAIL, typed (exit 1), not a graceful skip

Two runs:

- **Skip form** (no `GRAPHLAW_BROWSER_TESTS`, run within `npm run test`):
  skip reason verbatim from `test/browser.test.ts:29-35`:
  `GRAPHLAW_BROWSER_TESTS !== 1` — suite skips gracefully, recorded, NOT a pass.
- **Enabled form** `GRAPHLAW_BROWSER_TESTS=1 npm run test:browser`:
  ```
  FAIL  test/browser.test.ts > browser runtime in headless chromium (null) > ...
  Error: browserType.launch: Executable doesn't exist at
  /Users/sac/Library/Caches/ms-playwright/chromium_headless_shell-1248/...
  BROWSER_EXIT=1
  ```
  playwright-core is importable and `dist-web` is built, so the gate does not
  self-skip; it hard-fails on the missing Playwright-managed Chromium binary.
  Host has `/Applications/Google Chrome.app` only, and `test/browser.test.ts:108`
  hardcodes `pw.chromium.launch({ headless: true })` with no `executablePath`/
  `channel` escape hatch, so system Chrome cannot substitute without a code
  change (out of scope for this read-only lane). Local equivalent:
  `npx playwright install chromium` would unblock it.

## 4. Drift gates — ALL PASS (exit 0 x3)

- `node scripts/sync-version.mjs && git diff --exit-code -- js/npm/package.json`
  → `sync-version: already at 26.10.21`, exit 0.
- `node scripts/build.mjs && git diff --exit-code -- js/npm/src/generated`
  → rebuild emitted blake3 `332ea2188fafb465666e71407e8f940c5b60a15184e6ad6567bb4f723fe2c30c`,
  zero diff vs committed, exit 0.
- `node scripts/gen-types.mjs && git diff --exit-code -- js/npm/src/generated/types.ts`
  → exit 0.
- Artifact pin byte check: `b3sum priv/graphlaw.wasm` =
  `awk '{print $1}' priv/graphlaw.wasm.blake3` =
  `332ea2188fafb465666e71407e8f940c5b60a15184e6ad6567bb4f723fe2c30c`. Match.

## Verdict

| Surface | Verdict | Exit |
|---|---|---|
| build | PASS | 0 |
| unit (vitest) | **FAIL** — 197/198, limits mirror stale | 1 |
| browser | **FAIL** (typed: Chromium binary missing; skip form recorded verbatim) | 1 |
| drift gates (x3) | PASS | 0 |

Findings for the coordinator:

1. `js/npm/src/limits.ts:41` `max_outstanding_alloc_bytes: 268435456` must be
   `16777216` to match `registry/capability-registry.json` — js.yml `js-test`
   would fail on CI with this tree.
2. The browser court cannot run locally until `npx playwright install chromium`
   is run (or the test accepts `channel: 'chrome'`); currently exit 1, not a
   silent skip, when `GRAPHLAW_BROWSER_TESTS=1`.

## Re-run — 2026-10-22, lane N9

Environment: /Users/sac/graphlaw/js/npm, node >=20, wasm pin untouched
(graphlaw.wasm blake3 332ea2188fafb465666e71407e8f940c5b60a15184e6ad6567bb4f723fe2c30c,
re-confirmed by build output; no ABI digest changed).

| # | Command (cwd js/npm) | Exit | Result |
|---|---|---|---|
| 1 | `npm run build` | 0 | gen -> build.mjs (wasm in place, blake3 332ea218...) -> tsc -> assemble; dist-nodejs/dist-web/dist-bundler ready |
| 2 | `npm run test` | 0 | Test Files 11 passed (11); Tests 198 passed (198); Duration 8.48s |
| 3 | `npm run sync-version` | 0 | `sync-version: already at 26.10.21` (no drift) |
| 4 | `npm run gen` (idempotence) | 0 | regenerated types.ts matches in-tree registry/capability-registry.json; no further diff. Diff vs HEAD (+FrozenDuckdb engine, +SqlRdf/"sql" dialect, +DuckDb refusal code) tracks the already-modified registry — pre-existing milestone drift, not session-introduced |
| 5 | `GRAPHLAW_BROWSER_TESTS=1 npm run test:browser` | **1** | 1 failed (1): test/browser.test.ts:108 — `browserType.launch: Executable doesn't exist at /Users/sac/Library/Caches/ms-playwright/chromium_headless_shell-1248/...` (Playwright chromium not installed). Typed, loud fail, not a silent skip. Recovery per receipt note 2: `npx playwright install chromium` (or accept `channel: 'chrome'`) |

Counts: host JS suite 198/0 (matches L112's 198/0 witness; limits.ts fix holds —
test/limits.test.ts 5/5 green). Browser suite exit 1 = BLOCKED on missing
chromium binary, environment-dependent, unchanged from prior entry.
