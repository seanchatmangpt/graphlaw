# SPDX-License-Identifier: MIT
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
  `:raw` preserves the wire map losslessly, so `from_map(m).raw == m` for every `m`.
  `from_map/1` never raises; `from_map!/1` raises `ArgumentError` for a map missing
  `"step"`, `"parent"` or `"child"` (use `parse/1` for a structured
  `{:error, {:missing, keys}}`).
  `Inspect` omits `:raw`.
  """

  @required ["step", "parent", "child"]

  @derive {Inspect, except: [:raw]}
  defstruct [
    :step,
    :parent,
    :child,
    :added,
    :authority,
    :revision,
    :plan_blake3,
    :lease_id,
    :index,
    :subject_blake3,
    :raw
  ]

  @type t :: %__MODULE__{
          step: term(),
          parent: term(),
          child: term(),
          added: term(),
          authority: term(),
          revision: term(),
          plan_blake3: term(),
          lease_id: term(),
          # plan receipts only
          index: term(),
          # reserved payload field (src/attest.rs:506-526), not yet emitted on the wire
          subject_blake3: term(),
          raw: term()
        }

  @doc """
  Strict decode: `{:ok, receipt}` when the wire map carries `"step"`, `"parent"`
  and `"child"`, else `{:error, {:missing, keys}}` (or `{:error, :not_a_map}`).
  """
  @spec parse(term()) :: {:ok, t()} | {:error, {:missing, [String.t()]} | :not_a_map}
  def parse(map) when is_map(map) do
    case Enum.reject(@required, &Map.has_key?(map, &1)) do
      [] -> {:ok, build(map)}
      missing -> {:error, {:missing, missing}}
    end
  end

  def parse(_other), do: {:error, :not_a_map}

  @doc """
  Lossless build from the wire map; never raises, consistent with the other results.
  `:raw` carries the input unchanged, and a non-map input is kept in `:raw`.
  """
  @spec from_map(term()) :: t()
  def from_map(input), do: lossless(input)

  @doc false
  # Compatibility for callers passing `lossless: true` (now the only behavior of from_map).
  @spec from_map(term(), keyword()) :: t()
  def from_map(input, _opts), do: lossless(input)

  @doc """
  Strict build: raises `ArgumentError` when `"step"`, `"parent"` or `"child"` is
  missing or the input is not a map (the lossy cases of `from_map/1`).
  """
  @spec from_map!(term()) :: t()
  def from_map!(input) do
    case parse(input) do
      {:ok, receipt} ->
        receipt

      {:error, {:missing, keys}} ->
        raise ArgumentError, "receipt wire map missing required keys: #{inspect(keys)}"

      {:error, :not_a_map} ->
        raise ArgumentError, "receipt wire value is not a map: #{inspect(input)}"
    end
  end

  defp lossless(map) when is_map(map), do: build(map)
  defp lossless(other), do: %__MODULE__{raw: other}

  defp build(map) do
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
end
