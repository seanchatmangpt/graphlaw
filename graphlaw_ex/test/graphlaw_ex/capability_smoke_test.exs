# SPDX-FileCopyrightText: 2026 Sean Chatman <https://github.com/seanchatmangpt/graphlaw/graphs/contributors>
#
# SPDX-License-Identifier: MIT
defmodule GraphlawEx.CapabilitySmokeTest do
  use ExUnit.Case, async: false

  # Chicago: the real wasm artifact is the collaborator. One real-engine
  # smoke per op family, run through PRIV_WASM. Absence of the artifact is a
  # LOUD typed skip (module-level @moduletag skip) unless
  # GRAPHLAW_REQUIRE_ENGINE=1, in which case setup_all flunks. There is no
  # per-test wrapper that could turn a missing engine into a vacuous pass.
  # No mocks, no sleeps.

  alias GraphlawEx.Capability.API
  alias GraphlawEx.Capability.Refusal
  alias GraphlawEx.Capability.Registry
  alias GraphlawEx.Result.Admitted

  @wasm_path System.get_env("PRIV_WASM") ||
               if(
                 Code.ensure_loaded?(GraphlawEx.TestEngine) and
                   function_exported?(GraphlawEx.TestEngine, :wasm_path, 0),
                 do: GraphlawEx.TestEngine.wasm_path(),
                 else: Path.expand("../../priv/graphlaw/graphlaw.wasm", __DIR__)
               )

  @require_engine System.get_env("GRAPHLAW_REQUIRE_ENGINE") == "1"

  @turtle "<https://x/s> <https://x/p> <https://x/o> ."

  @kh "http://seanchatmangpt.github.io/graphlaw/kh#"
  @handler "http://seanchatmangpt.github.io/graphlaw/handler#sparql-construct"

  # GL28-7: construct-ceiling ops (n3, entail, datalog, hooks, law) require a
  # signed lease claiming :construct; the authority pre-gate refuses before
  # dispatch otherwise. This fixture is a REAL engine-verifiable lease
  # (Ed25519 over the canonical lease payload, minted offline with
  # graphlaw-attest, seed 07x32): the engine verifies signature, scope
  # (derive:rdfs) and expiry (year 2096) on the same request.
  @smoke_seed_pubkey "ea4a6c63e29c520abef5507b132ec5f9954776aebebe7b92421eea691446d22c"

  @construct_lease %{
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

  # Trust anchors come from Application env only (caller opts are ignored unless
  # `allow_caller_keys`); setup_all installs them and restores on exit.
  @construct_lease_opts [lease: @construct_lease]

  @wasm_present File.regular?(@wasm_path)

  if not @wasm_present and not @require_engine do
    @moduletag skip:
                 "[SKIP] capability smoke courts: wasm artifact not found at #{@wasm_path} " <>
                   "(set PRIV_WASM to the absolute path of the graphlaw wasm module, or " <>
                   "GRAPHLAW_REQUIRE_ENGINE=1 to make absence fatal). A loud skip is not a pass."
  end

  setup_all do
    previous = Application.fetch_env(:graphlaw_ex, :trusted_keys)
    Application.put_env(:graphlaw_ex, :trusted_keys, [@smoke_seed_pubkey])

    on_exit(fn ->
      case previous do
        {:ok, value} -> Application.put_env(:graphlaw_ex, :trusted_keys, value)
        :error -> Application.delete_env(:graphlaw_ex, :trusted_keys)
      end
    end)

    if @wasm_present do
      start_supervised!({GraphlawEx, wasm_path: @wasm_path, expected_sha256: :unpinned})
      assert GraphlawEx.Host.available?(GraphlawEx)
      :ok
    else
      if @require_engine do
        flunk(
          "GRAPHLAW_REQUIRE_ENGINE=1 but wasm artifact not found at #{@wasm_path} " <>
            "(set PRIV_WASM to the absolute path of the graphlaw wasm module)"
        )
      end

      :ok
    end
  end

  # ---------------------------------------------------------------------
  # One smoke per op family
  # ---------------------------------------------------------------------

  @tag :wasm
  test "capabilities: the engine self-reports the surface this registry pins" do
    assert {:ok, caps} = API.capabilities([])

    assert caps.abi_version == Registry.abi_version()
    assert is_binary(caps.crate)
    assert is_list(caps.ops)

    for name <- Registry.names() do
      assert name in caps.ops
    end

    # Bidirectional gate: the engine's self-reported surface must equal the
    # registry's pin exactly — set-equal ops, byte-equal digests, matching
    # authority set. Any drift on either side flips this court.
    assert Enum.sort(caps.ops) == Enum.sort(Registry.names())

    assert caps.registry_blake3 == Registry.digest()
    assert caps.surface_blake3 == Registry.surface_digest()

    assert is_list(caps.rdf_dialects) and is_list(caps.other_dialects)

    # The engine reports fine-grained authority ids ({capability, authority,
    # revision} objects). Pin the exact closed set so any authority added or
    # dropped on the engine side flips this court.
    authorities =
      for a <- caps.authorities || [],
          is_map(a),
          key <- [a["authority"]],
          is_binary(key),
          do: key

    assert Enum.uniq(Enum.sort(authorities)) == [
             "frozen-duckdb",
             "purrdf",
             "purrdf::datalog",
             "purrdf::entail",
             "purrdf::n3",
             "purrdf::shapes",
             "purrdf::shex",
             "purrdf::sparql"
           ]

    for a <- caps.authorities || [] do
      assert is_binary(a["capability"]) and is_binary(a["authority"]) and
               is_binary(a["revision"])
    end

    assert caps.raw["ok"] == true
  end

  @tag :wasm
  test "sniff/parse/convert/canonical: the RDF text family" do
    assert {:ok, sniff} = API.sniff(text: @turtle, hint: "ttl")
    assert sniff.dialect in ["Turtle", "NTriples"]
    assert sniff.engine in Registry.engines()

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

  @tag :wasm
  test "sparql: solutions decode with Term cells over the real engine" do
    assert {:ok, result} =
             API.sparql(data: @turtle, query: "SELECT ?s WHERE { ?s ?p ?o }")

    assert result.kind == :solutions
    assert result.variables == ["s"]
    assert [%GraphlawEx.Result.Term{value: "https://x/s"}] = result.rows |> hd()
  end

  @tag :wasm
  test "shacl: a conforming dataset conforms" do
    shapes = """
    @prefix sh: <http://www.w3.org/ns/shacl#> .
    [] a sh:NodeShape ; sh:targetSubjectsOf <https://x/p> .
    """

    assert {:ok, result} = API.shacl(data: @turtle, shapes: shapes)
    assert result.conforms == true
    assert result.results == []
  end

  @tag :wasm
  test "shex: a schema + shape map validates over the real engine" do
    schema = "BASE <https://x/>\n<S> { <https://x/p> . }"

    assert {:ok, result} =
             API.shex(data: @turtle, schema: schema, map: "<https://x/s>@<https://x/S>")

    assert result.conforms == true
    assert is_list(result.entries) and length(result.entries) == 1
  end

  @tag :wasm
  test "n3: bounded forward reasoning returns the reasoner output" do
    text = "@prefix : <https://x/> . :a :b :c ."
    assert {:ok, result} = API.n3(text: text)
    assert not is_nil(result.derived)
  end

  @tag :wasm
  test "entail: rdfs materialization adds quads to the closure" do
    data = """
    @prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .
    <https://x/s> rdfs:domain <https://x/C> .
    """

    assert {:ok, result} = API.entail(data: data, regime: "rdfs")
    assert is_binary(result.nquads)
    assert is_integer(result.added)
  end

  @tag :wasm
  test "datalog: a rule derives to fixpoint over real facts" do
    rules = [
      %{"head" => ["?x", "anc", "?z"], "body" => [["?x", "anc", "?y"], ["?y", "anc", "?z"]]}
    ]

    facts = [["a", "anc", "b"], ["b", "anc", "c"]]

    assert {:ok, result} = API.datalog(rules: rules, facts: facts)
    assert result.count >= 2
    assert ["a", "anc", "c"] in result.facts
  end

  @tag :wasm
  test "hooks: a kh pack that must admit fires exactly once and materializes" do
    data = %{
      text: "<https://e/x> <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <https://e/A> .\n",
      dialect: "ntriples"
    }

    assert {:ok, result} = API.hooks(pack: hook_pack(), data: data)

    assert [firing] = result.firings
    assert is_map(firing)
    assert result.rounds >= 1
    assert is_binary(result.id)
    # derived B-typed quad is present in the materialized state
    assert result.nquads =~ "<https://e/B>"
    assert result.nquads =~ "<https://e/A>"
  end

  @tag :wasm
  test "hooks: observe floor passes the gate; a signed lease it cannot carry is refused" do
    data = %{text: "<https://e/x> <https://e/p> <https://e/o> .\n", dialect: "ntriples"}

    assert {:ok, _} = API.hooks(pack: hook_pack(), data: data)

    assert {:error, %Refusal{code: :lease_not_carried}} =
             API.hooks([pack: hook_pack(), data: data], @construct_lease_opts)
  end

  @tag :wasm
  test "law: an rdfs step admits and returns receipts (Result.Admitted)" do
    assert {:ok, %Admitted{} = admitted} =
             API.law([data: @turtle, steps: [%{"step" => "rdfs"}]], @construct_lease_opts)

    assert is_list(admitted.states) and length(admitted.states) >= 1
    assert is_list(admitted.receipts)
    assert is_binary(admitted.nquads)
  end

  @tag :wasm
  test "policy: a strong-cyclic FOND policy admits with its goal and entries" do
    assert {:ok, result} = API.policy(problem: policy_problem(), policy: policy_good())

    assert result.initial_states == ["s0"]
    assert result.goal_states == ["g"]
    assert result.entries == [["s0", "flip"]]
    assert is_binary(result.ntriples) and result.ntriples != ""
  end

  @tag :wasm
  test "policy: a skewed-mass policy is refused with exactly :policy_refused" do
    skewed =
      put_in(
        policy_good(),
        ["policy", Access.at(0), "outcomes", Access.at(0), "probability_ppm"],
        400_000
      )

    assert {:error, %Refusal{code: :policy_refused} = refusal} =
             API.policy(problem: policy_problem(), policy: skewed)

    assert refusal.message =~ "BadMass"
  end

  @tag :wasm
  test "the engine refusal path stays typed through the facade" do
    # A well-typed request the engine refuses: non-RDF text with an RDF-only op.
    assert {:error, %Refusal{} = refusal} = API.canonical(data: "definitely not rdf at all")
    assert is_atom(refusal.code)
    assert refusal.message
  end

  # ---------------------------------------------------------------------

  # Two-hook chain mirrors graphlaw tests/knowledge_hooks.rs: h1 derives
  # ex:B from ex:A (sparql select, emit-delta, construct action).
  defp hook_pack do
    """
    @prefix kh: <#{@kh}> .
    @prefix ex: <https://e/> .
    ex:h1 a kh:Hook ; kh:kind "sparql" ; kh:effect "emit-delta" ; kh:priority 1 ;
      kh:action ex:a1 ;
      kh:query "SELECT ?x WHERE { ?x a <https://e/A> }" .
    ex:a1 a kh:Action ; kh:handler <#{@handler}> ;
      kh:query "CONSTRUCT { ?x a <https://e/B> } WHERE { ?x a <https://e/A> }" .
    """
  end

  defp policy_problem do
    %{
      "states" => [%{"id" => "s0"}, %{"id" => "g", "facts" => ["done"]}],
      "initial_states" => ["s0"],
      "goal" => %{"facts" => ["done"]},
      "transitions" => [
        %{"action" => "flip", "from" => "s0", "to" => "g", "probability_ppm" => 500_000},
        %{"action" => "flip", "from" => "s0", "to" => "s0", "probability_ppm" => 500_000}
      ]
    }
  end

  defp policy_good do
    %{
      "policy" => [
        %{
          "state" => "s0",
          "action" => "flip",
          "outcomes" => [
            %{"state" => "g", "probability_ppm" => 500_000},
            %{"state" => "s0", "probability_ppm" => 500_000}
          ]
        }
      ]
    }
  end
end
