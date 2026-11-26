# SPDX-FileCopyrightText: 2026 Sean Chatman <https://github.com/seanchatmangpt/graphlaw/graphs/contributors>
#
# SPDX-License-Identifier: MIT
defmodule GraphlawEx.Capability.ShaclTest do
  use ExUnit.Case, async: false

  alias GraphlawEx.Capability.Refusal
  alias GraphlawEx.Capability.Shacl

  @wasm_path System.get_env("PRIV_WASM") ||
               Path.expand("../../priv/graphlaw/graphlaw.wasm", __DIR__)
  @require_engine System.get_env("GRAPHLAW_REQUIRE_ENGINE") == "1"

  # Missing artifact: engine tests skip loudly with a reason, or setup_all
  # flunks under GRAPHLAW_REQUIRE_ENGINE=1.
  @wasm_skip if(File.regular?(@wasm_path) or @require_engine,
               do: false,
               else: "shacl court: wasm artifact not found at #{@wasm_path} (set PRIV_WASM)"
             )

  @shapes """
  @prefix sh: <http://www.w3.org/ns/shacl#> .
  @prefix ex: <https://x/> .
  ex:PS a sh:NodeShape ;
    sh:targetClass ex:Person ;
    sh:property [ sh:path ex:name ; sh:minCount 1 ] .
  """

  setup_all do
    if File.regular?(@wasm_path) do
      start_supervised!({GraphlawEx, wasm_path: @wasm_path, expected_sha256: :unpinned})
      :ok
    else
      if @require_engine do
        flunk("GRAPHLAW_REQUIRE_ENGINE=1: wasm artifact not found at #{@wasm_path}")
      end

      :ok
    end
  end

  @tag :wasm
  @tag skip: @wasm_skip
  test "conforming data conforms with no results" do
    data = """
    @prefix ex: <https://x/> .
    ex:alice a ex:Person ; ex:name "Alice" .
    """

    assert {:ok, result} = Shacl.run(data: data, shapes: @shapes)
    assert result.conforms == true
    assert result.results == []
  end

  @tag :wasm
  @tag skip: @wasm_skip
  test "violating data does not conform and reports a result" do
    data = """
    @prefix ex: <https://x/> .
    ex:bob a ex:Person .
    """

    assert {:ok, result} = Shacl.run(data: data, shapes: @shapes)
    assert result.conforms == false
    assert length(result.results) >= 1
  end

  @tag :wasm
  @tag skip: @wasm_skip
  test "unparseable shapes refuse typed" do
    assert {:error, %Refusal{code: code}} =
             Shacl.run(
               data: "<https://x/s> <https://x/p> <https://x/o> .",
               shapes: "@@@ not turtle"
             )

    assert is_atom(code)
  end

  test "missing shapes refuses :invalid_capability_request" do
    assert {:error, %Refusal{code: :invalid_capability_request}} = Shacl.build_request(data: "x")
  end
end
