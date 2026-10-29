defmodule GraphlawExTest do
  use ExUnit.Case, async: false

  # Chicago: the real wasm artifact is the collaborator. Absence of the
  # artifact is a loud typed skip, never a faked pass — and when
  # GRAPHLAW_REQUIRE_ENGINE=1 is set, a skip is a failure (fail-closed gate).
  # No mocks, no sleeps.
  @wasm_path System.get_env("PRIV_WASM") ||
               Path.expand("priv/graphlaw/graphlaw.wasm", __DIR__)

  @require_engine System.get_env("GRAPHLAW_REQUIRE_ENGINE") == "1"

  @honest_request %{
    "op" => "law",
    "data" => %{
      "text" =>
        "@prefix : <https://test.graphlaw.dev/> .\n:a :edge :b .\n:b :edge :c .\n",
      "dialect" => "turtle"
    },
    "steps" => [%{"step" => "rdfs"}]
  }

  setup_all do
    if File.regular?(@wasm_path) do
      GraphlawEx.EngineLoad.purge_cache()
      :ok
    else
      message =
        "[SKIP] graphlaw_ex artifact courts: wasm artifact not found at #{@wasm_path} " <>
          "(set PRIV_WASM to the absolute path of the graphlaw wasm module). A loud skip is not a pass."

      if @require_engine, do: flunk(message)

      IO.puts(message)
      {:ok, skipped: true}
    end
  end

  describe "the real-artifact court (facade + host over the real ABI)" do
    @tag :wasm_artifact
    test "loads the artifact, admits an honest order and surfaces engine refusals", ctx do
      skip_or_run(ctx, fn ->
        start_supervised!({GraphlawEx, wasm_path: @wasm_path, expected_sha256: :unpinned})
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
      end)
    end

    @tag :wasm_artifact
    test "a payload that is not valid UTF-8 is refused before the engine", ctx do
      skip_or_run(ctx, fn ->
        start_supervised!({GraphlawEx, wasm_path: @wasm_path, expected_sha256: :unpinned})

        assert {:error, %{code: :invalid_encoding}} = GraphlawEx.admit(<<0xFF, 0xFE, "op">>)
      end)
    end

    @tag :wasm_artifact
    test "start_link fails loudly (typed) on a missing artifact and on a digest mismatch", ctx do
      skip_or_run(ctx, fn ->
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
                   wasm_path: @wasm_path,
                   expected_sha256: String.duplicate("ab", 32),
                   name: GraphlawEx
                 )

        assert expected == String.duplicate("ab", 32)
        assert actual =~ ~r/\A[0-9a-f]{64}\z/
      end)
    end

    @tag :wasm_artifact
    test "the engine's packed result is unpacked through the signed-i64 normalization", ctx do
      skip_or_run(ctx, fn ->
        # The unit property is courted in GraphlawEx.ABITest; here the real
        # engine must answer repeatedly through the same path (the old positive
        # guard hung on it).
        start_supervised!({GraphlawEx, wasm_path: @wasm_path, expected_sha256: :unpinned})

        for _ <- 1..5 do
          assert {:ok, %{"ok" => true}} = GraphlawEx.admit(Jason.encode!(@honest_request))
        end
      end)
    end
  end

  describe "the pinned vendored load (no opts: manifest pin + allowlist + exports)" do
    @tag :wasm_artifact
    test "loads through the default resolution with the full admission order", ctx do
      skip_or_run(ctx, fn ->
        stop_default()
        GraphlawEx.EngineLoad.purge_cache()

        # No wasm_path opt: WasmConfig falls through env/app-config to the
        # vendored artifact, whose sha256 must match the MANIFEST pin.
        refute System.get_env("GRAPHLAW_WASM_PATH")

        start_supervised!(GraphlawEx)
        assert GraphlawEx.Host.available?(GraphlawEx)

        {:ok, info} = GraphlawEx.Host.info(GraphlawEx)
        assert info.wasm_sha256 == GraphlawEx.WasmConfig.pinned_sha256()
      end)
    end
  end

  describe "the pool" do
    @tag :wasm_artifact
    test "routes to the shortest mailbox over the real engine", ctx do
      skip_or_run(ctx, fn ->
        start_supervised!(
          {GraphlawEx.Pool, size: 1, wasm_path: @wasm_path, expected_sha256: :unpinned}
        )

        assert GraphlawEx.Pool.members() |> length() == 1
        assert is_pid(GraphlawEx.Pool.pick())

        assert {:ok, %{"ok" => true}} = GraphlawEx.Pool.request(@honest_request)

        assert {:ok, %{wasm_sha256: sha}} = GraphlawEx.Pool.info()
        assert byte_size(sha) == 64
      end)
    end

    test "with no member started, request is a typed refusal" do
      assert match?({:error, %{code: _}}, GraphlawEx.Pool.request(@honest_request))
    end
  end

  # The fail-closed gate: when GRAPHLAW_REQUIRE_ENGINE=1, an absent artifact
  # already flunked in setup_all; here the artifact tests no-op loudly.
  defp skip_or_run(%{skipped: true}, _fun) do
    IO.puts("[SKIP] artifact court: no wasm artifact (GRAPHLAW_REQUIRE_ENGINE turns a skip into a failure).")
    assert :ok
  end

  defp skip_or_run(_ctx, fun), do: fun.()

  defp stop_default do
    case Process.whereis(GraphlawEx) do
      nil -> :ok
      pid -> GenServer.stop(pid)
    end
  end
end
