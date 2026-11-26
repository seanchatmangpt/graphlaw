# SPDX-FileCopyrightText: 2026 Sean Chatman <https://github.com/seanchatmangpt/graphlaw/graphs/contributors>
#
# SPDX-License-Identifier: MIT
# projection target: ontologies/graphlaw-elixir.ttl; hand-written until projected
defmodule GraphlawEx.Config do
  @moduledoc """
  Public validation of `config :graphlaw_ex`.

  Keys: `:auto_start` (boolean), `:wasm_path` (non-empty string), `:fuel`,
  `:timeout_ms`, `:fuel_per_ms`, `:instantiate_fuel`, `:memory_limit_bytes`,
  `:recycle_bytes`, `:max_queue`, `:max_response_bytes` (pos integers), and
  `:table_elements`, `:instances`, `:tables`, `:memories` (integers >= 0).
  `:lease_policy` is `:required` (default, fail closed: a lease-less `law` request is
  refused) or `:none_allowed`; request opts can only tighten it, never loosen it.
  The table lives in `GraphlawEx.Config.Keys`; limit semantics live in
  `GraphlawEx.WasmConfig`. Pool size is not a config key: pass `size: n` to
  `GraphlawEx.Pool` (a `:pool_size` entry is rejected with that redirect).

      iex> GraphlawEx.Config.validate(timeout_ms: 100, auto_start: true)
      :ok
      iex> {:error, [msg]} = GraphlawEx.Config.validate(pool_size: 4)
      iex> msg =~ "size: n"
      true
      iex> {:error, [msg]} = GraphlawEx.Config.validate(timeout: 100)
      iex> msg =~ "did you mean :timeout_ms"
      true
      iex> {:error, [msg]} = GraphlawEx.Config.validate(lease_polcy: :required)
      iex> msg =~ "did you mean :lease_policy"
      true
      iex> Enum.all?(
      ...>   [:trusted_keys, :kill_switch_path, :allow_caller_keys, :kill_switch_test_mode,
      ...>    :lease_policy, :kill_switch_propagate],
      ...>   &(&1 in GraphlawEx.Config.keys())
      ...> )
      true
      iex> GraphlawEx.Config.validate(kill_switch_propagate: {Kernel, :send})
      :ok
  """

  alias GraphlawEx.Config.Keys

  @keys Keys.names()
  @limit_keys Keys.limit_names() ++ [:auto_start, :wasm_path]

  @doc "Every key accepted under `config :graphlaw_ex`."
  @spec keys() :: [atom()]
  def keys, do: @keys

  @doc "Smallest accepted integer for a limit key (0 for store counts, else 1)."
  @spec min_limit(atom()) :: 0 | 1
  def min_limit(key), do: Keys.min(key)

  @doc "Returns `:ok` or `{:error, messages}`, one actionable message per problem."
  @spec validate(keyword()) :: :ok | {:error, [String.t()]}
  def validate(config \\ Application.get_all_env(:graphlaw_ex)) do
    case Enum.flat_map(config, &check_entry/1) ++ check_cross(config) do
      [] -> :ok
      errors -> {:error, errors}
    end
  end

  # kill_switch_test_mode lets callers trip/clear the switch without authority;
  # it is refused when the application boots a live engine with a durable
  # kill-switch path (auto_start: true plus :kill_switch_path).
  defp check_cross(config) do
    if config[:kill_switch_test_mode] == true and config[:auto_start] == true and
         is_binary(config[:kill_switch_path]) do
      [
        "`:kill_switch_test_mode` must not be true together with `auto_start: true` " <>
          "and `:kill_switch_path` (test mode is for test environments only)"
      ]
    else
      []
    end
  end

  @doc false
  # Accepted shape mirrors `GraphlawEx.Authority.trusted_keys/1`: nil/[] (no
  # anchors) or a list of non-empty strings.
  @spec trusted_keys_errors(term()) :: [String.t()]
  def trusted_keys_errors(nil), do: []

  def trusted_keys_errors(keys) when is_list(keys) do
    if Enum.all?(keys, &(is_binary(&1) and &1 != "")),
      do: [],
      else: ["`:trusted_keys` must be a list of non-empty strings, got #{inspect(keys)}"]
  end

  def trusted_keys_errors(other),
    do: ["`:trusted_keys` must be a list of non-empty strings, got #{inspect(other)}"]

  @doc "Like `validate/1` but raises `ArgumentError` listing every problem."
  @spec validate!(keyword()) :: :ok
  def validate!(config \\ Application.get_all_env(:graphlaw_ex)) do
    case validate(config) do
      :ok ->
        :ok

      {:error, errors} ->
        raise ArgumentError,
              "invalid `config :graphlaw_ex`:\n" <>
                Enum.map_join(errors, "\n", &("  * " <> &1)) <>
                "\nValid keys: #{inspect(keys())}. See GraphlawEx.Config docs."
    end
  end

  defp check_entry({:included_applications, _value}), do: []

  # Trust and kill-switch settings live outside the limit table: they are
  # structured values (key maps, paths, flags), validated at their point of use.
  defp check_entry({:trusted_keys, value}), do: trusted_keys_errors(value)
  defp check_entry({:kill_switch_path, value}) when is_binary(value) and value != "", do: []

  defp check_entry({:kill_switch_path, value}),
    do: ["`:kill_switch_path` must be a non-empty string path, got #{inspect(value)}"]

  defp check_entry({key, value}) when key in [:allow_caller_keys, :kill_switch_test_mode] do
    if is_boolean(value),
      do: [],
      else: ["`#{inspect(key)}` must be true or false, got #{inspect(value)}"]
  end

  defp check_entry({:kill_switch_propagate, {mod, fun}}) when is_atom(mod) and is_atom(fun),
    do: []

  defp check_entry({:kill_switch_propagate, value}),
    do: ["`:kill_switch_propagate` must be a {module, function} tuple, got #{inspect(value)}"]

  defp check_entry({:lease_policy, value}) when value in [:required, :none_allowed], do: []

  defp check_entry({:lease_policy, value}),
    do: ["`:lease_policy` must be :required or :none_allowed, got #{inspect(value)}"]

  defp check_entry({:auto_start, value}) when is_boolean(value), do: []

  defp check_entry({:auto_start, value}),
    do: ["`:auto_start` must be true or false, got #{inspect(value)}"]

  defp check_entry({:wasm_path, value}) when is_binary(value) and value != "", do: []

  defp check_entry({:wasm_path, value}),
    do: ["`:wasm_path` must be a non-empty string path, got #{inspect(value)}"]

  defp check_entry({key, value}) when key in @limit_keys do
    min = min_limit(key)

    if is_integer(value) and value >= min,
      do: [],
      else: ["`#{inspect(key)}` must be an integer >= #{min}, got #{inspect(value)}"]
  end

  defp check_entry({:pool_size, _value}),
    do: [
      "`:pool_size` is not a config key: pass `size: n` to `GraphlawEx.Pool` " <>
        "(default `System.schedulers_online/0`)"
    ]

  defp check_entry({key, _value}) when is_atom(key) do
    hint =
      @keys
      |> Enum.map(&{&1, String.jaro_distance(Atom.to_string(&1), Atom.to_string(key))})
      |> Enum.max_by(&elem(&1, 1))
      |> case do
        {best, score} when score >= 0.8 -> " (did you mean #{inspect(best)}?)"
        _ -> ""
      end

    ["unknown key #{inspect(key)}#{hint}"]
  end

  defp check_entry({key, _value}), do: ["config key must be an atom, got #{inspect(key)}"]
end
