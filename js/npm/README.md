# @seanchatmangpt/graphlaw

GraphLaw is an RDF law-state composition engine compiled to WebAssembly: parse 9 RDF
dialects, run SPARQL queries, validate with SHACL and ShEx, reason in N3, apply
RDF/RDFS/OWL-RL entailment regimes, evaluate datalog fixpoints, fire knowledge hooks,
execute vectorized columnar pushdowns over embedded DuckDB 2.0,
and run law pipelines with receipts and signed leases — ending in FOND policy admission.
Every op returns a typed result or a typed refusal over a versioned JSON ABI
(`ABI_VERSION = 1`).

## Quickstart

```sh
npm i @seanchatmangpt/graphlaw
```

Node >= 20.

One client, one honest admit: the `law` op runs its steps over your data *inside* the
wasm module and returns the final state with a receipt per transition.

```js
import { createClient } from '@seanchatmangpt/graphlaw';

const gl = await createClient();

const res = await gl.law({
  data: {
    text: '@prefix : <https://test.graphlaw.dev/> .\n:a :edge :b .\n:b :edge :c .\n',
    dialect: 'turtle',
  },
  steps: [{ step: 'rdfs' }],
});

console.log(res.ok);              // true
console.log(res.states.length);   // 2 — blake3 state ids, the input state first
console.log(res.states);          // ['blake3:4912113c…', 'blake3:54f96d06…']
console.log(res.receipts.length); // 1 — one receipt per transition
console.log(res.nquads.length);   // 18113 — the final state as N-Quads
```

That is the whole drop-in. `createClient()` initializes the WASM engine once and fails
fast on init errors. `res.receipts[0]` carries `authority` (here `"purrdf::entail"`),
`added` (the triples the step derived), and the `parent`/`child` blake3 state chain, so
the admission is replayable evidence, not a boolean.

Refusals are typed the same way. Hand SHACL a shape the data violates and the same call
throws a `GraphlawRefusalError` carrying the kernel's own answer — here the stable
register code `NotAdmitted`; every code lives in the typed refusal register,
[Refusals](https://github.com/seanchatmangpt/graphlaw/blob/main/docs/refusals.md):

```js
import { createClient, GraphlawRefusalError } from '@seanchatmangpt/graphlaw';

const gl = await createClient();

try {
  await gl.law({
    data: { text: '@prefix ex: <https://e/> . ex:a a ex:T .', dialect: 'turtle' },
    steps: [{
      step: 'shacl',
      shapes: '@prefix sh: <http://www.w3.org/ns/shacl#> . @prefix ex: <https://e/> .\n' +
        'ex:S a sh:NodeShape ; sh:targetClass ex:T ; sh:property [ sh:path ex:name ; ' +
        'sh:minCount 1 ] .',
    }],
  });
} catch (err) {
  if (err instanceof GraphlawRefusalError) {
    console.log(err.kind);    // "EngineRejected"
    console.log(err.code);    // "NotAdmitted" — details.code, the stable register code
    console.log(err.engine);  // "PurRdf"
    console.log(err.dialect); // "Turtle"
    console.log(err.message); // "SHACL admission refused: 1 violation(s)"
    console.log(err.details.violations[0].component);
    // "<http://www.w3.org/ns/shacl#MinCountConstraintComponent>"
  } else {
    throw err;
  }
}
```

Each op also has a `tryX()` variant (`tryLaw`, `tryParse`, `trySparql`, ...) that returns
an `{ ok, error }` result instead of throwing.

<!-- EXECUTED (2026-10-29, node 26): both snippets ran against the real pinned
     artifact through this package's client methods (`gl.law`, error fields above are
     the actual values). Init used skipDigestCheck because this checkout's
     dist-nodejs/ pin predates the relinked artifact — see Publish state. -->

## Publish state

**Publish state** (verified 2026-10-24): the latest published version on npm is
`26.10.14` (published pre-OIDC). The sources in this directory are version
`26.10.28`; that version is packed and CI-validated (`publish-npm` run
40084544193, validate-and-pack GREEN) but its publish is typed-blocked on the
one-time npm Trusted Publisher configuration (E404,
`docs/jira/v26.10.23/PUBLISH-STATUS.md`) — `npm i` today gets `26.10.14`.

**Checkout build state** (executed 2026-10-29): `js/npm/wasm/graphlaw.wasm` hashes to
blake3 `c92caeb6d56792587f2939769678e02b77c868f2919ef8566060e02f3c6c1b9c`
(62,909,537 B — the 2026-10-29 GL28-13 relink) while the built `dist-nodejs/` still
carries the pre-relink pin `c38b1fdb97ba462dc96000301d7ddf6a5109a7208c68a52f5ddd63ccbd6fda2c`;
`createClient()` on that dist throws `GraphlawIntegrityError` (witnessed: expected
`c38b1fdb…`, actual `c92caeb6…`) until `npm run build` re-pins. Published tarballs are
unaffected: `prepublishOnly` rebuilds, and the pin is regenerated from the artifact on
every build, so pin and bytes always ship together.

## Parse and SPARQL

The single ops work the same way as `law` — typed request in, typed result or typed
refusal out.

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

<!-- EXECUTED (2026-10-29, node 26): quads = 1, id = blake3:ffe6ff3…, sparql returned
     kind 'solutions' with the uri/literal Term row; same skipDigestCheck disclosure as
     Quickstart. -->

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
    console.log(err.kind); // "EngineRejected"
    console.log(err.code); // "refused" — the fallback when the refusal carries no
                           // details.code; a coded refusal surfaces its register code
    console.log(err.message);
  } else {
    throw err;
  }
}
```

Other error types: `GraphlawInitError`, `GraphlawIntegrityError`,
`GraphlawAllocRefusedError`, `GraphlawRequestTooLargeError`.

<!-- EXECUTED (2026-10-29, node 26): kind = "EngineRejected", code = "refused",
     message = "syntax error in Turtle (PurRdf) at line 1:1: …unexpected token…";
     same skipDigestCheck disclosure as Quickstart. -->

## Raw passthrough

`graphlaw/raw` exposes the JSON ABI verbatim — one `call` per op, same envelope the
WASM module speaks. Useful when you want the raw refusal envelope instead of thrown
errors, or are driving the ABI from generated code.

```js
import { callRaw } from '@seanchatmangpt/graphlaw/raw';

const res = await callRaw({ op: 'sniff', text: 'PREFIX : <http://ex/> :s :p :o .' });
// { ok: true, dialect: 'Turtle', engine: 'PurRdf', ... }
```

<!-- EXECUTED (2026-10-29, node 26): ok = true, dialect = "Turtle", engine = "PurRdf";
     callRaw is exported only from the graphlaw/raw subpath, not the main entry; same
     skipDigestCheck disclosure as Quickstart. -->

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
`c92caeb6d56792587f2939769678e02b77c868f2919ef8566060e02f3c6c1b9c`
(62,909,537 B; the ICU-drop relink, GL28-13 2026-10-29; verified across all four
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
