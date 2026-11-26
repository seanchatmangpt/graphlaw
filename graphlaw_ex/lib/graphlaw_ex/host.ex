# SPDX-FileCopyrightText: 2026 Sean Chatman <https://github.com/seanchatmangpt/graphlaw/graphs/contributors>
#
# SPDX-License-Identifier: MIT
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
  alias GraphlawEx.Telemetry
  alias GraphlawEx.WasmConfig

  require Logger

  # Bound for the small ABI helper calls (gl_alloc, gl_free, _initialize probe).
  @abi_timeout 1_000
  @margin 1_000
  @recycle_codes [
    :call_exited,
    :call_trapped,
    :call_timeout,
    :fuel_exhausted,
    :malformed_response
  ]
  @max_request_bytes 16_777_216
  @retry_base_ms 500
  @retry_max_ms 30_000
  # Bound on waiting for a killed instance's DOWN before replacing it.
  @reap_timeout 2_000

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
      {nil, rest} ->
        GenServer.start_link(__MODULE__, rest)

      {name, rest} ->
        start_named(name, [{:registered_as, name} | rest], 5_000)
    end
  end

  # A supervisor restarting a killed parent can start the replacement while the
  # previous holder of the registered name is still unwinding. Wait for that
  # holder's DOWN (monitor, bounded) and retry once, instead of failing the start and
  # burning the supervisor's restart intensity.
  defp start_named(name, init_opts, budget_ms) do
    case GenServer.start_link(__MODULE__, init_opts, name: name) do
      {:error, {:already_started, old}} = err when is_pid(old) and budget_ms > 0 ->
        ref = Process.monitor(old)

        receive do
          {:DOWN, ^ref, :process, ^old, _} -> start_named(name, init_opts, 0)
        after
          budget_ms ->
            Process.demonitor(ref, [:flush])
            err
        end

      other ->
        other
    end
  end

  @doc """
  Runs one GraphLaw request (a string-keyed map such as `%{"op" => "law", ...}`).

  Returns the decoded response map whether the engine answered `"ok" => true`
  or `"ok" => false`; only transport and host failures are `{:error, refusal}`.
  """
  @spec request(map(), request_opts()) :: {:ok, map()} | {:error, Refusal.t()}
  def request(request, opts) when is_map(request) and is_list(opts) do
    {server, opts} = Keyword.pop(opts, :server, __MODULE__)
    request(server, request, opts)
  end

  @spec request(server(), map()) :: {:ok, map()} | {:error, Refusal.t()}
  def request(server, request), do: request(server, request, [])

  @spec request(map()) :: {:ok, map()} | {:error, Refusal.t()}
  def request(request), do: request(__MODULE__, request, [])

  @spec request(server(), map(), request_opts()) :: {:ok, map()} | {:error, Refusal.t()}
  def request(_server, request, opts) when not is_map(request) or not is_list(opts) do
    {:error,
     Refusal.new(:invalid_json, "request must be a map and opts a keyword list", %{
       bytes: size_of(request)
     })}
  end

  def request(server, request, opts) do
    limits = WasmConfig.limits(opts)
    timeout = limits.timeout_ms
    max_queue = Keyword.get(opts, :max_queue, limits.max_queue)

    with {:ok, body} <- ABI.encode_request(request),
         {:ok, target} <- shed(server, max_queue) do
      GenServer.call(
        target,
        request_message(body, op_of(request), timeout, opts, limits),
        call_timeout(timeout)
      )
    end
  catch
    :exit, {:noproc, _} ->
      {:error, Refusal.new(:call_exited, "no host is registered as #{inspect(server)}")}

    :exit, {:timeout, _} ->
      {:error,
       Refusal.new(:call_timeout, "host call exceeded its outer deadline", %{op: op_of(request)})}

    :exit, reason ->
      {:error,
       Refusal.new(:call_exited, "host exited during the call", %{
         reason: exit_tag(reason)
       })}
  end

  @doc """
  Runs one pre-encoded JSON request body (raw bytes) through the same shed +
  serialized-transaction path as `request/3`, skipping map encoding. Used by
  the `GraphlawEx.admit/2` facade, whose payload is already a JSON document.
  """
  @spec request_raw(server(), binary(), request_opts()) :: {:ok, map()} | {:error, Refusal.t()}
  def request_raw(server \\ __MODULE__, body, opts \\ [])

  def request_raw(_server, body, opts) when not is_binary(body) or not is_list(opts) do
    {:error,
     Refusal.new(:invalid_json, "raw request must be a binary and opts a keyword list", %{
       bytes: size_of(body)
     })}
  end

  def request_raw(server, body, opts) do
    limits = WasmConfig.limits(opts)
    timeout = limits.timeout_ms
    max_queue = Keyword.get(opts, :max_queue, limits.max_queue)

    with {:ok, body} <- ABI.recognize_request(body),
         {:ok, target} <- shed(server, max_queue) do
      GenServer.call(
        target,
        request_message(body, :admit, timeout, opts, limits),
        call_timeout(timeout)
      )
    end
  catch
    :exit, {:noproc, _} ->
      {:error, Refusal.new(:call_exited, "no host is registered as #{inspect(server)}")}

    :exit, {:timeout, _} ->
      {:error, Refusal.new(:call_timeout, "host call exceeded its outer deadline")}

    :exit, reason ->
      {:error,
       Refusal.new(:call_exited, "host exited during the call", %{
         reason: exit_tag(reason)
       })}
  end

  @doc "Whether the host has a live, admitted engine instance."
  @spec available?(server(), timeout()) :: boolean()
  def available?(server \\ __MODULE__, timeout \\ 5_000) do
    match?({:ok, :loaded}, GenServer.call(server, :status, timeout))
  catch
    :exit, _ -> false
    _kind, _reason -> false
  end

  @doc "`%{wasm_sha256, path, recycles}` for a loaded host (`path` is the basename), or the typed refusal."
  @spec info(server(), timeout()) ::
          {:ok, %{wasm_sha256: String.t(), path: String.t(), recycles: non_neg_integer()}}
          | {:error, Refusal.t()}
  def info(server \\ __MODULE__, timeout \\ 5_000), do: call_simple(server, :info, timeout)

  @doc "Current size in bytes of the engine's linear memory."
  @spec memory_size(server(), timeout()) :: {:ok, non_neg_integer()} | {:error, Refusal.t()}
  def memory_size(server \\ __MODULE__, timeout \\ 5_000),
    do: call_simple(server, :memory_size, timeout)

  defp call_simple(server, message, timeout) do
    GenServer.call(server, message, timeout)
  catch
    :exit, {:noproc, _} ->
      {:error, Refusal.new(:call_exited, "no host is registered as #{inspect(server)}")}

    :exit, {:timeout, _} ->
      {:error, Refusal.new(:call_timeout, "host did not answer #{message} in time")}
  end

  # A caller-supplied `:fuel` travels with the call; the host caps it by its own
  # ceiling. Without one, the 4-tuple keeps the host's configured budget.
  # The message carries an absolute monotonic deadline (ms): the host drops a
  # request whose caller has already given up without running it.
  defp request_message(body, op, timeout, opts, limits) do
    fuel = if Keyword.has_key?(opts, :fuel), do: limits.fuel, else: nil
    deadline = System.monotonic_time(:millisecond) + call_timeout(timeout)
    {:request, body, op, timeout, fuel, deadline}
  end

  defp size_of(value) when is_binary(value), do: byte_size(value)
  defp size_of(value), do: :erlang.external_size(value)

  # Payload-free tags for exit reasons and foreign terms.
  defp exit_tag({tag, _}) when is_atom(tag), do: tag
  defp exit_tag(tag) when is_atom(tag), do: tag
  defp exit_tag(_), do: :other

  defp type_tag(value) when is_atom(value), do: :atom
  defp type_tag(value) when is_binary(value), do: :binary
  defp type_tag(value) when is_integer(value), do: :integer
  defp type_tag(value) when is_float(value), do: :float
  defp type_tag(value) when is_list(value), do: :list
  defp type_tag(value) when is_tuple(value), do: :tuple
  defp type_tag(value) when is_map(value), do: :map
  defp type_tag(_), do: :other

  defp exception_name(%{__struct__: module}), do: inspect(module)

  defp op_of(request) when is_map(request), do: Map.get(request, "op") || Map.get(request, :op)
  defp op_of(_request), do: nil

  # Caller-side shedding: a fast, best-effort mailbox check against the caller's
  # own bound. The host re-checks behind its own bound when it dequeues.
  defp shed(server, max_queue) do
    with pid when is_pid(pid) <- GenServer.whereis(server),
         {:message_queue_len, len} when len >= max_queue <- Process.info(pid, :message_queue_len) do
      {:error,
       Refusal.new(:saturated, "host mailbox holds #{len} messages", %{
         queue: len,
         max_queue: max_queue
       })}
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
    # Wasmex instances live under this host-local supervisor (restart:
    # :temporary), never linked to the host: a NIF crash cannot propagate here.
    {:ok, sup} = DynamicSupervisor.start_link(strategy: :one_for_one)
    context = %{opts: opts, registry: Keyword.get(opts, :registry), retries: 0, sup: sup}

    # Default `load: :sync`: the (slow) engine load completes inside `init/1`, so
    # `start_link/1` returns a loaded (or typed-unavailable) host and the first
    # call never races the compile. `load: :async` defers the load to
    # `handle_continue/2` so N members can boot in parallel; until it completes
    # the host is announced :unavailable (never routable while loading).
    case Keyword.get(opts, :load, :sync) do
      :async ->
        state =
          context
          |> Map.put(:status, {:unavailable, Refusal.new(:resource_limit, "engine is loading")})
          |> announce(:unavailable)

        {:ok, state, {:continue, :initial_load}}

      _sync ->
        {:noreply, state} = handle_continue(:initial_load, context)
        {:ok, state}
    end
  end

  @impl GenServer
  def handle_continue(:initial_load, %{opts: opts, sup: sup} = context) do
    case load(opts, sup) do
      {:ok, state} ->
        live = Map.merge(state, Map.take(context, [:opts, :registry, :retries, :sup]))
        {:noreply, announce(live, :available)}

      {:error, %Refusal{} = refusal} ->
        Logger.warning("GraphlawEx.Host: engine unavailable (#{refusal.code}).")

        {:noreply,
         context
         |> Map.put(:status, {:unavailable, refusal})
         |> announce(:unavailable)
         |> schedule_retry()}
    end
  end

  # Redacts opts (bytes, paths), the store and in-flight request bodies from
  # crash reports and `:sys.get_status/1`.
  @impl GenServer
  def format_status(status) when is_map(status) do
    status
    |> Map.update(:state, nil, &redact_state/1)
    |> Map.update(:message, nil, &redact_message/1)
  end

  defp redact_state(%{} = state) do
    tag =
      case Map.get(state, :status) do
        {:unavailable, _} -> :unavailable
        other -> other
      end

    %{
      status: tag,
      wasm_sha256: Map.get(state, :wasm_sha256),
      recycles: Map.get(state, :recycles),
      path: Map.get(state, :path)
    }
  end

  defp redact_state(_), do: :redacted

  defp redact_message({:request, _body, op, _timeout, _fuel, _deadline}), do: {:request, op}
  defp redact_message({:request, _body, op, _timeout, _fuel}), do: {:request, op}
  defp redact_message({:request, _body, op, _timeout}), do: {:request, op}
  defp redact_message(other) when is_atom(other), do: other
  defp redact_message(_), do: :redacted

  @impl GenServer
  def terminate(_reason, %{sup: sup} = state) when is_pid(sup) do
    case state do
      %{pid: pid} when is_pid(pid) -> Process.exit(pid, :kill)
      _ -> :ok
    end

    try do
      DynamicSupervisor.stop(sup, :shutdown, 5_000)
    catch
      :exit, _ -> :ok
    end

    :ok
  end

  def terminate(_reason, _state), do: :ok

  @impl GenServer
  def handle_call(:status, _from, %{status: :loaded} = state), do: {:reply, {:ok, :loaded}, state}

  def handle_call(_message, _from, %{status: {:unavailable, refusal}} = state),
    do: {:reply, {:error, refusal}, state}

  def handle_call(:info, _from, state) do
    info =
      state |> Map.take([:wasm_sha256, :path, :recycles]) |> Map.update!(:path, &Path.basename/1)

    {:reply, {:ok, info}, state}
  end

  def handle_call(:memory_size, from, state) do
    case memory_probe(state) do
      {:ok, size} ->
        {:reply, {:ok, size}, state}

      :error ->
        refusal = Refusal.new(:call_exited, "engine memory probe failed")
        GenServer.reply(from, {:error, refusal})
        {:noreply, recycle(state, :memory_probe_failed)}
    end
  end

  def handle_call({:request, body, op, timeout}, from, state),
    do: handle_call({:request, body, op, timeout, nil, :infinity}, from, state)

  def handle_call({:request, body, op, timeout, fuel}, from, state),
    do: handle_call({:request, body, op, timeout, fuel, :infinity}, from, state)

  def handle_call({:request, body, op, timeout, fuel, deadline}, from, state) do
    # One span per request (start + stop, or exception + re-raise), emitted
    # through Telemetry.emit/3 so metadata is redacted. The caller is answered
    # BEFORE any recycle, which is outside the span.
    {result, post} =
      Telemetry.span([:graphlaw_ex, :host, :call], %{op: op}, fn ->
        {result, post} =
          execute_request(body, op, timeout, fuel || state.limits.fuel, deadline, from, state)

        {{result, post}, call_stop_meta(result)}
      end)

    GenServer.reply(from, result)
    {:noreply, maybe_recycle(state, result, post)}
  end

  defp expired?(deadline, {pid, _tag}) do
    (is_integer(deadline) and System.monotonic_time(:millisecond) >= deadline) or
      (is_pid(pid) and node(pid) == node() and not Process.alive?(pid))
  end

  defp expired?(_deadline, _from), do: false

  defp memory_probe(%{store: store, memory: memory}) do
    {:ok, Wasmex.Memory.size(store, memory)}
  rescue
    _ -> :error
  catch
    _kind, _reason -> :error
  end

  # Returns `{result, post}`. `post == :skip` marks a refusal issued before any
  # transaction: zero mutation (no fuel, gl_alloc, gl_call, recycle or probe).
  defp execute_request(body, op, timeout, fuel, deadline, from, state) do
    with :ok <- check_expired(deadline, from, op),
         :ok <- check_saturated(state),
         # Art.14 pre-envelope check (GL28-6): after shedding, before the
         # transaction.
         :ok <- GraphlawEx.KillSwitch.check(state.wasm_sha256),
         :ok <- check_request_size(body, state) do
      run_transaction(body, timeout, fuel, state)
    else
      {:error, %Refusal{}} = refused -> {refused, :skip}
    end
  end

  # Caller already gave up (or died): drop without running anything.
  defp check_expired(deadline, from, op) do
    if expired?(deadline, from),
      do:
        {:error,
         Refusal.new(:call_timeout, "request expired before the host dequeued it", %{op: op})},
      else: :ok
  end

  defp check_saturated(state) do
    case saturated(state) do
      nil -> :ok
      refusal -> {:error, refusal}
    end
  end

  # Refused before any fuel, gl_alloc or copy into linear memory.
  defp check_request_size(body, state) do
    max_request = Keyword.get(state.opts, :max_request_bytes, @max_request_bytes)

    if byte_size(body) > max_request do
      {:error,
       Refusal.new(
         :resource_limit,
         "request of #{byte_size(body)} bytes exceeds the request cap",
         %{bytes: byte_size(body), max_request_bytes: max_request}
       )}
    else
      :ok
    end
  end

  defp call_stop_meta(result) do
    {outcome, code} =
      case result do
        {:ok, %{"ok" => true}} -> {:ok, nil}
        {:ok, _} -> {:refused, nil}
        {:error, %Refusal{code: code}} -> {:error, code}
      end

    %{outcome: outcome, code: code}
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

  # Returns `{result, post}` for the transaction (see `transact/3`).
  defp run_transaction(body, timeout, fuel, state) do
    # Per-call fuel is capped by the host's configured ceiling.
    budget = min(fuel, state.limits.fuel)

    try do
      :ok = Wasmex.StoreOrCaller.set_fuel(state.store, budget)
      transact(state, body, timeout)
    rescue
      error ->
        {{:error,
          Refusal.new(:call_trapped, "host transaction raised", %{
            detail: exception_name(error)
          })}, nil}
    catch
      kind, _reason ->
        {{:error,
          Refusal.new(:call_trapped, "host transaction aborted", %{
            detail: type_tag(kind)
          })}, nil}
    end
  end

  @impl GenServer
  # The instance is monitored, not linked: its death is a recycle, never ours.
  def handle_info(
        {:DOWN, mon, :process, pid, reason},
        %{status: :loaded, pid: pid, mon: mon} = state
      ) do
    {:noreply, recycle(state, {:instance_exit, exit_tag(reason)})}
  end

  # The instance supervisor is this host's own child; losing it is fatal to the
  # host so the owning supervisor restarts the pair.
  def handle_info({:EXIT, sup, reason}, %{sup: sup} = state) do
    {:stop, {:instance_supervisor_down, exit_tag(reason)}, state}
  end

  def handle_info(:retry_load, %{status: {:unavailable, _}} = state) do
    case reap_zombie(state) do
      {:error, %Refusal{} = refusal} ->
        {:noreply,
         %{state | status: {:unavailable, refusal}, retries: state.retries + 1}
         |> schedule_retry()}

      {:ok, state} ->
        retry_load(state)
    end
  end

  def handle_info(:retry_load, state), do: {:noreply, state}

  def handle_info(_message, state), do: {:noreply, state}

  defp reap_zombie(%{zombie: {pid, mon}} = state) do
    case reap(pid, mon) do
      :ok -> {:ok, Map.delete(state, :zombie)}
      :timeout -> {:error, zombie_refusal()}
    end
  end

  defp reap_zombie(state), do: {:ok, state}

  defp zombie_refusal,
    do: Refusal.new(:resource_limit, "previous engine instance has not terminated")

  # Kills an instance and waits (bounded) for its DOWN. Already-dead is :ok.
  defp reap(pid, mon) do
    if Process.alive?(pid) do
      Process.exit(pid, :kill)

      receive do
        {:DOWN, ^mon, :process, ^pid, _} -> :ok
      after
        @reap_timeout -> :timeout
      end
    else
      Process.demonitor(mon, [:flush])
      :ok
    end
  end

  defp retry_load(state) do
    case load(state.opts, state.sup) do
      {:ok, loaded} ->
        Logger.info(
          "GraphlawEx.Host: engine #{Path.basename(loaded.path)} available again after " <>
            "#{state.retries + 1} retry(ies)."
        )

        live =
          state
          |> Map.take([:opts, :registry, :sup])
          |> Map.merge(loaded)
          |> Map.put(:retries, 0)

        {:noreply, announce(live, :available)}

      {:error, %Refusal{} = refusal} ->
        {:noreply,
         %{state | status: {:unavailable, refusal}, retries: state.retries + 1}
         |> schedule_retry()}
    end
  end

  # ---------------------------------------------------------------------
  # Loading and instantiation
  # ---------------------------------------------------------------------

  # Registry membership mirrors availability: a routable host is under
  # `:members`, an unavailable one under `:unavailable`.
  defp announce(%{registry: nil} = state, _availability), do: state

  defp announce(%{registry: registry} = state, availability) do
    {join, leave} =
      if availability == :available, do: {:members, :unavailable}, else: {:unavailable, :members}

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

  defp load(opts, sup) do
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
        recycles: 0,
        sup: sup
      }

      instantiate(state)
    end
  end

  # A nil pin means the manifest is unusable; that is a refusal, never "unpinned".
  defp require_pin(nil),
    do:
      {:error,
       Refusal.new(:wasm_digest_mismatch, "no engine pin is available (MANIFEST.json unusable)")}

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
        {:error,
         Refusal.new(:resource_limit, "no engine at #{path}", %{path: path, reason: :enoent})}

      {:error, reason} ->
        {:error,
         Refusal.new(:resource_limit, "cannot read #{path}: #{:file.format_error(reason)}", %{
           path: path
         })}
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
         {:ok, pid} <- start_instance(state.sup, store, module) do
      mon = Process.monitor(pid)

      case finish_instance(state, pid, mon) do
        {:ok, live} ->
          {:ok, live}

        other ->
          # No zombie Wasmex process: kill the half-built instance, bounded wait.
          _ = reap(pid, mon)

          case other do
            {:error, %Refusal{}} = refusal -> refusal
            _ -> {:error, failed_instantiation(other)}
          end
      end
    else
      other -> {:error, failed_instantiation(other)}
    end
  end

  defp finish_instance(state, pid, mon) do
    with {:ok, store} <- Wasmex.store(pid),
         {:ok, memory} <- Wasmex.memory(pid),
         live = Map.merge(state, %{pid: pid, mon: mon, store: store, memory: memory}),
         :ok <- run_initialize(live) do
      {:ok, live}
    end
  end

  # Zero ambient authority, stated explicitly (not left to struct defaults):
  # no argv, an empty environment, no preopened directories, no stdio pipes.
  # With no preopens the engine's `fd_*`/`path_*` imports see no filesystem,
  # which also keeps the `duckdb` op's ATTACH unreachable regardless of
  # GRAPHLAW_ATTACH_ROOTS.
  @wasi_options %Wasmex.Wasi.WasiOptions{
    args: [],
    env: %{},
    preopen: [],
    stdin: nil,
    stdout: nil,
    stderr: nil
  }

  @doc false
  @spec wasi_options() :: Wasmex.Wasi.WasiOptions.t()
  def wasi_options, do: @wasi_options

  defp new_store(memory_limits, engine) do
    Wasmex.Store.new_wasi(@wasi_options, memory_limits, engine)
  rescue
    error -> {:error, {:raised, exception_name(error)}}
  end

  defp start_instance(sup, store, module) do
    spec = %{
      id: Wasmex,
      start: {Wasmex, :start_link, [%{store: store, module: module}]},
      restart: :temporary
    }

    case DynamicSupervisor.start_child(sup, spec) do
      {:ok, pid} when is_pid(pid) -> {:ok, pid}
      {:ok, pid, _info} when is_pid(pid) -> {:ok, pid}
      {:error, reason} -> {:error, {:start_child, type_tag(reason)}}
      other -> {:error, other}
    end
  rescue
    error -> {:error, {:raised, exception_name(error)}}
  catch
    :exit, reason -> {:error, {:exit, exit_tag(reason)}}
  end

  defp failed_instantiation(reason) do
    Refusal.new(:resource_limit, "engine instantiation failed", %{
      reason: type_tag(reason)
    })
  end

  # Reactor-style modules export `_initialize`; call it exactly once per instance.
  defp run_initialize(%{initialize?: false}), do: :ok

  defp run_initialize(state) do
    case call_raw(state, "_initialize", [], @abi_timeout * 10) do
      {:ok, _} -> :ok
      {:error, reason} -> {:error, classify(reason)}
    end
  end

  defp recycle(%{pid: old, mon: mon} = state, reason) do
    count = state.recycles + 1

    # The reap/replace below can block for seconds: leave :members first so the
    # router never picks a host that cannot answer.
    state = announce(state, :unavailable)

    Telemetry.emit([:graphlaw_ex, :host, :recycle], %{count: count}, %{
      reason: reason,
      wasm_sha256: state.wasm_sha256
    })

    # Bounded wait for the old instance to die before replacing it, so zombies
    # cannot accumulate; on timeout stay unavailable and retry later.
    case reap(old, mon) do
      :ok ->
        replace(state, count)

      :timeout ->
        Logger.warning("GraphlawEx.Host: old instance did not terminate; retrying later.")

        state
        |> Map.take([:opts, :registry, :sup])
        |> Map.merge(%{
          status: {:unavailable, zombie_refusal()},
          retries: 0,
          zombie: {old, mon}
        })
        |> announce(:unavailable)
        |> schedule_retry()
    end
  end

  defp replace(state, count) do
    case instantiate(state) do
      {:ok, fresh} ->
        announce(%{fresh | recycles: count}, :available)

      {:error, refusal} ->
        Logger.warning("GraphlawEx.Host: recycle failed (#{refusal.code}); retrying.")

        state
        |> Map.take([:opts, :registry, :sup])
        |> Map.merge(%{status: {:unavailable, refusal}, retries: 0})
        |> announce(:unavailable)
        |> schedule_retry()
    end
  end

  # A refusal issued before any transaction touched the instance: no recycle,
  # no memory probe, no state change.
  defp maybe_recycle(state, _result, :skip), do: state

  # A failed gl_free leaks engine memory: the instance must not be reused.
  defp maybe_recycle(state, _result, :free_failed), do: recycle(state, :free_failed)

  defp maybe_recycle(state, {:error, %Refusal{code: code}}, _post) when code in @recycle_codes,
    do: recycle(state, code)

  # An oversize engine response means the instance grew memory it should not
  # have: replace it rather than reuse it.
  defp maybe_recycle(
         state,
         {:error, %Refusal{code: :resource_limit, details: %{max_response_bytes: _}}},
         _post
       ),
       do: recycle(state, :oversize_response)

  # gl_alloc returned [0] for a request that passed the request cap: the
  # instance heap is exhausted or poisoned, so it must not serve the next call.
  defp maybe_recycle(
         state,
         {:error, %Refusal{code: :resource_limit, details: %{bytes: _, step: :gl_alloc}}},
         _post
       ),
       do: recycle(state, :gl_alloc_null)

  defp maybe_recycle(%{limits: limits} = state, _result, _post) do
    case memory_probe(state) do
      {:ok, size} when size > limits.recycle_bytes -> recycle(state, :memory_high_water)
      {:ok, _size} -> state
      :error -> recycle(state, :memory_probe_failed)
    end
  end

  # ---------------------------------------------------------------------
  # The ABI transaction
  # ---------------------------------------------------------------------

  defp transact(state, body, timeout) do
    len = byte_size(body)

    # Returns `{result, post}`; `post` is `:free_failed` when gl_free failed.
    with {:ok, ptr} <- alloc(state, len),
         :ok <- write_request(state, ptr, body),
         {:ok, packed} <- call_engine(state, ptr, len, timeout) do
      read_response(state, packed)
    else
      {:error, %Refusal{}} = error -> {error, nil}
    end
  end

  defp alloc(state, len) do
    case call_raw(state, "gl_alloc", [len], @abi_timeout) do
      {:ok, [ptr]} when is_integer(ptr) and ptr > 0 ->
        {:ok, ptr}

      {:ok, [0]} ->
        {:error,
         Refusal.new(:resource_limit, "gl_alloc refused a #{len}-byte request", %{
           bytes: len,
           step: :gl_alloc
         })}

      {:ok, other} ->
        {:error,
         Refusal.new(:malformed_response, "gl_alloc returned an unexpected value", %{
           step: :gl_alloc,
           detail: type_tag(other)
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
         detail: exception_name(error)
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
           detail: type_tag(other)
         })}

      {:error, reason} ->
        {:error, classify(reason)}
    end
  end

  defp read_response(state, packed) do
    case ABI.unpack_result(packed) do
      {0, 0} ->
        {{:error,
          Refusal.new(:malformed_response, "gl_call returned an empty result slot", %{
            step: :read_result
          })}, nil}

      {out_ptr, out_len} ->
        result =
          try do
            if out_len > state.limits.max_response_bytes do
              {:error,
               Refusal.new(
                 :resource_limit,
                 "engine response of #{out_len} bytes exceeds the response cap",
                 %{
                   bytes: out_len,
                   max_response_bytes: state.limits.max_response_bytes
                 }
               )}
            else
              state |> read_memory(out_ptr, out_len) |> decode()
            end
          rescue
            error ->
              {:error,
               Refusal.new(:malformed_response, "decoding the response raised", %{
                 step: :read_result,
                 detail: exception_name(error)
               })}
          end

        # Always attempted, so a failed read cannot leak engine memory; a failed
        # free is surfaced so the instance is recycled.
        post =
          case call_raw(state, "gl_free", [out_ptr, out_len], @abi_timeout) do
            {:ok, _} -> nil
            _ -> :free_failed
          end

        {result, post}
    end
  end

  defp read_memory(%{store: store, memory: memory}, ptr, len) do
    {:ok, Wasmex.Memory.read_binary(store, memory, ptr, len)}
  rescue
    error ->
      {:error,
       Refusal.new(:malformed_response, "reading the response failed", %{
         step: :read_result,
         detail: exception_name(error)
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
    Wasmex.call_function(pid, fun, params, timeout)
  catch
    :exit, reason -> {:error, {:exit, reason}}
  end

  defp classify({:exit, {:timeout, _}}),
    do: Refusal.new(:call_timeout, "engine call exceeded its deadline")

  defp classify({:exit, reason}) do
    Refusal.new(:call_exited, "engine instance exited during the call", %{
      reason: exit_tag(reason)
    })
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
    Refusal.new(:call_trapped, "engine call failed in an unclassified way", %{
      reason: type_tag(reason)
    })
  end
end
