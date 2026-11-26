# SPDX-License-Identifier: MIT
defmodule GraphlawEx.Capability.Sql do
  @moduledoc ~S"""
  Hand-written SQL-in-RDF capability.

  The ABI exposes no standalone `sql` op: SQL-in-RDF is the `sql` dialect
  (Turtle plus a closed-world shape check, `parse`/`convert`) and the `sql`
  step of the `law` op (`admit:sql`, ceiling `observe`). `run/2` sends the
  admitted sql-ast graph through `GraphlawEx.Capability.Law` with a single
  `sql` step.

  Raw SQL text is never sent to the engine: it is refused client-side with
  `:REFUSED_RAW_SQL_FORBIDDEN`. An engine refusal carrying `SQLRDF-17` maps to
  the same code.

      iex> lease = %{signed_lease: %{"lease" => %{"ceiling" => "construct"}, "attestation" => %{}}}
      iex> {:error, r} = GraphlawEx.Capability.Sql.run([text: "SELECT a FROM t"], lease: lease)
      iex> GraphlawEx.Error.wrap(r).code
      :REFUSED_RAW_SQL_FORBIDDEN

      iex> GraphlawEx.Capability.Sql.raw_sql?("@prefix sql: <http://graphlaw.org/sql#> .")
      false
  """

  alias GraphlawEx.Authority
  alias GraphlawEx.Capability.Coerce
  alias GraphlawEx.Capability.Dispatch
  alias GraphlawEx.Capability.Law
  alias GraphlawEx.Capability.Refusal
  alias GraphlawEx.Error

  @sql_start ~r/\A\s*(?:--[^\n]*\n\s*)*(?:select|insert|update|delete|create|drop|alter|with|merge|truncate|copy|attach|pragma|explain|call|set)\b/i
  @rdf_marker ~r/(?:@prefix|@base|^\s*prefix\s|^\s*base\s|<http)/im

  @doc "True when `text` looks like raw SQL rather than an RDF (sql-ast) document."
  @spec raw_sql?(term()) :: boolean()
  def raw_sql?(text) when is_binary(text),
    do: Regex.match?(@sql_start, text) and not Regex.match?(@rdf_marker, text)

  def raw_sql?(_), do: false

  @doc """
  Admits the sql-ast graph in `args[:text]` (Turtle) on the engine and returns
  the decoded law result. `opts` are the options of `GraphlawEx.Capability.call/2`.
  """
  @spec run(keyword() | map(), keyword()) :: {:ok, term()} | {:error, Refusal.t()}
  def run(args, opts \\ []) do
    with {:ok, args} <- Coerce.normalize("sql", args),
         :ok <- Authority.check_op("sql", opts[:lease]) do
      case Map.get(args, "text") do
        text when is_binary(text) ->
          if raw_sql?(text), do: {:error, raw_refusal(true)}, else: execute(text, opts)

        _ ->
          {:error,
           Refusal.build(
             :invalid_capability_request,
             "sql capability requires a `text` binary (sql-ast Turtle)"
           )}
      end
    end
  end

  defp execute(text, opts) do
    data = %{"text" => text, "dialect" => "sql"}
    args = [data: data, steps: [%{"step" => "sql"}]]

    # Law args carry signed_lease/trusted_keys via Dispatch.lease_args (trust
    # anchors from Application env, fail closed when absent); engine options are
    # Authority.engine_opts. Caller :lease/:trusted_keys never reach call/2.
    with {:ok, args} <- Dispatch.lease_args(args, Law, opts),
         {:ok, result} <- Law.run(args, law_opts(opts)) do
      {:ok, result}
    else
      {:error, reason} -> {:error, classify(reason)}
    end
  end

  defp law_opts(opts) do
    opts
    |> Keyword.drop([:lease, :trusted_keys, :unverified_lease, :now_unix, :signed_lease])
    |> Keyword.merge(Authority.engine_opts(opts))
  end

  defp classify(%Refusal{} = r) do
    if Error.raw_sql_hint?(String.downcase(inspect(r.raw) <> " " <> to_string(r.message))),
      do: raw_refusal(false),
      else: r
  end

  defp raw_refusal(client_side?) do
    Refusal.build(
      :invalid_capability_request,
      "raw SQL text is forbidden; submit an sql-ast RDF graph",
      %{
        "kind" => "raw_sql_forbidden",
        "refusal_code" => "raw_sql_forbidden",
        "client_side" => client_side?
      }
    )
  end
end
