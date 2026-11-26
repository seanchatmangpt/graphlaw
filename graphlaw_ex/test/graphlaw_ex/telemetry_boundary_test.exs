# SPDX-FileCopyrightText: 2026 Sean Chatman <https://github.com/seanchatmangpt/graphlaw/graphs/contributors>
#
# SPDX-License-Identifier: MIT
defmodule GraphlawEx.TelemetryBoundaryTest do
  # async: false - telemetry handlers are VM-global.
  use ExUnit.Case, async: false

  alias GraphlawEx.Introspection.Tables
  alias GraphlawEx.Telemetry
  alias GraphlawEx.Telemetry.Catalog

  doctest GraphlawEx.Introspection.Tables

  defp collect(id, event) do
    me = self()
    ref = make_ref()
    on_exit(fn -> :telemetry.detach(id) end)

    :ok =
      :telemetry.attach(id, event, fn _e, m, md, _ -> send(me, {ref, m, md}) end, nil)

    ref
  end

  describe "catalog" do
    test "kill_switch events are cataloged with levels" do
      installed = [:graphlaw_ex, :kill_switch, :installed]
      refused = [:graphlaw_ex, :kill_switch, :refused]
      assert installed in Telemetry.events()
      assert refused in Telemetry.events()
      assert Catalog.log_level(installed) == :info
      assert Catalog.log_level(refused) == :warning
      assert Catalog.measurements_for(installed) == []
    end

    test "compile-time heads agree with catalog data for every entry" do
      for e <- Catalog.catalog() do
        assert Catalog.log_level(e.event) == e.level
        assert Catalog.measurements_for(e.event) == e.measurements
      end
    end
  end

  describe "emit redaction at the boundary" do
    test "span exception metadata arrives redacted exactly once" do
      id = "boundary-exc-#{System.unique_integer()}"
      ref = collect(id, [:graphlaw_ex, :admit, :exception])

      assert_raise ErlangError, fn ->
        Telemetry.admit("boundary-op", [], fn ->
          :erlang.error({:secret_tag, "payload-canary"})
        end)
      end

      assert_receive {^ref, _m, md}, 1_000
      assert md.reason == %{tag: :secret_tag}
      refute inspect(md) =~ "payload-canary"
      assert Enum.all?(md.stacktrace, fn f -> is_tuple(f) and not is_list(elem(f, 2)) end)
    end

    test "emit without handlers is a no-op returning :ok" do
      assert :ok = Telemetry.emit([:graphlaw_ex, :nobody, :listens], %{count: 1}, %{nquads: "x"})
    end
  end

  describe "admit classification" do
    test "GraphlawEx.Error and Capability.Refusal carry their code" do
      id = "boundary-cls-#{System.unique_integer()}"
      ref = collect(id, [:graphlaw_ex, :admit, :stop])

      err = %GraphlawEx.Error{code: :REFUSED_INVALID_INPUT}
      assert {:error, ^err} = Telemetry.admit("boundary-cls", [], fn -> {:error, err} end)

      assert_receive {^ref, _,
                      %{
                        op: "boundary-cls",
                        outcome: :refused,
                        refusal_code: :REFUSED_INVALID_INPUT
                      }},
                     1_000

      cap = %GraphlawEx.Capability.Refusal{code: :saturated}
      Telemetry.admit("boundary-cls2", [], fn -> {:error, cap} end)

      assert_receive {^ref, _,
                      %{op: "boundary-cls2", outcome: :refused, refusal_code: :saturated}},
                     1_000
    end

    test ":error and unexpected shapes are refused, not ok" do
      id = "boundary-bad-#{System.unique_integer()}"
      ref = collect(id, [:graphlaw_ex, :admit, :stop])

      Telemetry.admit("boundary-bad1", [], fn -> :error end)

      assert_receive {^ref, _, %{op: "boundary-bad1", outcome: :refused, refusal_code: nil}},
                     1_000

      Telemetry.admit("boundary-bad2", [], fn -> %{weird: true} end)
      assert_receive {^ref, _, %{op: "boundary-bad2", outcome: :refused}}, 1_000
    end
  end

  describe "introspection scope prefixes" do
    test "shacl/duckdb/datalog limit prefixes are scoped, not global" do
      for p <- ["shacl_", "duckdb_", "datalog_", "max_datalog_"] do
        assert p in Tables.scoped_prefixes()
      end

      assert Tables.scope_prefix("shacl") == "shacl_"
      assert Tables.scope_prefix("datalog") == "datalog_"
    end

    test "limits for a non-owning op do not leak shacl_/duckdb_/datalog_ keys" do
      {:ok, op} = GraphlawEx.Introspection.info("parse")
      keys = op.limits |> Map.keys() |> Enum.map(&Atom.to_string/1)

      refute Enum.any?(
               keys,
               &String.starts_with?(&1, ["shacl_", "duckdb_", "datalog_", "max_datalog_"])
             )
    end
  end
end
