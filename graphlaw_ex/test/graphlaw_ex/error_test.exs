# SPDX-FileCopyrightText: 2026 Sean Chatman <https://github.com/seanchatmangpt/graphlaw/graphs/contributors>
#
# SPDX-License-Identifier: MIT
defmodule GraphlawEx.ErrorTest do
  use ExUnit.Case, async: true

  alias GraphlawEx.Capability.Refusal, as: CapRefusal
  alias GraphlawEx.Error
  alias GraphlawEx.Error.Refused
  alias GraphlawEx.Refusal

  doctest GraphlawEx.Error
  doctest GraphlawEx.Error.Table

  describe "codes/0" do
    test "is a closed set of distinct REFUSED_ atoms" do
      codes = Error.codes()
      assert codes != []
      assert codes == Enum.uniq(codes)

      for c <- codes do
        assert is_atom(c) and String.starts_with?(Atom.to_string(c), "REFUSED_"), inspect(c)
      end
    end

    test "every host refusal code wraps to a member of the closed set" do
      for code <- Refusal.codes() do
        error = Error.from_refusal(Refusal.new(code))
        assert error.code in Error.codes(), "#{code} -> #{error.code}"
        assert error.class == Refusal.class_of(code)
        assert error.evidence["refusal_code"] == Atom.to_string(code)
      end
    end

    test "from_refusal agrees with code_for for every host code (no hint)" do
      for code <- Refusal.codes() do
        assert Error.from_refusal(Refusal.new(code)).code == Error.code_for(code), "#{code}"
      end
    end

    test "lease and authority codes map to LEASE_REQUIRED, capability extras included" do
      lease_codes =
        for code <- [:ceiling_unmet | Refusal.codes()],
            Error.code_for(code) == :REFUSED_LEASE_REQUIRED,
            do: code

      assert :ceiling_unmet in lease_codes
      assert :lease_revoked in lease_codes
      assert :lease_refused in lease_codes

      for code <- lease_codes do
        assert Error.from_capability(CapRefusal.build(code)).code == :REFUSED_LEASE_REQUIRED
      end
    end

    test "every non-set atom or term falls to REFUSED_INVALID_INPUT" do
      for t <- [:not_a_code, "str", 1, nil, {:x}] do
        assert Error.code_for(t) == :REFUSED_INVALID_INPUT, inspect(t)
      end
    end

    test "every public code is reachable" do
      reached =
        [
          Error.from_refusal(Refusal.new(:wasm_digest_mismatch)),
          Error.from_refusal(Refusal.new(:malformed_response)),
          Error.from_refusal(Refusal.new(:engine_refused, nil, %{"kind" => "Ambiguous"})),
          Error.from_refusal(Refusal.new(:engine_refused, nil, %{"kind" => "UnsupportedSparql"})),
          Error.from_refusal(Refusal.new(:saturated)),
          Error.from_refusal(Refusal.new(:engine_refused, nil, %{"kind" => "MemoryLimit"})),
          Error.from_refusal(Refusal.new(:call_timeout)),
          Error.from_refusal(Refusal.new(:call_trapped)),
          Error.from_refusal(Refusal.new(:engine_refused, nil, %{"kind" => "RawSqlForbidden"})),
          Error.from_refusal(Refusal.new(:lease_refused))
        ]
        |> Enum.map(& &1.code)

      assert Enum.sort(reached) == Enum.sort(Error.codes())
    end
  end

  describe "engine hints" do
    test "ambiguous kind, from details" do
      r = Refusal.new(:engine_refused, nil, %{"kind" => "Ambiguous"})
      assert Error.from_refusal(r).code == :REFUSED_AMBIGUOUS_DIALECT
    end

    test "tokens match whole, never as substrings" do
      r = Refusal.new(:engine_refused, nil, %{"kind" => "AmbiguousDialect"})
      assert Error.from_refusal(r).code == :REFUSED_INVALID_INPUT
    end

    test "free-text message never participates" do
      r = Refusal.new(:engine_refused, "SQLRDF-17 raw sql refused; ambiguous", %{})
      assert Error.from_refusal(r).code == :REFUSED_INVALID_INPUT
    end

    test "hint wins over a base code" do
      r = Refusal.new(:call_trapped, nil, %{"kind" => "MemoryLimit"})
      assert Error.from_refusal(r).code == :REFUSED_MEMORY_LIMIT_EXCEEDED
    end

    test "unsupported needs sparql too" do
      only_unsupported = Refusal.new(:engine_refused, nil, %{"kind" => "Unsupported"})
      assert Error.from_refusal(only_unsupported).code == :REFUSED_INVALID_INPUT

      both = Refusal.new(:engine_refused, nil, %{"kind" => "UnsupportedSparqlFeature"})
      assert Error.from_refusal(both).code == :REFUSED_UNSUPPORTED_SPARQL_FEATURE
    end

    test "hint is read from raw kind and raw code" do
      from_kind = %{Refusal.new(:engine_refused) | raw: %{"kind" => "Memory"}}
      assert Error.from_refusal(from_kind).code == :REFUSED_MEMORY_LIMIT_EXCEEDED

      from_code = %{Refusal.new(:engine_refused) | raw: %{"code" => "Ambiguous"}}
      assert Error.from_refusal(from_code).code == :REFUSED_AMBIGUOUS_DIALECT
    end

    test "from_engine payloads flow through" do
      r = Refusal.from_engine(%{"code" => "Refused", "kind" => "ambiguous", "message" => "x"})
      error = Error.from_refusal(r)
      assert error.code == :REFUSED_AMBIGUOUS_DIALECT
      assert error.detail == "x"

      # The engine payload is never copied; only whitelisted kind/code survive.
      refute Map.has_key?(error.evidence, "raw")
      assert error.evidence["details"] == %{"code" => "Refused", "kind" => "ambiguous"}
    end
  end

  describe "from_refusal/1 evidence" do
    test "string-keyed, wire-safe, nil fields dropped" do
      error = Error.from_refusal(Refusal.call_timeout())

      assert error.evidence == %{
               "refusal_code" => "call_timeout",
               "broken_term" => "R_missing_consequence"
             }

      assert error.detail == Refusal.doc_of(:call_timeout)
      assert error.class == :blocked_resource
    end

    test "details are whitelisted and scalar-clamped" do
      details = %{
        :kind => :an_atom,
        1 => {1, 2},
        "list" => [:a, %{b: 2}],
        "limit" => 1.5,
        "step" => {1, 2}
      }

      error = Error.from_refusal(Refusal.new(:saturated, "m", details))

      assert error.evidence["details"] == %{
               "kind" => "an_atom",
               "limit" => 1.5,
               "step" => "[redacted]"
             }
    end

    test "raw map is never carried, only kind/code" do
      r = Refusal.from_engine(%{"code" => "ResourceLimit", "extra" => [:x]})
      error = Error.from_refusal(r)
      assert error.code == :REFUSED_RESOURCE_BOUND
      refute Map.has_key?(error.evidence, "raw")
      assert error.evidence["details"] == %{"code" => "ResourceLimit", "kind" => nil}
    end

    test "structs inside whitelisted details are redacted, never leaked" do
      error = Error.from_refusal(Refusal.new(:saturated, "m", %{"reason" => ~D[2026-01-01]}))
      assert error.evidence["details"]["reason"] == "[redacted]"
    end
  end

  describe "from_capability/1" do
    test "code in the host table takes its class" do
      error = Error.from_capability(CapRefusal.build(:call_timeout))
      assert error.code == :REFUSED_QUERY_TIMEOUT
      assert error.class == :blocked_resource
      assert error.evidence["broken_term"] == "R_missing_consequence"
    end

    test "capability-only lease code gets refused_structure and LEASE_REQUIRED" do
      error =
        Error.from_capability(
          CapRefusal.ceiling_unmet("below floor", %{"floor" => "x", "reason" => "x"})
        )

      assert error.code == :REFUSED_LEASE_REQUIRED
      assert error.class == :refused_structure
      assert error.detail == "below floor"
      assert error.evidence["refusal_code"] == "ceiling_unmet"
      assert error.evidence["broken_term"] == "R_missing_standing"
      assert error.evidence["details"] == %{"reason" => "x"}
    end

    test "client code message is rendered" do
      error = Error.from_capability(CapRefusal.build(:unknown_capability))
      assert error.detail == "capability request refused: unknown_capability"
      assert error.class == :refused_structure
    end

    test "engine envelope with ambiguous kind" do
      cap =
        CapRefusal.from_engine(%{
          "ok" => false,
          "error" => %{"code" => "Refused", "kind" => "Ambiguous", "message" => "m"}
        })

      assert Error.from_capability(cap).code == :REFUSED_AMBIGUOUS_DIALECT
    end
  end

  describe "wrap/1" do
    test "is identity on an Error" do
      error = %Error{code: :REFUSED_WASM_TRAP, detail: "d"}
      assert Error.wrap(error) == error
    end

    test "dispatches Refusal and Capability.Refusal" do
      assert Error.wrap(Refusal.call_trapped()).code == :REFUSED_WASM_TRAP
      assert Error.wrap(CapRefusal.build(:call_exited)).code == :REFUSED_WASM_TRAP
    end

    test "unwraps {:error, _} at any depth" do
      assert Error.wrap({:error, Refusal.saturated()}).code == :REFUSED_RESOURCE_BOUND
      assert Error.wrap({:error, {:error, Refusal.saturated()}}).code == :REFUSED_RESOURCE_BOUND
    end

    test "unwraps Error.Refused" do
      refused = %Refused{refusal: CapRefusal.build(:call_timeout)}
      assert Error.wrap(refused).code == :REFUSED_QUERY_TIMEOUT
    end

    test "Error.Refused without a refusal is treated as a foreign term" do
      error = Error.wrap(%Refused{refusal: nil})
      assert error.code == :REFUSED_INVALID_INPUT
      assert error.class == :unsupported
      assert error.evidence["term_type"] == "atom"
      refute Map.has_key?(error.evidence, "raw")
    end

    test "foreign terms become unclassified invalid input with type tag and digest only" do
      error = Error.wrap({:nif_error, :boom})

      assert %Error{
               code: :REFUSED_INVALID_INPUT,
               class: :unsupported,
               detail: "unclassified failure",
               evidence: %{
                 "term_type" => "tuple",
                 "term_bytes" => bytes,
                 "term_sha256_prefix" => prefix
               }
             } = error

      assert is_integer(bytes) and bytes > 0
      assert byte_size(prefix) == 16
      refute inspect(error.evidence) =~ "boom"
    end

    test "is total on odd terms" do
      terms = [
        nil,
        :atom,
        1,
        1.5,
        "bin",
        [],
        [1, 2],
        %{},
        %{a: 1},
        self(),
        make_ref(),
        fn -> 1 end
      ]

      for term <- terms do
        error = Error.wrap(term)
        assert %Error{code: :REFUSED_INVALID_INPUT, evidence: %{"term_type" => type}} = error
        assert is_binary(type)
        refute Map.has_key?(error.evidence, "raw")
      end
    end

    test "huge terms never enter evidence" do
      error = Error.wrap(String.duplicate("a", 10_000))
      assert error.evidence["term_type"] == "binary"
      assert error.evidence["term_bytes"] > 10_000
      assert byte_size(inspect(error.evidence)) < 400
    end

    test "inspect hides evidence" do
      refute inspect(Error.wrap(:x)) =~ "term_sha256_prefix"
    end
  end

  describe "exception behaviour" do
    test "message/1 with and without detail" do
      assert Exception.message(%Error{code: :REFUSED_WASM_TRAP}) == "REFUSED_WASM_TRAP"

      assert Exception.message(%Error{code: :REFUSED_WASM_TRAP, detail: nil}) ==
               "REFUSED_WASM_TRAP"

      assert Exception.message(%Error{code: :REFUSED_WASM_TRAP, detail: "trapped"}) ==
               "REFUSED_WASM_TRAP: trapped"
    end

    test "is raisable and rescuable with fields intact" do
      error =
        try do
          raise Error.from_refusal(Refusal.call_timeout())
        rescue
          e in Error -> e
        end

      assert error.code == :REFUSED_QUERY_TIMEOUT
      assert Exception.message(error) =~ "REFUSED_QUERY_TIMEOUT: "
    end

    test "defaults" do
      assert %Error{} == %Error{
               code: :REFUSED_INVALID_INPUT,
               class: nil,
               detail: "",
               evidence: %{}
             }
    end
  end
end
