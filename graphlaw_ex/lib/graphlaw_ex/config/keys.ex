# SPDX-FileCopyrightText: 2026 Sean Chatman <https://github.com/seanchatmangpt/graphlaw/graphs/contributors>
#
# SPDX-License-Identifier: MIT
# projection target: ontologies/graphlaw-elixir.ttl; hand-written until projected
defmodule GraphlawEx.Config.Keys do
  @moduledoc """
  The single table of `config :graphlaw_ex` keys.

  Each row is `{name, type, min, default, zero_ok}`:

    * `type` is `:boolean`, `:path` (non-empty string) or `:limit` (integer);
    * `min` is the smallest accepted integer (`nil` for non-limits);
    * `default` is the built-in value (`nil` when derived or unset; `:fuel`
      derives `timeout_ms * fuel_per_ms`);
    * `zero_ok` is `true` when `0` is a real bound (store counts).

  `GraphlawEx.Config` and `GraphlawEx.WasmConfig` derive their key lists,
  minimums, defaults and typespec-facing semantics from this table.
  `:pool_size` is deliberately absent: pool size is the `size:` option of
  `GraphlawEx.Pool`, not application config.

      iex> GraphlawEx.Config.Keys.min(:memories)
      0
      iex> GraphlawEx.Config.Keys.min(:fuel)
      1
      iex> GraphlawEx.Config.Keys.defaults().timeout_ms
      5000
  """

  @type name :: atom()
  @type row :: {name(), :boolean | :path | :limit, 0 | 1 | nil, term(), boolean()}

  @table [
    {:auto_start, :boolean, nil, false, false},
    {:wasm_path, :path, nil, nil, false},
    {:fuel, :limit, 1, nil, false},
    {:timeout_ms, :limit, 1, 5_000, false},
    {:fuel_per_ms, :limit, 1, 1_000_000, false},
    {:instantiate_fuel, :limit, 1, 1_000_000_000, false},
    {:memory_limit_bytes, :limit, 1, 268_435_456, false},
    {:recycle_bytes, :limit, 1, 134_217_728, false},
    {:max_queue, :limit, 1, 64, false},
    {:max_response_bytes, :limit, 1, 33_554_432, false},
    {:table_elements, :limit, 0, 100_000, true},
    {:instances, :limit, 0, 10, true},
    {:tables, :limit, 0, 10, true},
    {:memories, :limit, 0, 4, true}
  ]

  @doc "The full key table."
  @spec table() :: [row()]
  def table, do: @table

  # Structured keys validated outside the limit table (see GraphlawEx.Config).
  @structured [
    :trusted_keys,
    :kill_switch_path,
    :allow_caller_keys,
    :kill_switch_test_mode,
    :kill_switch_propagate,
    :lease_policy
  ]

  @doc "Structured (non-table) keys: trust anchors, kill-switch settings, lease policy."
  @spec structured_names() :: [name()]
  def structured_names, do: @structured

  @doc """
  Every accepted key name, sorted: the table rows plus the structured keys.

      iex> :lease_policy in GraphlawEx.Config.Keys.names()
      true
      iex> :kill_switch_propagate in GraphlawEx.Config.Keys.names()
      true
  """
  @spec names() :: [name()]
  def names, do: (Enum.map(@table, &elem(&1, 0)) ++ @structured) |> Enum.sort()

  @doc "Limit keys (integer-valued), in table order."
  @spec limit_names() :: [name()]
  def limit_names, do: for({name, :limit, _, _, _} <- @table, do: name)

  @doc "Limit keys where `0` is accepted."
  @spec zero_ok() :: [name()]
  def zero_ok, do: for({name, :limit, _, _, true} <- @table, do: name)

  @doc "Smallest accepted integer for a limit key (1 for unknown keys)."
  @spec min(name()) :: 0 | 1
  def min(key) do
    case List.keyfind(@table, key, 0) do
      {_, :limit, min, _, _} -> min
      _ -> 1
    end
  end

  @doc "Built-in defaults for every limit that has a fixed default (not `:fuel`)."
  @spec defaults() :: %{name() => pos_integer() | 0}
  def defaults do
    for {name, :limit, _, default, _} <- @table,
        is_integer(default),
        into: %{},
        do: {name, default}
  end
end
