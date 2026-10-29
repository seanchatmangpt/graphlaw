defmodule GraphlawEx.Host do
  @moduledoc """
  One supervised Wasmtime instance of the GraphLaw engine.

  GraphLaw derives and validates; it never authorizes. A `{:ok, response}`
  from `request/3` is transport success: the response map may itself say
  `"ok" => false`. Neither is authority, and neither is a receipt.

  ## ABI (GraphLaw wasm ABI version 1)

  The engine exports `gl_alloc(len) -> ptr`, `gl_call(ptr, len) -> u64` and
  `gl_free(ptr, len)`. A request is UTF-8 JSON written at a `gl_alloc` pointer;
  `gl_call` consumes the request buffer and returns `(out_ptr << 32) | out_len`
  (unpacked by `GraphlawEx.ABI.unpack_result/1`, including the signed-`i64`
  normalization that fixes the old facade hang path); the host reads the UTF-8
  JSON response and then `gl_free`s it. A null `gl_alloc` is `:resource_limit`.

  ## Lifecycle

    * `init/1` never crashes. A missing or foreign engine, a digest mismatch or
      an instantiation failure leaves the server in `{:unavailable, refusal}`
      and every call returns that `GraphlawEx.Refusal`.
    * Admission (`GraphlawEx.EngineLoad.admit/2`) runs before the instance
      starts, so a foreign module cannot crash the caller.
    * One GenServer serializes the whole `gl_alloc` / write / `gl_call` / read /
      `gl_free` transaction; interleaving would corrupt shared instance state.
    * Every transaction starts with a fresh fuel budget.
    * Exits of the instance never kill the host; they are typed (`:call_exited`)
      and the instance is recycled over the cached compiled module.
    * The instance is recycled after a trap, exit, timeout, fuel exhaustion or
      ABI failure, and when linear memory passes `:recycle_bytes` (linear memory
      never shrinks). The caller is answered BEFORE the recycle starts
      (`GenServer.reply/2`, then recycle), so a slow recycle can never turn a
      typed refusal into a `:call_timeout`.
    * Requests are refused `:saturated` at two layers. The caller checks the
      target mailbox against ITS OWN `:max_queue` before enqueueing (fast path,
      best-effort). The host then checks the queue behind each request it
      dequeues against ITS configured `:max_queue` and sheds when that many
      callers are already waiting.
    * A response larger than `:max_response_bytes` is refused `:resource_limit`
      before it is copied out of engine memory; the engine buffer is still freed.
    * A host whose load or recycle failed is `{:unavailable, refusal}`, not dead:
      it retries the load with capped exponential backoff and, when a `:registry`
      is given, stays registered under `:unavailable` while it heals.

  ## Timeouts, nested

  native call deadline (`timeout`) < per-step `GenServer.call` (`timeout` +
  `@abi_timeout` margins) < outer caller deadline. The recycle runs after the
  reply, so it is outside every caller deadline.

  ## Telemetry

    * `[:graphlaw_ex, :host, :call, :stop]` `%{duration}`, metadata
      `%{op, outcome, code}`
    * `[:graphlaw_ex, :host, :recycle]` `%{count}`, metadata `%{reason, wasm_sha256}`
  """

  use GenServer

  alias GraphlawEx.ABI
  alias GraphlawEx.EngineLoad
  alias GraphlawEx.Refusal
  alias GraphlawEx.WasmConfig

  require Logger

  # Bound for the small ABI helper calls (gl_alloc, gl_free, _initialize probe).
  @abi_timeout 1_000
  @margin 1_000
  @recycle_codes [:call_exited, :call_trapped, :call_timeout, :fuel_exhausted]
  @retry_base_ms 500
  @retry_max_ms 30_000

  @typedoc "Server reference accepted by every function in this module."
  @type server :: GenServer.server()

  @typedoc "Options for `start_link/1`."
  @type opts :: [
          name: GenServer.name() | nil,
          wasm_path: String.t(),
          bytes: binary(),
          expected_sha256: String.t() | :unpinned,
          fuel: pos_integer(),
          memory_limit_bytes: pos_integer(),
          recycle_bytes: pos_integer(),
          max_queue: pos_integer(),
          max_response_bytes: pos_integer(),
          timeout_ms: pos_integer(),
          registry: atom(),
          retry_base_ms: pos_integer(),
          retry_max_ms: pos_integer()
        ]

  @typedoc "Options for `request/3`."
  @type request_opts :: [timeout_ms: pos_integer(), max_queue: pos_integer()]

  # ---------------------------------------------------------------------
  # Public API
  # ---------------------------------------------------------------------

  @doc """
  Starts a host. Options: `:name` (default this module; `nil` for an unnamed
  pool member), `:wasm_path`, `:bytes`, `:expected_sha256` (`:unpinned` opts
  out of the pin), `:fuel`, `:memory_limit_bytes`, `:recycle_bytes`,
  `:max_queue`, `:timeout_ms`, `:max_response_bytes`, `:registry` (a
  duplicate-key `Registry` the host joins under `:members` while available and
  `:unavailable` otherwise), `:retry_base_ms`, `:retry_max_ms`.

  A missing or unloadable engine never fails the start; it becomes a typed
  refusal on every call.
  """
  @spec start_link(opts()) :: GenServer.on_start()
  def start_link(opts \\ []) do
    case Keyword.pop(opts, :name, __MODULE__) do
      {nil, rest} -> GenServer.start_link(__MODULE__, rest)
      {name, rest} -> GenServer.start_link(__MODULE__, [{:registered_as, name} | rest], name: name)
    end
  end

  @doc """
  Runs one GraphLaw request (a string-keyed map such as `%{"op" => "law", ...}`).

  Returns the decoded response map whether the engine answered `"ok" => true`
  or `"ok" => false`; only transport and host failures are `{:error, refusal}`.
  """
  @spec request(server(), map(), request_opts()) :: {:ok, map()} | {:error, Refusal.t()}
  def request(server \\ __MODULE__, request, opts \\ []) when is_map(request) and is_list(opts) do
    limits = WasmConfig.limits(opts)
    timeout = limits.timeout_ms
    max_queue = Keyword.get(opts, :max_queue, limits.max_queue)

    with {:ok, body} <- ABI.encode_request(request),
         {:ok, target} <- shed(server, max_queue) do
      GenServer.call(target, {:request, body, op_of(request), timeout}, call_timeout(timeout))
    end
  catch
    :exit, {:noproc, _} ->
      {:error, Refusal.new(:call_exited, "no host is registered as #{inspect(server)}")}

    :exit, {:timeout, _} ->
      {:error, Refusal.new(:call_timeout, "host call exceeded its outer deadline", %{op: op_of(request)})}

    :exit, reason ->
      {:error, Refusal.new(:call_exited, "host exited during the call", %{reason: inspect(reason, limit: 5)})}
  end

  @doc """
  Runs one pre-encoded JSON request body (raw bytes) through the same shed +
  serialized-transaction path as `request/3`, skipping map encoding. Used by
  the `GraphlawEx.admit/2` facade, whose payload is already a JSON document.
  """
  @spec request_raw(server(), binary(), request_opts()) :: {:ok, map()} | {:error, Refusal.t()}
  def request_raw(server \\ __MODULE__, body, opts \\ []) when is_binary(body) and is_list(opts) do
    limits = WasmConfig.limits(opts)
    timeout = limits.timeout_ms
    max_queue = Keyword.get(opts, :max_queue, limits.max_queue)

    with {:ok, target} <- shed(server, max_queue) do
      GenServer.call(target, {:request, body, :admit, timeout}, call_timeout(timeout))
    end
  catch
    :exit, {:noproc, _} ->
      {:error, Refusal.new(:call_exited, "no host is registered as #{inspect(server)}")}

    :exit, {:timeout, _} ->
      {:error, Refusal.new(:call_timeout, "host call exceeded its outer deadline")}

    :exit, reason ->
      {:error, Refusal.new(:call_exited, "host exited during the call", %{reason: inspect(reason, limit: 5)})}
  end

  @doc "Whether the host has a live, admitted engine instance."
  @spec available?(server(), timeout()) :: boolean()
  def available?(server \\ __MODULE__, timeout \\ 5_000) do
    match?({:ok, :loaded}, GenServer.call(server, :status, timeout))
  catch
    :exit, _ -> false
  end

  @doc "`%{wasm_sha256, path, recycles}` for a loaded host, or the typed refusal."
  @spec info(server(), timeout()) :: {:ok, map()} | {:error, Refusal.t()}
  def info(server \\ __MODULE__, timeout \\ 5_000), do: call_simple(server, :info, timeout)

  @doc "Current size in bytes of the engine's linear memory."
  @spec memory_size(server(), timeout()) :: {:ok, non_neg_integer()} | {:error, Refusal.t()}
  def memory_size(server \\ __MODULE__, timeout \\ 5_000), do: call_simple(server, :memory_size, timeout)

  defp call_simple(server, message, timeout) do
    GenServer.call(server, message, timeout)
  catch
    :exit, {:noproc, _} ->
      {:error, Refusal.new(:call_exited, "no host is registered as #{inspect(server)}")}

    :exit, {:timeout, _} ->
      {:error, Refusal.new(:call_timeout, "host did not answer #{message} in time")}
  end

  defp op_of(request), do: Map.get(request, "op") || Map.get(request, :op)

  # Caller-side shedding: a fast, best-effort mailbox check against the caller's
  # own bound. The host re-checks behind its own bound when it dequeues.
  defp shed(server, max_queue) do
    with pid when is_pid(pid) <- GenServer.whereis(server),
         {:message_queue_len, len} when len >= max_queue <- Process.info(pid, :message_queue_len) do
      {:error, Refusal.new(:saturated, "host mailbox holds #{len} messages", %{queue: len, max_queue: max_queue})}
    else
      _ -> {:ok, server}
    end
  end

  # The caller outlives every inner step: alloc + call + free, each with its own
  # margin, plus the recycle window.
  defp call_timeout(timeout), do: timeout + @margin + 2 * (@abi_timeout + @margin) + @margin

  # ---------------------------------------------------------------------
  # GenServer
  # ---------------------------------------------------------------------

  @impl GenServer
  def init(opts) do
    Process.flag(:trap_exit, true)
    context = %{opts: opts, registry: Keyword.get(opts, :registry), retries: 0}

    case load(opts) do
      {:ok, state} ->
        {:ok, state |> Map.merge(context) |> announce(:available)}

      {:error, %Refusal{} = refusal} ->
        Logger.warning("GraphlawEx.Host: engine unavailable (#{refusal.code}: #{refusal.message}).")
        {:ok, context |> Map.put(:status, {:unavailable, refusal}) |> announce(:unavailable) |> schedule_retry()}
    end
  end

  @impl GenServer
  def handle_call(:status, _from, %{status: :loaded} = state), do: {:reply, {:ok, :loaded}, state}

  def handle_call(_message, _from, %{status: {:unavailable, refusal}} = state),
    do: {:reply, {:error, refusal}, state}

  def handle_call(:info, _from, state) do
    {:reply, {:ok, Map.take(state, [:wasm_sha256, :path, :recycles])}, state}
  end

  def handle_call(:memory_size, _from, %{store: store, memory: memory} = state) do
    {:reply, {:ok, Wasmex.Memory.size(store, memory)}, state}
  end

  def handle_call({:request, body, op, timeout}, from, state) do
    case saturated(state) do
      nil -> run_request(body, op, timeout, from, state)
      refusal -> {:reply, {:error, refusal}, state}
    end
  end

  # Server-side shedding with the host's own configured bound.
  defp saturated(%{limits: %{max_queue: max_queue}}) do
    {:message_queue_len, waiting} = Process.info(self(), :message_queue_len)

    if waiting >= max_queue,
      do:
        Refusal.new(:saturated, "host has #{waiting} requests waiting behind this one", %{
          queue: waiting,
          max_queue: max_queue
        })
  end

  defp run_request(body, op, timeout, from, state) do
    started = System.monotonic_time()
    :ok = Wasmex.StoreOrCaller.set_fuel(state.store, state.limits.fuel)
    result = transact(state, body, timeout)
    emit_call(started, op, result)
    # Answer first: a recycle can take seconds, and the caller's deadline covers
    # the transaction, not the housekeeping after it.
    GenServer.reply(from, result)
    {:noreply, maybe_recycle(state, result)}
  end

  @impl GenServer
  # The instance is linked and this host traps exits: replace it, do not die.
  def handle_info({:EXIT, pid, reason}, %{status: :loaded, pid: pid} = state) do
    {:noreply, recycle(state, {:instance_exit, inspect(reason, limit: 5)})}
  end

  def handle_info(:retry_load, %{status: {:unavailable, _}} = state) do
    case load(state.opts) do
      {:ok, loaded} ->
        Logger.info("GraphlawEx.Host: engine available again after #{state.retries + 1} retry(ies).")

        live =
          state
          |> Map.take([:opts, :registry])
          |> Map.merge(loaded)
          |> Map.put(:retries, 0)

        {:noreply, announce(live, :available)}

      {:error, %Refusal{} = refusal} ->
        {:noreply, %{state | status: {:unavailable, refusal}, retries: state.retries + 1} |> schedule_retry()}
    end
  end

  def handle_info(:retry_load, state), do: {:noreply, state}

  def handle_info(_message, state), do: {:noreply, state}

  # ---------------------------------------------------------------------
  # Loading and instantiation
  # ---------------------------------------------------------------------

  # Registry membership mirrors availability: a routable host is under
  # `:members`, an unavailable one under `:unavailable`.
  defp announce(%{registry: nil} = state, _availability), do: state

  defp announce(%{registry: registry} = state, availability) do
    {join, leave} = if availability == :available, do: {:members, :unavailable}, else: {:unavailable, :members}
    Registry.unregister(registry, leave)
    Registry.unregister(registry, join)
    {:ok, _} = Registry.register(registry, join, nil)
    state
  end

  defp schedule_retry(%{opts: opts, retries: retries} = state) do
    base = Keyword.get(opts, :retry_base_ms, @retry_base_ms)
    cap = Keyword.get(opts, :retry_max_ms, @retry_max_ms)
    delay = min(cap, base * Integer.pow(2, min(retries, 16)))
    Process.send_after(self(), :retry_load, delay)
    state
  end

  defp load(opts) do
    path = WasmConfig.wasm_path(opts)
    expected = WasmConfig.expected_sha256(opts)

    with {:ok, bytes} <- read_bytes(path, opts),
         :ok <- require_pin(expected),
         {:ok, admitted} <- EngineLoad.admit(bytes, expected_sha256: expected, fuel?: true) do
      state = %{
        status: :loaded,
        path: path,
        wasm_sha256: admitted.wasm_sha256,
        engine: admitted.engine,
        module: admitted.module,
        initialize?: Map.has_key?(admitted.exports, "_initialize"),
        limits: WasmConfig.limits(opts),
        recycles: 0
      }

      instantiate(state)
    end
  end

  # A nil pin means the manifest is unusable; that is a refusal, never "unpinned".
  defp require_pin(nil),
    do: {:error, Refusal.new(:wasm_digest_mismatch, "no engine pin is available (MANIFEST.json unusable)")}

  defp require_pin(_expected), do: :ok

  defp read_bytes(path, opts) do
    case Keyword.fetch(opts, :bytes) do
      {:ok, bytes} when is_binary(bytes) -> {:ok, bytes}
      _ -> read_file(path)
    end
  end

  defp read_file(path) do
    case File.read(path) do
      {:ok, bytes} ->
        {:ok, bytes}

      {:error, :enoent} ->
        {:error, Refusal.new(:resource_limit, "no engine at #{path}", %{path: path, reason: :enoent})}

      {:error, reason} ->
        {:error,
         Refusal.new(:resource_limit, "cannot read #{path}: #{:file.format_error(reason)}", %{path: path})}
    end
  end

  # One fresh WASI store + instance over the cached compiled module.
  defp instantiate(%{engine: engine, module: module, limits: limits} = state) do
    memory_limits = %Wasmex.StoreLimits{
      memory_size: limits.memory_limit_bytes,
      table_elements: limits.table_elements,
      instances: limits.instances,
      tables: limits.tables,
      memories: limits.memories
    }

    with {:ok, store} <- new_store(memory_limits, engine),
         :ok <- Wasmex.StoreOrCaller.set_fuel(store, max(limits.fuel, limits.instantiate_fuel)),
         {:ok, pid} <- start_instance(store, module),
         {:ok, store} <- Wasmex.store(pid),
         {:ok, memory} <- Wasmex.memory(pid),
         live = Map.merge(state, %{pid: pid, store: store, memory: memory}),
         :ok <- run_initialize(live) do
      {:ok, live}
    else
      {:error, %Refusal{}} = refusal -> refusal
      other -> {:error, failed_instantiation(other)}
    end
  end

  defp new_store(memory_limits, engine) do
    Wasmex.Store.new_wasi(%Wasmex.Wasi.WasiOptions{}, memory_limits, engine)
  rescue
    error -> {:error, Exception.message(error)}
  end

  defp start_instance(store, module) do
    Wasmex.start_link(%{store: store, module: module})
  rescue
    error -> {:error, Exception.message(error)}
  catch
    :exit, reason -> {:error, reason}
  end

  defp failed_instantiation(reason) do
    Refusal.new(:resource_limit, "engine instantiation failed", %{reason: inspect(reason, limit: 5)})
  end

  # Reactor-style modules export `_initialize`; call it exactly once per instance.
  defp run_initialize(%{initialize?: false}), do: :ok

  defp run_initialize(state) do
    case call_raw(state, "_initialize", [], @abi_timeout * 10) do
      {:ok, _} -> :ok
      {:error, reason} -> {:error, classify(reason)}
    end
  end

  defp recycle(%{pid: old} = state, reason) do
    Process.unlink(old)
    Process.exit(old, :kill)
    count = state.recycles + 1

    :telemetry.execute([:graphlaw_ex, :host, :recycle], %{count: count}, %{
      reason: reason,
      wasm_sha256: state.wasm_sha256
    })

    case instantiate(state) do
      {:ok, fresh} ->
        %{fresh | recycles: count}

      {:error, refusal} ->
        Logger.warning("GraphlawEx.Host: recycle failed (#{refusal.code}); host is unavailable, retrying.")

        state
        |> Map.take([:opts, :registry])
        |> Map.merge(%{status: {:unavailable, refusal}, retries: 0})
        |> announce(:unavailable)
        |> schedule_retry()
    end
  end

  defp maybe_recycle(state, {:error, %Refusal{code: code}}) when code in @recycle_codes, do: recycle(state, code)

  defp maybe_recycle(%{store: store, memory: memory, limits: limits} = state, _result) do
    if Wasmex.Memory.size(store, memory) > limits.recycle_bytes,
      do: recycle(state, :memory_high_water),
      else: state
  end

  # ---------------------------------------------------------------------
  # The ABI transaction
  # ---------------------------------------------------------------------

  defp transact(state, body, timeout) do
    len = byte_size(body)

    with {:ok, ptr} <- alloc(state, len),
         :ok <- write_request(state, ptr, body),
         {:ok, packed} <- call_engine(state, ptr, len, timeout) do
      read_response(state, packed)
    end
  end

  defp alloc(state, len) do
    case call_raw(state, "gl_alloc", [len], @abi_timeout) do
      {:ok, [ptr]} when is_integer(ptr) and ptr > 0 ->
        {:ok, ptr}

      {:ok, [0]} ->
        {:error, Refusal.new(:resource_limit, "gl_alloc refused a #{len}-byte request", %{bytes: len})}

      {:ok, other} ->
        {:error,
         Refusal.new(:malformed_response, "gl_alloc returned an unexpected value", %{
           step: :gl_alloc,
           detail: inspect(other)
         })}

      {:error, reason} ->
        {:error, classify(reason)}
    end
  end

  # gl_call has not consumed the buffer yet, so a failed write must free it.
  defp write_request(%{store: store, memory: memory} = state, ptr, body) do
    :ok = Wasmex.Memory.write_binary(store, memory, ptr, body)
  rescue
    error ->
      _ = call_raw(state, "gl_free", [ptr, byte_size(body)], @abi_timeout)

      {:error,
       Refusal.new(:malformed_response, "writing the request failed", %{
         step: :write_binary,
         detail: Exception.message(error)
       })}
  end

  # gl_call consumes (frees) the request buffer: no gl_free for it after this.
  defp call_engine(state, ptr, len, timeout) do
    case call_raw(state, "gl_call", [ptr, len], timeout) do
      {:ok, [packed]} when is_integer(packed) ->
        {:ok, packed}

      {:ok, other} ->
        {:error,
         Refusal.new(:malformed_response, "gl_call returned an unexpected value", %{
           step: :gl_call,
           detail: inspect(other)
         })}

      {:error, reason} ->
        {:error, classify(reason)}
    end
  end

  defp read_response(state, packed) do
    case ABI.unpack_result(packed) do
      {0, 0} ->
        {:error,
         Refusal.new(:malformed_response, "gl_call returned an empty result slot", %{step: :read_result})}

      {out_ptr, out_len} ->
        try do
          if out_len > state.limits.max_response_bytes do
            {:error,
             Refusal.new(:resource_limit, "engine response of #{out_len} bytes exceeds the response cap", %{
               bytes: out_len,
               max_response_bytes: state.limits.max_response_bytes
             })}
          else
            state |> read_memory(out_ptr, out_len) |> decode()
          end
        after
          # Runs even when the read raises, so a failed read cannot leak memory.
          _ = call_raw(state, "gl_free", [out_ptr, out_len], @abi_timeout)
        end
    end
  end

  defp read_memory(%{store: store, memory: memory}, ptr, len) do
    {:ok, Wasmex.Memory.read_binary(store, memory, ptr, len)}
  rescue
    error ->
      {:error,
       Refusal.new(:malformed_response, "reading the response failed", %{
         step: :read_result,
         detail: Exception.message(error)
       })}
  end

  defp decode({:ok, bytes}) when is_binary(bytes) do
    if String.valid?(bytes),
      do: ABI.decode_response(bytes),
      else: {:error, Refusal.new(:malformed_response, "engine response is not valid UTF-8")}
  end

  defp decode({:error, %Refusal{}} = error), do: error

  # Direct call into the Wasmex GenServer so the native deadline (`timeout`) and
  # the caller deadline differ: the native interrupt fires first and returns
  # `{:error, _}`. Any exit is caught and typed; it must never propagate into
  # this host's handle_call/3.
  defp call_raw(%{pid: pid}, fun, params, timeout) do
    GenServer.call(pid, {:call_function, fun, params, timeout}, timeout + @margin)
  catch
    :exit, reason -> {:error, {:exit, reason}}
  end

  defp classify({:exit, reason}) do
    Refusal.new(:call_exited, "engine instance exited during the call", %{reason: inspect(reason, limit: 5)})
  end

  defp classify(reason) when is_binary(reason) do
    down = String.downcase(reason)

    cond do
      String.contains?(down, "fuel") ->
        Refusal.new(:fuel_exhausted, "engine exhausted its fuel budget", %{reason: reason})

      String.contains?(down, ["interrupt", "timeout", "deadline", "epoch"]) ->
        Refusal.new(:call_timeout, "engine call exceeded its deadline", %{reason: reason})

      true ->
        Refusal.new(:call_trapped, "engine call trapped", %{reason: reason})
    end
  end

  defp classify(reason) do
    Refusal.new(:call_trapped, "engine call failed in an unclassified way", %{reason: inspect(reason, limit: 5)})
  end

  defp emit_call(started, op, result) do
    {outcome, code} =
      case result do
        {:ok, %{"ok" => true}} -> {:ok, nil}
        {:ok, _} -> {:refused, nil}
        {:error, %Refusal{code: code}} -> {:error, code}
      end

    :telemetry.execute(
      [:graphlaw_ex, :host, :call, :stop],
      %{duration: System.monotonic_time() - started},
      %{op: op, outcome: outcome, code: code}
    )
  end
end
