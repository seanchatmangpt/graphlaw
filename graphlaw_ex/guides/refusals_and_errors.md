# Refusals and Errors

A refusal is an observation about one exact input: the engine or the host declined to act, and
says why. It is not an exception and it grants or withdraws no authority. Three types carry that
information, at three layers.

## The three types

- `GraphlawEx.Refusal`: host and engine transport, the one closed code table. Fields: `code`,
  `class`, `message`, `details`, `broken_term`, `raw`.
- `GraphlawEx.Capability.Refusal`: capability layer re-export of that table. Fields: `code`,
  `message`, `details`, `raw`, `broken_term` (no `class`).
- `GraphlawEx.Error`: public exception with ten stable codes. Fields: `code`, `class`,
  `detail`, `evidence`.

`GraphlawEx.Refusal` is what host calls return. Its codes cover pool and WASM failures
(`:saturated`, `:call_timeout`, `:call_trapped`, `:fuel_exhausted`), identity failures
(`:wasm_digest_mismatch`, `:wasm_import_surface_mismatch`, `:lease_revoked`), structure failures
(`:invalid_json`, `:invalid_capability_request`, `:not_admitted`) and `:engine_unclassified` for
engine variants this library does not know. A future engine release maps there rather than
crashing.

`GraphlawEx.Capability.Refusal` is what the typed capability functions return. Its client-side
codes are built before the engine is called (`:invalid_capability_request`,
`:unknown_capability`, `:ceiling_unmet`); engine-side codes are lifted from the wire with the
payload kept losslessly in `raw`. Both structs hide `raw` and `details` from `inspect/2`, since
they can carry request content.

`GraphlawEx.Error` is the exception raised by the `!` functions. Normalize any of the above with
`GraphlawEx.Error.wrap/1`, which is total and never raises:

```elixir
case GraphlawEx.Capability.API.shacl(data: data, shapes: shapes) do
  {:ok, result} -> result
  {:error, refusal} -> handle(GraphlawEx.Error.wrap(refusal))
end
```

The fine-grained original code stays in `error.evidence["refusal_code"]`.

## Classes

`GraphlawEx.Refusal.class_of/1` returns the refusal class of a code and raises `ArgumentError`
(with a nearest-code hint) for a code outside the table:

```elixir
GraphlawEx.Refusal.class_of(:saturated)
#=> :blocked_resource
```

| class | meaning |
|---|---|
| `:blocked_resource` | a limit stopped the call: queue, timeout, fuel, size |
| `:refused_structure` | the input or response was malformed or not admitted |
| `:refused_identity` | an artifact, lease or receipt identity did not hold |
| `:unsupported` | the engine refused with a variant not classified here |

Each code also maps to a Chatman `broken_term` via `GraphlawEx.Refusal.broken_term_of/1` and has
a one-line meaning from `doc_of/1`.

## Error.Class categories

`GraphlawEx.Error.Class.of/1` (alias `category/1`) maps an error code, a `GraphlawEx.Error`, or
any term `wrap/1` accepts onto one of three framework categories. It is unrelated to the
`class` field of the struct.

| code | category |
|---|---|
| `:REFUSED_ARTIFACT_INTEGRITY` | `:invalid` |
| `:REFUSED_INVALID_INPUT` | `:invalid` |
| `:REFUSED_AMBIGUOUS_DIALECT` | `:invalid` |
| `:REFUSED_UNSUPPORTED_SPARQL_FEATURE` | `:invalid` |
| `:REFUSED_RESOURCE_BOUND` | `:invalid` |
| `:REFUSED_RAW_SQL_FORBIDDEN` | `:forbidden` |
| `:REFUSED_LEASE_REQUIRED` | `:forbidden` |
| `:REFUSED_MEMORY_LIMIT_EXCEEDED` | `:unknown` |
| `:REFUSED_QUERY_TIMEOUT` | `:unknown` |
| `:REFUSED_WASM_TRAP` | `:unknown` |

`:forbidden` needs a lease or receipt; `:invalid` means fix the input or bound; `:unknown` is
not attributable to the input's shape.

Decision: operational and unclassified failures are `:unknown`, caller-fixable ones are
`:invalid`. The table above is per public code; `Class.of/1` refines it by the original refusal
code kept in `evidence["refusal_code"]`. `:saturated` (shed load), `:duckdb_failure` and
`:engine_unclassified` classify `:unknown`, as does a non-refusal term wrapped by `wrap/1` (the
"unclassified failure" error). Their public codes do not change. Size caps (`:resource_limit`)
and malformed input stay `:invalid`. `GraphlawEx.Error.combine/1` aggregates child errors
(optionally `{path, term}`) with precedence `:forbidden > :invalid > :unknown`, keeping every
child and its path under `evidence["errors"]`.

