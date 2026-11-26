# SPDX-License-Identifier: MIT
# projection target: ontologies/graphlaw-governance.ttl; hand-written until projected
defmodule GraphlawEx.Error.Table do
  @moduledoc """
  Pure data for the public error contract: the closed public code set, the
  refusal-code to public-code groups, the public-code to class table and the
  raw-SQL hint markers. `GraphlawEx.Error` and `GraphlawEx.Error.Class` own
  the logic.

      iex> GraphlawEx.Error.Table.base_codes()[:saturated]
      :REFUSED_RESOURCE_BOUND
  """

  @codes [
    :REFUSED_ARTIFACT_INTEGRITY,
    :REFUSED_INVALID_INPUT,
    :REFUSED_AMBIGUOUS_DIALECT,
    :REFUSED_UNSUPPORTED_SPARQL_FEATURE,
    :REFUSED_RESOURCE_BOUND,
    :REFUSED_MEMORY_LIMIT_EXCEEDED,
    :REFUSED_QUERY_TIMEOUT,
    :REFUSED_WASM_TRAP,
    :REFUSED_RAW_SQL_FORBIDDEN,
    :REFUSED_LEASE_REQUIRED
  ]

  @base_code_groups [
    {:REFUSED_ARTIFACT_INTEGRITY,
     [:wasm_digest_mismatch, :wasm_import_surface_mismatch, :receipt_refused]},
    {:REFUSED_QUERY_TIMEOUT, [:call_timeout]},
    {:REFUSED_WASM_TRAP, [:call_trapped, :call_exited]},
    {:REFUSED_RESOURCE_BOUND,
     [:saturated, :fuel_exhausted, :resource_limit, :batch_member_failed, :duckdb_failure]},
    # Input of the wrong tier is an invalid input, never a new public code.
    {:REFUSED_INVALID_INPUT, [:REFUSED_TIER_MISMATCH]},
    {:REFUSED_LEASE_REQUIRED,
     [
       :ceiling_unmet,
       :lease_revoked,
       :lease_refused,
       :unverified_lease_refused,
       :receipt_required
     ]}
  ]

  @base_codes for {public, group} <- @base_code_groups,
                  code <- group,
                  into: %{},
                  do: {code, public}

  @classes %{
    REFUSED_ARTIFACT_INTEGRITY: :invalid,
    REFUSED_INVALID_INPUT: :invalid,
    REFUSED_AMBIGUOUS_DIALECT: :invalid,
    REFUSED_UNSUPPORTED_SPARQL_FEATURE: :invalid,
    REFUSED_RESOURCE_BOUND: :invalid,
    REFUSED_RAW_SQL_FORBIDDEN: :forbidden,
    REFUSED_LEASE_REQUIRED: :forbidden,
    REFUSED_MEMORY_LIMIT_EXCEEDED: :unknown,
    REFUSED_QUERY_TIMEOUT: :unknown,
    REFUSED_WASM_TRAP: :unknown
  }

  # Refusal codes that are not caller-fixable: operational host failures
  # (queue shed, DuckDB engine failure) and an engine variant this library does
  # not know. Their public code stays `:REFUSED_RESOURCE_BOUND` /
  # `:REFUSED_INVALID_INPUT` (closed set), but `GraphlawEx.Error.Class` files
  # them under Splode `:unknown`, as it does an unclassified non-refusal term.
  @unknown_refusal_codes [:saturated, :duckdb_failure, :engine_unclassified]

  @raw_sql_markers ["sqlrdf-17", "raw-sql", "raw_sql", "raw sql"]

  # Compile-time coverage: every public code has a class, no class is stray,
  # and every refusal-code group lands inside the closed set. A new code that
  # forgets its class fails the build instead of raising KeyError at runtime.
  missing = @codes -- Map.keys(@classes)
  stray = Map.keys(@classes) -- @codes
  bad_class = for {c, v} <- @classes, v not in [:forbidden, :invalid, :unknown], do: c
  bad_base = for {c, p} <- @base_codes, p not in @codes, do: c

  if missing != [] or stray != [] or bad_class != [] or bad_base != [] do
    raise CompileError,
      description:
        "GraphlawEx.Error.Table inconsistent: missing classes #{inspect(missing)}, " <>
          "stray #{inspect(stray)}, bad class #{inspect(bad_class)}, " <>
          "base codes outside the public set #{inspect(bad_base)}"
  end

  @doc "The closed public code set."
  @spec codes() :: [atom()]
  def codes, do: @codes

  @doc "Refusal-code groups: `[{public_code, [refusal_code]}]`."
  @spec base_code_groups() :: [{atom(), [atom()]}]
  def base_code_groups, do: @base_code_groups

  @doc "Flattened refusal code to public code map (codes outside it are invalid input)."
  @spec base_codes() :: %{atom() => atom()}
  def base_codes, do: @base_codes

  @doc "Public code to Splode/Ash-style class."
  @spec classes() :: %{atom() => :forbidden | :invalid | :unknown}
  def classes, do: @classes

  @doc """
  Refusal codes classified `:unknown` regardless of their public code's class
  (operational or unclassified, not caller-fixable).
  """
  @spec unknown_refusal_codes() :: [atom()]
  def unknown_refusal_codes, do: @unknown_refusal_codes

  @doc "Lowercase substrings marking an engine SQLRDF-17 / raw-SQL refusal."
  @spec raw_sql_markers() :: [String.t()]
  def raw_sql_markers, do: @raw_sql_markers
end
