# SPDX-FileCopyrightText: 2026 Sean Chatman <https://github.com/seanchatmangpt/graphlaw/graphs/contributors>
#
# SPDX-License-Identifier: MIT
defmodule GraphlawEx.Application do
  @moduledoc """
  Supervision tree for the GraphlawEx bridge.

  ## Opt-in start

  Auto-start is opt-in (`config :graphlaw_ex, auto_start: true`). By default the
  application starts an empty `:one_for_one` supervisor
  (`GraphlawEx.Supervisor`) and does no work at boot: no file read, no WASM
  compile, no processes. Consumers either enable `auto_start` or start
  `GraphlawEx.Pool` / `GraphlawEx.Host` themselves in their own tree. This
  keeps the tree from racing a test's own instance or loading the wrong
  artifact byte-slice.

  ## Boot order (auto_start: true)

  `:one_for_one`, children started in dependency order. When `:kill_switch_path` is
  configured, `GraphlawEx.KillSwitch.Server` is the first child (also started
  in the empty non-auto_start tree); it restores and re-verifies persisted
  revocations and refuses to start on failure. Without the key no server runs,
  and a one-time notice is logged only when `auto_start: true`.

    1. `:engine_boot` - sha256 pin + import-surface admission (existing
       MANIFEST path), compiles the `Wasmex.Module` once and publishes it in
       `:persistent_term` (`GraphlawEx.EngineLoad.boot/2`). Runs
       synchronously inside the supervisor's start sequence and returns
       `:ignore` on success, so nothing after it starts until the module
       exists. There is no `:atomics` gate at boot; the only concurrency gate
       is `GraphlawEx.Pool.Leases`.
    2. `GraphlawEx.Pool` - consumes the published module.

  `start/2` validates `config :graphlaw_ex` first (`GraphlawEx.Config.validate!/0`)
  and raises `ArgumentError` on an unknown or malformed key. Every boot error is
  logged with a remediation (`mix graphlaw_ex.fetch` / `GRAPHLAW_WASM_PATH`) and
  the cache path before it is returned.

  If the engine file is unreadable, `start/2` itself returns
  `{:error, {:wasm_unreadable, %{path: path, reason: posix}}}` before any
  supervisor exists (only the artifact path and the POSIX reason appear in the
  report; no bytes, hashes or environment). If admission refuses (sha256 pin or
  import surface), the `:engine_boot` child start returns `{:error, reason}`,
  the supervisor refuses to start, and the application exits with a reason of
  the form `{:shutdown, {:failed_to_start_child, :engine_boot, reason}}`. If
  `GraphlawEx.Pool` or `KillSwitch.Server` crashes later, only that child
  restarts (`:engine_boot` is `:ignore`d, never restarted; the module lives in
  `:persistent_term`).

  There is deliberately no BLAKE3 check at boot: that is `mix
  graphlaw_ex.verify`, run at publish/CI time, never in a consumer's boot.
  """
  use Application

  require Logger

  alias GraphlawEx.EngineLoad
  alias GraphlawEx.WasmConfig

  @impl true
  def start(_type, _args) do
    :ok = GraphlawEx.Config.validate!()

    auto_start? = Application.get_env(:graphlaw_ex, :auto_start, false) == true
    kill_switch = kill_switch_children(auto_start?)

    if auto_start? do
      boot(kill_switch)
    else
      Supervisor.start_link(kill_switch, supervisor_opts())
    end
  end

  # The durable trip path is opt-in via :kill_switch_path. When configured the
  # Server starts first, so a persisted trip is enforced before the engine boots
  # or any pool worker exists; a failed restore refuses the whole tree.
  # The "no durable path" notice is only relevant when the tree runs an engine
  # (auto_start: true); an idle bridge has nothing to trip.
  defp kill_switch_children(auto_start?) do
    case Application.get_env(:graphlaw_ex, :kill_switch_path) do
      nil ->
        if auto_start? do
          Logger.notice(
            "graphlaw_ex: no durable kill-switch path configured (:kill_switch_path); " <>
              "a tripped kill-switch will not survive a node restart"
          )
        end

        []

      _path ->
        [GraphlawEx.KillSwitch.Server]
    end
  end

  defp boot(kill_switch) do
    path = WasmConfig.wasm_path()

    case read(path) do
      {:ok, bytes} ->
        expected = WasmConfig.expected_sha256()
        Supervisor.start_link(kill_switch ++ children(bytes, expected), supervisor_opts())

      {:error, reason} = error ->
        log_boot_error(reason)
        error
    end
  end

  defp log_boot_error({:wasm_unreadable, %{path: path, reason: posix}}) do
    Logger.error(
      "graphlaw_ex: cannot read engine #{path} (#{:file.format_error(posix)}). " <>
        "Run `mix graphlaw_ex.fetch` or set GRAPHLAW_WASM_PATH. " <>
        "Cache path: #{cache_hint()}"
    )
  end

  defp log_boot_error(reason) do
    Logger.error(
      "graphlaw_ex: engine boot failed: #{inspect(reason, limit: 10, printable_limit: 200)}. " <>
        "Run `mix graphlaw_ex.fetch` or set GRAPHLAW_WASM_PATH. Cache path: #{cache_hint()}"
    )
  end

  defp cache_hint do
    case WasmConfig.cache_path_result() do
      {:ok, path} -> path
      {:error, {_code, message}} -> message
    end
  end

  @doc false
  @spec children(binary(), String.t() | nil) :: [Supervisor.child_spec()]
  def children(bytes, expected) do
    [
      %{
        id: :engine_boot,
        start: {__MODULE__, :engine_boot, [bytes, expected]},
        restart: :transient,
        type: :worker
      },
      {GraphlawEx.Pool, [bytes: bytes, expected_sha256: expected]}
    ]
  end

  @doc false
  # Synchronous one-shot child: `:ignore` on success (nothing to supervise
  # afterwards, state lives in :persistent_term), `{:error, reason}` on refusal
  # so the supervisor fails loudly with the typed reason.
  @spec engine_boot(binary(), String.t() | nil) :: :ignore | {:error, term()}
  def engine_boot(bytes, expected) do
    case EngineLoad.boot(bytes, expected_sha256: expected) do
      {:ok, _admitted} ->
        :ignore

      {:error, reason} ->
        log_boot_error(reason)
        {:error, reason}
    end
  end

  # Boot ORDER is the child list order. `:one_for_one`: `engine_boot` returns
  # `:ignore` (never restarted) and the data plane reads `:persistent_term`, so a
  # later KillSwitch.Server crash must not tear down the pool/engine.
  defp supervisor_opts,
    do: [strategy: :one_for_one, max_restarts: 10, max_seconds: 30, name: GraphlawEx.Supervisor]

  defp read(path) do
    case File.read(path) do
      {:ok, bytes} -> {:ok, bytes}
      {:error, reason} -> {:error, {:wasm_unreadable, %{path: path, reason: reason}}}
    end
  end
end
