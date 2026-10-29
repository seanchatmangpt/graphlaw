defmodule GraphlawEx.TelemetryTest do
  use ExUnit.Case, async: true

  # Courts for the telemetry surface: attach a real handler over
  # :telemetry and assert the emitted measurements and span metadata.
  # No engine needed for the span itself; the real-artifact span is
  # exercised through GraphlawEx.admit/2 when PRIV_WASM is present.

  alias GraphlawEx.Refusal
  alias GraphlawEx.Telemetry

  defmodule Collector do
    @doc "Attaches a real telemetry handler collecting `event -> [metadata]`."
    def attach(id, events) do
      collector = :ets.new(nil, [:bag, :public])
      on_exit(fn -> :telemetry.detach(id) end)

      :ok =
        :telemetry.attach_many(
          id,
          events,
          fn event, measurements, metadata, _config ->
            # Telemetry handlers are VM-global and the suite runs tests
            # concurrently: scope this collector to the facade's own span
            # (`op: "admit"` / host `op: :admit`) and op-less engine events,
            # so another test's capability span can never land in the table.
            keep? =
              not is_map_key(metadata, :op) or metadata.op in [:admit, "admit"]

            if keep? do
              :ets.insert(collector, {List.to_tuple(event), Map.new(measurements), Map.new(metadata)})
            end
          end,
          nil
        )

      collector
    end

    def take(collector, event) do
      :ets.lookup(collector, List.to_tuple(event))
    end
  end

  describe "admit span" do
    test "start + stop: {:ok, _} is outcome :ok with a nil refusal code and a duration" do
      collector = Collector.attach(:"admit-ok-#{System.unique_integer()}", Telemetry.events())

      assert {:ok, :ran} =
               Telemetry.admit("admit", [server: :court], fn -> {:ok, :ran} end)

      assert [_start] = Collector.take(collector, [:graphlaw_ex, :admit, :start])
      assert {_e, %{system_time: t}, %{op: "admit", server: :court}} = hd(Collector.take(collector, [:graphlaw_ex, :admit, :start]))
      assert is_integer(t)

      assert [_stop] = Collector.take(collector, [:graphlaw_ex, :admit, :stop])
      assert {_e, %{duration: d}, %{op: "admit", server: :court, outcome: :ok, refusal_code: nil}} =
               hd(Collector.take(collector, [:graphlaw_ex, :admit, :stop]))

      assert is_integer(d) and d >= 0
    end

    test "stop: a typed refusal is outcome :refused carrying its code" do
      collector = Collector.attach(:"admit-refused-#{System.unique_integer()}", Telemetry.events())
      refusal = Refusal.new(:call_trapped, "court refusal", %{"step" => :court})

      assert {:error, %Refusal{}} = Telemetry.admit("admit", [], fn -> {:error, refusal} end)

      assert {_e, _, %{outcome: :refused, refusal_code: :call_trapped, op: "admit"}} =
               hd(Collector.take(collector, [:graphlaw_ex, :admit, :stop]))
    end

    test "stop: an untyped {:error, _} is :refused with a nil code" do
      collector = Collector.attach(:"admit-plain-#{System.unique_integer()}", Telemetry.events())

      assert {:error, :boom} = Telemetry.admit("admit", [], fn -> {:error, :boom} end)

      assert {_e, _, %{outcome: :refused, refusal_code: nil}} =
               hd(Collector.take(collector, [:graphlaw_ex, :admit, :stop]))
    end

    test "exception: the event carries kind/reason/stacktrace, then the raise propagates" do
      collector = Collector.attach(:"admit-raise-#{System.unique_integer()}", Telemetry.events())

      assert_raise RuntimeError, "court boom", fn ->
        Telemetry.admit("admit", [], fn -> raise "court boom" end)
      end

      assert {_e, %{duration: d},
              %{
                outcome: :exception,
                refusal_code: nil,
                kind: :error,
                reason: %RuntimeError{message: "court boom"},
                stacktrace: stacktrace
              }} = hd(Collector.take(collector, [:graphlaw_ex, :admit, :exception]))

      assert is_integer(d) and d >= 0
      assert is_list(stacktrace) and stacktrace != []
    end

    test "events/0 lists the engine receipt and the host events" do
      events = Telemetry.events()
      assert [:graphlaw_ex, :engine, :admit] in events
      assert [:graphlaw_ex, :host, :call, :stop] in events
      assert [:graphlaw_ex, :host, :recycle] in events
    end
  end

  describe "end-to-end over the real engine" do
    @tag :wasm
    @tag timeout: 300_000
    test "a real admit through the facade emits the full span" do
      path = System.get_env("PRIV_WASM") || Path.expand("priv/graphlaw/graphlaw.wasm", __DIR__)

      if File.regular?(path) do
        collector = Collector.attach(:"admit-real-#{System.unique_integer()}", Telemetry.events())

        start_supervised!({GraphlawEx, wasm_path: path, expected_sha256: :unpinned})
        assert GraphlawEx.Host.available?(GraphlawEx)

        assert {:ok, %{"ok" => true} = response} =
                 GraphlawEx.admit(Jason.encode!(%{"op" => "capabilities"}))

        # If the engine ever refuses capabilities here the envelope itself is
        # the diagnosis; the span assertion below is meaningless without it.
        refute match?(%{"ok" => false, "error" => _}, response)

        assert hd(Collector.take(collector, [:graphlaw_ex, :admit, :start])) != []
        assert {_e, _, %{outcome: :ok}} = hd(Collector.take(collector, [:graphlaw_ex, :admit, :stop]))

        # The host call event carries op/outcome metadata too.
        assert {_e, %{duration: _}, %{op: :admit, outcome: :ok}} =
                 hd(Collector.take(collector, [:graphlaw_ex, :host, :call, :stop]))
      else
        message =
          "[SKIP] telemetry real-engine court: no wasm artifact at #{path} (set PRIV_WASM). A loud skip is not a pass."

        if System.get_env("GRAPHLAW_REQUIRE_ENGINE") == "1", do: flunk(message)
        IO.puts(message)
      end
    end
  end
end
