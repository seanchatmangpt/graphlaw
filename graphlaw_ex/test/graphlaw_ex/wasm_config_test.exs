defmodule GraphlawEx.WasmConfigTest do
  use ExUnit.Case, async: false

  alias GraphlawEx.WasmConfig

  @env "GRAPHLAW_WASM_PATH"
  @vendored WasmConfig.vendored_path()

  setup do
    System.delete_env(@env)
    Application.delete_env(:graphlaw_ex, :wasm_path)
    on_exit(fn ->
      System.delete_env(@env)
      Application.delete_env(:graphlaw_ex, :wasm_path)
    end)
    :ok
  end

  describe "path resolution (4 tiers)" do
    test "explicit opts win over everything" do
      System.put_env(@env, "/from/env.wasm")
      assert WasmConfig.wasm_path(wasm_path: "/from/opts.wasm") == "/from/opts.wasm"
    end

    test "the environment tier is next" do
      System.put_env(@env, "/from/env.wasm")
      assert WasmConfig.wasm_path() == "/from/env.wasm"
    end

    test "then application config, then the vendored default" do
      Application.put_env(:graphlaw_ex, :wasm_path, "/from/app_config.wasm")
      assert WasmConfig.wasm_path() == "/from/app_config.wasm"

      Application.delete_env(:graphlaw_ex, :wasm_path)
      assert WasmConfig.wasm_path() == @vendored
    end

    test "empty strings do not count as present" do
      System.put_env(@env, "")
      assert WasmConfig.wasm_path() == @vendored
    end
  end

  describe "the pin" do
    test "pinned_sha256/0 reads the vendored manifest" do
      assert {:ok, manifest} = WasmConfig.manifest()
      assert manifest["artifact"]["sha256"] == WasmConfig.pinned_sha256()
      assert WasmConfig.pinned_sha256() =~ ~r/\A[0-9a-f]{64}\z/
    end

    @tag :tmp_dir
    test "the security property: env and app-config paths STAY pinned", %{tmp_dir: tmp} do
      foreign = Path.join(tmp, "foreign.wasm")
      File.write!(foreign, "not really wasm")

      System.put_env(@env, foreign)
      assert WasmConfig.expected_sha256() == WasmConfig.pinned_sha256()

      Application.put_env(:graphlaw_ex, :wasm_path, foreign)
      assert WasmConfig.expected_sha256() == WasmConfig.pinned_sha256()

      System.delete_env(@env)
      Application.delete_env(:graphlaw_ex, :wasm_path)
      assert WasmConfig.expected_sha256(wasm_path: foreign) == :unpinned
      # The vendored path itself, named explicitly, is still pinned.
      assert WasmConfig.expected_sha256(wasm_path: @vendored) == WasmConfig.pinned_sha256()
      # opts[:expected_sha256] wins when given.
      assert WasmConfig.expected_sha256(wasm_path: foreign, expected_sha256: :unpinned) == :unpinned
      assert WasmConfig.expected_sha256(expected_sha256: "ab" |> String.duplicate(32)) == String.duplicate("ab", 32)
    end
  end

  describe "import allowlist" do
    test "is the closed wasi_snapshot_preview1 list from the real module" do
      assert {:ok, allow} = WasmConfig.import_allowlist()
      assert length(allow) == 28
      names = Enum.map(allow, &elem(&1, 0))
      assert "proc_exit" in names and "fd_write" in names and "path_open" in names

      for {_name, {:fn, params, results}} <- allow do
        assert Enum.all?(params ++ results, &(&1 in [:i32, :i64, :f32, :f64, :v128]))
      end
    end
  end

  describe "limits/1" do
    test "fuel follows the deadline by default" do
      assert %{fuel: 5_000_000_000, timeout_ms: 5_000, fuel_per_ms: 1_000_000} = WasmConfig.limits()
      assert WasmConfig.limits(timeout_ms: 100).fuel == 100_000_000
      assert WasmConfig.limits(fuel: 7).fuel == 7
    end

    test "invalid values fall through to the default" do
      assert WasmConfig.limits(timeout_ms: 0).timeout_ms == 5_000
      assert WasmConfig.limits(max_queue: -3).max_queue == 64
    end
  end
end
