# @seanchatmangpt/graphlaw

GraphLaw is an RDF law-state composition engine compiled to WebAssembly: parse 9 RDF
dialects, run SPARQL queries, validate with SHACL and ShEx, reason in N3, apply
RDF/RDFS/OWL-RL entailment regimes, evaluate datalog fixpoints, fire knowledge hooks,
execute vectorized columnar pushdowns over embedded DuckDB 2.0,
and run law pipelines with receipts and signed leases — ending in FOND policy admission.
Every op returns a typed result or a typed refusal over a versioned JSON ABI
(`ABI_VERSION = 1`).

## Install

```sh
npm i @seanchatmangpt/graphlaw
```

Node >= 20.

**Publish state** (verified 2026-10-24): the latest published version on npm is
`26.10.14` (published pre-OIDC). The sources in this directory are version
`26.10.22`; that version is packed and CI-validated (`publish-npm` run
40084544193, validate-and-pack GREEN) but its publish is typed-blocked on the
one-time npm Trusted Publisher configuration (E404,
`docs/jira/v26.10.23/PUBLISH-STATUS.md`) — `npm i` today gets `26.10.14`.

## Quickstart

```js
import { createClient } from '@seanchatmangpt/graphlaw';

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
import { createClient, GraphlawRefusalError } from '@seanchatmangpt/graphlaw';

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
import { callRaw } from '@seanchatmangpt/graphlaw/raw';

const res = await callRaw({ op: 'sniff', text: 'PREFIX : <http://ex/> :s :p :o .' });
// { ok: true, dialect: 'Turtle', engine: 'PurRdf', ... }
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

## WASM ABI contract

The engine is a single cdylib exporting four C ABI functions (see
`wasm/src/ffi.rs`, `docs/WASM.md`). The wrapper in this package drives them for
you; if you drive them yourself, the discipline is:

- `gl_alloc(len) -> ptr` — the host reserves `len` bytes and writes a UTF-8 JSON
  request. A `null` return means the request hit a memory wall (below) and the
  next `gl_call(0, 0)` on that null buffer yields a typed error response, never a
  trap.
- `gl_call(ptr, len) -> packed` — executes one request and **consumes** the input
  buffer (a side-table entry, freed on the wasm side; do not reuse the pointer).
  Returns `(out_ptr << 32) | out_len`.
- `gl_free(ptr, len)` — releases a buffer this module handed out, **once**, with
  the exact declared length. Unknown pointers or wrong lengths are ignored;
  a second free is ignored. Safe against a hostile `(ptr, len)`.
- **Memory walls**: `gl_alloc` refuses (returns `null`) when `len` exceeds the
  per-request JSON limit or the outstanding-byte cap (see `LIMITS` from the main
  entry point). Refusal is typed, not a trap.
- **Receipts**: receipts returned by law/policy ops carry a 64-byte Ed25519
  signature over the payload bytes; verify with `graphlaw-verify` or the
  Ed25519 API of your choice.

## WASM artifact state

The shipped `wasm/graphlaw.wasm` is pinned by BLAKE3
`58a4409505f32b5d9aebc8c0ce2e2adeaba05c8c4b55f951a244ab4ad7d2280a`
(58,124,612 B; the DuckDB-linked new-EH artifact; verified across all four
pin surfaces including this generated file — `src/generated/artifact-meta.ts`;
the digest is re-pinned on every
`npm run build`). The host for these bytes is the wasmtime-48 EH runner
(`graphlaw-run-eh`); the wasmi runner was retired 2026-10-24
(`docs/jira/v26.10.23/ADR-wasmi-retirement.md`). A SQL-in-RDF document
parsed as `sql` returns the same dataset id as the same bytes parsed as
Turtle.

Historical pins (prior generations, superseded):

- `472d5557ba8311c0fab68bed044be9c44254e65fd523a0ff00684c0869bf80ea`
  (58,121,500 B — the settle-commit pin at `d9cab2f`, superseded by the
  security-wave re-pin)
- `fa1558306690493249b176ec312a4f01e97798937e05cd49f2946d61a7e6d107`
  (57,781,384 B — the EH-closure settle pin)
- `f149433c6e1a84ec95e961ac92b16b65b28e15d6fd8546beb1779d3b54a02e7e`
  (7,104,147 B)
- `332ea2188fafb465666e71407e8f940c5b60a15184e6ad6567bb4f723fe2c30c`
  (6,793,617 B) — did not return the same dataset id across `sql` and
  Turtle parses.

`test/artifact.test.ts` classifies the artifact by size: the legacy band
(< 10 MB) covers the historical thin builds; the engine band (40–64 MiB)
covers the DuckDB-linked builds. A pin outside either band fails the
court.

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
