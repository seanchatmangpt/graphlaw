# SPDX-FileCopyrightText: 2026 Sean Chatman <https://github.com/seanchatmangpt/graphlaw/graphs/contributors>
#
# SPDX-License-Identifier: MIT
defmodule GraphlawEx.Safety.TelemetryRedactionTest do
  use ExUnit.Case, async: false

  import ExUnit.CaptureLog

  alias GraphlawEx.{Result, Telemetry}

  @canary "CANARY-SECRET-7f3a9c-do-not-leak"

  defp attach(events) do
    parent = self()
    id = "redaction-#{System.unique_integer([:positive])}"

    :telemetry.attach_many(
      id,
      events,
      fn event, m, md, _ -> send(parent, {:evt, event, m, md}) end,
      nil
    )

    on_exit(fn -> :telemetry.detach(id) end)
  end

  defp drain(acc \\ []) do
    receive do
      {:evt, _, _, _} = e -> drain([e | acc])
    after
      0 -> acc
    end
  end

  defp leaks?(term), do: inspect(term, limit: :infinity, printable_limit: :infinity) =~ @canary

  test "synthetic span: canary in opts, raise reason and exit reason never appears" do
    attach(Telemetry.events())
    Telemetry.attach_default_logger()
    on_exit(&Telemetry.detach_default_logger/0)

    log =
      capture_log([level: :debug], fn ->
        Telemetry.admit("admit", [source_digest: String.duplicate(@canary, 20)], fn -> :ok end)

        assert_raise RuntimeError, fn ->
          Telemetry.admit("admit", [], fn -> raise "payload=" <> @canary end)
        end

        catch_exit(Telemetry.admit("admit", [], fn -> exit({:timeout, {:call, @canary}}) end))
      end)

    events = drain()
    assert events != []
    refute leaks?(events)
    refute log =~ @canary
  end

  test "redact/1 collapses sensitive keys, long binaries and stacktrace args" do
    md = %{
      op: "x",
      nquads: @canary,
      lease_secret: @canary,
      other: String.duplicate("a", 500) <> @canary,
      nested: %{q: @canary},
      stacktrace: [{M, :f, [@canary], [line: 1]}]
    }

    out = Telemetry.redact(md)
    refute leaks?(out)
    assert out.op == "x"
    assert %{bytes: n} = out.nquads
    assert n == byte_size(@canary)
  end

  test "S8 canary: scalar secret under an unknown key name is redacted (default-deny)" do
    for key <- [:totally_unknown_key, :zz_payload_9, "string_key_unknown", :api_token_x] do
      out = Telemetry.redact(%{key => @canary})
      refute leaks?(out), "key #{inspect(key)} leaked: #{inspect(out)}"
    end

    short = "s3cr3t-short"
    refute inspect(Telemetry.redact(%{mystery_field: short})) =~ short
  end

  test "Result.from_decoded does not echo a raw request" do
    request = %{"op" => "sparql", "query" => @canary, "lease" => @canary}
    assert {:error, refusal} = Result.from_decoded(request)
    refute leaks?(refusal)
  end

  @tag :wasm
  @tag timeout: 300_000
  test "real engine admit with a canary request leaks nothing to telemetry or logs" do
    path = GraphlawEx.TestEngine.engine_path!()
    attach(Telemetry.events())
    Telemetry.attach_default_logger()
    on_exit(&Telemetry.detach_default_logger/0)

    start_supervised!({GraphlawEx, wasm_path: path, expected_sha256: :unpinned})

    log =
      capture_log([level: :debug], fn ->
        payload = Jason.encode!(%{"op" => "sparql", "query" => @canary, "nquads" => @canary})
        GraphlawEx.admit(payload)
        GraphlawEx.admit("not json " <> @canary)
      end)

    events = drain()
    assert events != []
    refute leaks?(events)
    refute log =~ @canary
  end
end
