# SPDX-FileCopyrightText: 2026 Sean Chatman <https://github.com/seanchatmangpt/graphlaw/graphs/contributors>
#
# SPDX-License-Identifier: MIT
defmodule GraphlawEx.KillSwitch.Server do
  @moduledoc """
  Supervised owner of kill-switch durability.

  At `init/1` it loads `GraphlawEx.KillSwitch.Store` and re-verifies every
  stored document, in epoch order, through `GraphlawEx.KillSwitch.install/2`
  with trusted keys from `Application.get_env(:graphlaw_ex, :trusted_keys, %{})`.
  Fail-closed: if the file exists but cannot be read, is corrupt, or any
  document fails verification, `start_link/1` returns
  `{:error, {:kill_switch_restore_failed, reason}}` and the supervisor refuses
  to boot. A missing file is a clean start.

  `install/1` serializes installs through the GenServer, calls
  `KillSwitch.install/2`, and persists the document only after success. If
  persisting fails the in-memory trip remains (fail-closed), `docs/0` keeps
  mirroring only what is durable, a `Logger.error` alarm is raised, and the
  error is returned as `{:error, {:persist_failed, reason}}`; re-sending the
  identical document retries persistence. The server is the single durable
  writer and uses the store path captured at boot. Trusted keys are read per
  install (rotation needs no restart).

  After a durable persist, `config :graphlaw_ex, :kill_switch_propagate,
  {Module, :function}` is invoked best-effort with the signed document in an
  unlinked task (errors logged, never blocking). Installs emit
  `[:graphlaw_ex, :kill_switch, :installed | :refused]` telemetry.

  Note: `KillSwitch` keeps only the highest-epoch installed scope in memory;
  the store keeps the full additive document set for audit and replay.

  ## Wiring order

  Start this server in `GraphlawEx.Application` children BEFORE
  `GraphlawEx.Pool`, so a persisted trip is enforced before any worker can
  serve a call:

      children = [
        GraphlawEx.KillSwitch.Server,
        GraphlawEx.Pool
        # ...
      ]

  ## Public functions

  | function | purpose |
  |---|---|
  | `start_link/1` | start; restores and re-verifies stored documents |
  | `child_spec/1` | supervisor child spec (permanent) |
  | `install/1`, `install/2` | serialized verified install, then persist |
  | `docs/0` | documents currently held (epoch order) |
  """

  use GenServer

  require Logger

  alias GraphlawEx.KillSwitch
  alias GraphlawEx.KillSwitch.Store

  @doc "Starts the server; refuses to start when stored documents fail verification."
  @spec start_link(keyword()) :: GenServer.on_start()
  def start_link(opts \\ []) do
    GenServer.start_link(__MODULE__, opts, name: Keyword.get(opts, :name, __MODULE__))
  end

  @doc "Install a signed revocation document; persists after verification succeeds."
  @spec install(map()) :: {:ok, map()} | {:error, term()}
  def install(doc), do: install(__MODULE__, doc)

  @spec install(GenServer.server(), map()) :: {:ok, map()} | {:error, term()}
  def install(server, doc), do: GenServer.call(server, {:install, doc})

  @doc "Documents currently held, in epoch order."
  @spec docs() :: [map()]
  def docs, do: GenServer.call(__MODULE__, :docs)

  @impl true
  def init(_opts) do
    keys = trusted_keys()
    path = Store.path()

    with {:ok, docs} <- Store.load(),
         :ok <- replay(docs, keys) do
      {:ok, %{docs: docs, path: path, pending: nil}}
    else
      {:error, reason} -> {:stop, {:kill_switch_restore_failed, reason}}
    end
  end

  # Trusted keys are read per install, so key rotation takes effect without a
  # restart (boot replay reads them once, at init).
  defp trusted_keys, do: Application.get_env(:graphlaw_ex, :trusted_keys, %{})

  @impl true
  def handle_call({:install, doc}, _from, %{docs: docs} = state) do
    case verify(doc, state) do
      {:ok, result} ->
        case persist(state, docs, doc) do
          :ok ->
            new_docs =
              Enum.sort_by([doc | Enum.reject(docs, &(epoch(&1) == epoch(doc)))], &epoch/1)

            propagate(doc)
            {:reply, {:ok, result}, %{state | docs: new_docs, pending: nil}}

          {:error, reason} ->
            # The in-memory trip stays (fail closed) but `docs` keeps mirroring
            # what is durable; the doc is remembered so an identical retry can
            # re-attempt persistence instead of hitting :stale_epoch.
            Logger.error(
              "kill-switch: epoch #{epoch(doc)} installed in memory but NOT persisted: " <>
                inspect(reason)
            )

            {:reply, {:error, {:persist_failed, reason}}, %{state | pending: doc}}
        end

      {:error, _} = err ->
        {:reply, err, state}
    end
  end

  def handle_call(:docs, _from, state), do: {:reply, state.docs, state}

  # An identical retry of a document whose persistence failed skips
  # re-installation (its epoch is already live in memory).
  defp verify(doc, %{pending: doc}) when not is_nil(doc),
    do: {:ok, %{epoch: epoch(doc), scope: get_in(doc, ["revocation", "scope"])}}

  defp verify(doc, _state), do: KillSwitch.install(doc, trusted_keys())

  defp persist(%{path: nil}, _docs, _doc), do: :ok
  defp persist(%{path: path}, docs, doc), do: Store.append(path, docs, doc)

  # Optional best-effort propagation: `config :graphlaw_ex, :kill_switch_propagate,
  # {Module, :function}`; invoked as `Module.function(signed_doc)` after durable
  # persist, in an unlinked task so it never blocks or fails the install.
  defp propagate(doc) do
    case Application.get_env(:graphlaw_ex, :kill_switch_propagate) do
      {mod, fun} when is_atom(mod) and is_atom(fun) ->
        Task.start(fn ->
          try do
            apply(mod, fun, [doc])
          rescue
            e -> Logger.error("kill-switch propagate failed: #{Exception.message(e)}")
          catch
            kind, reason ->
              Logger.error("kill-switch propagate failed: #{inspect({kind, reason})}")
          end
        end)

        :ok

      _ ->
        :ok
    end
  end

  defp replay(docs, keys) do
    Enum.reduce_while(docs, :ok, fn doc, :ok ->
      case KillSwitch.install(doc, keys) do
        {:ok, _} ->
          {:cont, :ok}

        {:error, refusal} ->
          # Same-VM supervisor restart: the persistent_term epoch already covers
          # this document, so only a stale_epoch refusal is tolerated.
          if stale_epoch?(refusal) and epoch(doc) <= KillSwitch.current_epoch() do
            {:cont, :ok}
          else
            {:halt, {:error, {:unverified_document, epoch(doc), refusal}}}
          end
      end
    end)
  end

  defp stale_epoch?(%GraphlawEx.Refusal{code: :lease_revoked, details: %{reason: :stale_epoch}}),
    do: true

  defp stale_epoch?(_), do: false

  defp epoch(%{"revocation" => %{"epoch" => e}}) when is_integer(e), do: e
  defp epoch(_), do: 0
end
