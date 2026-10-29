# SPDX-License-Identifier: Apache-2.0
# UNSUPPORTED(generator-capability): hand-written receipt struct backing
# `GraphlawEx.Result.Admitted` (the `law` capability result). Forward-compatible:
# every known receipt field is read when present, and `:raw` keeps the whole
# wire map losslessly.

defmodule GraphlawEx.Result.Receipt do
  @moduledoc ~S"""
  One transition receipt of a `law` result (`GraphlawEx.Result.Admitted`).

  Forward-compatible: `from_map/1` reads the fields the engine reports
  (`step`, `index`, `state`, `digest`, `attestation`) when present and keeps
  the whole wire map in `:raw`, so `from_map(m).raw == m` for every `m`.
  Never raises.
  """

  defstruct [:step, :index, :state, :digest, :attestation, :raw]

  @type t :: %__MODULE__{
          step: term(),
          index: term(),
          state: term(),
          digest: term(),
          attestation: term(),
          raw: map()
        }

  @doc "Builds a receipt from its wire map. Never raises; `:raw` carries the input unchanged."
  @spec from_map(term()) :: t()
  def from_map(map) when is_map(map) do
    %__MODULE__{
      step: Map.get(map, "step"),
      index: Map.get(map, "index"),
      state: Map.get(map, "state"),
      digest: Map.get(map, "digest") || Map.get(map, "blake3") || Map.get(map, "sha256"),
      attestation: Map.get(map, "attestation"),
      raw: map
    }
  end

  def from_map(other), do: %__MODULE__{raw: other}
end
