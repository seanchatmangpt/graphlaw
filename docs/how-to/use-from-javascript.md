# Use graphlaw from JavaScript or TypeScript

Call the full GraphLaw engine (parse, SPARQL, SHACL, ShEx, N3, entailment,
datalog, hooks, law, policy) from Node or the browser through the `graphlaw`
npm package: typed methods over the same JSON ABI the WASM module exposes.
Reference: [abi-reference](../abi-reference.md) · [refusals](../refusals.md).

## Install

```sh
npm i graphlaw
```

Node >= 20 is required for the Node entry point (`engines.node: ">=20"`).
The package ships `dist-nodejs/`, `dist-web/`, and `dist-bundler/` builds selected
by package `exports` conditions.

## Initialize and call

Create one client per process; the WASM module is loaded and instantiated once
and cached. Every method returns typed response objects.

```ts
import { createClient } from 'graphlaw';

const gl = await createClient();

// 1. Parse a Turtle document.
const parsed = await gl.parse({
  text: '@prefix ex: <http://example.org/> .\nex:alice ex:knows ex:bob .',
  dialect: 'turtle',
});
console.log(parsed.ok); // true

// 2. Run a SPARQL SELECT over a data spec (dialect sniffed when omitted).
const result = await gl.sparql({
  data: {
    text: '@prefix ex: <http://example.org/> .\nex:alice ex:knows ex:bob .',
  },
  query: 'SELECT ?who WHERE { ?who <http://example.org/knows> ?whom }',
});

if (result.kind === 'solutions') {
  console.log(result.variables); // ["who"]
  for (const row of result.rows) {
    console.log(row[0]); // term object: { type: "uri", value: "http://example.org/alice" }
  }
}
```

A SPARQL response is a tagged union on `kind`: `solutions` (`variables`,
`rows` of term-or-null cells), `graph` (`nquads`, `quads`), or `boolean`
(`value`). See the [`sparql` op](../abi-reference.md#6-sparql) for the full
shape.

## Handle refusals

The ergonomic client throws a `GraphlawRefusalError` when the engine returns a
refusal envelope (`{ok: false}`); the error carries the refusal's typed fields.
Every method also has a `tryX()` variant that returns the envelope as a
`Result` instead of throwing — pick one style per call site, not both.

```ts
import { GraphlawRefusalError } from 'graphlaw';

try {
  await gl.parse({ text: 'just some prose, not RDF' });
} catch (e) {
  if (e instanceof GraphlawRefusalError) {
    console.log(e.code, e.kind, e.details); // e.code comes from details.code
  }
}

const r = await gl.tryParse({ text: 'just some prose, not RDF' });
if (!r.ok) console.log(r.error.kind, r.error.message);
```

| `kind` | Meaning |
|---|---|
| `NotSemanticContent` | input is not RDF/semantic content |
| `Ambiguous` | input matches more than one dialect |
| `EngineRejected` | the backing engine refused the input |
| `Unsupported` | unknown op, missing field, or unsupported feature |
| `ResourceLimit` | a documented ABI limit was exceeded |

`details` is present only when the refusal carries a machine-readable `code`;
full taxonomy: [refusals](../refusals.md).

## Raw passthrough

When you need the JSON ABI verbatim (scripting, golden tests, or ops the typed
client does not wrap yet), use the `graphlaw/raw` entry point. It returns the
envelope as-is and never throws for ABI outcomes — refusals come back as
`{ok: false, error: ...}` values, not exceptions.

```ts
import { callRaw, callRawJson } from 'graphlaw/raw';

const envelope = await callRaw({
  op: 'sniff',
  text: '<urn:a> <urn:b> <urn:c> .',
});
if (envelope.ok) console.log(envelope.dialect); // "NTriples"

const json = await callRawJson('{"op":"capabilities"}');
```

## Runtimes

- **Node**: Node >= 20 (ESM); reads the bundled artifact from the installed
  package.
- **Browser / bundler**: `browser` and `default` export conditions map to
  `dist-web/` and `dist-bundler/` builds for browsers and bundlers.
- **WASI**: no external WASI runtime or shim package is needed. The wasm
  module imports only 7 trivial `wasi_snapshot_preview1` functions, satisfied
  by a zero-dependency self-contained stub set (empty `environ`, `sched_yield`
  no-op, `random_get` via `node:crypto` entropy, `fd_write` discard,
  `clock_time_get` via a `performance.now` monotonic clock, `proc_exit` typed
  throw) — identical on Node and browsers.
- **Integrity**: at load, the artifact's SHA-256 is computed with
  `crypto.subtle` and compared against the pinned digest in
  `generated/artifact-meta.js`; a mismatch throws `GraphlawIntegrityError`
  with `expected`/`actual` digests before any code runs. For local
  development with a rebuilt artifact, `initGraphlaw({ skipShaCheck: true })`
  bypasses the check — dev only, you are trusting the bytes.

## Limits

The ABI enforces caps before parsing or heavy work: a 16 MiB request cap
(`MAX_REQUEST_BYTES`) is enforced client-side and throws
`GraphlawRequestTooLargeError` (with `observed` and `max` byte counts) before
the module is called; JSON depth, plan actions, atoms per field, policy
entries, and N3 iteration caps surface as `ResourceLimit` refusals. Full
table: [abi-reference](../abi-reference.md#limits).
