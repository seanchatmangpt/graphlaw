# SPDX-FileCopyrightText: 2026 ash_graphlaw contributors <https://github.com/seanchatmangpt/ash_graphlaw/graphs/contributors>
#
# SPDX-License-Identifier: MIT

defmodule GraphlawEx.Introspection do
  @moduledoc """
  Declarative, plain-data introspection of the GraphLaw capability surface.

  Everything here is derived from `GraphlawEx.Capability.Registry`; no process is
  started and no engine is called. Results are atom-keyed maps and lists of strings,
  so downstream consumers (for example an Ash extension) can read them without
  depending on any struct defined here.

  Each entry has the keys:

    * `:name` - wire op name
    * `:summary` - one-line description
    * `:input` - request fields (`GraphlawEx.Capability.Registry.field/0`)
    * `:output` - response variants (`GraphlawEx.Capability.Registry.variant/0`)
    * `:limits` - engine resource limits that bound this op
    * `:authority_ceiling` - lease ceiling (`"observe" | "select" | "construct"`) when the
      op is also a `law` step, else `nil`
    * `:refusals` - refusal kinds, each paired with a Splode-compatible class
      (`:invalid | :forbidden | :unknown`)
    * `:refusal_codes` - `details.code` wire values the op may return
  """

  # projection target: ontologies/graphlaw-elixir.ttl; hand-written until projected
  alias GraphlawEx.Capability.Registry
  alias GraphlawEx.Introspection.Tables

  @typedoc "Splode-compatible error class."
  @type error_class :: :invalid | :forbidden | :unknown

  @typedoc "One introspected operation."
  @type info :: %{
          name: String.t(),
          summary: String.t(),
          input: [Registry.field()],
          output: [Registry.variant()],
          limits: %{optional(atom()) => non_neg_integer()},
          authority_ceiling: String.t() | nil,
          refusals: [%{kind: String.t(), class: error_class()}],
          refusal_codes: [String.t()]
        }

  @doc """
  All operations in registry order.

  ## Examples

      iex> caps = GraphlawEx.Introspection.capabilities()
      iex> hd(caps).name
      "capabilities"
      iex> Enum.map(caps, & &1.name) == GraphlawEx.Capability.Registry.names()
      true
  """
  @spec capabilities() :: [info()]
  def capabilities, do: Enum.map(Registry.ops(), &build/1)

  @doc """
  Introspect one operation by wire name.

  ## Examples

      iex> {:ok, info} = GraphlawEx.Introspection.info("shacl")
      iex> info.authority_ceiling
      "observe"
      iex> Enum.map(info.input, & &1.name)
      ["data", "shapes", "base"]

      iex> {:ok, info} = GraphlawEx.Introspection.info("n3")
      iex> info.authority_ceiling
      "construct"
      iex> Map.has_key?(info.limits, :n3_max_iterations)
      true

      iex> {:ok, info} = GraphlawEx.Introspection.info("sniff")
      iex> info.authority_ceiling
      nil

      iex> GraphlawEx.Introspection.info("nope")
      :error
  """
  @spec info(String.t()) :: {:ok, info()} | :error
  def info(name) when is_binary(name) do
    case Registry.op(name) do
      {:ok, op} -> {:ok, build(op)}
      :error -> :error
    end
  end

  def info(_name), do: :error

  @doc """
  The Splode-compatible class for an engine refusal kind.

  ## Examples

      iex> GraphlawEx.Introspection.error_class("EngineRejected")
      :invalid
      iex> GraphlawEx.Introspection.error_class("ResourceLimit")
      :forbidden
      iex> GraphlawEx.Introspection.error_class("Unsupported")
      :unknown
      iex> GraphlawEx.Introspection.error_class("anything else")
      :unknown
  """
  @spec error_class(String.t()) :: error_class()
  def error_class(kind), do: Tables.error_class(kind)

  defp build(op) do
    %{
      name: op.name,
      summary: op.summary,
      input: op.request,
      output: op.responses,
      limits: limits_for(op.name),
      authority_ceiling: ceiling(op.name),
      refusals: Enum.map(op.refusal_kinds, &%{kind: &1, class: error_class(&1)}),
      refusal_codes: op.refusal_codes
    }
  end

  defp ceiling(name) do
    case Enum.find(Registry.law_steps(), &(&1.name == name)) do
      %{ceiling: ceiling} -> ceiling
      nil -> nil
    end
  end

  # Ownership comes from `Registry.limit_meta/0` scope, not key-name prefixes. Scopes
  # `abi` and `wasm` bound every op; a key with no metadata is treated as shared.
  # An op owns the scope of its own name; `plan` limits bound `law`, and the `law`
  # scope (law-state and datalog/policy caps) also bounds `datalog` and `policy`.
  @shared_scopes ["abi", "wasm"]
  @extra_scopes %{
    # `duckdb` limits bound SQL-in-RDF, which only runs as the law's `admit:sql` step (the
    # `duckdb` wire op is hidden, never advertised).
    "law" => ["plan", "duckdb"],
    "datalog" => ["law"],
    "policy" => ["law"]
  }

  defp limits_for(name) do
    owned = [name | Map.get(@extra_scopes, name, [])] ++ @shared_scopes
    meta = Registry.limit_meta()

    Map.filter(Registry.limits(), fn {key, _value} ->
      case Map.fetch(meta, key) do
        {:ok, %{scope: scope}} -> scope in owned
        :error -> true
      end
    end)
  end
end
