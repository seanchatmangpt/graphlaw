# SPDX-FileCopyrightText: 2026 Sean Chatman <https://github.com/seanchatmangpt/graphlaw/graphs/contributors>
#
# SPDX-License-Identifier: MIT
defmodule GraphlawEx.EngineLoadSafetyTest do
  use ExUnit.Case, async: true

  # Lane S5: a nil pin is a distinct typed no-pin refusal, not a mismatch with
  # `expected: nil`. Real compiler, hand-built module bytes; no engine artifact
  # is needed, so nothing here can skip.

  alias GraphlawEx.EngineLoad
  alias GraphlawEx.Refusal

  @no_exports <<0, 97, 115, 109, 1, 0, 0, 0>>

  test "admit with a nil pin refuses :no_pin, not a mismatch against nil" do
    assert {:error, %Refusal{code: :wasm_digest_mismatch} = refusal} =
             EngineLoad.admit(@no_exports, expected_sha256: nil)

    assert refusal.details.reason == :no_pin
    refute Map.has_key?(refusal.details, :expected)

    assert refusal.details.actual ==
             :crypto.hash(:sha256, @no_exports) |> Base.encode16(case: :lower)
  end

  test "boot with a nil pin refuses :no_pin before compilation" do
    assert {:error, %Refusal{details: %{reason: :no_pin}}} =
             EngineLoad.boot(<<1, 2, 3>>, expected_sha256: nil)
  end

  test "a wrong hex pin stays a plain mismatch with no :reason" do
    assert {:error, %Refusal{code: :wasm_digest_mismatch, details: details}} =
             EngineLoad.admit(@no_exports, expected_sha256: String.duplicate("ab", 32))

    refute Map.has_key?(details, :reason)
    assert details.expected == String.duplicate("ab", 32)
  end

  test "verify_artifact with no pin is the typed integrity refusal" do
    assert {:error, {:REFUSED_ARTIFACT_INTEGRITY, %{reason: :no_pin}}} =
             EngineLoad.verify_artifact(@no_exports, expected_blake3: nil)
  end
end
