# Capability registry

The capability registry is the machine-readable statement of GraphLaw's public JSON ABI: which ops
exist, which fields each op takes and returns, which refusals it can emit, and which closed
vocabularies (dialects, regimes, lease ceilings, limits) apply. Hosts such as `ash_graphlaw` read
the registry instead of copying the op list by hand, so a change to the ABI shows up as a digest
change rather than as silent drift.

Schema id: `graphlaw.capability-registry/1`. The schema is frozen for v26.9.29; changes are
additive only. `ABI_VERSION` stays `1`.

Status of this page: written against the v26.9.29 registry contract. The emitter
(`src/registry.rs`, `src/bin/graphlaw_registry.rs`) and the `registry/` directory were not present
in the checkout when this page was written, so every statement about emitted files, flags and
digests is contract text, not observed output. Where a claim depends on a command that has not
been run, it is marked UNKNOWN.

## Files under `registry/`

| file | what it is |
|---|---|
| `registry/capability-registry.json` | The registry. Pretty JSON, keys sorted byte-wise at every level, 2-space indent, LF, trailing newline, ASCII-only strings, integers only. |
| `registry/capability-registry.ttl` | Deterministic Turtle projection of the same registry. |
| `registry/capability-registry.schema.json` | JSON Schema (draft 2020-12) for the JSON file. |
| `registry/capability-registry.shapes.ttl` | SHACL shapes for the Turtle file, checked with GraphLaw's own `shacl` op in tests. |
| `registry/op-examples.json` | Runnable request examples per op (schema `graphlaw.op-examples/1`, see below). |
| `registry/op-examples.schema.json` | JSON Schema for `op-examples.json`. |

The source of truth is the static table in `src/registry.rs` (feature `abi`). The JSON and Turtle
files are emitted from it and are never edited by hand.

## Regenerating and checking

The emitter is the `graphlaw-registry` binary (`src/bin/graphlaw_registry.rs`,
`required-features = ["abi"]`):

```sh
cargo run --features abi --bin graphlaw-registry -- --write         # rewrite registry/*.json and .ttl
cargo run --features abi --bin graphlaw-registry -- --check         # exit 1 + diff summary on drift
cargo run --features abi --bin graphlaw-registry -- --print-json
cargo run --features abi --bin graphlaw-registry -- --print-ttl
cargo run --features abi --bin graphlaw-registry -- --print-digests
```

`--write` emits `capability-registry.json` and `capability-registry.ttl`. `--check` compares the
committed files with a fresh emission. Whether these commands succeed on the current tree is
UNKNOWN until they are run; the flags above are the contract.

Release assets: `capability-registry.json`, `capability-registry.ttl` and `op-examples.json`, each
with a `.sha256` file.

## Top-level JSON shape

```json
{
  "schema": "graphlaw.capability-registry/1",
  "graphlaw_version": "26.9.29",
  "abi_version": 1,
  "surface_sha256": "sha256:<64 lowercase hex>",
  "registry_sha256": "sha256:<64 lowercase hex>",
  "authorities": [{"capability": "...", "authority": "..."}],
  "engines": ["PurRdf", "Eyeron"],
  "refusal_kinds": ["NotSemanticContent", "Ambiguous", "EngineRejected", "Unsupported", "ResourceLimit"],
  "rdf_dialects": [], "other_dialects": [],
  "regimes": ["simple", "rdf", "rdfs", "owl-rl", "d"],
  "lease_ceilings": ["observe", "select", "construct"],
  "lease_reasons": ["expired", "out_of_scope", "ceiling", "bad_signature", "untrusted_key", "clock_skew"],
  "receipt_reasons": ["unattested", "bad_signature", "untrusted_key"],
  "policy_refusal_kinds": ["MissingEntry", "InventedOutcome", "BadMass", "DeadEnd", "Malformed"],
  "limits": {"max_request_bytes": 16777216, "max_json_depth": 64, "max_plan_actions": 1000,
             "max_atoms_per_field": 10000, "max_policy_entries": 100000, "n3_max_iterations": 4000},
  "refusal_codes": [{"code": "...", "order": 1, "kind": null, "fields": []}],
  "law_steps": [{"name": "...", "order": 1, "ceiling": "observe", "fields": []}],
  "ops": []
}
```

- `authorities` is built from `BACKEND_AUTHORITIES`; the upstream `revision` is intentionally
  omitted.
- `rdf_dialects` and `other_dialects` hold `Dialect` objects: `name`, global `order` (RDF dialects
  first), `aliases`, `response_name`, `media_type` (null for non-RDF dialects) and `kind`
  (`rdf` or `other`).
- `refusal_codes` lists the `details.code` values: `Refused`, `ResourceLimit`, `PlanRefused`,
  `ReceiptRequired`, `ReceiptRefused`, `LeaseRefused`, `NotAdmitted`, `UnverifiedLeaseRefused`,
  `PolicyRefused`, each with its `details` fields.
- `law_steps` uses wire names (`owl-rl`, `record-receipts`, ...) with the lease ceiling a step
  needs, or null when it needs none.
- Each `ops` entry has `name`, `order` (1..14), `summary`, `request.fields`, `responses`
  (variants), `refusal_kinds` and `refusal_codes`. An untagged op has exactly one variant with
  `tag: null`; `sparql` has three variants tagged by the response field `kind`.
