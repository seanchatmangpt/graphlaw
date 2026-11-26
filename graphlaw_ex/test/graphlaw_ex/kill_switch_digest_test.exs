# SPDX-License-Identifier: MIT
defmodule GraphlawEx.KillSwitchDigestTest do
  # Mutates the node-global :persistent_term epoch: must not run concurrently.
  use ExUnit.Case, async: false

  alias GraphlawEx.KillSwitch
  alias GraphlawEx.KillSwitch.Ed25519

  @seed :binary.copy(<<7>>, 32)
  @digest String.duplicate("ab", 32)
  @other String.duplicate("cd", 32)

  setup do
    :ok = KillSwitch.reset()
    on_exit(fn -> KillSwitch.reset() end)
    {public, _sig} = Ed25519.sign(@seed, "x")
    %{keys: %{"ops" => Base.encode16(public, case: :lower)}}
  end

  defp signed_doc(scope, epoch) do
    rev = %{"epoch" => epoch, "scope" => scope, "issued_unix" => 1, "reason" => "t"}
    {_pub, sig} = Ed25519.sign(@seed, KillSwitch.canonical_json(rev))

    %{
      "revocation" => rev,
      "attestation" => %{
        "key_id" => "ops",
        "alg" => "ed25519",
        "signature" => Base.encode16(sig, case: :lower)
      }
    }
  end

  test "uppercase scope and uppercase probe digest both refuse; other digest passes", %{
    keys: keys
  } do
    upper = String.upcase(@digest)
    assert {:ok, %{scope: @digest}} = KillSwitch.install(signed_doc(upper, 1), keys)

    assert {:error, %GraphlawEx.Refusal{code: :lease_revoked}} = KillSwitch.check(upper)
    assert {:error, %GraphlawEx.Refusal{code: :lease_revoked}} = KillSwitch.check(@digest)
    assert KillSwitch.check(@other) == :ok
  end

  test "lowercase scope refuses an uppercase probe digest (no fail-open)", %{keys: keys} do
    assert {:ok, _} = KillSwitch.install(signed_doc(@digest, 1), keys)

    assert {:error, %GraphlawEx.Refusal{details: %{wasm_sha256: @digest}}} =
             KillSwitch.check(String.upcase(@digest))
  end
end
