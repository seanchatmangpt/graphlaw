# SPDX-FileCopyrightText: 2026 Sean Chatman <https://github.com/seanchatmangpt/graphlaw/graphs/contributors>
#
# SPDX-License-Identifier: MIT
# projection target: ontologies/graphlaw-elixir.ttl; hand-written until projected
defmodule GraphlawEx.WasmConfig do
  @moduledoc """
  Resolves where the GraphLaw engine lives, which digest it must have, and the
  resource limits the host runs it under.

  ## Path resolution

  In order: `opts[:wasm_path]`, the `GRAPHLAW_WASM_PATH` environment variable,
  `config :graphlaw_ex, :wasm_path`, the vendored `priv/graphlaw/graphlaw.wasm`
  when it exists, then the user cache
  `<cache root>/<blake3>/graphlaw.wasm` when it exists (no network), else the
  vendored path. Cache and priv paths stay pinned.

  The cache root is `GRAPHLAW_CACHE_DIR`, else `$XDG_CACHE_HOME/graphlaw_ex`,
  else `~/.cache/graphlaw_ex`; see `cache_root/0`. If none resolves (no `HOME`),
  `cache_path_result/0` returns a typed `:no_cache_dir` error.

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

  ## Configuration keys

  Every key under `config :graphlaw_ex` (check them with `validate/0` or
  `validate!/0`). Keys, minimums and defaults come from the single table in
  `GraphlawEx.Config.Keys`:

    * `:auto_start` (boolean, default `false`) - start the host pool with the
      application (`GraphlawEx.Application`).
    * `:wasm_path` (non-empty string) - engine path; the pin still applies.
    * `:fuel` (pos integer) - CPU budget per request; default
      `timeout_ms * fuel_per_ms`.
    * `:timeout_ms` (pos integer, default 5000) - per-request deadline.
    * `:fuel_per_ms` (pos integer, default 1_000_000)
    * `:instantiate_fuel` (pos integer, default 1_000_000_000)
    * `:memory_limit_bytes` (pos integer, default 268_435_456)
    * `:recycle_bytes` (pos integer, default 134_217_728)
    * `:max_queue` (pos integer, default 64)
    * `:max_response_bytes` (pos integer, default 33_554_432)
    * `:table_elements`, `:instances`, `:tables`, `:memories` (integer >= 0;
      defaults 100_000, 10, 10, 4)

  Pool size is not an application-config key: pass `size: n` to
  `GraphlawEx.Pool` (default `System.schedulers_online/0`).

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

  # Derived from GraphlawEx.Config.Keys (the one key table); `:fuel` is derived in limits/1.
  @defaults GraphlawEx.Config.Keys.defaults()

  @typedoc "Resource limits applied by `GraphlawEx.Host`."
  @type limits :: %{
          fuel: pos_integer(),
          fuel_per_ms: pos_integer(),
          instantiate_fuel: pos_integer(),
          memory_limit_bytes: pos_integer(),
          recycle_bytes: pos_integer(),
          max_queue: pos_integer(),
          max_response_bytes: pos_integer(),
          table_elements: non_neg_integer(),
          instances: non_neg_integer(),
          tables: non_neg_integer(),
          memories: non_neg_integer(),
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
          table_elements: non_neg_integer(),
          instances: non_neg_integer(),
          tables: non_neg_integer(),
          memories: non_neg_integer()
        ]

  @typedoc "One allowed WASI import: `{name, {:fn, params, results}}` with wasm value-type atoms."
  @type import_allow :: {String.t(), {:fn, [atom()], [atom()]}}

  @doc "Every key accepted under `config :graphlaw_ex` (see `GraphlawEx.Config`)."
  @spec config_keys() :: [atom()]
  defdelegate config_keys, to: GraphlawEx.Config, as: :keys

  @doc """
  Validates `config :graphlaw_ex` (or the given keyword list); delegates to
  `GraphlawEx.Config.validate/1`.

      iex> GraphlawEx.WasmConfig.validate(timeout_ms: 100, auto_start: true)
      :ok
      iex> {:error, [msg]} = GraphlawEx.WasmConfig.validate(timeout: 100)
      iex> msg =~ "did you mean :timeout_ms"
      true
  """
  @spec validate(keyword()) :: :ok | {:error, [String.t()]}
  def validate(config \\ Application.get_all_env(:graphlaw_ex)),
    do: GraphlawEx.Config.validate(config)

  @doc "Like `validate/1` but raises `ArgumentError`; delegates to `GraphlawEx.Config.validate!/1`."
  @spec validate!(keyword()) :: :ok
  def validate!(config \\ Application.get_all_env(:graphlaw_ex)),
    do: GraphlawEx.Config.validate!(config)

  @doc "The vendored engine path inside this application's `priv` directory."
  @spec vendored_path() :: String.t()
  def vendored_path, do: app_path(@wasm_rel)

  @doc "Resolves the engine path; see the module doc for the order."
  @spec wasm_path(opts()) :: String.t()
  def wasm_path(opts \\ []) do
    present(Keyword.get(opts, :wasm_path)) ||
      present(System.get_env(@env)) ||
      present(Application.get_env(:graphlaw_ex, :wasm_path)) ||
      staged_path()
  end

  @doc """
  The user-cache engine path `~/.cache/graphlaw_ex/<blake3>/graphlaw.wasm` for the
  pinned BLAKE3, or `nil` when the pin is unavailable. Pure path computation.
  """
  @spec cache_path() :: String.t() | nil
  def cache_path do
    case cache_path_result() do
      {:ok, path} -> path
      {:error, _} -> nil
    end
  end

  @typedoc "Why the cache path could not be computed."
  @type cache_error :: {:no_pin, String.t()} | {:no_cache_dir, String.t()}

  @doc """
  Like `cache_path/0` but typed: `{:error, {:no_pin, message}}` when the BLAKE3
  pin is unavailable, `{:error, {:no_cache_dir, message}}` when no cache root
  can be resolved (none of `GRAPHLAW_CACHE_DIR`, `XDG_CACHE_HOME`, `HOME` set).
  """
  @spec cache_path_result() :: {:ok, String.t()} | {:error, cache_error()}
  def cache_path_result do
    with {:ok, root} <- cache_root(),
         hex when is_binary(hex) <- pinned_blake3() || {:no_pin, nil} do
      {:ok, Path.join([root, hex, "graphlaw.wasm"])}
    else
      {:error, _} = error -> error
      {:no_pin, _} -> {:error, {:no_pin, "no BLAKE3 pin in MANIFEST.json or sidecar"}}
    end
  end

  @doc """
  The cache root directory: `GRAPHLAW_CACHE_DIR`, else `$XDG_CACHE_HOME/graphlaw_ex`,
  else `~/.cache/graphlaw_ex`. A typed error when none is resolvable (e.g. `HOME`
  unset), never a silent `nil`.
  """
  @spec cache_root() :: {:ok, String.t()} | {:error, {:no_cache_dir, String.t()}}
  def cache_root do
    cond do
      dir = present(System.get_env("GRAPHLAW_CACHE_DIR")) ->
        {:ok, dir}

      xdg = present(System.get_env("XDG_CACHE_HOME")) ->
        {:ok, Path.join(xdg, "graphlaw_ex")}

      home = present(System.user_home()) ->
        {:ok, Path.join([home, ".cache", "graphlaw_ex"])}

      true ->
        {:error,
         {:no_cache_dir,
          "cannot resolve the wasm cache directory: set GRAPHLAW_CACHE_DIR, XDG_CACHE_HOME or HOME"}}
    end
  end

  # Vendored priv path when present, else the user cache when it exists, else
  # the vendored path (so loaders report the canonical missing location).
  defp staged_path do
    vendored = vendored_path()
    cached = cache_path()

    cond do
      File.exists?(vendored) -> vendored
      is_binary(cached) and File.exists?(cached) -> cached
      true -> vendored
    end
  end

  @doc "Reads and decodes `priv/graphlaw/MANIFEST.json`."
  @spec manifest() :: {:ok, map()} | {:error, Refusal.t()}
  def manifest do
    with {:ok, path} <- app_path_result(@manifest_rel) do
      key = {__MODULE__, :manifest, path}

      case :persistent_term.get(key, nil) do
        %{} = cached ->
          {:ok, cached}

        nil ->
          with {:ok, raw} <- read_manifest(path),
               {:ok, decoded} <- decode_manifest(raw, path) do
            :persistent_term.put(key, decoded)
            {:ok, decoded}
          end
      end
    end
  end

  @doc """
  Drops the cached decoded manifest so the next `manifest/0` re-reads disk.
  Only successful decodes are cached; tests that rewrite `MANIFEST.json` call this.
  """
  @spec invalidate_manifest_cache() :: :ok
  def invalidate_manifest_cache do
    for {{__MODULE__, :manifest, _} = key, _} <- :persistent_term.get() do
      :persistent_term.erase(key)
    end

    :ok
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
  The pinned engine BLAKE3: `artifact.blake3` in the manifest (the authority),
  cross-checked against the `<wasm>.blake3` sidecar when one exists. `nil` when
  the manifest pin is unusable or the sidecar disagrees (fail closed); see
  `pinned_blake3_result/2` for the typed reason.
  """
  @spec pinned_blake3() :: String.t() | nil
  def pinned_blake3 do
    case pinned_blake3_result() do
      {:ok, hex} -> hex
      {:error, _} -> nil
    end
  end

  @doc """
  Typed form of `pinned_blake3/0`. The manifest `artifact.blake3` is the
  authority; a vendored sidecar is only a cross-check. A sidecar that disagrees
  with the manifest is `{:error, %Refusal{code: :invalid_json}}`-class refusal
  (`:blake3_pin_mismatch` detail), never a silent override. `pinned_blake3/0`
  maps every error to `nil` (fail closed), so `verify` and `doctor` fail.

  Paths default to the application's priv manifest and vendored engine.
  """
  @spec pinned_blake3_result(String.t() | nil, String.t() | nil) ::
          {:ok, String.t()} | {:error, Refusal.t() | :no_pin}
  def pinned_blake3_result(manifest_path \\ nil, wasm_path \\ nil) do
    with {:ok, manifest_path} <- resolve_manifest_path(manifest_path),
         wasm_path = wasm_path || vendored_path(),
         {:ok, raw} <- read_manifest(manifest_path),
         {:ok, manifest} <- decode_manifest(raw, manifest_path),
         hex when is_binary(hex) <- get_in(manifest, ["artifact", "blake3"]),
         true <- sha256_hex?(hex) do
      manifest_hex = String.downcase(hex)

      case sidecar(wasm_path) do
        :none ->
          {:ok, manifest_hex}

        {:ok, ^manifest_hex} ->
          {:ok, manifest_hex}

        {:ok, other} ->
          {:error,
           Refusal.new(
             :invalid_json,
             "blake3 sidecar #{wasm_path}.blake3 (#{other}) disagrees with MANIFEST " <>
               "artifact.blake3 (#{manifest_hex}); the manifest is the authority",
             %{path: wasm_path <> ".blake3", detail: :blake3_pin_mismatch}
           )}
      end
    else
      {:error, _} = error -> error
      _ -> {:error, :no_pin}
    end
  end

  @doc """
  The BLAKE3 a load must match: a hex string, `:unpinned` (explicit foreign
  `:wasm_path` without a sidecar), or `nil` (pin unavailable, fail closed).
  `opts[:expected_blake3]` wins when given.
  """
  @spec expected_blake3(keyword()) :: String.t() | :unpinned | nil
  def expected_blake3(opts \\ []) do
    case Keyword.fetch(opts, :expected_blake3) do
      {:ok, value} ->
        value

      :error ->
        case Keyword.get(opts, :wasm_path) do
          path when is_binary(path) and path != "" ->
            if Path.expand(path) == Path.expand(vendored_path()) do
              pinned_blake3()
            else
              case sidecar(path) do
                {:ok, hex} -> hex
                :none -> :unpinned
              end
            end

          _ ->
            pinned_blake3()
        end
    end
  end

  defp resolve_manifest_path(nil), do: app_path_result(@manifest_rel)
  defp resolve_manifest_path(path), do: {:ok, path}

  defp sidecar(wasm_path) do
    with {:ok, raw} <- File.read(wasm_path <> ".blake3"),
         [token | _] <- String.split(raw),
         true <- sha256_hex?(token) do
      {:ok, String.downcase(token)}
    else
      _ -> :none
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
         {:ok,
          [{name, {:fn, Enum.map(params, &type_atom/1), Enum.map(results, &type_atom/1)}} | acc]}}

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

  # Runtime opts are strict (invalid raises); the app-env value is validated by
  # GraphlawEx.Config at boot and keeps the lenient fallback.
  defp limit(opts, key, default) do
    case Keyword.get(opts, key) do
      nil ->
        env = Application.get_env(:graphlaw_ex, key)
        if is_integer(env) and env >= min_limit(key), do: env, else: default

      value when is_integer(value) ->
        if value >= min_limit(key), do: value, else: bad_limit!(key, value)

      value ->
        bad_limit!(key, value)
    end
  end

  @spec bad_limit!(atom(), term()) :: no_return()
  defp bad_limit!(key, value) do
    raise ArgumentError,
          "invalid option #{inspect(key)}: must be an integer >= #{min_limit(key)}, " <>
            "got #{inspect(value)}"
  end

  # Store count limits treat 0 as a real bound; every other limit must be positive.
  defp min_limit(key), do: GraphlawEx.Config.min_limit(key)

  defp read_manifest(path) do
    case File.read(path) do
      {:ok, raw} ->
        {:ok, raw}

      {:error, reason} ->
        {:error,
         Refusal.new(:invalid_json, "cannot read #{path}: #{:file.format_error(reason)}", %{
           path: path
         })}
    end
  end

  defp decode_manifest(raw, path) do
    case Jason.decode(raw) do
      {:ok, %{} = manifest} -> {:ok, manifest}
      _ -> {:error, Refusal.new(:invalid_json, "#{path} is not a JSON object", %{path: path})}
    end
  end

  # No compile-time __DIR__ fallback: a path baked at build time would point at a
  # build machine's tree. When the application is not loaded, say so.
  defp app_path_result(rel) do
    {:ok, Application.app_dir(:graphlaw_ex, rel)}
  rescue
    ArgumentError ->
      {:error,
       Refusal.new(
         :invalid_json,
         "application :graphlaw_ex is not loaded; cannot resolve #{rel} " <>
           "(start or load the application first)",
         %{path: rel, detail: :app_not_loaded}
       )}
  end

  defp app_path(rel) do
    case app_path_result(rel) do
      {:ok, path} -> path
      {:error, %Refusal{message: message}} -> raise ArgumentError, message
    end
  end

  defp present(value) when is_binary(value) and value != "", do: value
  defp present(_value), do: nil

  defp sha256_hex?(sha), do: byte_size(sha) == 64 and String.match?(sha, ~r/\A[0-9a-fA-F]{64}\z/)
end
