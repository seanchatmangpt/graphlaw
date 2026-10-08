# ABI reference

The JSON ABI (feature `abi`, `src/abi.rs`) exposes every GraphLaw capability as UTF-8 JSON in and
out. `ABI_VERSION` is `1`. The machine-readable form is the
[capability registry](capability-registry.md); this page is the prose companion, re-read from
`src/abi.rs` and `src/law.rs`.

## Envelope

Request: `{"op": "<name>", ...}`. Response on success: the op's fields plus `"ok": true`. Response
on refusal:

```json
{"ok": false,
 "error": {"kind": "EngineRejected", "engine": "PurRdf", "dialect": "Turtle",
           "message": "...", "details": {"code": "NotAdmitted"}}}
```

- `kind` is one of `NotSemanticContent`, `Ambiguous`, `EngineRejected`, `Unsupported`,
  `ResourceLimit`.
- `engine` (`PurRdf` or `Eyeron`) and `dialect` are `null` when not applicable.
- `details` is present only for refusals that carry a machine-readable `code`; see
  [Refusals](refusals.md).
- A request that is not JSON, lacks `op`, names an unknown op, or lacks a required field is refused
  with kind `Unsupported` and no `details`.

WebAssembly calling convention: exports `gl_alloc(len)`, `gl_free`, `gl_call(ptr, len)`, memory.
`gl_call` returns `(out_ptr << 32) | out_len`. See the README for detail.

## Limits

Checked before or during the call; exceeding one is a `ResourceLimit` refusal with
`details.code = "ResourceLimit"`.

| `limit` name | constant | value |
|---|---|---|
| `request_bytes` | `MAX_REQUEST_BYTES` | 16,777,216 |
| `json_depth` | `MAX_JSON_DEPTH` | 64 |
| `plan_actions` | `MAX_PLAN_ACTIONS` | 1,000 |
| `atoms_per_field` | `MAX_ATOMS_PER_FIELD` | 10,000 |
| `policy_entries` | `MAX_POLICY_ENTRIES` | 100,000 |
| `n3_iterations` | `law::N3_MAX_ITERATIONS` | 4,000 |

## Data spec versus flat text

The ABI is deliberately not uniform, and hosts must follow it per op.

- **Data spec** `{"text": "...", "dialect"?: "turtle", "hint"?: "ttl", "base"?: "..."}`: an object
  under `data` (and `pack` for `hooks`). Without `dialect` the router sniffs the content. A bare
  string where a data spec is required is refused (`"data" must be a data spec object`).
  Used by: `canonical`, `sparql`, `shacl`, `shex`, `entail`, `hooks`, `law`.
- **Flat text**: `text`, `dialect`, `hint`, `base` at the top level of the request. Used by:
  `sniff`, `parse`, `convert`, `n3` (which takes only `text`).
- `policy` takes `problem` and `policy` as a JSON object or array or a JSON string.

Dialect names accepted on the wire, case-insensitive: `turtle`/`ttl`, `trig`, `ntriples`/`nt`,
`nquads`/`nq`, `rdfxml`/`rdf`/`owl`, `jsonld`, `yamlld`, `trix`, `hextuples`/`hext`, `n3`,
`shexc`/`shex`, `shexj`, `sparql`/`rq`. Unknown names are refused (`Unsupported`).

## Terms

SPARQL solution cells and ShEx entry nodes are term objects:

| `type` | fields |
|---|---|
| `uri` | `value` |
| `bnode` | `value` (label) |
| `literal` | `value`, `datatype`, and `xml:lang` when a language tag exists |
| `triple` | `value` (Debug rendering of the quoted triple) |

An unbound SPARQL cell is JSON `null`. SHACL result fields are different: `focus`, `path`,
`value`, `shape` and `component` are Display strings, not term objects.

## Ops

Fourteen ops, in registry order.

### 1. `capabilities`

Request: none. Response: `abi`, `abi_version`, `crate`, `authorities[]` (`capability`,
`authority`, `revision`), `rdf_dialects[]`, `other_dialects[]`, `ops[]`. From v26.9.29 also
`registry_schema`, `registry_sha256`, `surface_sha256` (see
[capability registry](capability-registry.md)). In the pre-v26.9.29 source the lists are literals
in `abi.rs`; the contract moves them to `src/registry.rs`.

### 2. `sniff`

Request: `text`*, `hint`. Response: `dialect`, `engine` (Debug names, for example `Turtle`,
`PurRdf`). Refuses with `NotSemanticContent` or `Ambiguous`.

### 3. `parse`

Request (flat): `text`*, `dialect`, `hint`, `base`. Response: `dialect`, and either `quads`
and `id` (RDF dialects) or `valid: true` (non-RDF dialects such as `n3`, `sparql`, `shexc`,
`shexj`, which are syntax-checked only).

### 4. `convert`

Request (flat): `text`*, `to`*, `dialect`, `hint`, `base`. `to` must name an RDF dialect.
Response: `text` (serialization of the dataset), `id`.

### 5. `canonical`

Request: `data`* (data spec). Response: `id` (`sha256:` of the RDFC-1.0 canonical form), `nquads`,
`quads`.

### 6. `sparql`

Request: `data`*, `query`*, `base`. Response is a tagged union on `kind`:

