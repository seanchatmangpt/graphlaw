# SPDX-FileCopyrightText: 2026 Sean Chatman <https://github.com/seanchatmangpt/graphlaw/graphs/contributors>
#
# SPDX-License-Identifier: MIT
defmodule GraphlawEx.MixTasksTest do
  use ExUnit.Case, async: false

  alias GraphlawEx.WasmConfig
  alias Mix.Tasks.GraphlawEx.Doctor
  alias Mix.Tasks.GraphlawEx.Fetch
  alias Mix.Tasks.GraphlawEx.Verify

  @env "GRAPHLAW_WASM_PATH"

  setup do
    previous_shell = Mix.shell()
    Mix.shell(Mix.Shell.Process)

    previous_env = System.get_env(@env)

    app_path =
      case Application.fetch_env(:graphlaw_ex, :wasm_path) do
        {:ok, v} -> {:ok, v}
        :error -> :error
      end

    System.delete_env(@env)
    Application.delete_env(:graphlaw_ex, :wasm_path)

    on_exit(fn ->
      Mix.shell(previous_shell)
      if previous_env, do: System.put_env(@env, previous_env), else: System.delete_env(@env)

      case app_path do
        {:ok, v} -> Application.put_env(:graphlaw_ex, :wasm_path, v)
        :error -> Application.delete_env(:graphlaw_ex, :wasm_path)
      end
    end)

    :ok
  end

  test "task is discoverable with a shortdoc" do
    assert Mix.Task.get("graphlaw_ex.verify") == Verify
    assert Mix.Task.shortdoc(Verify) =~ "BLAKE3"
  end

  @tag :tmp_dir
  test "a foreign artifact raises REFUSED_ARTIFACT_INTEGRITY", %{tmp_dir: tmp} do
    foreign = Path.join(tmp, "foreign.wasm")
    File.write!(foreign, "not the pinned engine")
    System.put_env(@env, foreign)

    error = assert_raise Mix.Error, fn -> Verify.run([]) end
    assert Exception.message(error) =~ "REFUSED_ARTIFACT_INTEGRITY"
  end

  test "a missing artifact fails loudly rather than passing" do
    System.put_env(@env, "/nonexistent/graphlaw.wasm")
    error = assert_raise Mix.Error, fn -> Verify.run([]) end
    assert Exception.message(error) =~ "cannot read /nonexistent/graphlaw.wasm"
  end

  # :wasm - excluded loudly without the artifact (test_helper.exs); under
  # GRAPHLAW_REQUIRE_ENGINE=1 engine_path!/0 raises instead.
  @tag :wasm
  test "the vendored artifact verifies" do
    _ = GraphlawEx.TestEngine.engine_path!()
    vendored = WasmConfig.vendored_path()

    assert File.regular?(vendored),
           "the verify court checks the VENDORED artifact, none at #{vendored}"

    Verify.run([])
    assert_received {:mix_shell, :info, [path_msg]}
    assert path_msg =~ vendored
    assert path_msg =~ ~r/\(\d+ bytes\)/
    assert_received {:mix_shell, :info, [pin_msg]}
    assert pin_msg =~ ~r/blake3 [0-9a-f]{64} OK/
  end

  @tag :tmp_dir
  test "fetch fails closed on a digest mismatch and leaves no .part", %{tmp_dir: tmp} do
    foreign = Path.join(tmp, "foreign.wasm")
    File.write!(foreign, "not the pinned engine")
    cache = with_cache_dir(tmp)

    error = assert_raise Mix.Error, fn -> Fetch.run(["--path", foreign]) end
    assert Exception.message(error) =~ "REFUSED_ARTIFACT_INTEGRITY"
    assert Exception.message(error) =~ "GRAPHLAW_WASM_PATH"
    assert Path.wildcard(Path.join(cache, "**/*")) |> Enum.filter(&File.regular?/1) == []
  end

  @tag :tmp_dir
  test "fetch reports an unreadable source path readably", %{tmp_dir: tmp} do
    with_cache_dir(tmp)
    error = assert_raise Mix.Error, fn -> Fetch.run(["--path", Path.join(tmp, "nope.wasm")]) end
    assert Exception.message(error) =~ "nope.wasm"
    refute Exception.message(error) =~ ":enoent"
  end

  test "fetch and doctor reject unknown flags" do
    assert_raise OptionParser.ParseError, fn -> Fetch.run(["--bogus"]) end
    assert_raise OptionParser.ParseError, fn -> Doctor.run(["--bogus"]) end
  end

  @tag :tmp_dir
  test "doctor fails on a missing artifact", %{tmp_dir: tmp} do
    missing = Path.join(tmp, "absent.wasm")
    error = assert_raise Mix.Error, fn -> Doctor.run(["--path", missing]) end
    assert Exception.message(error) =~ "check(s) failed"
    assert_received {:mix_shell, :info, ["  [FAIL] artifact" <> _ = msg]}
    assert msg =~ "mix graphlaw_ex.fetch"
  end

  @tag :wasm
  @tag :tmp_dir
  test "fetch stages a local pinned artifact, then is a cache-hit no-op", %{tmp_dir: tmp} do
    _ = GraphlawEx.TestEngine.engine_path!()
    cache = with_cache_dir(tmp)
    vendored = WasmConfig.vendored_path()

    Fetch.run(["--path", vendored])
    assert_received {:mix_shell, :info, [staged]}
    assert staged =~ "staged"
    [placed] = Path.wildcard(Path.join(cache, "**/graphlaw.wasm"))
    assert File.read!(placed) == File.read!(vendored)

    Fetch.run([])
    assert_received {:mix_shell, :info, [hit]}
    assert hit =~ "present, digest ok"
  end

  defp with_cache_dir(tmp) do
    cache = Path.join(tmp, "cache")
    previous = System.get_env("GRAPHLAW_CACHE_DIR")
    System.put_env("GRAPHLAW_CACHE_DIR", cache)

    on_exit(fn ->
      if previous,
        do: System.put_env("GRAPHLAW_CACHE_DIR", previous),
        else: System.delete_env("GRAPHLAW_CACHE_DIR")
    end)

    cache
  end
end
