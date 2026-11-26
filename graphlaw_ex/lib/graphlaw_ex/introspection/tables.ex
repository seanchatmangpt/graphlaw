# SPDX-FileCopyrightText: 2026 Sean Chatman <https://github.com/seanchatmangpt/graphlaw/graphs/contributors>
#
# SPDX-License-Identifier: MIT
# projection target: ontologies/graphlaw-elixir.ttl; hand-written until projected
defmodule GraphlawEx.Introspection.Tables do
  @moduledoc false

  @typedoc "Splode-compatible error class."
  @type error_class :: :invalid | :forbidden | :unknown

  # Longest-first is irrelevant: no prefix is a prefix of another.
  @scoped_prefixes [
    "n3_",
    "hooks_",
    "max_plan_",
    "max_policy_",
    "datalog_",
    "max_datalog_",
    "duckdb_",
    "shacl_"
  ]

  @doc """
  The Splode-compatible class for an engine refusal kind.

      iex> GraphlawEx.Introspection.Tables.error_class("EngineRejected")
      :invalid
      iex> GraphlawEx.Introspection.Tables.error_class("ResourceLimit")
      :forbidden
      iex> GraphlawEx.Introspection.Tables.error_class("other")
      :unknown
  """
  @spec error_class(String.t()) :: error_class()
  defdelegate error_class(kind), to: GraphlawEx.Error.Class, as: :of_kind

  @doc "Limit-key prefixes that belong to exactly one op."
  @spec scoped_prefixes() :: [String.t()]
  def scoped_prefixes, do: @scoped_prefixes

  @doc """
  The limit-key prefix owned by an op, or `nil`.

      iex> GraphlawEx.Introspection.Tables.scope_prefix("n3")
      "n3_"
      iex> GraphlawEx.Introspection.Tables.scope_prefix("shacl")
      "shacl_"
      iex> GraphlawEx.Introspection.Tables.scope_prefix("parse")
      nil
  """
  @spec scope_prefix(String.t()) :: String.t() | nil
  def scope_prefix("n3"), do: "n3_"
  def scope_prefix("hooks"), do: "hooks_"
  def scope_prefix("law"), do: "max_plan_"
  def scope_prefix("policy"), do: "max_policy_"
  def scope_prefix("shacl"), do: "shacl_"
  def scope_prefix("datalog"), do: "datalog_"
  def scope_prefix(_name), do: nil
end
