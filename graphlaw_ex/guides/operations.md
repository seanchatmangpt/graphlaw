# Operations

How-to tasks for running `graphlaw_ex` in production. For the reasoning behind the supervision
tree see the Architecture guide.

## Persist the kill switch

Set a store path so a revocation survives a node restart:

```elixir
config :graphlaw_ex,
  auto_start: true,
  kill_switch_path: "/var/lib/myapp/kill_switch.json",
  trusted_keys: %{"ops-2026" => ed25519_public_key}
```

With `:kill_switch_path` set, `GraphlawEx.KillSwitch.Server` starts first. It loads the file and
re-verifies every stored document through `GraphlawEx.KillSwitch.install/2` using
`:trusted_keys`. If the file is unreadable, corrupt or any document fails verification, the
server returns `{:error, {:kill_switch_restore_failed, reason}}` and the tree refuses to boot.
A missing file is a clean start. Without the key, a notice is logged and a trip does not
survive a restart (`graphlaw_ex/lib/graphlaw_ex/application.ex:76-91`,
`kill_switch/server.ex:62-69`).

Install a signed revocation through the server so it is persisted:

```elixir
{:ok, _} = GraphlawEx.KillSwitch.Server.install(signed_revocation_doc)
```

Persistence runs only after verification succeeds. If the write fails the in-memory trip stays
and you get `{:error, {:persist_failed, reason}}` (`server.ex:79-91`). Epochs are monotone; an
old epoch is refused as stale. Writes are atomic (temp file, fsync, rename; `store.ex`).

## Attach telemetry

Dev logging:

```elixir
GraphlawEx.Telemetry.attach_default_logger()
```

Production handlers attach to the events in `GraphlawEx.Telemetry.events/0`. The catalog
(`telemetry/catalog.ex:86-157`) declares:

| event | measurements |
|---|---|
| `[:graphlaw_ex, :admit, :start \| :stop \| :exception]` | time, duration |
| `[:graphlaw_ex, :engine, :admit]` | none (metadata `outcome`, `code`, `wasm_sha256`) |
| `[:graphlaw_ex, :host, :call, :start \| :stop \| :exception]` | time, duration |
| `[:graphlaw_ex, :host, :recycle]` | `count` |

Metadata is redacted by default: undeclared keys collapse to `%{bytes:, sha256:}` or
`:redacted`, so request text never reaches handlers. Use `GraphlawEx.Telemetry.redact/1` if you
forward metadata elsewhere.

## Size the pool

Pool size is the `size:` option of `GraphlawEx.Pool`, not application config; a `:pool_size`
config entry is rejected. Default is `System.schedulers_online/0`. One host serves one call at a
time, so size bounds concurrency.

```elixir
{GraphlawEx.Pool, size: 8}
```

Limits live under `config :graphlaw_ex` (`config/keys.ex:34-47`), with defaults including
`max_queue: 64` and `timeout_ms: 5_000`. A full queue sheds with `:saturated`; raise
`max_queue` only after measuring. Supervise the pool with `restart: :transient`.

## Run with zero egress

The engine imports only the WASI functions in the manifest's allowlist
(`GraphlawEx.WasmConfig.import_allowlist/0`); `GraphlawEx.EngineLoad` refuses any other import
at boot. For the host side, avoid network at start-up:

1. Bake the engine with `mix graphlaw_ex.fetch` at build time, or copy it and set
   `GRAPHLAW_WASM_PATH` (Releases guide).
2. Run `mix graphlaw_ex.doctor` as a build step.
3. Signed leases and revocations verify offline against `:trusted_keys`; no key server is
   contacted.

## Check lease policy

Production keeps the default `lease_policy: :required`. Do not set `:none_allowed` outside tests.

## See Also

- Architecture guide
- Releases guide
- Refusals and Errors guide