| `kind` | fields |
|---|---|
| `solutions` | `variables` (list of strings), `rows` (list of rows of term-or-null) |
| `graph` | `nquads`, `quads` |
| `boolean` | `value` |

### 7. `shacl`

Request: `data`*, `shapes`* (Turtle string, not a data spec), `base`. Response: `conforms`,
`results[]` each with `focus`, `path` (nullable), `value` (nullable), `severity` (Debug name),
`component`, `shape`, `message`. A non-conforming graph is a normal `ok` response; refusal happens
only for unusable shapes or data.

### 8. `shex`

Request: `data`*, `schema`* (string), `schema_dialect` (`shexc` default, or `shexj`), `map`*
(shape map string), `base`. Response: `conforms`, `entries[]` each with `node` (term object),
`shape` (Debug string), `status` (Debug string), `reason`.

### 9. `n3`

Request (flat): `text`* (N3 with rules). Response: `derived` (the reasoner output; type `any`).
Bounded by `n3_iterations`.

### 10. `entail`

Request: `data`*, `regime`* (`simple`, `rdf`, `rdfs`, `owl-rl`, `d`; anything else is refused).
Response: `nquads` (closure), `added` (quads beyond the input).

### 11. `datalog`

Request: `rules`* (list of `{head, body}`; atoms are `[s, p, o]` where `p` is an IRI string;
terms starting with `?` are variables, terms starting with `"` are literals, others are IRIs),
`facts`* (list of `[s, p, o]` strings). Response: `count`, `facts` (sorted `[s, p, o]` triples).

### 12. `hooks`

Request: `pack`* (data spec holding `kh:` hooks), `data`* (data spec). Response: `id`, `rounds`,
`quads`, `nquads`, `firings[]` (`hook`, `round`, `added`, `row`).

### 13. `law`

Request: `data`* (data spec), `steps`* (list of step objects), lease fields (below). Response:
`states` (state ids: input first, then one per step), `receipts[]`, `nquads` (final state).

Receipt objects carry `step`, `parent`, `child`, `added`, `authority`, `revision`, and
`lease_id` when a lease is in force. Plan-action receipts also carry `plan_sha256` and `index`.

Steps, with the ceiling a lease must grant (from `Step::required_ceiling`) and the lease scope
name (from `Step::name`):

| wire step | fields | scope name | ceiling |
|---|---|---|---|
| `shacl` | `shapes`* | `admit:shacl` | observe |
| `n3` | `rules`* | `derive:n3` | construct |
| `rdfs` | none | `derive:rdfs` | construct |
| `owl-rl` | none | `derive:owl-rl` | construct |
| `hooks` | `pack`* (data spec) | `derive:hooks` | construct |
| `plan` | `plan`* (`actions[]` of `name`, `pre`, `pre_not`?, `add`, `del`; `goal`, `goal_not`?; N-Triples strings) | `admit:plan` | select |
| `record-receipts` | none | (not leased) | none |
| `require-receipt` | `step_name`* | `admit:require-receipt` | observe |
| `require-signed-receipt` | `step_name`*, `trusted_keys`* | `admit:require-signed-receipt` | observe |

`record-receipts` writes every receipt produced so far into the state; `require-receipt` refuses
unless that step's receipt is recorded; `require-signed-receipt` also needs a valid attestation
by one of the step's trusted keys. `plan` yields one receipt per action.

Lease request fields (all optional; give at most one of `signed_lease` and `lease`):

- `signed_lease`: `{"lease": {id, holder, ceiling, scope[], expires_unix, issued_unix?},
  "attestation": {key_id, payload_sha256, signature}}` plus `trusted_keys` (hex Ed25519 public
  keys) and optional `max_skew_secs` (default 60). Verified offline; expiry is judged by the
  module's own clock and any `now_unix` is ignored.
- `lease` with `unverified_lease: true` and `now_unix`: an unsigned lease at a caller-chosen time.
  Without `unverified_lease: true` the request is refused (`UnverifiedLeaseRefused`). It proves
  nothing about who issued the lease.
- `ceiling` is `observe`, `select` or `construct`.

### 14. `policy`

Request: `problem`*, `policy`* (each a JSON object/array or a JSON string). Strong-cyclic
admission of a FOND policy. Response: `initial_states`, `reachable`, `goal_states`, `entries`
(list of `[state, action]`), `ntriples`. Refusals: `PolicyRefused`.

## Not in the ABI

Signing keys, key distribution, and revocation are outside the module; the caller supplies
`trusted_keys`. The module holds no state between calls.

## See Also

- [Capability registry](capability-registry.md)
- [Refusals](refusals.md)
- [API stability](api-stability.md)

External sibling (`../ggen-marketplace`, separate repo):

- [How to represent a Rust ABI crate](../../ggen-marketplace/docs/how-to/represent-a-rust-abi-crate.md)
- [Ontology maturity L4-L5](../../ggen-marketplace/docs/reference/ONTOLOGY-MATURITY-L4-L5.md) (the ggen bridge doctrine)
- [rust-wasi-wasmex-pack](../../ggen-marketplace/packs/rust-wasi-wasmex-pack/README.md) (canonical consumer pack rendering this FFI)
