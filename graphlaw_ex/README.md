# graphlaw_ex

Version `26.11.25` (`mix.exs`; `wasmex ~> 0.18.1`, `jason`, `telemetry`), engine
graphlaw `26.11.25`, ABI 1 (`priv/graphlaw/MANIFEST.json`). Migrating ash_r2rml from a
`path:` dep: `guides/ash_r2rml_migration.md`.

BEAM bridge for the graphlaw WASM law kernel — the deterministic turnstile
boundary for Smalt. The BEAM side holds only the wasm byte-slice; the bridge
owns bytes in and out over ABI version 1; the host owns authority, standing
and refusals. Nothing here interprets RDF, SHACL, SQL, or receipts — all law
admission happens inside the module.

## Pitch

One executable law, many runtimes. graphlaw compiles RDF/SHACL law into a
wasm artifact; `graphlaw_ex` is how a BEAM system consumes it without
re-implementing a single rule on the host. Admission is a function call:
bytes in, receipt JSON out, or a typed refusal raised *inside the module*.
The host never interprets law, so the host cannot drift from it.

## Architecture

```
BEAM caller
    |
    v
GraphlawEx (GenServer) ---> packet-4 frame ---> wasm module (digest-pinned artifact)
    |                                              |
    v                                              v
{:ok, response_map}  <--- response JSON      {:error, %GraphlawEx.Refusal{}}
  (read response["ok"];                      (host-side typed refusal: framing,
   an engine refusal is ok: false)            digest, limits, kill-switch, ...)
```

The BEAM holds only the wasm byte-slice; the bridge owns bytes in/out; the
host owns authority, standing, refusals.

- `lib/graphlaw_ex.ex` — the facade: `start_link/1`, `admit/3`, `stop/1`, plus
  the 14 typed capability verbs delegated to `GraphlawEx.Capability.API`
  (`capabilities`, `sniff`, `parse`, `convert`, `canonical`, `sparql`, `shacl`,
  `shex`, `n3`, `entail`, `datalog`, `hooks`, `law`, `policy`, each with a `!`
  twin).
- `lib/graphlaw_ex/host.ex` — `GraphlawEx.Host`, one serialized wasm instance
  (`gl_alloc` / `gl_call` / `gl_free`, packet-4 framing), recycle-on-fault,
  `:saturated` shedding; `pool.ex` runs N hosts behind one name.
- `lib/graphlaw_ex/engine_load.ex` + `wasm_config.ex` — load boundary: sha256
  pin from the manifest, WASI import allowlist, required exports, compile
  cached on `:persistent_term`.
- `lib/graphlaw_ex/abi.ex` (pure codec), `refusal.ex` (closed typed-refusal
  table), `telemetry.ex` (admit spans), `authority.ex` (GL28-7 presentation
  pre-gate), `kill_switch.ex` (GL28-6 signed revocation).
- `lib/graphlaw_ex/application.ex` — supervision tree, opt-in auto-start.

## Install

```elixir
# mix.exs
defp deps do
  [
    {:graphlaw_ex, "~> 26.11.25"},
    {:wasmex, "~> 0.18.1"},
    {:jason, "~> 1.4"}
  ]
end
```

The Hex package does NOT ship `graphlaw.wasm` (excluded: the engine exceeds the 8 MB
Hex limit). It ships the pins only: `priv/graphlaw/MANIFEST.json`
(`artifact.blake3` / `artifact.sha256`), `priv/graphlaw/graphlaw.wasm.blake3` and
`priv/graphlaw/graphlaw.wasm.sha256`. Obtain the engine one of two ways:

```bash
mix graphlaw_ex.fetch   # downloads from GitHub Release v26.11.25 on
                        # seanchatmangpt/graphlaw; pins come from priv/graphlaw/MANIFEST.json
export GRAPHLAW_WASM_PATH=/abs/path/graphlaw.wasm   # air-gapped: vendor the file yourself
```

`:wasm_path` (start option) also overrides the path.
`lib/graphlaw_ex/wasm_config.ex` is the truth for how the pin is located and checked.

## Configuration

Every accepted key, with minimums and defaults, is `GraphlawEx.Config.keys/0` (validate
with `GraphlawEx.Config.validate/1`). Pool size is not a config key: pass `size:` to
`GraphlawEx.Pool`. The path and pin options:

