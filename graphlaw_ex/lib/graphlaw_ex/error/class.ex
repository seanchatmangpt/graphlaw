# SPDX-License-Identifier: MIT
defmodule GraphlawEx.Error.Class do
  @moduledoc """
  Pure mapping from `GraphlawEx.Error` codes to Splode/Ash-style error classes.

  graphlaw_ex has no Splode or Ash dependency; this module only names the class
  a downstream framework should file an error under. Classes:

    * `:forbidden` - raw SQL forbidden or a lease/receipt is required; the
      caller cannot fix it by changing the request (Ash `Forbidden`).
    * `:invalid` - artifact integrity, malformed or ambiguous input, an
      unsupported feature or a resource bound; fix the input and retry (Ash
      `Invalid`).
    * `:unknown` - memory limit, timeout or WASM trap; not attributable to
      the input's shape (Ash `Unknown`).

  | code | class |
  |---|---|
  | `:REFUSED_ARTIFACT_INTEGRITY` | `:invalid` |
  | `:REFUSED_INVALID_INPUT` | `:invalid` |
  | `:REFUSED_AMBIGUOUS_DIALECT` | `:invalid` |
  | `:REFUSED_UNSUPPORTED_SPARQL_FEATURE` | `:invalid` |
  | `:REFUSED_RESOURCE_BOUND` | `:invalid` |
  | `:REFUSED_RAW_SQL_FORBIDDEN` | `:forbidden` |
  | `:REFUSED_LEASE_REQUIRED` | `:forbidden` |
  | `:REFUSED_MEMORY_LIMIT_EXCEEDED` | `:unknown` |
  | `:REFUSED_QUERY_TIMEOUT` | `:unknown` |
  | `:REFUSED_WASM_TRAP` | `:unknown` |

  ## Decision: operational and unclassified failures are `:unknown`

  The table above is per public code. Two refinements sit on top, because a
  public code is coarser than the failure behind it (decision, lane E4):

    * Operational host failures (`:saturated` shed load, `:duckdb_failure`) and
      an engine variant this library does not know (`:engine_unclassified`) are
      not fixable by changing the request, so they classify `:unknown` even
      though their public code is `:REFUSED_RESOURCE_BOUND` /
      `:REFUSED_INVALID_INPUT`. Detected from `evidence["refusal_code"]`; see
      `GraphlawEx.Error.Table.unknown_refusal_codes/0`.
    * A non-refusal term wrapped by `GraphlawEx.Error.wrap/1` (the
      "unclassified failure" digest-tag error) is `:unknown`.
    * Caller-fixable failures (malformed input, `:resource_limit` size caps)
      stay `:invalid`.

  ## Wrapping from ash_r2rml

  The consumer normalizes with `GraphlawEx.Error.wrap/1` (total, never raises),
  asks for the class, and picks its own Splode module:

      def to_ash(term) do
        error = GraphlawEx.Error.wrap(term)

        case GraphlawEx.Error.Class.of(error) do
          :forbidden -> AshR2rml.Error.Forbidden.exception(error: error)
          :invalid -> AshR2rml.Error.Invalid.exception(error: error)
          :unknown -> AshR2rml.Error.Unknown.exception(error: error)
        end
      end

  Keep the `GraphlawEx.Error` struct as the wrapped payload: `code`, `class`
  (taxonomy), `detail` and string-keyed `evidence` stay available for policies
  and logs. The `class` field of `GraphlawEx.Error` is the Chatman taxonomy
  class and is unrelated to the value returned here: `Class.of/1` (alias
  `Class.category/1`) returns the framework category, never `error.class`.

  ## Examples

      iex> GraphlawEx.Error.Class.of(:REFUSED_LEASE_REQUIRED)
      :forbidden

      iex> GraphlawEx.Error.Class.of(:REFUSED_ARTIFACT_INTEGRITY)
      :invalid

      iex> GraphlawEx.Error.Class.of(%GraphlawEx.Error{code: :REFUSED_QUERY_TIMEOUT})
      :unknown

      iex> GraphlawEx.Error.Class.of({:nif_error, :boom})
      :unknown

      iex> GraphlawEx.Error.Class.of(:not_a_code)
      :invalid

      iex> GraphlawEx.Error.Class.of(:saturated)
      :unknown

      iex> GraphlawEx.Error.Class.of(GraphlawEx.Refusal.new(:saturated))
      :unknown

      iex> GraphlawEx.Error.Class.of(GraphlawEx.Refusal.new(:resource_limit))
      :invalid
  """

  alias GraphlawEx.Error

  @type t :: :forbidden | :invalid | :unknown

  @table GraphlawEx.Error.Table.classes()
  @unknown_refusal_codes GraphlawEx.Error.Table.unknown_refusal_codes()
  @unknown_refusal_code_strings Enum.map(@unknown_refusal_codes, &Atom.to_string/1)

  # Engine refusal kind -> class, derived from the generated registry (the single source:
  # `refusal_kind_classes` in registry/capability-registry.json, emitted by
  # src/registry/emit_json.rs and projected by the pack template). `Introspection.Tables`
  # delegates here.
  #
  # Kind class vs code class may differ, deliberately: a kind classifies the
  # engine refusal family, a code classifies the wire error. Documented
  # divergences (see guides/refusals_and_errors.md):
  #   ResourceLimit (kind :forbidden) vs REFUSED_RESOURCE_BOUND (code :invalid)
  #   Unsupported   (kind :unknown)   vs REFUSED_UNSUPPORTED_SPARQL_FEATURE (code :invalid)
  @kind_table Map.new(
                GraphlawEx.Capability.Registry.refusal_kind_classes(),
                &{&1.kind, &1.error_class}
              )

  @registry_kinds GraphlawEx.Capability.Registry.refusal_kinds()
  @unclassified Enum.reject(@registry_kinds, &Map.has_key?(@kind_table, &1))

  if @registry_kinds == [] do
    raise CompileError,
      description: "GraphlawEx.Error.Class: registry refusal_kinds is empty (vacuous check)"
  end

  if @unclassified != [] do
    raise CompileError,
      description:
        "GraphlawEx.Error.Class kind table lacks registry refusal kinds: " <>
          inspect(@unclassified)
  end

  @doc "The code-to-class table, one entry per `GraphlawEx.Error.codes/0` code."
  @spec table() :: %{Error.code() => t()}
  def table, do: @table

  @doc "The refusal-kind-to-class table (kind strings as in the registry vocabulary)."
  @spec kind_table() :: %{String.t() => t()}
  def kind_table, do: @kind_table

  @doc """
  Class for an engine refusal kind. A kind outside the table is `:unknown`
  (every kind in the committed registry is covered at compile time).

      iex> GraphlawEx.Error.Class.of_kind("ResourceLimit")
      :forbidden

      iex> GraphlawEx.Error.Class.of_kind("nope")
      :unknown
  """
  @spec of_kind(String.t()) :: t()
  def of_kind(kind), do: Map.get(@kind_table, kind, :unknown)

  @doc """
  Framework category (`:forbidden | :invalid | :unknown`) for an error code atom, a `GraphlawEx.Error`, or any term that
  `GraphlawEx.Error.wrap/1` accepts. Atoms resolve through
  `GraphlawEx.Error.code_for/1`, so an unknown atom is `:REFUSED_INVALID_INPUT`
  and therefore `:invalid`, agreeing with `code_for/1`. Total: never raises.
  """
  @spec of(term()) :: t()
  def of(%Error{code: code, evidence: evidence}) do
    if not_caller_fixable?(evidence), do: :unknown, else: lookup(code)
  end

  def of(code) when is_atom(code) do
    if code in @unknown_refusal_codes, do: :unknown, else: lookup(code)
  end

  def of(other), do: other |> Error.wrap() |> of()

  # Operational host failures and unclassified terms are Splode :unknown, not
  # caller-fixable :invalid. Evidence is string-keyed (see `Error`): a refusal
  # keeps its original code under "refusal_code"; a non-refusal term has only
  # the "term_*" digest tag.
  # An `engine_unclassified` refusal whose engine kind is `EngineRejected` is the engine
  # refusing the caller's input (e.g. SPARQL UPDATE / SERVICE): caller-fixable, :invalid.
  defp not_caller_fixable?(%{
         "refusal_code" => "engine_unclassified",
         "details" => %{"kind" => "EngineRejected"}
       }),
       do: false

  defp not_caller_fixable?(%{"refusal_code" => rc}) when is_binary(rc),
    do: rc in @unknown_refusal_code_strings

  defp not_caller_fixable?(%{"term_type" => _}), do: true
  defp not_caller_fixable?(_), do: false

  defp lookup(code), do: Map.get(@table, Error.code_for(code), :unknown)

  @doc """
  Alias of `of/1` with a name that does not collide with the `class` field of
  `GraphlawEx.Error` (the Chatman taxonomy class). Returns the framework
  category: `:forbidden | :invalid | :unknown`.

      iex> GraphlawEx.Error.Class.category(:REFUSED_LEASE_REQUIRED)
      :forbidden
  """
  @spec category(term()) :: t()
  def category(term), do: of(term)
end
