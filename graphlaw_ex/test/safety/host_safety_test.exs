# SPDX-FileCopyrightText: 2026 Sean Chatman <https://github.com/seanchatmangpt/graphlaw/graphs/contributors>
#
# SPDX-License-Identifier: MIT
defmodule GraphlawEx.HostSafetyTest do
  use ExUnit.Case, async: false

  # Chicago: real host GenServer over the real engine. :wasm tests raise via
  # TestEngine.engine_path!/0 when the artifact is missing and
  # GRAPHLAW_REQUIRE_ENGINE=1; otherwise they are excluded loudly.

  @moduletag timeout: 300_000

  test "non-map request is a typed refusal, not a crash" do
    for bad <- [nil, "op", 42, [1, 2], {:op, "law"}] do
      assert {:error, %GraphlawEx.Refusal{code: :invalid_json}} =
               GraphlawEx.Host.request(GraphlawEx.Host, bad)
    end
  end

  test "non-binary raw request is a typed refusal, not a crash" do
    for bad <- [nil, %{"op" => "capabilities"}, 42, [1]] do
      assert {:error, %GraphlawEx.Refusal{code: :invalid_json}} =
               GraphlawEx.Host.request_raw(GraphlawEx.Host, bad)
    end
  end

  @tag :wasm
  test "oversize request is refused :resource_limit before touching the engine" do
    with_engine(fn path ->
      host = start_host!(path, max_request_bytes: 32, recycle_bytes: huge())

      assert {:error, %GraphlawEx.Refusal{code: :resource_limit} = refusal} =
               GraphlawEx.Host.request_raw(host, String.duplicate("x", 1_000))

      assert refusal.details.max_request_bytes == 32
      assert refusal.details.bytes == 1_000

      assert {:ok, %{recycles: 0}} = GraphlawEx.Host.info(host)
      assert {:ok, %{"ok" => true}} = GraphlawEx.Host.request(host, %{"op" => "capabilities"})
      :ok = GraphlawEx.stop(host)
    end)
  end

  @tag :wasm
  test "oversize response is refused, the instance is recycled, and the host still serves" do
    with_engine(fn path ->
      host = start_host!(path, max_response_bytes: 1, recycle_bytes: huge())

      assert {:error, %GraphlawEx.Refusal{code: :resource_limit} = refusal} =
               GraphlawEx.Host.request(host, %{"op" => "capabilities"})

      assert refusal.details.max_response_bytes == 1

      assert {:ok, %{recycles: recycles}} = GraphlawEx.Host.info(host)
      assert recycles >= 1
      assert GraphlawEx.Host.available?(host)
      :ok = GraphlawEx.stop(host)
    end)
  end

  defp start_host!(path, extra) do
    start_link_supervised!(
      {GraphlawEx.Host, Keyword.merge([wasm_path: path, expected_sha256: :unpinned], extra)}
    )
  end

  defp huge, do: 268_435_456

  # Raises when the artifact is missing; :wasm tests are excluded by test_helper.exs
  # unless GRAPHLAW_REQUIRE_ENGINE=1.
  defp with_engine(fun), do: fun.(GraphlawEx.TestEngine.engine_path!())
end
