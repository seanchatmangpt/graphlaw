# SPDX-FileCopyrightText: 2026 Sean Chatman <https://github.com/seanchatmangpt/graphlaw/graphs/contributors>
#
# SPDX-License-Identifier: MIT
defmodule GraphlawEx.Context do
  @moduledoc """
  Non-authoritative caller context (Ash-agnostic).

  Carries `actor_id`, `tenant`, `trace_id` and free-form `metadata` for
  correlation, logging and telemetry. It carries NO authority: the
  `GraphlawEx.Authority` gate trusts signed leases only and never reads a
  context field to admit or widen an operation. A context claiming
  `actor_id: "root"` grants nothing.

  Authority reaches the gate through a `GraphlawEx.Context.LeaseResolver`,
  which maps a context to a signed lease (or `:none`). The lease is verified
  by the Authority gate regardless of who resolved it.

  ## Wiring snippet (coordinator)

      with {:ok, ctx} <- GraphlawEx.Context.new(actor_id: "u1", tenant: "t1"),
           {:ok, lease} <- resolve_lease(resolver, ctx) do
        Dispatch.call(op, args, lease: lease, telemetry: GraphlawEx.Context.to_telemetry(ctx))
      end

      defp resolve_lease(resolver, ctx) do
        case resolver.resolve(ctx) do
          {:ok, lease} -> {:ok, lease}
          :none -> {:ok, nil}
        end
      end

  ## Examples

      iex> {:ok, ctx} = GraphlawEx.Context.new(actor_id: "a", tenant: "t")
      iex> ctx.actor_id
      "a"
      iex> GraphlawEx.Context.new(actor_id: 1)
      {:error, {:invalid_field, :actor_id}}
  """

  @type t :: %__MODULE__{
          actor_id: String.t() | nil,
          tenant: String.t() | nil,
          trace_id: String.t() | nil,
          metadata: map()
        }

  defstruct actor_id: nil, tenant: nil, trace_id: nil, metadata: %{}

  @string_fields [:actor_id, :tenant, :trace_id]
  @fields @string_fields ++ [:metadata]

  defmodule LeaseResolver do
    @moduledoc """
    Behaviour mapping a `GraphlawEx.Context` to a signed lease map.

    Returning `{:ok, lease}` does not authorize anything by itself; the
    Authority gate verifies the lease signature and scope.
    """

    @callback resolve(GraphlawEx.Context.t()) :: {:ok, map()} | :none
  end

  @doc """
  Builds a context from a keyword list or map. Unknown keys and wrongly typed
  fields are refused.

      iex> GraphlawEx.Context.new(%{bogus: 1})
      {:error, {:unknown_field, :bogus}}
      iex> GraphlawEx.Context.new(metadata: [])
      {:error, {:invalid_field, :metadata}}
  """
  @spec new(keyword() | map()) :: {:ok, t()} | {:error, {:unknown_field | :invalid_field, term()}}
  def new(attrs \\ [])

  def new(attrs) when is_list(attrs) or is_map(attrs) do
    attrs |> Enum.to_list() |> validate(%__MODULE__{})
  end

  defp validate([], acc), do: {:ok, acc}

  defp validate([{k, v} | rest], acc) when k in @string_fields do
    if is_nil(v) or is_binary(v),
      do: validate(rest, Map.put(acc, k, v)),
      else: {:error, {:invalid_field, k}}
  end

  defp validate([{:metadata, v} | rest], acc) do
    if is_map(v),
      do: validate(rest, %{acc | metadata: v}),
      else: {:error, {:invalid_field, :metadata}}
  end

  defp validate([{k, _} | _], _acc) when k not in @fields, do: {:error, {:unknown_field, k}}
  defp validate([other | _], _), do: {:error, {:unknown_field, other}}

  @doc """
  Merges `overrides` into `ctx`. A field PRESENT in `overrides` wins, including an
  explicit `nil`, which clears it; an absent field is kept. Metadata is merged
  key-wise.

      iex> {:ok, a} = GraphlawEx.Context.new(actor_id: "a", metadata: %{x: 1})
      iex> {:ok, m} = GraphlawEx.Context.merge(a, tenant: "t", metadata: %{y: 2})
      iex> {m.actor_id, m.tenant, m.metadata}
      {"a", "t", %{x: 1, y: 2}}
      iex> {:ok, c} = GraphlawEx.Context.merge(a, actor_id: nil)
      iex> c.actor_id
      nil
  """
  @spec merge(t(), keyword() | map()) :: {:ok, t()} | {:error, term()}
  def merge(%__MODULE__{} = ctx, overrides) do
    with {:ok, o} <- new(overrides) do
      present = overrides |> Enum.map(fn {k, _} -> k end) |> MapSet.new()

      merged =
        Enum.reduce(@string_fields, ctx, fn f, acc ->
          if MapSet.member?(present, f), do: Map.put(acc, f, Map.fetch!(o, f)), else: acc
        end)

      {:ok, %{merged | metadata: Map.merge(ctx.metadata, o.metadata)}}
    end
  end

  @doc """
  Redact-safe telemetry metadata. Each identity field is plain only if
  `GraphlawEx.Telemetry.Catalog.redact_as/1` says `:plain` for its key
  (default-deny: digest); metadata contributes only its key count.

      iex> {:ok, c} = GraphlawEx.Context.new(actor_id: "secret")
      iex> t = GraphlawEx.Context.to_telemetry(c)
      iex> String.starts_with?(t.actor_id, "sha256:") and t.metadata_keys == 0
      true
  """
  @spec to_telemetry(t()) :: map()
  def to_telemetry(%__MODULE__{} = ctx) do
    base =
      for f <- @string_fields,
          v = Map.fetch!(ctx, f),
          into: %{metadata_keys: map_size(ctx.metadata)} do
        {f, redact(f, v)}
      end

    base
  end

  defp redact(key, value) do
    case GraphlawEx.Telemetry.Catalog.redact_as(key) do
      :plain ->
        value

      _ ->
        # Slice the raw digest to 8 bytes before encoding: 16 hex chars, no 64-char intermediate.
        <<head::binary-size(8), _::binary>> = :crypto.hash(:sha256, value)
        "sha256:" <> Base.encode16(head, case: :lower)
    end
  end
end

defimpl Inspect, for: GraphlawEx.Context do
  # actor_id and metadata can carry identity or request content; only the
  # non-identifying shape reaches logs.
  def inspect(%GraphlawEx.Context{} = ctx, _opts) do
    "#GraphlawEx.Context<tenant: #{inspect(ctx.tenant)}, trace_id: #{inspect(ctx.trace_id)}, " <>
      "metadata_keys: #{map_size(ctx.metadata)}>"
  end
end
