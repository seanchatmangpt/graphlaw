defmodule GraphlawEx.Telemetry do
  @moduledoc """
  Telemetry event names and the span around every admit call.

  `admit/3` emits, for the op `op`:

    * `[:graphlaw_ex, :admit, :start]` - measurements `%{system_time: integer}`,
      metadata `%{op: String.t(), server: term, source_digest: String.t() | nil}`.
    * `[:graphlaw_ex, :admit, :stop]` - measurements `%{duration: integer}`
      (native time units), metadata `%{op:, server:, source_digest:,
      outcome: :ok | :refused, refusal_code: atom | nil}`. `{:ok, _}` is
      `:ok`; `{:error, %GraphlawEx.Refusal{}}` is `:refused` with the
      refusal code. Any other `{:error, _}` is `:refused` with a nil code.
    * `[:graphlaw_ex, :admit, :exception]` - measurements `%{duration: integer}`,
      metadata `%{op:, server:, outcome: :exception, refusal_code: nil, kind:,
      reason:, stacktrace:}`. The exception is re-raised after the event.

  `events/0` also lists the load-time receipt event
  `[:graphlaw_ex, :engine, :admit]`, emitted by `GraphlawEx.EngineLoad` with the
  wasm digest (`wasm_sha256`) — the load receipt.

  `source_digest` is the caller-supplied identity of the law CONTENT the span
  executes: the engine-minted `blake3:` of the RDFC-1.0 canonical N-Quads
  (`LawState.id`, GL28-1 leg b / Art. 12). It is read from the admit opts and
  is `nil` when the caller omits it. Telemetry only OBSERVES: a nil never
  blocks emission here — enforcing the digest's presence is the consumer's
  law (the beam4pm bridge ingest refuses an event whose
  `attributes["pe:sourceDigest"]` is missing).

  ## Attaching a handler

      :telemetry.attach_many(
        "my-app-graphlaw",
        GraphlawEx.Telemetry.events(),
        fn event, measurements, metadata, _config -> ... end,
        nil
      )

  A telemetry observation is never authority; it records what happened.
  """

  alias GraphlawEx.Refusal

  @prefix [:graphlaw_ex, :admit]

  @doc "Every event name this library emits."
  @spec events() :: [[atom()]]
  def events do
    [
      @prefix ++ [:start],
      @prefix ++ [:stop],
      @prefix ++ [:exception],
      [:graphlaw_ex, :engine, :admit],
      [:graphlaw_ex, :host, :call, :stop],
      [:graphlaw_ex, :host, :recycle]
    ]
  end

  @doc """
  Runs `fun` inside an admit span for `op` and returns its result unchanged.
  """
  @spec admit(String.t(), keyword(), (-> result)) :: result when result: term()
  def admit(op, opts, fun) when is_binary(op) and is_list(opts) and is_function(fun, 0) do
    server = Keyword.get(opts, :server, GraphlawEx)

    # GL28-1 leg (b): the caller may pin the identity of the law content this
    # span executes (engine-minted `blake3:` LawState.id). Absent opt -> nil;
    # the metadata shape never changes, and a nil here is an observation, not
    # an enforcement point.
    base = %{op: op, server: server, source_digest: Keyword.get(opts, :source_digest, nil)}
    started = System.monotonic_time()

    :telemetry.execute(@prefix ++ [:start], %{system_time: System.system_time()}, base)

    try do
      result = fun.()
      {outcome, code} = classify(result)

      :telemetry.execute(
        @prefix ++ [:stop],
        %{duration: System.monotonic_time() - started},
        Map.merge(base, %{outcome: outcome, refusal_code: code})
      )

      result
    catch
      kind, reason ->
        stacktrace = __STACKTRACE__

        :telemetry.execute(
          @prefix ++ [:exception],
          %{duration: System.monotonic_time() - started},
          Map.merge(base, %{
            outcome: :exception,
            refusal_code: nil,
            kind: kind,
            reason: reason,
            stacktrace: stacktrace
          })
        )

        :erlang.raise(kind, reason, stacktrace)
    end
  end

  defp classify({:ok, _}), do: {:ok, nil}
  defp classify({:error, %Refusal{code: code}}), do: {:refused, code}
  defp classify({:error, _}), do: {:refused, nil}
  defp classify(_), do: {:ok, nil}
end
