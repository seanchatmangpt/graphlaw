# SPDX-License-Identifier: MIT

defmodule GraphlawEx.Capability.Dialect do
  @moduledoc ~S"""
  FR-4 dialect routing in front of the generated `sniff`, `shacl` and `n3` capabilities.

    * `sniff/2` - 14-dialect sniff. An ambiguous or unrecognised answer refuses with
      `:REFUSED_AMBIGUOUS_DIALECT`; the dialect is never guessed.
    * `admit_shacl/2` - SHACL admission over dataset-tier text. Text that sniffs to N3
      is refused (`:REFUSED_TIER_MISMATCH`), never down-converted to a dataset.
    * `admit_n3/2` - N3 is a separate logic tier: only text that sniffs to N3 reaches
      the `n3` op.

  Every function returns `{:ok, result}` or `{:error, %GraphlawEx.Capability.Refusal{}}`.

  ## Example

      iex> GraphlawEx.Capability.Dialect.tier("N3")
      :logic
      iex> GraphlawEx.Capability.Dialect.tier("Turtle")
      :dataset
      iex> GraphlawEx.Capability.Dialect.tier("Nope")
      :unknown
  """

  alias GraphlawEx.Capability.Coerce
  alias GraphlawEx.Capability.Dispatch
  alias GraphlawEx.Capability.Refusal
  alias GraphlawEx.Capability.Sniff
  alias GraphlawEx.Result.Sniff, as: SniffResult

  @type tier :: :dataset | :logic | :query | :schema | :sql | :unknown

  alias GraphlawEx.Capability.Dialect.Tiers

  @doc """
  Tier of a sniffed dialect (Rust variant name). The mapping is closed:
  `:dataset` only for RDF serializations, `:logic` for N3, `:query` for Sparql,
  `:sql` for SqlRdf, `:schema` for ShExC/ShExJ; anything else is `:unknown`.
  """
  @spec tier(term()) :: tier()
  def tier(name), do: Tiers.static_tier(name)

  @doc "Sniffs `text`; ambiguity or an unknown dialect refuses with `:REFUSED_AMBIGUOUS_DIALECT`."
  @spec sniff(String.t(), keyword()) :: {:ok, SniffResult.t()} | {:error, Refusal.t()}
  def sniff(text, opts \\ []) when is_binary(text) do
    args = [text: text] ++ Keyword.take(opts, [:hint])

    case Sniff.run(args, Keyword.delete(opts, :hint)) do
      {:ok, %SniffResult{dialect: dialect} = result} ->
        if tier(dialect) == :unknown,
          do: {:error, ambiguous(%{"dialect" => inspect(dialect)})},
          else: {:ok, result}

      {:error, %Refusal{} = refusal} ->
        {:error, if(ambiguous?(refusal), do: ambiguous(refusal.details), else: refusal)}
    end
  end

  @doc "SHACL admission: `:data` (text or `%{text: _}`) and `:shapes`; N3 data is refused, never down-converted."
  @spec admit_shacl(keyword() | map(), keyword()) ::
          {:ok, GraphlawEx.Result.Shacl.t()} | {:error, Refusal.t()}
  def admit_shacl(args, opts \\ []) do
    with {:ok, args} <- check_shape("admit_shacl", args, opts),
         {:ok, text} <- data_text("admit_shacl", :data, Map.get(args, "data")),
         {:ok, %SniffResult{dialect: dialect}} <- sniff(text, opts),
         :ok <- require_tier(dialect, :dataset, "shacl") do
      Dispatch.run("shacl", args, op_opts(opts))
    end
  end

  @doc "N3 admission: `:text` must sniff to N3; the logic tier never receives or yields a dataset."
  @spec admit_n3(keyword() | map(), keyword()) ::
          {:ok, GraphlawEx.Result.N3.t()} | {:error, Refusal.t()}
  def admit_n3(args, opts \\ []) do
    with {:ok, args} <- check_shape("admit_n3", args, opts),
         {:ok, text} <- data_text("admit_n3", :text, Map.get(args, "text")),
         {:ok, %SniffResult{dialect: dialect}} <- sniff(text, opts),
         :ok <- require_tier(dialect, :logic, "n3") do
      Dispatch.run("n3", args, op_opts(opts))
    end
  end

  defp check_shape(fun, args, opts) do
    with true <- is_list(opts) and Keyword.keyword?(opts),
         {:ok, normalized} <- Coerce.normalize(fun, args) do
      {:ok, normalized}
    else
      {:error, %Refusal{}} = error ->
        error

      _ ->
        {:error,
         Refusal.build(
           :invalid_capability_request,
           "Dialect.#{fun}/2: args must be a keyword list or map and opts a keyword list",
           %{
             "type_errors" => [
               %{"field" => "opts", "expected" => "keyword", "got" => inspect(opts)}
             ]
           }
         )}
    end
  end

  # :hint is a sniff-only option; the shacl/n3 ops take transport options only.
  defp op_opts(opts), do: Keyword.delete(opts, :hint)

  defp require_tier(dialect, want, op) do
    case tier(dialect) do
      ^want ->
        :ok

      :unknown ->
        {:error, ambiguous(%{"op" => op, "dialect" => inspect(dialect)})}

      _ ->
        mismatch(dialect, want, op)
    end
  end

  defp mismatch(dialect, want, op) do
    {:error,
     Refusal.build(
       :REFUSED_TIER_MISMATCH,
       "#{op} requires a #{want}-tier document; sniffed #{dialect} (#{tier(dialect)} tier, no down-conversion)",
       %{"op" => op, "dialect" => dialect}
     )}
  end

  defp data_text(_fun, _key, text) when is_binary(text), do: {:ok, text}
  defp data_text(_fun, _key, %{text: text}) when is_binary(text), do: {:ok, text}
  defp data_text(_fun, _key, %{"text" => text}) when is_binary(text), do: {:ok, text}

  defp data_text(fun, key, other),
    do:
      {:error,
       Refusal.build(
         :invalid_capability_request,
         "Dialect.#{fun}/2: args #{inspect(key)} must be document text (a binary or %{text: binary}), got #{inspect(other)}",
         %{"field" => Atom.to_string(key), "got" => inspect(other)}
       )}

  defp ambiguous?(%Refusal{details: details, raw: raw}) do
    kind = Map.get(details, "kind") || (is_map(raw) && Map.get(raw, "kind"))
    kind == "Ambiguous"
  end

  defp ambiguous(details) do
    Refusal.build(
      :REFUSED_AMBIGUOUS_DIALECT,
      "dialect is ambiguous or unrecognised; refusing to guess",
      details
    )
  end
end
