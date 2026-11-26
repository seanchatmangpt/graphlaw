# SPDX-FileCopyrightText: 2026 Sean Chatman <https://github.com/seanchatmangpt/graphlaw/graphs/contributors>
#
# SPDX-License-Identifier: MIT
defmodule GraphlawEx.Safety.WasiLockdownTest do
  use ExUnit.Case, async: false

  alias GraphlawEx.Host

  @wasm_path GraphlawEx.TestEngine.wasm_path()
  @require_engine System.get_env("GRAPHLAW_REQUIRE_ENGINE") == "1"
  @moduletag timeout: 300_000

  # The engine imports 28 wasi_snapshot_preview1 functions, including fd_*/path_*
  # and environ_*. Zero ambient authority means the host grants none of it.
  test "the host's WASI options grant no argv, env, preopens or stdio" do
    opts = Host.wasi_options()
    assert opts.args == []
    assert opts.env == %{}
    assert opts.preopen == []
    assert opts.stdin == nil and opts.stdout == nil and opts.stderr == nil
  end

  @tag :wasm
  test "ATTACH through the sql-rdf path is refused typed and touches no filesystem path" do
    if File.regular?(@wasm_path) do
      host =
        start_link_supervised!(
          {Host, wasm_path: @wasm_path, expected_sha256: :unpinned, name: nil}
        )

      sentinel =
        Path.join(System.tmp_dir!(), "gl_attach_#{System.unique_integer([:positive])}.db")

      for path <- ["../../../etc/passwd", sentinel] do
        request = %{
          "op" => "duckdb",
          "mode" => "exec",
          "query_ast" => "urn:s1:q_attach",
          "ast" => %{
            "turtle" => """
            @prefix sql: <http://graphlaw.org/sql#> .
            @prefix ex: <http://example.org/attach#> .
            ex:A a sql:AttachStatement ;
                sql:statementIndex 1 ;
                sql:attachTarget [ sql:databasePath "#{path}" ; sql:attachAlias "s1att" ] .
            """
          },
          "subject_prefix" => "urn:s1:r",
          "vocab_prefix" => "urn:s1:v"
        }

        # Host.request/3 answers transport success with the engine's refusal
        # envelope (`"ok" => false`); a typed host refusal is also lawful.
        error =
          case Host.request(host, request) do
            {:ok, %{"ok" => false, "error" => %{} = engine_error}} -> engine_error
            {:error, %GraphlawEx.Refusal{} = refusal} -> refusal
            {:error, %GraphlawEx.Error{} = refusal} -> refusal
          end

        # Typed, and the refusal carries the allowlist reason, not an OS error.
        text = inspect(error, limit: :infinity, printable_limit: :infinity)
        assert text =~ "GRAPHLAW_ATTACH_ROOTS" or text =~ "REFUSED"
        refute text =~ "enoent"
        refute text =~ "No such file"
      end

      # No filesystem effect observable from the host side.
      refute File.exists?(sentinel)

      # Host still serves after the refusals.
      assert {:ok, %{"ok" => true}} = Host.request(host, %{"op" => "capabilities"})
    else
      message = "[SKIP] wasi lockdown engine court: wasm artifact not found at #{@wasm_path}."
      if @require_engine, do: flunk(message)
      IO.puts(message)
    end
  end
end
