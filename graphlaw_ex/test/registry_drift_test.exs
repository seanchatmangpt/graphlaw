# SPDX-FileCopyrightText: 2026 Sean Chatman <https://github.com/seanchatmangpt/graphlaw/graphs/contributors>
#
# SPDX-License-Identifier: MIT
defmodule GraphlawEx.RegistryDriftTest do
  @moduledoc """
  Drift court: the generated `GraphlawEx.Capability.Registry` must equal the canonical
  `registry/capability-registry.json`, and the version triple (registry JSON, priv MANIFEST,
  mix project) must agree. No engine is loaded; this reads files and calls public functions.
  """
  use ExUnit.Case, async: true

  alias GraphlawEx.Capability.Registry

  @registry_path Path.expand("../../registry/capability-registry.json", __DIR__)
  @manifest_path Path.expand("../priv/graphlaw/MANIFEST.json", __DIR__)
  @hint "Run scripts/ggen_sync.sh to regenerate lib/graphlaw_ex/capability/registry.ex and sync versions."

  # JSON model_enums name -> public Registry accessor
  @enum_accessors %{
    "Ceiling" => :lease_ceilings,
    "LeaseReason" => :lease_reasons,
    "ReceiptReason" => :receipt_reasons,
    "PolicyRefusalKind" => :policy_refusal_kinds
  }

  setup_all do
    assert File.exists?(@registry_path), "missing #{@registry_path}. #{@hint}"
    {:ok, json: @registry_path |> File.read!() |> Jason.decode!()}
  end

  defp drift!(what, expected, actual) do
    flunk("""
    registry drift in #{what}
      JSON:     #{inspect(expected, limit: 50)}
      Registry: #{inspect(actual, limit: 50)}
    #{@hint}
    """)
  end

  defp assert_same(what, expected, actual) do
    if expected != actual, do: drift!(what, expected, actual)
  end

  defp require_fun!(name, arity) do
    Code.ensure_loaded!(Registry)

    assert function_exported?(Registry, name, arity),
           "GraphlawEx.Capability.Registry.#{name}/#{arity} is not exported. #{@hint}"
  end

  test "registry digest equals JSON registry_blake3", %{json: json} do
    require_fun!(:digest, 0)
    assert_same("registry digest", json["registry_blake3"], Registry.digest())
  end

  test "surface digest equals JSON surface_blake3", %{json: json} do
    require_fun!(:surface_digest, 0)
    assert_same("surface digest", json["surface_blake3"], Registry.surface_digest())
  end

  test "schema equals JSON schema", %{json: json} do
    require_fun!(:schema, 0)
    assert_same("schema", json["schema"], Registry.schema())
  end

  test "graphlaw_version and abi_version equal JSON", %{json: json} do
    require_fun!(:graphlaw_version, 0)
    require_fun!(:abi_version, 0)
    assert_same("graphlaw_version", json["graphlaw_version"], Registry.graphlaw_version())
    assert_same("abi_version", json["abi_version"], Registry.abi_version())
  end

  test "refusal_kinds names equal JSON set", %{json: json} do
    require_fun!(:refusal_kinds, 0)

    assert_same(
      "refusal_kinds",
      Enum.sort(json["refusal_kinds"]),
      Enum.sort(Registry.refusal_kinds())
    )
  end

  test "regimes equal JSON", %{json: json} do
    require_fun!(:regimes, 0)
    assert_same("regimes", json["regimes"], Registry.regimes())
  end

  test "limits map equals JSON limits by name and value", %{json: json} do
    require_fun!(:limits, 0)
    expected = Map.new(json["limits"], fn {k, v} -> {k, v} end)
    actual = Map.new(Registry.limits(), fn {k, v} -> {Atom.to_string(k), v} end)
    assert_same("limits", expected, actual)
  end

  test "op names and count equal JSON ops", %{json: json} do
    require_fun!(:ops, 0)
    require_fun!(:names, 0)
    json_names = Enum.map(json["ops"], & &1["name"])
    assert_same("op names (ops/0)", json_names, Enum.map(Registry.ops(), & &1.name))
    assert_same("op names (names/0)", json_names, Registry.names())
    assert_same("op count", length(json["ops"]), length(Registry.ops()))
  end

  test "per-op refusal_kinds equal JSON", %{json: json} do
    require_fun!(:ops, 0)

    expected = Map.new(json["ops"], &{&1["name"], Enum.sort(&1["refusal_kinds"])})
    actual = Map.new(Registry.ops(), &{&1.name, Enum.sort(&1.refusal_kinds)})
    assert_same("per-op refusal_kinds", expected, actual)
  end

  test "every JSON model_enum is present with equal values", %{json: json} do
    for %{"name" => name, "values" => values} <- json["model_enums"] do
      accessor = Map.get(@enum_accessors, name)

      assert accessor,
             "JSON model_enum #{name} has no accessor mapping in this test; add one. #{@hint}"

      require_fun!(accessor, 0)
      assert_same("model_enum #{name}", values, apply(Registry, accessor, []))
    end
  end

  test "every JSON model is present when Registry exposes models/0", %{json: json} do
    Code.ensure_loaded!(Registry)

    if function_exported?(Registry, :models, 0) do
      # the registry JSON is namespace-neutral (AshGraphLaw.*); the projection renames it to GraphlawEx.*
      json_names =
        json["models"]
        |> Enum.map(&String.replace_prefix(&1["elixir_module"], "AshGraphLaw.", "GraphlawEx."))
        |> Enum.sort()

      actual_names =
        Registry.models()
        |> Enum.map(fn m -> m[:elixir_module] || m["elixir_module"] end)
        |> Enum.map(&to_string/1)
        |> Enum.map(&String.replace_prefix(&1, "Elixir.", ""))
        |> Enum.sort()

      assert_same("models", json_names, actual_names)
    else
      # Optional surface: models are not exposed by the registry module at this revision.
      assert is_list(json["models"])
    end
  end

  test "MANIFEST graphlaw_version == mix project version == JSON version", %{json: json} do
    manifest = @manifest_path |> File.read!() |> Jason.decode!()
    mix_version = Mix.Project.config()[:version]

    unless manifest["graphlaw_version"] == json["graphlaw_version"] and
             mix_version == json["graphlaw_version"] do
      flunk("""
      version skew
        registry JSON:   #{inspect(json["graphlaw_version"])}
        priv MANIFEST:   #{inspect(manifest["graphlaw_version"])}
        mix project:     #{inspect(mix_version)}
      #{@hint}
      """)
    end

    assert manifest["abi_version"] == json["abi_version"],
           "MANIFEST abi_version #{inspect(manifest["abi_version"])} != JSON #{inspect(json["abi_version"])}. #{@hint}"
  end
end
