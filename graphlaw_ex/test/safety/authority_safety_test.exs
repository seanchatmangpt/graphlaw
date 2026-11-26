# SPDX-FileCopyrightText: 2026 Sean Chatman <https://github.com/seanchatmangpt/graphlaw/graphs/contributors>
#
# SPDX-License-Identifier: MIT
defmodule GraphlawEx.AuthoritySafetyTest do
  use ExUnit.Case, async: true

  # Ceiling matrix for the authority pre-gate. Pure presentation check: no
  # engine involved, so nothing here can skip under GRAPHLAW_REQUIRE_ENGINE=1.

  alias GraphlawEx.Authority
  alias GraphlawEx.Capability.Refusal

  defp signed(ceiling),
    do: %{signed_lease: %{"lease" => %{"ceiling" => ceiling}, "attestation" => %{}}}

  @registry_path Path.expand("../../../registry/capability-registry.json", __DIR__)
  @external_resource @registry_path
  @registry @registry_path |> File.read!() |> Jason.decode!()
  @ceilings ~w(observe select construct)

  # Registry ops; `sql` is an Elixir-side op (a dialect in the registry), pinned below.
  @registry_ops Enum.map(@registry["ops"], & &1["name"])
  @registry_steps Enum.filter(@registry["law_steps"], & &1["ceiling"])

  defp signed_scoped(ceiling, scope),
    do: %{
      signed_lease: %{
        "lease" => %{"ceiling" => ceiling, "scope" => scope},
        "attestation" => %{}
      }
    }

  # Lowest ceiling under which Authority admits `step` (scope covers it).
  defp step_floor(step) do
    Enum.find(@ceilings, fn c ->
      Authority.check_steps([step], signed_scoped(c, [step])) == :ok
    end)
  end

  describe "registry parity" do
    test "registry sanity: ops and steps were loaded" do
      assert @registry_ops != []
      assert @registry_steps != []
      assert @registry["lease_ceilings"] == @ceilings
    end

    test "every registry op is known to Authority at the observe floor" do
      for op <- @registry_ops do
        assert {:ok, :observe} = Authority.op_ceiling(op), op
      end
    end

    test "every Rust law step ceiling equals Authority's required floor" do
      for %{"name" => name, "ceiling" => ceiling} <- @registry_steps do
        known =
          for prefix <- ["admit:", "derive:"],
              step = prefix <> name,
              match?(:ok, Authority.check_steps([step], signed_scoped("construct", [step]))),
              do: step

        assert [step] = known, "registry step #{name} must map to exactly one Authority step"
        assert step_floor(step) == ceiling, "#{step}: registry says #{ceiling}"
      end
    end

    test "a step floor is enforced: below-floor lease is ceiling_unmet" do
      for %{"name" => name, "ceiling" => ceiling} <- @registry_steps, ceiling != "observe" do
        step = Enum.find(["admit:" <> name, "derive:" <> name], &step_floor/1)
        below = Enum.take_while(@ceilings, &(&1 != ceiling))

        for c <- below do
          assert {:error, %Refusal{code: :ceiling_unmet}} =
                   Authority.check_steps([step], signed_scoped(c, [step])),
                 "#{step} under #{c}"
        end
      end
    end
  end

  describe "ceiling matrix" do
    test "observe-floor ops pass for every lease shape" do
      for op <- @registry_ops,
          lease <- [nil, signed("observe"), signed("select"), signed("construct")] do
        assert :ok = Authority.check_op(op, lease), "#{op} with #{inspect(lease)}"
      end
    end

    test "sql has an explicit :construct floor" do
      assert {:ok, :construct} = Authority.op_ceiling("sql")
      assert {:ok, :construct} = Authority.op_ceiling(:sql)
    end

    test "S4: sql op floor is :construct and sql is not a registry op" do
      refute "sql" in @registry_ops
      assert {:ok, :construct} = Authority.op_ceiling("sql")
    end

    test "sql refuses lease-less, observe and select claims" do
      for lease <- [nil, signed("observe"), signed("select")] do
        assert {:error, %Refusal{code: :ceiling_unmet, details: details}} =
                 Authority.check_op("sql", lease)

        assert details["op"] == "sql"
        assert details["required"] == :construct
      end
    end

    test "sql passes a signed construct lease" do
      assert :ok = Authority.check_op("sql", signed("construct"))
    end

    test "authorize/2 maps sql denial to a typed error" do
      assert {:error, e} = Authority.authorize("sql", nil)
      assert e.code == :REFUSED_LEASE_REQUIRED
      assert e.evidence["refusal_code"] == "ceiling_unmet"
      assert :ok = Authority.authorize("sql", signed("construct"))
    end
  end

  describe "malformed leases fail closed for sql" do
    test "self-grant and garbage shapes never reach :construct" do
      # No signed lease presented: claims :observe, sql refuses :ceiling_unmet.
      unsigned = [
        [],
        %{ceiling: :construct},
        %{"ceiling" => "construct"},
        %{lease: %{"ceiling" => "construct"}},
        %{unverified_lease: %{"ceiling" => "construct"}},
        %{signed_lease: nil}
      ]

      for lease <- unsigned do
        assert %{ceiling: :observe, signed_lease: nil} = Authority.claim(lease)

        assert {:error, %Refusal{code: :ceiling_unmet}} = Authority.check_op("sql", lease),
               inspect(lease)
      end

      # A non-nil lease that cannot be parsed refuses :malformed_lease.
      malformed = [
        :construct,
        "construct",
        42,
        [:construct],
        %{signed_lease: "construct"},
        %{signed_lease: %{}},
        %{signed_lease: %{"lease" => "construct"}},
        %{signed_lease: %{"lease" => %{"ceiling" => "root"}}},
        %{signed_lease: %{"lease" => %{"ceiling" => 2}}},
        %{signed_lease: %{"lease" => %{"ceiling" => nil}}},
        %{signed_lease: %{"lease" => %{"ceiling" => ["construct"]}}},
        [{:signed_lease, %{"lease" => %{"ceiling" => "bogus"}}}],
        [1, 2, 3]
      ]

      for lease <- malformed do
        assert {:error, :malformed_lease} = Authority.claim(lease), inspect(lease)

        assert {:error, %Refusal{code: :malformed_lease}} = Authority.check_op("sql", lease),
               inspect(lease)
      end
    end

    test "engine_opts forwards no signed lease for malformed input" do
      refute Keyword.has_key?(
               Authority.engine_opts(lease: %{signed_lease: %{"lease" => "x"}}),
               :signed_lease
             )
    end
  end

  describe "unknown ops" do
    test "stay typed :unknown_capability" do
      assert {:error, %Refusal{code: :unknown_capability}} = Authority.check_op("sqll", nil)
      assert {:error, %Refusal{code: :unknown_capability}} = Authority.check_op(nil, nil)
    end
  end
end
