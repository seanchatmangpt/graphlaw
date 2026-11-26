# SPDX-FileCopyrightText: 2026 Sean Chatman <https://github.com/seanchatmangpt/graphlaw/graphs/contributors>
#
# SPDX-License-Identifier: MIT
defmodule GraphlawEx.Registry do
  @moduledoc """
  Spec-named introspection surface for the GraphLaw capability registry.

  Delegates to `GraphlawEx.Introspection`; plain data, no process, no engine call.

  ## Examples

      iex> GraphlawEx.Registry.count() == length(GraphlawEx.Registry.capabilities())
      true
      iex> {:ok, info} = GraphlawEx.Registry.info("shacl")
      iex> info.name
      "shacl"
      iex> GraphlawEx.Registry.info("nope")
      :error
  """

  alias GraphlawEx.Introspection

  @doc "All operations in registry order."
  @spec capabilities() :: [Introspection.info()]
  defdelegate capabilities(), to: Introspection

  @doc "Introspect one operation by wire name."
  @spec info(String.t()) :: {:ok, Introspection.info()} | :error
  defdelegate info(name), to: Introspection

  @doc "Splode-compatible class for an engine refusal kind."
  @spec error_class(String.t()) :: Introspection.error_class()
  defdelegate error_class(kind), to: Introspection

  @doc "Number of registered ops."
  @spec count() :: non_neg_integer()
  def count, do: length(Introspection.capabilities())
end
