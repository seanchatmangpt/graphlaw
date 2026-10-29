# SPDX-License-Identifier: Apache-2.0
# CONSOLIDATED (GL28-11): the closed refusal table lives in ONE module,
# `GraphlawEx.Refusal` (host + capability + engine-side codes, each with its
# class and Chatman `broken_term`). This module is the capability layer's
# re-export: a thin typed struct over that one table — every code, class,
# broken term and engine wire mapping is read from `GraphlawEx.Refusal`, never
# duplicated here. `GraphlawEx.Error.Refused` wraps this struct.

defmodule GraphlawEx.Capability.Refusal do
  @moduledoc ~S"""
  Typed refusal carried by every `{:error, _}` of the capability layer.

  The closed code table is `GraphlawEx.Refusal` — one table for the host
  transport codes (`:saturated`, `:call_trapped`, ...), the capability client
  codes and the engine-side codes. This module re-exports that table for the
  capability layer and carries the layer's own struct (no `class` field, since
  the capability layer never classifies transport failures itself).

  Client-side codes (built with `build/3`, see
  `GraphlawEx.Refusal.capability_client_codes/0`):

    * `:invalid_capability_request` - A typed capability request failed
      validation before the engine was called.
    * `:unknown_capability` - The named capability is not in the GraphLaw
      capability registry.
    * `:capability_response_undecodable` - The engine returned a capability
      response that is not a JSON object; the raw value is preserved on the
      refusal.
    * `:ceiling_unmet` - The authority pre-gate (`GraphlawEx.Authority`) refused
      the call before engine dispatch: the presented lease claims a ceiling
      below the op's floor. Class `:refused_structure`, broken term
      `:R_missing_standing` (GL28-7).

  Engine-side codes (built with `from_engine/1`): `:resource_limit`,
  `:policy_refused`, `:not_admitted`, `:plan_refused`, `:receipt_required`,
  `:engine_refused`, and `:engine_unclassified` for a variant this library
  does not know (carried verbatim in `:raw`, never guessed). The wire-code
  mapping is the one shared table in `GraphlawEx.Refusal`.

  Transport-side codes (built with `from_transport/1`): every
  `GraphlawEx.Refusal` code (`:saturated`, `:call_trapped`, ...), converted
  losslessly.
  """

  defstruct [:code, :message, :details, :raw, :broken_term]

  alias GraphlawEx.Refusal

  @type code ::
          Refusal.code()
          | atom()

  @typedoc "A typed refusal: `code`, `message`, `details`, lossless `raw`."
  @type t :: %__MODULE__{
          code: code(),
          message: String.t() | nil,
          details: map(),
          raw: map() | nil,
          broken_term: atom()
        }

  @doc """
  Every client-side refusal code this layer can build before the engine is
  called. `:ceiling_unmet` is the GL28-7 authority pre-gate code (added
  2026-10-28); the rest are the layer's pre-existing client codes.
  """
  @spec client_codes() :: [atom()]
  def client_codes,
    do: [
      :ceiling_unmet,
      :invalid_capability_request,
      :unknown_capability,
      :capability_response_undecodable
    ]

  @doc """
  Builds a client-side refusal. `message` defaults to a rendering of `code`;
  `details` defaults to `%{}`. The `broken_term` comes from the one table when
  the code is in it; never raises on data.
  """
  @spec build(code(), String.t() | nil, map()) :: t()
  def build(code, message \\ nil, details \\ %{}) do
    %__MODULE__{
      code: code,
      message: if(is_binary(message) and message != "", do: message, else: render(code)),
      details: if(is_map(details), do: details, else: %{}),
      broken_term: broken_term_of(code)
    }
  end

  @doc """
  Builds the `:ceiling_unmet` refusal of the authority pre-gate (GL28-7):
  class `:refused_structure`, broken term `:R_missing_standing`, wire-safe
  string-keyed details.
  """
  @spec ceiling_unmet(String.t(), map()) :: t()
  def ceiling_unmet(message, details) when is_binary(message) and is_map(details) do
    %__MODULE__{
      code: :ceiling_unmet,
      message: message,
      details: details,
      broken_term: :R_missing_standing
    }
  end

  @doc """
  Maps an engine refusal envelope (`%{"ok" => false, "error" => %{...}}`) onto
  a typed refusal, preserving the whole error map losslessly in `raw`. Never
  raises: an unknown or non-map payload becomes `:engine_unclassified`. The
  wire-code mapping is the host table's own `GraphlawEx.Refusal.from_engine/2`.
  """
  @spec from_engine(term()) :: t()
  def from_engine(%{"ok" => false, "error" => %{} = error} = _response) do
    %__MODULE__{
      code: Refusal.from_engine(%{"code" => Map.get(error, "code"), "kind" => Map.get(error, "kind")}).code,
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
  @spec from_transport(Refusal.t()) :: t()
  def from_transport(%Refusal{} = refusal) do
    %__MODULE__{
      code: refusal.code,
      message: refusal.message,
      details: refusal.details,
      raw: refusal.raw,
      broken_term: refusal.broken_term
    }
  end

  # The table lookup, total on data: a code in the one table uses its
  # broken_term; anything else stays :mu_on_O exactly as before consolidation.
  # `:ceiling_unmet` is the one client-side code this layer owns outright
  # (GL28-7 authority pre-gate): class `:refused_structure`, broken term
  # `:R_missing_standing` — the host table stays closed.
  defp broken_term_of(:ceiling_unmet), do: :R_missing_standing

  defp broken_term_of(code) do
    if code in Refusal.codes(), do: Refusal.broken_term_of(code), else: :mu_on_O
  rescue
    ArgumentError -> :mu_on_O
  end

  defp string_or_nil(value) when is_binary(value), do: value
  defp string_or_nil(_), do: nil

  defp render(code), do: "capability request refused: #{code}"
end
