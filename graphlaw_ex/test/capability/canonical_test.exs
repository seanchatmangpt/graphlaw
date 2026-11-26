# SPDX-FileCopyrightText: 2026 Sean Chatman <https://github.com/seanchatmangpt/graphlaw/graphs/contributors>
#
# SPDX-License-Identifier: MIT
defmodule GraphlawEx.Capability.CanonicalTest do
  use ExUnit.Case, async: false

  alias GraphlawEx.Capability.Canonical
  alias GraphlawEx.Capability.Refusal

  @wasm_path System.get_env("PRIV_WASM") ||
               Path.expand("../../priv/graphlaw/graphlaw.wasm", __DIR__)
  @require_engine System.get_env("GRAPHLAW_REQUIRE_ENGINE") == "1"

  # Missing artifact: engine tests skip loudly with a reason, or setup_all
  # flunks under GRAPHLAW_REQUIRE_ENGINE=1.
  @wasm_skip if(File.regular?(@wasm_path) or @require_engine,
               do: false,
               else: "canonical court: wasm artifact not found at #{@wasm_path} (set PRIV_WASM)"
             )

  setup_all do
    if File.regular?(@wasm_path) do
      start_supervised!({GraphlawEx, wasm_path: @wasm_path, expected_sha256: :unpinned})
      :ok
    else
      if @require_engine do
        flunk("GRAPHLAW_REQUIRE_ENGINE=1: wasm artifact not found at #{@wasm_path}")
      end

      :ok
    end
  end

  @tag :wasm
  @tag skip: @wasm_skip
  test "statement order does not change the content id" do
    a =
      "<https://x/a> <https://x/p> <https://x/b> .\n<https://x/c> <https://x/p> <https://x/d> .\n"

    b =
      "<https://x/c> <https://x/p> <https://x/d> .\n<https://x/a> <https://x/p> <https://x/b> .\n"

    assert {:ok, ra} = Canonical.run(data: a)
    assert {:ok, rb} = Canonical.run(data: b)
    assert ra.id == rb.id
    assert ra.nquads == rb.nquads
    assert ra.quads == 2
    assert String.starts_with?(ra.id, "blake3:")
  end

  @tag :wasm
  @tag skip: @wasm_skip
  test "different content yields a different id" do
    assert {:ok, ra} = Canonical.run(data: "<https://x/a> <https://x/p> <https://x/b> .")
    assert {:ok, rb} = Canonical.run(data: "<https://x/a> <https://x/p> <https://x/c> .")
    refute ra.id == rb.id
  end

  @tag :wasm
  @tag skip: @wasm_skip
  test "isomorphic blank-node renaming canonicalizes identically" do
    assert {:ok, ra} = Canonical.run(data: "_:a <https://x/p> \"v\" .")
    assert {:ok, rb} = Canonical.run(data: "_:zzz <https://x/p> \"v\" .")
    assert ra.id == rb.id
  end

  @tag :wasm
  @tag skip: @wasm_skip
  test "non-RDF text refuses typed" do
    assert {:error, %Refusal{code: code, message: message}} =
             Canonical.run(data: "definitely not rdf at all")

    assert is_atom(code)
    assert message
  end

  test "missing data refuses :invalid_capability_request" do
    assert {:error, %Refusal{code: :invalid_capability_request}} = Canonical.build_request([])
  end
end
