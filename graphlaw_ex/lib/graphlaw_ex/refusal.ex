# SPDX-FileCopyrightText: 2026 Sean Chatman <https://github.com/seanchatmangpt/graphlaw/graphs/contributors>
#
# SPDX-License-Identifier: MIT
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

  ### Capability-layer codes (the ONE table: host + capability + engine side)

    * `:invalid_capability_request` - class `:refused_structure`, witnesses `:mu_on_O`. A typed capability request failed validation before the engine was called.
    * `:unknown_capability` - class `:refused_structure`, witnesses `:mu_on_O`. The named capability is not in the GraphLaw capability registry.
    * `:capability_response_undecodable` - class `:refused_structure`, witnesses `:mu_on_O`. The engine returned a capability response that is not a JSON object.
    * `:policy_refused` - class `:refused_structure`, witnesses `:mu_on_O`. The engine refused the FOND policy as not strong-cyclic.
    * `:not_admitted` - class `:refused_structure`, witnesses `:mu_on_O`. The engine did not admit the law chain.
    * `:plan_refused` - class `:refused_structure`, witnesses `:mu_on_O`. The engine refused the plan step.
    * `:receipt_required` - class `:refused_identity`, witnesses `:R_missing_identity`. The engine required a receipt the caller did not supply.
    * `:engine_refused` - class `:unsupported`, witnesses `:R_missing_standing`. The engine refused without a more specific known code.
    * `:lease_revoked` - class `:refused_identity`, witnesses `:R_missing_standing`. An Art.14 operator revocation epoch covers this engine (`GraphlawEx.KillSwitch`); the call was shed pre-envelope, before any engine mutation.

  The data behind this table lives in `GraphlawEx.Refusal.Table`; this module
  is the logic (constructors, hint-free lookups, engine payload lifting).
  """

  alias GraphlawEx.Refusal.Table

  # The engine's error enums are non_exhaustive, so unknown wire codes map to
  # :engine_unclassified instead of crashing.
  @codes Table.codes()
  @engine_codes Table.engine_codes()
  @capability_client_codes Table.capability_client_codes()

  @typedoc "Closed union of every refusal code (host + capability layers)."
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
          | :invalid_capability_request
          | :unknown_capability
          | :capability_response_undecodable
          | :policy_refused
          | :not_admitted
          | :plan_refused
          | :receipt_required
          | :engine_refused
          | :lease_revoked
          | :receipt_refused
          | :lease_refused
          | :unverified_lease_refused
          | :fuse
          | :duckdb_failure
          | :batch_member_failed

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

  # `:raw` (whole engine payload) and `:details` can carry request content;
  # neither is printed by `inspect/2`.
  @derive {Inspect, except: [:raw, :details]}
  defexception code: nil,
               class: nil,
               message: nil,
               details: %{},
               broken_term: nil,
               raw: nil

  @doc "Every refusal code, in table order."
  @spec codes() :: [code()]
  def codes, do: @codes

  @doc "The capability-layer client codes (refused before the engine is called)."
  @spec capability_client_codes() :: [code()]
  def capability_client_codes, do: @capability_client_codes

  @doc """
  Maps an engine wire code (the `error.code` string of a refusal envelope) onto
  the one closed table. Unknown or non-binary codes map to `:engine_unclassified`.
  """
  @spec from_engine_code(term()) :: code()
  def from_engine_code(code) when is_map_key(@engine_codes, code),
    do: Map.fetch!(@engine_codes, code)

  def from_engine_code(_code), do: :engine_unclassified

  @doc "Every refusal class."
  @spec classes() :: [class()]
  def classes, do: Table.classes()

  @doc "Every failure-taxonomy term a refusal can witness."
  @spec broken_terms() :: [broken_term()]
  def broken_terms, do: Table.broken_terms()

  @doc """
  The class of a code. Total on the closed table and the capability client
  codes (`:ceiling_unmet`); raises `ArgumentError` for any other code.

      iex> GraphlawEx.Refusal.class_of(:saturated)
      :blocked_resource
  """
  @spec class_of(code() | :ceiling_unmet) :: class()
  def class_of(code), do: Table.class(code) || unknown_code!("class_of/1", code)

  @doc """
  The failure-taxonomy term of a code. Raises `ArgumentError` for a code outside
  the closed table.

      iex> GraphlawEx.Refusal.broken_term_of(:wasm_digest_mismatch)
      :R_missing_identity
  """
  @spec broken_term_of(code() | :ceiling_unmet) :: broken_term()
  def broken_term_of(code), do: Table.broken_term(code) || unknown_code!("broken_term_of/1", code)

  @doc "The one-line meaning of a code. Raises `ArgumentError` outside the closed table."
  @spec doc_of(code() | :ceiling_unmet) :: String.t()
  def doc_of(code), do: Table.doc(code) || unknown_code!("doc_of/1", code)

  # Raises with the nearest known code (jaro >= 0.7) and the full `codes/0` list.
  @spec unknown_code!(String.t(), term()) :: no_return()
  defp unknown_code!(fun, code) do
    known = Enum.uniq(@codes ++ [:ceiling_unmet])

    hint =
      case nearest(code, known) do
        nil -> ""
        near -> " Did you mean #{inspect(near)}?"
      end

    raise ArgumentError,
          "#{fun}: unknown refusal code #{inspect(code, limit: 5, printable_limit: 64)}." <>
            hint <> " Known codes: #{inspect(known)}"
  end

  defp nearest(code, known) when is_atom(code) or is_binary(code) do
    text = to_string(code)

    {score, best} =
      known |> Enum.map(&{String.jaro_distance(text, Atom.to_string(&1)), &1}) |> Enum.max()

    if score >= 0.7, do: best
  end

  defp nearest(_code, _known), do: nil

  @doc """
  Builds a refusal for `code`, filling `class` and `broken_term` from the closed
  table. A nil or empty `message` defaults to the code's meaning (`doc_of/1`).
  Raises `ArgumentError` for a code outside the closed table.

      iex> refusal = GraphlawEx.Refusal.new(:saturated, "custom", %{"k" => 1})
      iex> {refusal.message, refusal.details}
      {"custom", %{"k" => 1}}
  """
  @spec new(code() | :ceiling_unmet, String.t() | nil, map()) :: t()
  def new(code, message \\ nil, details \\ %{}) when is_map(details) do
    %__MODULE__{
      code: code,
      class: class_of(code),
      broken_term: broken_term_of(code),
      message: if(is_binary(message) and message != "", do: message, else: doc_of(code)),
      details: details
    }
  end

  for code <- Table.constructor_codes() do
    @doc "Per-code constructor; see `new/3`."
    @spec unquote(code)(String.t() | nil, map()) :: t()
    def unquote(code)(message \\ nil, details \\ %{}) when is_map(details),
      do: new(unquote(code), message, details)
  end

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
    # The wire envelope carries the machine code at `error.details.code` (the
    # kernel's `Fail.details`); a top-level `code` is also accepted.
    wire_details = if is_map(error["details"]), do: error["details"], else: %{}
    code = from_engine_code(Map.get(error, "code") || Map.get(wire_details, "code"))

    # Per-code payload lifting: the lift keys (mirroring the kernel's json!
    # keys) come from the table, so the fields the kernel emits for this code
    # surface in `details` without the caller re-reading `raw`. Unknown fields
    # stay lossless in `raw`.
    lifted = Map.take(Map.merge(wire_details, error), Table.lift_keys(code))

    refusal =
      new(
        code,
        string_or_nil(Map.get(error, "message")),
        Map.put(details, "kind", kind) |> Map.merge(lifted)
      )

    %{refusal | raw: error}
  end

  def from_engine(other, details) do
    details = if is_map(details), do: details, else: %{}

    # Never carry payload bytes: only a type tag, size and digest prefix.
    new(:engine_unclassified, nil, Map.put(details, "raw", digest_tag(other)))
    |> then(&%{&1 | raw: if(is_map(other), do: other)})
  end

  defp digest_tag(term) do
    bin = :erlang.term_to_binary(term)
    <<prefix::binary-size(8), _::binary>> = :crypto.hash(:sha256, bin)

    %{
      "type" => type_tag(term),
      "bytes" => byte_size(bin),
      "sha256_prefix" => Base.encode16(prefix, case: :lower)
    }
  end

  defp type_tag(v) when is_map(v), do: "map"
  defp type_tag(v) when is_binary(v), do: "binary"
  defp type_tag(v) when is_list(v), do: "list"
  defp type_tag(v) when is_tuple(v), do: "tuple"
  defp type_tag(v) when is_atom(v), do: "atom"
  defp type_tag(v) when is_number(v), do: "number"
  defp type_tag(_), do: "term"

  defp string_or_nil(value) when is_binary(value), do: value
  defp string_or_nil(_), do: nil

  @doc false
  @impl true
  def message(%__MODULE__{message: message}) when is_binary(message), do: message

  def message(%__MODULE__{code: code}) when is_atom(code) and not is_nil(code),
    do: "GraphLaw refused: #{code}"

  def message(_refusal), do: "GraphLaw refused"
end
