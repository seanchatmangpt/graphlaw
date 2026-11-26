# SPDX-License-Identifier: MIT
# projection target: ontologies/graphlaw-governance.ttl; hand-written until projected

defmodule GraphlawEx.Authority.Table do
  @moduledoc """
  Pure-data authority tables read by `GraphlawEx.Authority`.

  Projection target: `ontologies/graphlaw-governance.ttl`; hand-written until projected.
  No logic lives here beyond lookups.

  `step_required/0` mirrors `Step::required_ceiling` (src/law.rs:129-141).

  ## Example

      iex> GraphlawEx.Authority.Table.rank(:construct)
      2
      iex> GraphlawEx.Authority.Table.op_ceilings()["sql"]
      :construct
  """

  @type ceiling :: :observe | :select | :construct

  @ceiling_rank %{observe: 0, select: 1, construct: 2}
  @ceiling_names %{"observe" => :observe, "select" => :select, "construct" => :construct}

  # Step::name (src/law.rs:118-127).
  @law_step_names ~w(admit:shacl derive:n3 derive:hooks admit:plan admit:require-receipt
                     admit:require-signed-receipt derive:rdfs derive:owl-rl admit:sql
                     test:capability-probe)

  @law_step_required %{
    "admit:shacl" => :observe,
    "admit:sql" => :observe,
    "admit:require-receipt" => :observe,
    "admit:require-signed-receipt" => :observe,
    "test:capability-probe" => :observe,
    "admit:plan" => :select,
    "derive:n3" => :construct,
    "derive:hooks" => :construct,
    "derive:rdfs" => :construct,
    "derive:owl-rl" => :construct
  }

  @op_ceilings %{
    "capabilities" => :observe,
    "sniff" => :observe,
    "parse" => :observe,
    "convert" => :observe,
    "canonical" => :observe,
    "sparql" => :observe,
    "shacl" => :observe,
    "shex" => :observe,
    "policy" => :observe,
    "n3" => :observe,
    "entail" => :observe,
    "datalog" => :observe,
    "hooks" => :observe,
    "law" => :observe,
    # SQL-in-RDF executes caller-supplied statements: never lease-less.
    "sql" => :construct
  }

  @doc "Ceiling to rank map: observe 0 < select 1 < construct 2."
  @spec ceiling_rank() :: %{ceiling() => non_neg_integer()}
  def ceiling_rank, do: @ceiling_rank

  @doc "Rank of a ceiling."
  @spec rank(ceiling()) :: non_neg_integer()
  def rank(ceiling), do: Map.fetch!(@ceiling_rank, ceiling)

  @doc "Ceiling name string to atom map."
  @spec ceiling_names() :: %{String.t() => ceiling()}
  def ceiling_names, do: @ceiling_names

  @doc "Law step names."
  @spec step_names() :: [String.t()]
  def step_names, do: @law_step_names

  @doc "Required ceiling per law step."
  @spec step_required() :: %{String.t() => ceiling()}
  def step_required, do: @law_step_required

  @doc "Floor ceiling per engine op."
  @spec op_ceilings() :: %{String.t() => ceiling()}
  def op_ceilings, do: @op_ceilings
end
