# Migrating ash_r2rml to the Hex graphlaw_ex

Swap the `path:` dependency for the published package. No Ash or Splode dependency is
added; errors are plain structs you map into your own error classes.

## 1. Dependency

```elixir
# mix.exs
{:graphlaw_ex, "~> 26.11.25"},   # was: {:graphlaw_ex, path: "../graphlaw_ex"}
{:wasmex, "~> 0.18.1"},
{:jason, "~> 1.4"}
```

The package does NOT ship `graphlaw.wasm` (excluded, 8 MB Hex limit). It ships
`priv/graphlaw/MANIFEST.json`, `graphlaw.wasm.blake3` and `graphlaw.wasm.sha256`. Obtain
the engine with `mix graphlaw_ex.fetch` (GitHub Release v26.11.25 on
seanchatmangpt/graphlaw; pins come from `MANIFEST.json`), or set `GRAPHLAW_WASM_PATH`
for air-gapped hosts.

ash_r2rml must resolve the engine through `GraphlawEx.WasmConfig.wasm_path/0` rather than
a hard-coded `priv` path, and CI must run `mix graphlaw_ex.fetch` before tests.

## 2. Boot and config

```elixir
# config/runtime.exs (or a per-env file)
config :graphlaw_ex, auto_start: true
# optional: config :graphlaw_ex, wasm_path: "/abs/path/graphlaw.wasm"
```

`auto_start: true` starts `GraphlawEx.Pool` (default size: schedulers online) under the
name `GraphlawEx`. Do not combine `auto_start: true` with a self-supervised
`GraphlawEx.Pool` of your own: both claim the name and the second start fails.
Boot reads the engine, checks the sha256 pin and the WASI import
allowlist, then compiles the module once. It fails closed with a typed error, for
example `{:wasm_digest_mismatch, ...}`. Without `auto_start`, call
`GraphlawEx.start_link(wasm_path: ...)` yourself.

Lease policy defaults to `:required`: `law` calls need a signed lease unless the environment
sets `config :graphlaw_ex, lease_policy: :none_allowed` (test only). Durability, telemetry and
pool sizing are in the Operations guide.

BLAKE3 is not checked at boot. Run `mix graphlaw_ex.verify` in CI or before publishing.

## 3. Usage

```elixir
alias GraphlawEx.Capability.{Canonical, Graph, Sparql}

# SPARQL over N-Quads/Turtle text (dialect sniffed unless :dialect is given)
{:ok, %GraphlawEx.Result.Sparql{variables: vars, rows: rows}} =
  Sparql.query(nquads, "SELECT ?s ?o WHERE { ?s ?p ?o }", max_rows: 10_000)

# CONSTRUCT results arrive as result.nquads / result.quads; ASK as result.value.

# Canonical form and content id
{:ok, %GraphlawEx.Result.Canonical{id: "blake3:" <> _, nquads: _}} =
  Canonical.run(data: %{"text" => nquads})

# Graph helpers (text in, text out)
{:ok, digest} = Graph.digest(nquads)
{:ok, same?}  = Graph.equivalent?(a, b)
{:ok, canon}  = Graph.canonicalize(nquads)
```

`Graph.union/3`, `diff/3` and `intersection/3` also exist. Blank nodes are labeled per
input, so `diff` and `intersection` match blank-node quads only when labels coincide.

Every call takes `opts`: `:server` (pool name), `:timeout_ms`, and fuel options of
`GraphlawEx.Capability.call/2`.

## 4. Errors

Every verb returns `{:ok, result}` or `{:error, %GraphlawEx.Capability.Refusal{}}`
(`code`, `message`, `details`, `raw`, `broken_term`). The `!` twins raise
`GraphlawEx.Error` (`code/1`, `raw/1`, `broken_term/1`).

Mapping onto Ash/Splode classes is `GraphlawEx.Error.Class.of/1` (the one authority;
`GraphlawEx.Refusal.class_of/1` is the separate Chatman refusal class, not a Splode class):

| code | Splode class |
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

Engine refusal kinds, via `GraphlawEx.Error.Class.of_kind/1`:

| kind | Splode class |
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

The kind table equals the registry's `refusal_kind_classes` (tested). Kind and code classes
differ only for `ResourceLimit` (`:forbidden`) vs `:REFUSED_RESOURCE_BOUND` (`:invalid`) and
`Unsupported` (`:unknown`) vs `:REFUSED_UNSUPPORTED_SPARQL_FEATURE` (`:invalid`).

Special codes: `:REFUSED_UNSUPPORTED_SPARQL_FEATURE` (from `Sparql.query/3`, a
re-coded `:engine_refused`/`:not_admitted`) and `:ceiling_unmet` (authority pre-gate).
Pass a refusal code through `GraphlawEx.Error.wrap/1` before `Error.Class.of/1`; both are total.
The full register is `GraphlawEx.Refusal.codes/0` and `docs/refusals.md`.

```elixir
case Sparql.query(nq, q) do
  {:ok, r} -> r.rows
  {:error, %{code: :resource_limit, details: d}} -> {:too_big, d}
  {:error, refusal} -> {:error, refusal}
end
```

## 5. Limits

- `:max_rows` on `Sparql.query/3`: default 10_000; above it, `:resource_limit`.
- Request size cap: 16 MiB (`GraphlawEx.Capability.Limits.max_request_bytes/0`).
- JSON depth 64, outstanding alloc 16 MiB; read `Limits.all/0` for the full list.
- Calls are fuel- and timeout-bounded; a trap or timeout recycles the instance, and a
  full queue sheds with `:saturated`.

## See Also

- `README.md`, `CHANGELOG.md`, `docs/refusals.md`
- Operations guide
- Architecture guide
