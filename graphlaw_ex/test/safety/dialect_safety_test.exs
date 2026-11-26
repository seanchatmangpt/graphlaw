# SPDX-FileCopyrightText: 2026 Sean Chatman <https://github.com/seanchatmangpt/graphlaw/graphs/contributors>
#
# SPDX-License-Identifier: MIT
defmodule GraphlawEx.Safety.DialectSafetyTest do
  use ExUnit.Case, async: false

  # Lane S2: closed tier mapping; query/schema/sql text is never admitted as data.

  alias GraphlawEx.Capability.Dialect
  alias GraphlawEx.Capability.Refusal
  alias GraphlawEx.Result.Shacl, as: ShaclResult

  @wasm_path System.get_env("PRIV_WASM") ||
               Path.expand("../../priv/graphlaw/graphlaw.wasm", __DIR__)

  @ttl "@prefix ex: <https://x/> .\nex:s ex:p ex:o .\n"
  @shapes "@prefix sh: <http://www.w3.org/ns/shacl#> .\n@prefix ex: <https://x/> .\nex:S a sh:NodeShape .\n"
  @sparql "SELECT ?s ?p ?o WHERE { ?s ?p ?o }"

  @registry_path Path.expand("../../../registry/capability-registry.json", __DIR__)
  @external_resource @registry_path
  @registry @registry_path |> File.read!() |> Jason.decode!()
  @rdf_dialects Enum.map(@registry["rdf_dialects"], & &1["response_name"])
  @other_dialects Enum.map(@registry["other_dialects"], & &1["response_name"])

  describe "tier/1 closed mapping (pure, registry-derived)" do
    test "registry dialect lists are non-empty" do
      assert @rdf_dialects != []
      assert @other_dialects != []
    end

    test "every registry dialect classifies to a known tier" do
      for d <- @rdf_dialects ++ @other_dialects do
        assert Dialect.tier(d) in [:dataset, :logic, :query, :sql, :schema], d
      end
    end

    test "dataset tier is exactly the registry's RDF serializations" do
      for d <- @rdf_dialects, do: assert(Dialect.tier(d) == :dataset, d)
      for d <- @other_dialects, do: refute(Dialect.tier(d) == :dataset, d)
    end

    test "S2: sparql and sql-rdf are never dataset tier" do
      for d <- ["Sparql", "SqlRdf"] do
        assert d in @other_dialects, "#{d} missing from registry"
        refute Dialect.tier(d) == :dataset, d
      end

      assert Dialect.tier("Sparql") == :query
      assert Dialect.tier("SqlRdf") == :sql
    end

    test "unknown stays unknown" do
      assert Dialect.tier("turtle") == :unknown
      assert Dialect.tier(:Turtle) == :unknown
      assert Dialect.tier(nil) == :unknown
    end
  end

  describe "engine-backed" do
    setup do
      if File.regular?(@wasm_path) do
        start_supervised!({GraphlawEx, wasm_path: @wasm_path, expected_sha256: :unpinned})
        :ok
      else
        flunk("dialect safety court: wasm artifact not found at #{@wasm_path}")
      end
    end

    @tag :wasm
    test "SPARQL text is never admitted as SHACL data" do
      assert {:error, %Refusal{code: code}} = Dialect.admit_shacl(data: @sparql, shapes: @shapes)
      assert code in [:REFUSED_TIER_MISMATCH, :REFUSED_AMBIGUOUS_DIALECT]
    end

    @tag :wasm
    test "SPARQL text is never admitted as N3" do
      assert {:error, %Refusal{code: code}} = Dialect.admit_n3(text: @sparql)
      assert code in [:REFUSED_TIER_MISMATCH, :REFUSED_AMBIGUOUS_DIALECT]
    end

    @tag :wasm
    test "a :hint option does not leak into the shacl op" do
      assert {:ok, %ShaclResult{}} =
               Dialect.admit_shacl([data: @ttl, shapes: @shapes], hint: "Turtle")
    end
  end
end
