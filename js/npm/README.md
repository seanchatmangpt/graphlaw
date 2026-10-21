# graphlaw

GraphLaw is an RDF law-state composition engine compiled to WebAssembly: parse 9 RDF
dialects, run SPARQL queries, validate with SHACL and ShEx, reason in N3, apply
RDF/RDFS/OWL-RL entailment regimes, evaluate datalog fixpoints, fire knowledge hooks,
execute vectorized columnar pushdowns over embedded DuckDB 2.0,
and run law pipelines with receipts and signed leases — ending in FOND policy admission.
Every op returns a typed result or a typed refusal over a versioned JSON ABI
(`ABI_VERSION = 1`).

## Install

```sh
npm i graphlaw
```

Node >= 20.

## Quickstart

```js
import { createClient } from 'graphlaw';

const gl = await createClient();

const parsed = await gl.parse({
  text: '<http://ex/s> <http://ex/p> "hello" .',
  dialect: 'turtle',
});
console.log(parsed.quads); // 1
console.log(parsed.id);    // blake3:... content-addressed state id

const res = await gl.sparql({
  data: { text: '<http://ex/s> <http://ex/p> "hello" .', dialect: 'turtle' },
  query: 'SELECT ?s ?o WHERE { ?s ?p ?o }',
});
// res is a kind-tagged union: {kind:'solutions'} | {kind:'graph'} | {kind:'boolean'}
if (res.kind === 'solutions') {
  for (const row of res.rows) console.log(row); // cells are Term objects, null = unbound
}
```

`createClient()` initializes the WASM engine once and fails fast on init errors. Each
op also has a `tryX()` variant (`tryParse`, `trySparql`, ...) that returns an
`{ ok, error }` result instead of throwing.

## Typed errors

Refusals from the engine are thrown as typed error subclasses of `GraphlawError`;
refusals that carry a machine-readable code surface as `GraphlawRefusalError`.

```js
import { createClient, GraphlawRefusalError } from 'graphlaw';

const gl = await createClient();

try {
  await gl.parse({ text: 'not rdf', dialect: 'turtle' });
} catch (err) {
  if (err instanceof GraphlawRefusalError) {
    console.log(err.kind); // e.g. "EngineRejected"
    console.log(err.code); // machine-readable code, when present
    console.log(err.message);
  } else {
    throw err;
  }
}
```

Other error types: `GraphlawInitError`, `GraphlawIntegrityError`,
`GraphlawAllocRefusedError`, `GraphlawRequestTooLargeError`.

## Raw passthrough

`graphlaw/raw` exposes the JSON ABI verbatim — one `call` per op, same envelope the
WASM module speaks. Useful when you want the raw refusal envelope instead of thrown
errors, or are driving the ABI from generated code.

```js
import { callRaw } from 'graphlaw/raw';

const res = await callRaw({ op: 'sniff', text: 'PREFIX : <http://ex/> :s :p :o .' });
// { ok: true, dialect: 'turtle', ... }
```

## Runtimes

- **Node >= 20** — the `node` export condition resolves to `dist-nodejs/`, which
  instantiates the WASM module with zero-dependency WASI preview1 stubs (the cdylib
  needs only 7 trivial imports, served in-package — no `node:wasi`, no shim packages,
  no native addons, no postinstall).
- **Browser / bundler** — both the `browser` and `default` conditions resolve to
  `dist-web/`, which serves the module via `fetch` with the same zero-dependency
  WASI stubs, so Webpack/esbuild/Vite-style bundlers need no shim packages.
  TypeScript types ship as `.d.ts` for every entry point.

Both runtimes share the same WASM core and the same typed API surface.

For custom artifacts or dev setups, `initGraphlaw()` accepts `wasmBytes` and
`skipDigestCheck` options (see the type docs for `GraphlawInitOptions`).

## Resources

- [ABI reference](https://github.com/seanchatmangpt/graphlaw/blob/main/docs/abi-reference.md) —
  every op, the envelope, limits, and the WASM calling convention.
- [Refusals](https://github.com/seanchatmangpt/graphlaw/blob/main/docs/refusals.md) —
  refusal kinds and machine-readable codes.
- [Repository](https://github.com/seanchatmangpt/graphlaw)
- `js/npm/PUBLISHING.md` in the repository covers release/publish operations for
  this package.

## Upstream notice

graphlaw is a sovereign reasoning engine, offered under the MIT license.
