# SPDX-FileCopyrightText: 2026 Sean Chatman <https://github.com/seanchatmangpt/graphlaw/graphs/contributors>
#
# SPDX-License-Identifier: MIT
defmodule GraphlawEx.Blake3 do
  @moduledoc """
  Pure-Elixir BLAKE3 (default 32-byte hash mode) for verifying the engine
  against its `.blake3` sidecar; OTP ships no BLAKE3.

  The compression function is generated at compile time with all sixteen
  state words and sixteen message words as plain variables (seven fully
  unrolled rounds, message schedule resolved at compile time), so a block is
  one binary match plus straight-line integer arithmetic: no per-word lists,
  no `elem/put_elem` traffic. A chunk's chaining value is an 8-tuple.

  ## Examples

      iex> GraphlawEx.Blake3.hash_hex("")
      "af1349b9f5f9a1a6a0404dea36dcc9499bcb25c9adc112b7cc9a93cae41f3262"
  """

  import Bitwise

  @iv {0x6A09E667, 0xBB67AE85, 0x3C6EF372, 0xA54FF53A, 0x510E527F, 0x9B05688C, 0x1F83D9AB,
       0x5BE0CD19}
  @chunk_start 1
  @chunk_end 2
  @parent 4
  @root 8

  @doc "Lowercase hex BLAKE3-256 of `bytes`."
  @spec hash_hex(binary()) :: String.t()
  def hash_hex(bytes) when is_binary(bytes) do
    bytes |> chunks(0, []) |> cv_bin() |> Base.encode16(case: :lower)
  end

  # The final chunk (<= 1024 bytes) closes the tree.
  defp chunks(bin, idx, stack) when byte_size(bin) <= 1024 do
    if idx == 0 do
      blocks(bin, @iv, 0, @chunk_start, @root)
    else
      fold(stack, blocks(bin, @iv, idx, @chunk_start, 0))
    end
  end

  defp chunks(<<chunk::binary-size(1024), rest::binary>>, idx, stack) do
    cv = blocks(chunk, @iv, idx, @chunk_start, 0)
    chunks(rest, idx + 1, merge([cv | stack], idx + 1))
  end

  defp merge([r, l | rest], total) when (total &&& 1) == 0,
    do: merge([parent(l, r, 0) | rest], total >>> 1)

  defp merge(stack, _total), do: stack

  defp fold([l | rest], cv), do: fold(rest, parent(l, cv, if(rest == [], do: @root, else: 0)))
  defp fold([], cv), do: cv

  defp parent(l, r, root),
    do: compress(@iv, <<cv_bin(l)::binary, cv_bin(r)::binary>>, 0, 64, @parent ||| root)

  defp blocks(bin, cv, idx, start_flag, root) when byte_size(bin) <= 64 do
    len = byte_size(bin)
    block = <<bin::binary, 0::size((64 - len) * 8)>>
    compress(cv, block, idx, len, start_flag ||| @chunk_end ||| root)
  end

  defp blocks(<<b::binary-size(64), rest::binary>>, cv, idx, start_flag, root),
    do: blocks(rest, compress(cv, b, idx, 64, start_flag), idx, 0, root)

  defp cv_bin({a, b, c, d, e, f, g, h}),
    do:
      <<a::little-32, b::little-32, c::little-32, d::little-32, e::little-32, f::little-32,
        g::little-32, h::little-32>>

  # --- compile-time generated compression -------------------------------

  mask = 0xFFFFFFFF
  v = fn i -> Macro.var(:"v#{i}", nil) end
  m = fn i -> Macro.var(:"m#{i}", nil) end

  rot = fn x, n ->
    quote do
      unquote(x) =
        (unquote(x) >>> unquote(n) ||| unquote(x) <<< unquote(32 - n)) &&& unquote(mask)
    end
  end

  g = fn ai, bi, ci, di, mx, my ->
    [a, b, c, d] = Enum.map([ai, bi, ci, di], v)

    [
      quote(do: unquote(a) = unquote(a) + unquote(b) + unquote(mx) &&& unquote(mask)),
      quote(do: unquote(d) = Bitwise.bxor(unquote(d), unquote(a))),
      rot.(d, 16),
      quote(do: unquote(c) = unquote(c) + unquote(d) &&& unquote(mask)),
      quote(do: unquote(b) = Bitwise.bxor(unquote(b), unquote(c))),
      rot.(b, 12),
      quote(do: unquote(a) = unquote(a) + unquote(b) + unquote(my) &&& unquote(mask)),
      quote(do: unquote(d) = Bitwise.bxor(unquote(d), unquote(a))),
      rot.(d, 8),
      quote(do: unquote(c) = unquote(c) + unquote(d) &&& unquote(mask)),
      quote(do: unquote(b) = Bitwise.bxor(unquote(b), unquote(c))),
      rot.(b, 7)
    ]
  end

  perm = [2, 6, 3, 10, 7, 0, 4, 13, 1, 11, 12, 5, 9, 14, 15, 8]

  # Round 0 uses the identity order; each later round permutes the previous.
  schedules =
    Enum.reduce(1..6, [Enum.to_list(0..15)], fn _, [cur | _] = acc ->
      [Enum.map(perm, &Enum.at(cur, &1)) | acc]
    end)
    |> Enum.reverse()

  round_stmts = fn s ->
    mm = fn k -> m.(Enum.at(s, k)) end

    g.(0, 4, 8, 12, mm.(0), mm.(1)) ++
      g.(1, 5, 9, 13, mm.(2), mm.(3)) ++
      g.(2, 6, 10, 14, mm.(4), mm.(5)) ++
      g.(3, 7, 11, 15, mm.(6), mm.(7)) ++
      g.(0, 5, 10, 15, mm.(8), mm.(9)) ++
      g.(1, 6, 11, 12, mm.(10), mm.(11)) ++
      g.(2, 7, 8, 13, mm.(12), mm.(13)) ++
      g.(3, 4, 9, 14, mm.(14), mm.(15))
  end

  counter_v = Macro.var(:counter, nil)
  len_v = Macro.var(:len, nil)
  flags_v = Macro.var(:flags, nil)

  init_stmts = [
    quote(do: unquote(v.(8)) = 0x6A09E667),
    quote(do: unquote(v.(9)) = 0xBB67AE85),
    quote(do: unquote(v.(10)) = 0x3C6EF372),
    quote(do: unquote(v.(11)) = 0xA54FF53A),
    quote(do: unquote(v.(12)) = unquote(counter_v) &&& unquote(mask)),
    quote(do: unquote(v.(13)) = unquote(counter_v) >>> 32),
    quote(do: unquote(v.(14)) = unquote(len_v)),
    quote(do: unquote(v.(15)) = unquote(flags_v))
  ]

  out =
    {:{}, [], for(i <- 0..7, do: quote(do: Bitwise.bxor(unquote(v.(i)), unquote(v.(i + 8)))))}

  body = {:__block__, [], init_stmts ++ Enum.flat_map(schedules, round_stmts) ++ [out]}
  cv_pat = {:{}, [], Enum.map(0..7, v)}

  bin_pat =
    {:<<>>, [], for(i <- 0..15, do: {:"::", [], [m.(i), {:-, [], [{:little, [], nil}, 32]}]})}

  defp compress(
         unquote(cv_pat),
         unquote(bin_pat),
         unquote(counter_v),
         unquote(len_v),
         unquote(flags_v)
       ) do
    unquote(body)
  end
end
