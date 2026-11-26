# SPDX-FileCopyrightText: 2026 Sean Chatman <https://github.com/seanchatmangpt/graphlaw/graphs/contributors>
#
# SPDX-License-Identifier: MIT
defmodule GraphlawEx.DoctestsTest do
  # Doctests for every lib module carrying `iex>` examples that no other test
  # file already covers. None of these examples call the wasm engine; the
  # engine-needing ones live in their own modules' dedicated tests.
  use ExUnit.Case, async: true

  doctest GraphlawEx.ABI
  doctest GraphlawEx.Authority
  doctest GraphlawEx.Blake3
  doctest GraphlawEx.Config
  doctest GraphlawEx.EngineLoad.Blake3
  doctest GraphlawEx.Error
  doctest GraphlawEx.Error.Class
  doctest GraphlawEx.Introspection
  doctest GraphlawEx.Introspection.Tables
  doctest GraphlawEx.KillSwitch.Store
  doctest GraphlawEx.Refusal
  doctest GraphlawEx.Registry
  doctest GraphlawEx.Result.Term
  doctest GraphlawEx.Telemetry
  doctest GraphlawEx.WasmConfig
end
