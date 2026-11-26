# SPDX-FileCopyrightText: 2026 Sean Chatman <https://github.com/seanchatmangpt/graphlaw/graphs/contributors>
#
# SPDX-License-Identifier: MIT
# Engine gate, centralized. Tests that need the real wasm engine carry
# `@tag :wasm` (or `@moduletag :wasm`) and resolve the artifact through
# `GraphlawEx.TestEngine.engine_path!/0`, which raises when it is missing.
#
#   * artifact present                    -> :wasm tests run.
#   * artifact missing, REQUIRE_ENGINE=1  -> :wasm tests still run and FAIL
#                                            (engine_path!/0 raises): a skip is a failure.
#   * artifact missing, otherwise         -> :wasm tests are excluded, loudly, with
#                                            the reason; ExUnit reports them as excluded.
engine_path = GraphlawEx.TestEngine.wasm_path()
require_engine? = System.get_env("GRAPHLAW_REQUIRE_ENGINE") == "1"

exclude =
  cond do
    File.regular?(engine_path) ->
      []

    require_engine? ->
      []

    true ->
      IO.puts(
        "[SKIP] :wasm tests excluded: engine artifact not found at #{engine_path} " <>
          "(set PRIV_WASM or GRAPHLAW_WASM_PATH, or vendor priv/graphlaw/graphlaw.wasm; " <>
          "GRAPHLAW_REQUIRE_ENGINE=1 turns this skip into a failure). A skip is not a pass."
      )

      [:wasm]
  end

ExUnit.start(exclude: exclude)
