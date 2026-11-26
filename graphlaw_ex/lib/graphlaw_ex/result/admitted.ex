# SPDX-License-Identifier: MIT
# UNSUPPORTED(generator-capability): hand-written result struct for the `law`
# capability; the projection has no template for decoded result structs.
# Port of `AshGraphLaw.Admitted`: the decoded result of a successful `law`
# transition. Decodes to `GraphlawEx.Result.Admitted` (state ids, per-step
# receipts, final N-Quads) — NOT a `GraphlawEx.Result.Law`.

defmodule GraphlawEx.Result.Admitted do
  @moduledoc """
  A successful GraphLaw `law` transition, decoded from the engine response.

  Carries the state ids, the per-step receipts (`GraphlawEx.Result.Receipt`)
  and the resulting N-Quads. It is an observation of admission on one exact
  input; it does not execute any consequence and standing is never stored
  here.

  `:raw` is the entire decoded response map, so `from_map(m).raw == m` for
  every `m`. `from_map/1` never raises: missing fields default to `[]`,
  `[]`, `""`.
  """

  alias GraphlawEx.Result.Receipt

  @enforce_keys [:states, :receipts, :nquads]
  @derive {Inspect, except: [:raw]}
  defstruct [:states, :receipts, :nquads, :raw]

  @type t :: %__MODULE__{
          states: list(),
          receipts: [Receipt.t()],
          nquads: String.t(),
          raw: term()
        }

  @doc """
  Builds an admitted transition from an engine response map. Missing fields
  default to `[]`, `[]`, `""`; receipts decode through the lossless opt-in
  `GraphlawEx.Result.Receipt.from_map/2` (`lossless: true`), so this function
  never raises.
  """
  @spec from_map(term()) :: t()
  def from_map(map) when is_map(map) do
    %__MODULE__{
      states: map["states"] || [],
      receipts: decode_receipts(map["receipts"]),
      nquads: map["nquads"] || "",
      raw: map
    }
  end

  def from_map(other), do: %__MODULE__{states: [], receipts: [], nquads: "", raw: other}

  # A non-list `receipts` value (engine drift) decodes to `[]`; the original stays in `:raw`.
  defp decode_receipts(list) when is_list(list),
    do: Enum.map(list, &Receipt.from_map(&1, lossless: true))

  defp decode_receipts(_other), do: []
end
