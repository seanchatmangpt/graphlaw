defmodule GraphlawEx.HostTest do
  use ExUnit.Case, async: false

  # Chicago: the real engine is the collaborator. The courts below exercise the
  # REAL host GenServer over the real wasm artifact — the caller-side
  # :saturated shed, and the :memory_high_water recycle watermark (the
  # Host.info recycles counter). Absence of the artifact is a LOUD typed skip,
  # never a faked pass. No mocks, no sleeps: the mailbox fill is structural
  # (:sys.suspend/1 suspends the GenServer's message processing), never
  # time-based.

  @wasm_path System.get_env("PRIV_WASM") || Path.expand("priv/graphlaw/graphlaw.wasm", __DIR__)
  @require_engine System.get_env("GRAPHLAW_REQUIRE_ENGINE") == "1"

  # The first host start in a fresh VM compiles the real 74 MB artifact.
  @moduletag timeout: 300_000

  @tag :wasm
  test "shed: a caller at or above the host's max_queue is refused :saturated, not queued" do
    with_engine(fn path ->
      host = start_host!(path, max_queue: 2, recycle_bytes: huge())
      assert GraphlawEx.Host.available?(host)

      # Suspend the host's message processing so the mailbox fill is real and
      # stable, then pad it past the bound. `:sys.get_state/1` blocks until the
      # host has PROCESSED the suspend system message, so the padding cannot
      # drain before suspension lands (no sleeps, a real observable).
      :ok = :sys.suspend(host)
      _ = :sys.get_state(host)
      for _ <- 1..5, do: send(host, :court_padding)

      # The caller's own bound is the request option (default 64), not the
      # host config: the shed layers compose, caller bound first.
      assert {:error, %GraphlawEx.Refusal{code: :saturated} = refusal} =
               GraphlawEx.Host.request(host, %{"op" => "capabilities"}, max_queue: 2)

      assert refusal.details.max_queue == 2
      assert refusal.details.queue >= 2
      assert refusal.class == :blocked_resource
      assert refusal.broken_term == :R_missing_consequence

      # Resume: the padding drains and the same host serves the same request.
      :ok = :sys.resume(host)

      assert {:ok, %{"ok" => true}} = GraphlawEx.Host.request(host, %{"op" => "capabilities"})
      :ok = GraphlawEx.stop(host)
    end)
  end

  @tag :wasm
  test "shed: an idle host under the bound admits (the shed is a bound, not a refusal posture)" do
    with_engine(fn path ->
      host = start_host!(path, max_queue: 2, recycle_bytes: huge())
      assert {:ok, %{"ok" => true}} = GraphlawEx.Host.request(host, %{"op" => "capabilities"})
      assert {:ok, %{recycles: 0}} = GraphlawEx.Host.info(host)
      :ok = GraphlawEx.stop(host)
    end)
  end

  @tag :wasm
  test "recycle watermark: crossing :recycle_bytes recycles and the counter is observable" do
    with_engine(fn path ->
      # Watermark of 1 byte: linear memory after any instantiation is far above
      # it, so the first completed request must trip :memory_high_water.
      host = start_host!(path, recycle_bytes: 1)

      assert {:ok, %{"ok" => true}} = GraphlawEx.Host.request(host, %{"op" => "capabilities"})

      {:ok, info} = GraphlawEx.Host.info(host)
      assert info.recycles >= 1
      assert is_binary(info.wasm_sha256) and byte_size(info.wasm_sha256) == 64
      assert info.path == path

      # A recycled host still serves: recycle replaces the instance over the
      # cached compiled module, it never degrades availability.
      assert {:ok, %{"ok" => true}} = GraphlawEx.Host.request(host, %{"op" => "capabilities"})
      assert GraphlawEx.Host.available?(host)
      :ok = GraphlawEx.stop(host)
    end)
  end

  test "no engine: a host over a missing artifact is unavailable and refuses typed, never crashes" do
    missing = Path.join(System.tmp_dir!(), "graphlaw-host-court-missing-#{System.unique_integer()}.wasm")

    start_link_supervised!({GraphlawEx.Host, wasm_path: missing, expected_sha256: :unpinned})

    assert {:error, %GraphlawEx.Refusal{code: :resource_limit}} =
             GraphlawEx.Host.request(GraphlawEx.Host, %{"op" => "capabilities"})

    refute GraphlawEx.Host.available?(GraphlawEx.Host)
  end

  # ---------------------------------------------------------------------

  defp start_host!(path, extra) do
    start_link_supervised!(
      {GraphlawEx.Host, Keyword.merge([wasm_path: path, expected_sha256: :unpinned], extra)}
    )
  end

  defp huge, do: 268_435_456

  defp with_engine(fun) do
    if File.regular?(@wasm_path) do
      fun.(@wasm_path)
    else
      message =
        "[SKIP] host courts: wasm artifact not found at #{@wasm_path} (set PRIV_WASM). " <>
          "A loud skip is not a pass."

      if @require_engine, do: flunk(message)
      IO.puts(message)
    end
  end
end
