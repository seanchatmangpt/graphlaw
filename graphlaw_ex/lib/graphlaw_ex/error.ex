# SPDX-License-Identifier: MIT
defmodule GraphlawEx.Error do
  @moduledoc """
  Public error contract (FR-5): one exception with a fixed code set.

  Every GraphLaw refusal, WASM trap and timeout maps onto one of the ten
  `codes/0`. Lease/authority refusals (`:lease_revoked`, `:lease_refused`,
  `:unverified_lease_refused`, `:receipt_required`, `:ceiling_unmet`) map to
  `:REFUSED_LEASE_REQUIRED`; engine SQLRDF-17 / raw-SQL refusals map to
  `:REFUSED_RAW_SQL_FORBIDDEN`; `:saturated` maps to `:REFUSED_RESOURCE_BOUND`.
  The original fine-grained code and taxonomy term stay in `:evidence`
  (string-keyed, wire-safe) together with a whitelisted subset of the
  structured details (`kind`, `code`, `step`, `reason`, ...). The engine payload
  (`raw`) is never copied: unknown terms contribute only a type tag, byte size
  and digest prefix (`term_type`, `term_bytes`, `term_sha256_prefix`). `inspect/2`
  hides `:evidence`.

      iex> error = GraphlawEx.Error.from_refusal(GraphlawEx.Refusal.call_timeout())
      iex> {error.code, error.class, error.evidence["refusal_code"]}
      {:REFUSED_QUERY_TIMEOUT, :blocked_resource, "call_timeout"}

      iex> GraphlawEx.Error.wrap({:nif_error, :boom}).code
      :REFUSED_INVALID_INPUT
  """

  alias GraphlawEx.Capability.Refusal, as: CapRefusal
  alias GraphlawEx.Error.Class
  alias GraphlawEx.Error.Table
  alias GraphlawEx.Refusal

  @codes Table.codes()

  # Normalized (see `map_code/4`) whole-token matches on structured engine fields.
  @raw_sql_tokens [
    "rawsqlforbidden"
    | for(m <- Table.raw_sql_markers(), do: String.replace(m, ["_", "-", " "], ""))
  ]
  @sparql_tokens ["unsupportedsparqlfeature", "unsupportedsparql", "sparqlunsupported"]
  @memory_tokens ["memorylimit", "memorylimitexceeded", "memory"]

  @type code ::
          :REFUSED_ARTIFACT_INTEGRITY
          | :REFUSED_RAW_SQL_FORBIDDEN
          | :REFUSED_LEASE_REQUIRED
          | :REFUSED_INVALID_INPUT
          | :REFUSED_AMBIGUOUS_DIALECT
          | :REFUSED_UNSUPPORTED_SPARQL_FEATURE
          | :REFUSED_RESOURCE_BOUND
          | :REFUSED_MEMORY_LIMIT_EXCEEDED
          | :REFUSED_QUERY_TIMEOUT
          | :REFUSED_WASM_TRAP

  @type t :: %__MODULE__{
          code: code(),
          class: atom() | nil,
          detail: String.t(),
          evidence: map()
        }

  # Evidence is whitelisted at build time, but is still not printed by inspect.
  @derive {Inspect, except: [:evidence]}
  defexception code: :REFUSED_INVALID_INPUT, class: nil, detail: "", evidence: %{}

  @doc "The closed public code set."
  @spec codes() :: [code()]
  def codes, do: @codes

  @doc """
  Builds the exception for `raise GraphlawEx.Error, ...`. Total: never raises.

    * a binary becomes `detail` under `:REFUSED_INVALID_INPUT`;
    * a keyword list takes `:code` (normalized through `code_for/1`, so an
      arbitrary or unknown code can never escape the closed set), `:detail`
      (or `:message`), `:class` and `:evidence` (a map); unknown keys are ignored;
    * an existing error is returned as is; any other term goes through `wrap/1`.

      iex> raise GraphlawEx.Error, "bad input"
      ** (GraphlawEx.Error) REFUSED_INVALID_INPUT: bad input

      iex> GraphlawEx.Error.exception(code: :bogus, detail: "x").code
      :REFUSED_INVALID_INPUT

      iex> GraphlawEx.Error.exception(code: :lease_revoked).code
      :REFUSED_LEASE_REQUIRED
  """
  @impl true
  @spec exception(term()) :: t()
  def exception(%__MODULE__{} = error), do: error

  def exception(message) when is_binary(message),
    do: %__MODULE__{code: :REFUSED_INVALID_INPUT, detail: message}

  def exception(opts) when is_list(opts) do
    if opts != [] and Keyword.keyword?(opts) do
      detail = Keyword.get(opts, :detail) || Keyword.get(opts, :message)
      class = Keyword.get(opts, :class)
      evidence = Keyword.get(opts, :evidence)

      %__MODULE__{
        code: code_for(Keyword.get(opts, :code, :REFUSED_INVALID_INPUT)),
        class: if(is_atom(class), do: class),
        detail: if(is_binary(detail), do: detail, else: ""),
        evidence: if(is_map(evidence), do: evidence, else: %{})
      }
    else
      wrap(opts)
    end
  end

  def exception(other), do: wrap(other)

  @doc """
  The Chatman taxonomy class carried in the `:class` field (`nil` when none).

  The field name `:class` collides with the Splode/Ash notion of class, which
  is `GraphlawEx.Error.Class.of/1` (alias `category/1`). The field is kept
  (renaming it would break callers); use this accessor and `Class.category/1`
  to say which one you mean.

      iex> error = GraphlawEx.Error.from_refusal(GraphlawEx.Refusal.new(:saturated))
      iex> {GraphlawEx.Error.taxonomy_class(error), GraphlawEx.Error.Class.category(error)}
      {:blocked_resource, :unknown}
  """
  @spec taxonomy_class(t()) :: atom() | nil
  def taxonomy_class(%__MODULE__{class: class}), do: class

  @doc """
  Aggregates child errors into one error, with precedence
  `:forbidden > :invalid > :unknown` (framework category, `Class.of/1`).

  Each child is any term `wrap/1` accepts, or `{path, term}` with a list
  `path` locating it (default `[]`). The result takes the code, taxonomy class
  and detail count from the first child of the winning category, and keeps
  every child, in order, under `evidence["errors"]` as string-keyed maps
  (`path`, `code`, `category`, `detail`, plus the child's whitelisted
  `evidence`). Raises `ArgumentError` on an empty list.

      iex> a = GraphlawEx.Refusal.new(:resource_limit)
      iex> b = GraphlawEx.Refusal.new(:lease_revoked)
      iex> e = GraphlawEx.Error.combine([{["rows", 0], a}, {["lease"], b}])
      iex> {e.code, e.evidence["category"], length(e.evidence["errors"])}
      {:REFUSED_LEASE_REQUIRED, "forbidden", 2}
      iex> hd(e.evidence["errors"])["path"]
      ["rows", "0"]
  """
  @spec combine([term()]) :: t()
  def combine([_ | _] = children) do
    entries =
      Enum.map(children, fn
        {path, term} when is_list(path) -> {path, wrap(term)}
        term -> {[], wrap(term)}
      end)

    categorized = Enum.map(entries, fn {path, e} -> {path, e, Class.of(e)} end)
    {_path, winner, category} = Enum.min_by(categorized, fn {_, _, c} -> rank(c) end)

    %__MODULE__{
      code: winner.code,
      class: winner.class,
      detail: "#{length(entries)} errors; first #{category}: #{winner.detail}",
      evidence: %{
        "category" => Atom.to_string(category),
        "errors" =>
          for {path, e, c} <- categorized do
            %{
              "path" => Enum.map(path, &scalar_string/1),
              "code" => Atom.to_string(e.code),
              "category" => Atom.to_string(c),
              "detail" => e.detail,
              "evidence" => e.evidence
            }
          end
      }
    }
  end

  def combine(other),
    do: raise(ArgumentError, "combine/1 needs a non-empty list, got: #{inspect(other, limit: 3)}")

  defp rank(:forbidden), do: 0
  defp rank(:invalid), do: 1
  defp rank(:unknown), do: 2

  @impl true
  def message(%__MODULE__{code: code, detail: detail}) when detail in [nil, ""], do: "#{code}"
  def message(%__MODULE__{code: code, detail: detail}), do: "#{code}: #{detail}"

  @doc """
  Normalizes any refusal-shaped term (`GraphlawEx.Refusal`,
  `GraphlawEx.Capability.Refusal`, `GraphlawEx.Error.Refused`, an existing
  `GraphlawEx.Error`, `{:error, term}` or an arbitrary term) into an error.
  Total: never raises.
  """
  @spec wrap(term()) :: t()
  def wrap(%__MODULE__{} = error), do: error
  def wrap(%Refusal{} = refusal), do: from_refusal(refusal)
  def wrap(%CapRefusal{} = refusal), do: from_capability(refusal)
  def wrap({:error, inner}), do: wrap(inner)

  # Legacy GraphlawEx.Error.Refused (deprecated): unwrap to its refusal.
  def wrap(%{__struct__: GraphlawEx.Error.Refused, refusal: inner}) when not is_nil(inner),
    do: wrap(inner)

  def wrap(%{__struct__: GraphlawEx.Error.Refused}), do: from_other(:refused_without_refusal)

  def wrap(other), do: from_other(other)

  @doc "Maps a host/capability `GraphlawEx.Refusal` into the public error."
  @spec from_refusal(Refusal.t()) :: t()
  def from_refusal(%Refusal{} = r) do
    build(
      map_code(r.code, r.message, r.details, r.raw),
      r.class,
      r.message,
      r.code,
      r.broken_term,
      r.details,
      r.raw
    )
  end

  @doc "Maps a `GraphlawEx.Capability.Refusal` into the public error."
  @spec from_capability(CapRefusal.t()) :: t()
  def from_capability(%CapRefusal{} = r) do
    class = class_for(r.code)

    build(
      map_code(r.code, r.message, r.details, r.raw),
      class,
      r.message,
      r.code,
      r.broken_term,
      r.details,
      r.raw
    )
  end

  # Total: a type tag + size + digest prefix, never the term's content.
  defp from_other(other) do
    bin = :erlang.term_to_binary(other)
    <<prefix::binary-size(8), _::binary>> = :crypto.hash(:sha256, bin)

    %__MODULE__{
      code: :REFUSED_INVALID_INPUT,
      class: :unsupported,
      detail: "unclassified failure",
      evidence: %{
        "term_type" => type_tag(other),
        "term_bytes" => byte_size(bin),
        "term_sha256_prefix" => Base.encode16(prefix, case: :lower)
      }
    }
  end

  defp type_tag(v) when is_map(v), do: "map"
  defp type_tag(v) when is_binary(v), do: "binary"
  defp type_tag(v) when is_list(v), do: "list"
  defp type_tag(v) when is_tuple(v), do: "tuple"
  defp type_tag(v) when is_atom(v), do: "atom"
  defp type_tag(v) when is_number(v), do: "number"
  defp type_tag(_), do: "term"

  # Keys of `details` allowed into evidence (structured engine fields only).
  @evidence_detail_keys ~w(kind code step reason lease_id member limit observed max)
  @max_evidence_bytes 128

  defp build(code, class, message, orig, broken_term, details, raw) do
    %__MODULE__{
      code: code,
      class: if(is_atom(class), do: class),
      detail: if(is_binary(message), do: message, else: ""),
      evidence:
        %{
          "refusal_code" => scalar_string(orig),
          "broken_term" => if(broken_term, do: scalar_string(broken_term)),
          "details" => whitelist(details, raw)
        }
        |> Enum.reject(fn {_k, v} -> is_nil(v) or v == %{} end)
        |> Map.new()
    }
  end

  # Only whitelisted keys, scalar-clamped; `raw` contributes its structured
  # `kind`/`code` only, never its payload.
  defp whitelist(details, raw) do
    from_raw = if is_map(raw), do: Map.take(stringify_keys(raw), ["kind", "code"]), else: %{}
    from_details = if is_map(details), do: stringify_keys(details), else: %{}

    from_raw
    |> Map.merge(from_details)
    |> Map.take(@evidence_detail_keys)
    |> Map.new(fn {k, v} -> {k, clamp(v)} end)
  end

  defp stringify_keys(m) when is_map(m) and not is_struct(m) do
    Map.new(m, fn
      {k, v} when is_binary(k) -> {k, v}
      {k, v} when is_atom(k) -> {Atom.to_string(k), v}
      {k, v} -> {inspect(k, limit: 5, printable_limit: 32), v}
    end)
  end

  defp stringify_keys(_), do: %{}

  defp clamp(v) when is_number(v) or is_boolean(v) or is_nil(v), do: v
  defp clamp(v) when is_atom(v), do: Atom.to_string(v)
  defp clamp(v) when is_binary(v), do: truncate(v)
  defp clamp(_v), do: "[redacted]"

  defp truncate(v) do
    cond do
      not String.valid?(v) -> "[binary #{byte_size(v)}b]"
      byte_size(v) > @max_evidence_bytes -> String.slice(v, 0, @max_evidence_bytes)
      true -> v
    end
  end

  # Total over every term (tuples, improper lists, pids, binaries).
  defp scalar_string(v) when is_atom(v), do: Atom.to_string(v)
  defp scalar_string(v) when is_binary(v), do: truncate(v)
  defp scalar_string(v), do: inspect(v, limit: 5, printable_limit: 64)

  @blocked_resource_codes [
    :REFUSED_RESOURCE_BOUND,
    :REFUSED_MEMORY_LIMIT_EXCEEDED,
    :REFUSED_QUERY_TIMEOUT
  ]

  defp class_for(code) when code in @blocked_resource_codes, do: :blocked_resource

  defp class_for(code) do
    if code in Refusal.codes(), do: Refusal.class_of(code), else: :refused_structure
  end

  # Engine kind/message hints refine a generic code (kinds mirror graphlaw
  # src/dialect.rs RefusalKind); otherwise the table below decides.
  # Structured fields only (`kind`/`code` of details and raw); the free-text
  # message never participates. Each token is matched whole, after
  # normalization (downcase, `_`/`-`/space removed), never as a substring.
  defp map_code(code, _message, details, raw) do
    tokens = structured_tokens(details, raw)

    cond do
      Enum.any?(tokens, &(&1 in @raw_sql_tokens)) -> :REFUSED_RAW_SQL_FORBIDDEN
      "ambiguous" in tokens -> :REFUSED_AMBIGUOUS_DIALECT
      Enum.any?(tokens, &(&1 in @sparql_tokens)) -> :REFUSED_UNSUPPORTED_SPARQL_FEATURE
      Enum.any?(tokens, &(&1 in @memory_tokens)) -> :REFUSED_MEMORY_LIMIT_EXCEEDED
      true -> base_code(code)
    end
  end

  defp structured_tokens(details, raw) do
    [details, raw]
    |> Enum.filter(&is_map/1)
    |> Enum.flat_map(fn m -> [field(m, "kind"), field(m, "code")] end)
    |> Enum.filter(&is_binary/1)
    |> Enum.map(&normalize_token/1)
  end

  defp field(m, key), do: Map.get(m, key) || Map.get(m, String.to_existing_atom(key))

  defp normalize_token(s), do: s |> String.downcase() |> String.replace(["_", "-", " "], "")

  @doc """
  True when a hint string names an engine SQLRDF-17 / raw-SQL refusal
  (case-insensitive; non-binaries are `false`). The one matcher for that
  refusal; markers live in `GraphlawEx.Error.Table.raw_sql_markers/0`.

      iex> GraphlawEx.Error.raw_sql_hint?("SQLRDF-17 raw sql forbidden")
      true

      iex> GraphlawEx.Error.raw_sql_hint?("ambiguous")
      false

      iex> GraphlawEx.Error.raw_sql_hint?(nil)
      false
  """
  @spec raw_sql_hint?(term()) :: boolean()
  def raw_sql_hint?(hint) when is_binary(hint) do
    down = String.downcase(hint)
    Enum.any?(Table.raw_sql_markers(), &String.contains?(down, &1))
  end

  def raw_sql_hint?(_), do: false

  @doc """
  The public code a refusal code maps to. Total: every `GraphlawEx.Refusal`
  code, the capability-layer extras (`:ceiling_unmet`) and the re-coded
  engine-side atoms (`:REFUSED_*`, including `:REFUSED_TIER_MISMATCH`) land
  inside `codes/0`; anything unknown maps to `:REFUSED_INVALID_INPUT`.

      iex> GraphlawEx.Error.code_for(:saturated)
      :REFUSED_RESOURCE_BOUND

      iex> GraphlawEx.Error.code_for(:REFUSED_TIER_MISMATCH)
      :REFUSED_INVALID_INPUT

      iex> GraphlawEx.Error.code_for(:lease_revoked)
      :REFUSED_LEASE_REQUIRED

      iex> GraphlawEx.Error.code_for(:not_a_code)
      :REFUSED_INVALID_INPUT

      iex> GraphlawEx.Error.code_for(:REFUSED_AMBIGUOUS_DIALECT)
      :REFUSED_AMBIGUOUS_DIALECT
  """
  @spec code_for(term()) :: code()
  def code_for(code), do: base_code(code)

  defp base_code(c) when c in @codes, do: c
  defp base_code(c) when not is_atom(c), do: :REFUSED_INVALID_INPUT

  # Group map lives in `GraphlawEx.Error.Table`; everything else (structure,
  # encoding, policy refusals, unknown atoms) is an invalid input.
  defp base_code(c), do: Map.get(Table.base_codes(), c, :REFUSED_INVALID_INPUT)
end
