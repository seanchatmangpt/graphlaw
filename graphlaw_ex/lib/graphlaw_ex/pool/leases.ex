# SPDX-FileCopyrightText: 2026 Sean Chatman <https://github.com/seanchatmangpt/graphlaw/graphs/contributors>
#
# SPDX-License-Identifier: MIT
defmodule GraphlawEx.Pool.Leases do
  @moduledoc false
  # Owns the gate and monitors lease holders. A DOWN frees whatever slot the
  # dead caller held. Slots are the sole admission state (no counter).
  #
  # Window: a caller claims a slot with a CAS and only then casts
  # `{:lease, pid, id}` here. A caller killed between the claim and the cast
  # has no monitor, so its slot would leak. The reconcile sweep closes it:
  # every @sweep_ms it scans the slots; a nonzero slot whose id this server does
  # not track is "suspect", and one still suspect on the next sweep is freed.
  # The two-sweep rule tolerates a lease cast still in the mailbox.
  #
  # Bottleneck note (DEFERRED): each request costs one monitor + two casts here.
  # Admission itself is lock-free (CAS), so this server is off the admit path;
  # dropping the per-request monitor would reopen the killed-caller leak that
  # the monitor closes, so it is kept deliberately.
  use GenServer

  require Logger

  alias GraphlawEx.Pool

  @sweep_ms 1_000

  def start_link(opts), do: GenServer.start_link(__MODULE__, opts, name: __MODULE__)

  @impl GenServer
  def init(opts) do
    Process.flag(:trap_exit, true)
    cap = Keyword.fetch!(opts, :capacity)
    slots = :atomics.new(cap, signed: true)

    :persistent_term.put(
      {Pool, :gate},
      %{
        slots: slots,
        capacity: cap,
        size: Keyword.get(opts, :size, cap),
        leases: self()
      }
    )

    schedule_sweep()
    {:ok, %{slots: slots, cap: cap, mons: %{}, ids: %{}, suspect: MapSet.new()}}
  end

  @impl GenServer
  def handle_cast({:lease, pid, id}, state) do
    ref = Process.monitor(pid)

    {:noreply,
     %{
       state
       | mons: Map.put(state.mons, ref, id),
         ids: Map.put(state.ids, id, ref),
         suspect: MapSet.delete(state.suspect, id)
     }}
  end

  def handle_cast({:release, id}, state) do
    case Map.pop(state.ids, id) do
      {nil, _} ->
        {:noreply, state}

      {ref, ids} ->
        Process.demonitor(ref, [:flush])
        {:noreply, %{state | ids: ids, mons: Map.delete(state.mons, ref)}}
    end
  end

  @impl GenServer
  def handle_info({:DOWN, ref, :process, _pid, _reason}, state) do
    case Map.pop(state.mons, ref) do
      {nil, _} ->
        {:noreply, state}

      {id, mons} ->
        # Free what the dead caller held, if anything.
        Pool.free_slot(state.slots, state.cap, id)

        {:noreply, %{state | mons: mons, ids: Map.delete(state.ids, id)}}
    end
  end

  def handle_info(:sweep, state) do
    schedule_sweep()
    {:noreply, %{state | suspect: reconcile(state)}}
  end

  def handle_info({:EXIT, from, reason}, state) do
    Logger.warning(
      "GraphlawEx.Pool.Leases: unexpected EXIT from #{inspect(from)}: " <>
        inspect(reason, limit: 5, printable_limit: 100)
    )

    {:noreply, state}
  end

  def handle_info(_other, state), do: {:noreply, state}

  @impl GenServer
  def terminate(_reason, _state) do
    # Erase only our own gate: a newer init may already have overwritten it.
    case :persistent_term.get({Pool, :gate}, nil) do
      %{leases: pid} when pid == self() -> :persistent_term.erase({Pool, :gate})
      _ -> :ok
    end

    :ok
  end

  defp schedule_sweep, do: Process.send_after(self(), :sweep, @sweep_ms)

  # Returns the new suspect set: untracked ids seen for the first time.
  #
  # Mailbox-lag guard: a `{:lease, _, id}` cast for a live in-flight slot may
  # still be queued behind this very `:sweep`. When the mailbox is non-empty a
  # suspect is kept suspect (not freed) and re-judged next sweep, so a lagging
  # server never evicts a live slot.
  defp reconcile(%{slots: slots, cap: cap, ids: ids, suspect: suspect}) do
    {:message_queue_len, backlog} = Process.info(self(), :message_queue_len)

    Enum.reduce(1..cap, MapSet.new(), fn i, acc ->
      case :atomics.get(slots, i) do
        0 ->
          acc

        id ->
          cond do
            Map.has_key?(ids, id) ->
              acc

            MapSet.member?(suspect, id) and backlog > 0 ->
              MapSet.put(acc, id)

            MapSet.member?(suspect, id) ->
              Pool.free_slot(slots, cap, id)
              acc

            true ->
              MapSet.put(acc, id)
          end
      end
    end)
  end
end
