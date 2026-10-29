defmodule GraphlawEx do
  @moduledoc """
  Deterministic turnstile facade over the GraphLaw wasm kernel: the BEAM holds
  only the wasm byte-slice of the kernel; all law admission happens *inside*
  the module. Nothing on the Elixir side interprets RDF, SHACL, SQL, or
  receipts.

  This module is a thin facade. The machinery lives in:

    * `GraphlawEx.Host` — one serialized wasm instance
      (`gl_alloc` -> write -> `gl_call` -> read -> `gl_free`), recycle-on-fault,
      two-layer `:saturated` shedding
    * `GraphlawEx.Pool` — N hosts behind one name, shortest-mailbox routing
    * `GraphlawEx.EngineLoad` — digest pin -> import allowlist -> required
      exports -> compile, cached on `:persistent_term`
    * `GraphlawEx.WasmConfig` — path resolution and the manifest pin
    * `GraphlawEx.ABI` — the pure packet-4 / packed-result codec
    * `GraphlawEx.Refusal` — the closed typed-refusal table

  A packet-4 frame is `<<length::32-big-endian, payload::binary>>`; the payload
  alone (its byte length is the declared length) is what lands in linear
  memory.
  """

  alias GraphlawEx.{ABI, Host, Refusal, Telemetry, WasmConfig}

  @typedoc "Typed start failure, returned before any process is spawned."
  @type start_error ::
          {:wasm_load_failed, String.t(), :enoent}
          | {:wasm_digest_mismatch, %{expected: String.t() | nil, actual: String.t(), path: String.t()}}

  @doc """
  Starts the default host under the name `GraphlawEx`. Options are the
  `GraphlawEx.Host` options (`:wasm_path`, `:expected_sha256`, `:fuel`,
  `:max_queue`, ...).

  Fail-closed preconditions run BEFORE the spawn, so a missing artifact is
  `{:error, {:wasm_load_failed, path, :enoent}}` and bytes that do not match
  the manifest pin are `{:error, {:wasm_digest_mismatch, details}}` — returned
  errors, never linked-process exits.
  """
  @spec start_link(keyword()) :: GenServer.on_start() | {:error, start_error()}
  def start_link(opts \\ []) do
    path = WasmConfig.wasm_path(opts)

    with :ok <- require_regular(path),
         :ok <- require_digest(path, opts) do
      Host.start_link(Keyword.put_new(opts, :name, __MODULE__))
    end
  end

  @doc "Child spec so `GraphlawEx` (or the Pool) can sit in a supervision tree."
  def child_spec(opts) do
    %{id: __MODULE__, start: {__MODULE__, :start_link, [opts]}, type: :worker}
  end

  @doc """
  Admits an order through the law kernel inside the wasm module.

  `payload` is a UTF-8 JSON request document (`{"op": ...}`). It is framed as a
  packet-4 frame — `GraphlawEx.ABI.frame/1` refuses non-UTF-8
  (`:invalid_encoding`) and over-cap (`:resource_limit`) payloads BEFORE any
  memory is allocated in the engine, closing the old hang path — and handed to
  the host's serialized ABI transaction. The reply is `{:ok, response_map}`
  with the decoded JSON response, or `{:error, %GraphlawEx.Refusal{}}`. An
  engine-side refusal (`"ok": false`) is still `{:ok, response}` at the
  transport layer; read `response["ok"]` to classify it.
  """
  @spec admit(GenServer.server(), binary(), keyword()) ::
          {:ok, map()} | {:error, Refusal.t()}
  def admit(server \\ __MODULE__, payload, opts \\ []) when is_binary(payload) do
    # GL28-1 leg (b): opts passthrough — the caller may pin the source
    # digest of the law content this span executes (engine-minted
    # `blake3:` LawState.id). Arity-2 callers keep their exact behavior:
    # absent opts -> nil digest, metadata shape unchanged.
    telemetry_opts =
      [server: server]
      |> Keyword.put(:source_digest, Keyword.get(opts, :source_digest))

    Telemetry.admit("admit", telemetry_opts, fn ->
      with {:ok, _framed} <- ABI.frame(payload) do
        Host.request_raw(server, payload)
      end
    end)
  end

  @doc "Stops the default host."
  @spec stop(GenServer.server()) :: :ok
  def stop(server \\ __MODULE__) do
    GenServer.stop(server, :normal)
  end

  # ---------------------------------------------------------------------
  # Typed capability verbs (re-exported from GraphlawEx.Capability.API)
  # ---------------------------------------------------------------------

  # GL28-11: the 14 typed capability verbs surface on the facade. Each
  # `op(args, opts)` delegates to `GraphlawEx.Capability.API.op/2` and each
  # `op!(args, opts)` to the raising twin; `{:error, %GraphlawEx.Capability.Refusal{}}`
  # on refusal, or `GraphlawEx.Error.Refused` raised.
  # Results are observations, never authority.

  @doc "See `GraphlawEx.Capability.API.capabilities/2`."
  defdelegate capabilities(args, opts \\ []), to: GraphlawEx.Capability.API

  @doc "See `GraphlawEx.Capability.API.capabilities!/2`."
  defdelegate capabilities!(args, opts \\ []), to: GraphlawEx.Capability.API

  @doc "See `GraphlawEx.Capability.API.sniff/2`."
  defdelegate sniff(args, opts \\ []), to: GraphlawEx.Capability.API

  @doc "See `GraphlawEx.Capability.API.sniff!/2`."
  defdelegate sniff!(args, opts \\ []), to: GraphlawEx.Capability.API

  @doc "See `GraphlawEx.Capability.API.parse/2`."
  defdelegate parse(args, opts \\ []), to: GraphlawEx.Capability.API

  @doc "See `GraphlawEx.Capability.API.parse!/2`."
  defdelegate parse!(args, opts \\ []), to: GraphlawEx.Capability.API

  @doc "See `GraphlawEx.Capability.API.convert/2`."
  defdelegate convert(args, opts \\ []), to: GraphlawEx.Capability.API

  @doc "See `GraphlawEx.Capability.API.convert!/2`."
  defdelegate convert!(args, opts \\ []), to: GraphlawEx.Capability.API

  @doc "See `GraphlawEx.Capability.API.canonical/2`."
  defdelegate canonical(args, opts \\ []), to: GraphlawEx.Capability.API

  @doc "See `GraphlawEx.Capability.API.canonical!/2`."
  defdelegate canonical!(args, opts \\ []), to: GraphlawEx.Capability.API

  @doc "See `GraphlawEx.Capability.API.sparql/2`."
  defdelegate sparql(args, opts \\ []), to: GraphlawEx.Capability.API

  @doc "See `GraphlawEx.Capability.API.sparql!/2`."
  defdelegate sparql!(args, opts \\ []), to: GraphlawEx.Capability.API

  @doc "See `GraphlawEx.Capability.API.shacl/2`."
  defdelegate shacl(args, opts \\ []), to: GraphlawEx.Capability.API

  @doc "See `GraphlawEx.Capability.API.shacl!/2`."
  defdelegate shacl!(args, opts \\ []), to: GraphlawEx.Capability.API

  @doc "See `GraphlawEx.Capability.API.shex/2`."
  defdelegate shex(args, opts \\ []), to: GraphlawEx.Capability.API

  @doc "See `GraphlawEx.Capability.API.shex!/2`."
  defdelegate shex!(args, opts \\ []), to: GraphlawEx.Capability.API

  @doc "See `GraphlawEx.Capability.API.n3/2`."
  defdelegate n3(args, opts \\ []), to: GraphlawEx.Capability.API

  @doc "See `GraphlawEx.Capability.API.n3!/2`."
  defdelegate n3!(args, opts \\ []), to: GraphlawEx.Capability.API

  @doc "See `GraphlawEx.Capability.API.entail/2`."
  defdelegate entail(args, opts \\ []), to: GraphlawEx.Capability.API

  @doc "See `GraphlawEx.Capability.API.entail!/2`."
  defdelegate entail!(args, opts \\ []), to: GraphlawEx.Capability.API

  @doc "See `GraphlawEx.Capability.API.datalog/2`."
  defdelegate datalog(args, opts \\ []), to: GraphlawEx.Capability.API

  @doc "See `GraphlawEx.Capability.API.datalog!/2`."
  defdelegate datalog!(args, opts \\ []), to: GraphlawEx.Capability.API

  @doc "See `GraphlawEx.Capability.API.hooks/2`."
  defdelegate hooks(args, opts \\ []), to: GraphlawEx.Capability.API

  @doc "See `GraphlawEx.Capability.API.hooks!/2`."
  defdelegate hooks!(args, opts \\ []), to: GraphlawEx.Capability.API

  @doc "See `GraphlawEx.Capability.API.law/2`."
  defdelegate law(args, opts \\ []), to: GraphlawEx.Capability.API

  @doc "See `GraphlawEx.Capability.API.law!/2`."
  defdelegate law!(args, opts \\ []), to: GraphlawEx.Capability.API

  @doc "See `GraphlawEx.Capability.API.policy/2`."
  defdelegate policy(args, opts \\ []), to: GraphlawEx.Capability.API

  @doc "See `GraphlawEx.Capability.API.policy!/2`."
  defdelegate policy!(args, opts \\ []), to: GraphlawEx.Capability.API

  # ---------------------------------------------------------------------
  # Fail-closed preconditions
  # ---------------------------------------------------------------------

  defp require_regular(path) do
    if File.regular?(path), do: :ok, else: {:error, {:wasm_load_failed, path, :enoent}}
  end

  # The pin is judged before spawn: hashing is cheap and a mismatched artifact
  # must never instantiate. `:unpinned` opts out; a `nil` pin (manifest
  # unusable) is fail-closed when the path is the vendored one, and skipped for
  # an explicit opt-in path.
  defp require_digest(path, opts) do
    case WasmConfig.expected_sha256(opts) do
      :unpinned ->
        :ok

      expected when is_binary(expected) ->
        actual = :crypto.hash(:sha256, File.read!(path)) |> Base.encode16(case: :lower)

        if actual == expected,
          do: :ok,
          else: {:error, {:wasm_digest_mismatch, %{expected: expected, actual: actual, path: path}}}

      nil ->
        {:error, {:wasm_digest_mismatch, %{expected: nil, actual: nil, path: path}}}
    end
  end
end
