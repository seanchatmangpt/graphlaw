# SPDX-FileCopyrightText: 2026 Sean Chatman <https://github.com/seanchatmangpt/graphlaw/graphs/contributors>
#
# SPDX-License-Identifier: MIT
defmodule GraphlawExTest do
  use ExUnit.Case, async: false

  # Chicago: the real wasm artifact is the collaborator. Artifact tests are
  # :wasm: excluded loudly when the artifact is absent (test_helper.exs), and
  # when GRAPHLAW_REQUIRE_ENGINE=1 is set an absent artifact is a failure
  # (GraphlawEx.TestEngine.engine_path!/0 raises) — fail-closed gate.
  # No mocks, no sleeps.

  @moduletag timeout: 300_000

  @honest_request %{
    "op" => "law",
    "data" => %{
      "text" => "@prefix : <https://test.graphlaw.dev/> .\n:a :edge :b .\n:b :edge :c .\n",
      "dialect" => "turtle"
    },
    "steps" => [%{"step" => "rdfs"}]
  }

  setup context do
    if context[:wasm] do
      GraphlawEx.EngineLoad.purge_cache()
      {:ok, wasm_path: GraphlawEx.TestEngine.engine_path!()}
    else
      :ok
    end
  end

  describe "the real-artifact court (facade + host over the real ABI)" do
    @tag :wasm
    test "loads the artifact, admits an honest order and surfaces engine refusals", %{
      wasm_path: wasm_path
    } do
      start_supervised!({GraphlawEx, wasm_path: wasm_path, expected_sha256: :unpinned})
      assert GraphlawEx.Host.available?(GraphlawEx)

      {:ok, info} = GraphlawEx.Host.info(GraphlawEx)
      assert byte_size(info.wasm_sha256) == 64

      assert {:ok, %{"ok" => true} = response} =
               GraphlawEx.admit(Jason.encode!(@honest_request))

      assert is_binary(response["nquads"])
      assert is_list(response["states"]) and is_list(response["receipts"])

      # An unknown op is a typed refusal raised *inside* the module: transport
      # success (`{:ok, _}`), engine refusal (`"ok" => false`) — never an
      # exception, never a crash.
      assert {:ok, %{"ok" => false, "error" => error}} =
               GraphlawEx.admit(Jason.encode!(%{"op" => "definitely-not-an-op"}))

      assert is_binary(error["message"])
      assert error["message"] =~ "unknown op"
    end

    @tag :wasm
    test "a payload that is not valid UTF-8 is refused before the engine", %{
      wasm_path: wasm_path
    } do
      start_supervised!({GraphlawEx, wasm_path: wasm_path, expected_sha256: :unpinned})

      assert {:error, %{code: :invalid_encoding}} = GraphlawEx.admit(<<0xFF, 0xFE, "op">>)
    end

    @tag :wasm
    test "start_link fails loudly (typed) on a missing artifact and on a digest mismatch", %{
      wasm_path: wasm_path
    } do
      stop_default()

      assert {:error, {:wasm_load_failed, _path, :enoent}} =
               GraphlawEx.start_link(
                 wasm_path: "/nonexistent/graphlaw.wasm",
                 expected_sha256: :unpinned,
                 name: GraphlawEx
               )

      # The fail-closed digest precondition runs before the spawn.
      assert {:error, {:wasm_digest_mismatch, %{expected: expected, actual: actual}}} =
               GraphlawEx.start_link(
                 wasm_path: wasm_path,
                 expected_sha256: String.duplicate("ab", 32),
                 name: GraphlawEx
               )

      assert expected == String.duplicate("ab", 32)
      assert actual =~ ~r/\A[0-9a-f]{64}\z/
    end

    @tag :wasm
    test "the engine's packed result is unpacked through the signed-i64 normalization", %{
      wasm_path: wasm_path
    } do
      # The unit property is courted in GraphlawEx.ABITest; here the real
      # engine must answer repeatedly through the same path (the old positive
      # guard hung on it).
      start_supervised!({GraphlawEx, wasm_path: wasm_path, expected_sha256: :unpinned})

      for _ <- 1..5 do
        assert {:ok, %{"ok" => true}} = GraphlawEx.admit(Jason.encode!(@honest_request))
      end
    end
  end

  describe "the pinned vendored load (no opts: manifest pin + allowlist + exports)" do
    @tag :wasm
    test "loads through the default resolution with the full admission order" do
      stop_default()
      GraphlawEx.EngineLoad.purge_cache()

      # No wasm_path opt: WasmConfig falls through env/app-config to the
      # vendored artifact, whose sha256 must match the MANIFEST pin.
      refute System.get_env("GRAPHLAW_WASM_PATH")

      start_supervised!(GraphlawEx)
      assert GraphlawEx.Host.available?(GraphlawEx)

      {:ok, info} = GraphlawEx.Host.info(GraphlawEx)
      assert info.wasm_sha256 == GraphlawEx.WasmConfig.pinned_sha256()
    end
  end

  describe "the pool" do
    @tag :wasm
    test "routes to the shortest mailbox over the real engine", %{wasm_path: wasm_path} do
      start_supervised!(
        {GraphlawEx.Pool, size: 1, wasm_path: wasm_path, expected_sha256: :unpinned}
      )

      assert GraphlawEx.Pool.members() |> length() == 1
      assert is_pid(GraphlawEx.Pool.pick())

      assert {:ok, %{"ok" => true}} = GraphlawEx.Pool.request(@honest_request)

      assert {:ok, %{wasm_sha256: sha}} = GraphlawEx.Pool.info()
      assert byte_size(sha) == 64
    end

    test "with no member started, request is a typed refusal" do
      assert match?({:error, %{code: _}}, GraphlawEx.Pool.request(@honest_request))
    end
  end

  defp stop_default do
    case Process.whereis(GraphlawEx) do
      nil -> :ok
      pid -> GenServer.stop(pid)
    end
  end
end
