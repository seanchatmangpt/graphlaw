# SPDX-FileCopyrightText: 2026 Sean Chatman <https://github.com/seanchatmangpt/graphlaw/graphs/contributors>
#
# SPDX-License-Identifier: MIT
# projection target: ontologies/graphlaw-elixir.ttl; hand-written until projected
defmodule GraphlawEx.Telemetry.Catalog do
  @moduledoc """
  Pure-data telemetry catalog: event names, their documented measurement and
  metadata keys, per-event log levels, and per-key redaction policy (`redactAs`).

  No process is started and nothing is emitted here; `GraphlawEx.Telemetry`
  reads this module for emission, logging and redaction.

  ## Per-key `redactAs`

    * `:plain` - declared non-sensitive; only atoms, numbers and short strings
      pass (long binaries become `%{bytes:, sha256:}`; pids, refs and
      containers become `:redacted`).
    * `:span_context` - the span reference; refs pass, anything else is `:redacted`.
    * `:digest` - sensitive; the value collapses to `%{bytes:, sha256:}` (binary),
      `nil`, or `:redacted`.
    * `:server` - atoms and pids pass; everything else is `:redacted`.
    * `:reason`, `:stacktrace` - structural keys with dedicated handling.

  Any key not declared here is default-deny: `redact_as/1` returns `:digest`.
  """

  @prefix [:graphlaw_ex, :admit]

  @typedoc "One catalog entry."
  @type entry :: %{
          event: [atom()],
          description: String.t(),
          measurements: [atom()],
          metadata: [atom()],
          level: Logger.level()
        }

  @typedoc "Per-key redaction policy."
  @type redact_as :: :plain | :digest | :server | :reason | :stacktrace | :span_context

  @max_plain_bytes 128

  @key_policy %{
    # sensitive
    nquads: :digest,
    sparql: :digest,
    sql: :digest,
    query: :digest,
    payload: :digest,
    request: :digest,
    body: :digest,
    data: :digest,
    lease: :digest,
    lease_secret: :digest,
    secret: :digest,
    token: :digest,
    password: :digest,
    # structural
    server: :server,
    reason: :reason,
    stacktrace: :stacktrace,
    telemetry_span_context: :span_context,
    # declared non-sensitive
    op: :plain,
    outcome: :plain,
    refusal_code: :plain,
    code: :plain,
    kind: :plain,
    source_digest: :plain,
    wasm_sha256: :plain,
    duration: :plain,
    system_time: :plain,
    monotonic_time: :plain,
    count: :plain,
    # kill-switch revocation epoch / scope: public document fields, not secrets
    epoch: :plain,
    scope: :plain
  }

  @catalog [
    %{
      event: @prefix ++ [:start],
      description: "An admit call began.",
      measurements: [:system_time, :monotonic_time],
      metadata: [:op, :server, :source_digest, :telemetry_span_context],
      level: :debug
    },
    %{
      event: @prefix ++ [:stop],
      description: "An admit call returned (ok or refused).",
      measurements: [:duration, :monotonic_time],
      metadata: [
        :op,
        :server,
        :source_digest,
        :telemetry_span_context,
        :outcome,
        :refusal_code
      ],
      level: :info
    },
    %{
      event: @prefix ++ [:exception],
      description: "An admit call raised, exited or threw; re-raised after the event.",
      measurements: [:duration, :monotonic_time],
      metadata: [
        :op,
        :server,
        :source_digest,
        :telemetry_span_context,
        :outcome,
        :refusal_code,
        :kind,
        :reason,
        :stacktrace
      ],
      level: :warning
    },
    %{
      event: [:graphlaw_ex, :engine, :admit],
      description: "The wasm engine load receipt.",
      measurements: [],
      metadata: [:outcome, :code, :wasm_sha256],
      level: :debug
    },
    %{
      event: [:graphlaw_ex, :host, :call, :start],
      description: "A host call into the wasm instance began.",
      measurements: [:system_time, :monotonic_time],
      metadata: [:op, :telemetry_span_context],
      level: :debug
    },
    %{
      event: [:graphlaw_ex, :host, :call, :stop],
      description: "A host call into the wasm instance finished.",
      measurements: [:duration, :monotonic_time],
      metadata: [:op, :telemetry_span_context, :outcome, :code],
      level: :debug
    },
    %{
      event: [:graphlaw_ex, :host, :call, :exception],
      description: "A host call raised, exited or threw; re-raised after the event.",
      measurements: [:duration, :monotonic_time],
      metadata: [:op, :telemetry_span_context, :kind, :reason, :stacktrace],
      level: :warning
    },
    %{
      event: [:graphlaw_ex, :host, :recycle],
      description: "The wasm instance was recycled.",
      measurements: [:count],
      metadata: [:reason, :wasm_sha256],
      level: :debug
    },
    %{
      event: [:graphlaw_ex, :kill_switch, :installed],
      description: "A kill switch was installed on a server.",
      measurements: [],
      metadata: [:server, :outcome, :epoch, :scope],
      level: :info
    },
    %{
      event: [:graphlaw_ex, :kill_switch, :refused],
      description: "A call was refused because a kill switch is installed.",
      measurements: [],
      metadata: [:op, :server, :outcome, :code, :epoch, :scope, :reason],
      level: :warning
    }
  ]

  @doc "Every event name `GraphlawEx.Telemetry` emits."
  @spec events() :: [[atom()]]
  def events, do: Enum.map(@catalog, & &1.event)

  @doc """
  The event catalog as data.

      iex> Enum.all?(GraphlawEx.Telemetry.Catalog.catalog(), &match?(%{event: [_ | _], level: _}, &1))
      true
  """
  @spec catalog() :: [entry()]
  def catalog, do: @catalog

  @doc """
  Log level for an event name; `:debug` for events not in the catalog.

      iex> GraphlawEx.Telemetry.Catalog.log_level([:graphlaw_ex, :admit, :exception])
      :warning
      iex> GraphlawEx.Telemetry.Catalog.log_level([:other])
      :debug
  """
  @spec log_level([atom()]) :: Logger.level()
  for %{event: event, level: level} <- @catalog do
    def log_level(unquote(event)), do: unquote(level)
  end

  def log_level(_event), do: :debug

  @doc """
  Declared measurement keys for an event; `nil` for an event not in the catalog.

      iex> GraphlawEx.Telemetry.Catalog.measurements_for([:graphlaw_ex, :host, :recycle])
      [:count]
      iex> GraphlawEx.Telemetry.Catalog.measurements_for([:other])
      nil
  """
  @spec measurements_for([atom()]) :: [atom()] | nil
  for %{event: event, measurements: keys} <- @catalog do
    def measurements_for(unquote(event)), do: unquote(keys)
  end

  def measurements_for(_event), do: nil

  @doc """
  Redaction policy for a metadata key; unknown keys are default-deny (`:digest`).

      iex> GraphlawEx.Telemetry.Catalog.redact_as(:op)
      :plain
      iex> GraphlawEx.Telemetry.Catalog.redact_as(:mystery)
      :digest
  """
  @spec redact_as(term()) :: redact_as()
  def redact_as(key), do: Map.get(@key_policy, key, :digest)

  @doc "Keys whose values always collapse to a summary (`redactAs: :digest`)."
  @spec sensitive_keys() :: [atom()]
  def sensitive_keys do
    for {k, :digest} <- @key_policy, do: k
  end

  @doc "Binaries longer than this many bytes are summarized regardless of key."
  @spec max_plain_bytes() :: pos_integer()
  def max_plain_bytes, do: @max_plain_bytes
end
