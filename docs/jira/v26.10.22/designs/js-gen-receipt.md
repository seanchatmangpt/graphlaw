# js/npm regen + limits.ts provenance fix — Lane M13 receipt

Date: 2026-10-22. Lane M13 (guard builder). Lease: `js/npm/src/generated/` (regen only),
`js/npm/src/limits.ts` (comment fix), this file. Defect class AA9: stale
`registry_blake3` pins in generated TS + provenance comment.

## Commands and exits

```text
$ cd /Users/sac/graphlaw/js/npm && npm run gen
> @seanchatmangpt/graphlaw@26.10.21 gen
> node scripts/gen-types.mjs
generated src/generated/types.ts from /Users/sac/graphlaw/registry/capability-registry.json
(exit 0)

$ cd /Users/sac/graphlaw/js/npm && npm test
Test Files  11 passed (11)
      Tests  198 passed (198)
   Duration  3.51s
(exit 0)
```

## Before/after digests (sha256)

| file | before | after |
|---|---|---|
| `js/npm/src/generated/types.ts` | `d931750a452c830b3a4dde374927ee3f6069ba22fb99a86501e888d295a1b39f` | `02cd89d366290b05deedbd9dd21f060b08769e9a81c95a8be667ae2714f0a128` |
| `js/npm/src/limits.ts` | `0e37ae8257d33965a14594cec17585f1b6517938f5ad8b4d0f4bf2d333352549` | `9b67820be80a8ac7e63e8ae2917ff630ee1d9c68a0baaaab9eca63565d2069b2` |

## Pin flip

- `src/generated/types.ts` line 2 header, via documented regen (`npm run gen`, real
  generator — not hand-edit):
  `blake3:22b7478c47293741a17c0ec1f884472bac8fcb0ec7320b0925b48aa04d80908b` →
  `blake3:e227367e13ce8110d97fe29f252bdb69e22e872a5be4117ccbce4bbc9c6477d7`
  (matches `registry/ARTIFACTS.blake3` registry row).
- `src/limits.ts` line 12 provenance comment (single-line hand edit, lease allows):
  `blake3:1c129dd6…` → `blake3:e227367e…`.

## Verification

Real `npm test` output above: 11 files / 198 tests, 0 failures, including
`test/limits.test.ts` (5 passed) and `test/generated-types.test.ts` (5 passed) —
the drift guards that previously failed on the stale pin now pass against the
current registry. No SKIPs in the drift tests.

## Falsifier

Revert either pin to the old digest → `test/limits.test.ts` /
`test/generated-types.test.ts` must fail; rerun `npm run gen` restores.
