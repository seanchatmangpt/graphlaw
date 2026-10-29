defmodule A2AAuthorityCeilingCourtTest do
  use ExUnit.Case, async: false

  # GL28-7 authority pre-gate court (design §4): an agent presenting no lease
  # (or an unsigned / atom ceiling claim) that invokes a construct-ceiling op
  # is refused `:ceiling_unmet` BEFORE Host dispatch; sniff/parse (`:observe`
  # floor) survive. Chicago: the real wasm artifact is the collaborator.
  # Zero-dispatch is asserted via the real `[:graphlaw_ex, :admit, :start]`
  # telemetry event count, not a mock.

  alias GraphlawEx.Capability.API
  alias GraphlawEx.Capability.Refusal
  alias GraphlawEx.Result.Admitted

  @wasm_path System.get_env("PRIV_WASM") || Path.expand("../priv/graphlaw.wasm", __DIR__)
  @require_engine System.get_env("GRAPHLAW_REQUIRE_ENGINE") == "1"

  @n3 "@prefix : <https://x/> . :a :b :c ."
  @turtle "<https://x/s> <https://x/p> <https://x/o> ."

  @lease_construct %{
    signed_lease: %{
      "lease" => %{"id" => "lease-1", "ceiling" => "construct"},
      "attestation" => %{"key_id" => "ops-2026", "signature" => "aa"}
    }
  }

  setup_all do
    if File.regular?(@wasm_path) do
      start_supervised!({GraphlawEx, wasm_path: @wasm_path, expected_sha256: :unpinned})
      assert GraphlawEx.Host.available?(GraphlawEx)
      :ok
    else
      message =
        "[SKIP] a2a authority ceiling court: wasm artifact not found at #{@wasm_path} " <>
          "(set PRIV_WASM to the absolute path of the graphlaw wasm module). A loud skip is not a pass."

      if @require_engine, do: flunk(message)

      IO.puts(message)
      {:ok, skipped: true}
    end
  end

  defp engine_present?, do: File.regular?(@wasm_path)

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
      count = count_admits(op)
      {result, count}
    after
      :telemetry.detach({:a2a_ceiling_court, test_pid, op})
    end
  end

  defp count_admits(op) do
    count = count_admits(op, 0)
    count
  end

  defp count_admits(op, acc) do
    receive do
      {:admit_start, ^op} -> count_admits(op, acc + 1)
    after
      0 -> acc
    end
  end

  # Case 1: unsigned agent invokes `law`.
  test "law without a lease is refused :ceiling_unmet before dispatch" do
    if engine_present?() do
      args = %{data: %{text: @n3, dialect: "n3"}, steps: [%{"kind" => "derive", "rules" => []}]}

      {result, admits} =
        with_admit_count("law", fn -> API.run("law", args, %{}) end)

      assert {:error, %Refusal{code: :ceiling_unmet} = refusal} = result
      assert refusal.details == %{"op" => "law", "required" => :construct, "granted" => :observe}
      assert refusal.broken_term == :R_missing_standing
      assert admits == 0
    end
  end

  # Case 2: unsigned agent invokes `hooks`.
  test "hooks without a lease is refused :ceiling_unmet before dispatch" do
    if engine_present?() do
      args = %{data: %{text: @turtle}}

      {result, admits} = with_admit_count("hooks", fn -> API.run("hooks", args, %{}) end)

      assert {:error, %Refusal{code: :ceiling_unmet}} = result
      assert admits == 0
    end
  end

  # Case 3: sniff/parse survive unsigned.
  @tag :wasm
  test "sniff and parse run unsigned on the real engine" do
    if engine_present?() do
      assert {:ok, %GraphlawEx.Result.Sniff{}} = API.run("sniff", %{text: @turtle}, %{})
      assert {:ok, %GraphlawEx.Result.Parse{}} = API.run("parse", %{text: @turtle}, %{})
    end
  end

  # Case 4: over-presented authority is allowed (>=).
  @tag :wasm
  test "parse with a construct lease also succeeds" do
    if engine_present?() do
      assert {:ok, %GraphlawEx.Result.Parse{}} =
               API.run("parse", %{text: @turtle}, lease: @lease_construct)
    end
  end

  # Case 5: a select-claiming lease on `law` is refused (rank 1 < 2).
  test "select lease on law is refused :ceiling_unmet before dispatch" do
    if engine_present?() do
      lease = %{signed_lease: %{"lease" => %{"ceiling" => "select"}, "attestation" => %{"key_id" => "k"}}}
      args = %{data: %{text: @n3, dialect: "n3"}, steps: []}

      {result, admits} = with_admit_count("law", fn -> API.run("law", args, lease: lease) end)

      assert {:error, %Refusal{code: :ceiling_unmet} = refusal} = result
      assert refusal.details["granted"] == :select
      assert admits == 0
    end
  end

  # Case 6: self-grant attacks never dispatch.
  test "self-grant attacks claim :observe and refuse on law" do
    if engine_present?() do
      args = %{data: %{text: @n3, dialect: "n3"}, steps: []}

      for attack <- [
            :construct,
            %{ceiling: :construct},
            %{"ceiling" => "construct"},
            %{lease: %{"ceiling" => "construct"}},
            %{unverified_lease: %{"ceiling" => "construct"}},
            nil
          ] do
        {result, admits} =
          with_admit_count("law", fn -> API.run("law", args, lease: attack) end)

        assert {:error, %Refusal{code: :ceiling_unmet}} = result, "attack #{inspect(attack)} dispatched"
        assert admits == 0, "attack #{inspect(attack)} dispatched"
      end
    end
  end

  # Case 7: a signed container with a forged/garbage attestation passes the
  # presentation gate and the signed lease is forwarded verbatim in the
  # request; behavior is engine-defined. Assert forwarding only.
  @tag :wasm
  test "forged-ceiling signed lease passes the gate and reaches the request" do
    if engine_present?() do
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
    assert is_binary(identity.lease_digest) and Regex.match?(~r/^[0-9a-f]{64}$/, identity.lease_digest)

    assert GraphlawEx.Authority.identity(nil) == nil
  end
end
