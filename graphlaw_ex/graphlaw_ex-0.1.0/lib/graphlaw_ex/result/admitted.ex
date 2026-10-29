# SPDX-License-Identifier: Apache-2.0
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
  defstruct [:states, :receipts, :nquads, :raw]

  @type t :: %__MODULE__{
          states: list(),
          receipts: [Receipt.t()],
          nquads: String.t(),
          raw: map()
        }

  @doc """
  Builds an admitted transition from an engine response map. Missing fields
  default to `[]`, `[]`, `""`; receipts decode through
  `GraphlawEx.Result.Receipt.from_map/1`.
  """
  @spec from_map(map()) :: t()
  def from_map(map) when is_map(map) do
    %__MODULE__{
      states: map["states"] || [],
      receipts: Enum.map(map["receipts"] || [], &Receipt.from_map/1),
      nquads: map["nquads"] || "",
      raw: map
    }
  end

  def from_map(other), do: %__MODULE__{states: [], receipts: [], nquads: "", raw: other}
end
