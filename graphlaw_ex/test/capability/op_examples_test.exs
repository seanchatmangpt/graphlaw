# SPDX-FileCopyrightText: 2026 Sean Chatman <https://github.com/seanchatmangpt/graphlaw/graphs/contributors>
#
# SPDX-License-Identifier: MIT
defmodule GraphlawEx.Capability.OpExamplesTest do
  @moduledoc """
  Executes every example of `registry/op-examples.json`
  (`graphlaw.op-examples/1`) through `GraphlawEx.Capability.API.run/3` on the
  real engine: one generated test per example id, so the registry file is the
  only place the example count lives.
  """

  use GraphlawEx.ExampleCase, async: false

  alias GraphlawEx.Capability.Registry

  @examples_path Path.expand("../../../registry/op-examples.json", __DIR__)
  @external_resource @examples_path

  @document @examples_path |> File.read!() |> Jason.decode!()
  @examples Map.fetch!(@document, "examples")
  @skip skip_reason()
  @rdf_dialects Registry.rdf_dialects()

  # -- one test per example id ------------------------------------------------

  for example <- @examples do
    @tag :wasm
    @tag skip: @skip
    test "example #{example["id"]}" do
      example = unquote(Macro.escape(example))
      result = execute(example)

      case example["outcome"] do
        "ok" ->
          assert {:ok, ok} = result, "#{example["id"]} expected ok, got #{inspect(result)}"
          assert_ok_shape(example, ok)

        "refused" ->
          if Map.has_key?(divergences(), example["id"]) do
            assert_divergence(example, result)
          else
            assert_refusal(example, result)
          end
      end
    end
  end

  test "guard: every registered divergence names a refused registry example" do
    refused = for e <- @examples, e["outcome"] == "refused", do: e["id"]
    assert Map.keys(divergences()) -- refused == []
  end

  # -- per op: a positive control executes ---------------------------------------

  for op_name <- Registry.names() do
    controls =
      for e <- @examples, e["op"] == op_name, e["positive_control"] == true, do: e

    @tag :wasm
    @tag skip: @skip
    test "positive control executes for op #{op_name}" do
      controls = unquote(Macro.escape(controls))
      assert controls != [], "op #{unquote(op_name)} has no positive_control example"

      for example <- controls do
        assert example["outcome"] == "ok"
        assert {:ok, ok} = execute(example)
        assert_ok_shape(example, ok)
      end
    end
  end

  # -- round trips: id/blake3 equivalence ------------------------------------------

  for example <- @examples,
      example["outcome"] == "ok",
      example["op"] in ["parse", "convert", "canonical"],
      example["op"] != "parse" or
        is_nil(example["request"]["dialect"]) or
        example["request"]["dialect"] in @rdf_dialects do
    @tag :wasm
    @tag skip: @skip
    test "roundtrip #{example["id"]}" do
      example = unquote(Macro.escape(example))
      request = example["request"]
      assert {:ok, ok} = execute(example)

      case example["op"] do
        "parse" ->
          assert String.starts_with?(ok.id, "blake3:")
          assert canonical_id(source_spec(request)) == ok.id

        "convert" ->
          assert String.starts_with?(ok.id, "blake3:")
          assert canonical_id(source_spec(request)) == ok.id
          assert canonical_id(%{"text" => ok.text, "dialect" => request["to"]}) == ok.id

        "canonical" ->
          assert String.starts_with?(ok.id, "blake3:")
          assert canonical_id(%{"text" => ok.nquads, "dialect" => "nquads"}) == ok.id
      end
    end
  end

  defp source_spec(%{"data" => data}) when is_map(data), do: data
  defp source_spec(%{"data" => data}) when is_binary(data), do: %{"text" => data}
  defp source_spec(request), do: Map.take(request, ["text", "dialect", "hint", "base"])

  # -- guards (no engine needed) -------------------------------------------------------

  test "guard: document schema and counts are self-consistent" do
    assert @document["schema"] == "graphlaw.op-examples/1"
    assert @document["abi_version"] == Registry.abi_version()

    ids = Enum.map(@examples, & &1["id"])
    assert ids == Enum.uniq(ids)
    assert @examples != []

    outcomes = Enum.frequencies_by(@examples, & &1["outcome"])
    assert Map.keys(outcomes) -- ["ok", "refused"] == []
    assert Enum.sum(Map.values(outcomes)) == length(@examples)

    for e <- @examples do
      assert {:ok, _op} = Registry.op(e["op"]), "#{e["id"]} names unknown op #{e["op"]}"
      assert e["request"]["op"] == e["op"]
    end
  end

  test "guard: generated test count equals the example count" do
    generated =
      for {name, 1} <- __MODULE__.__info__(:functions),
          String.starts_with?(Atom.to_string(name), "test example "),
          do: name

    expected = for e <- @examples, do: :"test example #{e["id"]}"

    assert length(generated) == length(@examples)
    assert Enum.sort(generated) == Enum.sort(expected)
  end

  test "guard: every registry op has a positive_control example" do
    missing =
      for op_name <- Registry.names(),
          not Enum.any?(@examples, &(&1["op"] == op_name and &1["positive_control"] == true)),
          do: op_name

    assert missing == []
  end
end
