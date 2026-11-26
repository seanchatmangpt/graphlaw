defmodule GraphlawEx.IntrospectionTest do
  use ExUnit.Case, async: true

  alias GraphlawEx.Capability.Registry
  alias GraphlawEx.Introspection

  defp scopes(name) do
    {:ok, info} = Introspection.info(name)
    meta = Registry.limit_meta()
    info.limits |> Map.keys() |> Enum.map(&meta[&1].scope) |> MapSet.new()
  end

  test "n3 owns exactly the n3 scope plus shared scopes" do
    assert scopes("n3") == MapSet.new(["n3", "abi", "wasm"])
  end

  test "an op with no scope of its own sees only shared scopes" do
    assert scopes("parse") == MapSet.new(["abi", "wasm"])
  end

  test "law owns plan, law and duckdb scopes (admit:sql); no other op sees duckdb limits" do
    assert MapSet.subset?(MapSet.new(["law", "plan", "duckdb"]), scopes("law"))
    refute MapSet.member?(scopes("parse"), "duckdb")
    refute MapSet.member?(scopes("sparql"), "duckdb")
  end

  test "every limit lands on at least one op (no limit orphaned)" do
    seen =
      Introspection.capabilities() |> Enum.flat_map(&Map.keys(&1.limits)) |> MapSet.new()

    assert MapSet.new(Map.keys(Registry.limits())) == seen
  end
end
