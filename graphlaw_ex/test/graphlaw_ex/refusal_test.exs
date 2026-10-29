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
  end

  test "per-code constructors fill class and broken_term" do
    assert %{code: :call_trapped, class: :blocked_resource} = Refusal.call_trapped()
    assert %{code: :wasm_digest_mismatch, broken_term: :R_missing_identity} = Refusal.wasm_digest_mismatch()
  end
end
