# SPDX-License-Identifier: MIT
defmodule GraphlawEx.Capability.CoerceNormalizeTest do
  use ExUnit.Case, async: true

  alias GraphlawEx.Capability.Coerce
  alias GraphlawEx.Capability.Refusal

  doctest GraphlawEx.Capability.Coerce

  test "keyword, atom-key map and string-key map normalize identically" do
    expected = {:ok, %{"a" => 1}}
    assert Coerce.normalize("op", a: 1) == expected
    assert Coerce.normalize("op", %{a: 1}) == expected
    assert Coerce.normalize("op", %{"a" => 1}) == expected
    assert Coerce.normalize("op", []) == {:ok, %{}}
  end

  test "non-keyword lists, scalars and structs refuse without raising" do
    for bad <- [42, "x", nil, [1, 2], {:a, 1}, %URI{}] do
      assert {:error, %Refusal{code: :invalid_capability_request}} = Coerce.normalize("op", bad)
    end
  end

  test "request/3 with a string key never needs a pre-existing atom" do
    fields = [%{name: "zz_never_an_atom_fixture", type: "string", required: true}]

    assert {:ok, %{"zz_never_an_atom_fixture" => "v"}} =
             Coerce.request("op", fields, %{"zz_never_an_atom_fixture" => "v"})
  end
end
