# SPDX-FileCopyrightText: 2026 Sean Chatman <https://github.com/seanchatmangpt/graphlaw/graphs/contributors>
#
# SPDX-License-Identifier: MIT
defmodule GraphlawEx.PoolIsolationTest do
  use ExUnit.Case, async: false

  # Gate 3: a 64-way storm mixing killer queries (timeout, OOM, malformed op),
  # callers killed mid-call, and healthy queries against a REAL GraphlawEx.Pool
  # over the real wasm engine. Final-state assertions only: zero healthy
  # failures, every member available again, no orphan Wasmex instance.
  #
  # Absent engine artifact: :wasm tests are excluded loudly (test_helper.exs);
  # GRAPHLAW_REQUIRE_ENGINE=1 makes them fail instead.

  alias GraphlawEx.Host
  alias GraphlawEx.Pool

  @pool_size 4
  @storm 64
  @deadline_ms 60_000

  @healthy %{
    "op" => "law",
    "data" => %{
      "text" => "@prefix : <https://test.graphlaw.dev/> .\n:a :edge :b .\n:b :edge :c .\n",
      "dialect" => "turtle"
    },
    "steps" => [%{"step" => "rdfs"}]
  }

  # Every test here needs the real engine: :wasm tests are excluded loudly by
  # test_helper.exs when the artifact is absent, and fail (engine_path!/0
  # raises) under GRAPHLAW_REQUIRE_ENGINE=1.
  @moduletag :wasm
  @moduletag timeout: 600_000

  @wasm_path GraphlawEx.TestEngine.wasm_path()

  setup_all do
    _ = GraphlawEx.TestEngine.engine_path!()
    GraphlawEx.EngineLoad.purge_cache()
    :ok
  end

  test "64-way storm of killers, healthy queries and killed callers leaves the pool whole" do
    run_storm()
  end

  defp run_storm do
    start_supervised!(
      {Pool,
       size: @pool_size, wasm_path: @wasm_path, expected_sha256: :unpinned, retry_base_ms: 50}
    )

    assert wait_until(&pool_whole?/0), "pool never became whole before the storm"
    baseline = wasmex_count()
    assert baseline >= @pool_size

    # One oversize body, built once. At 272 MiB it exceeds the host's 16 MiB
    # request cap (`max_request_bytes`), so this killer is refused
    # :resource_limit BEFORE any gl_alloc; it does NOT reach the engine's
    # 256 MiB linear-memory limit. The memory-limit trap is courted by
    # GraphlawEx.PoolFaultTest ("a memory-limit abort ...").
    oom_text = :binary.copy("a", 272 * 1024 * 1024)
    oom = %{"op" => "law", "data" => %{"text" => oom_text, "dialect" => "turtle"}, "steps" => []}

    heavy_text =
      Enum.map_join(1..3000, "", fn i -> ":n#{i} :edge :n#{i + 1} .\n" end)

    heavy = %{
      "op" => "law",
      "data" => %{
        "text" => "@prefix : <https://test.graphlaw.dev/> .\n" <> heavy_text,
        "dialect" => "turtle"
      },
      "steps" => [%{"step" => "rdfs"}]
    }

    parent = self()

    jobs =
      for i <- 0..(@storm - 1) do
        kind =
          cond do
            i == 1 -> :oom
            rem(i, 8) == 0 -> :timeout
            rem(i, 8) == 2 -> :malformed
            rem(i, 8) == 3 -> :caller_killed
            true -> :healthy
          end

        {kind, spawn_job(parent, kind, i, %{healthy: @healthy, heavy: heavy, oom: oom})}
      end

    # Kill the doomed callers while their request is (or may be) in flight.
    for {:caller_killed, {pid, _ref, _mref}} <- jobs, do: Process.exit(pid, :kill)

    healthy =
      for {:healthy, {_pid, ref, _mref}} <- jobs do
        receive do
          {^ref, result} -> result
        after
          @deadline_ms -> :no_answer
        end
      end

    # Killers must also terminate with a typed answer (never hang the caller).
    for {kind, {_pid, ref, _mref}} <- jobs, kind in [:oom, :timeout, :malformed] do
      assert_receive {^ref, answer}, @deadline_ms

      case kind do
        :malformed ->
          # An unknown op is an engine-level answer, not a transport refusal.
          # (or, under the storm, a typed saturation refusal before it reaches the engine)
          assert match?({:ok, %{"ok" => false, "error" => %{"kind" => "Unsupported"}}}, answer) or
                   match?({:error, %GraphlawEx.Refusal{code: :saturated}}, answer)

        :timeout ->
          # The 1 ms deadline fires: :call_timeout, or (only under the storm) a
          # typed pool saturation before it reaches an engine.
          assert match?({:error, %GraphlawEx.Refusal{code: :call_timeout}}, answer) or
                   match?({:error, %GraphlawEx.Refusal{code: :saturated}}, answer),
                 "timeout killer answered #{inspect(answer, limit: 5)}"

        :oom ->
          # Over the request cap: refused before the engine, or saturated.
          assert match?({:error, %GraphlawEx.Refusal{code: :resource_limit}}, answer) or
                   match?({:error, %GraphlawEx.Refusal{code: :saturated}}, answer),
                 "oom killer answered #{inspect(answer, limit: 5)}"
      end
    end

    healthy_failures = Enum.reject(healthy, &match?({:ok, %{"ok" => true}}, &1))

    assert healthy_failures == [],
           "healthy queries failed: #{inspect(healthy_failures, limit: 5)}"

    # Every caller process terminates (monitor-observed, no liveness polling).
    for {_kind, {pid, _ref, mref}} <- jobs do
      assert_receive {:DOWN, ^mref, :process, ^pid, _reason}, @deadline_ms
    end

    # Final state: slot-leak free, every member back in :members, no leaked instance.
    assert wait_until(&pool_whole?/0), "pool did not return to available == pool_size"
    assert Pool.available() == @pool_size
    assert length(Pool.members()) == @pool_size

    assert wait_until(fn -> wasmex_count() == baseline end),
           "orphan Wasmex instances: #{wasmex_count()} live, expected #{baseline}"

    # And it still serves.
    assert {:ok, %{"ok" => true}} = Pool.request(@healthy)
  end

  test "killed workers are restarted by supervision, leak no slot and no engine" do
    start_supervised!(
      {Pool,
       size: @pool_size, wasm_path: @wasm_path, expected_sha256: :unpinned, retry_base_ms: 50}
    )

    assert wait_until(&pool_whole?/0)
    baseline = wasmex_count()
    old = Pool.members()
    mons = for pid <- old, do: {pid, Process.monitor(pid)}

    callers =
      for _ <- 1..16 do
        spawn_monitor(fn -> Pool.request(@healthy) end)
      end

    for pid <- old, do: Process.exit(pid, :kill)

    for {pid, mref} <- mons do
      assert_receive {:DOWN, ^mref, :process, ^pid, :killed}, @deadline_ms
    end

    for {pid, mref} <- callers do
      assert_receive {:DOWN, ^mref, :process, ^pid, _reason}, @deadline_ms
    end

    assert wait_until(&pool_whole?/0), "pool did not heal after killing every worker"
    assert Pool.available() == @pool_size
    assert Enum.all?(Pool.members(), &(&1 not in old))

    assert wait_until(fn -> wasmex_count() == baseline end),
           "orphan Wasmex instances after worker kill: #{wasmex_count()}, expected #{baseline}"

    assert {:ok, %{"ok" => true}} = Pool.request(@healthy)
  end

  test "tiny fuel budget really exhausts fuel: typed refusal, recycle, pool whole" do
    # The pool name is global, so the small-fuel pool is THE pool for this test.
    start_supervised!(
      {Pool,
       size: 2,
       wasm_path: @wasm_path,
       expected_sha256: :unpinned,
       retry_base_ms: 50,
       fuel: 10_000,
       timeout_ms: 30_000}
    )

    assert wait_until(fn ->
             length(Pool.members()) == 2 and Pool.available() == 2 and
               Enum.all?(Pool.members(), &Host.available?/1)
           end),
           "small-fuel pool never became whole"

    old = Pool.members()

    heavy_text = Enum.map_join(1..3000, "", fn i -> ":n#{i} :edge :n#{i + 1} .\n" end)

    heavy = %{
      "op" => "law",
      "data" => %{
        "text" => "@prefix : <https://test.graphlaw.dev/> .\n" <> heavy_text,
        "dialect" => "turtle"
      },
      "steps" => [%{"step" => "rdfs"}]
    }

    answers =
      for _ <- 1..8 do
        {pid, mref} = spawn_monitor(fn -> exit({:answer, Pool.request(heavy)}) end)

        receive do
          {:DOWN, ^mref, :process, ^pid, {:answer, answer}} -> answer
        after
          @deadline_ms -> :no_answer
        end
      end

    for answer <- answers do
      assert {:error, %GraphlawEx.Refusal{code: :fuel_exhausted}} = answer
    end

    # Recycled in place: every member heals, no slot leaked, fresh instances
    # serve again, and the recycle counters moved.
    assert wait_until(fn ->
             length(Pool.members()) == 2 and Pool.available() == 2 and
               Enum.all?(Pool.members(), &Host.available?/1)
           end),
           "pool did not heal after fuel exhaustion"

    assert Pool.available() == 2
    assert Enum.sort(Pool.members()) == Enum.sort(old)

    recycles =
      for pid <- old do
        {:ok, %{recycles: n}} = Host.info(pid)
        n
      end

    assert Enum.sum(recycles) >= 8,
           "expected a recycle per exhausted call, got #{inspect(recycles)}"
  end

  defp spawn_job(parent, kind, _i, reqs) do
    ref = make_ref()

    {pid, mref} =
      spawn_monitor(fn ->
        result =
          case kind do
            :healthy -> healthy_request(reqs.healthy)
            :timeout -> Pool.request(reqs.heavy, timeout_ms: 1)
            :oom -> Pool.request(reqs.oom)
            :malformed -> Pool.request(%{"op" => "definitely-not-an-op"})
            :caller_killed -> Pool.request(reqs.heavy)
          end

        send(parent, {ref, result})
      end)

    {pid, ref, mref}
  end

  defp pool_whole? do
    members = Pool.members()

    length(members) == @pool_size and Pool.available() == @pool_size and
      Enum.all?(members, &Host.available?/1)
  end

  defp wasmex_count do
    Enum.count(Process.list(), fn pid ->
      case Process.info(pid, :dictionary) do
        {:dictionary, dict} ->
          match?({Wasmex, _, _}, Keyword.get(dict, :"$initial_call"))

        _ ->
          false
      end
    end)
  end

  # The single audited bounded poll: observable + Instant-style monotonic deadline.
  defp wait_until(fun, deadline_ms \\ @deadline_ms) do
    deadline = System.monotonic_time(:millisecond) + deadline_ms
    poll(fun, deadline)
  end

  defp poll(fun, deadline) do
    cond do
      fun.() ->
        true

      System.monotonic_time(:millisecond) >= deadline ->
        false

      true ->
        receive do
        after
          25 -> poll(fun, deadline)
        end
    end
  end

  # A healthy caller whose request meets a saturated pool is refused typed
  # (:REFUSED_RESOURCE_BOUND); it retries inside a monotonic deadline, which is
  # the contract a caller of a bounded pool lives under. Any other answer is final.
  defp healthy_request(req) do
    deadline = System.monotonic_time(:millisecond) + @deadline_ms
    healthy_request(req, deadline)
  end

  defp healthy_request(req, deadline) do
    case Pool.request(req) do
      {:error, %GraphlawEx.Refusal{code: :saturated}} = refused ->
        if System.monotonic_time(:millisecond) < deadline do
          :erlang.yield()
          healthy_request(req, deadline)
        else
          refused
        end

      other ->
        other
    end
  end
end
