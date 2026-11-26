# SPDX-FileCopyrightText: 2026 Sean Chatman <https://github.com/seanchatmangpt/graphlaw/graphs/contributors>
#
# SPDX-License-Identifier: MIT
defmodule GraphlawEx.TestEngine do
  @moduledoc """
  Shared test helper resolving the graphlaw WASM engine artifact for tests.

  Resolution order in `wasm_path/0`:

  1. `PRIV_WASM` env override (lane/probe escape hatch)
  2. `GraphlawEx.WasmConfig.wasm_path/0` — the app's own chain:
     `:wasm_path` opt, `GRAPHLAW_WASM_PATH` env, app env, then the
     vendored `priv/graphlaw/graphlaw.wasm`.

  Fail-closed gate: when `GRAPHLAW_REQUIRE_ENGINE=1` is set, `engine_path!/0`
  flunks instead of returning a missing path — a missing artifact is a
  failure, never a silent skip (matches the existing test-suite semantics,
  e.g. `test/graphlaw_ex_test.exs`).
  """

  @priv_env "PRIV_WASM"
  @require_engine "GRAPHLAW_REQUIRE_ENGINE"

  @doc "Resolved engine path: `PRIV_WASM` override, else the app's `WasmConfig` chain."
  @spec wasm_path() :: String.t()
  def wasm_path do
    System.get_env(@priv_env) || GraphlawEx.WasmConfig.wasm_path()
  end

  @doc """
  Like `wasm_path/0` but fail-closed: under `GRAPHLAW_REQUIRE_ENGINE=1` a
  missing artifact flunks the test; otherwise returns `{:ok, path}` or
  `{:error, :missing}`.
  """
  @spec engine_path!() :: String.t() | no_return()
  def engine_path! do
    path = wasm_path()

    if File.exists?(path) do
      path
    else
      message =
        "graphlaw engine artifact missing at #{inspect(path)} " <>
          "(#{wasm_path_origin(path)}). " <>
          if(require_engine?(),
            do: "GRAPHLAW_REQUIRE_ENGINE=1: a skip is a failure (fail-closed gate).",
            else: "Set PRIV_WASM or GRAPHLAW_WASM_PATH, or vendor priv/graphlaw/graphlaw.wasm."
          )

      # Fail-closed: under the gate this is an ExUnit failure; without the
      # gate it is still a loud error, never a silent skip.
      flunk(message)
    end
  end

  defp require_engine?, do: System.get_env(@require_engine) == "1"

  defp wasm_path_origin(path) do
    cond do
      path == System.get_env(@priv_env) -> "from PRIV_WASM"
      path == System.get_env("GRAPHLAW_WASM_PATH") -> "from GRAPHLAW_WASM_PATH"
      true -> "vendored/app-resolved"
    end
  end

  defp flunk(message) do
    # Fail loudly inside the running ExUnit context when available; a bare
    # raise still fails any test process that calls this.
    raise(ExUnit.AssertionError, message: message)
  end
end
