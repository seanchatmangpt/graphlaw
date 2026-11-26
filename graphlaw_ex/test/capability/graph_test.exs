# SPDX-FileCopyrightText: 2026 Sean Chatman <https://github.com/seanchatmangpt/graphlaw/graphs/contributors>
#
# SPDX-License-Identifier: MIT
defmodule GraphlawEx.Capability.GraphTest do
  use ExUnit.Case, async: false

  # Chicago: the real engine canonicalizes. :wasm — excluded loudly when the
  # artifact is absent (test_helper.exs), failing under GRAPHLAW_REQUIRE_ENGINE=1
  # (engine_path!/0 raises in setup_all).
  @moduletag :wasm
  @moduletag timeout: 300_000

  alias GraphlawEx.Capability.Graph
  alias GraphlawEx.Capability.Refusal

  @ab "<https://x/a> <https://x/p> <https://x/b> .\n"
  @cd "<https://x/c> <https://x/p> <https://x/d> .\n"
  @bn "_:x <https://x/p> \"v\" .\n"
  @bn_other "_:y <https://x/p> \"w\" .\n"
  # A ground graph whose literal merely LOOKS like a blank-node label.
  @lit_bnode "<https://x/a> <https://x/p> \"_:x is only text\" .\n"

  setup_all do
    path = GraphlawEx.TestEngine.engine_path!()
    start_supervised!({GraphlawEx, wasm_path: path, expected_sha256: :unpinned})
    :ok
  end

  test "digest is blake3-prefixed and stable across statement order" do
    assert {:ok, d1} = Graph.digest(@ab <> @cd)
    assert {:ok, d2} = Graph.digest(@cd <> @ab)
    assert d1 == d2
    assert String.starts_with?(d1, "blake3:")
  end

  test "equivalent?/2 true for reordered, false for different graphs" do
    assert {:ok, true} = Graph.equivalent?(@ab <> @cd, @cd <> @ab)
    assert {:ok, false} = Graph.equivalent?(@ab, @cd)
  end

  test "union, diff, intersection compute set algebra over canonical quads" do
    both = @ab <> @cd

    assert {:ok, union} = Graph.union(@ab, @cd)
    assert {:ok, true} = Graph.equivalent?(union, both)

    assert {:ok, diff} = Graph.diff(both, @cd)
    assert {:ok, true} = Graph.equivalent?(diff, @ab)

    assert {:ok, inter} = Graph.intersection(both, @cd)
    assert {:ok, true} = Graph.equivalent?(inter, @cd)

    assert {:ok, ""} = Graph.intersection(@ab, @cd)
    assert {:ok, ""} = Graph.diff(@ab, @ab)
  end

  test "non-RDF input refuses typed through the algebra" do
    assert {:error, %Refusal{code: code}} = Graph.digest("not rdf at all")
    assert is_atom(code)
    assert {:error, %Refusal{}} = Graph.union(@ab, "not rdf at all")
  end

  describe "blank nodes" do
    test "union is an RDF merge: a bnode graph unioned with itself is two disjoint copies" do
      assert {:ok, merged} = Graph.union(@bn, @bn)

      assert length(quad_lines(merged)) == 2

      # Not collapsed back to the single-quad original.
      assert {:ok, false} = Graph.equivalent?(merged, @bn)
      assert {:ok, single} = Graph.canonicalize(@bn)
      assert length(quad_lines(single)) == 1
    end

    test "union renames blank nodes apart: same label in both inputs stays two nodes" do
      assert {:ok, merged} = Graph.union(@bn, @bn_other)
      assert length(quad_lines(merged)) == 2
      assert {:ok, canon} = Graph.canonicalize(merged)
      assert canon == merged

      # Ground quads are untouched by the renaming.
      assert {:ok, with_ground} = Graph.union(@bn <> @ab, @cd)
      assert length(quad_lines(with_ground)) == 3
      assert String.contains?(with_ground, "<https://x/a> <https://x/p> <https://x/b>")
    end

    test "union leaves quoted literals that look like bnode labels verbatim" do
      assert {:ok, merged} = Graph.union(@lit_bnode, @cd)
      assert String.contains?(merged, "\"_:x is only text\"")
      assert length(quad_lines(merged)) == 2
    end

    test "diff refuses when either input contains a blank node" do
      for {a, b} <- [{@bn, @ab}, {@ab, @bn}, {@bn, @bn}, {@bn <> @ab, @bn_other}] do
        assert {:error, %Refusal{code: :invalid_capability_request, details: details}} =
                 Graph.diff(a, b)

        assert details["reason"] == "blank_nodes_in_set_algebra"
      end
    end

    test "intersection refuses when either input contains a blank node" do
      for {a, b} <- [{@bn, @ab}, {@ab, @bn}, {@bn, @bn}, {@bn <> @ab, @bn_other}] do
        assert {:error, %Refusal{code: :invalid_capability_request, details: details}} =
                 Graph.intersection(a, b)

        assert details["reason"] == "blank_nodes_in_set_algebra"
      end
    end

    test "a literal that merely looks like a blank node does not trigger the refusal" do
      assert {:ok, diff} = Graph.diff(@lit_bnode <> @cd, @cd)
      assert {:ok, true} = Graph.equivalent?(diff, @lit_bnode)

      assert {:ok, inter} = Graph.intersection(@lit_bnode <> @cd, @lit_bnode)
      assert {:ok, true} = Graph.equivalent?(inter, @lit_bnode)
    end

    test "equivalent? is exact over blank nodes: isomorphic graphs match, different ones do not" do
      assert {:ok, true} = Graph.equivalent?(@bn, "_:zzz <https://x/p> \"v\" .\n")
      assert {:ok, false} = Graph.equivalent?(@bn, @bn_other)
    end
  end

  defp quad_lines(nquads), do: String.split(nquads, "\n", trim: true)
end
