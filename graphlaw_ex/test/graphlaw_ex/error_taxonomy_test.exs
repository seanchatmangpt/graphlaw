# SPDX-License-Identifier: MIT
defmodule GraphlawEx.ErrorTaxonomyTest do
  use ExUnit.Case, async: true

  alias GraphlawEx.Error
  alias GraphlawEx.Error.Class
  alias GraphlawEx.Refusal

  describe "Class.of/1 refinements" do
    test "an unclassified non-refusal term is :unknown" do
      assert Class.of({:nif_error, :boom}) == :unknown
      assert Class.of(Error.wrap(%{"anything" => 1})) == :unknown
    end

    test "operational host failures are :unknown although their public code is not" do
      saturated = Error.wrap(Refusal.new(:saturated))
      assert saturated.code == :REFUSED_RESOURCE_BOUND
      assert Class.of(saturated) == :unknown

      duck = Error.wrap(Refusal.new(:duckdb_failure))
      assert Class.of(duck) == :unknown

      assert Class.of(Refusal.new(:engine_unclassified)) == :unknown
    end

    test "caller-fixable size caps stay :invalid" do
      assert Class.of(Refusal.new(:resource_limit)) == :invalid
    end
  end

  describe "Error.exception/1 normalization" do
    test "binary becomes detail under REFUSED_INVALID_INPUT" do
      e = Error.exception("nope")
      assert {e.code, e.detail} == {:REFUSED_INVALID_INPUT, "nope"}
    end

    test "keyword: unknown code is normalized into the closed set, bad fields defaulted" do
      e = Error.exception(code: :bogus, detail: 5, evidence: :not_a_map, class: "str")
      assert e.code == :REFUSED_INVALID_INPUT
      assert e.detail == ""
      assert e.evidence == %{}
      assert e.class == nil
    end

    test "keyword :message is accepted as detail" do
      assert Error.exception(message: "m").detail == "m"
    end

    test "existing error is returned as is" do
      e = %Error{code: :REFUSED_WASM_TRAP, detail: "x"}
      assert Error.exception(e) == e
    end

    test "empty list, non-keyword list and arbitrary terms are wrapped, not raised" do
      for term <- [[], [1, 2], :atom, 42, {:a, :b}] do
        e = Error.exception(term)
        assert e.code == :REFUSED_INVALID_INPUT
        assert e.detail == "unclassified failure"
        assert is_binary(e.evidence["term_type"])
      end
    end
  end

  describe "Error.combine/1 precedence" do
    setup do
      %{
        forbidden: Refusal.new(:lease_revoked),
        invalid: Refusal.new(:invalid_json),
        unknown: {:nif_error, :boom}
      }
    end

    test "forbidden beats invalid beats unknown regardless of position", ctx do
      e = Error.combine([ctx.unknown, ctx.invalid, ctx.forbidden])
      assert e.code == :REFUSED_LEASE_REQUIRED
      assert e.evidence["category"] == "forbidden"
      assert length(e.evidence["errors"]) == 3

      e = Error.combine([ctx.unknown, ctx.invalid])
      assert e.code == :REFUSED_INVALID_INPUT
      assert e.evidence["category"] == "invalid"

      e = Error.combine([ctx.unknown])
      assert e.evidence["category"] == "unknown"
    end

    test "children keep input order with categories", ctx do
      e = Error.combine([{["a"], ctx.unknown}, {["b"], ctx.forbidden}])
      assert Enum.map(e.evidence["errors"], & &1["category"]) == ["unknown", "forbidden"]
      assert Enum.map(e.evidence["errors"], & &1["path"]) == [["a"], ["b"]]
    end

    test "empty list raises ArgumentError" do
      assert_raise ArgumentError, fn -> Error.combine([]) end
    end
  end
end
