# SPDX-FileCopyrightText: 2026 Sean Chatman <https://github.com/seanchatmangpt/graphlaw/graphs/contributors>
#
# SPDX-License-Identifier: MIT
defmodule GraphlawEx.FakeEngine do
  @moduledoc """
  A hand-written, REAL implementation of the GraphLaw wasm ABI (not a mock):
  a tiny WebAssembly module exporting `gl_alloc`, `gl_call`, `gl_free` and
  `memory`, compiled by the real Wasmtime through `Wasmex`, admitted by the real
  `GraphlawEx.EngineLoad`, and driven by the real `GraphlawEx.Host`.

  It exists so engine-FREE courts can exercise host/pool fault paths that the
  real 74 MB engine cannot be made to produce on demand (a hang, a trap, a
  malformed response, a memory-growth abort) without any artifact on disk.

  Behaviour is selected by request size, which keeps one module able to be both
  faulty and healthy: a request body of `bad_len/0` bytes or more triggers the
  fault, a smaller one is answered `{"ok":true}`.

    * `:healthy`   - never faults
    * `:malformed` - answers the JSON array `[1]` (`:malformed_response`)
    * `:trap`      - executes `unreachable` (`:call_trapped`)
    * `:hang`      - spins forever (host deadline)
    * `:grow`      - grows memory by 64 MiB and aborts (`unreachable`) when the
                     store's memory limit refuses, as a Rust allocator abort does
  """

  @bad_len 64

  @template ~S"""
  (module
    (memory (export "memory") 1)
    (data (i32.const 2048) "{\"ok\":true}")
    (data (i32.const 3072) "[1]")
    (func (export "gl_alloc") (param i32) (result i32) i32.const 1024)
    (func (export "gl_free") (param i32 i32))
    (func (export "gl_call") (param $ptr i32) (param $len i32) (result i64)
      (if (i32.ge_u (local.get $len) (i32.const __BAD_LEN__))
        (then __BAD__))
      ;; (2048 << 32) | 11
      (i64.const 8796093022219)))
  """

  @bodies %{
    healthy: "(nop)",
    malformed: "(return (i64.const 13194139533315))",
    trap: "(unreachable)",
    hang: "(loop $spin (br $spin))",
    grow: "(if (i32.eq (memory.grow (i32.const 1024)) (i32.const -1)) (then (unreachable)))"
  }

  @doc "Request-body length (bytes) at or above which the fault fires."
  @spec bad_len() :: pos_integer()
  def bad_len, do: @bad_len

  @doc "The compiled wasm bytes of the fake engine with the given fault `kind`."
  @spec bytes(:healthy | :malformed | :trap | :hang | :grow) :: binary()
  def bytes(kind) when is_map_key(@bodies, kind) do
    wat =
      @template
      |> String.replace("__BAD_LEN__", Integer.to_string(@bad_len))
      |> String.replace("__BAD__", Map.fetch!(@bodies, kind))

    {:ok, wasm} = Wasmex.Wat.to_wasm(wat)
    wasm
  end

  @doc "A healthy request body (below the fault threshold)."
  @spec good_request() :: map()
  def good_request, do: %{"op" => "capabilities"}

  @doc "A request body at the fault threshold."
  @spec bad_request() :: map()
  def bad_request,
    do: %{"op" => "capabilities", "pad" => String.duplicate("x", @bad_len * 2)}
end
