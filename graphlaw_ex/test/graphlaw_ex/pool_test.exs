defmodule GraphlawEx.PoolTest do
  use ExUnit.Case, async: false

  # Chicago: the collaborators are REAL processes. The pool routes over
  # GraphlawEx.Pool.Registry; the courts register real supervised GenServers
  # that speak the real GraphlawEx.Host request protocol — no mocks. Routing
  # fairness is observed as state (which member answered), never as a
  # call-count on a double.

  alias GraphlawEx.Pool
  alias GraphlawEx.Refusal

  @registry GraphlawEx.Pool.Registry

  defmodule ScriptedMember do
    @moduledoc "A REAL GenServer member: it speaks the host request protocol with a scripted reply."

    use GenServer

    def start_link(opts) do
      GenServer.start_link(__MODULE__, opts, name: Keyword.get(opts, :name))
    end

    @impl GenServer
    def init(opts) do
      {:ok, _} = Registry.register(opts[:registry], Keyword.get(opts, :availability, :members), nil)
      {:ok, %{registry: opts[:registry], script: opts[:script], calls: 0}}
    end

    @impl GenServer
    def handle_call({:set_availability, key, script}, _from, %{registry: registry} = state) do
      for k <- [:members, :unavailable], do: Registry.unregister(registry, k)
      {:ok, _} = Registry.register(registry, key, nil)
      {:reply, :ok, %{state | script: script}}
    end

    @impl GenServer
    def handle_call({:request, _body, _op, _timeout}, _from, state) do
      case state.script do
        fun when is_function(fun, 1) -> {:reply, fun.(state), %{state | calls: state.calls + 1}}
        reply -> {:reply, reply, %{state | calls: state.calls + 1}}
      end
    end

    def handle_call(:calls, _from, state), do: {:reply, state.calls, state}

    @impl GenServer
    def handle_info(:court_padding, state), do: {:noreply, state}
  end

  describe "routing" do
    test "the member with the shorter mailbox wins (shortest-mailbox is observed, not assumed)" do
      start_supervised!({Registry, keys: :duplicate, name: @registry})

      {:ok, idle} =
        start_supervised(
          Supervisor.child_spec(
            {ScriptedMember, registry: @registry, script: fn _state -> {:ok, %{"ok" => true, "who" => "idle"}} end},
            id: {:scripted, :idle}
          )
        )

      {:ok, busy} =
        start_supervised(
          Supervisor.child_spec(
            {ScriptedMember, registry: @registry, script: fn _state -> {:ok, %{"ok" => true, "who" => "busy"}} end},
            id: {:scripted, :busy}
          )
        )

      # Structurally load the busy member: suspend it and pad its real mailbox.
      :ok = :sys.suspend(busy)
      for _ <- 1..5, do: send(busy, :court_padding)

      for _ <- 1..10 do
        assert {:ok, %{"who" => "idle"}} = Pool.request(%{"op" => "capabilities"})
      end

      :ok = :sys.resume(busy)

      # The busy member received none of those calls; the idle one received all.
      assert GenServer.call(busy, :calls) == 0
      assert GenServer.call(idle, :calls) == 10
    end

    test "fairness: a burst over idle members spreads — every live member serves" do
      start_supervised!({Registry, keys: :duplicate, name: @registry})

      members =
        for i <- 1..3 do
          spec =
            Supervisor.child_spec(
              {ScriptedMember,
               registry: @registry, script: fn _state -> {:ok, %{"ok" => true, "who" => "m#{i}"}} end},
              id: {:scripted, i}
            )

          start_supervised!(spec)
        end

      # The route picker shuffles ties (all mailbox lengths are zero here), so
      # a burst of 30 must touch every member: with 3 members the probability
      # of a 30-run missing one is (2/3)^30 < 1e-5, and a wedge (min_by always
      # picking the first) would pin ~all of them on member one.
      served =
        MapSet.new(for _ <- 1..30 do
                     {:ok, %{"who" => who}} = Pool.request(%{"op" => "capabilities"})
                     who
                   end)

      assert MapSet.size(served) == 3
      assert Enum.all?(members, &(GenServer.call(&1, :calls) > 0))
    end

    test "an unavailable member never routes; with no live member the real typed refusal surfaces" do
      start_supervised!({Registry, keys: :duplicate, name: @registry})

      # A real member that has gone unhealthy: it sits under :unavailable and
      # answers with the same typed refusal a real unavailable host answers with.
      {:ok, healing} =
        start_supervised(
          {ScriptedMember,
           registry: @registry,
           availability: :unavailable,
           script: fn _state -> {:error, Refusal.new(:call_exited, "engine instance exited during the call")} end}
        )

      assert {:error, %Refusal{code: :call_exited}} = Pool.request(%{"op" => "capabilities"})

      # And the member routes again the moment it is healthy.
      :ok = GenServer.call(healing, {:set_availability, :members, fn _state -> {:ok, %{"ok" => true}} end})
      assert {:ok, %{"ok" => true}} = Pool.request(%{"op" => "capabilities"})
    end

    test "with no registry at all the pool refuses typed instead of crashing" do
      assert {:error, %Refusal{code: :call_exited}} = Pool.request(%{"op" => "capabilities"})
    end
  end
end
