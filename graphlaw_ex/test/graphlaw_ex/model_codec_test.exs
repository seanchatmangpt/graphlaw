# SPDX-License-Identifier: MIT
defmodule GraphlawEx.ModelCodecTest do
  use ExUnit.Case, async: true

  alias GraphlawEx.Model.Codec
  alias GraphlawEx.Result.Admitted

  test "normalize_keys: string key beats atom key spelling the same wire name" do
    assert Codec.normalize_keys(%{:a => 1, "a" => 2}) == %{"a" => 2}
    assert Codec.normalize_keys(%{"a" => 2, :a => 1}) == %{"a" => 2}
  end

  test "normalize_keys: atom keys stringified, non-atom keys never override real keys" do
    out = Codec.normalize_keys(%{3 => :x, :b => 1})
    assert out["b"] == 1
    assert out["3"] == :x
    assert Codec.normalize_keys(%{"3" => :real, 3 => :fake}) == %{"3" => :real}
  end

  test "encode_one passes a foreign struct through untouched" do
    foreign = ~D[2026-01-01]
    assert Codec.encode_one(foreign, GraphlawEx.Model.SignedLease) == foreign
    assert Codec.encode_one(%{"a" => 1}, GraphlawEx.Model.SignedLease) == %{"a" => 1}
    assert Codec.encode_list([foreign], GraphlawEx.Model.SignedLease) == [foreign]
  end

  test "inspect of a result struct never prints :raw" do
    admitted =
      Admitted.from_map(%{"states" => [], "nquads" => "", "leak" => "RAW-SECRET-PAYLOAD"})

    assert admitted.raw["leak"] == "RAW-SECRET-PAYLOAD"
    out = inspect(admitted)
    refute out =~ "RAW-SECRET-PAYLOAD"
    refute out =~ "raw:"
  end
end
