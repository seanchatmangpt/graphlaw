# js

JavaScript/TypeScript distribution of GraphLaw: the npm package `graphlaw`, a
typed client over the pinned wasm32-wasip1 artifact — no wasm-bindgen, no
second implementation of any engine.

## What this is

Package `graphlaw` wraps the single byte-pinned wasm32-wasip1 artifact built
from this repo. Node (>= 20, ESM) and browsers/bundlers use the same
zero-dependency WASI preview1 stub set — the wasm module imports only 7
trivial `wasi_snapshot_preview1` functions, satisfied by a self-contained stub
shim in the client, so no external WASI runtime or shim package is needed
anywhere. There is no wasm-bindgen layer — the
same artifact bytes are driven through the JSON ABI in every runtime, so
Node, browser, and bundler builds differ only in WASI plumbing, never in
engine behavior.

## Layout

| path | contents |
|---|---|
| `npm/` | package root (`graphlaw`) |
| `npm/src/` | hand-written TypeScript sources (client, runtime, wire, errors, limits) |
| `npm/src/generated/` | generated from `registry/capability-registry.json` — do not edit |
| `npm/scripts/` | codegen and build scripts (`gen-types.mjs`, `build.mjs`, `sync-version.mjs`) |
| `npm/test/` | vitest suites (node project, browser project, corpus, limits, forward-compat) |
| `HANDWRITTEN.md` | ledger of hand-written vs generated surfaces |

## Build and test

```sh
cd js/npm
npm install
npm run gen      # regenerate src/generated/ from registry/capability-registry.json
npm run build    # gen + bundle (build.mjs) + tsc -p tsconfig.build.json
npm test         # vitest run --project node
```

Browser suite: `npm run test:browser`. Lint/pack checks: `npm run lint:pkg`
(`publint` + `attw`).

## The three build targets

The build emits exactly three targets, never a root `dist/`:

- `dist-nodejs/` — ESM for Node (>= 20), driven by the WASI preview1 stub set
  in `src/runtime.ts` (the package `main`)
- `dist-web/` — browser build; web and bundler targets share
  `src/runtime.web.ts`
- `dist-bundler/` — neutral build for webpack/esbuild/rollup consumers

`dist-web/` and `dist-bundler/` are assembled by `scripts/assemble.mjs` from
the same sources; only the runtime module differs (`runtime.web.ts` for
web/bundler, `runtime.ts` for nodejs).

The `exports` map in `package.json` routes each condition (`node`,
`browser`, default) to its target.

## Generated vs handwritten

`npm/src/generated/` is a projection of `registry/capability-registry.json`
via `npm run gen`; edits there are overwritten. Everything else under
`npm/src/` is hand-written and tracked, with its status, in
[HANDWRITTEN.md](HANDWRITTEN.md).

## Version sync

The npm version is derived, not declared: `npm run sync-version`
(`scripts/sync-version.mjs`) reads the root `Cargo.toml` version and writes
it into `package.json`, failing loudly on drift. One release train, one
version number.
