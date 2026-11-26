# SPDX-FileCopyrightText: 2026 Sean Chatman <https://github.com/seanchatmangpt/graphlaw/graphs/contributors>
#
# SPDX-License-Identifier: MIT
defmodule GraphlawEx.AshConsumerContractTest do
  use ExUnit.Case, async: false

  # Pins the surface the downstream ash_r2rml consumer depends on. Real wasm
  # engine, no mocks, no Ash/Splode dependency. Engine-backed tests are :wasm:
  # excluded loudly when the artifact is absent (test_helper.exs) and FAILING
  # (TestEngine.engine_path!/0 raises) under GRAPHLAW_REQUIRE_ENGINE=1.

  alias GraphlawEx.Capability.Canonical
  alias GraphlawEx.Capability.Graph
  alias GraphlawEx.Capability.Refusal
  alias GraphlawEx.Capability.Registry
  alias GraphlawEx.Capability.Sparql
  alias GraphlawEx.Error
  alias GraphlawEx.Result.Term

  @ab "<https://x/a> <https://x/p> <https://x/b> .\n"
  @cd "<https://x/c> <https://x/p> <https://x/d> .\n"
  @lit "<https://x/a> <https://x/name> \"Ada\"@en .\n"

  setup context do
    if context[:wasm] do
      path = GraphlawEx.TestEngine.engine_path!()
      start_supervised!({GraphlawEx, wasm_path: path, expected_sha256: :unpinned})
      assert GraphlawEx.Host.available?(GraphlawEx)
    end

    :ok
  end

  # ---- engine-free contract: error classes ---------------------------------

  test "refusal classes are the closed set a consumer can map to forbidden/invalid/unknown" do
    assert Enum.sort(GraphlawEx.Refusal.classes()) ==
             Enum.sort([:blocked_resource, :refused_structure, :refused_identity, :unsupported])
  end

  test "public error codes are a closed REFUSED_* set" do
    codes = Error.codes()
    assert length(codes) == length(Enum.uniq(codes))
    assert Enum.all?(codes, &String.starts_with?(Atom.to_string(&1), "REFUSED_"))
  end

  test "every capability client refusal wraps into an Error with code, class and evidence" do
    for code <- Refusal.client_codes() do
      error = code |> Refusal.build() |> Error.wrap()

      assert %Error{} = error
      assert error.code in Error.codes()
      assert error.class in GraphlawEx.Refusal.classes()
      assert error.evidence["refusal_code"] == Atom.to_string(code)
    end
  end

  test "ceiling_unmet carries the refused_structure class and R_missing_standing term" do
    error =
      "no"
      |> Refusal.ceiling_unmet(%{"step" => "derive:n3", "floor" => "CONSTRUCT"})
      |> Error.wrap()

    assert error.class == :refused_structure
    assert error.evidence["broken_term"] == "R_missing_standing"
    # Evidence carries only whitelisted structured keys; "floor" is dropped.
    assert error.evidence["details"] == %{"step" => "derive:n3"}
  end

  test "unclassified terms wrap totally into :unsupported, never raise" do
    error = Error.wrap({:error, :whatever})
    assert error.code == :REFUSED_INVALID_INPUT
    assert error.class == :unsupported
    # Never carries term content: type tag, size and digest prefix only.
    assert error.evidence["term_type"] == "atom"
    assert is_integer(error.evidence["term_bytes"])
    assert error.evidence["term_sha256_prefix"] =~ ~r/\A[0-9a-f]{16}\z/
    refute Map.has_key?(error.evidence, "raw")
  end

  test "Error is a real exception with a stable message" do
    error = Error.wrap(Refusal.build(:invalid_capability_request))
    assert Exception.message(error) =~ "REFUSED_INVALID_INPUT"
  end

  test "Sparql field registry declares the response keys the consumer reads" do
    names = Enum.map(Sparql.fields(:response), & &1.name)
    assert names == ["variables", "rows", "nquads", "quads", "value"]
  end

  test "Sparql response is a tagged union: one variant per kind, tag_field kind" do
    assert {:ok, op} = Registry.op("sparql")
    assert Enum.map(op.responses, & &1.tag) == ["solutions", "graph", "boolean"]
    assert Enum.all?(op.responses, &(&1.tag_field == "kind"))

    by_tag = Map.new(op.responses, fn v -> {v.tag, Enum.map(v.fields, & &1.name)} end)
    assert by_tag["solutions"] == ["variables", "rows"]
    assert by_tag["graph"] == ["nquads", "quads"]
    assert by_tag["boolean"] == ["value"]

    # The flattened field view is the union of the variants, not a single shape.
    flat = Sparql.fields(:response) |> Enum.map(& &1.name) |> Enum.sort()
    assert flat == by_tag |> Map.values() |> List.flatten() |> Enum.sort()
  end

  test "untagged ops expose exactly one variant with a nil tag" do
    for op <- Registry.ops(), length(op.responses) == 1 do
      assert [%{tag: nil, tag_field: nil}] = op.responses
    end

    for op <- Registry.ops(), length(op.responses) > 1 do
      assert Enum.all?(op.responses, &(is_binary(&1.tag) and is_binary(&1.tag_field)))
      assert op.responses |> Enum.map(& &1.tag) |> Enum.uniq() |> length() == length(op.responses)
    end
  end

  test "Registry digests are blake3 literals, distinct, and consistent across accessors" do
    digests = Registry.digests()
    assert Enum.sort(Map.keys(digests)) == [:receipt_surface, :registry, :surface]

    for {_name, d} <- digests do
      assert d =~ ~r/\Ablake3:[0-9a-f]{64}\z/
    end

    assert digests.registry == Registry.digest()
    assert digests.registry == Registry.registry_blake3()
    assert digests.surface == Registry.surface_digest()
    assert digests.receipt_surface == Registry.receipt_surface_digest()
    assert digests |> Map.values() |> Enum.uniq() |> length() == 3
  end

  test "every registry model has a projected module whose from_map/to_map round-trips" do
    models = Registry.models()
    assert models != []

    for m <- models do
      # The registry names the upstream (AshGraphLaw) namespace; this consumer projects the
      # same models under its own `GraphlawEx` namespace (projection-target.ttl).
      mod = Module.concat([String.replace_prefix(m.elixir_module, "AshGraphLaw", "GraphlawEx")])
      assert Code.ensure_loaded?(mod), "model module #{inspect(mod)} is not loadable"

      keys = mod.keys()
      assert Enum.uniq(keys) == keys

      wire =
        keys
        |> Map.new(fn k -> {k, "v-" <> k} end)
        |> Map.put("unmodelled_wire_key", "kept")

      model = mod.from_map(wire)
      assert %{__struct__: ^mod, extra: %{"unmodelled_wire_key" => "kept"}} = model

      # Models with nested-model fields decode scalar placeholders structurally, so the
      # generic law is a fixed point (decode/encode is idempotent) with the unknown key
      # kept; the exact wire round-trip is pinned on the scalar Receipt model below.
      once = mod.to_map(model)
      assert once["unmodelled_wire_key"] == "kept"
      assert mod.to_map(mod.from_map(once)) == once

      # atom keys decode identically; a non-map decodes to an empty model, never raises
      atom_wire = Map.new(wire, fn {k, v} -> {String.to_atom(k), v} end)
      assert mod.from_map(atom_wire) == model
      assert %{__struct__: ^mod} = mod.from_map(:not_a_map)
    end
  end

  test "Model.Receipt omits nil optional keys and keeps nil required keys" do
    alias GraphlawEx.Model.Receipt

    full = %{
      "parent" => "p",
      "child" => "c",
      "step" => "s",
      "authority" => "a",
      "revision" => "r",
      "added" => 3,
      "lease_id" => "l",
      "plan_blake3" => "blake3:x",
      "subject_blake3" => "blake3:y",
      "future_key" => 1
    }

    assert full |> Receipt.from_map() |> Receipt.to_map() == full

    wire = Receipt.from_map(%{"parent" => "p", "child" => "c", "step" => "s"}) |> Receipt.to_map()
    refute Map.has_key?(wire, "lease_id")
    assert Map.has_key?(wire, "authority") and is_nil(wire["authority"])
    assert Receipt.rust_path() == "graphlaw::law::Receipt"
  end

  # ---- engine-backed contract ----------------------------------------------

  @tag :wasm
  test "Sparql.query SELECT: kind, variables, rows of Term cells, unbound is nil" do
    q = "SELECT ?s ?n WHERE { ?s <https://x/p> ?o . OPTIONAL { ?s <https://x/name> ?n } }"
    assert {:ok, result} = Sparql.query(@ab <> @lit, q)

    assert result.kind == :solutions
    assert result.variables == ["s", "n"]
    assert [[%Term{type: "uri", value: "https://x/a"}, %Term{} = name]] = result.rows
    assert name.value == "Ada"
    assert name.lang == "en"

    assert {:ok, unbound} = Sparql.query(@ab, q)
    assert [[%Term{value: "https://x/a"}, nil]] = unbound.rows
  end

  @tag :wasm
  test "Sparql.query ASK and CONSTRUCT shapes" do
    assert {:ok, ask} = Sparql.query(@ab, "ASK { ?s ?p ?o }")
    assert ask.kind == :boolean
    assert ask.value == true

    assert {:ok, no} = Sparql.query(@ab, "ASK { <https://x/zzz> ?p ?o }")
    assert no.value == false

    assert {:ok, graph} =
             Sparql.query(
               @ab <> @cd,
               "CONSTRUCT { ?s <https://x/q> ?o } WHERE { ?s <https://x/p> ?o }"
             )

    assert graph.kind == :graph
    assert graph.quads == 2
    assert is_binary(graph.nquads)
    assert {:ok, 2} = count_quads(graph.nquads)
  end

  @tag :wasm
  test "Sparql refusals are typed and wrap into the public Error" do
    assert {:error, %Refusal{} = refusal} = Sparql.query(@ab, "SELECT WHERE {{{")
    assert is_atom(refusal.code)

    error = Error.wrap(refusal)
    assert error.code in Error.codes()
    assert error.class in GraphlawEx.Refusal.classes()

    assert {:error, %Refusal{code: :REFUSED_RESOURCE_BOUND} = limit} =
             Sparql.query(@ab <> @cd, "SELECT ?s WHERE { ?s ?p ?o }", max_rows: 1)

    assert Error.wrap(limit).code == :REFUSED_RESOURCE_BOUND
    assert Error.wrap(limit).class == :blocked_resource
  end

  @tag :wasm
  test "Canonical id is blake3-prefixed, order-stable, blank-node-label-stable" do
    assert {:ok, r1} = Canonical.run(data: @ab <> @cd)
    assert {:ok, r2} = Canonical.run(data: @cd <> @ab)
    assert r1.id == r2.id
    assert "blake3:" <> hex = r1.id
    assert hex =~ ~r/\A[0-9a-f]{64}\z/
    assert r1.quads == 2

    assert {:ok, b1} = Canonical.run(data: "_:a <https://x/p> \"v\" .")
    assert {:ok, b2} = Canonical.run(data: "_:zzz <https://x/p> \"v\" .")
    assert b1.id == b2.id

    assert {:ok, d} = Graph.digest(@ab <> @cd)
    assert d == r1.id
  end

  @tag :wasm
  test "Graph equivalent?/diff/union/intersection satisfy set-algebra laws" do
    both = @ab <> @cd

    assert {:ok, true} = Graph.equivalent?(both, @cd <> @ab)
    assert {:ok, false} = Graph.equivalent?(@ab, @cd)

    assert {:ok, union} = Graph.union(@ab, @cd)
    assert {:ok, true} = Graph.equivalent?(union, both)

    # idempotence and commutativity
    assert {:ok, same} = Graph.union(@ab, @ab)
    assert {:ok, true} = Graph.equivalent?(same, @ab)
    assert {:ok, ucd} = Graph.union(@cd, @ab)
    assert ucd == union

    assert {:ok, diff} = Graph.diff(both, @cd)
    assert {:ok, true} = Graph.equivalent?(diff, @ab)
    assert {:ok, ""} = Graph.diff(@ab, @ab)

    assert {:ok, inter} = Graph.intersection(both, @cd)
    assert {:ok, true} = Graph.equivalent?(inter, @cd)
    assert {:ok, ""} = Graph.intersection(@ab, @cd)

    # canonicalize output is itself canonical (fixed point)
    assert {:ok, canon} = Graph.canonicalize(both)
    assert {:ok, ^canon} = Graph.canonicalize(canon)
  end

  @tag :wasm
  test "Graph refuses non-RDF input typed, and the refusal wraps into Error" do
    for result <- [
          Graph.digest("not rdf at all"),
          Graph.equivalent?(@ab, "not rdf at all"),
          Graph.union(@ab, "not rdf at all"),
          Graph.diff("not rdf at all", @ab),
          Graph.intersection(@ab, "not rdf at all")
        ] do
      assert {:error, %Refusal{code: code} = refusal} = result
      assert is_atom(code)

      error = Error.wrap(refusal)
      assert error.code in Error.codes()
      assert error.class in GraphlawEx.Refusal.classes()
    end
  end

  defp count_quads(nquads) do
    with {:ok, r} <- Canonical.run(data: nquads), do: {:ok, r.quads}
  end
end