| Option | Where | Default | Meaning |
|---|---|---|---|
| `:wasm_path` | start option, then `GRAPHLAW_WASM_PATH` env, then `:graphlaw_ex, :wasm_path` app env (`GraphlawEx.WasmConfig.wasm_path/1`) | vendored `priv/graphlaw/graphlaw.wasm` if present, else `~/.cache/graphlaw_ex/<blake3>/graphlaw.wasm` (cache root also `XDG_CACHE_HOME` or `GRAPHLAW_CACHE_DIR`) | Filesystem path of the graphlaw wasm artifact. Resolution order: start option, `GRAPHLAW_WASM_PATH`, app env, vendored priv, user cache, else the vendored path (so the error names the canonical location). A missing file is a typed `{:error, {:wasm_load_failed, path, :enoent}}` from `start_link/1`; with `auto_start: true`, boot fails closed with `{:error, {:wasm_unreadable, %{path: path, reason: posix}}}`. Neither is a linked-process exit. |
| `:expected_sha256` | start option | manifest pin | Digest the artifact must match. `:unpinned` is the explicit opt-out; an explicit `:wasm_path` that differs from the vendored path is treated as `:unpinned`, while an env/app-config path stays pinned. |
| `:auto_start` | `:graphlaw_ex, :auto_start` app env | `false` | When `true`, the supervision tree starts `GraphlawEx` as a named child. Opt-in so depending on the library never instantiates wasm implicitly and never races a caller's own instance. |

Digest pin security property: the pin in `priv/graphlaw/MANIFEST.json` binds admission
to exact artifact bytes. `artifact.sha256` is enforced on load; `artifact.blake3` is the
published identity. Read both from the manifest; this README carries no digest literals.
Boot checks only the sha256 pin. The BLAKE3 check is a publish/CI gate, never run at
boot: `mix graphlaw_ex.verify` (exits non-zero with `REFUSED_ARTIFACT_INTEGRITY`). Manual
equivalent: run `b3sum` on the artifact and compare to `artifact.blake3` in `MANIFEST.json`.

Any other digest means the bytes are not the pinned artifact — do not admit
through them; results from unpinned bytes are not graphlaw results. The pin
is what makes a receipt replayable: evidence computed under artifact A does
not stand for artifact B.

## Releases

In a release or container, resolve the engine at runtime, never from `~/.cache`:

```elixir
# config/runtime.exs
if path = System.get_env("GRAPHLAW_WASM_PATH") do
  config :graphlaw_ex, wasm_path: path
end
```

Bake the wasm into the image (run `mix graphlaw_ex.fetch` at image build, copy the file
to a fixed path) and set `GRAPHLAW_WASM_PATH` to it. A path from the env or app config
stays pinned to the manifest sha256. Do not rely on the `~/.cache` fallback in
containers: the home directory is ephemeral or absent.

## Lease policy

`config :graphlaw_ex, :lease_policy` defaults to `:required`: a `law` request with no
signed lease is refused (`lib/graphlaw_ex/authority.ex`, `check_lease_policy/3`). Local
experiments set `config :graphlaw_ex, lease_policy: :none_allowed` explicitly; production
presents `signed_lease:` plus `config :graphlaw_ex, trusted_keys: [...]`. Walkthrough:
`guides/getting_started.md`.

## Zero-ceremony usage

Quickstart with the fetch flow and lease choice: `guides/getting_started.md`.

```bash
mix graphlaw_ex.fetch
```

```elixir
# config/runtime.exs (or a per-env file such as config/dev.exs; not config.exs,
# which would auto-start in every env including ones without the engine)
config :graphlaw_ex, auto_start: true
```

With `auto_start: true` the supervision tree starts the bridge under the name
`GraphlawEx` against the digest-pinned artifact resolved by
`GraphlawEx.WasmConfig.wasm_path/0`. No `start_link/1`, no digest handling.

```elixir
data = %{"text" => "@prefix : <https://ex.test/> .\n:a :p :b .\n", "dialect" => "turtle"}

{:ok, %GraphlawEx.Result.Sparql{kind: :solutions, variables: vars, rows: rows}} =
  GraphlawEx.sparql(data: data, query: "SELECT ?s ?o WHERE { ?s ?p ?o }")

# Bulk form over raw N-Quads text (opts: :max_rows, :timeout_ms, :dialect, :base):
{:ok, %GraphlawEx.Result.Sparql{rows: _}} =
  GraphlawEx.Capability.Sparql.query(data["text"], "SELECT ?s WHERE { ?s ?p ?o }",
    max_rows: 1_000)

# Raising twin: GraphlawEx.sparql!/2 raises GraphlawEx.Error on refusal.
{:error, %GraphlawEx.Capability.Refusal{code: :invalid_capability_request}} =
  GraphlawEx.sparql(data: data)
```

Every verb returns `{:ok, %GraphlawEx.Result.*{}}` or `{:error, refusal}`; the
`!` twin raises `GraphlawEx.Error` (`code/1`, `raw/1`, `broken_term/1`).

## Error codes

Refusal codes are a closed table of 27 (`GraphlawEx.Refusal.codes/0`; unknown engine
variants map to `:engine_unclassified`). Each carries a class and a
failure-taxonomy `broken_term`. The 10-code taxonomy is the smaller `GraphlawEx.Error`
set of top-level error kinds; the 27 refusal codes are the finer-grained table below,
and `GraphlawEx.Refusal.class_of/1` maps each code to one of four classes.

