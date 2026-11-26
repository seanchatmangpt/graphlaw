# SPDX-License-Identifier: MIT
defmodule GraphlawEx.ContextAbiTest do
  use ExUnit.Case, async: true

  alias GraphlawEx.ABI
  alias GraphlawEx.Context
  alias GraphlawEx.Refusal

  describe "Context" do
    test "merge with explicit nil clears the field; absent field is kept" do
      {:ok, ctx} = Context.new(actor_id: "a", tenant: "t")
      assert {:ok, cleared} = Context.merge(ctx, actor_id: nil)
      assert cleared.actor_id == nil
      assert cleared.tenant == "t"
    end

    test "inspect leaks neither actor_id nor metadata values" do
      {:ok, ctx} =
        Context.new(actor_id: "actor-SECRET-42", tenant: "t1", metadata: %{k: "meta-SECRET"})

      out = inspect(ctx)
      refute out =~ "actor-SECRET-42"
      refute out =~ "meta-SECRET"
      assert out =~ "t1"
    end
  end

  describe "ABI" do
    test "oversize non-UTF8 payload is :resource_limit (size cap runs first)" do
      payload = :binary.copy(<<0xFF>>, ABI.max_request_bytes() + 1)
      assert {:error, %Refusal{code: :resource_limit}} = ABI.recognize_request(payload)
    end

    test "small non-UTF8 payload is :invalid_encoding" do
      assert {:error, %Refusal{code: :invalid_encoding}} = ABI.recognize_request(<<0xFF, 0xFE>>)
    end

    test "frame_iodata/1 bytes equal frame/1 bytes" do
      payload = ~s({"op":"capabilities"})
      {:ok, framed} = ABI.frame(payload)
      {:ok, io} = ABI.frame_iodata(payload)
      assert IO.iodata_to_binary(io) == framed
    end

    test "frame_iodata/1 shares frame/1 refusals" do
      bad = <<0xFF>>
      assert ABI.frame_iodata(bad) == ABI.frame(bad)
    end

    test "encode_request refuses deep nesting with :resource_limit" do
      depth = ABI.max_json_depth() + 1
      deep = Enum.reduce(1..depth, "leaf", fn _, acc -> %{"k" => acc} end)
      assert {:error, %Refusal{code: :resource_limit}} = ABI.encode_request(deep)
    end

    test "encode_request accepts nesting at the limit" do
      # the outer request map is depth 1; build limit-1 more levels
      deep = Enum.reduce(1..(ABI.max_json_depth() - 1), "leaf", fn _, acc -> %{"k" => acc} end)
      assert {:ok, _} = ABI.encode_request(deep)
    end
  end
end
