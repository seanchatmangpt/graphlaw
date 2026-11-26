# SPDX-FileCopyrightText: 2026 Sean Chatman <https://github.com/seanchatmangpt/graphlaw/graphs/contributors>
#
# SPDX-License-Identifier: MIT
defmodule GraphlawEx.Safety.PoolLeaseLeakTest do
  @moduledoc """
  Lane S1-kernel. Real `GraphlawEx.Pool` over the real engine: callers killed
  in every phase of a lease (before the lease cast, mid-call) leak no slot, and
  a saturated pool refuses typed `:REFUSED_RESOURCE_BOUND`.

  Gate: a missing wasm artifact flunks under `GRAPHLAW_REQUIRE_ENGINE=1` and is
  a loud skip otherwise.
  """
  use ExUnit.Case, async: false

  alias GraphlawEx.Pool

  @wasm_path GraphlawEx.TestEngine.wasm_path()
  @require_engine System.get_env("GRAPHLAW_REQUIRE_ENGINE") == "1"
  @moduletag timeout: 300_000
  @moduletag :wasm

  test "callers killed at arbitrary phases leak no slot" do
    with_pool(fn ->
      capacity = Pool.size()

      callers =
        for i <- 1..(capacity * 8) do
          spawn_monitor(fn -> loop_requests(i) end)
        end

      # Kill each caller at a different phase: immediately (before/at the lease
      # cast) or after it has had a chance to enter the call.
      callers
      |> Enum.with_index()
      |> Enum.each(fn {{pid, _ref}, i} ->
        if rem(i, 2) == 1, do: :erlang.yield()
        Process.exit(pid, :kill)
      end)

      for {pid, ref} <- callers do
        assert_receive {:DOWN, ^ref, :process, ^pid, _}, 10_000
      end

      assert poll(fn -> Pool.available() == capacity end),
             "leaked slot: available=#{inspect(Pool.available())} capacity=#{capacity}"
    end)
  end

  test "a caller killed mid-call (member suspended) leaks no slot; saturation refuses typed" do
    with_pool(fn ->
      capacity = Pool.size()
      members = Pool.members()
      assert members != []
      for m <- members, do: :ok = :sys.suspend(m)

      try do
        callers =
          for _ <- 1..capacity do
            spawn_monitor(fn -> Pool.request(%{"op" => "capabilities"}) end)
          end

        assert poll(fn -> Pool.available() == 0 end), "slots never became leased"

        assert {:error, %GraphlawEx.Refusal{code: :saturated}} =
                 Pool.request(%{"op" => "capabilities"})

        assert Pool.available() == 0

        for {pid, _} <- callers, do: Process.exit(pid, :kill)

        for {pid, ref} <- callers do
          assert_receive {:DOWN, ^ref, :process, ^pid, :killed}, 10_000
        end

        assert poll(fn -> Pool.available() == capacity end),
               "leaked slot after mid-call kill: available=#{inspect(Pool.available())}"
      after
        for m <- members, Process.alive?(m), do: :sys.resume(m)
      end

      # Pool still serves once members resume.
      assert poll(fn ->
               match?({:ok, %{"ok" => true}}, Pool.request(%{"op" => "capabilities"}))
             end)

      assert poll(fn -> Pool.available() == capacity end)
    end)
  end

  defp loop_requests(_i) do
    _ = Pool.request(%{"op" => "capabilities"})
    loop_requests(0)
  end

  defp with_pool(fun) do
    cond do
      not File.regular?(@wasm_path) ->
        message = "[SKIP] pool lease leak courts: wasm artifact not found at #{@wasm_path}."
        if @require_engine, do: flunk(message)
        IO.puts(message)

      Process.whereis(Pool) != nil ->
        fun.()

      true ->
        start_supervised!({Pool, size: 2, wasm_path: @wasm_path, expected_sha256: :unpinned})
        fun.()
    end
  end

  defp poll(fun, deadline_ms \\ 15_000) do
    do_poll(fun, System.monotonic_time(:millisecond) + deadline_ms)
  end

  defp do_poll(fun, deadline) do
    cond do
      fun.() ->
        true

      System.monotonic_time(:millisecond) >= deadline ->
        false

      true ->
        receive do
        after
          5 -> do_poll(fun, deadline)
        end
    end
  end
end
