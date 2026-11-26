# SPDX-FileCopyrightText: 2026 Sean Chatman <https://github.com/seanchatmangpt/graphlaw/graphs/contributors>
#
# SPDX-License-Identifier: MIT
defmodule GraphlawEx.KillSwitchHardeningTest do
  @moduledoc """
  E6 courts: case-normalized check, persist-failure consistency, per-install
  key reads, telemetry, propagation hook. Real files, real signatures.
  """
  use ExUnit.Case, async: false

  alias GraphlawEx.KillSwitch
  alias GraphlawEx.KillSwitch.{Server, Store}

  @key_id "ops-2026"
  @digest String.duplicate("ab", 32)

  # Propagation target: a real function that reports to a named process.
  def propagate(doc), do: send(:ks_hardening_test, {:propagated, doc})

  setup do
    dir = Path.join(System.tmp_dir!(), "gl_ksh_#{System.unique_integer([:positive])}")
    File.mkdir_p!(dir)
    path = Path.join(dir, "kill_switch.json")
    seed = :crypto.strong_rand_bytes(32)
    {pub, _} = KillSwitch.Ed25519.sign(seed, "")

    keys = [:kill_switch_path, :trusted_keys, :kill_switch_propagate]
    prev = for k <- keys, do: {k, Application.fetch_env(:graphlaw_ex, k)}

    Application.put_env(:graphlaw_ex, :kill_switch_path, path)
    Application.put_env(:graphlaw_ex, :trusted_keys, %{@key_id => pub})
    Application.delete_env(:graphlaw_ex, :kill_switch_propagate)
    :ok = KillSwitch.reset()

    on_exit(fn ->
      for {k, prior} <- prev do
        case prior do
          {:ok, v} -> Application.put_env(:graphlaw_ex, k, v)
          :error -> Application.delete_env(:graphlaw_ex, k)
        end
      end

      KillSwitch.reset()
      File.rm_rf!(dir)
    end)

    {:ok, dir: dir, path: path, seed: seed, pub: pub}
  end

  test "check/1 normalizes the digest case (no fail-open)", %{seed: seed} do
    assert {:ok, %{scope: @digest}} =
             KillSwitch.install(doc(1, @digest, seed), %{@key_id => pub_of(seed)})

    assert {:error, %GraphlawEx.Refusal{code: :lease_revoked}} = KillSwitch.check(@digest)

    assert {:error, %GraphlawEx.Refusal{code: :lease_revoked}} =
             KillSwitch.check(String.upcase(@digest))

    assert :ok = KillSwitch.check(String.duplicate("cd", 32))
  end

  test "persist failure keeps docs == durable docs and an identical retry persists",
       %{dir: dir, path: path, seed: seed} do
    start_supervised!(Server)
    assert {:ok, _} = Server.install(doc(1, "all", seed))

    # Break the store: replace its directory with a regular file.
    File.rm_rf!(dir)
    File.write!(dir, "not a dir")

    d2 = doc(2, "all", seed)
    assert {:error, {:persist_failed, _}} = Server.install(d2)
    assert Enum.map(Server.docs(), &get_in(&1, ["revocation", "epoch"])) == [1]
    assert KillSwitch.current_epoch() == 2

    # Repair and retry the identical document.
    File.rm!(dir)
    File.mkdir_p!(dir)
    assert {:ok, %{epoch: 2}} = Server.install(d2)
    assert Enum.map(Server.docs(), &get_in(&1, ["revocation", "epoch"])) == [1, 2]
    assert {:ok, stored} = Store.load(path)
    # The retry rewrote the full in-memory set, so the store matches docs/0.
    assert [1, 2] == Enum.map(stored, &get_in(&1, ["revocation", "epoch"]))
  end

  test "trusted keys are read per install (rotation without restart)", %{seed: seed, pub: pub} do
    start_supervised!(Server)
    assert {:ok, _} = Server.install(doc(1, "all", seed))

    Application.put_env(:graphlaw_ex, :trusted_keys, %{})

    assert {:error, %GraphlawEx.Refusal{details: %{reason: :unknown_key_id}}} =
             Server.install(doc(2, "all", seed))

    Application.put_env(:graphlaw_ex, :trusted_keys, %{@key_id => pub})
    assert {:ok, %{epoch: 3}} = Server.install(doc(3, "all", seed))
  end

  test "install emits installed and refused telemetry", %{seed: seed, pub: pub} do
    id = "ks-hardening-#{System.unique_integer([:positive])}"
    me = self()

    :telemetry.attach_many(
      id,
      [[:graphlaw_ex, :kill_switch, :installed], [:graphlaw_ex, :kill_switch, :refused]],
      fn event, _m, meta, _ -> send(me, {:telemetry, event, meta}) end,
      nil
    )

    on_exit(fn -> :telemetry.detach(id) end)

    assert {:ok, _} = KillSwitch.install(doc(4, "all", seed), %{@key_id => pub})
    assert_receive {:telemetry, [:graphlaw_ex, :kill_switch, :installed], %{epoch: 4}}, 1_000

    assert {:error, _} = KillSwitch.install(doc(4, "all", seed), %{@key_id => pub})

    assert_receive {:telemetry, [:graphlaw_ex, :kill_switch, :refused], %{reason: :stale_epoch}},
                   1_000
  end

  test "propagation hook receives the signed doc after persist", %{seed: seed} do
    Process.register(self(), :ks_hardening_test)
    Application.put_env(:graphlaw_ex, :kill_switch_propagate, {__MODULE__, :propagate})
    start_supervised!(Server)

    d = doc(6, "all", seed)
    assert {:ok, _} = Server.install(d)
    assert_receive {:propagated, ^d}, 1_000
  end

  defp pub_of(seed), do: elem(KillSwitch.Ed25519.sign(seed, ""), 0)

  defp doc(epoch, scope, seed) do
    rev = %{
      "epoch" => epoch,
      "scope" => scope,
      "issued_unix" => System.system_time(:second),
      "reason" => "art14-order-2026-10-28"
    }

    {_pub, sig} = KillSwitch.Ed25519.sign(seed, KillSwitch.canonical_json(rev))

    %{
      "revocation" => rev,
      "attestation" => %{
        "key_id" => @key_id,
        "alg" => "ed25519",
        "signature" => Base.encode16(sig, case: :lower)
      }
    }
  end
end
