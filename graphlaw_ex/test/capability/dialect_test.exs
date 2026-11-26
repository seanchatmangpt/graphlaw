# SPDX-FileCopyrightText: 2026 Sean Chatman <https://github.com/seanchatmangpt/graphlaw/graphs/contributors>
#
# SPDX-License-Identifier: MIT
defmodule GraphlawEx.Capability.DialectTest do
  use ExUnit.Case, async: false

  # Hand-written FR-4 dialect router. Pure tier/refusal laws run everywhere;
  # sniff/admit laws run on the real engine (:wasm — excluded loudly when the
  # artifact is absent, failing under GRAPHLAW_REQUIRE_ENGINE=1).

  alias GraphlawEx.Capability.Dialect
  alias GraphlawEx.Capability.Refusal
  alias GraphlawEx.Result.Sniff

  doctest GraphlawEx.Capability.Dialect
  doctest GraphlawEx.Capability.Dialect.Tiers

  @ttl "@prefix ex: <https://x/> .\nex:s ex:p ex:o .\n"
  @n3 "@prefix : <https://x/> .\n{ ?x :p ?y } => { ?y :p ?x } .\n"
  @shapes "@prefix sh: <http://www.w3.org/ns/shacl#> .\n@prefix ex: <https://x/> .\nex:S a sh:NodeShape .\n"

  describe "tier/1 (pure)" do
    test "N3 is the logic tier" do
      assert Dialect.tier("N3") == :logic
    end

    test "known non-N3 dialect is dataset tier" do
      assert Dialect.tier("Turtle") == :dataset
    end

    test "unknown names and non-binaries are :unknown" do
      assert Dialect.tier("Nope") == :unknown
      assert Dialect.tier(nil) == :unknown
      assert Dialect.tier(42) == :unknown
    end
  end

  describe "request validation (pure)" do
    test "admit_shacl without document text refuses :invalid_capability_request" do
      assert {:error, %Refusal{code: :invalid_capability_request}} =
               Dialect.admit_shacl(data: 42, shapes: @shapes)

      assert {:error, %Refusal{code: :invalid_capability_request}} =
               Dialect.admit_shacl(shapes: @shapes)
    end

    test "admit_n3 without text refuses :invalid_capability_request" do
      assert {:error, %Refusal{code: :invalid_capability_request}} = Dialect.admit_n3([])
    end

    test "map args are normalized, not raised on" do
      assert {:error, %Refusal{code: :invalid_capability_request}} = Dialect.admit_n3(%{})

      assert {:error, %Refusal{code: :invalid_capability_request}} =
               Dialect.admit_shacl(%{"data" => 42, "shapes" => @shapes})

      assert {:error, %Refusal{code: :invalid_capability_request}} =
               Dialect.admit_shacl(%{shapes: @shapes})
    end

    test "non keyword/map args and non keyword opts refuse without raising" do
      for bad <- [42, "text", [1, 2], nil] do
        assert {:error, %Refusal{code: :invalid_capability_request}} = Dialect.admit_n3(bad)
        assert {:error, %Refusal{code: :invalid_capability_request}} = Dialect.admit_shacl(bad)
      end

      assert {:error, %Refusal{code: :invalid_capability_request}} =
               Dialect.admit_n3([text: @n3], :nope)

      assert {:error, %Refusal{code: :invalid_capability_request}} =
               Dialect.admit_n3([text: @n3], [1])
    end
  end

  describe "engine-backed routing" do
    # Scoped to THIS describe only (a @moduletag here would also tag the pure
    # tests above). Excluded loudly when the artifact is absent (test_helper.exs);
    # engine_path!/0 raises under GRAPHLAW_REQUIRE_ENGINE=1.
    @describetag :wasm

    setup do
      path = GraphlawEx.TestEngine.engine_path!()
      start_supervised!({GraphlawEx, wasm_path: path, expected_sha256: :unpinned})
      :ok
    end

    test "sniff identifies Turtle as dataset tier" do
      assert {:ok, %Sniff{dialect: "Turtle"} = sniff} = Dialect.sniff(@ttl)
      assert Dialect.tier(sniff.dialect) == :dataset
    end

    test "sniff identifies N3 as logic tier" do
      assert {:ok, %Sniff{dialect: "N3"}} = Dialect.sniff(@n3)
    end

    test "sniff never guesses: unrecognisable text is a typed refusal or a known dialect" do
      case Dialect.sniff("\u0000\u0001 ??? not a document") do
        {:error, %Refusal{code: code}} -> assert is_atom(code)
        {:ok, %Sniff{dialect: d}} -> assert Dialect.tier(d) != :unknown
      end
    end

    test "admit_shacl refuses N3 data with :REFUSED_TIER_MISMATCH" do
      assert {:error, %Refusal{code: :REFUSED_TIER_MISMATCH}} =
               Dialect.admit_shacl(data: @n3, shapes: @shapes)

      assert {:error, %Refusal{code: :REFUSED_TIER_MISMATCH}} =
               Dialect.admit_shacl(data: %{text: @n3}, shapes: @shapes)
    end

    test "admit_n3 refuses dataset-tier text with :REFUSED_TIER_MISMATCH" do
      assert {:error, %Refusal{code: :REFUSED_TIER_MISMATCH}} = Dialect.admit_n3(text: @ttl)
    end
  end
end
