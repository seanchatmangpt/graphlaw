# Capabilities

The engine exposes 14 operations, generated from the capability registry
(`GraphlawEx.Capability.Registry`). Each has a typed function in `GraphlawEx.Capability.API`
taking `(args, opts \\ [])`, where `args` is a keyword list or map with snake_case keys.

- `op(args, opts)` returns `{:ok, result}` or `{:error, %GraphlawEx.Refusal{}}`-shaped refusal.
- `op!(args, opts)` returns the result or raises `GraphlawEx.Error`.
- `GraphlawEx.Capability.API.run/3` dispatches by wire name and refuses unknown names with
  `:unknown_capability`.

Results are observations. A successful result is not standing and grants no authority.

## Tiers

Every operation's pre-gate floor is `observe`. Tiers matter inside `law` steps, where each step
has a ceiling checked against the lease you present:

| ceiling | law steps |
|---|---|
| `observe` | `shacl`, `sql`, `require-receipt`, `require-signed-receipt` |
| `select` | `plan` |
| `construct` | `n3`, `rdfs`, `owl-rl`, `hooks` |

Ordering is `observe` < `select` < `construct`. A lease below a step's ceiling refuses the call
(`:ceiling_unmet`) before the engine runs.

Dialects have their own tiers (`GraphlawEx.Capability.Dialect.Tiers`): `:dataset` (Turtle, TriG,
NTriples, NQuads, RdfXml, JsonLd, YamlLd, TriX, HexTuples), `:schema` (ShExC, ShExJ), `:logic`
(N3), `:query` (Sparql) and `:sql` (SqlRdf).

## The operations

Only `sparql` and the `Graph` helpers have positional entry points. Everything else uses the
keyword form. `data` is a string of RDF text or a map `%{"text" => ..., "dialect" => ...}`.

### capabilities

Reports the ABI version, dialects, ops and registry digests of the live engine.

```elixir
{:ok, caps} = GraphlawEx.Capability.API.capabilities([])
```

### sniff

Routes text to a dialect and owning engine without parsing it.

```elixir
{:ok, r} = GraphlawEx.Capability.API.sniff(text: "<urn:s> <urn:p> <urn:o> .")
```

### parse

Parses with the owning engine; returns a quad count and content id for RDF, or validity for
non-RDF dialects.

```elixir
{:ok, r} = GraphlawEx.Capability.API.parse(text: "<urn:s> <urn:p> <urn:o> .", dialect: "ntriples")
```

### convert

Serializes RDF text into another RDF dialect.

```elixir
{:ok, r} = GraphlawEx.Capability.API.convert(text: "<urn:s> <urn:p> <urn:o> .", to: "turtle")
```

### canonical

Canonicalizes to N-Quads (RDFC-1.0) and returns the content-addressed id. Positional helpers
live in `GraphlawEx.Capability.Graph`.

```elixir
{:ok, nquads} = GraphlawEx.Capability.Graph.canonicalize("<urn:s> <urn:p> <urn:o> .")
{:ok, id} = GraphlawEx.Capability.Graph.digest("<urn:s> <urn:p> <urn:o> .")
```

`Graph` also offers `equivalent?/3`, `union/3`, `diff/3` and `intersection/3`. `diff` and
`intersection` refuse input containing blank nodes, because canonical labels are per input.

### sparql

Evaluates a query over a dataset. Responses are tagged `solutions`, `graph` or `boolean`.

```elixir
{:ok, r} = GraphlawEx.Capability.Sparql.query("<urn:s> <urn:p> <urn:o> .", "ASK { ?s ?p ?o }")
```

### shacl

Validates data against a SHACL shapes graph given as Turtle.

```elixir
{:ok, r} = GraphlawEx.Capability.API.shacl(data: data_ttl, shapes: shapes_ttl)
r.conforms
```

### shex

Validates against a ShEx schema and shape map. `schema_dialect` is `shexc` (default) or `shexj`.

```elixir
{:ok, r} = GraphlawEx.Capability.API.shex(data: data_ttl, schema: shexc, map: "<urn:s>@<urn:S>")
```

### n3

Runs bounded Notation3 forward reasoning over a document containing data and rules.

```elixir
{:ok, r} = GraphlawEx.Capability.API.n3(text: n3_document)
```

### entail

Materializes an entailment regime. Regimes: `simple`, `rdf`, `rdfs`, `owl-rl`, `d`.

```elixir
{:ok, r} = GraphlawEx.Capability.API.entail(data: data_ttl, regime: "rdfs")
```

### datalog

Evaluates rules over triple facts to a fixpoint. Terms starting with `?` are variables.
Evaluation is bounded: `datalog_derived_facts` 200000, `datalog_join_steps` 50000000 and
`datalog_output_bytes` 8 MiB (`src/abi/ops/query_engines.rs:232-250`; values emitted by
`src/registry/emit_json.rs:190-192`). Read the live values from
`GraphlawEx.Capability.Registry.limits/0`. Exceeding a bound refuses with a resource-limit
refusal.

```elixir
{:ok, r} =
  GraphlawEx.Capability.API.datalog(
    rules: [%{head: ["?x", "ancestor", "?y"], body: [["?x", "parent", "?y"]]}],
    facts: [["a", "parent", "b"]]
  )
```

### hooks

Materializes a knowledge-hook pack (the `kh:` vocabulary) over a dataset. A hook with
`kh:Effect "refuse"` halts the transaction with no state committed, refusing with a
`hook-refused: <hook iri>: <reason>` message. Refuse hooks always evaluate, so a caller-supplied
`kh:Firing` marker cannot suppress them (`src/hooks.rs:341`, `docs/refusals.md:50`).

```elixir
{:ok, r} = GraphlawEx.Capability.API.hooks(pack: %{"text" => pack_ttl}, data: data_ttl)
```

### law

Runs an ordered chain of law steps with receipts and optional lease authorization. Signed leases
are verified offline against `config :graphlaw_ex, :trusted_keys`; with no trusted keys the
call fails closed.

```elixir
{:ok, r} =
  GraphlawEx.Capability.API.law(
    data: data_ttl,
    steps: [%{"step" => "shacl", "shapes" => shapes_ttl}, %{"step" => "record-receipts"}]
  )
```

The step list is in `GraphlawEx.Capability.Registry.law_steps/0`.

A `law` request without a lease is refused under the default `lease_policy: :required` (see the
Refusals and Errors guide). The Rust host sets the same policy with `abi::set_lease_policy`,
or per request with `"lease_policy": "required"` (`docs/refusals.md:49`).

### policy

Admits a FOND policy as strong-cyclic against a planning problem.

```elixir
{:ok, r} = GraphlawEx.Capability.API.policy(problem: problem_json, policy: policy_json)
```

## Hidden native-only op

The registry's hashed table has 14 operations. A fifteenth, `duckdb`, is a diagnostic op that
dispatches inside the graphlaw-wasm unit but stays off the hashed table so the registry digest
is stable (`graphlaw_ex/lib/graphlaw_ex/capability/registry.ex:1781`). It has no typed function in
`GraphlawEx.Capability.API`. The experimental `abi_bin` v2 framing is not wired into this
library.

## Introspection

`GraphlawEx.Capability.Registry` exposes `names/0`, `ops/0`, `op/1`, `dialects/0`, `limits/0`
and `law_steps/0` as compile-time data, so you can inspect request and response fields without
calling the engine.

## See Also

- Getting Started guide
- Refusals and Errors guide
- Operations guide
- Architecture guide
