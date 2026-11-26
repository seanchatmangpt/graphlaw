# SPDX-FileCopyrightText: 2026 Sean Chatman <https://github.com/seanchatmangpt/graphlaw/graphs/contributors>
#
# SPDX-License-Identifier: MIT
defmodule GraphlawEx.EngineLoad.SingleFlightWiringTest do
  # purge_cache/0 is process-wide: serialize.
  use ExUnit.Case, async: false

  alias GraphlawEx.EngineLoad
  alias GraphlawEx.EngineLoad.SingleFlight

  @n 8
  @wasm Path.expand("../../priv/graphlaw/graphlaw.wasm", __DIR__)

  setup do
    EngineLoad.purge_cache()
    bytes = File.read!(@wasm)
    digest = :crypto.hash(:sha256, bytes) |> Base.encode16(case: :lower)
    {:ok, bytes: bytes, digest: digest}
  end

  test "N concurrent cold admits compile once and share one admitted entry", %{
    bytes: bytes,
    digest: digest
  } do
    test = self()
    id = {:wiring, make_ref()}

    :telemetry.attach(
      id,
      [:graphlaw_ex, :engine, :compile],
      fn _e, _m, meta, _c -> send(test, {:compiled, meta.wasm_sha256}) end,
      nil
    )

    on_exit(fn -> :telemetry.detach(id) end)

    results =
      1..@n
      |> Enum.map(fn _ ->
        Task.async(fn -> EngineLoad.admit(bytes, expected_sha256: :unpinned, fuel?: true) end)
      end)
      |> Enum.map(&Task.await(&1, 30_000))

    assert Enum.all?(results, &match?({:ok, %{wasm_sha256: ^digest}}, &1))
    assert_receive {:compiled, ^digest}
    refute_receive {:compiled, _}, 0
    assert EngineLoad.cached?(digest, true)
  end

  test "a pin mismatch is refused per caller even when the entry is cached", %{
    bytes: bytes,
    digest: digest
  } do
    assert {:ok, _} = EngineLoad.admit(bytes, expected_sha256: :unpinned)

    assert {:error, %GraphlawEx.Refusal{code: :wasm_digest_mismatch}} =
             EngineLoad.admit(bytes, expected_sha256: String.duplicate("0", 64))

    assert EngineLoad.cached?(digest, true)
  end

  test "purge_cache/0 drops the unified cache key", %{bytes: bytes, digest: digest} do
    assert {:ok, _} = EngineLoad.admit(bytes, expected_sha256: :unpinned)
    assert EngineLoad.cached?(digest, true)
    assert EngineLoad.purge_cache() == :ok
    refute EngineLoad.cached?(digest, true)
  end

  test "published entries are capped; oldest are evicted" do
    keys = for i <- 1..40, do: {:cap_test, i, make_ref()}
    for k <- keys, do: assert({:ok, _} = SingleFlight.fetch_or_compile(k, fn -> {:ok, :v} end))

    cached = Enum.count(keys, &(SingleFlight.cached(&1) != :error))
    assert cached == 32
    assert SingleFlight.cached(hd(keys)) == :error
    assert SingleFlight.cached(List.last(keys)) == {:ok, :v}
  end
end
