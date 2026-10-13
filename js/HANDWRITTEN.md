# handwritten

The generated surface is a projection of a source of truth (the capability
registry and the `graphlaw-wasm.ttl` ontology); hand edits to generated files
are refused by CI drift checks (`scripts/gen-types.mjs --check`,
`scripts/build.mjs`). Edit the source, re-generate. This ledger follows the
`zcode-cli` HANDWRITTEN pattern: list the irreducible handwritten residue and
the source of truth for everything else.

## Ledger

| File | Generated? | Source of truth |
| --- | --- | --- |
| `npm/src/generated/` (types.ts, artifact-meta.ts) | GENERATED | Committed and CI drift-guarded: the `js-drift` job in `.github/workflows/js.yml` rebuilds the wasm, re-pins via `build.mjs`, regenerates types via `gen-types.mjs`, and fails on any `git diff` in `npm/src/generated/` or `npm/package.json` |
| `npm/src/generated/types.ts` | GENERATED | `registry/capability-registry.json` via `npm/scripts/gen-types.mjs`; drift check fails CI on hand edit |
| `npm/src/generated/artifact-meta.ts` | GENERATED | `priv/graphlaw.wasm` via `npm/scripts/build.mjs` (artifact hash + ABI metadata) |
| `npm/src/limits.ts` | GENERATED-ADJACENT | Mirror of the limits in `registry/capability-registry.json`; kept in sync by test (`test/limits.test.ts`), not re-rendered |
| `npm/src/wire.ts` | HAND-WRITTEN | Irreducible residue: wire protocol port of `tests/wasm_abi.rs` (gl_* packed-u64 ABI framing) |
| `npm/src/runtime.ts` | HAND-WRITTEN | Irreducible residue: port of the Rust reference client (`tests/wasm_abi.rs`) calling convention plus a minimal `wasi_snapshot_preview1` stub surface. Zero-dependency; no `node:wasi`. Stub set derived from `wasm-objdump -x priv/graphlaw.wasm` imports — REVISIT if the artifact's import set changes. |
| `npm/src/runtime.web.ts` | HAND-WRITTEN | Browser twin of `runtime.ts`: same calling convention and WASI stub surface, browser-compatible instantiation (no Node APIs) |
| `npm/src/init.ts` | HAND-WRITTEN | Irreducible residue: module instantiation / artifact loading |
| `npm/src/errors.ts` | HAND-WRITTEN | Irreducible residue: typed error classes |
| `npm/src/client.ts`, `npm/src/raw.ts` | HAND-WRITTEN | Irreducible residue: ergonomic client and raw passthrough |
| `npm/src/index.ts`, `npm/src/types.ts` | HAND-WRITTEN | Irreducible residue: public surface re-exports and hand types |
| `npm/scripts/gen-types.mjs` | HAND-WRITTEN | Generator itself: registry -> `src/generated/types.ts` (with `--check` drift mode) |
| `npm/scripts/build.mjs` | HAND-WRITTEN | Generator itself: artifact hash/ABI -> `src/generated/artifact-meta.ts` |
| `npm/scripts/sync-version.mjs` | HAND-WRITTEN | Version sync from the workspace/Cargo source of truth into `npm/package.json` |
| `npm/scripts/assemble.mjs` | HAND-WRITTEN | Dist assembly: stages generated + hand-written sources into the publishable `dist/` layout |
| `npm/scripts/pack-smoke.mjs` | HAND-WRITTEN | Pack audit: smoke-checks the packed tarball surface before publish |
| `npm/wasm/graphlaw.wasm` | GENERATED | ggen-rendered from `ontologies/graphlaw-wasm.ttl`, byte-pinned by `priv/graphlaw.wasm.sha256`; NEVER hand-edited; rebuild only via `cargo build -p graphlaw-wasm --target wasm32-wasip1 --profile wasm` |
