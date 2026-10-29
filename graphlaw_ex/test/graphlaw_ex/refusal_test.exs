defmodule GraphlawEx.RefusalTest do
  use ExUnit.Case, async: true

  alias GraphlawEx.Refusal

  describe "closed table" do
    test "every code has a class, a broken term and a doc" do
      for code <- Refusal.codes() do
        assert Refusal.class_of(code) in Refusal.classes()
        assert Refusal.broken_term_of(code) in Refusal.broken_terms()
        assert is_binary(Refusal.doc_of(code))
        assert %Refusal{code: ^code} = Refusal.new(code)
      end
    end

    test "codes outside the table raise on the table functions" do
      assert_raise ArgumentError, fn -> Refusal.class_of(:not_a_code) end
      assert_raise ArgumentError, fn -> Refusal.new(:not_a_code) end
    end

    test "the table carries the required host codes" do
      for code <- ~w(saturated call_timeout call_trapped call_exited fuel_exhausted
                     wasm_digest_mismatch wasm_import_surface_mismatch malformed_response
                     invalid_json invalid_encoding resource_limit engine_unclassified)a do
        assert code in Refusal.codes()
      end
    end
  end

  describe "from_engine/2 forward compatibility" do
    test "maps a known engine code onto the table" do
      refusal = Refusal.from_engine(%{"code" => "ResourceLimit", "message" => "too big"})
      assert refusal.code == :resource_limit
      assert refusal.message == "too big"
    end

    test "a future engine variant is :engine_unclassified, never a crash" do
      future = %{"code" => "QuantumRefused", "kind" => "Entangled", "message" => "no", "novel_field" => 1}
      refusal = Refusal.from_engine(future)
      assert refusal.code == :engine_unclassified
      # Lossless: fields this library does not know are never dropped.
      assert refusal.raw == future
      assert refusal.raw["novel_field"] == 1
    end

    test "non-map payloads never raise" do
      for payload <- [:atom, "text", 42, [1, 2], nil] do
        refusal = Refusal.from_engine(payload)
        assert refusal.code == :engine_unclassified
        assert is_binary(refusal.details["raw"])
      end
    end

    test "preserves kind and caller details" do
      refusal = Refusal.from_engine(%{"code" => "Refused", "kind" => "Ambiguous"}, %{"step" => :admit})
      assert refusal.details["kind"] == "Ambiguous"
      assert refusal.details["step"] == :admit
    end

    test "formerly-coarsened refusal codes are no longer :engine_refused or :engine_unclassified" do
      for wire <- ~w(ReceiptRefused LeaseRefused UnverifiedLeaseRefused Fuse DuckDb BatchMemberFailed) do
        refusal = Refusal.from_engine(%{"code" => wire, "message" => "no"})
        assert refusal.code != :engine_unclassified
        assert refusal.code != :engine_refused
      end
    end

    test "ReceiptRefused lifts step and reason" do
      error = %{"code" => "ReceiptRefused", "message" => "no receipt", "step" => "mint", "reason" => "unsigned"}
      refusal = Refusal.from_engine(error)
      assert %{code: :receipt_refused, class: :refused_identity, broken_term: :R_missing_identity} = refusal
      assert refusal.details["step"] == "mint"
      assert refusal.details["reason"] == "unsigned"
      assert refusal.raw == error
    end

    test "LeaseRefused lifts reason, lease_id and step" do
      error = %{"code" => "LeaseRefused", "reason" => "expired", "lease_id" => "l-1", "step" => "acquire"}
      refusal = Refusal.from_engine(error)
      assert %{code: :lease_refused, class: :refused_identity, broken_term: :R_missing_standing} = refusal
      assert refusal.details["reason"] == "expired"
      assert refusal.details["lease_id"] == "l-1"
      assert refusal.details["step"] == "acquire"
    end

    test "UnverifiedLeaseRefused lifts reason, lease_id and step" do
      error = %{"code" => "UnverifiedLeaseRefused", "reason" => "no proof", "lease_id" => "l-2"}
      refusal = Refusal.from_engine(error)
      assert %{code: :unverified_lease_refused, class: :refused_structure, broken_term: :mu_on_O} = refusal
      assert refusal.details["reason"] == "no proof"
      assert refusal.details["lease_id"] == "l-2"
    end

    test "Fuse lifts message and classifies mu_on_O" do
      refusal = Refusal.from_engine(%{"code" => "Fuse", "message" => "bad structure"})
      assert %{code: :fuse, class: :refused_structure, broken_term: :mu_on_O} = refusal
      assert refusal.message == "bad structure"
    end

    test "DuckDb lifts message and classifies blocked_resource" do
      refusal = Refusal.from_engine(%{"code" => "DuckDb", "message" => "stmt failed"})
      assert %{code: :duckdb_failure, class: :blocked_resource, broken_term: :R_missing_consequence} = refusal
      assert refusal.message == "stmt failed"
    end

    test "BatchMemberFailed lifts member, message, partial_receipt, limit, observed, max" do
      error = %{
        "code" => "BatchMemberFailed",
        "message" => "member 2 failed",
        "member" => 2,
        "partial_receipt" => "b3:abc",
        "limit" => 3,
        "observed" => 2,
        "max" => 3
      }

      refusal = Refusal.from_engine(error)
      assert %{code: :batch_member_failed, class: :blocked_resource, broken_term: :R_missing_consequence} = refusal

      for key <- ~w(member message partial_receipt limit observed max) do
        assert refusal.details[key] == error[key]
      end
    end
  end

  test "per-code constructors fill class and broken_term" do
    assert %{code: :call_trapped, class: :blocked_resource} = Refusal.call_trapped()
    assert %{code: :wasm_digest_mismatch, broken_term: :R_missing_identity} = Refusal.wasm_digest_mismatch()
  end
end
