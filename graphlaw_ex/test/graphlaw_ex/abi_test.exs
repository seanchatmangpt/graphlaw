# SPDX-FileCopyrightText: 2026 Sean Chatman <https://github.com/seanchatmangpt/graphlaw/graphs/contributors>
#
# SPDX-License-Identifier: MIT
defmodule GraphlawEx.ABITest do
  use ExUnit.Case, async: true

  import Bitwise

  alias GraphlawEx.ABI
  alias GraphlawEx.Refusal

  @cap 16 * 1_048_576

  describe "unpack_result/1" do
    test "splits a positive packed value into {ptr, len}" do
      assert ABI.unpack_result(bsl(123, 32) + 45) == {123, 45}
      assert ABI.unpack_result(0) == {0, 0}
    end

    test "normalizes a signed-i64 view of the same 64 bits" do
      # -1 as i64 is 0xFFFF_FFFF_FFFF_FFFF: ptr and len both max u32.
      assert ABI.unpack_result(-1) == {4_294_967_295, 4_294_967_295}
      # 0x8000_0000_0000_0001 as i64 is negative (the old facade guard dropped it).
      packed = -9_223_372_036_854_775_807
      assert ABI.unpack_result(packed) == {2_147_483_648, 1}
    end

    test "round-trips a pointer in the high half of the 32-bit space" do
      ptr = 3 * bsl(1, 30)
      len = 4096
      signed = bsl(ptr, 32) + len - bsl(1, 64)
      assert ABI.unpack_result(signed) == {ptr, len}
    end
  end

  describe "recognize_request/1" do
    test "accepts valid JSON payloads" do
      payload = ~s({"op":"law","nested":{"items":[1,2,3],"value":-42.5e+10}})
      assert {:ok, ^payload} = ABI.recognize_request(payload)
    end

    test "refuses JSON nesting depth above 64 with :resource_limit" do
      at_limit = String.duplicate("[", 64) <> "1" <> String.duplicate("]", 64)
      assert {:ok, ^at_limit} = ABI.recognize_request(at_limit)

      too_deep = String.duplicate("[", 65) <> "1" <> String.duplicate("]", 65)
      assert {:error, %Refusal{code: :resource_limit} = refusal} = ABI.recognize_request(too_deep)
      assert refusal.details.depth == 65
      assert refusal.details.limit == 64
    end

    test "refuses numeric tokens above 64 digits (CVE-2026-115274 class) with :resource_limit" do
      at_limit = ~s({"n":#{String.duplicate("9", 64)}})
      assert {:ok, ^at_limit} = ABI.recognize_request(at_limit)

      too_long = ~s({"n":#{String.duplicate("9", 65)}})
      assert {:error, %Refusal{code: :resource_limit} = refusal} = ABI.recognize_request(too_long)
      assert refusal.details.digits == 65
      assert refusal.details.limit == 64
    end

    test "refuses invalid UTF-8 with :invalid_encoding" do
      assert {:error, %Refusal{code: :invalid_encoding}} = ABI.recognize_request(<<0xFF, 0xFE>>)
    end
  end

  describe "frame/1" do
    test "prefixes the payload with its 4-byte big-endian length" do
      assert {:ok, <<len::unsigned-integer-size(32), "hello">>} = ABI.frame("hello")
      assert len == 5
    end

    test "refuses a non-UTF-8 payload before any engine allocation" do
      assert {:error, %Refusal{code: :invalid_encoding}} = ABI.frame(<<0xFF, 0xFE>>)
    end

    test "refuses a payload above the 16 MiB cap" do
      assert {:error, %Refusal{code: :resource_limit} = refusal} =
               ABI.frame(String.duplicate("a", @cap + 1))

      assert refusal.details.bytes == @cap + 1
    end

    test "refuses a non-binary payload" do
      assert {:error, %Refusal{code: :invalid_json}} = ABI.frame(%{"op" => "law"})
    end

    test "the cap boundary itself frames fine" do
      assert {:ok, <<len::unsigned-integer-size(32), _rest::binary>>} =
               ABI.frame(String.duplicate("a", @cap))

      assert len == @cap
    end
  end

  describe "encode_request/1" do
    test "encodes a map to JSON bytes" do
      assert {:ok, ~s({"op":"law"})} = ABI.encode_request(%{"op" => "law"})
    end

    test "refuses an over-cap request as :resource_limit" do
      big = %{"op" => "law", "data" => %{"text" => String.duplicate("a", @cap)}}
      assert {:error, %Refusal{code: :resource_limit}} = ABI.encode_request(big)
    end

    test "refuses a non-encodable term as :invalid_json" do
      assert {:error, %Refusal{code: :invalid_json}} = ABI.encode_request(%{"bad" => self()})
    end
  end

  describe "decode_response/1" do
    test "decodes an ok and a refused envelope" do
      assert {:ok, %{"ok" => true}} = ABI.decode_response(~s({"ok":true}))
      assert {:ok, %{"ok" => false}} = ABI.decode_response(~s({"ok":false,"error":{}}))
    end

    test "non-JSON bytes are :invalid_json, non-objects are :malformed_response" do
      assert {:error, %Refusal{code: :invalid_json}} = ABI.decode_response(<<0, 1, 2>>)
      assert {:error, %Refusal{code: :malformed_response}} = ABI.decode_response("[1,2]")
    end
  end
end
