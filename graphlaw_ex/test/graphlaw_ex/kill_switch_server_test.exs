# SPDX-FileCopyrightText: 2026 Sean Chatman <https://github.com/seanchatmangpt/graphlaw/graphs/contributors>
#
# SPDX-License-Identifier: MIT
defmodule GraphlawEx.KillSwitchServerTest do
  @moduledoc """
  Courts for the durable kill-switch owner: install persists, a restart restores
  the trip, a tampered store refuses start, concurrent installs serialize.
  Real files in a tmp dir, real Ed25519 signatures, no doubles.
  """
  use ExUnit.Case, async: false

  alias GraphlawEx.KillSwitch
  alias GraphlawEx.KillSwitch.{Server, Store}

  @key_id "ops-2026"

  setup do
    dir = Path.join(System.tmp_dir!(), "gl_ks_#{System.unique_integer([:positive])}")
    File.mkdir_p!(dir)
    path = Path.join(dir, "kill_switch.json")

    seed = :crypto.strong_rand_bytes(32)
    {pub, _} = KillSwitch.Ed25519.sign(seed, "")

    prev =
      for k <- [:kill_switch_path, :trusted_keys], do: {k, Application.fetch_env(:graphlaw_ex, k)}

    Application.put_env(:graphlaw_ex, :kill_switch_path, path)
    Application.put_env(:graphlaw_ex, :trusted_keys, %{@key_id => pub})
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

    {:ok, path: path, seed: seed}
  end

  test "install persists to the store", %{path: path, seed: seed} do
    start_supervised!(Server)
    assert {:ok, %{epoch: 3, scope: "all"}} = Server.install(doc(3, seed))

    assert {:ok, [%{"revocation" => %{"epoch" => 3}}]} = Store.load(path)
    assert KillSwitch.current_epoch() == 3
  end

  test "restart restores the trip from the store", %{seed: seed} do
    start_supervised!(Server)
    assert {:ok, _} = Server.install(doc(5, seed))
    :ok = stop_supervised(Server)

    # Fresh-node simulation: in-memory epoch gone, only the file remains.
    :ok = KillSwitch.reset()
    assert KillSwitch.current_epoch() == 0

    start_supervised!(Server)
    assert KillSwitch.current_epoch() == 5
    assert {:error, %GraphlawEx.Refusal{code: :lease_revoked}} = KillSwitch.check(nil)
  end

  test "same-VM restart tolerates stale_epoch replay", %{seed: seed} do
    start_supervised!(Server)
    assert {:ok, _} = Server.install(doc(2, seed))
    :ok = stop_supervised(Server)

    start_supervised!(Server)
    assert KillSwitch.current_epoch() == 2
  end

  test "tampered store refuses start", %{path: path, seed: seed} do
    start_supervised!(Server)
    assert {:ok, _} = Server.install(doc(4, seed))
    :ok = stop_supervised(Server)
    :ok = KillSwitch.reset()

    tampered = File.read!(path) |> String.replace("art14-order", "forged-order")
    refute tampered == File.read!(path)
    File.write!(path, tampered)

    assert {:error, {reason, _spec}} = start_supervised(Server)
    assert {:kill_switch_restore_failed, {:unverified_document, 4, _refusal}} = reason
    assert KillSwitch.current_epoch() == 0
  end

  test "corrupt store file refuses start", %{path: path} do
    File.write!(path, "not json")

    assert {:error, {{:kill_switch_restore_failed, :corrupt}, _spec}} = start_supervised(Server)
  end

  test "concurrent installs serialize", %{path: path, seed: seed} do
    start_supervised!(Server)

    results =
      1..12
      |> Task.async_stream(fn e -> {e, Server.install(doc(e, seed))} end,
        max_concurrency: 12,
        timeout: 30_000
      )
      |> Enum.map(fn {:ok, r} -> r end)

    accepted = for {e, {:ok, _}} <- results, do: e
    refused = for {_e, {:error, _} = r} <- results, do: r

    assert 12 in accepted

    assert Enum.all?(
             refused,
             &match?({:error, %GraphlawEx.Refusal{details: %{reason: :stale_epoch}}}, &1)
           )

    assert KillSwitch.current_epoch() == 12

    {:ok, stored} = Store.load(path)
    assert Enum.map(stored, &get_in(&1, ["revocation", "epoch"])) == Enum.sort(accepted)
    assert Enum.map(Server.docs(), &get_in(&1, ["revocation", "epoch"])) == Enum.sort(accepted)
  end

  defp doc(epoch, seed) do
    rev = %{
      "epoch" => epoch,
      "scope" => "all",
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
