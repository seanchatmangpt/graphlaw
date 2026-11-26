# SPDX-FileCopyrightText: 2026 Sean Chatman <https://github.com/seanchatmangpt/graphlaw/graphs/contributors>
#
# SPDX-License-Identifier: MIT
defmodule GraphlawEx.Capability.N3Test do
  use ExUnit.Case, async: false

  alias GraphlawEx.Capability.N3
  alias GraphlawEx.Capability.Refusal

  @wasm_path System.get_env("PRIV_WASM") ||
               Path.expand("../../priv/graphlaw/graphlaw.wasm", __DIR__)
  @require_engine System.get_env("GRAPHLAW_REQUIRE_ENGINE") == "1"

  # Real engine-verifiable construct lease (same fixture as capability_smoke_test).
  @pubkey "ea4a6c63e29c520abef5507b132ec5f9954776aebebe7b92421eea691446d22c"
  @lease %{
    signed_lease: %{
      "lease" => %{
        "id" => "smoke-lease",
        "holder" => "graphlaw-ex-smoke",
        "ceiling" => "construct",
        "scope" => ["derive:rdfs"],
        "expires_unix" => 4_000_000_000,
        "issued_unix" => 0
      },
      "attestation" => %{
        "key_id" => "0871f3aabc26e4582c508af5c03884e6a96f0989d1dd8cfb49cd17ed25792433",
        "payload_blake3" => "33383ea00d21b84670233d4dbb3b03c442fe4475675e41f04b3b648daf087c9a",
        "signature" =>
          "0e5145971ddf69b02640f299e6a82000422f1eb608dd009b103d313e38cc9820a3aceaf2920e37373c891e3ec893eb900cd38c3a9544e3f9b86ab6a0e777230e"
      }
    }
  }
  @lease_opts [lease: @lease, trusted_keys: [@pubkey]]

  # Compile-time: a missing artifact skips the engine tests LOUDLY (reason shown
  # by ExUnit) unless GRAPHLAW_REQUIRE_ENGINE=1, in which case setup_all flunks.
  @wasm_skip if(File.regular?(@wasm_path) or @require_engine,
               do: false,
               else: "n3 court: wasm artifact not found at #{@wasm_path} (set PRIV_WASM)"
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
  test "a forward rule derives output over the real reasoner" do
    text = """
    @prefix : <https://x/> .
    :a :parent :b .
    { ?x :parent ?y } => { ?y :child ?x } .
    """

    # The n3 request carries no `signed_lease` field and its op floor is
    # :observe, so the derivation runs lease-less ...
    assert {:ok, result} = N3.run(text: text)
    refute is_nil(result.derived)
    assert inspect(result.derived) =~ "child"

    # ... and a signed lease the request cannot carry is refused typed, never dropped.
    # (the lease-carriage check lives in Dispatch, the layer that owns lease forwarding;
    # `N3.run/2` below it hands the engine options through unchanged).
    assert {:error, %Refusal{code: :lease_not_carried}} =
             GraphlawEx.Capability.Dispatch.run("n3", [text: text], @lease_opts)
  end

  # N3's op floor is :observe (Authority @op_ceilings), so a lease-less call is
  # kernel-legal and passes the pre-gate; it derives nothing without rules.
  @tag :wasm
  @tag skip: @wasm_skip
  test "lease-less N3 passes the pre-gate and derives nothing without rules" do
    assert {:ok, result} = N3.run(text: "@prefix : <https://x/> . :a :b :c .")
    refute inspect(result.derived) =~ "child"
  end

  @tag :wasm
  @tag skip: @wasm_skip
  test "malformed N3 refuses typed under a valid lease" do
    assert {:error, %Refusal{code: code}} = N3.run([text: "{{{ not n3"], @lease_opts)
    assert is_atom(code)
  end

  test "missing text refuses :invalid_capability_request" do
    assert {:error, %Refusal{code: :invalid_capability_request}} = N3.build_request([])
  end
end
