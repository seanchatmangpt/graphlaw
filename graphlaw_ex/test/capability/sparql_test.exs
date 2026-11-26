# SPDX-FileCopyrightText: 2026 Sean Chatman <https://github.com/seanchatmangpt/graphlaw/graphs/contributors>
#
# SPDX-License-Identifier: MIT
defmodule GraphlawEx.Capability.SparqlTest do
  use ExUnit.Case, async: false

  # Chicago: real wasm engine. Missing artifact skips loudly with a reason
  # (unless GRAPHLAW_REQUIRE_ENGINE=1, which flunks in setup_all). No mocks, no sleeps.

  alias GraphlawEx.Capability.Refusal
  alias GraphlawEx.Capability.Sparql
  alias GraphlawEx.Result.Term

  @wasm_path System.get_env("PRIV_WASM") ||
               Path.expand("../../priv/graphlaw/graphlaw.wasm", __DIR__)
  @require_engine System.get_env("GRAPHLAW_REQUIRE_ENGINE") == "1"

  @wasm_skip if(File.regular?(@wasm_path) or @require_engine,
               do: false,
               else: "sparql court: wasm artifact not found at #{@wasm_path} (set PRIV_WASM)"
             )

  @nt """
  <https://x/a> <https://x/p> <https://x/1> .
  <https://x/b> <https://x/p> <https://x/2> .
  <https://x/c> <https://x/p> <https://x/3> .
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
  test "query/3 returns all solution rows as Term cells" do
    assert {:ok, result} =
             Sparql.query(@nt, "SELECT ?s WHERE { ?s <https://x/p> ?o } ORDER BY ?s")

    assert result.kind == :solutions
    assert result.variables == ["s"]

    assert Enum.map(result.rows, fn [%Term{value: v}] -> v end) ==
             ["https://x/a", "https://x/b", "https://x/c"]
  end

  @tag :wasm
  @tag skip: @wasm_skip
  test "max_rows at the exact row count admits; one below refuses :REFUSED_RESOURCE_BOUND" do
    q = "SELECT ?s WHERE { ?s ?p ?o }"
    assert {:ok, result} = Sparql.query(@nt, q, max_rows: 3)
    assert length(result.rows) == 3

    assert {:error, %Refusal{code: :REFUSED_RESOURCE_BOUND, details: details}} =
             Sparql.query(@nt, q, max_rows: 2)

    assert details["max"] == 2
    assert details["observed"] == 3
  end

  @tag :wasm
  @tag skip: @wasm_skip
  test "timeout_ms generous enough to finish does not change the answer" do
    assert {:ok, result} = Sparql.query(@nt, "ASK { ?s ?p ?o }", timeout_ms: 30_000)
    assert result.value == true
  end

  @tag :wasm
  @tag skip: @wasm_skip
  test "malformed SPARQL refuses typed, never raises" do
    assert {:error, %Refusal{} = refusal} = Sparql.query(@nt, "SELECT WHERE {{{")
    assert is_atom(refusal.code)
  end

  @tag :wasm
  @tag skip: @wasm_skip
  test "non-RDF data refuses typed" do
    assert {:error, %Refusal{code: code}} =
             Sparql.query("definitely not rdf", "ASK { ?s ?p ?o }")

    assert is_atom(code)
  end

  test "build_request refuses a missing query before the engine is called" do
    assert {:error, %Refusal{code: :invalid_capability_request}} = Sparql.build_request(data: @nt)
  end
end
