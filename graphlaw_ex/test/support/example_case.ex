# SPDX-FileCopyrightText: 2026 Sean Chatman <https://github.com/seanchatmangpt/graphlaw/graphs/contributors>
#
# SPDX-License-Identifier: MIT
defmodule GraphlawEx.ExampleCase do
  @moduledoc """
  Shared case module for the registry op-examples court.

  Engine gate (one place, never per test body):

    * `GRAPHLAW_REQUIRE_ENGINE=1` with a missing wasm: `setup_all` FAILS.
    * no gate with a missing wasm: tests carry `skip: skip_reason()` so they
      are reported skipped, never passed with zero assertions.

  Tests built on this case never wrap their body in an engine-present branch.
  """

  use ExUnit.CaseTemplate

  alias GraphlawEx.Capability.API
  alias GraphlawEx.Capability.Refusal
  alias GraphlawEx.Capability.Registry

  using do
    quote do
      import GraphlawEx.ExampleCase,
        only: [
          skip_reason: 0,
          execute: 1,
          canonical_id: 1,
          assert_ok_shape: 2,
          assert_refusal: 2,
          assert_divergence: 2,
          divergences: 0
        ]
    end
  end

  setup_all do
    path = GraphlawEx.TestEngine.wasm_path()

    cond do
      File.regular?(path) ->
        start_supervised!({GraphlawEx, wasm_path: path, expected_sha256: :unpinned})
        :ok

      GraphlawEx.ExampleCase.require_engine?() ->
        flunk("GRAPHLAW_REQUIRE_ENGINE=1: wasm artifact not found at #{path}")

      true ->
        # Every test of the using module is tagged skip with a reason.
        :ok
    end
  end

  @doc "Compile-time `@tag skip:` value: `false` to run, a reason string to skip loudly."
  @spec skip_reason() :: false | String.t()
  def skip_reason do
    path = GraphlawEx.TestEngine.wasm_path()

    if File.regular?(path) or require_engine?() do
      false
    else
      "op-examples court: wasm artifact not found at #{path} (set PRIV_WASM or GRAPHLAW_WASM_PATH)"
    end
  end

  @doc false
  def require_engine?, do: System.get_env("GRAPHLAW_REQUIRE_ENGINE") == "1"

  @doc "Runs one example through `GraphlawEx.Capability.API.run/3` on the real engine."
  @spec execute(map()) :: {:ok, term()} | {:error, Refusal.t()}
  def execute(%{"op" => op, "request" => request}) do
    args = Map.delete(request, "op")

    opts =
      []
      |> put_opt(:lease, request["lease"])
      |> put_opt(:trusted_keys, request["trusted_keys"])

    API.run(op, args, opts)
  end

  defp put_opt(opts, _key, nil), do: opts
  defp put_opt(opts, key, value), do: Keyword.put(opts, key, value)

  @doc "Canonical content id of a data spec (`%{\"text\" => ..., \"dialect\" => ...}`)."
  @spec canonical_id(map()) :: String.t()
  def canonical_id(data_spec) do
    case API.run("canonical", %{"data" => data_spec}) do
      {:ok, result} ->
        result.id

      {:error, refusal} ->
        ExUnit.Assertions.flunk(
          "canonical refused for #{inspect(data_spec)}: #{inspect(refusal)}"
        )
    end
  end

  @doc """
  Asserts an ok result carries every required, non-nullable response field of
  the registry op (for tagged responses, of the variant the result selects).
  """
  def assert_ok_shape(%{"op" => op_name} = example, result) do
    {:ok, op} = Registry.op(op_name)
    variant = select_variant(op, result)

    ExUnit.Assertions.assert(
      variant != nil,
      "#{example["id"]}: no registry response variant matches #{inspect(result)}"
    )

    for field <- variant.fields, field.required do
      key = String.to_existing_atom(field.name)

      ExUnit.Assertions.assert(
        Map.has_key?(result, key),
        "#{example["id"]}: result lacks required key #{field.name}"
      )

      if not field.nullable do
        ExUnit.Assertions.refute(
          is_nil(Map.fetch!(result, key)),
          "#{example["id"]}: required key #{field.name} is nil"
        )
      end
    end

    :ok
  end

  defp select_variant(%{responses: [only]}, _result), do: only

  defp select_variant(%{responses: variants}, result) do
    tag_field = variants |> hd() |> Map.fetch!(:tag_field) |> String.to_existing_atom()
    tag = result |> Map.get(tag_field) |> to_string()
    Enum.find(variants, &(&1.tag == tag))
  end

  @doc """
  Asserts a refused example refuses with exactly the registry kind (and code
  when the example names one).

  Engine refusals carry the wire envelope in `raw`: `kind` is compared with
  the example's `refusal_kind` and the code atom with the shared engine-code
  table. A refusal raised before the engine is called (`raw == nil`) is the
  typed client-side form of an unsupported request: it must be one of the
  layer's client codes and the example must expect kind `Unsupported`.
  """
  def assert_refusal(example, {:error, %Refusal{} = refusal}) do
    id = example["id"]
    kind = example["refusal_kind"]
    code = example["refusal_code"]

    case refusal.raw do
      %{} = raw ->
        ExUnit.Assertions.assert(
          raw["kind"] == kind,
          "#{id}: expected kind #{kind}, got #{inspect(raw["kind"])} (#{inspect(refusal)})"
        )

        if code do
          expected = GraphlawEx.Refusal.from_engine_code(code)

          ExUnit.Assertions.refute(
            expected == :engine_unclassified,
            "#{id}: registry code #{code} is not in the engine code table"
          )

          ExUnit.Assertions.assert(
            refusal.code == expected,
            "#{id}: expected code #{inspect(expected)} (#{code}), got #{inspect(refusal.code)}"
          )
        end

      nil ->
        ExUnit.Assertions.assert(
          refusal.code in Refusal.client_codes(),
          "#{id}: refusal without an engine envelope must be a client code, got #{inspect(refusal.code)}"
        )

        ExUnit.Assertions.assert(
          kind == "Unsupported" and is_nil(code),
          "#{id}: client-side refusal #{inspect(refusal.code)} cannot satisfy kind #{kind} code #{inspect(code)}"
        )
    end

    :ok
  end

  def assert_refusal(example, other) do
    ExUnit.Assertions.flunk("#{example["id"]}: expected a refusal, got #{inspect(other)}")
  end

  # Registered divergences between the registry examples (written against the
  # raw kernel ABI) and the Elixir layer's authority design. Each row pins the
  # lawful Elixir outcome instead of skipping the example.
  #
  # * unsigned-lease examples: `GraphlawEx.Authority` never forwards an
  #   unverified lease (`lease`/`unverified_lease`/`now_unix` are stripped by
  #   Dispatch), so the kernel sees a lease-less chain, which is kernel-legal
  #   and admitted with no `lease_id` on its receipts.
  # * canonical.string-data-refused: `Capability.Coerce` documents a bare
  #   binary for a `data_spec` field as `%{"text" => binary}`; the kernel's
  #   refusal of a raw string is a wire-level shape the typed layer lifts.
  @divergences %{
    "law.lease-expired-refused" => :lease_less_admitted,
    "law.lease-ceiling-refused" => :lease_less_admitted,
    "law.unverified-lease-refused" => :lease_less_admitted,
    "canonical.string-data-refused" => :string_data_coerced
  }

  @doc "Example ids whose registry outcome is a registered Elixir-layer divergence."
  @spec divergences() :: %{String.t() => atom()}
  def divergences, do: @divergences

  @doc "Asserts the pinned Elixir-layer outcome of a registered divergence."
  def assert_divergence(example, result) do
    id = example["id"]

    case {Map.fetch!(@divergences, id), result} do
      {:lease_less_admitted, {:ok, admitted}} ->
        ExUnit.Assertions.assert(
          admitted.receipts != [] and Enum.all?(admitted.receipts, &is_nil(&1.lease_id)),
          "#{id}: lease-less admission must not carry a lease_id"
        )

      {:string_data_coerced, {:ok, canonical}} ->
        ExUnit.Assertions.assert(String.starts_with?(canonical.id, "blake3:"), id)

      {pin, other} ->
        ExUnit.Assertions.flunk("#{id}: divergence #{pin} expected ok, got #{inspect(other)}")
    end

    :ok
  end
end
