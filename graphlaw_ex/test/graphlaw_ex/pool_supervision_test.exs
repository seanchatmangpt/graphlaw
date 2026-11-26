# SPDX-FileCopyrightText: 2026 Sean Chatman <https://github.com/seanchatmangpt/graphlaw/graphs/contributors>
#
# SPDX-License-Identifier: MIT
defmodule GraphlawEx.PoolSupervisionTest do
  # Real pool, real engine. Shares the globally named Registry/Leases/Host, so
  # it cannot run concurrently with other pool tests. :wasm: excluded loudly when
  # the artifact is absent (test_helper.exs), failing under GRAPHLAW_REQUIRE_ENGINE=1.
  use ExUnit.Case, async: false

  alias GraphlawEx.Pool

  @deadline_ms 10_000
  @moduletag :wasm
  @moduletag timeout: 300_000

  setup do
    {:ok, wasm_path: GraphlawEx.TestEngine.engine_path!()}
  end

  test "killing one Host leaves its siblings untouched", %{wasm_path: path} do
    sup =
      start_supervised!(
        {Pool,
         name: :pool_supervision_test_pool, size: 3, wasm_path: path, expected_sha256: :unpinned}
      )

    assert [_, _, _] = hosts = await_hosts(sup, 3)
    [victim | siblings] = hosts

    ref = Process.monitor(victim)
    Process.exit(victim, :kill)
    assert_receive {:DOWN, ^ref, :process, ^victim, :killed}, @deadline_ms

    replaced =
      poll(fn ->
        now = hosts_of(sup)
        length(now) == 3 and victim not in now and now
      end)

    assert Enum.all?(siblings, &(&1 in replaced)), "healthy siblings were restarted"
    assert Enum.all?(siblings, &Process.alive?/1)
  end

  defp workers(sup) do
    {_, pid, _, _} =
      sup |> Supervisor.which_children() |> List.keyfind(Pool.Workers, 0)

    pid
  end

  defp hosts_of(sup) do
    sup
    |> workers()
    |> Supervisor.which_children()
    |> Enum.map(fn {_id, pid, _type, _mods} -> pid end)
    |> Enum.filter(&is_pid/1)
  end

  defp await_hosts(sup, n), do: poll(fn -> (h = hosts_of(sup)) |> length() == n and h end)

  defp poll(fun) do
    deadline = System.monotonic_time(:millisecond) + @deadline_ms
    do_poll(fun, deadline)
  end

  defp do_poll(fun, deadline) do
    case fun.() do
      result when result not in [false, nil] ->
        result

      _ ->
        if System.monotonic_time(:millisecond) > deadline do
          flunk("condition not met within #{@deadline_ms}ms")
        else
          :erlang.yield()
          do_poll(fun, deadline)
        end
    end
  end
end
