defmodule GraphlawEx.Refusal do
  @moduledoc """
  Typed refusal carried by every `{:error, _}` this library returns.

  A refusal is an observation about one exact input: it names a closed
  `t:code/0` and the Chatman failure-taxonomy `broken_term` the refusal
  witnesses. GraphLaw derives and validates; it never authorizes, so a refusal
  never grants or withdraws authority.

  `:raw` keeps the whole engine error map exactly as decoded (lossless, so
  fields this library does not know are never dropped); it is `nil` for
  client-side refusals built with `new/3`.

  The code table is closed: a new engine variant maps to `:engine_unclassified`
  instead of crashing, so a future engine release is forward-compatible —
  `from_engine/2` never raises on an unknown payload shape.

  ## Usage

      iex> refusal = GraphlawEx.Refusal.new(:saturated)
      iex> {refusal.code, refusal.class}
      {:saturated, :blocked_resource}

      iex> {:error, %GraphlawEx.Refusal{code: code}} =
      ...>   {:error, GraphlawEx.Refusal.call_trapped()}
      iex> code
      :call_trapped

  ## Typed failure table

    * `:saturated` - class `:blocked_resource`, witnesses `:R_missing_consequence`. The host queue is full (max_queue), so the request was shed instead of waiting.
    * `:call_timeout` - class `:blocked_resource`, witnesses `:R_missing_consequence`. The call did not finish within the configured timeout_ms; the instance is recycled.
    * `:call_trapped` - class `:blocked_resource`, witnesses `:R_missing_consequence`. The WASM call trapped while executing; the instance is recycled.
    * `:call_exited` - class `:blocked_resource`, witnesses `:R_missing_consequence`. The WASM program exited during the call; the instance is recycled.
    * `:fuel_exhausted` - class `:blocked_resource`, witnesses `:R_missing_consequence`. The call used up its fuel budget before the engine returned; the instance is recycled.
    * `:wasm_digest_mismatch` - class `:refused_identity`, witnesses `:R_missing_identity`. The sha256 of the WASM bytes differs from the pinned digest, so the module is not the admitted GraphLaw release.
    * `:wasm_import_surface_mismatch` - class `:refused_identity`, witnesses `:R_missing_identity`. The module imports something outside the manifest's closed WASI allowlist, so it is not the pinned GraphLaw shape.
    * `:malformed_response` - class `:refused_structure`, witnesses `:mu_on_O`. The engine response was valid JSON but lacked the expected envelope fields.
    * `:invalid_json` - class `:refused_structure`, witnesses `:mu_on_O`. A request could not be encoded as JSON, or an engine response was not valid JSON.
    * `:invalid_encoding` - class `:refused_structure`, witnesses `:mu_on_O`. The request contained text that is not valid UTF-8 and was refused before reaching the engine.
    * `:resource_limit` - class `:blocked_resource`, witnesses `:R_missing_consequence`. A request or response exceeded a size cap, or the engine hit a resource limit, before it could decide.
    * `:engine_unclassified` - class `:unsupported`, witnesses `:R_missing_standing`. The engine returned a refusal variant this library does not know; it is carried verbatim and classified unsupported rather than guessed.
  """

  @codes [
    :saturated,
    :call_timeout,
    :call_trapped,
    :call_exited,
    :fuel_exhausted,
    :wasm_digest_mismatch,
    :wasm_import_surface_mismatch,
    :malformed_response,
    :invalid_json,
    :invalid_encoding,
    :resource_limit,
    :engine_unclassified
  ]

  @classes [:blocked_resource, :refused_structure, :refused_identity, :unsupported]

  @broken_terms [:mu_on_O, :R_missing_consequence, :R_missing_identity, :R_missing_standing]

  # UNSUPPORTED(generator-capability): engine error payload -> code policy. The
  # engine's error enums are non_exhaustive, so unknown variants map to
  # :engine_unclassified instead of crashing.
  @engine_codes %{
    "ResourceLimit" => :resource_limit
  }

  # Engine `RefusalKind` variants (graphlaw src/dialect.rs), carried in `details`
  # for classification by the caller; the host table itself has no engine
  # admission codes, so a known kind still maps through the table above.

  @typedoc "Closed union of every refusal code."
  @type code ::
          :saturated
          | :call_timeout
          | :call_trapped
          | :call_exited
          | :fuel_exhausted
          | :wasm_digest_mismatch
          | :wasm_import_surface_mismatch
          | :malformed_response
          | :invalid_json
          | :invalid_encoding
          | :resource_limit
          | :engine_unclassified

  @typedoc "Closed union of refusal classes."
  @type class :: :blocked_resource | :refused_structure | :refused_identity | :unsupported

  @typedoc "Chatman failure-taxonomy term a refusal witnesses."
  @type broken_term ::
          :mu_on_O | :R_missing_consequence | :R_missing_identity | :R_missing_standing

  @typedoc "A typed refusal: closed `code`, its `class` and `broken_term`, and `details`."
  @type t :: %__MODULE__{
          code: code(),
          class: class() | nil,
          message: String.t() | nil,
          details: map(),
          broken_term: broken_term() | nil,
          raw: map() | nil
        }

  defexception code: nil,
               class: nil,
               message: nil,
               details: %{},
               broken_term: nil,
               raw: nil

  @doc "Every refusal code, in table order."
  @spec codes() :: [code()]
  def codes, do: @codes

  @doc "Every refusal class."
  @spec classes() :: [class()]
  def classes, do: @classes

  @doc "Every failure-taxonomy term a refusal can witness."
  @spec broken_terms() :: [broken_term()]
  def broken_terms, do: @broken_terms

  @doc """
  The class of a code. Raises `ArgumentError` for a code outside the closed table.

      iex> GraphlawEx.Refusal.class_of(:saturated)
      :blocked_resource
  """
  @spec class_of(code()) :: class()
  def class_of(:saturated), do: :blocked_resource
  def class_of(:call_timeout), do: :blocked_resource
  def class_of(:call_trapped), do: :blocked_resource
  def class_of(:call_exited), do: :blocked_resource
  def class_of(:fuel_exhausted), do: :blocked_resource
  def class_of(:wasm_digest_mismatch), do: :refused_identity
  def class_of(:wasm_import_surface_mismatch), do: :refused_identity
  def class_of(:malformed_response), do: :refused_structure
  def class_of(:invalid_json), do: :refused_structure
  def class_of(:invalid_encoding), do: :refused_structure
  def class_of(:resource_limit), do: :blocked_resource
  def class_of(:engine_unclassified), do: :unsupported
  def class_of(other), do: raise(ArgumentError, "unknown refusal code: #{inspect(other)}")

  @doc """
  The failure-taxonomy term of a code. Raises `ArgumentError` for a code outside
  the closed table.

      iex> GraphlawEx.Refusal.broken_term_of(:wasm_digest_mismatch)
      :R_missing_identity
  """
  @spec broken_term_of(code()) :: broken_term()
  def broken_term_of(:saturated), do: :R_missing_consequence
  def broken_term_of(:call_timeout), do: :R_missing_consequence
  def broken_term_of(:call_trapped), do: :R_missing_consequence
  def broken_term_of(:call_exited), do: :R_missing_consequence
  def broken_term_of(:fuel_exhausted), do: :R_missing_consequence
  def broken_term_of(:wasm_digest_mismatch), do: :R_missing_identity
  def broken_term_of(:wasm_import_surface_mismatch), do: :R_missing_identity
  def broken_term_of(:malformed_response), do: :mu_on_O
  def broken_term_of(:invalid_json), do: :mu_on_O
  def broken_term_of(:invalid_encoding), do: :mu_on_O
  def broken_term_of(:resource_limit), do: :R_missing_consequence
  def broken_term_of(:engine_unclassified), do: :R_missing_standing
  def broken_term_of(other), do: raise(ArgumentError, "broken_term unknown for code: #{inspect(other)}")

  @doc "The one-line meaning of a code. Raises `ArgumentError` outside the closed table."
  @spec doc_of(code()) :: String.t()
  def doc_of(:saturated),
    do: "The host queue is full (max_queue), so the request was shed instead of waiting."

  def doc_of(:call_timeout), do: "The call did not finish within the configured timeout_ms."
  def doc_of(:call_trapped), do: "The WASM call trapped while executing; the instance is recycled."
  def doc_of(:call_exited), do: "The WASM program exited during the call; the instance is recycled."
  def doc_of(:fuel_exhausted), do: "The call used up its fuel budget before the engine returned."
  def doc_of(:wasm_digest_mismatch),
    do: "The sha256 of the WASM bytes differs from the pinned digest, so the module is not the admitted GraphLaw release."

  def doc_of(:wasm_import_surface_mismatch),
    do: "The module imports outside the manifest's closed WASI allowlist, so it is not the pinned GraphLaw shape."

  def doc_of(:malformed_response),
    do: "The engine response was valid JSON but lacked the expected envelope fields."

  def doc_of(:invalid_json), do: "A request could not be encoded as JSON, or an engine response was not valid JSON."

  def doc_of(:invalid_encoding),
    do: "The request contained text that is not valid UTF-8 and was refused before reaching the engine."

  def doc_of(:resource_limit),
    do: "A request or response exceeded a size cap, or the engine hit a resource limit, before it could decide."

  def doc_of(:engine_unclassified),
    do: "The engine returned a refusal variant this library does not know; it is carried verbatim and classified unsupported rather than guessed."

  def doc_of(other), do: raise(ArgumentError, "unknown refusal code: #{inspect(other)}")

  @doc """
  Builds a refusal for `code`, filling `class` and `broken_term` from the closed
  table. A nil or empty `message` defaults to the code's meaning (`doc_of/1`).
  Raises `ArgumentError` for a code outside the closed table.

      iex> refusal = GraphlawEx.Refusal.new(:saturated, "custom", %{"k" => 1})
      iex> {refusal.message, refusal.details}
      {"custom", %{"k" => 1}}
  """
  @spec new(code(), String.t() | nil, map()) :: t()
  def new(code, message \\ nil, details \\ %{}) when is_map(details) do
    %__MODULE__{
      code: code,
      class: class_of(code),
      broken_term: broken_term_of(code),
      message: if(is_binary(message) and message != "", do: message, else: doc_of(code)),
      details: details
    }
  end

  @doc "Per-code constructor; see `new/3`."
  @spec saturated(String.t() | nil, map()) :: t()
  def saturated(message \\ nil, details \\ %{}) when is_map(details), do: new(:saturated, message, details)

  @doc "Per-code constructor; see `new/3`."
  @spec call_timeout(String.t() | nil, map()) :: t()
  def call_timeout(message \\ nil, details \\ %{}) when is_map(details), do: new(:call_timeout, message, details)

  @doc "Per-code constructor; see `new/3`."
  @spec call_trapped(String.t() | nil, map()) :: t()
  def call_trapped(message \\ nil, details \\ %{}) when is_map(details), do: new(:call_trapped, message, details)

  @doc "Per-code constructor; see `new/3`."
  @spec call_exited(String.t() | nil, map()) :: t()
  def call_exited(message \\ nil, details \\ %{}) when is_map(details), do: new(:call_exited, message, details)

  @doc "Per-code constructor; see `new/3`."
  @spec fuel_exhausted(String.t() | nil, map()) :: t()
  def fuel_exhausted(message \\ nil, details \\ %{}) when is_map(details),
    do: new(:fuel_exhausted, message, details)

  @doc "Per-code constructor; see `new/3`."
  @spec wasm_digest_mismatch(String.t() | nil, map()) :: t()
  def wasm_digest_mismatch(message \\ nil, details \\ %{}) when is_map(details),
    do: new(:wasm_digest_mismatch, message, details)

  @doc "Per-code constructor; see `new/3`."
  @spec wasm_import_surface_mismatch(String.t() | nil, map()) :: t()
  def wasm_import_surface_mismatch(message \\ nil, details \\ %{}) when is_map(details),
    do: new(:wasm_import_surface_mismatch, message, details)

  @doc "Per-code constructor; see `new/3`."
  @spec malformed_response(String.t() | nil, map()) :: t()
  def malformed_response(message \\ nil, details \\ %{}) when is_map(details),
    do: new(:malformed_response, message, details)

  @doc "Per-code constructor; see `new/3`."
  @spec invalid_json(String.t() | nil, map()) :: t()
  def invalid_json(message \\ nil, details \\ %{}) when is_map(details), do: new(:invalid_json, message, details)

  @doc "Per-code constructor; see `new/3`."
  @spec invalid_encoding(String.t() | nil, map()) :: t()
  def invalid_encoding(message \\ nil, details \\ %{}) when is_map(details),
    do: new(:invalid_encoding, message, details)

  @doc "Per-code constructor; see `new/3`."
  @spec resource_limit(String.t() | nil, map()) :: t()
  def resource_limit(message \\ nil, details \\ %{}) when is_map(details),
    do: new(:resource_limit, message, details)

  @doc "Per-code constructor; see `new/3`."
  @spec engine_unclassified(String.t() | nil, map()) :: t()
  def engine_unclassified(message \\ nil, details \\ %{}) when is_map(details),
    do: new(:engine_unclassified, message, details)

  @doc """
  Maps a decoded engine error payload onto a typed refusal, preserving the whole
  payload losslessly in `raw`. Never raises: an unknown or non-map payload
  becomes `:engine_unclassified` with the value carried in `details`.

  The engine reports a refusal as `%{"ok" => false, "error" => %{...}}`; the
  `error` map is what lands here. Known engine codes (`"ResourceLimit"`) map to
  their table code; every other code or kind maps to `:engine_unclassified`, so
  a future engine release is forward-compatible.
  """
  @spec from_engine(term(), map()) :: t()
  def from_engine(error, details \\ %{})

  def from_engine(error, details) when is_map(error) and is_map(details) do
    kind = Map.get(error, "kind")
    engine_code = Map.get(error, "code")
    code = engine_code_to_code(engine_code, kind)

    refusal = new(code, string_or_nil(Map.get(error, "message")), Map.put(details, "kind", kind))

    %{refusal | raw: error}
  end

  def from_engine(other, details) do
    details = if is_map(details), do: details, else: %{}

    new(:engine_unclassified, nil, Map.put(details, "raw", inspect(other, limit: 20, printable_limit: 256)))
    |> then(&%{&1 | raw: if(is_map(other), do: other)})
  end

  defp engine_code_to_code(code, _kind) when is_map_key(@engine_codes, code), do: Map.get(@engine_codes, code)
  defp engine_code_to_code(_code, _kind), do: :engine_unclassified

  defp string_or_nil(value) when is_binary(value), do: value
  defp string_or_nil(_), do: nil

  @doc false
  def message(%__MODULE__{message: message}) when is_binary(message), do: message
  def message(%__MODULE__{code: code}) when is_atom(code) and not is_nil(code), do: "GraphLaw refused: #{code}"
  def message(_refusal), do: "GraphLaw refused"
end
