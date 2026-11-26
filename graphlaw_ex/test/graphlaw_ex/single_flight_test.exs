# SPDX-FileCopyrightText: 2026 Sean Chatman <https://github.com/seanchatmangpt/graphlaw/graphs/contributors>
#
# SPDX-License-Identifier: MIT
defmodule GraphlawEx.EngineLoad.SingleFlightTest do
  # purge/0 erases every published flight value process-wide; other modules
  # are unaffected but this module's own tests serialize against it by using
  # unique keys and asserting on results, not on cache residency.
  use ExUnit.Case, async: true

  alias GraphlawEx.EngineLoad.SingleFlight
  alias GraphlawEx.Refusal

  @n 8
  @budget_ms 5_000

  defp key, do: {:sf_test, make_ref()}

  defp poll(fun, deadline \\ System.monotonic_time(:millisecond) + @budget_ms) do
    cond do
      fun.() -> :ok
      System.monotonic_time(:millisecond) > deadline -> flunk("poll budget exceeded")
      true -> poll(fun, deadline)
    end
  end

  defp waiting(pid) do
    case Process.info(pid, :messages) do
      {:messages, msgs} -> Enum.count(msgs, &match?({SingleFlight, :wait, _, _}, &1))
      nil -> 0
    end
  end

  test "N concurrent callers on one key run fun once and all get the value" do
    k = key()
    counter = :counters.new(1, [])
    test = self()

    fun = fn ->
      :counters.add(counter, 1, 1)
      send(test, {:leader, self()})

      receive do
        :go -> {:ok, :value}
      end
    end

    leader = Task.async(fn -> SingleFlight.fetch_or_compile(k, fun) end)
    leader_pid = receive do: ({:leader, pid} -> pid)

    waiters =
      for _ <- 1..(@n - 1) do
        Task.async(fn -> SingleFlight.fetch_or_compile(k, fun) end)
      end

    poll(fn -> waiting(leader_pid) == @n - 1 end)
    send(leader_pid, :go)

    results = Enum.map([leader | waiters], &Task.await(&1, @budget_ms))
    assert results == List.duplicate({:ok, :value}, @n)
    assert :counters.get(counter, 1) == 1
    assert SingleFlight.cached(k) == {:ok, :value}
  end

  test "leader crash releases waiters with a typed call_exited refusal" do
    k = key()
    test = self()

    leader_pid =
      spawn(fn ->
        SingleFlight.fetch_or_compile(k, fn ->
          send(test, :leading)

          receive do
            :never -> {:ok, :unreachable}
          end
        end)
      end)

    assert_receive :leading, @budget_ms

    waiters =
      for _ <- 1..@n do
        Task.async(fn -> SingleFlight.fetch_or_compile(k, fn -> {:ok, :rerun} end) end)
      end

    poll(fn -> waiting(leader_pid) == @n end)
    Process.exit(leader_pid, :kill)

    # Disjunctive law: a waiter whose post-send re-check observes the departed leader
    # retries as the new leader (`{:ok, :rerun}`); a waiter still waiting on its monitor is
    # released with the typed refusal. Which one happens depends on scheduling.
    outcomes = for t <- waiters, do: Task.await(t, @budget_ms)

    assert Enum.all?(outcomes, fn
             {:error, %Refusal{code: :call_exited}} -> true
             {:ok, :rerun} -> true
             _ -> false
           end)

    case SingleFlight.cached(k) do
      :error -> assert Enum.all?(outcomes, &match?({:error, %Refusal{code: :call_exited}}, &1))
      {:ok, :rerun} -> assert {:ok, :rerun} in outcomes
    end
  end

  test "failure is not cached; the next call reruns" do
    k = key()
    counter = :counters.new(1, [])

    fun = fn ->
      :counters.add(counter, 1, 1)
      {:error, :boom}
    end

    assert SingleFlight.fetch_or_compile(k, fun) == {:error, :boom}
    assert SingleFlight.fetch_or_compile(k, fun) == {:error, :boom}
    assert :counters.get(counter, 1) == 2
    assert SingleFlight.cached(k) == :error
  end

  test "purge/0 clears published values" do
    k = key()
    assert {:ok, 1} = SingleFlight.fetch_or_compile(k, fn -> {:ok, 1} end)
    assert SingleFlight.cached(k) == {:ok, 1}
    assert SingleFlight.purge() == :ok
    assert SingleFlight.cached(k) == :error
  end
end
