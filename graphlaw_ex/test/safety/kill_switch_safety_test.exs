# SPDX-FileCopyrightText: 2026 Sean Chatman <https://github.com/seanchatmangpt/graphlaw/graphs/contributors>
#
# SPDX-License-Identifier: MIT
defmodule GraphlawEx.KillSwitchSafetyTest do
  @moduledoc """
  Lane S7 safety courts for the kill switch, real Host + real engine.

  Covers: refusal is typed and fast, in-flight and concurrent requests are not
  corrupted, reset is explicit, the tripped state survives a worker recycle, and
  hostile install input refuses instead of crashing. Under
  `GRAPHLAW_REQUIRE_ENGINE=1` a missing artifact FAILS (engine_path!/0 raises);
  otherwise the :wasm cases are excluded loudly by test_helper.exs.
  """

  use ExUnit.Case, async: false

  alias GraphlawEx.Host
  alias GraphlawEx.KillSwitch
  alias GraphlawEx.Refusal

  @req %{"op" => "capabilities"}

  setup do
    :ok = KillSwitch.reset()
    on_exit(fn -> KillSwitch.reset() end)
    :ok
  end

  describe "install input hardening (engine-free)" do
    test "non-canonicalizable and non-map input refuse with a typed code" do
      {pub, seed} = keypair()
      keys = %{"ops" => pub}
      doc = signed(3, "all", "ops", seed)

      bad = put_in(doc, ["revocation", "reason"], {:not, :json})
      assert {:error, %Refusal{code: :lease_revoked}} = KillSwitch.install(bad, keys)

      assert {:error, %Refusal{code: :lease_revoked}} = KillSwitch.install(:nope, keys)
      assert {:error, %Refusal{code: :lease_revoked}} = KillSwitch.install(doc, [])
      assert KillSwitch.current_epoch() == 0
    end

    test "a trusted key of the wrong shape refuses instead of crashing" do
      {_pub, seed} = keypair()
      doc = signed(3, "all", "ops", seed)

      for junk <- ["zz", "abcd", 42, nil] do
        assert {:error, %Refusal{code: :lease_revoked}} =
                 KillSwitch.install(doc, %{"ops" => junk})
      end

      assert KillSwitch.current_epoch() == 0
    end

    test "reset is explicit: trip persists until reset/0, then the epoch is 0" do
      {pub, seed} = keypair()
      assert {:ok, _} = KillSwitch.install(signed(5, "all", "ops", seed), %{"ops" => pub})
      assert {:error, %Refusal{code: :lease_revoked}} = KillSwitch.check("any")
      assert KillSwitch.current_epoch() == 5

      assert :ok = KillSwitch.reset()
      assert KillSwitch.current_epoch() == 0
      assert KillSwitch.check("any") == :ok
    end
  end

  describe "real engine" do
    @describetag :wasm
    @describetag timeout: 300_000

    test "tripped switch refuses fast with a typed code; reset re-admits" do
      with_engine(fn path ->
        host = start_host!(path)
        {pub, seed} = keypair()

        assert {:ok, %{"ok" => true}} = Host.request(host, @req)
        assert {:ok, _} = KillSwitch.install(signed(7, "all", "ops", seed), %{"ops" => pub})

        {micros, result} = :timer.tc(fn -> Host.request(host, @req) end)

        assert {:error, %Refusal{code: :lease_revoked, class: :refused_identity}} = result
        assert micros < 2_000_000

        :ok = KillSwitch.reset()
        assert {:ok, %{"ok" => true}} = Host.request(host, @req)
      end)
    end

    test "concurrent requests around a trip are each answered ok or refused, never corrupted" do
      with_engine(fn path ->
        host = start_host!(path)
        {pub, seed} = keypair()

        tasks = for _ <- 1..12, do: Task.async(fn -> Host.request(host, @req) end)
        assert {:ok, _} = KillSwitch.install(signed(7, "all", "ops", seed), %{"ops" => pub})

        for result <- Task.await_many(tasks, 60_000) do
          case result do
            {:ok, %{"ok" => true}} -> :ok
            {:error, %Refusal{code: :lease_revoked}} -> :ok
            other -> flunk("unexpected result under trip: #{inspect(other)}")
          end
        end

        :ok = KillSwitch.reset()
        assert {:ok, %{"ok" => true}} = Host.request(host, @req)
      end)
    end

    test "tripped state survives a worker recycle and stays tripped for the new instance" do
      with_engine(fn path ->
        host = start_host!(path, id: :s7_a)

        assert {:ok, %{"ok" => true}} = Host.request(host, @req)

        {pub, seed} = keypair()
        assert {:ok, _} = KillSwitch.install(signed(9, "all", "ops", seed), %{"ops" => pub})

        # Worker replaced: the old host process is stopped, a fresh one boots.
        :ok = stop_supervised(:s7_a)
        host2 = start_host!(path, id: :s7_b)

        assert KillSwitch.current_epoch() == 9
        assert {:error, %Refusal{code: :lease_revoked}} = Host.request(host2, @req)

        :ok = KillSwitch.reset()
        assert {:ok, %{"ok" => true}} = Host.request(host2, @req)
      end)
    end
  end

  defp start_host!(path, extra \\ []) do
    {id, extra} = Keyword.pop(extra, :id, make_ref())
    spec = {Host, Keyword.merge([wasm_path: path, expected_sha256: :unpinned], extra)}
    start_supervised!(Supervisor.child_spec(spec, id: id))
  end

  # Raises when the artifact is missing; :wasm tests are excluded by
  # test_helper.exs unless GRAPHLAW_REQUIRE_ENGINE=1.
  defp with_engine(fun), do: fun.(GraphlawEx.TestEngine.engine_path!())

  defp keypair do
    seed = :crypto.strong_rand_bytes(32)
    {pub, _sig} = KillSwitch.Ed25519.sign(seed, "")
    {pub, seed}
  end

  defp signed(epoch, scope, key_id, seed) do
    rev = %{
      "epoch" => epoch,
      "scope" => scope,
      "issued_unix" => System.system_time(:second),
      "reason" => "s7-safety"
    }

    {_pub, sig} = KillSwitch.Ed25519.sign(seed, KillSwitch.canonical_json(rev))

    %{
      "revocation" => rev,
      "attestation" => %{
        "key_id" => key_id,
        "alg" => "ed25519",
        "signature" => Base.encode16(sig, case: :lower)
      }
    }
  end
end
