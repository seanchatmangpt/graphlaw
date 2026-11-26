# SPDX-FileCopyrightText: 2026 Sean Chatman <https://github.com/seanchatmangpt/graphlaw/graphs/contributors>
#
# SPDX-License-Identifier: MIT
defmodule GraphlawEx do
  @moduledoc """
  Deterministic turnstile facade over the GraphLaw wasm kernel: the BEAM holds
  only the wasm byte-slice of the kernel; all law admission happens *inside*
  the module. Nothing on the Elixir side interprets RDF, SHACL, SQL, or
  receipts.

  ## Quickstart

  1. Add `{:graphlaw_ex, "~> 26.11"}` to `deps`.
  2. Stage the digest-pinned wasm artifact (it is not shipped in the Hex package):
     `mix graphlaw_ex.fetch` (or set `GRAPHLAW_WASM_PATH`). Check the install
     with `mix graphlaw_ex.doctor`.
  3. Opt in to the boot-time pool in `config/config.exs`:
     `config :graphlaw_ex, auto_start: true`. Without it the application starts
     nothing; put `GraphlawEx.Pool` or `{GraphlawEx, opts}` in your own tree.

      nquads = ~s(<http://example.org/s1> <http://example.org/p> "v1" .\\n)
      query = "SELECT ?s ?o WHERE { ?s <http://example.org/p> ?o }"

      case GraphlawEx.sparql(data: %{"text" => nquads}, query: query) do
        {:ok, result} -> result.rows
        {:error, refusal} -> {:refused, refusal.code}
      end

  Every typed verb (`parse`, `shacl`, `sparql`, ...) returns `{:ok, result}` or
  `{:error, refusal}`; the `!` twin raises `GraphlawEx.Error` instead.
  Inspect the build with `GraphlawEx.metadata/0`, and watch admissions through
  `GraphlawEx.Telemetry` events.

  To run your own named host (custom wasm path, fuel, queue bound), put
  `{GraphlawEx, opts}` in your supervision tree; `start_link/1` returns
  `{:error, {:wasm_load_failed, path, :enoent}}` or
  `{:error, {:wasm_digest_mismatch, details}}` before spawning anything when
  the artifact is missing or off-pin. `available?/1` reports whether a host
  has a live, admitted engine.

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

  alias GraphlawEx.Capability.Refusal, as: CapRefusal
  alias GraphlawEx.{ABI, Host, Refusal, Telemetry, WasmConfig}

  @typedoc "Capability request arguments: a keyword list or a map with atom or string keys."
  @type args :: GraphlawEx.Capability.args()

  @typedoc "Typed start failure, returned before any process is spawned."
  @type start_error ::
          {:wasm_load_failed, String.t(), :enoent}
          | {:wasm_unreadable, %{path: String.t(), reason: :file.posix()}}
          | {:wasm_digest_mismatch,
             %{expected: String.t() | nil, actual: String.t() | nil, path: String.t()}}

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

  @doc """
  Library version, read from the compiled application spec (the Mix project
  version). Returns `"unknown"` only when the application is not loaded.
  """
  @spec version() :: String.t()
  def version do
    case Application.spec(:graphlaw_ex, :vsn) do
      vsn when is_list(vsn) -> List.to_string(vsn)
      _ -> "unknown"
    end
  end

  @doc """
  The pinned engine artifact digest (`artifact.sha256` in the vendored
  `MANIFEST.json`), or `nil` when the manifest is unusable.
  """
  @spec artifact_sha256() :: String.t() | nil
  def artifact_sha256, do: WasmConfig.pinned_sha256()

  @doc """
  Version and artifact metadata: `%{version, abi_version, artifact_sha256,
  wasm_path}`. Observation only; never authority.
  """
  @spec metadata() :: %{
          version: String.t(),
          abi_version: pos_integer(),
          artifact_sha256: String.t() | nil,
          wasm_path: String.t()
        }
  def metadata do
    %{
      version: version(),
      abi_version: ABI.version(),
      artifact_sha256: artifact_sha256(),
      wasm_path: WasmConfig.vendored_path()
    }
  end

  @doc "Whether the host registered under `server` has a live, admitted engine."
  @spec available?(GenServer.server()) :: boolean()
  defdelegate available?(server \\ __MODULE__), to: Host

  @doc "Child spec so `GraphlawEx` (or the Pool) can sit in a supervision tree."
  @spec child_spec(keyword()) :: Supervisor.child_spec()
  def child_spec(opts) do
    %{
      id: Keyword.get(opts, :name, __MODULE__),
      start: {__MODULE__, :start_link, [opts]},
      type: :worker
    }
  end

  @doc """
  Admits an order through the law kernel inside the wasm module.

  `payload` is a UTF-8 JSON request document (`{"op": ...}`). It is checked by
  `GraphlawEx.ABI.recognize_request/1`, which refuses non-UTF-8
  (`:invalid_encoding`) and over-cap (`:resource_limit`) payloads BEFORE any
  memory is allocated in the engine, closing the old hang path, and is then
  handed to the host's serialized ABI transaction (no framed copy is built
  here). The reply is `{:ok, response_map}` with the decoded JSON response, or
  `{:error, %GraphlawEx.Refusal{}}`. An engine-side refusal (`"ok": false`) is
  still `{:ok, response}` at the transport layer; read `response["ok"]` to
  classify it.

  Options: `:server` (host name or pid, default `GraphlawEx`) and
  `:source_digest` (the engine-minted `blake3:` LawState.id of the law content
  this span executes; telemetry metadata only).

  The deprecated 3-arity form `admit(server, payload, opts)` (binary payload in
  the second position) is kept as a shim; it is unambiguous because the first
  argument of the 2-arity form is always the binary payload.
  """
  @spec admit(binary(), keyword()) :: {:ok, map()} | {:error, Refusal.t()}
  def admit(payload, opts \\ []) when is_binary(payload) and is_list(opts) do
    server = Keyword.get(opts, :server, __MODULE__)

    telemetry_opts = [server: server, source_digest: Keyword.get(opts, :source_digest)]

    Telemetry.admit("admit", telemetry_opts, fn ->
      with {:ok, _validated} <- ABI.recognize_request(payload) do
        Host.request_raw(server, payload)
      end
    end)
  end

  @doc false
  @deprecated "Use admit(payload, server: server) instead"
  @spec admit(GenServer.server(), binary(), keyword()) ::
          {:ok, map()} | {:error, Refusal.t()}
  def admit(server, payload, opts) when is_binary(payload) and is_list(opts) do
    admit(payload, Keyword.put(opts, :server, server))
  end

  @doc "Stops the default host."
  @spec stop(GenServer.server()) :: :ok
  def stop(server \\ __MODULE__) do
    GenServer.stop(server, :normal)
  end

  # ---------------------------------------------------------------------
  # Typed capability verbs (re-exported from GraphlawEx.Capability.API)
  # ---------------------------------------------------------------------

  # =====================================================================
  # BEGIN projection target: op list
  # 28 defdelegates (14 ops x typed + bang twin). Hand-written today; this
  # block is the span a projection of the registry op list will replace.
  # =====================================================================

  # GL28-11: the 14 typed capability verbs surface on the facade. Each
  # `op(args, opts)` delegates to `GraphlawEx.Capability.API.op/2` and each
  # `op!(args, opts)` to the raising twin; `{:error, %GraphlawEx.Capability.Refusal{}}`
  # on refusal, or `GraphlawEx.Error` raised.
  # Results are observations, never authority.

  @doc "Lists the operations and dialects the engine supports. Returns `{:ok, result}` or `{:error, refusal}`."
  @spec capabilities(args(), keyword()) ::
          {:ok, GraphlawEx.Capability.Capabilities.t()} | {:error, CapRefusal.t()}
  defdelegate capabilities(args \\ [], opts \\ []), to: GraphlawEx.Capability.API

  @doc "Like `capabilities/2` but returns the result directly and raises `GraphlawEx.Error` on refusal."
  @spec capabilities!(args(), keyword()) :: GraphlawEx.Capability.Capabilities.t()
  defdelegate capabilities!(args \\ [], opts \\ []), to: GraphlawEx.Capability.API

  @doc "Detects the RDF dialect of the given text. Returns `{:ok, result}` or `{:error, refusal}`."
  @spec sniff(args(), keyword()) ::
          {:ok, GraphlawEx.Capability.Sniff.t()} | {:error, CapRefusal.t()}
  defdelegate sniff(args, opts \\ []), to: GraphlawEx.Capability.API

  @doc "Like `sniff/2` but returns the result directly and raises `GraphlawEx.Error` on refusal."
  @spec sniff!(args(), keyword()) :: GraphlawEx.Capability.Sniff.t()
  defdelegate sniff!(args, opts \\ []), to: GraphlawEx.Capability.API

  @doc "Parses RDF text and reports its quad count or syntax errors. Returns `{:ok, result}` or `{:error, refusal}`."
  @spec parse(args(), keyword()) ::
          {:ok, GraphlawEx.Capability.Parse.t()} | {:error, CapRefusal.t()}
  defdelegate parse(args, opts \\ []), to: GraphlawEx.Capability.API

  @doc "Like `parse/2` but returns the result directly and raises `GraphlawEx.Error` on refusal."
  @spec parse!(args(), keyword()) :: GraphlawEx.Capability.Parse.t()
  defdelegate parse!(args, opts \\ []), to: GraphlawEx.Capability.API

  @doc "Converts RDF text from one serialization dialect to another. Returns `{:ok, result}` or `{:error, refusal}`."
  @spec convert(args(), keyword()) ::
          {:ok, GraphlawEx.Capability.Convert.t()} | {:error, CapRefusal.t()}
  defdelegate convert(args, opts \\ []), to: GraphlawEx.Capability.API

  @doc "Like `convert/2` but returns the result directly and raises `GraphlawEx.Error` on refusal."
  @spec convert!(args(), keyword()) :: GraphlawEx.Capability.Convert.t()
  defdelegate convert!(args, opts \\ []), to: GraphlawEx.Capability.API

  @doc "Canonicalizes a dataset to N-Quads and returns its content-addressed id. Returns `{:ok, result}` or `{:error, refusal}`."
  @spec canonical(args(), keyword()) ::
          {:ok, GraphlawEx.Capability.Canonical.t()} | {:error, CapRefusal.t()}
  defdelegate canonical(args, opts \\ []), to: GraphlawEx.Capability.API

  @doc "Like `canonical/2` but returns the result directly and raises `GraphlawEx.Error` on refusal."
  @spec canonical!(args(), keyword()) :: GraphlawEx.Capability.Canonical.t()
  defdelegate canonical!(args, opts \\ []), to: GraphlawEx.Capability.API

  @doc "Runs a SPARQL query or update over a dataset inside the engine. Returns `{:ok, result}` or `{:error, refusal}`."
  @spec sparql(args(), keyword()) ::
          {:ok, GraphlawEx.Capability.Sparql.t()} | {:error, CapRefusal.t()}
  defdelegate sparql(args, opts \\ []), to: GraphlawEx.Capability.API

  @doc "Like `sparql/2` but returns the result directly and raises `GraphlawEx.Error` on refusal."
  @spec sparql!(args(), keyword()) :: GraphlawEx.Capability.Sparql.t()
  defdelegate sparql!(args, opts \\ []), to: GraphlawEx.Capability.API

  @doc "Validates a data graph against a SHACL shapes graph and reports conformance. Returns `{:ok, result}` or `{:error, refusal}`."
  @spec shacl(args(), keyword()) ::
          {:ok, GraphlawEx.Capability.Shacl.t()} | {:error, CapRefusal.t()}
  defdelegate shacl(args, opts \\ []), to: GraphlawEx.Capability.API

  @doc "Like `shacl/2` but returns the result directly and raises `GraphlawEx.Error` on refusal."
  @spec shacl!(args(), keyword()) :: GraphlawEx.Capability.Shacl.t()
  defdelegate shacl!(args, opts \\ []), to: GraphlawEx.Capability.API

  @doc "Validates a data graph against a ShEx schema. Returns `{:ok, result}` or `{:error, refusal}`."
  @spec shex(args(), keyword()) ::
          {:ok, GraphlawEx.Capability.Shex.t()} | {:error, CapRefusal.t()}
  defdelegate shex(args, opts \\ []), to: GraphlawEx.Capability.API

  @doc "Like `shex/2` but returns the result directly and raises `GraphlawEx.Error` on refusal."
  @spec shex!(args(), keyword()) :: GraphlawEx.Capability.Shex.t()
  defdelegate shex!(args, opts \\ []), to: GraphlawEx.Capability.API

  @doc "Evaluates Notation3 rules over a dataset inside the engine. Returns `{:ok, result}` or `{:error, refusal}`."
  @spec n3(args(), keyword()) :: {:ok, GraphlawEx.Capability.N3.t()} | {:error, CapRefusal.t()}
  defdelegate n3(args, opts \\ []), to: GraphlawEx.Capability.API

  @doc "Like `n3/2` but returns the result directly and raises `GraphlawEx.Error` on refusal."
  @spec n3!(args(), keyword()) :: GraphlawEx.Capability.N3.t()
  defdelegate n3!(args, opts \\ []), to: GraphlawEx.Capability.API

  @doc "Computes the entailment closure of a dataset under a chosen regime. Returns `{:ok, result}` or `{:error, refusal}`."
  @spec entail(args(), keyword()) ::
          {:ok, GraphlawEx.Capability.Entail.t()} | {:error, CapRefusal.t()}
  defdelegate entail(args, opts \\ []), to: GraphlawEx.Capability.API

  @doc "Like `entail/2` but returns the result directly and raises `GraphlawEx.Error` on refusal."
  @spec entail!(args(), keyword()) :: GraphlawEx.Capability.Entail.t()
  defdelegate entail!(args, opts \\ []), to: GraphlawEx.Capability.API

  @doc "Evaluates a Datalog program over a dataset inside the engine. Returns `{:ok, result}` or `{:error, refusal}`."
  @spec datalog(args(), keyword()) ::
          {:ok, GraphlawEx.Capability.Datalog.t()} | {:error, CapRefusal.t()}
  defdelegate datalog(args, opts \\ []), to: GraphlawEx.Capability.API

  @doc "Like `datalog/2` but returns the result directly and raises `GraphlawEx.Error` on refusal."
  @spec datalog!(args(), keyword()) :: GraphlawEx.Capability.Datalog.t()
  defdelegate datalog!(args, opts \\ []), to: GraphlawEx.Capability.API

  @doc "Runs the engine's hook set against a dataset and reports the outcomes. Returns `{:ok, result}` or `{:error, refusal}`."
  @spec hooks(args(), keyword()) ::
          {:ok, GraphlawEx.Capability.Hooks.t()} | {:error, CapRefusal.t()}
  defdelegate hooks(args, opts \\ []), to: GraphlawEx.Capability.API

  @doc "Like `hooks/2` but returns the result directly and raises `GraphlawEx.Error` on refusal."
  @spec hooks!(args(), keyword()) :: GraphlawEx.Capability.Hooks.t()
  defdelegate hooks!(args, opts \\ []), to: GraphlawEx.Capability.API

  @doc "Admits a law document through the engine's law kernel. Returns `{:ok, result}` or `{:error, refusal}`."
  @spec law(args(), keyword()) :: {:ok, GraphlawEx.Capability.Law.t()} | {:error, CapRefusal.t()}
  defdelegate law(args, opts \\ []), to: GraphlawEx.Capability.API

  @doc "Like `law/2` but returns the result directly and raises `GraphlawEx.Error` on refusal."
  @spec law!(args(), keyword()) :: GraphlawEx.Capability.Law.t()
  defdelegate law!(args, opts \\ []), to: GraphlawEx.Capability.API

  @doc "Evaluates a policy over a dataset and reports the decision. Returns `{:ok, result}` or `{:error, refusal}`."
  @spec policy(args(), keyword()) ::
          {:ok, GraphlawEx.Capability.Policy.t()} | {:error, CapRefusal.t()}
  defdelegate policy(args, opts \\ []), to: GraphlawEx.Capability.API

  @doc "Like `policy/2` but returns the result directly and raises `GraphlawEx.Error` on refusal."
  @spec policy!(args(), keyword()) :: GraphlawEx.Capability.Policy.t()
  defdelegate policy!(args, opts \\ []), to: GraphlawEx.Capability.API

  # =====================================================================
  # END projection target: op list
  # =====================================================================

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
        case File.read(path) do
          {:ok, bytes} ->
            actual = :crypto.hash(:sha256, bytes) |> Base.encode16(case: :lower)

            if actual == expected,
              do: :ok,
              else:
                {:error,
                 {:wasm_digest_mismatch, %{expected: expected, actual: actual, path: path}}}

          {:error, reason} ->
            {:error, {:wasm_unreadable, %{path: path, reason: reason}}}
        end

      nil ->
        {:error, {:wasm_digest_mismatch, %{expected: nil, actual: nil, path: path}}}
    end
  end
end
