# SPDX-FileCopyrightText: 2026 Sean Chatman <https://github.com/seanchatmangpt/graphlaw/graphs/contributors>
#
# SPDX-License-Identifier: MIT
defmodule GraphlawEx.Capability.SqlTest do
  use ExUnit.Case, async: false

  alias GraphlawEx.Capability.Sql
  alias GraphlawEx.Error

  doctest GraphlawEx.Capability.Sql

  @sql_ast """
  @prefix sql: <http://graphlaw.org/sql#> .
  @prefix ex: <http://example.org/queries#> .
  ex:Q a sql:SelectStatement ;
    sql:statementIndex 1 ;
    sql:fromTable [ sql:tableName "telemetry_events" ] ;
    sql:selectList ( [ sql:columnName "batch_id" ] ) .
  """

  describe "raw SQL refusal (no engine needed)" do
    # The sql op needs a construct ceiling before any text is inspected.
    defp construct_lease,
      do: %{signed_lease: %{"lease" => %{"ceiling" => "construct"}, "attestation" => %{}}}

    test "raw SQL is refused client-side" do
      for sql <- ["SELECT a FROM t", "  drop table t;", "-- c\nINSERT INTO t VALUES (1)"] do
        assert {:error, r} = Sql.run([text: sql], lease: construct_lease())
        assert %Error{code: :REFUSED_RAW_SQL_FORBIDDEN} = Error.wrap(r)
      end
    end

    test "an unleased run is refused before text inspection" do
      assert {:error, r} = Sql.run(text: "SELECT a FROM t")
      assert %Error{code: :REFUSED_LEASE_REQUIRED} = Error.wrap(r)
    end

    test "sql-ast Turtle is not classified as raw SQL" do
      refute Sql.raw_sql?(@sql_ast)
    end

    test "bad args refuse :invalid_capability_request instead of raising" do
      for bad <- [42, "SELECT 1", nil, [1, 2], {:text, "x"}] do
        assert {:error, %GraphlawEx.Capability.Refusal{code: :invalid_capability_request}} =
                 Sql.run(bad, lease: construct_lease())
      end
    end

    test "map args with string or atom keys reach the raw-SQL check" do
      for args <- [%{"text" => "SELECT a FROM t"}, %{text: "SELECT a FROM t"}] do
        assert {:error, r} = Sql.run(args, lease: construct_lease())
        assert %Error{code: :REFUSED_RAW_SQL_FORBIDDEN} = Error.wrap(r)
      end
    end

    test "missing text is an invalid input" do
      assert {:error, r} = Sql.run([], lease: construct_lease())
      assert %Error{code: :REFUSED_INVALID_INPUT} = Error.wrap(r)
    end
  end

  describe "real engine" do
    # Excluded loudly by test_helper.exs when the artifact is absent; under
    # GRAPHLAW_REQUIRE_ENGINE=1 engine_path!/0 raises, so a skip is a failure.
    @describetag :wasm

    setup do
      path = GraphlawEx.TestEngine.engine_path!()
      start_supervised!({GraphlawEx, wasm_path: path, expected_sha256: :unpinned})
      assert GraphlawEx.Host.available?(GraphlawEx)
      :ok
    end

    test "sql-ast graph without a lease is refused LEASE_REQUIRED, engine untouched" do
      assert {:error, r} = Sql.run([text: @sql_ast], [])
      assert %Error{code: :REFUSED_LEASE_REQUIRED} = Error.wrap(r)
    end

    test "raw SQL is refused before the engine" do
      assert {:error, r} = Sql.run([text: "SELECT 1"], lease: construct_lease())
      assert %Error{code: :REFUSED_RAW_SQL_FORBIDDEN} = Error.wrap(r)
    end
  end
end
