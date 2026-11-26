# SPDX-FileCopyrightText: 2026 Sean Chatman <https://github.com/seanchatmangpt/graphlaw/graphs/contributors>
#
# SPDX-License-Identifier: MIT
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
      refusal code; `%GraphlawEx.Error{}` and `%Capability.Refusal{}` likewise
      carry their `code`. Any other `{:error, _}`, `:error` or unexpected
      shape is `:refused` with a nil code.
    * `[:graphlaw_ex, :admit, :exception]` - measurements `%{duration: integer}`,
      metadata `%{op:, server:, source_digest:, outcome: :exception,
      refusal_code: nil, kind:, reason:, stacktrace:}`. The exception is re-raised after the event.

  Each admit event also carries `telemetry_span_context` (one reference per
  span, as in `:telemetry.span/3`); start/stop measurements add `monotonic_time`.
  All events are emitted through `emit/3`, which redacts metadata and filters
  measurements to the catalog's declared numeric keys. Host calls emit
  `[:graphlaw_ex, :host, :call, :start | :stop | :exception]` via `span/4`.

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

  ## Catalog and dev logging

  `catalog/0` returns every event as data (name, description, measurement keys,
  metadata keys). `attach_default_logger/0` logs them through `Logger` while
  developing; `detach_default_logger/0` removes it. Host events:
  `[:graphlaw_ex, :host, :call, :stop]` (`%{duration}`; `%{op:, outcome:, code:}`)
  and `[:graphlaw_ex, :host, :recycle]` (`%{count}`; `%{reason:, wasm_sha256:}`).

  A telemetry observation is never authority; it records what happened.
  """

  require Logger

  # projection target: ontologies/graphlaw-elixir.ttl; hand-written until projected
  alias GraphlawEx.Refusal
  alias GraphlawEx.Telemetry.Catalog

  @prefix [:graphlaw_ex, :admit]
  @logger_id "graphlaw-ex-default-logger"

  @typedoc "One catalog entry: an event name with its documented measurement and metadata keys."
  @type entry :: Catalog.entry()

  @doc """
  The event catalog as data: every event this library emits, with the
  measurement and metadata keys it carries.

  Useful for wiring metrics reporters without a Telemetry.Metrics dependency.

      iex> entries = GraphlawEx.Telemetry.catalog()
      iex> Enum.all?(entries, &match?(%{event: [_ | _], measurements: _, metadata: _}, &1))
      true
  """
  @spec catalog() :: [entry()]
  def catalog, do: Catalog.catalog()

  @doc "Every event name this library emits (derived from `catalog/0`)."
  @spec events() :: [[atom()]]
  def events, do: Catalog.events()

  @doc """
  Attaches a handler that logs every event in `events/0` via `Logger` (for dev).

  Admit `:stop` and `:exception` log at `:info` and `:warning` respectively; all
  other events log at `:debug`. Returns `:ok`, or `{:error, :already_exists}` if
  already attached. Undo with `detach_default_logger/0`.
  """
  @spec attach_default_logger() :: :ok | {:error, :already_exists}
  def attach_default_logger do
    :telemetry.attach_many(@logger_id, events(), &__MODULE__.handle_event/4, nil)
  end

  @doc "Detaches the handler installed by `attach_default_logger/0`."
  @spec detach_default_logger() :: :ok | {:error, :not_found}
  def detach_default_logger, do: :telemetry.detach(@logger_id)

  @sensitive_keys Catalog.sensitive_keys()
  @max_plain_bytes Catalog.max_plain_bytes()

  @doc """
  Redacts event metadata so it carries only sizes, digests, op names and codes.

  Sensitive keys (request text, lease secrets) and any binary longer than 128
  bytes become `%{bytes: n, sha256: hex}`; exception reasons collapse to a type
  descriptor; stacktrace frames lose their argument lists; non-scalar values
  collapse to `:redacted`. Default-deny: a key the catalog does not declare
  non-sensitive redacts even a scalar value. Never raises.

      iex> redacted = GraphlawEx.Telemetry.redact(%{op: "admit", nquads: "<a> <b> <c> ."})
      iex> {redacted.op, redacted.nquads.bytes}
      {"admit", 13}
  """
  @spec redact(term()) :: term()
  def redact(metadata) when is_map(metadata) and not is_struct(metadata) do
    Map.new(metadata, fn {k, v} -> {k, redact_value(k, v)} end)
  end

  def redact(_other), do: :redacted

  defp redact_value(k, v), do: redact_value(Catalog.redact_as(k), k, v)

  defp redact_value(:server, _k, v) when is_atom(v) or is_pid(v), do: v
  defp redact_value(:server, _k, _v), do: :redacted
  defp redact_value(:span_context, _k, v) when is_reference(v), do: v
  defp redact_value(:span_context, _k, _v), do: :redacted
  defp redact_value(:reason, _k, v), do: reason_summary(v)
  defp redact_value(:stacktrace, _k, v) when is_list(v), do: Enum.map(v, &strip_frame/1)
  defp redact_value(:stacktrace, _k, _), do: []

  defp redact_value(:plain, _k, v) when is_binary(v) and byte_size(v) > @max_plain_bytes,
    do: summarize(v)

  defp redact_value(:plain, _k, v) when is_atom(v) or is_number(v), do: v

  defp redact_value(:plain, _k, v) when is_binary(v),
    do: if(String.valid?(v), do: v, else: summarize(v))

  defp redact_value(:plain, _k, _v), do: :redacted

  # :digest covers catalog-sensitive keys and, default-deny, every undeclared key:
  # scalars never pass in the clear. pids/references carry no content and pass
  # under undeclared keys only.
  defp redact_value(:digest, k, v) when is_pid(v) or is_reference(v) do
    if k in @sensitive_keys, do: :redacted, else: v
  end

  defp redact_value(:digest, _k, v), do: summarize(v)

  # Idempotent: an already-summarized value is not re-collapsed.
  defp summarize(%{bytes: b, sha256: h} = v)
       when is_integer(b) and is_binary(h) and map_size(v) == 2,
       do: v

  defp summarize(v) when is_binary(v),
    do: %{bytes: byte_size(v), sha256: Base.encode16(:crypto.hash(:sha256, v), case: :lower)}

  defp summarize(nil), do: nil
  defp summarize(_), do: :redacted

  defp reason_summary(%{__exception__: true, __struct__: mod}), do: %{exception: mod}
  # Idempotent: span/4 summarizes the reason, then emit/3 redacts it again.
  defp reason_summary(%{exception: mod} = v) when is_atom(mod) and map_size(v) == 1, do: v
  defp reason_summary(%{tag: tag} = v) when is_atom(tag) and map_size(v) == 1, do: v
  defp reason_summary(v) when is_atom(v), do: v
  defp reason_summary({tag, _}) when is_atom(tag), do: %{tag: tag}
  defp reason_summary({tag, _, _}) when is_atom(tag), do: %{tag: tag}
  defp reason_summary(_), do: :redacted

  defp strip_frame({m, f, args, loc}) when is_list(args), do: {m, f, length(args), loc}
  defp strip_frame({m, f, args}) when is_list(args), do: {m, f, length(args)}
  defp strip_frame(frame) when is_tuple(frame), do: frame
  defp strip_frame(_), do: :redacted

  defp drop_stacktrace(m) when is_map(m), do: Map.drop(m, [:stacktrace])
  defp drop_stacktrace(other), do: other

  defp safe_digest(d) when is_binary(d) and byte_size(d) <= @max_plain_bytes, do: d
  defp safe_digest(_), do: nil

  @doc false
  @spec handle_event([atom()], map(), map(), term()) :: :ok
  def handle_event(event, measurements, metadata, _config) do
    Logger.log(Catalog.log_level(event), fn ->
      "[graphlaw_ex] #{Enum.join(event, ".")} " <>
        "measurements=#{inspect(measurements)} " <>
        "metadata=#{inspect(metadata |> redact() |> drop_stacktrace())}"
    end)
  end

  @doc """
  The single emit point: redacts `metadata` (`redact/1`) and filters
  `measurements` to the numeric keys the catalog declares for `event` (numeric
  keys only for an event the catalog does not list), then executes the event.
  Every emitter in this library goes through here, so nothing unredacted
  reaches a handler.

      iex> ref = make_ref()
      iex> id = "emit-doctest-" <> inspect(ref)
      iex> me = self()
      iex> :telemetry.attach(id, [:graphlaw_ex, :host, :recycle], fn _e, m, md, _ -> send(me, {ref, m, md}) end, nil)
      iex> GraphlawEx.Telemetry.emit([:graphlaw_ex, :host, :recycle], %{count: 1, junk: 2}, %{reason: :x, nquads: "<a> <b> <c> ."})
      iex> receive do {^ref, m, md} -> {m, md.reason, md.nquads.bytes} end
      {%{count: 1}, :x, 13}
      iex> :telemetry.detach(id)
      :ok
  """
  @spec emit([atom()], map(), map()) :: :ok
  def emit(event, measurements, metadata) when is_list(event) do
    # Redaction is the one boundary: callers pass raw metadata, redact runs here
    # once. With no handler attached `:telemetry.execute/3` is a no-op, so the
    # redact/filter work is skipped entirely.
    case :telemetry.list_handlers(event) do
      [] ->
        :ok

      _handlers ->
        :telemetry.execute(
          event,
          filter_measurements(event, measurements),
          safe_metadata(metadata)
        )
    end
  end

  defp safe_metadata(m) when is_map(m) and not is_struct(m), do: redact(m)
  defp safe_metadata(_), do: %{}

  defp filter_measurements(event, m) when is_map(m) and not is_struct(m) do
    allowed = Catalog.measurements_for(event)

    Map.filter(m, fn {k, v} -> is_number(v) and (is_nil(allowed) or k in allowed) end)
  end

  defp filter_measurements(_event, _m), do: %{}

  @doc """
  Runs `fun` inside a span named by `prefix`, emitting `prefix ++ [:start]`,
  `[:stop]` or `[:exception]` through `emit/3` (the `:telemetry.span/3` event
  contract: `system_time`/`monotonic_time` on start, `duration`/`monotonic_time`
  on stop and exception, and a `telemetry_span_context` reference shared by all
  events of one span in the metadata). `fun` returns `{result, stop_metadata}`;
  the result is returned unchanged. On a raise/exit/throw the exception event
  carries `kind`, a type-only `reason` and an argument-stripped `stacktrace`
  (plus `exception_meta`), then the original exception is re-raised.
  """
  @spec span([atom()], map(), (-> {result, map()}), map()) :: result when result: term()
  def span(prefix, start_meta, fun, exception_meta \\ %{})
      when is_list(prefix) and is_map(start_meta) and is_function(fun, 0) do
    base = Map.put(start_meta, :telemetry_span_context, make_ref())
    started = System.monotonic_time()

    emit(prefix ++ [:start], %{system_time: System.system_time(), monotonic_time: started}, base)

    try do
      {result, stop_meta} = fun.()
      now = System.monotonic_time()

      emit(
        prefix ++ [:stop],
        %{duration: now - started, monotonic_time: now},
        Map.merge(base, stop_meta)
      )

      result
    catch
      kind, reason ->
        stacktrace = __STACKTRACE__
        now = System.monotonic_time()

        emit(
          prefix ++ [:exception],
          %{duration: now - started, monotonic_time: now},
          base
          |> Map.merge(exception_meta)
          |> Map.merge(%{
            kind: kind,
            reason: reason,
            stacktrace: stacktrace
          })
        )

        :erlang.raise(kind, reason, stacktrace)
    end
  end

  @doc """
  Runs `fun` inside an admit span for `op` and returns its result unchanged.
  Built on `span/4`, so the three admit events share a `telemetry_span_context`.
  """
  @spec admit(String.t(), keyword(), (-> result)) :: result when result: term()
  def admit(op, opts, fun) when is_binary(op) and is_list(opts) and is_function(fun, 0) do
    server = Keyword.get(opts, :server, GraphlawEx)

    # GL28-1 leg (b): the caller may pin the identity of the law content this
    # span executes (engine-minted `blake3:` LawState.id). Absent opt -> nil;
    # the metadata shape never changes, and a nil here is an observation, not
    # an enforcement point.
    base = %{
      op: op,
      server: server,
      source_digest: safe_digest(Keyword.get(opts, :source_digest, nil))
    }

    span(
      @prefix,
      base,
      fn ->
        result = fun.()
        {outcome, code} = classify(result)
        {result, %{outcome: outcome, refusal_code: code}}
      end,
      %{outcome: :exception, refusal_code: nil}
    )
  end

  defp classify(:ok), do: {:ok, nil}
  defp classify({:ok, _}), do: {:ok, nil}
  defp classify({:ok, _, _}), do: {:ok, nil}
  defp classify({:error, %Refusal{code: code}}), do: {:refused, code}
  defp classify({:error, %GraphlawEx.Error{code: code}}), do: {:refused, code}
  defp classify({:error, %GraphlawEx.Capability.Refusal{code: code}}), do: {:refused, code}
  defp classify({:error, _}), do: {:refused, nil}
  defp classify({:error, _, _}), do: {:refused, nil}
  defp classify(:error), do: {:refused, nil}
  # Unexpected shapes are not admissions.
  defp classify(_), do: {:refused, nil}
end
