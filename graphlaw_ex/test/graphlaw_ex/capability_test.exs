defmodule GraphlawEx.CapabilityTest do
  use ExUnit.Case, async: true

  # Client-side unit courts: coercion, decoding, registry parity and the
  # typed refusal/error surface. The real engine is exercised in
  # capability_smoke_test.exs (tag :wasm). No mocks.

  alias GraphlawEx.Capability.API
  alias GraphlawEx.Capability.Refusal
  alias GraphlawEx.Capability.Registry
  alias GraphlawEx.Error.Refused

  @ops ~w(capabilities sniff parse convert canonical sparql shacl shex n3 entail datalog hooks law policy)

  describe "registry" do
    test "names/0 lists the 14 ops in ABI order" do
      assert Registry.names() == @ops
    end

    test "module_for/1 round-trips every op and refuses an unknown name" do
      for name <- Registry.names() do
        assert {:ok, module} = Registry.module_for(name)
        assert module.op() == name
      end

      assert Registry.module_for("definitely-not-an-op") == :error
      assert Registry.module_for(42) == :error
    end

    test "op/1 carries request and response field maps for every op" do
      for name <- Registry.names() do
        assert {:ok, op} = Registry.op(name)
        assert is_binary(op.summary)
        assert is_list(op.request)
        assert is_list(op.responses)

        for field <- op.request do
          assert is_binary(field.name)
          assert is_binary(field.type)
          assert is_boolean(field.required)
        end
      end
    end

    test "digests match the canonical registry projection" do
      assert Registry.schema() == "graphlaw.capability-registry/1"
      assert Registry.graphlaw_version() == "26.10.28"
      assert Registry.abi_version() == 1
      assert String.starts_with?(Registry.digest(), "blake3:")
      assert String.starts_with?(Registry.surface_digest(), "blake3:")
    end

    test "the unadvertised duckdb op stays out of the facade" do
      refute "duckdb" in Registry.names()
      assert Registry.module_for("duckdb") == :error
    end
  end

  describe "build_request/1 field coercion" do
    test "capabilities takes no fields; anything else is an unknown key" do
      assert {:ok, %{"op" => "capabilities"}} =
               GraphlawEx.Capability.Capabilities.build_request([])

      assert {:error, %Refusal{code: :invalid_capability_request} = refusal} =
               GraphlawEx.Capability.Capabilities.build_request([bogus: 1])

      assert refusal.details["unknown_keys"] == ["bogus"]
    end

    test "sniff: keyword, atom-key map and string-key map all coerce identically" do
      text = "@prefix : <https://x/> . :a :b :c ."

      for args <- [[text: text], %{text: text}, %{"text" => text}] do
        assert {:ok, %{"op" => "sniff", "text" => ^text}} = GraphlawEx.Capability.Sniff.build_request(args)
      end
    end

    test "parse: optional fields coerce (atom values stringify), nil optional means absent" do
      assert {:ok, %{"op" => "parse", "text" => "<s> <p> <o> .", "dialect" => "turtle", "base" => "https://base/"}} =
               GraphlawEx.Capability.Parse.build_request(
                 text: "<s> <p> <o> .",
                 dialect: "turtle",
                 base: "https://base/"
               )

      # faithful to the spec coercion: a bare atom is NOT accepted on a
      # "string"-typed field (only binaries pass; atoms stringify only under
      # any/object/data_spec/json_or_string)
      assert {:error, %Refusal{code: :invalid_capability_request}} =
               GraphlawEx.Capability.Parse.build_request(text: "x", dialect: :turtle)

      assert {:ok, request} = GraphlawEx.Capability.Parse.build_request(text: "x", dialect: nil)
      refute Map.has_key?(request, "dialect")

      assert {:error, %Refusal{code: :invalid_capability_request} = refusal} =
               GraphlawEx.Capability.Parse.build_request(%{})

      assert refusal.details["missing"] == ["text"]
    end

    test "type mismatches refuse with type_errors details" do
      assert {:error, %Refusal{code: :invalid_capability_request} = refusal} =
               GraphlawEx.Capability.Sniff.build_request(text: 42)

      assert [%{"field" => "text", "expected" => "string", "got" => "integer"}] =
               refusal.details["type_errors"]
    end

    test "data_spec: a bare binary becomes %{\"text\" => binary}; atom keys stringify" do
      assert {:ok, %{"data" => spec}} =
               GraphlawEx.Capability.Canonical.build_request(
                 data: %{text: "<s> <p> <o> .", dialect: :turtle}
               )

      assert spec == %{"text" => "<s> <p> <o> .", "dialect" => "turtle"}

      assert {:ok, %{"data" => spec}} =
               GraphlawEx.Capability.Canonical.build_request(data: "<s> <p> <o> .")

      assert spec == %{"text" => "<s> <p> <o> ."}
    end

    test "json_or_string passes a string through and stringifies a map" do
      problem = %{"initial_states" => ["s0"]}

      assert {:ok, %{"problem" => ^problem, "policy" => "raw-json"}} =
               GraphlawEx.Capability.Policy.build_request(problem: problem, policy: "raw-json")

      assert {:ok, %{"policy" => %{"entries" => [["s0", "a"]]}}} =
               GraphlawEx.Capability.Policy.build_request(
                 problem: "{}",
                 policy: %{entries: [["s0", "a"]]}
               )
    end

    test "datalog: list<object> and list<list<string>> coerce with deep stringification" do
      rules = [%{head: ["?s", "p", "?o"], body: [["?s", "edge", "?o"]]}]

      assert {:ok, %{"rules" => rules, "facts" => [["a", "edge", "b"]]}} =
               GraphlawEx.Capability.Datalog.build_request(
                 rules: rules,
                 facts: [["a", "edge", "b"]]
               )

      # deep stringification rewrote the atom keys inside the rule maps
      assert rules == [%{"head" => ["?s", "p", "?o"], "body" => [["?s", "edge", "?o"]]}]
    end
  end

  describe "decode/1" do
    test "never raises on unexpected shapes; extra and missing keys are tolerated" do
      for name <- Registry.names() do
        assert {:ok, struct} = Registry.module_for(name) |> elem(1) |> then(& &1.decode(%{"totally" => "unexpected"}))
        assert struct.raw == %{"totally" => "unexpected"}
      end
    end

    test "a non-map, non-JSON-object response refuses with :capability_response_undecodable" do
      assert {:error, %Refusal{code: :capability_response_undecodable}} =
               GraphlawEx.Capability.Sniff.decode("[1,2,3]")
    end

    test "a JSON binary object decodes" do
      assert {:ok, %{dialect: "Turtle", engine: engine}} =
               GraphlawEx.Capability.Sniff.decode(Jason.encode!(%{dialect: "Turtle", engine: "PurRdf"}))

      assert engine in Registry.engines()
    end

    test "sparql rows decode into Term structs" do
      assert {:ok, result} =
               GraphlawEx.Capability.Sparql.decode(%{
                 "ok" => true,
                 "kind" => "solutions",
                 "variables" => ["s"],
                 "rows" => [[%{"type" => "uri", "value" => "https://x/a"}]]
               })

      assert result.kind == :solutions

      assert [%GraphlawEx.Result.Term{type: "uri", value: "https://x/a"}] =
               result.rows |> hd()
    end

    test "law decodes to %Result.Admitted{} with receipts, NOT a Result.Law" do
      assert {:ok, admitted} =
               GraphlawEx.Capability.Law.decode(%{
                 "states" => ["s0", "s1"],
                 "receipts" => [%{"step" => "rdfs", "index" => 0}],
                 "nquads" => "<a> <b> <c> .\n"
               })

      assert %GraphlawEx.Result.Admitted{} = admitted
      assert admitted.states == ["s0", "s1"]
      assert [%GraphlawEx.Result.Receipt{step: "rdfs", index: 0}] = admitted.receipts
      assert admitted.nquads == "<a> <b> <c> .\n"
    end
  end

  describe "refusal mapping" do
    test "an engine refusal envelope maps losslessly onto Capability.Refusal" do
      refusal =
        Refusal.from_engine(%{
          "ok" => false,
          "error" => %{"code" => "ResourceLimit", "message" => "too big"}
        })

      assert refusal.code == :resource_limit
      assert refusal.message == "too big"
      assert refusal.raw == %{"code" => "ResourceLimit", "message" => "too big"}
      assert refusal.broken_term == :mu_on_O
    end

    test "an unclassified engine error maps to :engine_unclassified, never a crash" do
      assert %Refusal{code: :engine_unclassified} = Refusal.from_engine(%{"ok" => false, "error" => %{"code" => "FutureVariant"}})
      assert %Refusal{code: :engine_unclassified} = Refusal.from_engine("not even a map")
    end

    test "a transport refusal converts losslessly" do
      transport = GraphlawEx.Refusal.new(:saturated, nil, %{"queue" => 3})
      refusal = Refusal.from_transport(transport)
      assert {refusal.code, refusal.details} == {:saturated, %{"queue" => 3}}
    end

    test "client-side build/3 carries code, message fallback and details" do
      refusal = Refusal.build(:unknown_capability, nil, %{"op" => "x"})
      assert refusal.code == :unknown_capability
      assert refusal.message =~ "unknown_capability"
      assert Refusal.client_codes() == ~w(invalid_capability_request unknown_capability capability_response_undecodable)a
    end
  end

  describe "API facade" do
    test "run/3 refuses unknown capabilities (including the unadvertised duckdb op)" do
      assert {:error, %Refusal{code: :unknown_capability}} = API.run("definitely-not-an-op", [])
      assert {:error, %Refusal{code: :unknown_capability}} = API.run("duckdb", [])
    end

    test "every op has a run/2, op/0 and both op/op! pairs on the facade" do
      Code.ensure_loaded!(API)

      for name <- Registry.names() do
        assert function_exported?(API, String.to_atom(name), 2)
        assert function_exported?(API, String.to_atom("#{name}!"), 2)
        {:ok, mod} = Registry.module_for(name)
        assert mod.op() == name
      end
    end

    test "op!/2 raises GraphlawEx.Error.Refused carrying the typed refusal" do
      assert_raise Refused, ~r/GraphLaw refused: invalid_capability_request/, fn ->
        API.capabilities!(bogus: 1)
      end

      assert_raise Refused, fn -> API.sniff!(text: 42) end
    end
  end

  describe "Error.Refused wrapper" do
    test "exposes code/raw/broken_term of the wrapped refusal" do
      {:error, refusal} = API.capabilities([bogus: 1])
      error = struct(Refused, refusal: refusal)

      assert Refused.code(error) == :invalid_capability_request
      assert Refused.broken_term(error) == :mu_on_O
      assert Refused.raw(error) == nil
    end

    test "message falls back to unknown when built without a refusal" do
      assert Refused.message(%Refused{}) == "GraphLaw refused: unknown"
      assert Refused.code(%Refused{}) == nil
    end
  end
end
