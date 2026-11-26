# SPDX-FileCopyrightText: 2026 Sean Chatman <https://github.com/seanchatmangpt/graphlaw/graphs/contributors>
#
# SPDX-License-Identifier: MIT
defmodule GraphlawEx.PoolRouteTest do
  # Court: with capacity == size, an unavailable member can never win routing.
  # Real Pool/Host over the real Wasmtime running GraphlawEx.FakeEngine. One
  # member is made genuinely unavailable by corrupting the artifact file and
  # forcing that member to recycle (malformed response), then healthy-sized
  # requests must all be served by the remaining live member.
  use ExUnit.Case, async: false

  alias GraphlawEx.FakeEngine
  alias GraphlawEx.Pool

  @deadline_ms 30_000

  @tag :tmp_dir
  test "capacity == size: slot-selected unavailable member falls through to a live one", %{
    tmp_dir: tmp
  } do
    path = Path.join(tmp, "engine.wasm")
    File.write!(path, FakeEngine.bytes(:malformed))

    start_supervised!(
      {Pool,
       size: 2, wasm_path: path, expected_sha256: :unpinned, retry_base_ms: 20, retry_max_ms: 100}
    )

    assert poll(fn -> length(Pool.members()) == 2 and Pool.available() == 2 end)

    File.write!(path, <<1, 2, 3>>)

    # The faulting call recycles whichever member served it; the reload reads
    # the corrupt file, so that member becomes unavailable.
    assert {:error, _} = Pool.request(FakeEngine.bad_request())

    assert poll(fn ->
             length(Pool.members()) == 1 and length(Pool.unavailable_members()) == 1
           end),
           "no member became unavailable"

    assert poll(fn -> Pool.available() == 2 end)

    # Slots are claimed from a random start, so both slot indexes are hit.
    for _ <- 1..40 do
      assert {:ok, %{"ok" => true}} = Pool.request(FakeEngine.good_request())
    end

    assert poll(fn -> Pool.available() == 2 end)
  end

  defp poll(fun) do
    do_poll(fun, System.monotonic_time(:millisecond) + @deadline_ms)
  end

  defp do_poll(fun, deadline) do
    cond do
      fun.() ->
        true

      System.monotonic_time(:millisecond) >= deadline ->
        false

      true ->
        receive do
        after
          5 -> do_poll(fun, deadline)
        end
    end
  end
end
