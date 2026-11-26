# SPDX-FileCopyrightText: 2026 Sean Chatman <https://github.com/seanchatmangpt/graphlaw/graphs/contributors>
#
# SPDX-License-Identifier: MIT
defmodule GraphlawEx.EngineLoad do
  @moduledoc """
  The load boundary for the GraphLaw WASM engine.

  `admit/2` judges the engine bytes in a fixed order before any instance
  exists, so a foreign module is refused as a typed `GraphlawEx.Refusal` and
  never crashes a caller through `Wasmex.start_link/1`:

    1. digest pin (`:wasm_digest_mismatch`) when a hex pin is expected: a cheap
       sha256 comparison that runs BEFORE any untrusted bytes reach the Wasmtime
       compiler, so a pinned load of a foreign module costs one hash, not a
       compilation. With `expected_sha256: :unpinned` there is no pin to check.
    2. import surface: every import must be a function in the manifest's closed
       `wasi_snapshot_preview1` allowlist (`GraphlawEx.WasmConfig.import_allowlist/0`),
       by module, name and type (`:wasm_import_surface_mismatch`)
    3. required exports `required_exports/0`: a missing export is refused as
       `:wasm_import_surface_mismatch` (the code table is closed; see
       `check_exports/1`)
    4. compile (`wasmex` raises/returns; untyped failure is a crash of `admit/2`,
       which is why the pin and the surface run first)

  Only a fully admitted module is cached (via `GraphlawEx.EngineLoad.SingleFlight`,
  `:persistent_term`, keyed by `{:engine, sha256, fuel?}`, node-local, at most 32
  entries with oldest-first eviction); concurrent cold-start callers for one key
  compile once. `purge_cache/0` drops the cache.

  Telemetry: `[:graphlaw_ex, :engine, :admit]` with metadata `outcome`, `code`,
  `wasm_sha256`; `[:graphlaw_ex, :engine, :compile]` once per actual compile.
  """

  alias GraphlawEx.EngineLoad.Blake3
  alias GraphlawEx.EngineLoad.SingleFlight
  alias GraphlawEx.Refusal
  alias GraphlawEx.WasmConfig

  @import_module "wasi_snapshot_preview1"
  @required_exports ["gl_alloc", "gl_call", "gl_free", "memory"]
  @event [:graphlaw_ex, :engine, :admit]
  @compile_event [:graphlaw_ex, :engine, :compile]

  @typedoc "An admitted engine. `engine`/`module` are handed to the host for instantiation."
  @type admitted :: %{
          wasm_sha256: String.t(),
          imports: %{optional(String.t()) => term()},
          exports: %{optional(String.t()) => term()},
          engine: Wasmex.Engine.t(),
          module: Wasmex.Module.t()
        }

  @doc "Exports the engine must provide."
  @spec required_exports() :: [String.t()]
  def required_exports, do: @required_exports

  @doc "The pinned engine SHA-256 (`GraphlawEx.WasmConfig.pinned_sha256/0`)."
  @spec pinned_sha256() :: String.t() | nil
  def pinned_sha256, do: WasmConfig.pinned_sha256()

  @typedoc "Options for `admit/2`."
  @type opts :: [expected_sha256: String.t() | :unpinned | nil, fuel?: boolean()]

  @doc "Compiles and admits `bytes`; see the module doc for the order of checks."
  @spec admit(binary(), opts()) :: {:ok, admitted()} | {:error, Refusal.t()}
  def admit(bytes, opts \\ [])

  def admit(bytes, opts) when is_binary(bytes) and is_list(opts) do
    digest = :crypto.hash(:sha256, bytes) |> Base.encode16(case: :lower)
    fuel? = Keyword.get(opts, :fuel?, true)
    expected = expected(opts)

    # The pin is judged per caller BEFORE the flight, so a coalesced waiter
    # with a different pin never inherits the leader's admission.
    result =
      with :ok <- check_digest(digest, expected) do
        SingleFlight.fetch_or_compile(cache_key(digest, fuel?), fn ->
          compile_and_surface(digest, bytes, fuel?)
        end)
      end

    emit(digest, result)
    result
  end

  def admit(_bytes, _opts) do
    refusal = Refusal.new(:invalid_json, "engine bytes must be a binary")
    emit(nil, {:error, refusal})
    {:error, refusal}
  end

  @doc """
  Verifies `bytes` against the pinned BLAKE3 (`GraphlawEx.WasmConfig.expected_blake3/1`).
  Any mismatch, including a 1-byte mutation, or a missing pin is
  `{:error, {:REFUSED_ARTIFACT_INTEGRITY, details}}`.
  """
  @spec verify_artifact(binary(), keyword()) ::
          :ok | {:error, {:REFUSED_ARTIFACT_INTEGRITY, map()}}
  def verify_artifact(bytes, opts \\ []) when is_binary(bytes) do
    case WasmConfig.expected_blake3(opts) do
      :unpinned ->
        :ok

      nil ->
        {:error, {:REFUSED_ARTIFACT_INTEGRITY, %{expected: nil, actual: nil, reason: :no_pin}}}

      expected ->
        case Blake3.hash_hex(bytes) do
          ^expected -> :ok
          actual -> {:error, {:REFUSED_ARTIFACT_INTEGRITY, %{expected: expected, actual: actual}}}
        end
    end
  end

  @doc """
  Boot sequence: sha256 pin + surface admission (compiles the `Wasmex.Module`
  once, cached by `admit/2`). Returns the admitted engine. The former boot-time
  `:atomics` gate and `boot_module/0` were dead (no reader) and are removed.

  The BLAKE3 artifact check (`verify_artifact/2`) is deliberately NOT part of
  boot: it is a publish/CI-time gate (`mix graphlaw_ex.verify`), so downstream
  consumers never pay for it at startup. Runtime integrity is the sha256 pin.
  """
  @spec boot(binary(), keyword()) ::
          {:ok, admitted()} | {:error, {:REFUSED_ARTIFACT_INTEGRITY, map()} | Refusal.t()}
  def boot(bytes, opts \\ []) when is_binary(bytes) do
    admit(bytes, Keyword.take(opts, [:expected_sha256, :fuel?]))
  end

  @doc "Drops every cached compiled module."
  @spec purge_cache() :: :ok
  def purge_cache, do: SingleFlight.purge()

  @doc "Whether an admitted module for `{sha256, fuel?}` is cached on this node."
  @spec cached?(String.t(), boolean()) :: boolean()
  def cached?(digest, fuel? \\ true),
    do: SingleFlight.cached(cache_key(digest, fuel?)) != :error

  defp cache_key(digest, fuel?), do: {:engine, digest, fuel?}

  defp expected(opts) do
    case Keyword.fetch(opts, :expected_sha256) do
      {:ok, value} -> value
      :error -> WasmConfig.expected_sha256()
    end
  end

  # A hex pin is judged first: hashing is cheap, compiling untrusted bytes is
  # not, and a foreign module never needs a diagnosis when the caller already
  # knows which bytes it wants. With `:unpinned` (the caller chose the bytes)
  # the surface diagnosis is the only check that remains. Only bytes that pass
  # every check are cached.
  defp compile_and_surface(digest, bytes, fuel?) do
    :telemetry.execute(@compile_event, %{}, %{wasm_sha256: digest, fuel?: fuel?})

    with {:ok, engine, module} <- compile(bytes, fuel?),
         imports = Wasmex.Module.imports(module),
         exports = Wasmex.Module.exports(module),
         :ok <- check_imports(imports),
         :ok <- check_exports(exports) do
      entry = %{
        wasm_sha256: digest,
        imports: imports,
        exports: exports,
        engine: engine,
        module: module
      }

      # The single flight publishes the entry (bounded cache); no put here.
      {:ok, entry}
    end
  end

  defp compile(bytes, fuel?) do
    config =
      if fuel?,
        do: Wasmex.EngineConfig.consume_fuel(%Wasmex.EngineConfig{}, true),
        else: %Wasmex.EngineConfig{}

    with {:ok, engine} <- engine_new(config),
         {:ok, store} <- store_new(engine),
         {:ok, module} <- module_compile(store, bytes) do
      {:ok, engine, module}
    end
  end

  defp engine_new(config) do
    case Wasmex.Engine.new(config) do
      {:ok, engine} -> {:ok, engine}
      {:error, reason} -> {:error, invalid(reason)}
    end
  end

  defp store_new(engine) do
    case Wasmex.Store.new(nil, engine) do
      {:ok, store} -> {:ok, store}
      {:error, reason} -> {:error, invalid(reason)}
    end
  end

  defp module_compile(store, bytes) do
    case Wasmex.Module.compile(store, bytes) do
      {:ok, module} -> {:ok, module}
      {:error, reason} -> {:error, invalid(reason)}
    end
  rescue
    error -> {:error, invalid(error)}
  end

  defp invalid(reason) do
    Refusal.new(:invalid_json, "engine bytes did not compile as a WebAssembly module", %{
      reason: inspect(reason, limit: 5, printable_limit: 200),
      type: reason_type(reason)
    })
  end

  defp reason_type(%module{}), do: inspect(module)
  defp reason_type(reason) when is_binary(reason), do: "binary"

  defp check_imports(imports) do
    case WasmConfig.import_allowlist() do
      {:ok, allow} -> judge_imports(imports, MapSet.new(allow))
      :unavailable -> {:error, allowlist_unavailable()}
    end
  end

  defp judge_imports(imports, allow) do
    unexpected_modules =
      imports |> Map.keys() |> Enum.reject(&(&1 == @import_module)) |> Enum.sort()

    unexpected_functions =
      for {module, functions} <- imports,
          module == @import_module,
          {name, type} <- functions,
          not MapSet.member?(allow, {name, type}) do
        "#{module}.#{name}"
      end
      |> Enum.sort()

    if unexpected_modules == [] and unexpected_functions == [] do
      :ok
    else
      offenders = unexpected_modules ++ unexpected_functions

      {:error,
       Refusal.new(
         :wasm_import_surface_mismatch,
         "engine imports outside the allowlist: #{Enum.join(offenders, ", ")}",
         %{
           unexpected: unexpected_modules,
           unexpected_functions: unexpected_functions,
           allowed: [@import_module],
           allowed_functions: allow |> Enum.map(&elem(&1, 0)) |> Enum.sort()
         }
       )}
    end
  end

  defp allowlist_unavailable do
    Refusal.new(
      :wasm_import_surface_mismatch,
      "no import allowlist is available (MANIFEST.json host_abi.imports unusable); refusing every engine",
      %{
        unexpected: [],
        unexpected_functions: [],
        allowed: [@import_module],
        reason: "allowlist_unavailable"
      }
    )
  end

  # The module-shape codes are closed, and export absence is an identity break
  # with the pinned release: it uses :wasm_import_surface_mismatch so the code
  # table stays closed (no per-symptom code explosion).
  defp check_exports(exports) do
    missing = Enum.reject(@required_exports, &Map.has_key?(exports, &1))

    if missing == [] do
      :ok
    else
      {:error,
       Refusal.new(
         :wasm_import_surface_mismatch,
         "engine lacks required exports: #{Enum.join(missing, ", ")}",
         %{
           missing: missing,
           required: @required_exports
         }
       )}
    end
  end

  defp check_digest(_digest, :unpinned), do: :ok
  defp check_digest(digest, digest) when is_binary(digest), do: :ok

  # No pin resolved at all (nil): distinct from a mismatch. The refusal code
  # table is closed, so the code stays :wasm_digest_mismatch and the typed
  # `reason: :no_pin` detail (with no `:expected` key) tells the cases apart.
  defp check_digest(digest, nil) do
    {:error,
     Refusal.new(
       :wasm_digest_mismatch,
       "no sha256 pin is configured; pass a hex pin or expected_sha256: :unpinned",
       %{reason: :no_pin, actual: digest}
     )}
  end

  defp check_digest(digest, expected) do
    {:error,
     Refusal.new(:wasm_digest_mismatch, "engine sha256 does not match the pin", %{
       expected: expected,
       actual: digest
     })}
  end

  defp emit(digest, result) do
    {outcome, code} =
      case result do
        {:ok, _} -> {:admitted, nil}
        {:error, %Refusal{code: code}} -> {:refused, code}
      end

    :telemetry.execute(@event, %{}, %{outcome: outcome, code: code, wasm_sha256: digest})
  end
end
