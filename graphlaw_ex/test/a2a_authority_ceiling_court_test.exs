# SPDX-FileCopyrightText: 2026 Sean Chatman <https://github.com/seanchatmangpt/graphlaw/graphs/contributors>
#
# SPDX-License-Identifier: MIT
defmodule A2AAuthorityCeilingCourtTest do
  use ExUnit.Case, async: false

  # GL28-7 authority pre-gate court (design §4, revised 2026-10-29 per lane A5):
  # the kernel authorizes leases PER STEP and authorizes lease-less dispatch
  # unconditionally (LeaseAuth::None), so the Elixir pre-gate is an additive
  # presentation check, not an enforcement boundary. All ops now carry an
  # `:observe` floor: lease-less and forged-ceiling calls pass the gate and
  # reach the engine (engine-side signed-lease verification is the real
  # boundary). Zero-dispatch is asserted via the real
  # `[:graphlaw_ex, :admit, :start]` telemetry event count, not a mock.
  #
  # Engine-backed cases are :wasm: excluded loudly when the artifact is absent
  # (test_helper.exs) and FAILING (TestEngine.engine_path!/0 raises) under
  # GRAPHLAW_REQUIRE_ENGINE=1.

  alias GraphlawEx.Capability.API
  alias GraphlawEx.Capability.Refusal
  alias GraphlawEx.Result.Admitted

  @n3 "@prefix : <https://x/> . :a :b :c ."
  @turtle "<https://x/s> <https://x/p> <https://x/o> ."

  @lease_construct %{
    signed_lease: %{
      "lease" => %{"id" => "lease-1", "ceiling" => "construct"},
      "attestation" => %{"key_id" => "ops-2026", "signature" => "aa"}
    }
  }

  # Trust anchors come from Application env only (caller opts are ignored unless
  # `allow_caller_keys`). A well-formed anchor that does not match the forged
  # leases here: dispatch happens, the engine judges the lease.
  @anchor String.duplicate("ab", 32)

  setup context do
    previous = Application.fetch_env(:graphlaw_ex, :trusted_keys)
    Application.put_env(:graphlaw_ex, :trusted_keys, [@anchor])

    on_exit(fn ->
      case previous do
        {:ok, value} -> Application.put_env(:graphlaw_ex, :trusted_keys, value)
        :error -> Application.delete_env(:graphlaw_ex, :trusted_keys)
      end
    end)

    if context[:wasm] do
      path = GraphlawEx.TestEngine.engine_path!()
      start_supervised!({GraphlawEx, wasm_path: path, expected_sha256: :unpinned})
      assert GraphlawEx.Host.available?(GraphlawEx)
    end

    :ok
  end

  # Counts real admit-span starts for `op` during `fun` — zero means the gate
  # refused before `module.run/2` and no WASM dispatch happened.
  defp with_admit_count(op, fun) do
    test_pid = self()

    :telemetry.attach(
      {:a2a_ceiling_court, test_pid, op},
      [:graphlaw_ex, :admit, :start],
      fn [:graphlaw_ex, :admit, :start], _measure, meta, _ ->
        if meta[:op] == op, do: send(test_pid, {:admit_start, op})
      end,
      nil
    )

    try do
      result = fun.()
      count = count_admits(op, 0)
      {result, count}
    after
      :telemetry.detach({:a2a_ceiling_court, test_pid, op})
    end
  end

  defp count_admits(op, acc) do
    receive do
      {:admit_start, ^op} -> count_admits(op, acc + 1)
    after
      0 -> acc
    end
  end

  # Case 1 (revised per lane A5): lease-less `law` passes the observe-floor
  # gate; the kernel (LeaseAuth::None) is the enforcement boundary.
  @tag :wasm
  test "law without a lease passes the gate and dispatches" do
    args = %{data: %{text: @n3, dialect: "n3"}, steps: []}

    {result, admits} =
      with_admit_count("law", fn -> API.run("law", args, %{}) end)

    assert_gate_pass(result)
    assert admits >= 1, "expected real dispatch, got #{admits} admit starts"
  end

  # Case 2 (revised per lane A5): lease-less `hooks` likewise dispatches.
  @tag :wasm
  test "hooks without a lease passes the gate and dispatches" do
    args = %{data: %{text: @turtle}}

    {result, admits} = with_admit_count("hooks", fn -> API.run("hooks", args, %{}) end)

    assert_gate_pass(result)

    assert admits >= 1, "expected real dispatch, got #{admits} admit starts"
  end

  # Case 3: sniff/parse survive unsigned.
  @tag :wasm
  test "sniff and parse run unsigned on the real engine" do
    assert {:ok, %GraphlawEx.Result.Sniff{}} = API.run("sniff", %{text: @turtle}, %{})
    assert {:ok, %GraphlawEx.Result.Parse{}} = API.run("parse", %{text: @turtle}, %{})
  end

  # Case 4: a signed lease on an op whose request cannot carry it is refused
  # loudly (:lease_not_carried), never silently dropped, and never dispatched.
  @tag :wasm
  test "parse with a signed lease it cannot carry is refused before dispatch" do
    {result, admits} =
      with_admit_count("parse", fn ->
        API.run("parse", %{text: @turtle}, lease: @lease_construct)
      end)

    assert {:error, %Refusal{code: :lease_not_carried}} = result
    assert admits == 0
  end

  # Case 5 (revised per lane A5): a select-claiming lease passes the
  # observe-floor gate; the forwarded lease is judged engine-side per step.
  @tag :wasm
  test "select lease on law passes the gate and dispatches" do
    lease = %{
      signed_lease: %{"lease" => %{"ceiling" => "select"}, "attestation" => %{"key_id" => "k"}}
    }

    args = %{data: %{text: @n3, dialect: "n3"}, steps: []}

    {result, admits} = with_admit_count("law", fn -> API.run("law", args, lease: lease) end)

    assert_gate_pass(result)
    assert admits >= 1, "expected real dispatch, got #{admits} admit starts"
  end

  # Case 6 (revised per lane A5): forged/unverified ceiling claims are not a
  # gate boundary anymore — they pass through and the engine judges the
  # forwarded lease. Zero-:ceiling_unmet is the invariant; dispatch is expected.
  @tag :wasm
  test "self-grant attacks pass the gate and are judged engine-side" do
    args = %{data: %{text: @n3, dialect: "n3"}, steps: []}

    # A non-container lease is malformed: refused before dispatch.
    {malformed, zero} =
      with_admit_count("law", fn -> API.run("law", args, lease: :construct) end)

    assert {:error, %Refusal{code: :malformed_lease}} = malformed
    assert zero == 0

    for attack <- [
          %{ceiling: :construct},
          %{"ceiling" => "construct"},
          %{lease: %{"ceiling" => "construct"}},
          %{unverified_lease: %{"ceiling" => "construct"}},
          nil
        ] do
      {result, admits} =
        with_admit_count("law", fn -> API.run("law", args, lease: attack) end)

      assert_gate_pass(result)
      assert admits >= 1, "attack #{inspect(attack)} did not dispatch"
    end
  end

  # Case 7: a signed container with a forged/garbage attestation passes the
  # presentation gate and the signed lease is forwarded verbatim in the
  # request; behavior is engine-defined. Assert forwarding only.
  @tag :wasm
  test "forged-ceiling signed lease passes the gate and reaches the request" do
    forged = %{
      signed_lease: %{
        "lease" => %{"id" => "forged", "ceiling" => "construct"},
        "attestation" => %{"key_id" => "ghost", "signature" => "deadbeef"}
      }
    }

    args = %{data: %{text: @n3, dialect: "n3"}, steps: []}

    {result, _admits} = with_admit_count("law", fn -> API.run("law", args, lease: forged) end)

    # Presentation check only: the gate lets it through. The pinned engine
    # ignores lease keys, so the op runs (or refuses engine-side — either
    # way the refusal is NOT :ceiling_unmet).
    case result do
      {:ok, %Admitted{}} -> :ok
      {:error, %Refusal{code: code}} -> assert code != :ceiling_unmet
      other -> flunk("unexpected outcome: #{inspect(other)}")
    end
  end

  # Case 8: unknown op, unchanged path.
  test "unknown op refuses :unknown_capability" do
    assert {:error, %Refusal{code: :unknown_capability}} = API.run("nope", %{}, %{})
  end

  # Case 9: identity/1 for evidence/receipts.
  test "identity of the presented lease" do
    identity = GraphlawEx.Authority.identity(@lease_construct)

    assert identity.ceiling == :construct
    assert identity.lease_id == "lease-1"
    assert identity.key_id == "ops-2026"

    assert is_binary(identity.lease_digest) and
             Regex.match?(~r/^[0-9a-f]{64}$/, identity.lease_digest)

    assert GraphlawEx.Authority.identity(nil) == nil
  end

  defp assert_gate_pass({:ok, _}), do: :ok

  defp assert_gate_pass({:error, %Refusal{code: code}}) when code != :ceiling_unmet, do: :ok

  defp assert_gate_pass(other), do: flunk("unexpected outcome: #{inspect(other)}")
end