The struct's `class` field is the Chatman taxonomy class, not the category above; read it with
`GraphlawEx.Error.taxonomy_class/1` and the category with `Class.category/1`. Raising
`GraphlawEx.Error` with a message or keyword (`raise GraphlawEx.Error, "msg"`) normalizes any
`:code` through `code_for/1`. Engine refusal kinds use
`GraphlawEx.Error.Class.of_kind/1`:

| kind | category |
|---|---|
| `NotSemanticContent` | `:invalid` |
| `Ambiguous` | `:invalid` |
| `EngineRejected` | `:invalid` |
| `DuckDb` | `:invalid` |
| `BareSql` | `:invalid` |
| `ResourceLimit` | `:forbidden` |
| `TierMismatch` | `:forbidden` |
| `Fuse` | `:forbidden` |
| `Unsupported` | `:unknown` |

The kind table equals `refusal_kind_classes` in `registry/capability-registry.json` (tested).
A kind class and a code class differ only where listed here: kind `ResourceLimit` is
`:forbidden` but code `:REFUSED_RESOURCE_BOUND` is `:invalid`; kind `Unsupported` is `:unknown`
but code `:REFUSED_UNSUPPORTED_SPARQL_FEATURE` is `:invalid`. The kind classifies the engine
refusal family; the code classifies the wire error after re-coding.

`:saturated` maps to `:REFUSED_RESOURCE_BOUND`; lease and authority refusals (`:lease_revoked`,
`:lease_refused`, `:unverified_lease_refused`, `:receipt_required`, `:ceiling_unmet`) map to
`:REFUSED_LEASE_REQUIRED`.

## Retryability

The library publishes no `retryable?` flag, so derive a policy from the code and category:

- `:forbidden` is not retryable without new authority. Acquire a lease or receipt first.
- `:invalid` is not retryable unchanged. Retrying the same input yields the same refusal; repair
  it, or raise `max_rows`, `max_bytes` or `timeout_ms` if the bound was the cause.
- `:unknown` is the only category worth an unchanged retry, with backoff and a cap. A timeout,
  trap, exit or fuel exhaustion recycles that instance, so the next call gets a fresh one.
- `:saturated` is shed load, not a defect. Back off, or raise `max_queue` after sizing.

The pool already retries once when a member exits mid-call (`:call_exited`). Engine load failures
are retried by the supervised host with capped exponential backoff, so do not wrap start-up in a
retry loop of your own.

## Mapping to Ash and Splode

`graphlaw_ex` has no Ash or Splode dependency. It names the category and leaves the module
choice to the consumer:

```elixir
def to_ash(term) do
  error = GraphlawEx.Error.wrap(term)

  case GraphlawEx.Error.Class.of(error) do
    :forbidden -> MyApp.Error.Forbidden.exception(error: error)
    :invalid -> MyApp.Error.Invalid.exception(error: error)
    :unknown -> MyApp.Error.Unknown.exception(error: error)
  end
end
```

Keep the `GraphlawEx.Error` struct as the wrapped payload so policies and logs still see `code`,
`detail` and the string-keyed `evidence`.

## Lease policy

`config :graphlaw_ex, lease_policy: :required` (the default) makes production fail closed:
a `law` request with no signed lease is refused before the engine as `:ceiling_unmet`
(public code `:REFUSED_LEASE_REQUIRED`, `evidence["details"]["required_by"] == "lease_policy"`),
and every `law` request sent through `GraphlawEx.Capability.Dispatch` carries
`"lease_policy" => "required"` so the engine refuses it too. `:none_allowed` is for test
environments only; per-call `lease_policy:` opts can tighten the policy, never loosen it.

## Engine-side refusals added in v26.11.25

These come from the Rust engine and surface through `raw` with the engine message tag. Their
triggers and recovery are in `docs/refusals.md` (lines 48-50):

| tag | trigger | recovery |
|---|---|---|
| `RECEIPT-IRI-INVALID` | child IRI had a character forbidden in N-Triples | fix the IRI |
| `LeaseRequired` | policy `Required`, `law` request had no lease | present a lease |
| `hook-refused` | a `kh:Effect "refuse"` hook fired in `materialize` | nothing committed |

`LeaseRequired` reaches Elixir callers as the `:REFUSED_LEASE_REQUIRED` family described above.

## Refusal register

The typed refusal register, with stable codes, triggers and recovery steps, is in the GraphLaw
repository: <https://github.com/seanchatmangpt/graphlaw/blob/main/docs/refusals.md>. Codes there
are append-only and never renumbered.

## See Also

- Capabilities guide
- Getting Started guide
- Operations guide
