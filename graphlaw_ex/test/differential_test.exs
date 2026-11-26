# SPDX-FileCopyrightText: 2026 Sean Chatman <https://github.com/seanchatmangpt/graphlaw/graphs/contributors>
#
# SPDX-License-Identifier: MIT
defmodule GraphlawEx.DifferentialTest do
  use ExUnit.Case, async: false

  # Gate 2: the wasm engine reached through the Elixir facade must match the
  # NATIVE graphlaw build byte for byte. The goldens `native_canonical.json`,
  # `native_sparql.json` and `id.blake3` are minted by running the same ABI
  # requests through native `graphlaw::abi::call_json`
  # (`cargo run --example golden_mint -- <golden_dir>` in the parent repo), not
  # derived by hand. Real engine, no mocks. A missing engine fails closed when
  # GRAPHLAW_REQUIRE_ENGINE=1; every test asserts.

  alias GraphlawEx.Capability.API

  @golden Path.expand("fixtures/golden", __DIR__)
  @wasm_path System.get_env("PRIV_WASM") ||
               Path.expand("../priv/graphlaw/graphlaw.wasm", __DIR__)
  @pin Path.expand("../priv/graphlaw/MANIFEST.json", __DIR__)
       |> File.read!()
       |> Jason.decode!()
       |> get_in(["artifact", "sha256"])
  @engine? File.regular?(@wasm_path)
  @require_engine System.get_env("GRAPHLAW_REQUIRE_ENGINE") == "1"

  unless @engine? or @require_engine do
    IO.puts("[SKIP] differential courts: no wasm at #{@wasm_path}. A loud skip is not a pass.")
    @moduletag skip: "wasm artifact missing at #{@wasm_path}"
  end

  setup_all do
    if @engine? do
      start_supervised!({GraphlawEx, wasm_path: @wasm_path, expected_sha256: @pin})
      :ok
    else
      flunk(
        "differential courts: wasm artifact not found at #{@wasm_path} (GRAPHLAW_REQUIRE_ENGINE=1)"
      )
    end
  end

  defp golden(name), do: File.read!(Path.join(@golden, name))
  defp native(name), do: name |> golden() |> Jason.decode!()

  test "canonical N-Quads and quad count equal native output" do
    native = native("native_canonical.json")
    assert {:ok, result} = API.canonical(data: golden("data.ttl"))
    assert result.nquads == native["nquads"]
    assert result.quads == native["quads"]
  end

  test "BLAKE3 content id equals native output and the minted id.blake3" do
    native = native("native_canonical.json")
    assert {:ok, result} = API.canonical(data: golden("data.ttl"))
    assert result.id == native["id"]
    assert result.id == String.trim(golden("id.blake3"))
  end

  test "canonical id from the engine loaded under the real manifest sha256 pin equals id.blake3" do
    manifest =
      Path.expand("../priv/graphlaw/MANIFEST.json", __DIR__) |> File.read!() |> Jason.decode!()

    pin = manifest["artifact"]["sha256"]
    assert is_binary(pin) and byte_size(pin) == 64
    assert pin == @pin

    # The engine started in setup_all was admitted under this pin (not
    # :unpinned); a wrong pin would have refused the load and failed setup.
    assert {:ok, result} = API.canonical(data: golden("data.ttl"))
    assert result.id == String.trim(golden("id.blake3"))
  end

  test "SPARQL variables and rows equal native output" do
    native = native("native_sparql.json")
    assert {:ok, result} = API.sparql(data: golden("data.ttl"), query: golden("query.rq"))
    assert result.kind == String.to_existing_atom(native["kind"])
    assert result.variables == native["variables"]

    rows = for row <- result.rows, do: Enum.map(row, &%{"type" => &1.type, "value" => &1.value})
    assert rows == native["rows"]
  end
end
