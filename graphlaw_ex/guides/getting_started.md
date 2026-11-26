# Getting Started

`graphlaw_ex` runs the GraphLaw law-admission engine as a pinned WASM module inside your BEAM
node. This guide takes you from an empty dependency list to one verified SPARQL query.

## Add the dependency

```elixir
# mix.exs
defp deps do
  [
    # use the current release from Hex
    {:graphlaw_ex, "~> <current release>"}
  ]
end
```

The WASM engine is not shipped in the Hex package (it exceeds Hex's size limit). Stage it once:

```bash
mix deps.get
mix graphlaw_ex.fetch
```

`graphlaw_ex.fetch` downloads the GitHub Release asset for your package version, verifies both
the SHA-256 and BLAKE3 pins from the packaged manifest, and places it in the user cache. It fails
closed and removes partial files. It is the only code in the library that touches the network.
See the Releases guide for images and air-gapped hosts.

Air-gapped host: vendor the file yourself and point at it. The path is still checked
against the manifest SHA-256 pin when it comes from the environment or app config.

```bash
export GRAPHLAW_WASM_PATH=/abs/path/graphlaw.wasm
```

## Start the pool with your application

The host pool does not start by default. Turn it on in `config/runtime.exs`:

```elixir
# config/runtime.exs
import Config

config :graphlaw_ex, auto_start: true
```

Optional keys (all validated by `GraphlawEx.Config.validate/0`):

```elixir
config :graphlaw_ex,
  auto_start: true,
  timeout_ms: 5_000,
  max_queue: 64
```

An unknown key is refused with a "did you mean" hint, so a typo does not silently fall back to a
default.

## Choose a lease policy

`config :graphlaw_ex, :lease_policy` defaults to `:required` (fail closed). Under `:required`,
a `law` request that presents no signed lease is refused before it reaches the engine
(`GraphlawEx.Authority.check_lease_policy/3`; public code `:REFUSED_LEASE_REQUIRED`). Other
operations, including `sparql` below, are not lease-gated. Per-call options can tighten the
policy with `lease_policy: :required` but never loosen it.

For local experiments only, opt out explicitly:

```elixir
# config/dev.exs  (never in a production config)
config :graphlaw_ex, lease_policy: :none_allowed
```

For anything else, present a signed lease and configure the keys that verify it:

```elixir
# config/runtime.exs
config :graphlaw_ex, trusted_keys: ["<hex Ed25519 public key of the lease signer>"]
```

```elixir
# signed_lease is the {lease, attestation} map issued by your authority
GraphlawEx.law(
  [data: %{"text" => text, "dialect" => "turtle"}, steps: [%{"step" => "rdfs"}]],
  signed_lease: signed_lease
)
```

A signed lease with no `:trusted_keys` configured is refused (`:trusted_keys_absent`);
signature, signer and expiry are verified by the engine. The step ceiling still applies
(`GraphlawEx.Authority.check_law_steps/2`).

## One query

`GraphlawEx.Capability.Sparql.query/3` takes raw RDF text and a SPARQL string. The dialect is
sniffed from the text.

```elixir
nquads = ~s(<urn:s> <urn:p> "ok" .\n)

{:ok, result} =
  GraphlawEx.Capability.Sparql.query(nquads, "SELECT ?o WHERE { <urn:s> <urn:p> ?o }")

result
```

Every call returns `{:ok, result}` or `{:error, refusal}`. Nothing in the library raises on bad
input; see the Refusals and Errors guide for what a refusal carries. The bang variants in
`GraphlawEx.Capability.API` (for example `sparql!/2`) raise `GraphlawEx.Error` instead.

Options for `query/3` include `:max_rows` (default 10,000), `:max_bytes` and `:timeout_ms`.
Exceeding a bound refuses with `:REFUSED_RESOURCE_BOUND` rather than truncating.

## Verify the install

```bash
mix graphlaw_ex.doctor
```

The doctor prints one `[ok]`, `[warn]` or `[FAIL]` line per check: library version, config
validity, resolved artifact path, SHA-256 and BLAKE3 pins, engine boot (pin, import surface,
required exports), a canonical round trip, and pool state. Only `[FAIL]` produces a non-zero
exit, which makes it suitable as a CI step. With `auto_start` off, the round trip runs through
a short-lived host and the pool line is a warning.

## See Also

- Capabilities guide: the 14 operations
- Refusals and Errors guide
- Releases guide: baking the artifact into images