| Class | Codes |
|---|---|
| `:blocked_resource` | `saturated`, `call_timeout`, `call_trapped`, `call_exited`, `fuel_exhausted`, `resource_limit` |
| `:refused_structure` | `malformed_response`, `invalid_json`, `invalid_encoding`, `invalid_capability_request`, `unknown_capability`, `capability_response_undecodable`, `policy_refused`, `not_admitted`, `plan_refused` |
| `:refused_identity` | `wasm_digest_mismatch`, `wasm_import_surface_mismatch`, `receipt_required`, `lease_revoked` |
| `:unsupported` | `engine_unclassified`, `engine_refused` |

The table also holds `receipt_refused`, `lease_refused`, `unverified_lease_refused`,
`fuse`, `duckdb_failure` and `batch_member_failed`; read
`lib/graphlaw_ex/refusal.ex` for their class and meaning.

## Sandbox guarantees

- Law runs inside a wasm module; the BEAM holds only bytes. No host code
  interprets RDF, SHACL, SQL or receipts.
- Artifact identity: the sha256 pin from `priv/graphlaw/MANIFEST.json` is
  checked before bytes reach the compiler (`wasm_digest_mismatch`).
- Import surface: only the manifest's `wasi_snapshot_preview1` allowlist is
  accepted (`wasm_import_surface_mismatch`).
- Bounded calls: fuel budget, `timeout_ms`, queue cap (`saturated`) and request
  size cap (16 MiB, `GraphlawEx.Capability.Limits`). A trap, timeout or fuel
  exhaustion recycles the instance.
- Revocation: a signed kill-switch epoch sheds calls with `lease_revoked`.

## Explicit start (raw `admit/3`)

```elixir
# after `mix graphlaw_ex.fetch`, or any path you vendored (GRAPHLAW_WASM_PATH)
wasm_path = GraphlawEx.WasmConfig.wasm_path()
{:ok, _bridge} = GraphlawEx.start_link(wasm_path: wasm_path)

honest = %{
  "op" => "law",
  "data" => %{
    "text" => "@prefix : <https://test.graphlaw.dev/> .\n:a :edge :b .\n:b :edge :c .\n",
    "dialect" => "turtle"
  },
  "steps" => [%{"step" => "rdfs"}]
}

{:ok, %{"ok" => true, "nquads" => nquads, "receipts" => receipts}} =
  GraphlawEx.admit(Jason.encode!(honest))

# Tampered: an unknown op is refused inside the module. At the transport layer
# that is still {:ok, response}; classify it by response["ok"].
{:ok, %{"ok" => false, "error" => %{"kind" => "Unsupported"}}} =
  GraphlawEx.admit(Jason.encode!(%{"op" => "definitely-not-an-op"}))

GraphlawEx.stop()
```

`admit/3` replies `{:ok, response_map}` with the decoded JSON response (an
engine-side refusal is `{:ok, %{"ok" => false, ...}}`), or
`{:error, %GraphlawEx.Refusal{}}` for host-side typed refusals (invalid
encoding, over-cap payload, trap, malformed framing, kill-switch, saturation).
The older `{:error, {:refused, _}}` / `{:abi_trap, _}` / `{:abi_frame, _}`
tuples are gone.

Walk the same flow interactively in `graphlaw_ex.livemd`.

## Tests

```bash
cd graphlaw_ex
mix deps.get
mix graphlaw_ex.fetch            # the wasm is not in the repo-published package
PRIV_WASM=/abs/path/graphlaw.wasm mix test
```

`mix test` runs the engine courts against the fetched artifact by default;
`PRIV_WASM` overrides the artifact path and `GRAPHLAW_REQUIRE_ENGINE=1` (set by
`.github/workflows/elixir.yml`) turns a missing artifact into a failure. Chicago
discipline: the real wasm artifact is the collaborator; no mocks, no sleeps.

## Loud-skip semantics

- Artifact absent (without `GRAPHLAW_REQUIRE_ENGINE=1`) → `[SKIP]` with the
  path and how to fix it (`PRIV_WASM`). The skip is printed, never swallowed,
  and is not a pass.
- Artifact absent with `GRAPHLAW_REQUIRE_ENGINE=1` → the court flunks.
- Artifact digest drift → enforced: `start_link/1` returns
  `{:error, {:wasm_digest_mismatch, ...}}` and `GraphlawEx.EngineLoad` checks the
  sha256 pin before any bytes reach the compiler.

## See Also

- `guides/getting_started.md` — tutorial (fetch, lease policy, first query).
- `guides/ash_r2rml_migration.md` — consumer migration guide.
- `graphlaw_ex.livemd` — self-installing livebook driving a real admit.
- `../docs/refusals.md` — the typed refusal register.
