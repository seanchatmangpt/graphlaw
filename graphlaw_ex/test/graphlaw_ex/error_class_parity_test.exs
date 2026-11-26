# SPDX-License-Identifier: MIT
defmodule GraphlawEx.ErrorClassParityTest do
  use ExUnit.Case, async: true

  alias GraphlawEx.Capability.Registry
  alias GraphlawEx.Error.Class
  alias GraphlawEx.Introspection
  alias GraphlawEx.Introspection.Tables

  @guides ["ash_r2rml_migration.md", "refusals_and_errors.md"]

  defp guide_rows(file) do
    ("../../guides/" <> file)
    |> Path.expand(__DIR__)
    |> File.read!()
    |> String.split("\n")
    |> Enum.flat_map(fn line ->
      case Regex.run(~r/^\| `(:?[A-Za-z_]+)` \| `:(forbidden|invalid|unknown)`/, line) do
        [_, key, class] -> [{key, String.to_existing_atom(class)}]
        nil -> []
      end
    end)
  end

  test "Introspection.Tables delegates to the one kind table" do
    for {kind, class} <- Class.kind_table() do
      assert Tables.error_class(kind) == class
      assert Introspection.error_class(kind) == class
    end

    assert Tables.error_class("anything else") == :unknown
  end

  test "every committed registry refusal kind is classified" do
    assert Registry.refusal_kinds() != []
    for kind <- Registry.refusal_kinds(), do: assert(Map.has_key?(Class.kind_table(), kind))
  end

  test "kind table equals registry JSON refusal_kind_classes" do
    json =
      "../../../registry/capability-registry.json"
      |> Path.expand(__DIR__)
      |> File.read!()
      |> Jason.decode!()

    expected =
      for {kind, %{"error_class" => c}} <- json["refusal_kind_classes"],
          into: %{},
          do: {kind, String.to_existing_atom(c)}

    assert map_size(expected) > 0
    assert Class.kind_table() == expected
  end

  test "kind-vs-code class divergences are exactly the documented ones" do
    pairs = [
      {"ResourceLimit", :REFUSED_RESOURCE_BOUND},
      {"Unsupported", :REFUSED_UNSUPPORTED_SPARQL_FEATURE}
    ]

    for {kind, code} <- pairs do
      assert Class.of_kind(kind) != Class.table()[code]
    end

    # Every other kind that maps to a code of the same family must agree.
    assert Class.of_kind("Ambiguous") == Class.table()[:REFUSED_AMBIGUOUS_DIALECT]
  end

  for guide <- @guides do
    test "#{guide} mapping tables equal the code tables" do
      rows = guide_rows(unquote(guide))

      code_rows =
        for {":" <> name, class} <- rows, do: {String.to_atom(name), class}

      kind_rows = for {key, class} <- rows, not String.starts_with?(key, ":"), do: {key, class}

      assert Map.new(code_rows) == Class.table()
      assert length(code_rows) == map_size(Class.table())
      assert Map.new(kind_rows) == Class.kind_table()
      assert length(kind_rows) == map_size(Class.kind_table())
    end
  end
end
