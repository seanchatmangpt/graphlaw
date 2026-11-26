# SPDX-FileCopyrightText: 2026 Sean Chatman <https://github.com/seanchatmangpt/graphlaw/graphs/contributors>
#
# SPDX-License-Identifier: MIT
defmodule GraphlawEx.Pool do
  @moduledoc """
  N supervised `GraphlawEx.Host` instances behind one name.

  A single host serializes every `gl_alloc`/`gl_call`/`gl_free` transaction, so
  one host caps throughput at one core. This supervisor starts `:size` hosts
  (default `System.schedulers_online/0`). Member 0 keeps the registered name
  `GraphlawEx.Host`; every member joins a duplicate-key
  `GraphlawEx.Pool.Registry` under `:members` while it has a live engine
  instance, and `request/2` routes to the member with the shortest mailbox
  (`pick/1`). A member whose engine is unavailable leaves `:members` and joins
  `:unavailable` until it heals, so a dead member with an always-empty mailbox
  can never win the routing. When no member is live, `request/2` forwards to an
  unavailable one so the caller receives that member's real typed refusal
  instead of an opaque error.

  Routing by mailbox length is best-effort: the length is read before the call
  is enqueued. Ties are broken at random (shuffled first), which spreads a
  simultaneous burst across the members instead of piling it on one.

  The strategy is `:rest_for_one` over `[Registry, Pool.Leases, Pool.Workers]`.
  `Pool.Workers` is a nested `:one_for_one` supervisor holding the Host members,
  so one Host crash restarts only that Host. A registry crash drops all
  registrations, so the workers started after it restart and re-register.

  Restart intensity is explicit: 10 restarts in 30 seconds for the Pool, and
  `max(10, 3 * size)` in 30 seconds for `Pool.Workers` (scales with members). Past that the supervisor exits; consumers should supervise
  `GraphlawEx.Pool` with `restart: :transient` so a deliberate shutdown is not
  restarted but an abnormal exit is.

  When `:capacity` equals `:size` (the default), the claimed slot index selects
  the member, so at most one request is in flight per host. If the slot's member
  is unavailable (or the member set is incomplete) routing falls through to
  `pick/1`; for any other capacity routing uses `pick/1` directly.
  """

  use Supervisor, restart: :transient

  alias GraphlawEx.Refusal

  @registry GraphlawEx.Pool.Registry
  @gate_key {__MODULE__, :gate}

  @typedoc "Pool options: `:name`, `:size`, plus any `GraphlawEx.Host` option."
  @type opts :: [
          {:name, atom()}
          | {:size, pos_integer()}
          | {:wasm_path, String.t()}
          | {:bytes, binary()}
          | {:expected_sha256, String.t() | :unpinned}
          | {atom(), term()}
        ]

  @doc "The registry the pool's members join (under the key `:members`)."
  @spec registry() :: atom()
  def registry, do: @registry

  @doc """
  Starts the pool. Options: `:name` (supervisor name, default this module),
  `:size` (default `System.schedulers_online/0`), and any `GraphlawEx.Host`
  option, passed to every member.
  """
  @spec start_link(opts()) :: Supervisor.on_start()
  def start_link(opts \\ []) do
    {name, opts} = Keyword.pop(opts, :name, __MODULE__)
    Supervisor.start_link(__MODULE__, opts, name: name)
  end

  @doc false
  @impl Supervisor
  @spec init(opts()) :: {:ok, {Supervisor.sup_flags(), [Supervisor.child_spec()]}}
  def init(opts) do
    n = size(opts)
    host_opts = Keyword.drop(opts, [:size, :capacity, :registry, :name])

    capacity = Keyword.get(opts, :capacity, n)

    unless is_integer(capacity) and capacity >= 1 do
      raise ArgumentError,
            "GraphlawEx.Pool :capacity must be an integer >= 1, got: #{inspect(capacity)}"
    end

    Supervisor.init(
      [
        {Registry, keys: :duplicate, name: @registry},
        {__MODULE__.Leases, capacity: capacity, size: n},
        {__MODULE__.Workers, size: n, registry: @registry, host_opts: host_opts}
      ],
      strategy: :rest_for_one,
      max_restarts: 10,
      max_seconds: 30
    )
  end

  @doc """
  Member count of the running pool, read from the gate the lease server owns.
  Without a running pool it is the size a pool started with no options would
  have (`System.schedulers_online/0`).
  """
  @spec size() :: pos_integer()
  def size do
    case :persistent_term.get(@gate_key, nil) do
      %{size: n} -> n
      nil -> size([])
    end
  end

  @doc "Pool size resolved from `opts[:size]`, then the scheduler count."
  @spec size(opts()) :: pos_integer()
  def size(opts) do
    case Keyword.get(opts, :size) do
      n when is_integer(n) and n > 0 -> n
      _ -> System.schedulers_online()
    end
  end

  @doc """
  Routes `request` to the member with the shortest mailbox via
  `GraphlawEx.Host.request/3`. With no member at all, returns a typed
  `:call_exited` refusal (the closed table has no separate not-started code).

  The contract is uniform: every failure is `{:error, %GraphlawEx.Refusal{}}`,
  including admission saturation (code `:saturated`, details `capacity`). Callers
  wanting the public `GraphlawEx.Error` wrap it with `GraphlawEx.Error.wrap/1`
  (`:saturated` maps to `:REFUSED_RESOURCE_BOUND`).
  """
  @spec request(map(), keyword()) :: {:ok, map()} | {:error, Refusal.t()}
  def request(request, opts \\ []) when is_map(request) do
    with {:ok, lease} <- acquire() do
      try do
        case route(lease) do
          nil ->
            {:error,
             Refusal.new(
               :call_exited,
               "no GraphlawEx.Host member is registered in the pool; start GraphlawEx.Pool " <>
                 "(GraphlawEx.Application with auto_start: true starts it)"
             )}

          pid ->
            retry_once(GraphlawEx.Host.request(pid, request, opts), pid, request, opts)
        end
      after
        release(lease)
      end
    end
  end

  # One retry through `pick/1` when the chosen member exited mid-call and a
  # different live member exists.
  defp retry_once({:error, %Refusal{code: :call_exited}} = original, pid, request, opts) do
    case pick() do
      other when is_pid(other) and other != pid ->
        GraphlawEx.Host.request(other, request, opts)

      # No other live member: never re-send to the pid that just exited.
      _ ->
        original
    end
  end

  defp retry_once(result, _pid, _request, _opts), do: result

  # capacity == size: slot index maps to member index, so each host sees at
  # most one in-flight request. Otherwise (or when the member set is not
  # complete) fall back to shortest-mailbox routing.
  defp route(nil), do: pick() || List.first(unavailable_members())

  defp route({%{capacity: cap, size: size}, _id, slot}) when cap == size do
    live = members()
    all = Enum.sort(live ++ unavailable_members())

    # The slot-selected member wins only if it is a live (routable) member; an
    # unavailable one falls through to `pick/1`, which sees live members only.
    # Forwarding to an unavailable member happens solely when none is live, so
    # the caller gets that member's real typed refusal.
    case length(all) == size && Enum.at(all, slot - 1) do
      pid when is_pid(pid) ->
        if pid in live and Process.alive?(pid),
          do: pid,
          else: pick() || List.first(unavailable_members())

      _ ->
        pick() || List.first(unavailable_members())
    end
  end

  defp route(_lease), do: pick() || List.first(unavailable_members())

  @doc """
  Free admission slots. Equals the pool size when no request is in flight,
  including after external `:kill` of callers or workers. Without a running
  pool supervisor there is no gate and the result is `:ungated`.
  """
  @spec available() :: non_neg_integer() | :ungated
  def available do
    case :persistent_term.get(@gate_key, nil) do
      nil -> :ungated
      %{slots: slots, capacity: cap} -> cap - occupied(slots, cap)
    end
  end

  # Lock-free admission: ownership IS the slot. A caller claims a free slot with
  # `:atomics.compare_exchange/4` under a unique lease id; no slot free means
  # saturated. There is no separate in-flight counter to drift. A caller killed
  # after the claim and before its `{:lease, pid, id}` cast is processed is NOT
  # covered by a monitor, so that slot would leak; `Pool.Leases` runs a
  # reconcile sweep every second that frees any slot whose id it has not
  # tracked on two consecutive sweeps. A caller killed after the cast is freed
  # by the lease monitor (exactly the slot holding the dead caller's id).
  defp acquire do
    case :persistent_term.get(@gate_key, nil) do
      nil ->
        if supervisor_registered?() do
          # Fail closed: a pool exists but its gate is absent (restarting).
          {:error,
           Refusal.new(:call_exited, "pool admission gate is unavailable (pool restarting)")}
        else
          {:ok, nil}
        end

      %{slots: slots, capacity: cap, leases: leases} = gate ->
        id = :erlang.unique_integer([:positive])

        if slot = claim(slots, cap, id) do
          # Monitor only after a successful claim: saturated bursts never touch
          # the Leases server.
          GenServer.cast(leases, {:lease, self(), id})
          {:ok, {gate, id, slot}}
        else
          {:error,
           Refusal.new(
             :saturated,
             "REFUSED_RESOURCE_BOUND: all #{cap} pool slots are leased",
             %{
               refused: :REFUSED_RESOURCE_BOUND,
               capacity: cap
             }
           )}
        end
    end
  end

  defp supervisor_registered? do
    # The registry alone does not make a gated pool: only the supervisor or the
    # lease server (which owns the gate) being up with the gate absent is a
    # restart window.
    Enum.any?([__MODULE__, __MODULE__.Leases], &Process.whereis/1)
  end

  # Returns the claimed 1-based slot index, or nil when saturated.
  defp claim(slots, cap, id) do
    start = :rand.uniform(cap)

    Enum.find_value(0..(cap - 1), fn off ->
      slot = rem(start + off - 1, cap) + 1
      if :atomics.compare_exchange(slots, slot, 0, id) == :ok, do: slot
    end)
  end

  defp release(nil), do: :ok

  defp release({%{slots: slots, capacity: cap, leases: leases}, id, slot}) do
    # O(1): the claimed slot is known; the scan is only a fallback.
    if :atomics.compare_exchange(slots, slot, id, 0) != :ok, do: free_slot(slots, cap, id)
    GenServer.cast(leases, {:release, id})
  end

  @doc false
  @spec free_slot(:atomics.atomics_ref(), pos_integer(), integer()) :: boolean()
  def free_slot(slots, cap, id) do
    Enum.any?(1..cap, fn i -> :atomics.compare_exchange(slots, i, id, 0) == :ok end)
  end

  @doc false
  @spec occupied(:atomics.atomics_ref(), pos_integer()) :: non_neg_integer()
  def occupied(slots, cap), do: Enum.count(1..cap, fn i -> :atomics.get(slots, i) != 0 end)

  @doc """
  Engine identity of the pool: `GraphlawEx.Host.info/2` of the first live member
  (`{:ok, %{wasm_sha256: _, ...}}`), or a typed refusal when no member is live.
  """
  @spec info(atom()) :: {:ok, map()} | {:error, Refusal.t()}
  def info(registry \\ @registry) do
    case pick(registry) do
      nil ->
        {:error,
         Refusal.new(:call_exited, "no live GraphlawEx.Host member is registered in the pool")}

      pid ->
        # Routed to the member with the shortest mailbox (never a fixed, possibly
        # busy member); one retry through `pick/1` if it exited mid-call.
        case GraphlawEx.Host.info(pid) do
          {:error, %Refusal{code: :call_exited}} = err ->
            case pick(registry) do
              nil -> err
              other -> GraphlawEx.Host.info(other)
            end

          result ->
            result
        end
    end
  end

  @doc "The live member with the shortest mailbox, or `nil` when none is routable."
  @spec pick(atom()) :: pid() | nil
  def pick(registry \\ @registry) do
    # Shuffled first: a burst of callers reads the same (often all-zero) mailbox
    # lengths, and `Enum.min_by/3` would send every one of them to the first
    # member of the list.
    registry
    |> members()
    |> Enum.shuffle()
    |> Enum.map(fn pid -> {queue_len(pid), pid} end)
    |> Enum.reject(fn {len, _pid} -> is_nil(len) end)
    |> Enum.min_by(&elem(&1, 0), fn -> {nil, nil} end)
    |> elem(1)
  end

  @doc "Every live member pid."
  @spec members(atom()) :: [pid()]
  def members(registry \\ @registry), do: registered(registry, :members)

  @doc "Every member pid whose engine is currently unavailable (still supervised, retrying)."
  @spec unavailable_members(atom()) :: [pid()]
  def unavailable_members(registry \\ @registry), do: registered(registry, :unavailable)

  defp registered(registry, key) do
    if Process.whereis(registry),
      do: registry |> Registry.lookup(key) |> Enum.map(&elem(&1, 0)),
      else: []
  rescue
    # A registry mid-restart has its name registered before its key table
    # exists; `Registry.lookup/2` raises. No member is routable in that window.
    ArgumentError -> []
  end

  defp queue_len(pid) do
    case Process.info(pid, :message_queue_len) do
      {:message_queue_len, len} -> len
      nil -> nil
    end
  end
end
