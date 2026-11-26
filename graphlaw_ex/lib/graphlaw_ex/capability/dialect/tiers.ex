# SPDX-License-Identifier: MIT
# projection target: ontologies/graphlaw-governance.ttl; hand-written until projected

defmodule GraphlawEx.Capability.Dialect.Tiers do
  @moduledoc """
  Pure-data dialect tier tables read by `GraphlawEx.Capability.Dialect`.

  Projection target: `ontologies/graphlaw-governance.ttl`; hand-written until projected.
  The tier map is closed: anything not listed is `:unknown`.

  ## Example

      iex> GraphlawEx.Capability.Dialect.Tiers.static_tier("N3")
      :logic
      iex> GraphlawEx.Capability.Dialect.Tiers.static_tier("Nope")
      :unknown
  """

  @type tier :: :dataset | :logic | :query | :schema | :sql | :unknown

  @dataset_dialects ~w(Turtle TriG NTriples NQuads RdfXml JsonLd YamlLd TriX HexTuples)
  @schema_dialects ~w(ShExC ShExJ)

  @tier_map %{"N3" => :logic, "Sparql" => :query, "SqlRdf" => :sql}

  # Compile-time closed lookup: name => tier, one O(log n) map hit per call.
  @all_tiers Map.merge(
               Map.new(@dataset_dialects, &{&1, :dataset}),
               Map.merge(Map.new(@schema_dialects, &{&1, :schema}), @tier_map)
             )

  @doc "RDF dataset serializations."
  @spec dataset_dialects() :: [String.t()]
  def dataset_dialects, do: @dataset_dialects

  @doc "Schema-tier dialects."
  @spec schema_dialects() :: [String.t()]
  def schema_dialects, do: @schema_dialects

  @doc "Fixed non-dataset, non-schema tiers."
  @spec tier_map() :: %{String.t() => tier()}
  def tier_map, do: @tier_map

  @doc "Tier from the static tables alone (dataset names are `:dataset`, registry check is the caller's)."
  @spec static_tier(term()) :: tier()
  def static_tier(name) when is_binary(name), do: Map.get(@all_tiers, name, :unknown)

  def static_tier(_), do: :unknown
end
