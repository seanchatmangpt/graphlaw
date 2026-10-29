# SPDX-FileCopyrightText: 2026 ash_graphlaw contributors <https://github.com/seanchatmangpt/ash_graphlaw/graphs/contributors>
#
# SPDX-License-Identifier: MIT
# UNSUPPORTED(generator-capability): pure helper shared by every generated capability module;
# no pack template emits canonical JSON, argument coercion or response decoding.

defmodule GraphlawEx.Capability.Coerce do
  @moduledoc """
  Argument normalization for typed capability requests.

  `request/3` turns caller `args` (keyword list or map, atom or string keys) into the string-keyed
  request body of one capability, given the registry `fields` of its request:

    * nested maps under `data_spec`/`object`/`any`/`json_or_string` fields are deep-stringified;
    * a bare binary for a `data_spec` field means `%{"text" => binary}`;
    * `nil` for a non-required field means absent;
    * unknown top-level keys (including `op`) refuse with `details["unknown_keys"]`;
    * missing required fields refuse with `details["missing"]`;
    * type mismatches refuse with `details["type_errors"]` (`field`, `expected`, `got`);
    * `enum` is informational and never enforced; defaults are applied by the engine.

  Every refusal is `:invalid_capability_request`. The function is pure and never raises on bad data.
  """

  alias GraphlawEx.Capability.Refusal

  @type field_map :: %{
          required(:name) => String.t(),
          required(:type) => String.t(),
          optional(atom()) => term()
        }

  @doc "Coerces `args` against the request `fields` of `op`."
  @spec request(String.t(), [field_map()], term()) ::
          {:ok, %{String.t() => term()}} | {:error, Refusal.t()}
  def request(op, fields, args) do
    with {:ok, given} <- normalize_args(op, args),
         :ok <- check_unknown(op, fields, given),
         given = drop_absent(fields, given),
         :ok <- check_missing(op, fields, given) do
      coerce_types(op, fields, given)
    end
  end

  @doc """
  Deep-stringifies keys and atom values: map keys become strings, keyword lists become maps,
  non-boolean atoms become strings. Idempotent.
  """
  @spec stringify(term()) :: term()
  def stringify(nil), do: nil
  def stringify(bool) when is_boolean(bool), do: bool
  def stringify(atom) when is_atom(atom), do: Atom.to_string(atom)
  def stringify(%_{} = struct), do: struct |> Map.from_struct() |> stringify()

  def stringify(%{} = map) do
    Map.new(map, fn {key, value} -> {key_string(key), stringify(value)} end)
  end

  def stringify([{key, _} | _] = list) when is_atom(key) do
    if Keyword.keyword?(list), do: list |> Map.new() |> stringify(), else: Enum.map(list, &stringify/1)
  end

  def stringify(list) when is_list(list), do: Enum.map(list, &stringify/1)
  def stringify(other), do: other

  defp key_string(key) when is_binary(key), do: key
  defp key_string(key) when is_atom(key), do: Atom.to_string(key)
  defp key_string(key), do: inspect(key)

  defp normalize_args(_op, %{} = map) when not is_struct(map), do: {:ok, stringify_keys(map)}
  defp normalize_args(_op, []), do: {:ok, %{}}

  defp normalize_args(op, list) when is_list(list) do
    if Keyword.keyword?(list) do
      {:ok, list |> Map.new() |> stringify_keys()}
    else
      bad_args(op, list)
    end
  end

  defp normalize_args(op, other), do: bad_args(op, other)

  defp bad_args(op, other) do
    {:error,
     Refusal.build(
       :invalid_capability_request,
       "capability #{op}: args must be a keyword list or a map, got #{type_name(other)}",
       %{"type_errors" => [%{"field" => "args", "expected" => "map", "got" => type_name(other)}]}
     )}
  end

  defp stringify_keys(map), do: Map.new(map, fn {key, value} -> {key_string(key), value} end)

  defp check_unknown(op, fields, given) do
    known = MapSet.new(fields, &fetch!(&1, :name))

    case given |> Map.keys() |> Enum.reject(&MapSet.member?(known, &1)) |> Enum.sort() do
      [] ->
        :ok

      unknown ->
        {:error,
         Refusal.build(
           :invalid_capability_request,
           "capability #{op}: unknown request keys #{Enum.join(unknown, ", ")}",
           %{"unknown_keys" => unknown}
         )}
    end
  end

  defp drop_absent(fields, given) do
    Enum.reduce(fields, given, fn field, acc ->
      name = fetch!(field, :name)
      if Map.get(acc, name, :absent) == nil, do: Map.delete(acc, name), else: acc
    end)
  end

  defp check_missing(op, fields, given) do
    missing =
      fields
      |> Enum.filter(&(fetch!(&1, :required) == true))
      |> Enum.map(&fetch!(&1, :name))
      |> Enum.reject(&Map.has_key?(given, &1))

    case missing do
      [] ->
        :ok

      _ ->
        {:error,
         Refusal.build(
           :invalid_capability_request,
           "capability #{op}: missing required fields #{Enum.join(missing, ", ")}",
           %{"missing" => missing}
         )}
    end
  end

  defp coerce_types(op, fields, given) do
    {body, errors} = Enum.reduce(fields, {%{}, []}, &coerce_field(&1, given, &2))

    case Enum.reverse(errors) do
      [] ->
        {:ok, body}

      type_errors ->
        {:error,
         Refusal.build(
           :invalid_capability_request,
           "capability #{op}: type errors in " <> Enum.map_join(type_errors, ", ", & &1["field"]),
           %{"type_errors" => type_errors}
         )}
    end
  end

  defp coerce_field(field, given, {body, errors}) do
    name = fetch!(field, :name)
    type = fetch!(field, :type)

    with {:ok, raw} <- Map.fetch(given, name),
         {:ok, value} <- coerce(type, raw) do
      {Map.put(body, name, value), errors}
    else
      :error ->
        case Map.fetch(given, name) do
          {:ok, raw} ->
            error = %{"field" => name, "expected" => type, "got" => type_name(raw)}
            {body, [error | errors]}

          :error ->
            {body, errors}
        end
    end
  end

  defp fetch!(field, key) do
    case Map.fetch(field, key) do
      {:ok, value} -> value
      :error -> Map.get(field, Atom.to_string(key))
    end
  end

  defp coerce("string", value) when is_binary(value), do: {:ok, value}
  defp coerce("integer", value) when is_integer(value), do: {:ok, value}
  defp coerce("boolean", value) when is_boolean(value), do: {:ok, value}
  defp coerce("any", value), do: {:ok, stringify(value)}
  defp coerce("object", value), do: object(value)
  defp coerce("term", value), do: object(value)
  defp coerce("data_spec", value) when is_binary(value), do: {:ok, %{"text" => value}}
  defp coerce("data_spec", value), do: data_spec(stringify(value))

  defp coerce("json_or_string", value) when is_binary(value), do: {:ok, value}

  defp coerce("json_or_string", value) when is_map(value) or is_list(value) do
    {:ok, stringify(value)}
  end

  defp coerce("list<" <> rest, value) when is_list(value) do
    inner = String.trim_trailing(rest, ">")
    each(value, inner)
  end

  defp coerce(_type, _value), do: :error

  defp object(value) when is_map(value) or (is_list(value) and value != []) do
    case stringify(value) do
      %{} = map -> {:ok, map}
      _ -> :error
    end
  end

  defp object([]), do: {:ok, %{}}
  defp object(_), do: :error

  defp data_spec(%{"text" => text} = spec) when is_binary(text) do
    spec = spec |> Enum.reject(fn {_key, value} -> is_nil(value) end) |> Map.new()

    if Enum.all?(~w(dialect hint base), &is_binary(Map.get(spec, &1, ""))), do: {:ok, spec}, else: :error
  end

  defp data_spec(_), do: :error

  defp each(values, inner) do
    Enum.reduce_while(values, {:ok, []}, fn value, {:ok, acc} ->
      case coerce(inner, value) do
        {:ok, coerced} -> {:cont, {:ok, [coerced | acc]}}
        :error -> {:halt, :error}
      end
    end)
    |> case do
      {:ok, acc} -> {:ok, Enum.reverse(acc)}
      :error -> :error
    end
  end

  defp type_name(nil), do: "null"
  defp type_name(value) when is_boolean(value), do: "boolean"
  defp type_name(value) when is_binary(value), do: "string"
  defp type_name(value) when is_integer(value), do: "integer"
  defp type_name(value) when is_float(value), do: "float"
  defp type_name(value) when is_atom(value), do: "atom"
  defp type_name(value) when is_map(value), do: "object"
  defp type_name(value) when is_list(value), do: "list"
  defp type_name(value) when is_tuple(value), do: "tuple"
  defp type_name(_), do: "other"
end
