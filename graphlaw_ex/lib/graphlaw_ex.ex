defmodule GraphlawEx do
  @moduledoc """
  Deterministic turnstile boundary for Smalt: the BEAM holds only the wasm
  byte-slice of the graphlaw kernel; all law admission happens *inside* the
  module. Nothing on the Elixir side interprets RDF, SHACL, SQL, or receipts.

  The GenServer owns the wasmex store/instance for the loaded artifact and
  speaks the module's ABI version 1:

    1. `gl_alloc(len) -> ptr` — reserve `len` bytes of linear memory.
    2. write the packet-4 payload (JSON request) at `ptr`.
    3. `gl_call(ptr, len) -> packed` with `packed = (out_ptr << 32) | out_len`.
    4. read `out_len` bytes at `out_ptr` (UTF-8 JSON response), `gl_free`.

  A packet-4 frame is `<<length::32-big-endian, payload::binary>>`; the
  length prefix travels as framing metadata — the payload alone (its byte
  length is the declared length) is what lands in linear memory.
  """

  use GenServer

  require Logger

  @default_wasm_path "../priv/graphlaw.wasm"

  @abi_exports ~w(gl_alloc gl_call gl_free)

  # ---------------------------------------------------------------------------
  # Public API
  # ---------------------------------------------------------------------------

  @doc """
  Starts the bridge. Options:

    * `:wasm_path` — filesystem path to the graphlaw wasm artifact
      (falls back to application env `:graphlaw_ex, :wasm_path`, then
      #{@default_wasm_path} relative to the CWD).

  A missing or unreadable artifact is a typed `{:error, {:wasm_load_failed,
  path, :enoent}}` returned from this function, never a linked-process exit.
  """
  @spec start_link(keyword()) :: GenServer.on_start()
  def start_link(opts \\ []) do
    # Typed fail-closed precondition: refuse before spawning so a missing or
    # unreadable artifact is a returned error, never a linked-process exit.
    path = resolve_wasm_path(opts)

    if File.regular?(path) do
      GenServer.start_link(__MODULE__, opts, name: __MODULE__)
    else
      {:error, {:wasm_load_failed, path, :enoent}}
    end
  end

  @doc """
  Admits an order through the law kernel inside the wasm module.

  `payload` is a UTF-8 JSON request document (`{"op": ...}`). It is framed
  as a packet-4 frame and the payload is handed to the module's ABI:
  `gl_alloc` -> `Wasmex.Memory.write_binary/4` -> `gl_call` -> memory read
  -> `gl_free`. The reply is `{:ok, response_map}` with the decoded JSON
  response, or a typed error:

    * `{:error, {:refused, error_map}}` — the kernel answered `ok: false`
      (a typed refusal inside the module, e.g. an unknown op).
    * `{:error, {:abi_trap, reason}}` — the export call trapped/failed.
    * `{:error, {:abi_frame, term}}` — malformed response framing
      (null buffer, truncated response, non-UTF-8, non-JSON).
  """
  @spec admit(GenServer.server(), binary()) ::
          {:ok, map()} | {:error, {:refused, map()} | {:abi_trap | :abi_frame, term()}}
  def admit(server \\ __MODULE__, payload) when is_binary(payload) do
    GenServer.call(server, {:admit, payload}, 60_000)
  end

  @doc "Stops the bridge and drops the wasm instance."
  @spec stop(GenServer.server()) :: :ok
  def stop(server \\ __MODULE__) do
    GenServer.stop(server, :normal)
  end

  # ---------------------------------------------------------------------------
  # Callbacks
  # ---------------------------------------------------------------------------

  @impl true
  def init(opts) do
    wasm_path = resolve_wasm_path(opts)

    with {:ok, bytes} <- File.read(wasm_path),
         {:ok, store} <- Wasmex.Store.new_wasi(%Wasmex.Wasi.WasiOptions{}),
         {:ok, module} <- Wasmex.Module.compile(store, bytes),
         {:ok, instance} <- Wasmex.Instance.new(store, module, %{}),
         {:ok, memory} <- Wasmex.Memory.from_instance(store, instance),
         :ok <- check_exports(store, instance) do
      {:ok, %{store: store, module: module, instance: instance, memory: memory, wasm_path: wasm_path, pending: nil}}
    else
      {:error, reason} ->
        Logger.error(
          "GraphlawEx: failed to load wasm artifact #{inspect(wasm_path)}: #{inspect(reason)}"
        )

        {:stop, {:wasm_load_failed, wasm_path, reason}}
    end
  end

  @impl true
  def handle_call({:admit, payload}, from, state) do
    # Packet-4 framing: the 4-byte big-endian length prefix is framing
    # metadata; the payload (whose byte length IS that prefix) is what gets
    # written into exported linear memory. The wasmex NIF replies to `from`
    # when the scheduled wasm call finishes, so each ABI step is driven from
    # handle_info; one admitted op is in flight at a time.
    <<length::unsigned-big-integer-size(32), framed::binary>> = frame(payload)

    :ok =
      Wasmex.Instance.call_exported_function(
        state.store,
        state.instance,
        "gl_alloc",
        [length],
        {self(), from_ref(payload)},
        nil
      )

    {:noreply, %{state | pending: {:alloc, from, framed, length}}}
  end

  @impl true
  def handle_info({ref, {:ok, [ptr]}}, %{pending: {:alloc, from, framed, length}} = state)
      when is_reference(ref) and ptr != 0 do
    :ok = Wasmex.Memory.write_binary(state.store, state.memory, ptr, framed)

    :ok =
      Wasmex.Instance.call_exported_function(
        state.store,
        state.instance,
        "gl_call",
        [ptr, length],
        {self(), ref},
        nil
      )

    {:noreply, %{state | pending: {:call, from, ptr, length}}}
  end

  def handle_info({ref, {:ok, [0]}}, %{pending: {:alloc, _from, _framed, _length}} = state)
      when is_reference(ref) do
    # gl_alloc returned null: oversize request or exhausted outstanding cap.
    {:noreply, complete(state, {:error, {:abi_frame, :alloc_refused}})}
  end

  def handle_info(
        {_ref, {:ok, [packed]}},
        %{pending: {:call, _from, _ptr, _length}} = state
      )
      when is_integer(packed) and packed > 0 do
    out_ptr = Bitwise.bsr(packed, 32)
    out_len = Bitwise.band(packed, 0xFFFFFFFF)

    if out_ptr == 0 or out_len == 0 do
      {:noreply, complete(state, {:error, {:abi_frame, {:empty_response, packed}}})}
    else
      response = Wasmex.Memory.read_binary(state.store, state.memory, out_ptr, out_len)
      :ok = gl_free(state, out_ptr, out_len)
      {:noreply, complete(state, decode(response))}
    end
  end

  def handle_info({_ref, {:error, reason}}, %{pending: {step, _from, _, _}} = state)
      when step in [:alloc, :call] do
    {:noreply, complete(state, {:error, {:abi_trap, reason}})}
  end

  # Any other message (stray replies, monitors): ignore.
  def handle_info(_msg, state), do: {:noreply, state}

  # ---------------------------------------------------------------------------
  # Internals
  # ---------------------------------------------------------------------------

  defp frame(payload) when is_binary(payload), do: <<byte_size(payload)::32, payload::binary>>

  defp from_ref(_payload), do: make_ref()

  defp gl_free(state, ptr, len) do
    # gl_free is a host-cleanup call with no interesting result; the NIF's
    # reply to a throwaway ref is drained by the catch-all handle_info.
    ref = make_ref()

    :ok =
      Wasmex.Instance.call_exported_function(
        state.store,
        state.instance,
        "gl_free",
        [ptr, len],
        {self(), ref},
        nil
      )

    :ok
  end

  defp complete(%{pending: {_step, from, _, _}} = state, reply) do
    GenServer.reply(from, reply)
    %{state | pending: nil}
  end

  defp decode(response) when is_binary(response) do
    with :ok <- utf8?(response),
         {:ok, json} <- Jason.decode(response) do
      case json do
        %{"ok" => true} -> {:ok, json}
        %{"ok" => false, "error" => error} -> {:error, {:refused, error}}
        other -> {:error, {:abi_frame, {:missing_ok_field, other}}}
      end
    else
      {:error, reason} -> {:error, {:abi_frame, reason}}
    end
  end

  defp utf8?(bin), do: if(String.valid?(bin), do: :ok, else: {:error, :non_utf8_response})

  defp check_exports(store, instance) do
    missing = Enum.filter(@abi_exports, fn name ->
      not Wasmex.Instance.function_export_exists(store, instance, name)
    end)

    if missing == [] do
      :ok
    else
      {:error, {:abi_exports_missing, missing}}
    end
  end

  defp resolve_wasm_path(opts) do
    opts[:wasm_path] || Application.get_env(:graphlaw_ex, :wasm_path) || @default_wasm_path
    |> Path.expand()
  end
end
