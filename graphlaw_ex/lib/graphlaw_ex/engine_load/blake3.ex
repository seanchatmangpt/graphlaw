# SPDX-FileCopyrightText: 2026 Sean Chatman <https://github.com/seanchatmangpt/graphlaw/graphs/contributors>
#
# SPDX-License-Identifier: MIT
defmodule GraphlawEx.EngineLoad.Blake3 do
  @moduledoc """
  Thin delegate to `GraphlawEx.Blake3`, kept so `GraphlawEx.EngineLoad` and
  existing callers keep working.

  ## Examples

      iex> GraphlawEx.EngineLoad.Blake3.hash_hex("")
      "af1349b9f5f9a1a6a0404dea36dcc9499bcb25c9adc112b7cc9a93cae41f3262"
  """

  @doc "Lowercase hex BLAKE3-256 of `bytes`."
  @spec hash_hex(binary()) :: String.t()
  defdelegate hash_hex(bytes), to: GraphlawEx.Blake3
end