- A field is `{name, order, type, required, nullable, doc, enum, default}`. `enum` is
  informational. `ok` is the response envelope and is not a registry field.

Closed type vocabulary: `string`, `integer`, `boolean`, `object`, `any`, `data_spec`, `term`,
`json_or_string`, `list<string>`, `list<object>`, `list<any>`, `list<term>`,
`list<list<string>>`, `list<list<term>>`. `data_spec` is the object `{text, dialect?, hint?,
base?}`; `term` is the SPARQL-JSON-like term object (see [ABI reference](abi-reference.md)).

## Digest rule

Digests are over canonical bytes, never over the pretty file bytes. Rust and Elixir implement the
same rule.

`canonical(v)`:

- compact JSON with no whitespace;
- object keys sorted by byte order; arrays keep their order;
- strings escaped per RFC 8259 minimally: escape `"` and `\`, and control characters below 0x20 as
  `\b \t \n \f \r`, otherwise lowercase `\u00xx`; no other escapes; non-ASCII stays raw UTF-8;
- integers in decimal; floats are not allowed.

A `serde_json::Value` with the default `BTreeMap` map, serialized with `to_string`, satisfies
this.

Two digests:

- `surface_sha256` = `"sha256:" + hex(SHA-256(canonical(S)))` where
  `S = {"abi_version": 1, "ops": [14 op names in order], "other_dialects": [names in order],
  "rdf_dialects": [names in order]}`.
- `registry_sha256` = `"sha256:" + hex(SHA-256(canonical(R)))` where `R` is the registry document
  with the key `registry_sha256` removed. `surface_sha256` is part of `R`.

Examples in `op-examples.json` are not part of the registry digest.

## `capabilities` op exposure

The `capabilities` response gains three top-level string fields, additively; `abi_version` stays
`1`:

| field | value |
|---|---|
| `registry_schema` | `graphlaw.capability-registry/1` |
| `registry_sha256` | the registry digest |
| `surface_sha256` | the surface digest |

Existing fields (`abi`, `abi_version`, `crate`, `authorities`, `rdf_dialects`, `other_dialects`,
`ops`) keep their names and values. `ops`, `rdf_dialects` and `other_dialects` are built from
`src/registry.rs`, not from a second literal.

Engines older than v26.9.29 (for example the v26.9.28 wasm pinned by `ash_graphlaw`) return none
of the three fields. A consumer then computes `surface_sha256` host-side from the live response
using `S` above and treats `registry_sha256` as UNKNOWN.

## Dispatch court

`src/abi.rs` keeps one `match` arm per op in `dispatch`, each on its own line of the form
`"<op>" => ...`, and a fallthrough `bad("unknown op ...")`. The registry-vs-dispatch test parses
those arms from source and compares them with the registry op list. Do not reformat `dispatch`
into multi-op arms.

## How consumers vendor and pin the registry

`ash_graphlaw` copies the three release files into `priv/graphlaw/` (`scripts/vendor_registry.sh`),
recomputes the digest over canonical bytes and compares it with the literal it carries, then
imports the Turtle into its ontology between generated-registry marker lines. The pin is the
`registry_sha256` value; a registry change moves the pin. A consumer running against an engine
whose live `surface_sha256` differs from the pinned registry must report the drift, not mask it.

## Forward-compatibility rules for decoders

Decoders of engine responses follow these rules so an additive engine release never breaks them:

1. Unknown response keys are kept, not rejected. Keep the whole decoded response as a raw map
   alongside any typed view, so decoding is lossless.
2. Enum-like values (`dialect`, `engine`, `severity`, `status`, `kind`, ...) are open strings.
   The `enum` list in the registry is informational and is never enforced client-side.
3. A tagged response (`sparql`) with an unrecognized tag decodes to an unknown-tag value carrying
   the tag string, not to an error.
4. `engine` and `dialect` in an error envelope are nullable; `details` is optional.
5. A field whose type does not match the registry is returned untouched, never raised on.
6. Missing optional fields decode to absent (`nil`/null); the registry marks them `required:
   false`.
7. Refusal `details.code` values not in `refusal_codes` are surfaced with the raw error map.

## `op-examples.json` contract

Schema `graphlaw.op-examples/1`:

```json
{
  "schema": "graphlaw.op-examples/1",
  "abi_version": 1,
  "examples": [
    {"id": "<op>.<slug>", "op": "sniff", "description": "...",
     "request": {"op": "sniff", "text": "..."},
     "outcome": "ok", "refusal_kind": null, "refusal_code": null,
     "positive_control": true}
  ]
}
```

- Every op has at least one `ok` example, listed before that op's `refused` examples.
- Every op except `capabilities` has at least one `refused` example.
- Requests are self-contained inline text: no files and no signatures. `law` examples that use a
  lease set `unverified_lease: true` and `now_unix`.
- `outcome: "refused"` examples name the expected `refusal_kind` and, when the refusal carries
  one, `refusal_code`.
- `positive_control: true` marks the example that proves the op works before negatives are read.

## See Also

- [ABI reference](abi-reference.md)
- [Refusals](refusals.md)
- [API stability](api-stability.md)
