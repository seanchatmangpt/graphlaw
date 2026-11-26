# SPDX-FileCopyrightText: 2026 Sean Chatman <https://github.com/seanchatmangpt/graphlaw/graphs/contributors>
#
# SPDX-License-Identifier: MIT
defmodule GraphlawEx.Safety.SparqlLimitsTest do
  use ExUnit.Case, async: false

  # Chicago: real wasm engine. Every test is :wasm: excluded loudly when the
  # artifact is absent (test_helper.exs), failing under GRAPHLAW_REQUIRE_ENGINE=1
  # (engine_path!/0 raises in setup_all).
  @moduletag :wasm

  alias GraphlawEx.Capability.Refusal
  alias GraphlawEx.Capability.SparqlQuery

  @nt "<https://x/a> <https://x/p> <https://x/1> .\n<https://x/b> <https://x/p> <https://x/2> .\n"
  @q "SELECT ?s WHERE { ?s ?p ?o }"

  setup_all do
    path = GraphlawEx.TestEngine.engine_path!()
    start_supervised!({GraphlawEx, wasm_path: path, expected_sha256: :unpinned})
    :ok
  end

  for {name, key} <- [
        {"max_rows", :max_rows},
        {"timeout_ms", :timeout_ms},
        {"max_bytes", :max_bytes}
      ],
      bad <- [nil, "5", 0, -1, 1.5] do
    test "#{name}: #{inspect(bad)} refuses :REFUSED_INVALID_INPUT" do
      assert {:error, %Refusal{code: :REFUSED_INVALID_INPUT, details: d}} =
               SparqlQuery.query(@nt, @q, [{unquote(key), unquote(Macro.escape(bad))}])

      assert d["option"] == unquote(name)
    end
  end

  test "non-binary nquads or sparql refuses :REFUSED_INVALID_INPUT" do
    assert {:error, %Refusal{code: :REFUSED_INVALID_INPUT}} = SparqlQuery.query(nil, @q)
    assert {:error, %Refusal{code: :REFUSED_INVALID_INPUT}} = SparqlQuery.query(@nt, :ask)
    assert {:error, %Refusal{code: :REFUSED_INVALID_INPUT}} = SparqlQuery.query(123, 456)
  end

  test "oversized nquads refuse :REFUSED_RESOURCE_BOUND before the engine" do
    assert {:error, %Refusal{code: :REFUSED_RESOURCE_BOUND, details: d}} =
             SparqlQuery.query(@nt, @q, max_bytes: 10)

    assert d["limit"] == "max_bytes"
    assert d["observed"] == byte_size(@nt)
  end

  test "valid limits still admit and max_rows is enforced on the response" do
    assert {:ok, result} =
             SparqlQuery.query(@nt, @q,
               max_rows: 2,
               timeout_ms: 30_000,
               max_bytes: byte_size(@nt)
             )

    assert length(result.rows) == 2

    assert {:error, %Refusal{code: :REFUSED_RESOURCE_BOUND}} =
             SparqlQuery.query(@nt, @q, max_rows: 1)
  end
end
