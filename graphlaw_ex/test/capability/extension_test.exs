# SPDX-FileCopyrightText: 2026 Sean Chatman <https://github.com/seanchatmangpt/graphlaw/graphs/contributors>
#
# SPDX-License-Identifier: MIT
defmodule GraphlawEx.Capability.ExtensionTest do
  use ExUnit.Case, async: true

  defmodule ToyTarget do
    def admit(data, shapes, opts \\ []), do: {:ok, {data, shapes, opts}}
  end

  defp compile!(body) do
    n = System.unique_integer([:positive])

    Code.compile_string("""
    defmodule GraphlawEx.Capability.ExtensionTest.Toy#{n} do
      @type t :: term()
      #{body}
    end
    """)

    Module.concat(GraphlawEx.Capability.ExtensionTest, "Toy#{n}")
  end

  test "literal override delegates, including defaulted trailing arg" do
    toy =
      compile!("""
      use GraphlawEx.Capability.Extension,
        op: "shacl",
        overrides: [
          %{
            name: :admit,
            target: GraphlawEx.Capability.ExtensionTest.ToyTarget,
            doc: "Routed admission.",
            args: [:data, :shapes, {:opts, []}],
            arg_types: [:binary, :binary, :keyword]
          }
        ]
      """)

    assert toy.admit("d", "s") == {:ok, {"d", "s", []}}
    assert toy.admit("d", "s", a: 1) == {:ok, {"d", "s", [a: 1]}}
  end

  test "returns is spliced as a type expression" do
    toy =
      compile!("""
      use GraphlawEx.Capability.Extension,
        op: "x",
        overrides: [
          %{
            name: :admit,
            target: GraphlawEx.Capability.ExtensionTest.ToyTarget,
            args: [:data, :shapes],
            returns: {:ok, term()}
          }
        ]
      """)

    assert toy.admit(1, 2) == {:ok, {1, 2, []}}
  end

  test "non-literal override value raises ArgumentError at compile time" do
    assert_raise ArgumentError, ~r/non-literal extension override value/, fn ->
      compile!("""
      use GraphlawEx.Capability.Extension,
        op: "x",
        overrides: [
          %{name: :admit, target: GraphlawEx.Capability.ExtensionTest.ToyTarget,
            doc: String.upcase("x")}
        ]
      """)
    end
  end

  test "non-list overrides raises ArgumentError" do
    assert_raise ArgumentError, ~r/literal list of maps/, fn ->
      compile!(~s|use GraphlawEx.Capability.Extension, op: "x", overrides: System.get_env()|)
    end
  end

  test "built-in sparql override still expands" do
    toy = compile!(~s|use GraphlawEx.Capability.Extension, op: "sparql"|)
    assert function_exported?(toy, :query, 3)
    assert function_exported?(toy, :query, 2)
  end

  test "op without overrides raises" do
    assert_raise ArgumentError, ~r/no extension overrides/, fn ->
      compile!(~s|use GraphlawEx.Capability.Extension, op: "nope"|)
    end
  end

  test "override without target raises ArgumentError naming name and target" do
    assert_raise ArgumentError, ~r/atom :name and an atom :target.*target=:missing/s, fn ->
      compile!(~s|use GraphlawEx.Capability.Extension, op: "x", overrides: [%{name: :admit}]|)
    end
  end

  test "non-atom target raises ArgumentError" do
    assert_raise ArgumentError, ~r/atom :target/, fn ->
      compile!(
        ~s|use GraphlawEx.Capability.Extension, op: "x", overrides: [%{name: :admit, target: "T"}]|
      )
    end
  end

  test "unsupported arg_type raises ArgumentError listing the vocabulary" do
    assert_raise ArgumentError, ~r/unsupported extension arg_type :integer.*:binary/s, fn ->
      compile!("""
      use GraphlawEx.Capability.Extension,
        op: "x",
        overrides: [
          %{name: :admit, target: GraphlawEx.Capability.ExtensionTest.ToyTarget,
            args: [:a], arg_types: [:integer]}
        ]
      """)
    end
  end
end
