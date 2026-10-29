# SPDX-License-Identifier: Apache-2.0
# UNSUPPORTED(generator-capability): open refusal struct for the typed
# capability layer. `GraphlawEx.Refusal` (the host transport refusal) is a
# defexception over a CLOSED code table that does not contain the
# capability-layer codes (`:invalid_capability_request`,
# `:unknown_capability`, `:capability_response_undecodable`), so this layer
# carries its own typed refusal and `GraphlawEx.Error.Refused` wraps it.

defmodule GraphlawEx.Capability.Refusal do
  @moduledoc ~S"""
  Typed refusal carried by every `{:error, _}` of the capability layer.

  A refusal is an observation about one exact input: it names a code, the
  Chatman failure-taxonomy `broken_term` it witnesses and a closed-ish class.
  `:raw` keeps the engine error map losslessly (`nil` for client-side
  refusals built with `build/3`). A refusal never grants or withdraws
  authority.

  Client-side codes (built with `build/3`):

    * `:invalid_capability_request` - class `:refused_structure`,
      witnesses `:mu_on_O`. A typed capability request failed validation
      before the engine was called: unknown top-level keys, missing required
      fields, or a value whose type does not match the registry field type.
    * `:unknown_capability` - class `:refused_structure`, witnesses
      `:mu_on_O`. The named capability is not in the GraphLaw capability
      registry, so no typed request can be built for it.
    * `:capability_response_undecodable` - class `:refused_structure`,
      witnesses `:mu_on_O`. The engine returned a capability response that
      is not a JSON object, so it cannot be decoded into a typed result; the
      raw value is preserved on the refusal.

  Engine-side codes (built with `from_engine/1`): `:resource_limit`,
  `:policy_refused`, `:not_admitted`, `:plan_refused`, `:receipt_required`,
  `:engine_refused`, and `:engine_unclassified` for a variant this library
  does not know (carried verbatim in `:raw`, never guessed).

  Transport-side codes (built with `from_transport/1`): every
  `GraphlawEx.Refusal` code (`:saturated`, `:call_trapped`, ...), converted
  losslessly.
  """

  defstruct [:code, :message, :details, :raw, :broken_term]

  @type code ::
          :invalid_capability_request
          | :unknown_capability
          | :capability_response_undecodable
          | :resource_limit
          | :policy_refused
          | :not_admitted
          | :plan_refused
          | :receipt_required
          | :engine_refused
          | :engine_unclassified
          | GraphlawEx.Refusal.code()
          | atom()

  @type class :: :refused_structure | :engine | :transport

  @typedoc "A typed refusal: `code`, `message`, `details`, lossless `raw`."
  @type t :: %__MODULE__{
          code: code(),
          message: String.t() | nil,
          details: map(),
          raw: map() | nil,
          broken_term: atom()
        }

  @client_codes ~w(invalid_capability_request unknown_capability capability_response_undecodable)a

  @doc "Every client-side refusal code this layer can build before the engine is called."
  @spec client_codes() :: [code()]
  def client_codes, do: @client_codes

  @doc """
  Builds a client-side refusal. `message` defaults to a rendering of `code`;
  `details` defaults to `%{}`. Never raises on data.
  """
  @spec build(code(), String.t() | nil, map()) :: t()
  def build(code, message \\ nil, details \\ %{}) do
    %__MODULE__{
      code: code,
      message: if(is_binary(message) and message != "", do: message, else: render(code)),
      details: if(is_map(details), do: details, else: %{}),
      broken_term: :mu_on_O
    }
  end

  @doc """
  Maps an engine refusal envelope (`%{"ok" => false, "error" => %{...}}`) onto
  a typed refusal, preserving the whole error map losslessly in `raw`. Never
  raises: an unknown or non-map payload becomes `:engine_unclassified`.
  """
  @spec from_engine(term()) :: t()
  def from_engine(%{"ok" => false, "error" => %{} = error} = _response) do
    code = engine_code(Map.get(error, "code"), Map.get(error, "kind"))

    %__MODULE__{
      code: code,
      message: string_or_nil(Map.get(error, "message")),
      details: %{"kind" => Map.get(error, "kind")},
      raw: error,
      broken_term: :mu_on_O
    }
  end

  def from_engine(other),
    do: %__MODULE__{
      code: :engine_unclassified,
      message: "engine response was not a refusal envelope",
      details: %{},
      raw: if(is_map(other), do: other),
      broken_term: :R_missing_standing
    }

  @doc "Converts a host transport refusal (`GraphlawEx.Refusal`) losslessly."
  @spec from_transport(GraphlawEx.Refusal.t()) :: t()
  def from_transport(%GraphlawEx.Refusal{} = refusal) do
    %__MODULE__{
      code: refusal.code,
      message: refusal.message,
      details: refusal.details,
      raw: refusal.raw,
      broken_term: refusal.broken_term
    }
  end

  # Engine detail codes (`details.code` wire values) -> typed codes. Unknown
  # variants map to :engine_unclassified (forward compatibility, never a
  # crash, never a guess).
  defp engine_code("ResourceLimit", _), do: :resource_limit
  defp engine_code("PolicyRefused", _), do: :policy_refused
  defp engine_code("NotAdmitted", _), do: :not_admitted
  defp engine_code("PlanRefused", _), do: :plan_refused
  defp engine_code("ReceiptRequired", _), do: :receipt_required
  defp engine_code("ReceiptRefused", _), do: :engine_refused
  defp engine_code("LeaseRefused", _), do: :engine_refused
  defp engine_code("UnverifiedLeaseRefused", _), do: :engine_refused
  defp engine_code("Refused", _), do: :engine_refused
  defp engine_code(_, _), do: :engine_unclassified

  defp string_or_nil(value) when is_binary(value), do: value
  defp string_or_nil(_), do: nil

  defp render(code), do: "capability request refused: #{code}"
end
