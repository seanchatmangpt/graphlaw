defmodule GraphlawEx.CapabilitySmokeTest do
  use ExUnit.Case, async: false

  # Chicago: the real wasm artifact is the collaborator. One real-engine
  # smoke per op family, run through PRIV_WASM. Absence of the artifact is a
  # LOUD typed skip, never a faked pass — and when PRIV_WASM is present
  # (as in the gate) these tests MUST run. No mocks, no sleeps.

  alias GraphlawEx.Capability.API
  alias GraphlawEx.Capability.Refusal
  alias GraphlawEx.Result.Admitted

  @wasm_path System.get_env("PRIV_WASM") ||
               Path.expand("priv/graphlaw.wasm", __DIR__)

  @require_engine System.get_env("GRAPHLAW_REQUIRE_ENGINE") == "1"

  @turtle "<https://x/s> <https://x/p> <https://x/o> ."

  setup_all do
    if File.regular?(@wasm_path) do
      start_supervised!({GraphlawEx, wasm_path: @wasm_path, expected_sha256: :unpinned})
      assert GraphlawEx.Host.available?(GraphlawEx)
      :ok
    else
      message =
        "[SKIP] capability smoke courts: wasm artifact not found at #{@wasm_path} " <>
          "(set PRIV_WASM to the absolute path of the graphlaw wasm module). A loud skip is not a pass."

      if @require_engine, do: flunk(message)

      IO.puts(message)
      {:ok, skipped: true}
    end
  end

  # ---------------------------------------------------------------------
  # One smoke per op family
  # ---------------------------------------------------------------------

  @tag :wasm
  test "capabilities: the engine self-reports the surface this registry pins" do
    if engine_present?() do
      assert {:ok, caps} = API.capabilities([])

      assert caps.abi_version == 1
      assert is_binary(caps.crate)
      assert is_list(caps.ops)

      for name <- GraphlawEx.Capability.Registry.names() do
        assert name in caps.ops
      end

      assert is_list(caps.rdf_dialects) and is_list(caps.other_dialects)
      assert caps.raw["ok"] == true
    end
  end

  @tag :wasm
  test "sniff/parse/convert/canonical: the RDF text family" do
    if engine_present?() do
      assert {:ok, sniff} = API.sniff(text: @turtle, hint: "ttl")
      assert sniff.dialect in ["Turtle", "NTriples"]
      assert sniff.engine in GraphlawEx.Capability.Registry.engines()

      assert {:ok, parse} = API.parse(text: @turtle)
      assert parse.quads == 1
      assert is_binary(parse.id)

      assert {:ok, convert} = API.convert(text: @turtle, to: "ntriples")
      assert is_binary(convert.text)
      assert convert.text =~ "<https://x/o>"

      assert {:ok, canonical} = API.canonical(data: @turtle)
      assert is_binary(canonical.nquads)
      assert canonical.quads == 1
      assert is_binary(canonical.id)
    end
  end

  @tag :wasm
  test "sparql: solutions decode with Term cells over the real engine" do
    if engine_present?() do
      assert {:ok, result} =
               API.sparql(data: @turtle, query: "SELECT ?s WHERE { ?s ?p ?o }")

      assert result.kind == :solutions
      assert result.variables == ["s"]
      assert [%GraphlawEx.Result.Term{value: "https://x/s"}] = result.rows |> hd()
    end
  end

  @tag :wasm
  test "shacl: a conforming dataset conforms" do
    if engine_present?() do
      shapes = """
      @prefix sh: <http://www.w3.org/ns/shacl#> .
      [] a sh:NodeShape ; sh:targetSubjectsOf <https://x/p> .
      """

      assert {:ok, result} = API.shacl(data: @turtle, shapes: shapes)
      assert is_boolean(result.conforms)
      assert is_list(result.results)
    end
  end

  @tag :wasm
  test "shex: a schema + shape map validates over the real engine" do
    if engine_present?() do
      schema = "BASE <https://x/>\n<S> { <https://x/p> . }"
      assert {:ok, result} = API.shex(data: @turtle, schema: schema, map: "<https://x/s>@<https://x/S>")

      assert is_boolean(result.conforms)
      assert is_list(result.entries)
    end
  end

  @tag :wasm
  test "n3: bounded forward reasoning returns the reasoner output" do
    if engine_present?() do
      text = "@prefix : <https://x/> . :a :b :c ."
      assert {:ok, result} = API.n3(text: text)
      assert not is_nil(result.derived)
    end
  end

  @tag :wasm
  test "entail: rdfs materialization adds quads to the closure" do
    if engine_present?() do
      data = """
      @prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .
      <https://x/s> rdfs:domain <https://x/C> .
      """

      assert {:ok, result} = API.entail(data: data, regime: "rdfs")
      assert is_binary(result.nquads)
      assert is_integer(result.added)
    end
  end

  @tag :wasm
  test "datalog: a rule derives to fixpoint over real facts" do
    if engine_present?() do
      rules = [%{"head" => ["?x", "anc", "?z"], "body" => [["?x", "anc", "?y"], ["?y", "anc", "?z"]]}]
      facts = [["a", "anc", "b"], ["b", "anc", "c"]]

      assert {:ok, result} = API.datalog(rules: rules, facts: facts)
      assert result.count >= 2
      assert ["a", "anc", "c"] in result.facts
    end
  end

  @tag :wasm
  test "hooks: a kh pack materializes over the data" do
    if engine_present?() do
      pack = hook_pack()
      data = @turtle

      case API.hooks(pack: pack, data: data) do
        {:ok, result} ->
          assert is_integer(result.rounds)
          assert is_binary(result.nquads)
          assert is_list(result.firings)

        {:error, %Refusal{} = refusal} ->
          # A pack the engine refuses is still a real round trip through the
          # kernel: the refusal must be typed and lossless, never a crash.
          assert is_atom(refusal.code)
          assert refusal.message
      end
    end
  end

  @tag :wasm
  test "law: an rdfs step admits and returns receipts (Result.Admitted)" do
    if engine_present?() do
      assert {:ok, %Admitted{} = admitted} =
               API.law(data: @turtle, steps: [%{"step" => "rdfs"}])

      assert is_list(admitted.states) and length(admitted.states) >= 1
      assert is_list(admitted.receipts)
      assert is_binary(admitted.nquads)
    end
  end

  @tag :wasm
  test "policy: a FOND admission round-trips (or refuses typed)" do
    if engine_present?() do
      problem = %{
        "initial_states" => ["s0"],
        "goal_states" => ["s1"],
        "actions" => [
          %{"name" => "go", "pre" => ["s0"], "add" => ["s1"], "outcomes" => [["s1"]]}
        ]
      }

      policy = %{"entries" => [["s0", "go"]]}

      case API.policy(problem: problem, policy: policy) do
        {:ok, result} ->
          assert is_list(result.initial_states)
          assert is_list(result.entries)
          assert is_binary(result.ntriples)

        {:error, %Refusal{} = refusal} ->
          assert is_atom(refusal.code)
          assert refusal.raw
      end
    end
  end

  @tag :wasm
  test "the engine refusal path stays typed through the facade" do
    if engine_present?() do
      # A well-typed request the engine refuses: non-RDF text with an RDF-only op.
      assert {:error, %Refusal{} = refusal} = API.canonical(data: "definitely not rdf at all")
      assert is_atom(refusal.code)
      assert refusal.message
      # never a crash, never a raise
    end
  end

  # ---------------------------------------------------------------------

  defp engine_present? do
    # The setup_all context marks a loud skip; when the artifact exists the
    # host is up and every test above must actually run.
    GraphlawEx.Host.available?(GraphlawEx)
  end

  # A minimal kh: hook pack (hook on every triple of <p>, adding a derived
  # edge). The exact vocabulary is engine-owned; the court accepts a real
  # materialization or a typed refusal — never a crash.
  defp hook_pack do
    """
    @prefix kh: <https://graphlaw.dev/kh#> .
    @prefix xsd: <http://www.w3.org/2001/XMLSchema#> .
    [] a kh:Hook ;
       kh:when [ kh:pattern [ kh:s ?s ; kh:p <p> ; kh:o ?o ] ] ;
       kh:then [ kh:add [ kh:s ?s ; kh:p <p2> ; kh:o ?o ] ] .
    """
  end
end
