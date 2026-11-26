# SPDX-FileCopyrightText: 2026 Sean Chatman <https://github.com/seanchatmangpt/graphlaw/graphs/contributors>
#
# SPDX-License-Identifier: MIT
defmodule GraphlawEx.ContextTest do
  use ExUnit.Case, async: true

  alias GraphlawEx.Context

  doctest GraphlawEx.Context

  defmodule StaticResolver do
    @behaviour GraphlawEx.Context.LeaseResolver
    @impl true
    def resolve(%Context{actor_id: "known"}), do: {:ok, %{"sig" => "x"}}
    def resolve(_), do: :none
  end

  test "new/1 accepts keyword and map, defaults metadata" do
    assert {:ok, %Context{actor_id: "a", metadata: %{}}} = Context.new(actor_id: "a")
    assert {:ok, %Context{tenant: "t"}} = Context.new(%{tenant: "t"})
  end

  test "new/1 refuses bad types and unknown keys" do
    assert {:error, {:invalid_field, :tenant}} = Context.new(tenant: :x)
    assert {:error, {:unknown_field, :lease}} = Context.new(lease: %{})
  end

  test "merge/2 overrides non-nil and merges metadata" do
    {:ok, a} = Context.new(actor_id: "a", trace_id: "t1", metadata: %{k: 1})
    assert {:ok, m} = Context.merge(a, trace_id: "t2", metadata: %{j: 2})
    assert {m.actor_id, m.trace_id, m.metadata} == {"a", "t2", %{k: 1, j: 2}}
  end

  test "to_telemetry/1 never leaks raw identity values or metadata" do
    {:ok, c} =
      Context.new(actor_id: "alice", tenant: "acme", trace_id: "tr", metadata: %{pw: "hunter2"})

    t = Context.to_telemetry(c)
    dumped = inspect(t)
    refute dumped =~ "alice"
    refute dumped =~ "acme"
    refute dumped =~ "hunter2"
    assert t.metadata_keys == 1
  end

  test "resolver behaviour maps context to lease or :none" do
    {:ok, k} = Context.new(actor_id: "known")
    {:ok, u} = Context.new(actor_id: "other")
    assert {:ok, %{"sig" => _}} = StaticResolver.resolve(k)
    assert :none = StaticResolver.resolve(u)
  end
end
