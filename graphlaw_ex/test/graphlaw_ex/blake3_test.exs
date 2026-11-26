# SPDX-FileCopyrightText: 2026 Sean Chatman <https://github.com/seanchatmangpt/graphlaw/graphs/contributors>
#
# SPDX-License-Identifier: MIT
defmodule GraphlawEx.EngineLoad.Blake3Test do
  use ExUnit.Case, async: true

  alias GraphlawEx.EngineLoad.Blake3

  @moduletag timeout: 300_000
  @wasm_path System.get_env("PRIV_WASM") || GraphlawEx.TestEngine.wasm_path()

  test "known vector: empty input" do
    assert Blake3.hash_hex("") ==
             "af1349b9f5f9a1a6a0404dea36dcc9499bcb25c9adc112b7cc9a93cae41f3262"
  end

  test "known vector: single zero byte (official test vector, input_len 1)" do
    assert Blake3.hash_hex(<<0>>) ==
             "2d3adedff11b61f14c886e35afa036736dcd87a74d27b5c1510225d0f592e213"
  end

  test "known vector: abc" do
    assert Blake3.hash_hex("abc") ==
             "6437b3ac38465133ffb63b75273a8db548c558465d79db03fd359c6cd5bd9d85"
  end

  test "digest is 32 bytes lowercase hex and sensitive to a 1-bit change across chunk sizes" do
    for size <- [63, 64, 65, 1023, 1024, 1025, 2048, 3000] do
      bytes = :binary.copy(<<1>>, size)
      hex = Blake3.hash_hex(bytes)
      assert hex =~ ~r/\A[0-9a-f]{64}\z/
      assert hex == Blake3.hash_hex(bytes)
      refute hex == Blake3.hash_hex(bytes <> <<0>>)
    end
  end

  test "real wasm hashes to its sidecar pin" do
    sidecar = @wasm_path <> ".blake3"

    if File.regular?(@wasm_path) and File.regular?(sidecar) do
      [pin | _] = sidecar |> File.read!() |> String.split()
      assert Blake3.hash_hex(File.read!(@wasm_path)) == String.downcase(pin)
    else
      message =
        "[SKIP] blake3 real-wasm vector: need #{@wasm_path} and #{sidecar}. A loud skip is not a pass."

      if System.get_env("GRAPHLAW_REQUIRE_ENGINE") == "1", do: flunk(message)
      IO.puts(message)
    end
  end
end
