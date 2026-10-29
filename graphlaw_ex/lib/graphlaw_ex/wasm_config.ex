defmodule GraphlawEx.WasmConfig do
  @moduledoc """
  Resolves where the GraphLaw engine lives, which digest it must have, and the
  resource limits the host runs it under.

  ## Path resolution

  In order: `opts[:wasm_path]`, the `GRAPHLAW_WASM_PATH` environment variable,
  `config :graphlaw_ex, :wasm_path`, then the vendored
  `priv/graphlaw/graphlaw.wasm` inside this application.

  ## Pin

  The pin is `artifact.sha256` in `priv/graphlaw/MANIFEST.json`.
  `expected_sha256/1` decides which digest a load must match:

    * `opts[:expected_sha256]` wins when given (`:unpinned` is the explicit
      opt-out);
    * an explicit `opts[:wasm_path]` that differs from the vendored path is
      `:unpinned` (the caller chose the bytes and says so in code);
    * anything else, including a path supplied through the environment or
      application config, STAYS pinned to the manifest, so the environment can
      not swap the admission engine.

  If the manifest is unreadable the pin is `nil`; loaders must treat `nil` as a
  refusal, never as "unpinned".

  ## Import allowlist

  `import_allowlist/0` reads `host_abi.imports` from the manifest: the closed
  set of `{name, {:fn, params, results}}` WASI functions the engine may import.
  `GraphlawEx.EngineLoad` refuses any other import.

  ## Limits

  `limits/1` bounds every request. `fuel` defaults to `timeout_ms * fuel_per_ms`,
  so the CPU budget follows the deadline instead of being a separate, looser
  number. `max_response_bytes` caps what the host copies out of engine memory.

  ## Usage

      GraphlawEx.WasmConfig.wasm_path()
      GraphlawEx.WasmConfig.expected_sha256(wasm_path: "/tmp/other.wasm")
      #=> :unpinned

  ## Failure modes

  `manifest/0` returns `:wasm_unreadable` when `MANIFEST.json` cannot be read
  and `:invalid_json` when it is not a JSON object. `pinned_sha256/0` collapses
  every manifest defect to `nil`; `import_allowlist/0` collapses them to
  `:unavailable`. Both are fail-closed for `GraphlawEx.EngineLoad`.
  """

  alias GraphlawEx.Refusal

  @env "GRAPHLAW_WASM_PATH"
  @manifest_rel "priv/graphlaw/MANIFEST.json"
  @wasm_rel "priv/graphlaw/graphlaw.wasm"

  @defaults %{
    fuel_per_ms: 1_000_000,
    instantiate_fuel: 1_000_000_000,
    memory_limit_bytes: 268_435_456,
    recycle_bytes: 134_217_728,
    max_queue: 64,
    max_response_bytes: 33_554_432,
    table_elements: 100_000,
    instances: 10,
    tables: 10,
    memories: 4,
    timeout_ms: 5_000
  }

  @typedoc "Resource limits applied by `GraphlawEx.Host`."
  @type limits :: %{
          fuel: pos_integer(),
          fuel_per_ms: pos_integer(),
          instantiate_fuel: pos_integer(),
          memory_limit_bytes: pos_integer(),
          recycle_bytes: pos_integer(),
          max_queue: pos_integer(),
          max_response_bytes: pos_integer(),
          table_elements: pos_integer(),
          instances: pos_integer(),
          tables: pos_integer(),
          memories: pos_integer(),
          timeout_ms: pos_integer()
        }

  @typedoc "Options accepted by the resolvers in this module (all optional)."
  @type opts :: [
          wasm_path: String.t(),
          expected_sha256: String.t() | :unpinned,
          fuel: pos_integer(),
          timeout_ms: pos_integer(),
          fuel_per_ms: pos_integer(),
          instantiate_fuel: pos_integer(),
          memory_limit_bytes: pos_integer(),
          recycle_bytes: pos_integer(),
          max_queue: pos_integer(),
          max_response_bytes: pos_integer(),
          table_elements: pos_integer(),
          instances: pos_integer(),
          tables: pos_integer(),
          memories: pos_integer()
        ]

  @typedoc "One allowed WASI import: `{name, {:fn, params, results}}` with wasm value-type atoms."
  @type import_allow :: {String.t(), {:fn, [atom()], [atom()]}}

  @doc "The vendored engine path inside this application's `priv` directory."
  @spec vendored_path() :: String.t()
  def vendored_path, do: app_path(@wasm_rel)

  @doc "Resolves the engine path; see the module doc for the order."
  @spec wasm_path(opts()) :: String.t()
  def wasm_path(opts \\ []) do
    present(Keyword.get(opts, :wasm_path)) ||
      present(System.get_env(@env)) ||
      present(Application.get_env(:graphlaw_ex, :wasm_path)) ||
      vendored_path()
  end

  @doc "Reads and decodes `priv/graphlaw/MANIFEST.json`."
  @spec manifest() :: {:ok, map()} | {:error, Refusal.t()}
  def manifest do
    path = app_path(@manifest_rel)

    with {:ok, raw} <- read_manifest(path) do
      decode_manifest(raw, path)
    end
  end

  @doc "The pinned engine SHA-256 (`artifact.sha256`), or `nil` when the manifest is unusable."
  @spec pinned_sha256() :: String.t() | nil
  def pinned_sha256 do
    with {:ok, manifest} <- manifest(),
         sha when is_binary(sha) <- get_in(manifest, ["artifact", "sha256"]),
         true <- sha256_hex?(sha) do
      String.downcase(sha)
    else
      _ -> nil
    end
  end

  @doc """
  The digest a load must match: a hex string, `:unpinned`, or `nil` when the
  pin is unavailable (fail closed).
  """
  @spec expected_sha256(opts()) :: String.t() | :unpinned | nil
  def expected_sha256(opts \\ []) do
    case Keyword.fetch(opts, :expected_sha256) do
      {:ok, value} -> value
      :error -> default_expected(opts)
    end
  end

  @doc """
  Resource limits: `opts` over `config :graphlaw_ex` over the defaults. `fuel` is
  `opts[:fuel]` (or config) when given, else `timeout_ms * fuel_per_ms`.
  """
  @spec limits(opts()) :: limits()
  def limits(opts \\ []) do
    base = Map.new(@defaults, fn {key, default} -> {key, limit(opts, key, default)} end)
    Map.put(base, :fuel, limit(opts, :fuel, base.timeout_ms * base.fuel_per_ms))
  end

  @doc """
  The closed WASI import allowlist from the manifest (`host_abi.imports`), or
  `:unavailable` when the manifest is unusable or declares none (loaders must
  refuse, never allow-all).
  """
  @spec import_allowlist() :: {:ok, [import_allow()]} | :unavailable
  def import_allowlist do
    with {:ok, manifest} <- manifest(),
         [_ | _] = rows <- get_in(manifest, ["host_abi", "imports"]),
         {:ok, allow} <- decode_imports(rows) do
      {:ok, allow}
    else
      _ -> :unavailable
    end
  end

  defp decode_imports(rows) do
    Enum.reduce_while(rows, {:ok, []}, fn
      %{"name" => name, "params" => params, "results" => results}, {:ok, acc}
      when is_binary(name) and is_list(params) and is_list(results) ->
        {:cont,
         {:ok, [{name, {:fn, Enum.map(params, &type_atom/1), Enum.map(results, &type_atom/1)}} | acc]}}

      _other, _acc ->
        {:halt, :error}
    end)
    |> case do
      {:ok, allow} -> {:ok, Enum.reverse(allow)}
      :error -> :error
    end
  end

  defp type_atom("i32"), do: :i32
  defp type_atom("i64"), do: :i64
  defp type_atom("f32"), do: :f32
  defp type_atom("f64"), do: :f64
  defp type_atom("v128"), do: :v128
  defp type_atom(other), do: {:unknown_type, other}

  defp default_expected(opts) do
    case Keyword.get(opts, :wasm_path) do
      path when is_binary(path) and path != "" ->
        if Path.expand(path) == Path.expand(vendored_path()), do: pinned_sha256(), else: :unpinned

      _ ->
        pinned_sha256()
    end
  end

  defp limit(opts, key, default) do
    [Keyword.get(opts, key), Application.get_env(:graphlaw_ex, key)]
    |> Enum.find(default, &(is_integer(&1) and &1 >= min_limit(key)))
  end

  # Store count limits treat 0 as a real bound; every other limit must be positive.
  @zero_ok [:table_elements, :instances, :tables, :memories]
  defp min_limit(key) when key in @zero_ok, do: 0
  defp min_limit(_key), do: 1

  defp read_manifest(path) do
    case File.read(path) do
      {:ok, raw} ->
        {:ok, raw}

      {:error, reason} ->
        {:error,
         Refusal.new(:invalid_json, "cannot read #{path}: #{:file.format_error(reason)}", %{path: path})}
    end
  end

  defp decode_manifest(raw, path) do
    case Jason.decode(raw) do
      {:ok, %{} = manifest} -> {:ok, manifest}
      _ -> {:error, Refusal.new(:invalid_json, "#{path} is not a JSON object", %{path: path})}
    end
  end

  defp app_path(rel) do
    Application.app_dir(:graphlaw_ex, rel)
  rescue
    ArgumentError -> Path.expand(Path.join("..", Path.join("..", rel)), __DIR__)
  end

  defp present(value) when is_binary(value) and value != "", do: value
  defp present(_value), do: nil

  defp sha256_hex?(sha), do: byte_size(sha) == 64 and String.match?(sha, ~r/\A[0-9a-fA-F]{64}\z/)
end
