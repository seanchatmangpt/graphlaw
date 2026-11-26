# SPDX-FileCopyrightText: 2026 Sean Chatman <https://github.com/seanchatmangpt/graphlaw/graphs/contributors>
#
# SPDX-License-Identifier: MIT
defmodule GraphlawEx.Capability.ErrorCoverageTest do
  use ExUnit.Case, async: false

  # Chicago: real wasm engine, assertions through GraphlawEx.Error.wrap/1.
  # A missing artifact flunks under GRAPHLAW_REQUIRE_ENGINE=1, else skips loudly.
  # No mocks, no sleeps.

  alias GraphlawEx.Capability.Dialect
  alias GraphlawEx.Capability.Refusal
  alias GraphlawEx.Capability.Sparql
  alias GraphlawEx.Error
  alias GraphlawEx.Error.Class

  @wasm_path System.get_env("PRIV_WASM") ||
               Path.expand("../../priv/graphlaw/graphlaw.wasm", __DIR__)
  @require_engine System.get_env("GRAPHLAW_REQUIRE_ENGINE") == "1"

  @wasm_skip if(File.regular?(@wasm_path) or @require_engine,
               do: false,
               else:
                 "error coverage court: wasm artifact not found at #{@wasm_path} (set PRIV_WASM)"
             )

  @nt "<https://x/a> <https://x/p> <https://x/1> .\n"

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

  # Real trigger: src/dialect/sniff.rs routes a brace-delimited block with no
  # `trig`/`shex` hint to RefusalKind::Ambiguous ("supply a `trig` or `shex` hint").
  @tag :wasm
  @tag skip: @wasm_skip
  test "brace block without hint wraps to :REFUSED_AMBIGUOUS_DIALECT / :invalid" do
    text = "{ <https://x/a> <https://x/p> <https://x/1> . }"

    assert {:error, %Refusal{} = refusal} = Dialect.sniff(text)
    error = Error.wrap(refusal)

    assert error.code == :REFUSED_AMBIGUOUS_DIALECT
    assert Class.of(error) == :invalid
    assert Error.wrap({:error, refusal}).code == :REFUSED_AMBIGUOUS_DIALECT
  end

  # Real trigger: src/dialect/sniff.rs route_sql_rdf refuses a `sql:` predicate
  # usage with no sql:*Statement type assertion.
  @tag :wasm
  @tag skip: @wasm_skip
  test "sql: vocabulary without statement class wraps to :REFUSED_AMBIGUOUS_DIALECT" do
    text = """
    @prefix sql: <http://graphlaw.org/sql#> .
    <https://x/q> sql:text "select 1" .
    """

    assert {:error, %Refusal{} = refusal} = Dialect.sniff(text)
    assert Error.wrap(refusal).code == :REFUSED_AMBIGUOUS_DIALECT
  end

  # No engine-side trigger for a SPARQL feature refusal was found in
  # src/abi/ops/query_engines.rs or src/law.rs (the engine's `Unsupported` kind
  # covers N3/hook/plan cases, not SPARQL syntax). Closest real inputs: SPARQL
  # Update and federated SERVICE sent to the query op. Strict contract: each is
  # a typed refusal, wraps into the :invalid class, and if the engine message
  # names an unsupported feature the code MUST be :REFUSED_UNSUPPORTED_SPARQL_FEATURE.
  @tag :wasm
  @tag skip: @wasm_skip
  test "SPARQL features outside the query op refuse typed and wrap :invalid" do
    candidates = [
      "INSERT DATA { <https://x/s> <https://x/p> <https://x/o> }",
      "SELECT ?s WHERE { SERVICE <https://example.invalid/sparql> { ?s ?p ?o } }"
    ]

    for q <- candidates do
      case Sparql.query(@nt, q) do
        {:error, %Refusal{} = refusal} ->
          error = Error.wrap(refusal)
          assert error.code in [:REFUSED_UNSUPPORTED_SPARQL_FEATURE, :REFUSED_INVALID_INPUT]
          assert Class.of(error) == :invalid

          if String.match?(refusal.message || "", ~r/unsupported|not supported|not implemented/i) do
            assert error.code == :REFUSED_UNSUPPORTED_SPARQL_FEATURE
          end

        {:ok, _} ->
          # Engine admitted it (e.g. SERVICE evaluates to empty): not a refusal
          # trigger; the other candidate carries the assertion.
          :ok
      end
    end

    assert match?({:error, %Refusal{}}, Sparql.query(@nt, hd(candidates)))
  end
end
