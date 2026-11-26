# SPDX-FileCopyrightText: 2026 Sean Chatman <https://github.com/seanchatmangpt/graphlaw/graphs/contributors>
#
# SPDX-License-Identifier: MIT
defmodule GraphlawEx.Capability.Graph do
  @moduledoc ~S"""
  RDFC-1.0 canonicalization, BLAKE3 digest and N-Quads set algebra (FR-3).

  Engine surface check: the WASM engine exposes `canonical` (RDFC-1.0 canonical N-Quads plus a
  `blake3:` content id and quad count) but no diff/union/intersection/equivalence op. This module
  therefore delegates all canonicalization and hashing to `GraphlawEx.Capability.Canonical` and
  implements the set algebra over the canonical N-Quads *lines* (one quad per line), then
  re-canonicalizes every result through the engine so outputs are themselves canonical.

  No Elixir RDF structs: graphs are text (`String.t()` or a `data_spec` map `%{"text" => ...}`).

  ## Blank-node caveat

  RDFC-1.0 labels blank nodes per input dataset. `equivalent?/3` is exact (identical canonical
  form iff isomorphic). `union/3` is a true RDF merge: each input is canonicalized, its blank
  nodes are renamed apart (`_:a_...` / `_:b_...`, quoted literals untouched), then the result is
  re-canonicalized, so distinct blank nodes never collapse (a bnode graph unioned with itself
  therefore yields two disjoint copies, per RDF merge semantics). `diff/3` and `intersection/3`
  are unsound over blank nodes (labels are per-input, not comparable), so they refuse with
  `:invalid_capability_request` (`details["reason"] == "blank_nodes_in_set_algebra"`) when either
  input contains a blank node; ground graphs are computed exactly.
  """

  alias GraphlawEx.Capability.Canonical
  alias GraphlawEx.Capability.Refusal

  @type graph :: String.t() | map()
  @type digest :: String.t()

  @doc "Canonical N-Quads text of `graph` (engine RDFC-1.0)."
  @spec canonicalize(graph(), keyword()) :: {:ok, String.t()} | {:error, Refusal.t()}
  def canonicalize(graph, opts \\ []) do
    with {:ok, r} <- Canonical.run([data: graph], opts), do: {:ok, r.nquads}
  end

  @doc "BLAKE3 content id (`blake3:<hex>`) minted by the engine over the canonical N-Quads."
  @spec digest(graph(), keyword()) :: {:ok, digest()} | {:error, Refusal.t()}
  def digest(graph, opts \\ []) do
    with {:ok, r} <- Canonical.run([data: graph], opts), do: {:ok, r.id}
  end

  @doc "True iff both graphs have the same canonical digest (graph isomorphism)."
  @spec equivalent?(graph(), graph(), keyword()) :: {:ok, boolean()} | {:error, Refusal.t()}
  def equivalent?(a, b, opts \\ []) do
    with {:ok, da} <- digest(a, opts),
         {:ok, db} <- digest(b, opts) do
      {:ok, da == db}
    end
  end

  @doc "Canonical N-Quads of the union of `a` and `b` (sources concatenated, then canonicalized)."
  @spec union(graph(), graph(), keyword()) :: {:ok, String.t()} | {:error, Refusal.t()}
  def union(a, b, opts \\ []) do
    with {:ok, ta} <- canonicalize(a, opts),
         {:ok, tb} <- canonicalize(b, opts) do
      canonicalize(rename_bnodes(ta, "a_") <> rename_bnodes(tb, "b_"), opts)
    end
  end

  @doc "Canonical N-Quads of the quads in `a` that are not in `b`."
  @spec diff(graph(), graph(), keyword()) :: {:ok, String.t()} | {:error, Refusal.t()}
  def diff(a, b, opts \\ []), do: set_op(a, b, opts, &MapSet.difference/2)

  @doc "Canonical N-Quads of the quads present in both `a` and `b`."
  @spec intersection(graph(), graph(), keyword()) :: {:ok, String.t()} | {:error, Refusal.t()}
  def intersection(a, b, opts \\ []), do: set_op(a, b, opts, &MapSet.intersection/2)

  defp set_op(a, b, opts, fun) do
    with {:ok, ta} <- canonicalize(a, opts),
         {:ok, tb} <- canonicalize(b, opts),
         :ok <- ensure_ground(ta, tb) do
      lines = fun.(lines(ta), lines(tb)) |> MapSet.to_list() |> Enum.sort()

      case lines do
        [] -> {:ok, ""}
        _ -> canonicalize(Enum.join(lines, "\n") <> "\n", opts)
      end
    end
  end

  # Matches a quoted literal (kept verbatim) or a canonical blank-node label.
  @token ~r/"(?:[^"\\]|\\.)*"|_:[A-Za-z0-9_.\-]+/

  defp rename_bnodes(text, prefix) do
    Regex.replace(@token, text, fn
      "_:" <> label -> "_:" <> prefix <> label
      literal -> literal
    end)
  end

  defp has_bnode?(text) do
    @token |> Regex.scan(text) |> Enum.any?(fn [t] -> String.starts_with?(t, "_:") end)
  end

  defp ensure_ground(ta, tb) do
    if has_bnode?(ta) or has_bnode?(tb) do
      {:error,
       Refusal.build(
         :invalid_capability_request,
         "diff/intersection are unsound over blank nodes (labels are per-input); refused",
         %{"reason" => "blank_nodes_in_set_algebra"}
       )}
    else
      :ok
    end
  end

  defp lines(text) do
    text |> String.split("\n", trim: true) |> MapSet.new()
  end
end
