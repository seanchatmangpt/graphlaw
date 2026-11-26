# SPDX-FileCopyrightText: 2026 Sean Chatman <https://github.com/seanchatmangpt/graphlaw/graphs/contributors>
#
# SPDX-License-Identifier: MIT
defmodule GraphlawEx.Safety.GraphSafetyTest do
  use ExUnit.Case, async: false

  alias GraphlawEx.Capability.Graph
  alias GraphlawEx.Capability.Refusal

  @wasm_path System.get_env("PRIV_WASM") ||
               Path.expand("../../priv/graphlaw/graphlaw.wasm", __DIR__)
  @engine? File.regular?(@wasm_path)
  @require_engine System.get_env("GRAPHLAW_REQUIRE_ENGINE") == "1"

  if not @engine? and not @require_engine do
    IO.puts("[SKIP] graph safety court: wasm artifact not found at #{@wasm_path}")
    @moduletag skip: "wasm artifact missing at #{@wasm_path}"
  end

  # Each graph has one distinct blank node with a different property.
  @a "_:x <https://x/p> <https://x/o1> .\n"
  @b "_:x <https://x/q> <https://x/o2> .\n"
  @ground "<https://x/a> <https://x/p> <https://x/b> .\n"

  setup_all do
    if @engine? do
      start_supervised!({GraphlawEx, wasm_path: @wasm_path, expected_sha256: :unpinned})
      :ok
    else
      flunk("graph safety court: wasm artifact not found at #{@wasm_path}")
    end
  end

  @tag :wasm
  test "union keeps distinct blank nodes from a and b apart" do
    assert {:ok, union} = Graph.union(@a, @b)
    # Collapsed (the bug) would be a single subject; a true merge has two.
    subjects =
      union
      |> String.split("\n", trim: true)
      |> Enum.map(&hd(String.split(&1, " ")))
      |> Enum.uniq()

    assert length(subjects) == 2

    collapsed = "_:x <https://x/p> <https://x/o1> .\n_:x <https://x/q> <https://x/o2> .\n"
    assert {:ok, false} = Graph.equivalent?(union, collapsed)
    assert {:ok, true} = Graph.equivalent?(union, @a <> "_:y <https://x/q> <https://x/o2> .\n")
  end

  @tag :wasm
  test "union leaves literals that look like bnode labels untouched" do
    lit = "_:x <https://x/p> \"_:c14n0 \\\" _:c14n1\" .\n"
    assert {:ok, union} = Graph.union(lit, @ground)
    assert String.contains?(union, "\"_:c14n0 \\\" _:c14n1\"")
  end

  @tag :wasm
  test "diff and intersection refuse typed over blank nodes" do
    for fun <- [&Graph.diff/2, &Graph.intersection/2],
        {x, y} <- [{@a, @b}, {@ground, @a}, {@a, @ground}] do
      assert {:error, %Refusal{code: :invalid_capability_request, details: details}} = fun.(x, y)
      assert details["reason"] == "blank_nodes_in_set_algebra"
    end
  end

  @tag :wasm
  test "diff and intersection stay exact on ground graphs" do
    both = @ground <> "<https://x/c> <https://x/p> <https://x/d> .\n"
    assert {:ok, d} = Graph.diff(both, @ground)
    assert {:ok, true} = Graph.equivalent?(d, "<https://x/c> <https://x/p> <https://x/d> .\n")
    assert {:ok, i} = Graph.intersection(both, @ground)
    assert {:ok, true} = Graph.equivalent?(i, @ground)
  end
end
