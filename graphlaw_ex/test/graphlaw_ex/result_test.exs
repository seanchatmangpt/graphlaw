# SPDX-FileCopyrightText: 2026 Sean Chatman <https://github.com/seanchatmangpt/graphlaw/graphs/contributors>
#
# SPDX-License-Identifier: MIT
defmodule GraphlawEx.ResultTest do
  use ExUnit.Case, async: true

  alias GraphlawEx.Refusal
  alias GraphlawEx.Result
  alias GraphlawEx.Result.{Admitted, Canonical, Receipt, Sparql}

  doctest GraphlawEx.Result

  @hex String.duplicate("ab", 32)
  @hex2 String.duplicate("01", 32)

  defp uri(v), do: %{"type" => "uri", "value" => v}

  defp assert_malformed({:error, %Refusal{} = r}, fragment) do
    assert r.code == :malformed_response
    assert r.class == :refused_structure
    assert r.message =~ fragment
    assert r.details["reason"] == r.message
  end

  describe "Sparql" do
    test "solutions become ordered bindings with unbound cells as nil" do
      s = %Sparql{variables: ["x", "y"], rows: [[uri("a"), nil], [uri("b"), uri("c")]]}
      assert {:ok, %Result{} = r} = Result.from_decoded(s)

      assert Enum.map(r.bindings, &Map.keys/1) == [["x", "y"], ["x", "y"]]

      assert [%{"x" => %{value: "a"}, "y" => nil}, %{"x" => %{value: "b"}, "y" => %{value: "c"}}] =
               r.bindings

      assert r.quads == []
      assert r.receipt_blake3 == nil
      assert r.stats == %{rows: 2, quads: 0, receipts: 0}
    end

    test "empty struct is an empty result" do
      assert {:ok, r} = Result.from_decoded(%Sparql{})
      assert r.bindings == [] and r.quads == [] and r.stats == %{rows: 0, quads: 0, receipts: 0}
    end

    test "variables without rows is empty" do
      assert {:ok, %Result{bindings: []}} = Result.from_decoded(%Sparql{variables: ["x"]})
    end

    test "graph result: blank lines dropped, declared count honoured" do
      s = %Sparql{nquads: "<a> <b> <c> .\n\n   \n<d> <e> <f> .\n", quads: 2}
      assert {:ok, r} = Result.from_decoded(s)
      assert r.quads == ["<a> <b> <c> .", "<d> <e> <f> ."]
      assert r.stats.quads == 2
    end

    test "rows without variables" do
      assert_malformed(Result.from_decoded(%Sparql{rows: [[]]}), "rows without variables")
    end

    test "non-string variables" do
      assert_malformed(Result.from_decoded(%Sparql{variables: [1], rows: []}), "must be strings")
    end

    test "non-list variables or rows" do
      assert_malformed(Result.from_decoded(%Sparql{variables: "x", rows: []}), "must be lists")
      assert_malformed(Result.from_decoded(%Sparql{variables: ["x"], rows: %{}}), "must be lists")
    end

    test "ragged rows, short and long, and non-list rows" do
      assert_malformed(
        Result.from_decoded(%Sparql{variables: ["x", "y"], rows: [[nil]]}),
        "row width does not match 2 variables"
      )

      assert_malformed(
        Result.from_decoded(%Sparql{variables: ["x"], rows: [[nil], [nil, nil]]}),
        "row width does not match 1 variables"
      )

      assert_malformed(
        Result.from_decoded(%Sparql{variables: ["x"], rows: ["nope"]}),
        "row width"
      )
    end

    test "declared quad count disagreeing with nquads" do
      assert_malformed(
        Result.from_decoded(%Sparql{nquads: "<a> <b> <c> .\n", quads: 2}),
        "declared 2 quads, found 1"
      )
    end

    test "quad count that is not an integer" do
      assert_malformed(Result.from_decoded(%Sparql{quads: "2"}), "not an integer")
    end
  end

  describe "Canonical" do
    test "valid id and quads" do
      c = %Canonical{id: @hex, nquads: "<a> <b> <c> .\n", quads: 1}
      assert {:ok, r} = Result.from_decoded(c)
      assert r.receipt_blake3 == @hex
      assert r.bindings == []
      assert r.quads == ["<a> <b> <c> ."]
      assert r.stats == %{rows: 0, quads: 1, receipts: 0}
    end

    test "missing id is allowed and yields nil identity" do
      assert {:ok, %Result{receipt_blake3: nil}} = Result.from_decoded(%Canonical{})
    end

    test "uppercase, short, long and non-hex digests are refused" do
      for bad <- [
            String.upcase(@hex),
            "abc",
            @hex <> "0",
            String.duplicate("g", 64),
            @hex <> "\n"
          ] do
        assert_malformed(Result.from_decoded(%Canonical{id: bad}), "64 lowercase hex")
      end
    end

    test "non-string id" do
      assert_malformed(Result.from_decoded(%Canonical{id: 123}), "not a string")
    end

    test "quad count mismatch" do
      assert_malformed(
        Result.from_decoded(%Canonical{id: @hex, nquads: "", quads: 3}),
        "declared 3 quads, found 0"
      )
    end

    test "quad count refusal precedes digest refusal" do
      assert_malformed(Result.from_decoded(%Canonical{id: "bad", quads: 1}), "declared 1 quads")
    end
  end

  describe "Admitted" do
    test "identity is the last receipt's plan digest; receipts counted" do
      a = %Admitted{
        states: [],
        receipts: [%Receipt{plan_blake3: @hex2}, %Receipt{plan_blake3: @hex}],
        nquads: "<a> <b> <c> .\n"
      }

      assert {:ok, r} = Result.from_decoded(a)
      assert r.receipt_blake3 == @hex
      assert r.quads == ["<a> <b> <c> ."]
      assert r.stats == %{rows: 0, quads: 1, receipts: 2}
    end

    test "no receipts yields nil identity" do
      assert {:ok, r} = Result.from_decoded(%Admitted{states: [], receipts: [], nquads: ""})
      assert r.receipt_blake3 == nil
      assert r.stats.receipts == 0
    end

    test "last receipt without a plan digest yields nil identity" do
      a = %Admitted{states: [], receipts: [%Receipt{plan_blake3: @hex}, %Receipt{}], nquads: ""}
      assert {:ok, %Result{receipt_blake3: nil, stats: %{receipts: 2}}} = Result.from_decoded(a)
    end

    test "malformed last digest is refused, earlier digests are not inspected" do
      bad = %Admitted{states: [], receipts: [%Receipt{plan_blake3: "xyz"}], nquads: ""}
      assert_malformed(Result.from_decoded(bad), "64 lowercase hex")

      ok = %Admitted{
        states: [],
        receipts: [%Receipt{plan_blake3: "xyz"}, %Receipt{plan_blake3: @hex}],
        nquads: ""
      }

      assert {:ok, %Result{receipt_blake3: @hex}} = Result.from_decoded(ok)
    end

    test "decoded from an engine map end to end" do
      a =
        Admitted.from_map(%{
          "states" => [],
          "receipts" => [%{"step" => "s", "plan_blake3" => @hex}],
          "nquads" => "<a> <b> <c> .\n"
        })

      assert {:ok, %Result{receipt_blake3: @hex, stats: %{receipts: 1, quads: 1}}} =
               Result.from_decoded(a)
    end
  end

  describe "foreign input" do
    test "anything else is a malformed_response, never a raise" do
      for term <- [nil, :x, 1, "s", [], %{}, %{"variables" => []}, {:ok, %Sparql{}}, %Receipt{}] do
        assert {:error, %Refusal{code: :malformed_response} = r} = Result.from_decoded(term)
        assert r.message == "not a decodable result struct"
        assert is_binary(r.details["type"])
        assert is_integer(r.details["bytes"])
        refute Map.has_key?(r.details, "value")
      end
    end

    test "value rendering is bounded" do
      {:error, r} = Result.from_decoded(String.duplicate("z", 10_000))
      refute Map.has_key?(r.details, "value")
      assert r.details["type"] == "binary"
    end
  end
end
