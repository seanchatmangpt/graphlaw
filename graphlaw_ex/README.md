# graphlaw_ex

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
{:ok, response_map}  <--- ok: true          {:error, {:refused, error_map}}
                            (typed refusal raised inside the module)
```

The BEAM holds only the wasm byte-slice; the bridge owns bytes in/out; the
host owns authority, standing, refusals.

- `lib/graphlaw_ex.ex` — GenServer owning the wasmex store/instance; exposes
  `admit/2` and `stop/1`; speaks ABI version 1 (`gl_alloc` / `gl_call` /
  `gl_free`) with packet-4 framing.
- `lib/graphlaw_ex/application.ex` — supervision tree, `GraphlawEx` as a
  named child, opt-in auto-start.

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

The wasm artifact is not a dependency — copy or vendor `priv/graphlaw.wasm`
and its pin `priv/graphlaw.wasm.blake3` next to your application.

## Configuration

| Option | Where | Default | Meaning |
|---|---|---|---|
| `:wasm_path` | start option → `:graphlaw_ex, :wasm_path` app env | `../priv/graphlaw.wasm` | Filesystem path of the graphlaw wasm artifact. A missing or unreadable file is a typed `{:error, {:wasm_load_failed, path, :enoent}}` from `start_link/1`, never a linked-process exit. |
| `:auto_start` | `:graphlaw_ex, :auto_start` app env | `false` | When `true`, the supervision tree starts `GraphlawEx` as a named child. Opt-in so depending on the library never instantiates wasm implicitly and never races a caller's own instance. |

Digest pin security property: the pin in `priv/graphlaw.wasm.blake3`
(`c38b1fdb97ba462dc96000301d7ddf6a5109a7208c68a52f5ddd63ccbd6fda2c`) binds
admission to exact artifact bytes. Verify before starting:

```bash
b3sum priv/graphlaw.wasm
# c38b1fdb97ba462dc96000301d7ddf6a5109a7208c68a52f5ddd63ccbd6fda2c  priv/graphlaw.wasm
```

Any other digest means the bytes are not the pinned artifact — do not admit
through them; results from unpinned bytes are not graphlaw results. The pin
is what makes a receipt replayable: evidence computed under artifact A does
not stand for artifact B.

## Quickstart

```elixir
wasm_path = Path.expand("priv/graphlaw.wasm", File.cwd!())
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

# Tampered: unknown op is refused inside the module.
{:error, {:refused, %{"kind" => "Unsupported", "message" => msg}}} =
  GraphlawEx.admit(Jason.encode!(%{"op" => "definitely-not-an-op"}))

GraphlawEx.stop()
```

`admit/2` replies `{:ok, response_map}` with the decoded JSON response, or a
typed error:

- `{:error, {:refused, error_map}}` — the kernel answered `ok: false` (a
  typed refusal inside the module, e.g. an unknown op).
- `{:error, {:abi_trap, reason}}` — an export call trapped/failed.
- `{:error, {:abi_frame, term}}` — malformed response framing (null buffer,
  truncated response, non-UTF-8, non-JSON).

Walk the same flow interactively in `graphlaw_ex.livemd`.

## Tests

```bash
cd graphlaw_ex
mix deps.get
PRIV_WASM=$PWD/../priv/graphlaw.wasm mix test
```

With the artifact present the instantiation court loads the real module and
admits through the real ABI (last run: `2 tests, 0 failures`, 8.6 s). Without
it the court prints a loud typed skip (`[SKIP] ... A loud skip is not a
pass.`) — a skip is never a pass, and the artifact-missing `start_link` court
still runs because it asserts the typed failure. Chicago discipline: the real
wasm artifact is the collaborator; no mocks, no sleeps.

## Loud-skip semantics

- Artifact absent → `[SKIP]` with the path and how to fix it (`PRIV_WASM`).
  Exit status still green only because nothing was claimed — the skip is
  printed, never swallowed.
- Artifact digest drift → out of scope for the bridge today: the bridge
  checks existence and ABI exports, not the blake3. Pin verification is the
  operator's step above; digest enforcement inside the bridge is tracked by
  the lib surface.

## See Also

- `graphlaw_ex.livemd` — self-installing livebook driving a real admit.
- `../docs/refusals.md` — the typed refusal register.
