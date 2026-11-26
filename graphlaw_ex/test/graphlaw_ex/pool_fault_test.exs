# SPDX-FileCopyrightText: 2026 Sean Chatman <https://github.com/seanchatmangpt/graphlaw/graphs/contributors>
#
# SPDX-License-Identifier: MIT
defmodule GraphlawEx.PoolFaultTest do
  # Fault courts for the real GraphlawEx.Pool / Host supervision tree.
  #
  # Collaborators are REAL: the real Pool supervisor, Registry, Leases, Workers
  # and Hosts, over the real Wasmtime, running GraphlawEx.FakeEngine - a
  # hand-written implementation of the engine ABI (not a mock) that can hang,
  # trap, answer malformed JSON, or abort on a refused memory growth on demand.
  # No artifact is needed, so none of these courts can be skipped by a missing
  # engine. Assertions are on final state (pids, availability, recycle counters,
  # typed refusal codes); waits are bounded polls on observables.
  #
  # The pool's names (Registry, Leases, Host) are global: async: false.
  use ExUnit.Case, async: false

  alias GraphlawEx.FakeEngine
  alias GraphlawEx.Host
  alias GraphlawEx.Pool
  alias GraphlawEx.Refusal

  @deadline_ms 30_000

  defp start_pool!(size, kind, extra \\ []) do
    opts =
      Keyword.merge(
        [
          size: size,
          bytes: FakeEngine.bytes(kind),
          expected_sha256: :unpinned,
          retry_base_ms: 20,
          retry_max_ms: 100
        ],
        extra
      )

    start_supervised!({Pool, opts})
  end

  describe "supervision faults" do
    test "killing Leases rebuilds the gate: available == capacity and members re-register" do
      pool = start_pool!(3, :healthy)
      assert poll(fn -> whole?(3) end), "pool never became whole"

      old_leases = Process.whereis(Pool.Leases)
      old_hosts = Pool.members()
      ref = Process.monitor(old_leases)
      Process.exit(old_leases, :kill)
      assert_receive {:DOWN, ^ref, :process, ^old_leases, :killed}, 5_000

      assert poll(fn ->
               new = Process.whereis(Pool.Leases)

               is_pid(new) and new != old_leases and whole?(3) and
                 Enum.all?(Pool.members(), &(&1 not in old_hosts))
             end),
             "pool did not recover after Leases was killed"

      assert Pool.available() == 3
      assert Process.alive?(pool)
      assert {:ok, %{"ok" => true}} = Pool.request(FakeEngine.good_request())
      assert poll(fn -> Pool.available() == 3 end)
    end

    test "killing Workers re-registers fresh members and leaves Registry and Leases alone" do
      pool = start_pool!(3, :healthy)
      assert poll(fn -> whole?(3) end), "pool never became whole"

      leases = Process.whereis(Pool.Leases)
      old_workers = workers(pool)
      old_hosts = Pool.members()
      ref = Process.monitor(old_workers)
      Process.exit(old_workers, :kill)
      assert_receive {:DOWN, ^ref, :process, ^old_workers, :killed}, 5_000

      assert poll(fn ->
               new = workers(pool)

               is_pid(new) and new != old_workers and whole?(3) and
                 Enum.all?(Pool.members(), &(&1 not in old_hosts))
             end),
             "members did not re-register after Workers was killed"

      assert Process.whereis(Pool.Leases) == leases
      assert Pool.available() == 3
      assert {:ok, %{"ok" => true}} = Pool.request(FakeEngine.good_request())
    end
  end

  describe "engine faults" do
    test "a hung call is typed :call_timeout, the instance is replaced, and the host serves again" do
      # Fuel is set far above anything a 200 ms spin burns, so the DEADLINE (not
      # fuel exhaustion) is what ends the call.
      start_pool!(1, :hang, fuel: 1_000_000_000_000_000)
      assert poll(fn -> whole?(1) end)
      [host] = Pool.members()

      assert {:error, %Refusal{code: :call_timeout}} =
               Pool.request(FakeEngine.bad_request(), timeout_ms: 200)

      assert poll(fn -> Pool.available() == 1 and Host.available?(host) end),
             "host did not become healthy after the hang"

      assert {:ok, %{recycles: recycles}} = Host.info(host)
      assert recycles >= 1
      assert {:ok, %{"ok" => true}} = Pool.request(FakeEngine.good_request())
      assert Pool.members() == [host]
    end

    test "a memory-limit abort is a typed trap, the instance is replaced, and the host serves again" do
      start_pool!(1, :grow, memory_limit_bytes: 16 * 1024 * 1024)
      assert poll(fn -> whole?(1) end)
      [host] = Pool.members()

      assert {:error, %Refusal{code: :call_trapped}} = Pool.request(FakeEngine.bad_request())

      assert poll(fn -> Pool.available() == 1 and Host.available?(host) end)
      assert {:ok, %{recycles: 1}} = Host.info(host)
      assert {:ok, %{"ok" => true}} = Pool.request(FakeEngine.good_request())
    end

    test "a malformed response recycles the instance every time and the host keeps serving" do
      start_pool!(1, :malformed)
      assert poll(fn -> whole?(1) end)
      [host] = Pool.members()

      for n <- 1..3 do
        assert {:error, %Refusal{code: :malformed_response}} =
                 Pool.request(FakeEngine.bad_request())

        # Host.info is serialized behind the post-reply recycle, so the counter
        # is settled when it answers.
        assert {:ok, %{recycles: ^n}} = Host.info(host)
      end

      assert {:ok, %{"ok" => true}} = Pool.request(FakeEngine.good_request())
      assert {:ok, %{recycles: 3}} = Host.info(host)
      assert Pool.members() == [host]
    end

    test "20 consecutive traps are bounded: no host crash, no supervisor restart, 20 recycles" do
      pool = start_pool!(1, :trap)
      assert poll(fn -> whole?(1) end)
      [host] = Pool.members()
      workers = workers(pool)
      leases = Process.whereis(Pool.Leases)

      for _ <- 1..20 do
        assert {:error, %Refusal{code: :call_trapped}} = Pool.request(FakeEngine.bad_request())
      end

      assert {:ok, %{recycles: 20}} = Host.info(host)
      assert {:ok, %{"ok" => true}} = Pool.request(FakeEngine.good_request())

      # Same processes all the way through: traps were absorbed by recycling,
      # never by a crash-restart loop.
      assert Pool.members() == [host]
      assert workers(pool) == workers
      assert Process.whereis(Pool.Leases) == leases
      assert Process.alive?(host)
      assert poll(fn -> Pool.available() == 1 end)
    end
  end

  describe "an unloadable engine" do
    test "a corrupt artifact leaves every member unavailable and typed, without a crash loop" do
      pool = start_pool!(2, :healthy, bytes: <<1, 2, 3>>)

      assert Pool.members() == []
      assert [_, _] = hosts = Enum.sort(Pool.unavailable_members())
      workers = workers(pool)

      for _ <- 1..5 do
        assert {:error, %Refusal{code: :invalid_json}} = Pool.request(FakeEngine.good_request())
      end

      assert Enum.sort(Pool.unavailable_members()) == hosts
      assert Enum.all?(hosts, &Process.alive?/1)
      assert workers(pool) == workers
      assert Process.alive?(pool)
      assert poll(fn -> Pool.available() == 2 end)
    end

    test "a pin that does not match refuses every call :wasm_digest_mismatch, without a crash loop" do
      pool = start_pool!(2, :healthy, expected_sha256: String.duplicate("ab", 32))

      assert Pool.members() == []
      assert [_, _] = hosts = Enum.sort(Pool.unavailable_members())

      for _ <- 1..5 do
        assert {:error, %Refusal{code: :wasm_digest_mismatch}} =
                 Pool.request(FakeEngine.good_request())
      end

      assert Enum.sort(Pool.unavailable_members()) == hosts
      assert Enum.all?(hosts, &Process.alive?/1)
      assert Process.alive?(pool)
    end

    @tag :tmp_dir
    test "members retry the load and heal once a valid artifact appears at the path", %{
      tmp_dir: tmp
    } do
      path = Path.join(tmp, "engine.wasm")
      File.write!(path, <<1, 2, 3>>)

      pool =
        start_supervised!(
          {Pool,
           size: 2,
           wasm_path: path,
           expected_sha256: :unpinned,
           retry_base_ms: 10,
           retry_max_ms: 50}
        )

      assert Pool.members() == []
      hosts = Enum.sort(Pool.unavailable_members())
      assert length(hosts) == 2

      File.write!(path, FakeEngine.bytes(:healthy))

      assert poll(fn -> whole?(2) end), "members never healed after the artifact was fixed"
      assert Pool.unavailable_members() == []

      # Healed in place: the same host processes, no restarts.
      assert Enum.sort(Pool.members()) == hosts
      assert Process.alive?(pool)
      assert {:ok, %{"ok" => true}} = Pool.request(FakeEngine.good_request())
    end
  end

  # ---------------------------------------------------------------------

  defp whole?(n) do
    members = Pool.members()

    length(members) == n and Enum.all?(members, &Process.alive?/1) and
      Pool.available() == n and Enum.all?(members, &Host.available?/1)
  end

  defp workers(pool) do
    {_id, pid, _type, _mods} =
      pool |> Supervisor.which_children() |> List.keyfind!(Pool.Workers, 0)

    pid
  end

  # The audited bounded poll: observable + monotonic deadline, false on expiry.
  defp poll(fun, deadline_ms \\ @deadline_ms) do
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
