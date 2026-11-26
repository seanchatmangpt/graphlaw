# SPDX-FileCopyrightText: 2026 ash_graphlaw contributors <https://github.com/seanchatmangpt/ash_graphlaw/graphs/contributors>
#
# SPDX-License-Identifier: MIT

defmodule GraphlawEx.Capability.SparqlQuery do
  @moduledoc """
  Hand-written FR-2 bulk SPARQL query over raw RDF text.

  Lives outside the projected `GraphlawEx.Capability.Sparql` so re-projection cannot drop it.
  `GraphlawEx.Capability.Sparql.query/3` delegates here via `GraphlawEx.Capability.Extension`.
  """

  alias GraphlawEx.Capability.Refusal
  alias GraphlawEx.Capability.Sparql
  alias GraphlawEx.Result.Sparql, as: Result

  @default_max_rows 10_000
  @default_max_bytes 16 * 1024 * 1024

  # Query-level options plus the transport options Authority.engine_opts/1 understands.
  @allowed_opts [
    :max_rows,
    :max_bytes,
    :timeout_ms,
    :dialect,
    :base,
    :server,
    :timeout,
    :lease,
    :trusted_keys,
    :max_skew_secs,
    :source_digest
  ]

  @doc """
  One engine crossing over raw N-Quads/N-Triples/Turtle text (dialect sniffed).

  Options: `:max_rows` (positive integer, default 10_000; a solution set above it refuses
  `:REFUSED_RESOURCE_BOUND`), `:timeout_ms` (positive integer, forwarded to the host worker call, which
  recycles the instance on expiry), `:max_bytes` (positive integer, default 16 MiB; a larger
  `nquads` refuses `:REFUSED_RESOURCE_BOUND` before crossing into WASM). A non-positive or
  non-integer `:max_rows`/`:timeout_ms`/`:max_bytes`, or non-binary `nquads`/`sparql`, refuses
  `:REFUSED_INVALID_INPUT`.
  `:dialect`, `:base`, plus any `GraphlawEx.Capability.Sparql.run/2` option. An engine refusal
  that names an unsupported SPARQL feature is re-coded `:REFUSED_UNSUPPORTED_SPARQL_FEATURE`.

  The engine wire has no row cap, so `:max_rows` is enforced on the decoded response
  (post-engine), not in-worker: the engine still materializes the full solution set and the
  response is decoded before the cap is applied. It bounds what is returned to the caller, not
  engine memory or CPU; use `:timeout_ms` and `:max_bytes` to bound those.
  """
  @spec query(String.t(), String.t(), keyword()) :: {:ok, Result.t()} | {:error, Refusal.t()}
  def query(nquads, sparql, opts \\ [])

  def query(nquads, sparql, opts)
      when is_binary(nquads) and is_binary(sparql) and is_list(opts) do
    with :ok <- check_keyword(opts),
         :ok <- check_known(opts),
         {:ok, max_rows} <- positive_int(opts, :max_rows, @default_max_rows),
         {:ok, _timeout} <- positive_int(opts, :timeout_ms, :unset),
         {:ok, max_bytes} <- positive_int(opts, :max_bytes, @default_max_bytes),
         :ok <- check_size(nquads, max_bytes) do
      run(nquads, sparql, opts, max_rows)
    end
  end

  def query(nquads, sparql, opts) do
    bad =
      [
        {"nquads", is_binary(nquads)},
        {"sparql", is_binary(sparql)},
        {"opts", is_list(opts)}
      ]
      |> Enum.reject(&elem(&1, 1))
      |> Enum.map(&elem(&1, 0))

    {:error,
     Refusal.build(
       :REFUSED_INVALID_INPUT,
       "SparqlQuery.query/3: argument(s) #{Enum.join(bad, ", ")} have the wrong type " <>
         "(nquads and sparql must be binaries, opts a keyword list)",
       %{
         "nquads_binary" => is_binary(nquads),
         "sparql_binary" => is_binary(sparql),
         "opts_list" => is_list(opts),
         "invalid_args" => bad
       }
     )}
  end

  defp check_keyword(opts) do
    if Keyword.keyword?(opts) do
      :ok
    else
      {:error,
       Refusal.build(
         :REFUSED_INVALID_INPUT,
         "SparqlQuery.query/3: opts must be a keyword list of {atom, value} pairs, got #{inspect(opts)}",
         %{"option" => "opts", "got" => inspect(opts)}
       )}
    end
  end

  defp check_known(opts) do
    case opts |> Keyword.keys() |> Enum.uniq() |> Enum.reject(&(&1 in @allowed_opts)) do
      [] ->
        :ok

      unknown ->
        {:error,
         Refusal.build(
           :REFUSED_INVALID_INPUT,
           "SparqlQuery.query/3: unknown option #{Enum.map_join(unknown, ", ", &inspect/1)}; " <>
             "allowed: #{Enum.map_join(@allowed_opts, ", ", &inspect/1)}",
           %{
             "unknown_options" => Enum.map(unknown, &Atom.to_string/1),
             "allowed" => Enum.map(@allowed_opts, &Atom.to_string/1)
           }
         )}
    end
  end

  defp positive_int(opts, key, default) do
    case Keyword.fetch(opts, key) do
      {:ok, n} when is_integer(n) and n > 0 ->
        {:ok, n}

      {:ok, other} ->
        {:error,
         Refusal.build(
           :REFUSED_INVALID_INPUT,
           "SparqlQuery.query/3: option #{inspect(key)} must be a positive integer, got #{inspect(other)}",
           %{"option" => Atom.to_string(key), "got" => inspect(other)}
         )}

      :error ->
        {:ok, default}
    end
  end

  defp check_size(nquads, max_bytes) do
    size = byte_size(nquads)

    if size > max_bytes do
      {:error,
       Refusal.build(
         :REFUSED_RESOURCE_BOUND,
         "request nquads (#{size} bytes) exceed max_bytes (#{max_bytes})",
         %{
           "limit" => "max_bytes",
           "max" => max_bytes,
           "observed" => size
         }
       )}
    else
      :ok
    end
  end

  defp run(nquads, sparql, opts, max_rows) do
    spec = %{"text" => nquads} |> put_opt("dialect", opts[:dialect])
    args = [data: spec, query: sparql] ++ if(opts[:base], do: [base: opts[:base]], else: [])
    run_opts = Keyword.drop(opts, [:max_rows, :max_bytes, :dialect, :base])

    case Sparql.run(args, run_opts) do
      {:ok, %Result{rows: rows} = result} when is_list(rows) ->
        enforce_rows(result, length(rows), max_rows)

      {:ok, result} ->
        {:ok, result}

      {:error, %Refusal{message: message} = refusal} ->
        {:error, classify_unsupported(refusal, message)}
    end
  end

  defp enforce_rows(result, count, max_rows) when count <= max_rows, do: {:ok, result}

  defp enforce_rows(_result, count, max_rows) do
    {:error,
     Refusal.build(
       :REFUSED_RESOURCE_BOUND,
       "SPARQL solutions (#{count}) exceed max_rows (#{max_rows}); raise :max_rows or narrow the query",
       %{"limit" => "max_rows", "max" => max_rows, "observed" => count}
     )}
  end

  defp put_opt(map, _key, nil), do: map
  defp put_opt(map, key, value), do: Map.put(map, key, to_string(value))

  defp classify_unsupported(%Refusal{code: code} = refusal, message)
       when code in [:engine_refused, :not_admitted] and is_binary(message) do
    if String.match?(message, ~r/unsupported|not supported|not implemented/i) do
      %Refusal{refusal | code: :REFUSED_UNSUPPORTED_SPARQL_FEATURE}
    else
      refusal
    end
  end

  defp classify_unsupported(refusal, _message), do: refusal
end
