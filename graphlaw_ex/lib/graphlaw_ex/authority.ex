# SPDX-License-Identifier: Apache-2.0
# UNSUPPORTED(generator-capability): no pack template emits authority semantics.
# Hand-written verbatim port of `AshGraphLaw.Authority` (GL28-7 design,
# docs/jira/v26.10.28/GL28-7-AUTHORITY-PORT-DESIGN.md) — no Spark, no Ash, no
# `Admissions`, no `Evidence`. Signature and expiry verification stays
# engine-side; this module is a presentation check only.

defmodule GraphlawEx.Authority do
  @moduledoc ~S"""
  The authority pre-gate for `GraphlawEx.Capability.API.run/3` (GL28-7).

  GraphLaw derives and validates; it never authorizes. This module decides only
  whether the caller has *presented* the material a capability's ceiling demands,
  and builds the engine options. It grants nothing.

  ## What counts as a lease

  Only a **signed** lease claims a ceiling above `:observe`: the `:lease` opt is
  a map or keyword list with a `:signed_lease` entry, itself a map
  `%{"lease" => %{"ceiling" => ...}, "attestation" => ...}` (string or atom
  keys). Everything else grants `:observe`: a bare ceiling atom, a map with a
  bare `:ceiling` key, an unsigned `:lease`, an `:unverified_lease`, `nil`.

  A caller can therefore never self-grant by writing an atom into opts. The
  ceiling a signed lease *claims* is compared with the op's floor here, before
  the engine is called; the signed lease is forwarded to the engine verbatim on
  the same request. The pinned engine ignores lease keys entirely
  (`UNSUPPORTED(engine-capability)`): the ONLY lease check enforced today is
  this ceiling pre-check on the *claimed* ceiling. A lease-verifying engine
  would refuse a forged claim as `:lease_refused`; the pinned one does not.

  ## Ceiling ordering (unchanged)

  `:observe(0) < :select(1) < :construct(2)`. A lease satisfies a demand iff
  `rank(claimed) >= rank(required)`; `:observe`-floor ops always pass; a lease
  never widens derivation.

  ## Trust anchors and time

  `engine_opts/1` takes `trusted_keys` and `max_skew_secs` only from caller
  opts and never forwards `now_unix`, `lease` or `unverified_lease`. There is
  no runtime section (no `Admissions`); flagged
  `UNSUPPORTED(generator-capability)`.
  """

  alias GraphlawEx.Capability.Refusal

  @ceiling_rank %{observe: 0, select: 1, construct: 2}
  @ceiling_names %{"observe" => :observe, "select" => :select, "construct" => :construct}

  @op_ceilings %{
    "capabilities" => :observe,
    "sniff" => :observe,
    "parse" => :observe,
    "convert" => :observe,
    "canonical" => :observe,
    "sparql" => :observe,
    "shacl" => :observe,
    "shex" => :observe,
    "policy" => :observe,
    "n3" => :construct,
    "entail" => :construct,
    "datalog" => :construct,
    "hooks" => :construct,
    "law" => :construct
  }

  @typedoc """
  What a lease container presented: the ceiling it claims and the signed lease (if any).

  `signed_lease` is `nil` exactly when `ceiling` is `:observe` because nothing signed was
  presented.
  """
  @type claim :: %{ceiling: :observe | :select | :construct, signed_lease: map() | nil}

  @typedoc """
  Identity of the presented lease for receipts/logs.

  `lease_digest` is the lowercase-hex SHA-256 of the canonical (sorted-key JSON) signed lease.
  `lease_id` and `key_id` are `nil` when the signed lease omits them.
  """
  @type identity :: %{
          ceiling: :observe | :select | :construct,
          lease_id: String.t() | nil,
          key_id: String.t() | nil,
          lease_digest: String.t() | nil
        }

  @doc """
  What `lease` (the `:lease` opt of `run/3`) presents. Only a signed lease claims a ceiling
  above `:observe`.

  Total: any term is accepted, and anything that is not a well-formed signed-lease container
  with a known ceiling claims `:observe` with `signed_lease: nil`.

  ## Examples

      iex> GraphlawEx.Authority.claim(nil)
      %{ceiling: :observe, signed_lease: nil}

      iex> GraphlawEx.Authority.claim(%{ceiling: :construct})
      %{ceiling: :observe, signed_lease: nil}

      iex> signed = %{"lease" => %{"ceiling" => "select"}, "attestation" => %{}}
      iex> GraphlawEx.Authority.claim(%{signed_lease: signed}).ceiling
      :select
  """
  @spec claim(term()) :: claim()
  def claim(lease) do
    with %{} = signed <- signed_lease(lease),
         %{} = body <- get(signed, "lease"),
         ceiling when is_map_key(@ceiling_rank, ceiling) <- ceiling_of(get(body, "ceiling")) do
      %{ceiling: ceiling, signed_lease: signed}
    else
      _ -> %{ceiling: :observe, signed_lease: nil}
    end
  end

  @doc """
  Checks the presented lease against a required ceiling
  (`required ∈ [:observe, :select, :construct]`).

  `:ok`, or `{:error, %GraphlawEx.Capability.Refusal{code: :ceiling_unmet}}` before the
  engine is called. The refusal details carry `required_by`, `required` and `granted`.
  A required ceiling of `:observe` is always `:ok`.

  Presentation check only; the engine still verifies signature, signer and expiry.
  """
  @spec check_ceiling(:observe | :select | :construct, term()) ::
          :ok | {:error, Refusal.t()}
  def check_ceiling(required, lease) when required in [:observe, :select, :construct] do
    %{ceiling: granted} = claim(lease)

    if Map.fetch!(@ceiling_rank, granted) >= Map.fetch!(@ceiling_rank, required) do
      :ok
    else
      {:error,
       Refusal.ceiling_unmet(
         "operation requires ceiling #{required}, lease grants #{granted}",
         %{"required_by" => "operation", "required" => required, "granted" => granted}
       )}
    end
  end

  @doc """
  The minimum ceiling for engine op `op` (atom or string).

  `:observe` for capabilities, sniff, parse, convert, canonical, sparql, shacl, shex and
  policy; `:construct` for n3, entail, datalog, hooks and law. Any other op is a typed
  `:unknown_capability` refusal.
  """
  @spec op_ceiling(atom() | String.t()) ::
          {:ok, :observe | :select | :construct} | {:error, Refusal.t()}
  def op_ceiling(op) when is_atom(op) and not is_nil(op), do: op_ceiling(Atom.to_string(op))

  def op_ceiling(op) when is_binary(op) do
    case Map.fetch(@op_ceilings, op) do
      {:ok, ceiling} ->
        {:ok, ceiling}

      :error ->
        {:error,
         Refusal.build(:unknown_capability, "no capability #{inspect(op)} in the GraphLaw registry", %{
           "op" => op,
           "known" => @op_ceilings |> Map.keys() |> Enum.sort()
         })}
    end
  end

  def op_ceiling(other),
    do: {:error,
         Refusal.build(:unknown_capability, "capability #{inspect(other)} is not a name", %{
           "op" => inspect(other)
         })}

  @doc """
  Checks the lease-vs-op-floor half of the authority gate: the ceiling `lease` claims must
  be at least op `op`'s floor.

  `:observe`-floor ops always pass (back-compat: no lease → pass); a `:construct`-floor op
  (`n3`, `entail`, `datalog`, `hooks`, `law`) with a lease claiming `:observe`/`:select` is
  `{:error, :ceiling_unmet}` before any engine dispatch. Unknown ops are
  `:unknown_capability`. Presentation check only; the engine still verifies signature,
  signer and expiry.
  """
  @spec check_op(atom() | String.t(), term()) :: :ok | {:error, Refusal.t()}
  def check_op(op, lease) do
    with {:ok, required} <- op_ceiling(op) do
      case check_ceiling(required, lease) do
        :ok ->
          :ok

        {:error, %Refusal{} = refusal} ->
          {:error,
           %Refusal{
             refusal
             | message:
                 "capability #{inspect(op)} requires ceiling #{required}, lease grants #{refusal.details["granted"]}",
               details: %{
                 "op" => to_string(op),
                 "required" => required,
                 "granted" => refusal.details["granted"]
               }
           }}
        end
    end
  end
  @doc """
  The engine options `run/3` sends: `:server`, `:timeout`, `:trusted_keys`,
  `:max_skew_secs` from caller opts (no runtime section), plus `:signed_lease` only when a
  well-formed signed lease was presented.

  Never forwards `now_unix`, `lease` or `unverified_lease` (same deny-list as upstream):
  caller context can never supply a clock or an unsigned lease. Every other key is ignored,
  so caller options can never smuggle authority past the gate.
  """
  @spec engine_opts(keyword()) :: keyword()
  def engine_opts(opts) when is_map(opts), do: engine_opts(Map.to_list(opts))

  def engine_opts(opts) when is_list(opts) do
    base =
      [
        server: opts[:server] || GraphlawEx,
        timeout: opts[:timeout] || opts[:timeout_ms]
      ]
      |> maybe_put(:trusted_keys, opts[:trusted_keys])
      |> maybe_put(:max_skew_secs, opts[:max_skew_secs])
      |> Enum.reject(fn {_k, v} -> is_nil(v) end)

    case claim(opts[:lease]) do
      %{signed_lease: %{} = signed} -> base ++ [signed_lease: signed]
      %{signed_lease: nil} -> base
    end
  end

  @doc """
  Identity of the presented lease for receipts/logs: ceiling, lease id, signer key id and
  the SHA-256 of the canonical signed lease. `nil` when no signed lease was presented.
  """
  @spec identity(term()) :: identity() | nil
  def identity(lease) do
    case claim(lease) do
      %{signed_lease: nil} ->
        nil

      %{ceiling: ceiling, signed_lease: signed} ->
        %{
          ceiling: ceiling,
          lease_id: signed |> get("lease") |> get("id"),
          key_id: signed |> get("attestation") |> get("key_id"),
          lease_digest: signed |> normalize() |> canonical_json() |> sha256()
        }
    end
  end

  # -- internals (ported) --------------------------------------------------

  defp maybe_put(opts, _key, nil), do: opts
  defp maybe_put(opts, key, value), do: Keyword.put(opts, key, value)

  defp signed_lease(lease) when is_map(lease) and not is_struct(lease), do: fetch_signed(lease)
  defp signed_lease(lease) when is_list(lease), do: if(Keyword.keyword?(lease), do: fetch_signed(Map.new(lease)))
  defp signed_lease(_lease), do: nil

  defp fetch_signed(container) do
    case Map.get(container, :signed_lease) do
      %{} = signed -> signed
      _ -> nil
    end
  end

  defp ceiling_of(value) when is_binary(value), do: Map.get(@ceiling_names, value)
  defp ceiling_of(value) when is_atom(value) and not is_nil(value), do: ceiling_of(Atom.to_string(value))
  defp ceiling_of(_value), do: nil

  # String or atom keys; total on data.
  defp get(map, key) when is_map(map) do
    case Map.fetch(map, key) do
      {:ok, value} -> value
      :error -> Map.get(map, String.to_existing_atom(key))
    end
  rescue
    ArgumentError -> nil
  end

  defp get(_other, _key), do: nil

  defp normalize(map) when is_map(map) and not is_struct(map),
    do: Map.new(map, fn {k, v} -> {to_string(k), normalize(v)} end)

  defp normalize(list) when is_list(list), do: Enum.map(list, &normalize/1)
  defp normalize(other), do: other

  # Evidence.canonical_json/1 replacement: sorted-key JSON of the normalized
  # (string-keyed) map. Terms that Jason cannot encode fall back to inspect/1
  # per key so identity/1 is total on data.
  defp canonical_json(normalized) do
    normalized
    |> jsonable()
    |> Jason.encode!([])
  rescue
    _ -> Jason.encode!(jsonable_fallback(normalized))
  end

  defp jsonable(%Date{} = d), do: Date.to_iso8601(d)
  defp jsonable(%DateTime{} = d), do: DateTime.to_iso8601(d)
  defp jsonable(map) when is_map(map) and not is_struct(map), do: Map.new(map, fn {k, v} -> {to_string(k), jsonable(v)} end)
  defp jsonable(list) when is_list(list), do: Enum.map(list, &jsonable/1)
  defp jsonable(other) when is_binary(other) or is_number(other) or is_boolean(other) or is_nil(other), do: other
  defp jsonable(other), do: inspect(other)

  defp jsonable_fallback(term), do: %{"unencodable" => inspect(term)}

  defp sha256(bytes), do: :sha256 |> :crypto.hash(bytes) |> Base.encode16(case: :lower)
end
