defmodule GraphlawEx.AuthorityTest do
  use ExUnit.Case, async: true

  # GL28-7 authority pre-gate unit courts: `claim/1` totality and the
  # self-grant attack corpus (design §4 case 6). No engine needed — the gate
  # is a pure presentation check.

  alias GraphlawEx.Authority
  alias GraphlawEx.Capability.Refusal

  @signed_select %{"lease" => %{"id" => "lease-1", "ceiling" => "select"}, "attestation" => %{"key_id" => "ops-2026"}}

  describe "claim/1 totality + self-grant attacks" do
    test "nil claims observe" do
      assert Authority.claim(nil) == %{ceiling: :observe, signed_lease: nil, scope: [], expires_unix: nil}
    end

    test "a bare ceiling atom cannot self-grant" do
      assert Authority.claim(:construct) == %{ceiling: :observe, signed_lease: nil, scope: [], expires_unix: nil}
    end

    test "a map with a bare :ceiling key cannot self-grant" do
      assert Authority.claim(%{ceiling: :construct}) == %{ceiling: :observe, signed_lease: nil, scope: [], expires_unix: nil}
      assert Authority.claim(%{"ceiling" => "construct"}) == %{ceiling: :observe, signed_lease: nil, scope: [], expires_unix: nil}
    end

    test "an unsigned :lease key cannot self-grant" do
      assert Authority.claim(%{lease: %{"ceiling" => "construct"}}) ==
               %{ceiling: :observe, signed_lease: nil, scope: [], expires_unix: nil}
    end

    test "an :unverified_lease cannot self-grant" do
      assert Authority.claim(%{unverified_lease: %{"ceiling" => "construct"}}) ==
               %{ceiling: :observe, signed_lease: nil, scope: [], expires_unix: nil}
    end

    test "a full signed container claims the written ceiling" do
      assert %{ceiling: :select, signed_lease: signed} = Authority.claim(%{signed_lease: @signed_select})
      assert signed == @signed_select
    end

    test "scope and expires_unix are surfaced from the lease body" do
      lease = %{
        signed_lease: %{
          "lease" => %{"ceiling" => "construct", "scope" => ["derive:n3", "admit:plan"], "expires_unix" => 1_900_000_000},
          "attestation" => %{}
        }
      }

      claim = Authority.claim(lease)
      assert claim.scope == ["derive:n3", "admit:plan"]
      assert claim.expires_unix == 1_900_000_000
    end

    test "scope/expires_unix default when the body omits them" do
      claim = Authority.claim(%{signed_lease: @signed_select})
      assert claim.scope == []
      assert claim.expires_unix == nil
    end
  end

  describe "op_ceiling/1" do
    test "the 14-op split is all-observe (kernel dispatch is unguarded)" do
      for op <- ~w(capabilities sniff parse convert canonical sparql shacl shex policy n3 entail datalog hooks law) do
        assert {:ok, :observe} = Authority.op_ceiling(op)
      end
    end

    test "unknown op is a typed :unknown_capability refusal" do
      assert {:error, %Refusal{code: :unknown_capability}} = Authority.op_ceiling("nope")
      assert {:error, %Refusal{code: :unknown_capability}} = Authority.op_ceiling(42)
    end
  end

  describe "check_ceiling/2 + check_op/2" do
    test "observe floor always passes without a lease" do
      assert :ok = Authority.check_op("parse", nil)
      assert :ok = Authority.check_ceiling(:observe, nil)
    end

    test "law without a lease passes through (kernel LeaseAuth::None)" do
      assert :ok = Authority.check_op("law", nil)
      assert :ok = Authority.check_op("n3", nil)
    end

    test "a presented observe-claiming lease on law keeps the additive pre-check" do
      lease = %{signed_lease: %{"lease" => %{"ceiling" => "observe"}, "attestation" => %{}}}
      # op floors are all :observe now, so the op gate passes regardless; the
      # additive pre-check lives in check_ceiling/2 and check_steps/2.
      assert :ok = Authority.check_op("law", lease)
      assert {:error, %Refusal{code: :ceiling_unmet}} = Authority.check_ceiling(:select, lease)
      assert {:error, %Refusal{code: :ceiling_unmet}} = Authority.check_steps(["derive:n3"], %{lease | signed_lease: put_in(lease.signed_lease, ["lease", "scope"], ["derive:n3"])})
    end

    test "select claim vs :construct required refuses via check_ceiling (rank 1 < 2)" do
      lease = %{signed_lease: %{"lease" => %{"ceiling" => "select"}, "attestation" => %{}}}
      assert {:error, %Refusal{code: :ceiling_unmet}} = Authority.check_ceiling(:construct, lease)
    end

    test "a construct lease passes check_ceiling :construct" do
      lease = %{signed_lease: %{"lease" => %{"ceiling" => "construct", "id" => "lease-1"}, "attestation" => %{"key_id" => "k"}}}
      assert :ok = Authority.check_ceiling(:construct, lease)
    end

    test "atom-keyed containers work like string-keyed ones" do
      lease = %{signed_lease: %{:lease => %{:ceiling => :construct}, :attestation => %{:key_id => "k"}}}
      assert %{ceiling: :construct} = Authority.claim(lease)
      assert :ok = Authority.check_op("hooks", lease)
    end
  end

  describe "check_steps/2" do
    test "unknown step name is a typed :unknown_capability refusal" do
      assert {:error, %Refusal{code: :unknown_capability}} = Authority.check_steps(["derive:zzz"], nil)
    end

    test "step outside lease scope refuses :ceiling_unmet" do
      lease = %{signed_lease: %{"lease" => %{"ceiling" => "construct", "scope" => ["derive:n3"]}, "attestation" => %{}}}
      assert {:error, %Refusal{code: :ceiling_unmet}} = Authority.check_steps(["admit:plan"], lease)
    end

    test "derivation step under an observe ceiling refuses (rank 0 < 2)" do
      lease = %{signed_lease: %{"lease" => %{"ceiling" => "observe", "scope" => ["derive:n3"]}, "attestation" => %{}}}
      assert {:error, %Refusal{code: :ceiling_unmet}} = Authority.check_steps(["derive:n3"], lease)
    end

    test "in-scope steps with sufficient ceiling pass" do
      lease = %{
        signed_lease: %{
          "lease" => %{"ceiling" => "construct", "scope" => ["derive:n3", "admit:plan", "admit:shacl"]},
          "attestation" => %{}
        }
      }

      assert :ok = Authority.check_steps(["derive:n3", "admit:plan", "admit:shacl"], lease)
    end

    test "admit:plan needs select, not just observe" do
      lease = %{signed_lease: %{"lease" => %{"ceiling" => "observe", "scope" => ["admit:plan"]}, "attestation" => %{}}}
      assert {:error, %Refusal{code: :ceiling_unmet}} = Authority.check_steps(["admit:plan"], lease)
    end

    test "the first failing step halts with that step's refusal" do
      lease = %{signed_lease: %{"lease" => %{"ceiling" => "select", "scope" => ["admit:plan", "derive:rdfs"]}, "attestation" => %{}}}
      assert {:error, %Refusal{details: %{"step" => "derive:rdfs"}}} = Authority.check_steps(["admit:plan", "derive:rdfs"], lease)
    end
  end

  describe "identity/1" do
    test "nil lease -> nil identity" do
      assert Authority.identity(nil) == nil
    end

    test "a signed lease yields ceiling, ids and a 64-hex digest" do
      lease = %{
        signed_lease: %{
          "lease" => %{"id" => "lease-1", "ceiling" => "construct"},
          "attestation" => %{"key_id" => "ops-2026", "signature" => "aa"}
        }
      }

      identity = Authority.identity(lease)
      assert identity.ceiling == :construct
      assert identity.lease_id == "lease-1"
      assert identity.key_id == "ops-2026"
      assert is_binary(identity.lease_digest) and byte_size(identity.lease_digest) == 64

      # Deterministic: same input, same digest.
      assert Authority.identity(lease).lease_digest == identity.lease_digest
    end
  end

  describe "engine_opts/1" do
    test "deny-list: lease/now_unix/unverified_lease never forwarded" do
      opts = [lease: %{signed_lease: %{"lease" => %{"ceiling" => "construct"}, "attestation" => %{}}}, now_unix: 1, unverified_lease: true]
      engine_opts = Authority.engine_opts(opts)
      assert engine_opts[:signed_lease] == %{"lease" => %{"ceiling" => "construct"}, "attestation" => %{}}
      refute Keyword.has_key?(engine_opts, :lease)
      refute Keyword.has_key?(engine_opts, :now_unix)
      refute Keyword.has_key?(engine_opts, :unverified_lease)
    end

    test "no lease -> no signed_lease opt" do
      refute Keyword.has_key?(Authority.engine_opts([]), :signed_lease)
    end
  end
end
