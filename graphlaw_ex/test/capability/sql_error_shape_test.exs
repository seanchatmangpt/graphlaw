# SPDX-License-Identifier: MIT
defmodule GraphlawEx.Capability.SqlErrorShapeTest do
  use ExUnit.Case, async: true

  alias GraphlawEx.Capability.Refusal
  alias GraphlawEx.Capability.Sql
  alias GraphlawEx.Error
  alias GraphlawEx.Result
  alias GraphlawEx.Result.Sparql

  defp construct_lease,
    do: %{signed_lease: %{"lease" => %{"ceiling" => "construct"}, "attestation" => %{}}}

  test "raw SQL is a Capability.Refusal that wraps to REFUSED_RAW_SQL_FORBIDDEN" do
    assert {:error, %Refusal{} = r} =
             Sql.run([text: "SELECT a FROM t"], lease: construct_lease())

    assert Error.wrap(r).code == :REFUSED_RAW_SQL_FORBIDDEN
  end

  test "missing text is a Capability.Refusal wrapping to REFUSED_INVALID_INPUT" do
    assert {:error, %Refusal{} = r} = Sql.run([], lease: construct_lease())
    assert Error.wrap(r).code == :REFUSED_INVALID_INPUT

    assert {:error, %Refusal{}} = Sql.run(%{"text" => 42}, lease: construct_lease())
  end

  test "an unleased sql run is refused REFUSED_LEASE_REQUIRED" do
    assert {:error, %Refusal{code: :ceiling_unmet} = r} =
             Sql.run(text: "@prefix a: <http://x/> .")

    assert Error.wrap(r).code == :REFUSED_LEASE_REQUIRED
  end

  test "an observe-only lease is refused before raw-SQL inspection" do
    lease = %{signed_lease: %{"lease" => %{"ceiling" => "observe"}, "attestation" => %{}}}
    assert {:error, %Refusal{code: :ceiling_unmet}} = Sql.run([text: "SELECT 1"], lease: lease)
  end

  test "Result.new!/1 raises a typed Error, not ArgumentError, without echoing the value" do
    secret = "lease-secret-xyz"
    bad = Sparql.from_map(%{"variables" => ["x", "y"], "rows" => [[secret]]})

    error = assert_raise Error, fn -> Result.new!(bad) end
    assert error.code == :REFUSED_INVALID_INPUT
    refute inspect(error) =~ secret

    assert_raise Error, fn -> Result.new!(%{"token" => secret}) end
  end
end
