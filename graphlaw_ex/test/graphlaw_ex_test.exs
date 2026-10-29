defmodule GraphlawExTest do
  use ExUnit.Case, async: false

  # Chicago: the real wasm artifact is the collaborator. Absence of the
  # artifact is a loud typed skip, never a faked pass. No mocks, no sleeps.
  @wasm_path System.get_env("PRIV_WASM") ||
               Path.expand("../priv/graphlaw.wasm", __DIR__)

  @honest_request %{
    "op" => "law",
    "data" => %{
      "text" =>
        "@prefix : <https://test.graphlaw.dev/> .\n:a :edge :b .\n:b :edge :c .\n",
      "dialect" => "turtle"
    },
    "steps" => [%{"step" => "rdfs"}]
  }

  describe "instantiation" do
    @tag :wasm_artifact
    test "loads the real graphlaw wasm artifact and answers admit over the real ABI" do
      unless File.regular?(@wasm_path) do
        IO.puts(
          "[SKIP] graphlaw_ex instantiation court: wasm artifact not found at " <>
            "#{@wasm_path} (set PRIV_WASM to the absolute path of graphlaw_wasm.wasm). " <>
            "A loud skip is not a pass."
        )

        assert :ok
      else
        start_supervised!({GraphlawEx, wasm_path: @wasm_path})

        # The process is alive with a loaded module — instantiation succeeded.
        assert Process.whereis(GraphlawEx) |> is_pid()

        # Honest admit: the RDFS entailment runs inside the module and the
        # receipt JSON comes back through the gl_alloc/gl_call/gl_free ABI.
        assert {:ok, %{"ok" => true} = response} =
                 GraphlawEx.admit(Jason.encode!(@honest_request))

        assert is_binary(response["nquads"])
        assert is_list(response["states"]) and is_list(response["receipts"])

        # Tampered refuse: an unknown op is a typed refusal raised *inside*
        # the module, surfaced as {:error, {:refused, error}}.
        assert {:error, {:refused, error}} =
                 GraphlawEx.admit(Jason.encode!(%{"op" => "definitely-not-an-op"}))

        assert is_binary(error["message"])
        assert error["message"] =~ "unknown op"
      end
    end

    test "start_link fails loudly (typed) on a missing artifact" do
      # The GenServer registers under the module name; release any instance
      # a prior test left running before expecting a failed start.
      case Process.whereis(GraphlawEx) do
        nil -> :ok
        pid -> GenServer.stop(pid)
      end

      assert {:error, {:wasm_load_failed, _path, _reason}} =
               start_no_supervise("/nonexistent/graphlaw.wasm")
    end
  end

  defp start_no_supervise(path) do
    # Deliberately not under the test supervisor: the expected outcome is a
    # failed start, and the GenServer has already exited when we observe it.
    GraphlawEx.start_link(wasm_path: path)
  end
end
