# SPDX-FileCopyrightText: 2026 ash_graphlaw contributors <https://github.com/seanchatmangpt/ash_graphlaw/graphs/contributors>
#
# SPDX-License-Identifier: MIT
# UNSUPPORTED(generator-capability): pure helper shared by every generated capability module;
# no pack template emits canonical JSON, argument coercion or response decoding.

defmodule GraphlawEx.Capability.Decode do
  @moduledoc """
  Forward-compatible, lossless decoding helpers for capability responses.

  Nothing here raises on data: a value whose shape does not match its registry `type` is
  returned untouched, so callers keep the raw engine value. Only `term`, `list<term>` and
  `list<list<term>>` are rewritten (into `GraphlawEx.Result.Term`); every other type of the
  closed vocabulary passes through as-is.
  """

  alias GraphlawEx.Capability.Refusal
  alias GraphlawEx.Result.Term

  @doc "Accepts a decoded response map, or a JSON binary that decodes to one."
  @spec ensure_map(term()) :: {:ok, map()} | {:error, Refusal.t()}
  def ensure_map(%{} = response) when not is_struct(response), do: {:ok, response}

  def ensure_map(response) when is_binary(response) do
    case Jason.decode(response) do
      {:ok, %{} = map} -> {:ok, map}
      _ -> undecodable(response)
    end
  end

  def ensure_map(response), do: undecodable(response)

  defp undecodable(response) do
    {:error,
     Refusal.build(
       :capability_response_undecodable,
       "capability response is not a JSON object",
       %{"got" => type_name(response)}
     )}
  end

  @doc "Decodes `term` per registry `type`; mismatches return `term` unchanged."
  @spec value(term(), String.t() | nil) :: term()
  def value(nil, _type), do: nil
  def value(term, "term"), do: term(term)

  def value(term, "list<term>") when is_list(term), do: Enum.map(term, &term/1)

  def value(term, "list<list<term>>") when is_list(term) do
    Enum.map(term, fn
      row when is_list(row) -> Enum.map(row, &term/1)
      other -> other
    end)
  end

  def value(term, _type), do: term

  defp term(%{} = map) when not is_struct(map), do: Term.from_map(map)
  defp term(other), do: other

  @doc """
  Resolves the response variant named by `map[tag_field]` among `tags`: the matching tag as an
  atom, or `{:unknown, tag}` (`""` when the tag is absent or not a string).
  """
  @spec variant(term(), String.t() | nil, [String.t()]) :: atom() | {:unknown, String.t()}
  def variant(%{} = map, tag_field, tags) when is_binary(tag_field) and is_list(tags) do
    case Map.get(map, tag_field) do
      tag when is_binary(tag) ->
        if tag in tags, do: String.to_atom(tag), else: {:unknown, tag}

      _ ->
        {:unknown, ""}
    end
  end

  def variant(_map, _tag_field, _tags), do: {:unknown, ""}

  @doc "Reads `name` from a response map and decodes it per `type`; `nil` when absent."
  @spec get(term(), String.t(), String.t() | nil) :: term()
  def get(%{} = map, name, type) when is_binary(name), do: value(Map.get(map, name), type)
  def get(_map, _name, _type), do: nil

  defp type_name(value) when is_list(value), do: "list"
  defp type_name(value) when is_binary(value), do: "string"
  defp type_name(nil), do: "null"
  defp type_name(_), do: "other"
end
