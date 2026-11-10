# graphlaw_ex

As of 2026-11-09 this README was verified against `mix.exs` (version `0.1.0`,
`wasmex ~> 0.15.1`, `jason`, `telemetry`), `lib/`, `priv/graphlaw/MANIFEST.json`
(graphlaw `26.10.28`, ABI 1) and `test/`.

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
    {:graphlaw_ex, path: "../graphlaw_ex"},
    {:wasmex, "~> 0.15.1"},
    {:jason, "~> 1.4"}
  ]
end
```

The wasm artifact is not a dependency — copy or vendor `priv/graphlaw/graphlaw.wasm`
and its pin `priv/graphlaw/MANIFEST.json` (`artifact.blake3` / `artifact.sha256`)
next to your application. `lib/graphlaw_ex/wasm_config.ex` is the truth for how
the pin is located and checked.

## Configuration

| Option | Where | Default | Meaning |
|---|---|---|---|
| `:wasm_path` | start option, then `GRAPHLAW_WASM_PATH` env, then `:graphlaw_ex, :wasm_path` app env (`GraphlawEx.WasmConfig.wasm_path/1`) | `priv/graphlaw/graphlaw.wasm` | Filesystem path of the graphlaw wasm artifact. A missing or unreadable file is a typed `{:error, {:wasm_load_failed, path, :enoent}}` from `start_link/1`, never a linked-process exit. |
| `:expected_sha256` | start option | manifest pin | Digest the artifact must match. `:unpinned` is the explicit opt-out; an explicit `:wasm_path` that differs from the vendored path is treated as `:unpinned`, while an env/app-config path stays pinned. |
| `:auto_start` | `:graphlaw_ex, :auto_start` app env | `false` | When `true`, the supervision tree starts `GraphlawEx` as a named child. Opt-in so depending on the library never instantiates wasm implicitly and never races a caller's own instance. |

Digest pin security property: the pin in `priv/graphlaw/MANIFEST.json`
(enforced on load via `artifact.sha256`; `artifact.blake3` is the published identity)
(`artifact.blake3`
`c92caeb6d56792587f2939769678e02b77c868f2919ef8566060e02f3c6c1b9c`,
`artifact.sha256`
`62cd95d4ef7294dcd5976a2f685b71580bb834816e9f4bba0d13d5555e35c8f8`,
artifact size 62,909,537 bytes) binds admission to exact artifact bytes.
Verify before starting:

```bash
b3sum priv/graphlaw/graphlaw.wasm
# c92caeb6d56792587f2939769678e02b77c868f2919ef8566060e02f3c6c1b9c  priv/graphlaw/graphlaw.wasm
```

Any other digest means the bytes are not the pinned artifact — do not admit
through them; results from unpinned bytes are not graphlaw results. The pin
is what makes a receipt replayable: evidence computed under artifact A does
not stand for artifact B.

## Quickstart

```elixir
wasm_path = Path.expand("priv/graphlaw/graphlaw.wasm", File.cwd!())
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
PRIV_WASM=$PWD/priv/graphlaw/graphlaw.wasm mix test
```

`mix test` runs the engine courts against the vendored artifact by default;
`PRIV_WASM` overrides the artifact path and `GRAPHLAW_REQUIRE_ENGINE=1` (set by
`.github/workflows/elixir.yml`) turns a missing artifact into a failure. As of
2026-11-09 `test/` declares 149 `test "..."` cases across 13 files; that is a
static count, this pass did not run `mix test` (the earlier "2 tests, 0 failures,
8.6 s" line was from an older, smaller suite and is retired). Chicago
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

- `graphlaw_ex.livemd` — self-installing livebook driving a real admit.
- `../docs/refusals.md` — the typed refusal register.
