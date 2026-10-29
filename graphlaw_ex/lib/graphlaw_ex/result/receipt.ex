# SPDX-License-Identifier: Apache-2.0
# UNSUPPORTED(generator-capability): hand-written receipt struct backing
# `GraphlawEx.Result.Admitted` (the `law` capability result). Forward-compatible:
# every known receipt field is read when present, and `:raw` keeps the whole
# wire map losslessly.

defmodule GraphlawEx.Result.Receipt do
  @moduledoc ~S"""
  One transition receipt of a `law` result (`GraphlawEx.Result.Admitted`).

  The fields mirror the kernel wire producers exactly (src/abi/ops/governance.rs):
  plan receipts (:432-441, step/parent/child/added/authority/revision/
  plan_blake3/lease_id, with `index` for plan steps) and transition receipts
  (:501-508, same core fields plus `subject_blake3`).

  Note: attestations are recorded as RDF triples inside the state
  (src/receipt.rs:104-108), never as JSON on the wire; `subject_blake3` is a
  reserved payload field (src/attest.rs:506-526) not yet emitted on the wire.
  `:raw` preserves the wire map losslessly, so `from_map(m).raw == m` for
  every `m`. Never raises.
  """

  defstruct [:step, :parent, :child, :added, :authority, :revision, :plan_blake3, :lease_id, :index, :subject_blake3, :raw]

  @type t :: %__MODULE__{
          step: term(),
          parent: term(),
          child: term(),
          added: integer() | nil,
          authority: String.t() | nil,
          revision: String.t() | nil,
          plan_blake3: String.t() | nil,
          lease_id: String.t() | nil,
          # plan receipts only
          index: non_neg_integer() | nil,
          # reserved payload field (src/attest.rs:506-526), not yet emitted on the wire
          subject_blake3: String.t() | nil,
          raw: map()
        }

  @doc "Builds a receipt from its wire map. Never raises; `:raw` carries the input unchanged."
  @spec from_map(term()) :: t()
  def from_map(map) when is_map(map) do
    %__MODULE__{
      step: Map.get(map, "step"),
      parent: Map.get(map, "parent"),
      child: Map.get(map, "child"),
      added: Map.get(map, "added"),
      authority: Map.get(map, "authority"),
      revision: Map.get(map, "revision"),
      plan_blake3: Map.get(map, "plan_blake3"),
      lease_id: Map.get(map, "lease_id"),
      index: Map.get(map, "index"),
      # reserved payload field (src/attest.rs:506-526), not yet emitted on the wire
      subject_blake3: Map.get(map, "subject_blake3"),
      raw: map
    }
  end

  def from_map(other), do: %__MODULE__{raw: other}
end
