# SPDX-FileCopyrightText: 2026 ash_graphlaw contributors <https://github.com/seanchatmangpt/ash_graphlaw/graphs/contributors>
#
# SPDX-License-Identifier: MIT

defmodule GraphlawEx.Capability.Dispatch do
  @moduledoc """
  Hand-written dispatch for `GraphlawEx.Capability.API.run/3`.

  Resolves a wire name through `GraphlawEx.Capability.Registry`, applies the authority
  pre-gate, forwards a presented signed lease, and calls the capability module.
  """

  alias GraphlawEx.Authority
  alias GraphlawEx.Capability.Refusal
  alias GraphlawEx.Capability.Registry

  @doc """
  Runs the capability named `op_name` (a wire name from `GraphlawEx.Capability.Registry.names/0`).

  The authority pre-gate runs BEFORE `module.run/2`: `GraphlawEx.Authority.check_op/2`
  (the `sql` op floor is `:construct`; a malformed lease is `:malformed_lease`) and,
  for `law`, `GraphlawEx.Authority.check_law_steps/2` (floor = max floor of the
  requested steps). A well-formed signed lease is forwarded to the engine on the
  same request with trust anchors from `config :graphlaw_ex, :trusted_keys`
  (fail closed when absent); signature, signer and expiry verification are the
  engine's job. `GraphlawEx.Capability.call/2` re-gates every request.

  Refuses with `:unknown_capability` when the registry has no such capability.
  """
  @spec run(String.t(), GraphlawEx.Capability.args(), keyword()) ::
          {:ok, term()} | {:error, Refusal.t()}
  def run(op_name, args, opts \\ []) when is_binary(op_name) do
    case Registry.module_for(op_name) do
      {:ok, module} ->
        # `:lease` and `:signed_lease` are normalized once; every gate below
        # (and engine_opts/1) sees the same container.
        lease = opts[:lease] || signed_opt(opts)
        opts = put_opt(opts, :lease, lease)

        with :ok <- Authority.check_op(op_name, lease),
             :ok <- check_law_steps(op_name, args, lease),
             :ok <- Authority.check_lease_policy(op_name, lease, opts),
             {:ok, args} <- lease_args(args, module, opts) do
          run_module(op_name, module, args, opts)
        end

      :error ->
        {:error,
         Refusal.build(:unknown_capability, "unknown capability #{inspect(op_name)}", %{
           "op" => op_name
         })}
    end
  end

  # Under `:required`, the finalized law request carries `"lease_policy" =>
  # "required"` so the engine refuses a lease-less chain itself (LeaseRequired).
  # The generated law request schema has no such field, so the request is built
  # here and stamped after coercion, then sent through the same call/decode path
  # as `Capability.Law.run/2`.
  defp run_module("law", module, args, opts) do
    if Authority.effective_lease_policy(opts) == :required do
      # Same build -> call -> decode pipeline as `module.run/2`, with the
      # request stamped; kept in one helper so telemetry/error handling has a
      # single definition on this path.
      GraphlawEx.Telemetry.admit("law", opts, fn ->
        run_pipeline(module, args, opts, &Map.put(&1, "lease_policy", "required"))
      end)
    else
      module.run(args, module_opts(opts))
    end
  end

  defp run_module(_op, module, args, opts), do: module.run(args, module_opts(opts))

  # Capability modules re-gate with `Authority.check_op(@op, opts[:lease])`, so
  # the normalized lease container must travel with the engine options.
  # (Every module filters through `Authority.engine_opts/1` before
  # `Capability.call/2`, so `:lease` never reaches the engine.)
  defp module_opts(opts) do
    engine = Authority.engine_opts(opts)

    case opts[:lease] do
      nil -> engine
      lease -> put_opt(engine, :lease, lease)
    end
  end

  # Options arrive as a keyword list or a map; never put a nil container.
  defp put_opt(opts, _key, nil), do: opts
  defp put_opt(opts, key, value) when is_map(opts), do: Map.put(opts, key, value)
  defp put_opt(opts, key, value), do: Keyword.put(opts, key, value)

  defp run_pipeline(module, args, opts, stamp) do
    with {:ok, request} <- module.build_request(args),
         {:ok, response} <-
           GraphlawEx.Capability.call(stamp.(request), Authority.engine_opts(opts)) do
      module.decode(response)
    end
  end

  defp check_law_steps("law", args, lease) do
    case fetch_arg(args, "steps") do
      steps when is_list(steps) -> Authority.check_law_steps(steps, lease)
      _ -> :ok
    end
  end

  defp check_law_steps(_op, _args, _lease), do: :ok

  # String key first; the atom lookup only happens when the string key is
  # absent and the atom already exists (never mints atoms).
  defp fetch_arg(args, key) when is_map(args) do
    case Map.fetch(args, key) do
      {:ok, v} -> v
      :error -> atom_arg(args, key)
    end
  end

  defp fetch_arg(args, key) when is_list(args) do
    if Keyword.keyword?(args), do: atom_arg(Map.new(args), key), else: nil
  end

  defp fetch_arg(_args, _key), do: nil

  defp atom_arg(map, key) do
    Map.get(map, String.to_existing_atom(key))
  rescue
    ArgumentError -> nil
  end

  @stripped ~w(unverified_lease lease now_unix)

  @doc false
  # Lease forwarding: strips caller-supplied `unverified_lease`/`lease`/
  # `now_unix` from args (Dispatch never adds them), then, when a signed lease
  # was presented, merges it plus trust anchors from `Authority.trusted_keys/1`
  # (Application env) and `max_skew_secs` into the args. A signed lease for a
  # module whose request cannot carry it is `:lease_not_carried`; a signed lease
  # with no configured trusted_keys is `:trusted_keys_absent` (fail closed).
  @spec lease_args(GraphlawEx.Capability.args(), module(), keyword()) ::
          {:ok, GraphlawEx.Capability.args()} | {:error, Refusal.t()}
  def lease_args(args, module, opts) do
    args = strip(args)

    case Authority.claim(opts[:lease] || signed_opt(opts)) do
      {:error, :malformed_lease} ->
        {:error, Refusal.build(:malformed_lease, "presented lease is malformed", %{})}

      %{signed_lease: nil} ->
        {:ok, args}

      %{signed_lease: signed} ->
        forwards = module.fields(:request) |> Map.new(&{&1.name, true})

        cond do
          not Map.has_key?(forwards, "signed_lease") ->
            {:error,
             Refusal.build(
               :lease_not_carried,
               "capability #{inspect(module.op())} cannot carry a signed lease",
               %{"op" => module.op()}
             )}

          is_nil(Authority.trusted_keys(opts)) ->
            {:error,
             Refusal.build(
               :trusted_keys_absent,
               "signed lease presented but no trusted_keys are configured (fail closed)",
               %{"op" => module.op()}
             )}

          true ->
            {:ok,
             args
             |> maybe_arg(forwards, "signed_lease", signed)
             |> maybe_arg(forwards, "trusted_keys", Authority.trusted_keys(opts))
             |> maybe_arg(forwards, "max_skew_secs", opts[:max_skew_secs])}
        end
    end
  end

  defp signed_opt(opts), do: if(opts[:signed_lease], do: %{signed_lease: opts[:signed_lease]})

  defp strip(args) when is_map(args),
    do: Map.reject(args, fn {k, _} -> to_string(k) in @stripped end)

  defp strip(args) when is_list(args),
    do: Enum.reject(args, fn {k, _} -> to_string(k) in @stripped end)

  defp strip(args), do: args

  defp maybe_arg(args, _forwards, _key, nil), do: args

  defp maybe_arg(args, forwards, key, value) do
    if Map.has_key?(forwards, key) do
      if is_list(args) do
        atom_key = String.to_existing_atom(key)
        Keyword.put(args, atom_key, value)
      else
        Map.put(args, key, value)
      end
    else
      args
    end
  end
end
