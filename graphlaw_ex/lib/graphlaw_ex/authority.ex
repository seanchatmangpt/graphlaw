# SPDX-License-Identifier: MIT
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
  keys; inner lease keys are string keys only). `:unverified_lease` is never
  honoured by this library: it is stripped from every request not built by
  `GraphlawEx.Capability.Dispatch` and is never forwarded, so a container carrying
  only it claims `:observe`. A container without `:signed_lease` (nil, a bare
  `:ceiling` key) claims `:observe`. A **non-nil lease that cannot be parsed**
  (non-container, non-map `:signed_lease`, missing lease body, unknown ceiling)
  is `{:error, :malformed_lease}` from `claim/1`, and every `check_*` refuses it
  with a typed `:malformed_lease` refusal rather than degrading to observe.

  A caller can therefore never self-grant by writing an atom into opts. When a
  caller *presents* a signed lease, its *claimed* ceiling is compared with the
  op's floor here, before the engine is called; the lease is forwarded to the
  engine on the same request. The kernel enforces a per-step lease check
  (`LeaseAuth::authorize` runs on every `Step`): this module is an additive
  presentation check. `GraphlawEx.Capability.call/2` runs `check_request/2`
  on every request, so no `Capability.<Op>.run/2` bypasses the gate.

  ## n3/entail/datalog/hooks floors and the step-derived check

  Their op floors stay `:observe` (the previous `:construct` floors gated only on a
  forgeable claimed ceiling). The `law` op instead carries a step-derived
  check, `check_law_steps/2`: the chain's floor is the maximum floor of its
  requested steps (`derive:n3`, `derive:hooks`, ... are `:construct`). With a
  signed lease each step is checked against scope and ceiling; lease-less chains
  stay kernel-legal and pass (existing lease-less courts pin that).

  ## The `sql` floor is `:construct`

  Unlike the ops above, `sql` has an explicit `:construct` floor: a lease-less
  call or an unsigned lease (claims `:observe`) is refused `:ceiling_unmet`, and a
  malformed lease `:malformed_lease`, before engine dispatch. Only a
  well-formed signed lease claiming `:construct` passes.

  ## Ceiling ordering (unchanged)

  `:observe(0) < :select(1) < :construct(2)`. A lease satisfies a demand iff
  `rank(claimed) >= rank(required)`; `:observe`-floor ops always pass; a lease
  never widens derivation.

  ## Trust anchors and time

  `trusted_keys/1` reads `config :graphlaw_ex, :trusted_keys` only; caller opts
  are consulted only under `config :graphlaw_ex, allow_caller_keys: true`. A signed
  lease with no trust anchors fails closed before the engine. `engine_opts/1`
  takes `max_skew_secs` from caller opts and never forwards `now_unix`, `lease`
  or `unverified_lease`. There is
  no runtime section (no `Admissions`); flagged
  `UNSUPPORTED(generator-capability)`.
  """

  alias GraphlawEx.Capability.Refusal

  # Tables live in GraphlawEx.Authority.Table (pure data, compile-time reads).
  alias GraphlawEx.Authority.Table

  @ceiling_rank Table.ceiling_rank()
  @ceiling_names Table.ceiling_names()
  @law_step_names Table.step_names()
  @law_step_required Table.step_required()
  @op_ceilings Table.op_ceilings()

  @typedoc """
  What a lease container presented: the ceiling it claims and the signed lease (if any).

  `signed_lease` is `nil` exactly when `ceiling` is `:observe` because nothing signed was
  presented. `scope` is the lease's requested law-step scope (default `[]`);
  `expires_unix` is the lease's expiry as claimed in its body (`nil` when absent —
  no local expiry check is performed; the kernel owns the clock).
  """
  @type claim :: %{
          ceiling: :observe | :select | :construct,
          signed_lease: map() | nil,
          scope: [String.t()],
          expires_unix: integer() | nil
        }

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
  with a known ceiling claims `:observe` with `signed_lease: nil`, `scope: []` and
  `expires_unix: nil`.

  ## Examples

      iex> GraphlawEx.Authority.claim(nil)
      %{ceiling: :observe, signed_lease: nil, scope: [], expires_unix: nil}

      iex> GraphlawEx.Authority.claim(%{ceiling: :construct})
      %{ceiling: :observe, signed_lease: nil, scope: [], expires_unix: nil}

      iex> signed = %{"lease" => %{"ceiling" => "select"}, "attestation" => %{}}
      iex> GraphlawEx.Authority.claim(%{signed_lease: signed}).ceiling
      :select
  """
  @spec claim(term()) :: claim() | {:error, :malformed_lease}
  def claim(nil), do: observe_claim()

  def claim(lease) do
    case signed_lease(lease) do
      :none ->
        observe_claim()

      {:ok, signed} ->
        with %{} = body <- get(signed, "lease"),
             ceiling when is_map_key(@ceiling_rank, ceiling) <- ceiling_of(get(body, "ceiling")) do
          %{
            ceiling: ceiling,
            signed_lease: signed,
            scope: scope_of(get(body, "scope")),
            expires_unix: expires_unix_of(get(body, "expires_unix"))
          }
        else
          _ -> {:error, :malformed_lease}
        end

      :malformed ->
        {:error, :malformed_lease}
    end
  end

  defp observe_claim, do: %{ceiling: :observe, signed_lease: nil, scope: [], expires_unix: nil}

  defp malformed_refusal do
    Refusal.build(
      :malformed_lease,
      "presented lease is malformed and was not silently downgraded to observe",
      %{"refusal_code" => "malformed_lease"}
    )
  end

  @typedoc """
  What `authorize/2` is asked to authorize: an engine op name (atom or string), or
  `{:steps, names}` for a law step list, or `{:ceiling, ceiling}` for a bare floor.
  """
  @type subject ::
          atom()
          | String.t()
          | {:steps, [String.t() | atom()]}
          | {:ceiling, :observe | :select | :construct}

  @doc """
  Policy-style entry point: is the lease presented by the caller sufficient for `subject`?

  Pure and total on data. Returns `:ok` or `{:error, %GraphlawEx.Error{}}`; every denial
  maps to `:REFUSED_INVALID_INPUT`, with the typed cause preserved in
  `error.evidence["refusal_code"]` (`"ceiling_unmet"` or `"unknown_capability"`) and
  the required/granted ceilings in `error.evidence["details"]`. Consumers (for example an
  Ash policy layer) can map `ceiling_unmet` to a forbidden class and
  `unknown_capability` to an unknown/invalid class without this library depending on them.

  Authority inputs, in order of evaluation: (1) the subject's required ceiling, (2) the
  ceiling claimed by the *signed* lease, (3) the lease scope (for steps). The operator
  kill-switch (`GraphlawEx.KillSwitch`) is a further, independent authority input enforced
  per call by the host after this presentation check; it is not consulted here.
  Presentation check only; the kernel verifies signature, signer and expiry.

  ## Examples

      iex> GraphlawEx.Authority.authorize("sparql", nil)
      :ok

      iex> {:error, e} = GraphlawEx.Authority.authorize("no-such-op", nil)
      iex> {e.code, e.evidence["refusal_code"]}
      {:REFUSED_INVALID_INPUT, "unknown_capability"}

      iex> {:error, e} = GraphlawEx.Authority.authorize({:ceiling, :construct}, nil)
      iex> {e.code, e.evidence["refusal_code"]}
      {:REFUSED_LEASE_REQUIRED, "ceiling_unmet"}
  """
  @spec authorize(subject(), term()) :: :ok | {:error, GraphlawEx.Error.t()}
  def authorize({:steps, steps}, lease) when is_list(steps),
    do: steps |> check_steps(lease) |> wrap_result()

  def authorize({:ceiling, required}, lease) when required in [:observe, :select, :construct],
    do: required |> check_ceiling(lease) |> wrap_result()

  def authorize(op, lease), do: op |> check_op(lease) |> wrap_result()

  defp wrap_result(:ok), do: :ok
  defp wrap_result({:error, refusal}), do: {:error, GraphlawEx.Error.wrap(refusal)}

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
    case claim(lease) do
      {:error, :malformed_lease} ->
        {:error, malformed_refusal()}

      %{ceiling: granted} ->
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
  end

  @doc """
  The minimum ceiling for engine op `op` (atom or string).

  `:observe` for capabilities, sniff, parse, convert, canonical, sparql, shacl, shex,
  policy, n3, entail, datalog, hooks and law; `:construct` for `sql`. Any other op is a
  typed `:unknown_capability` refusal.
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
         Refusal.build(
           :unknown_capability,
           "no capability #{inspect(op)} in the GraphLaw registry",
           %{
             "op" => op,
             "known" => @op_ceilings |> Map.keys() |> Enum.sort()
           }
         )}
    end
  end

  def op_ceiling(other),
    do:
      {:error,
       Refusal.build(:unknown_capability, "capability #{inspect(other)} is not a name", %{
         "op" => inspect(other)
       })}

  @doc """
  Checks the lease-vs-op-floor half of the authority gate: when a lease is
  presented, the ceiling it claims must be at least op `op`'s floor.

  `:observe`-floor ops always pass (back-compat: no lease → pass). A presented
  lease claiming `:observe` on a `:select`/`:construct`-floor op is
  `{:error, :ceiling_unmet}` before any engine dispatch. Unknown ops are
  `:unknown_capability`. Presentation check only; the kernel verifies signature,
  signer, scope and expiry per step.
  """
  @spec check_op(atom() | String.t(), term()) :: :ok | {:error, Refusal.t()}
  def check_op(op, lease) do
    with {:ok, required} <- op_ceiling(op) do
      case check_ceiling(required, lease) do
        :ok ->
          :ok

        {:error, %Refusal{code: :ceiling_unmet} = refusal} ->
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

        {:error, %Refusal{}} = other ->
          other
      end
    end
  end

  @doc """
  Checks the presented lease against the requested law `steps` (names as in
  `Step::name`, src/law.rs:118-127).

  For each step: an unknown step name is `{:error, :unknown_capability}`;
  a step outside the lease's `scope` is `{:error, :ceiling_unmet}` with
  `"required_by" => "step"`; a step whose required ceiling exceeds the claimed
  ceiling is `{:error, :ceiling_unmet}`. No local expiry check — the kernel owns
  the clock. Presentation check only; the kernel re-checks every step.
  """
  @spec check_steps([String.t() | atom()], term()) :: :ok | {:error, Refusal.t()}
  def check_steps(steps, lease) when is_list(steps) do
    case claim(lease) do
      {:error, :malformed_lease} ->
        {:error, malformed_refusal()}

      %{ceiling: granted, scope: scope} ->
        Enum.reduce_while(steps, :ok, fn step, :ok ->
          with :ok <- check_step(step, granted, scope) do
            {:cont, :ok}
          else
            {:error, %Refusal{} = refusal} -> {:halt, {:error, refusal}}
          end
        end)
    end
  end

  @doc """
  Step-derived floor for the `law` op: the `law` op floor is `:observe`, but the
  chain's effective floor is the maximum floor of its requested steps.

  With a signed lease presented this is `check_steps/2` (scope + ceiling per
  step; bare wire tags such as `"rdfs"` resolve to `"derive:rdfs"`). Lease-less
  chains are kernel-legal (`LeaseAuth::None`), so only step names are validated
  (unknown steps are `:unknown_capability`). A malformed lease is
  `:malformed_lease`.
  """
  @spec check_law_steps([term()], term()) :: :ok | {:error, Refusal.t()}
  def check_law_steps(steps, lease) when is_list(steps) do
    names = Enum.map(steps, &step_name/1)

    case claim(lease) do
      {:error, :malformed_lease} ->
        {:error, malformed_refusal()}

      %{signed_lease: nil} ->
        # Lease-less chains are kernel-legal (`LeaseAuth::None`; existing
        # lease-less rdfs/n3 courts pin this), so only step names are validated.
        known_steps(names)

      %{} ->
        check_steps(names, lease)
    end
  end

  # Wire-only steps: handled by the ABI loop, no `Step` variant, no ceiling
  # (registry `record-receipts`, ceiling None), so never scope-gated.
  @wire_only_steps ["record-receipts"]

  defp known_steps(names) do
    case Enum.find(
           names,
           &(not Map.has_key?(@law_step_required, &1) and &1 not in @wire_only_steps)
         ) do
      nil ->
        :ok

      bad ->
        {:error,
         Refusal.build(:unknown_capability, "no law step #{inspect(bad)}", %{
           "step" => inspect(bad),
           "known" => @law_step_names
         })}
    end
  end

  defp step_name(%{"step" => name}) when is_binary(name), do: canonical_step(name)

  defp step_name(%{step: name}) when is_binary(name) or is_atom(name),
    do: canonical_step(to_string(name))

  defp step_name(name) when is_binary(name), do: canonical_step(name)

  defp step_name(name) when is_atom(name) and not is_nil(name),
    do: canonical_step(Atom.to_string(name))

  defp step_name(other), do: inspect(other)

  # Wire step tags are bare ("rdfs", "sql"); Step::name is prefixed
  # ("derive:rdfs", "admit:sql"). Resolve bare tags to the canonical name.
  defp canonical_step(name) do
    Enum.find(
      [name, "admit:" <> name, "derive:" <> name, "test:" <> name],
      name,
      &Map.has_key?(@law_step_required, &1)
    )
  end

  @doc """
  The configured lease policy (`config :graphlaw_ex, :lease_policy`), default
  `:required` (fail closed).
  """
  @spec lease_policy() :: :required | :none_allowed
  def lease_policy do
    case Application.get_env(:graphlaw_ex, :lease_policy, :required) do
      :none_allowed -> :none_allowed
      _ -> :required
    end
  end

  @doc """
  The policy in force for one call: the configured policy, tightened (never
  loosened) by `opts[:lease_policy] == :required`.
  """
  @spec effective_lease_policy(keyword()) :: :required | :none_allowed
  def effective_lease_policy(opts \\ []) do
    if lease_policy() == :required or opts[:lease_policy] == :required,
      do: :required,
      else: :none_allowed
  end

  @doc """
  Lease-policy gate: under `:required`, a `law` request presenting no signed lease
  is refused up front (`:ceiling_unmet`, public `:REFUSED_LEASE_REQUIRED`), matching
  the engine's `LeaseRequired`. A malformed lease is `:malformed_lease`. Other ops pass.
  """
  @spec check_lease_policy(String.t(), term(), keyword()) :: :ok | {:error, Refusal.t()}
  def check_lease_policy("law", lease, opts) do
    case {effective_lease_policy(opts), claim(lease)} do
      {_, {:error, :malformed_lease}} ->
        {:error, malformed_refusal()}

      {:required, %{signed_lease: nil}} ->
        {:error,
         Refusal.ceiling_unmet(
           "lease policy is required: a law request must present a signed lease",
           %{"required_by" => "lease_policy", "op" => "law", "lease_policy" => "required"}
         )}

      _ ->
        :ok
    end
  end

  def check_lease_policy(_op, _lease, _opts), do: :ok

  @doc """
  Full request gate used by `GraphlawEx.Capability.call/2`: op floor from
  `request["op"]` plus, for `law`, the step-derived floor of `request["steps"]`
  and the lease policy (`check_lease_policy/3`).
  """
  @spec check_request(map(), term(), keyword()) :: :ok | {:error, Refusal.t()}
  def check_request(request, lease, opts \\ [])

  def check_request(%{"op" => op} = request, lease, opts) when is_binary(op) do
    with :ok <- check_op(op, lease),
         :ok <- check_lease_policy(op, lease, opts) do
      case {op, request["steps"]} do
        {"law", steps} when is_list(steps) ->
          check_law_steps(steps, lease)

        {"law", nil} ->
          :ok

        {"law", other} ->
          # fail closed: a non-list step chain cannot be floor-checked
          {:error,
           Refusal.build(:unknown_capability, "law steps are not a list", %{
             "step" => inspect(other)
           })}

        _ ->
          :ok
      end
    end
  end

  def check_request(_request, _lease, _opts),
    do: {:error, Refusal.build(:unknown_capability, "request carries no \"op\"", %{})}

  @doc """
  Trust anchors for lease verification. Read from `Application` env
  `:graphlaw_ex, :trusted_keys` only. Caller opts are consulted only when
  `config :graphlaw_ex, allow_caller_keys: true`. `nil` when none: callers
  presenting a signed lease must then fail closed.
  """
  @spec trusted_keys(keyword()) :: [String.t()] | nil
  def trusted_keys(opts \\ []) do
    configured = nonempty_keys(Application.get_env(:graphlaw_ex, :trusted_keys))

    cond do
      configured ->
        configured

      Application.get_env(:graphlaw_ex, :allow_caller_keys, false) ->
        nonempty_keys(opts[:trusted_keys])

      true ->
        nil
    end
  end

  defp nonempty_keys(keys) when is_list(keys) and keys != [], do: keys
  defp nonempty_keys(_), do: nil

  defp check_step(step, granted, scope) when is_atom(step) and not is_nil(step),
    do: check_step(Atom.to_string(step), granted, scope)

  defp check_step(step, _granted, _scope) when not is_binary(step) do
    {:error,
     Refusal.build(:unknown_capability, "no law step #{inspect(step)}", %{
       "step" => inspect(step),
       "known" => @law_step_names
     })}
  end

  defp check_step(step, granted, scope) when is_binary(step) do
    cond do
      step in @wire_only_steps ->
        :ok

      not Map.has_key?(@law_step_required, step) ->
        {:error,
         Refusal.build(:unknown_capability, "no law step #{inspect(step)}", %{
           "step" => step,
           "known" => @law_step_names
         })}

      step not in scope ->
        {:error,
         Refusal.ceiling_unmet("step #{step} is outside the lease scope", %{
           "required_by" => "step",
           "step" => step,
           "scope" => scope
         })}

      true ->
        required = Map.fetch!(@law_step_required, step)

        if Map.fetch!(@ceiling_rank, granted) >= Map.fetch!(@ceiling_rank, required) do
          :ok
        else
          {:error,
           Refusal.ceiling_unmet(
             "step #{step} requires ceiling #{required}, lease grants #{granted}",
             %{
               "required_by" => "step",
               "step" => step,
               "required" => required,
               "granted" => granted
             }
           )}
        end
    end
  end

  @doc """
  The engine options `run/3` sends: `:server`, `:timeout`, `:max_skew_secs` from caller
  opts, `:trusted_keys` from `trusted_keys/1` (Application env; never caller opts unless
  `allow_caller_keys` is configured), plus `:signed_lease` only when a well-formed signed
  lease was presented. A malformed lease contributes nothing here; the gate
  (`check_op/2`) refuses it before this is reached.

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
      |> maybe_put(:trusted_keys, trusted_keys(opts))
      |> maybe_put(:max_skew_secs, opts[:max_skew_secs])
      # GL28-1 leg (b): capability-verb callers may pin the source digest of
      # the law content being executed; nil (absent) is dropped, shape unchanged.
      |> maybe_put(:source_digest, opts[:source_digest])
      # Tighten-only: an explicit :required is forwarded so Capability.call/2
      # enforces it; :none_allowed is never forwarded (cannot loosen config).
      |> maybe_put(:lease_policy, if(opts[:lease_policy] == :required, do: :required))
      |> Enum.reject(fn {_k, v} -> is_nil(v) end)

    case claim(opts[:lease] || signed_container(opts)) do
      %{signed_lease: %{} = signed} -> base ++ [signed_lease: signed]
      _ -> base
    end
  end

  @doc """
  Identity of the presented lease for receipts/logs: ceiling, lease id, signer key id and
  the SHA-256 of the canonical signed lease. `nil` when no signed lease was presented.
  """
  @spec identity(term()) :: identity() | nil
  def identity(lease) do
    case claim(lease) do
      {:error, :malformed_lease} ->
        nil

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

  @doc false
  # Lease container from a flat opts list carrying `:signed_lease` directly
  # (the shape `engine_opts/1` emits and `Capability.call/2` receives).
  @spec signed_container(keyword() | map()) :: map() | nil
  def signed_container(opts) when is_list(opts) or is_map(opts) do
    case opts[:signed_lease] do
      nil -> nil
      signed -> %{signed_lease: signed}
    end
  end

  defp maybe_put(opts, _key, nil), do: opts
  defp maybe_put(opts, key, value), do: Keyword.put(opts, key, value)

  defp signed_lease(lease) when is_map(lease) and not is_struct(lease), do: fetch_signed(lease)

  defp signed_lease(lease) when is_list(lease),
    do: if(Keyword.keyword?(lease), do: fetch_signed(Map.new(lease)), else: :malformed)

  defp signed_lease(_lease), do: :malformed

  defp fetch_signed(container) do
    value =
      case Map.fetch(container, :signed_lease) do
        {:ok, v} -> {:ok, v}
        :error -> Map.fetch(container, "signed_lease")
      end

    case value do
      :error ->
        :none

      {:ok, nil} ->
        :none

      {:ok, %GraphlawEx.Model.SignedLease{} = signed} ->
        {:ok, GraphlawEx.Model.SignedLease.to_map(signed)}

      {:ok, %_{}} ->
        :malformed

      {:ok, %{} = signed} ->
        {:ok, signed}

      {:ok, _other} ->
        :malformed
    end
  end

  defp ceiling_of(value) when is_binary(value), do: Map.get(@ceiling_names, value)

  defp ceiling_of(value) when is_atom(value) and not is_nil(value),
    do: ceiling_of(Atom.to_string(value))

  defp ceiling_of(_value), do: nil

  defp scope_of(list) when is_list(list),
    do: Enum.filter(list, &is_binary/1)

  defp scope_of(_other), do: []

  defp expires_unix_of(n) when is_integer(n), do: n
  defp expires_unix_of(_other), do: nil

  # String keys only (no atom fallback); total on data.
  defp get(map, key) when is_map(map), do: Map.get(map, key)

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

  defp jsonable(map) when is_map(map) and not is_struct(map),
    do: Map.new(map, fn {k, v} -> {to_string(k), jsonable(v)} end)

  defp jsonable(list) when is_list(list), do: Enum.map(list, &jsonable/1)

  defp jsonable(other)
       when is_binary(other) or is_number(other) or is_boolean(other) or is_nil(other), do: other

  defp jsonable(other), do: inspect(other)

  defp jsonable_fallback(term), do: %{"unencodable" => inspect(term)}

  defp sha256(bytes), do: :sha256 |> :crypto.hash(bytes) |> Base.encode16(case: :lower)
end
