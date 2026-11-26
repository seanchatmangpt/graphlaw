# SPDX-FileCopyrightText: 2026 Sean Chatman <https://github.com/seanchatmangpt/graphlaw/graphs/contributors>
#
# SPDX-License-Identifier: MIT
defmodule GraphlawEx.LeasePolicyTest do
  use ExUnit.Case, async: false

  alias GraphlawEx.Authority
  alias GraphlawEx.Capability.Dispatch
  alias GraphlawEx.Config

  @args [data: "<a> <b> <c> .", steps: ["rdfs"]]

  setup do
    previous = Application.fetch_env(:graphlaw_ex, :lease_policy)

    on_exit(fn ->
      case previous do
        {:ok, value} -> Application.put_env(:graphlaw_ex, :lease_policy, value)
        :error -> Application.delete_env(:graphlaw_ex, :lease_policy)
      end
    end)

    :ok
  end

  defp put_policy(value), do: Application.put_env(:graphlaw_ex, :lease_policy, value)

  test "config validates :lease_policy" do
    assert :ok = Config.validate(lease_policy: :required)
    assert :ok = Config.validate(lease_policy: :none_allowed)
    assert {:error, [msg]} = Config.validate(lease_policy: :off)
    assert msg =~ ":lease_policy"
    assert_raise ArgumentError, fn -> Config.validate!(lease_policy: "required") end
  end

  test "default is :required when the key is unset" do
    Application.delete_env(:graphlaw_ex, :lease_policy)
    assert Authority.lease_policy() == :required
  end

  test "test env runs :none_allowed" do
    put_policy(:none_allowed)
    assert Authority.effective_lease_policy([]) == :none_allowed
    assert :ok = Authority.check_lease_policy("law", nil, [])
  end

  test ":required refuses a lease-less law admit with the typed lease-required code" do
    put_policy(:required)

    assert {:error, refusal} = Dispatch.run("law", @args, [])
    assert refusal.code == :ceiling_unmet
    assert refusal.details["required_by"] == "lease_policy"

    error = GraphlawEx.Error.wrap(refusal)
    assert error.code == :REFUSED_LEASE_REQUIRED
  end

  test "opts cannot loosen a :required policy" do
    put_policy(:required)

    assert Authority.effective_lease_policy(lease_policy: :none_allowed) == :required

    assert {:error, refusal} = Dispatch.run("law", @args, lease_policy: :none_allowed)
    assert refusal.code == :ceiling_unmet
  end

  test "opts can tighten a :none_allowed policy" do
    put_policy(:none_allowed)

    assert Authority.effective_lease_policy(lease_policy: :required) == :required
    assert {:error, refusal} = Dispatch.run("law", @args, lease_policy: :required)
    assert refusal.details["required_by"] == "lease_policy"
  end

  test "non-law ops are not subject to the lease policy gate" do
    put_policy(:required)
    assert :ok = Authority.check_lease_policy("sparql", nil, [])
  end
end
