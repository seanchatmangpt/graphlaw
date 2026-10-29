defmodule GraphlawEx.ABI do
  @moduledoc """
  Pure codec for the GraphLaw wasm ABI (version 1).

  This module holds no process state and makes no Wasmex calls. `GraphlawEx.Host`
  owns linear-memory operations (`gl_alloc`, write, `gl_call`, read, `gl_free`);
  this module only converts between Elixir terms and the bytes and packed
  integers that cross the boundary.

  ## Packed result

  `gl_call` returns one 64-bit integer: the high 32 bits are the response pointer
  and the low 32 bits the response length. Hosts that surface the value as a
  signed `i64` hand back a negative integer; `unpack_result/1` normalizes it by
  adding `2^64` before splitting. This replaces the old `packed > 0` guard in the
  facade's packed-result path, which silently dropped every response whose
  buffer landed in the high half of the 32-bit address space (a hang with the
  request left allocated).

  ## Frame

  A packet-4 frame is `<<length::32-big-endian, payload::binary>>`; the length
  prefix travels as framing metadata — the payload alone is what lands in linear
  memory.

  ## Limits

  Requests larger than 16 MiB are refused with `:resource_limit` before any
  memory is allocated in the engine (`max_request_bytes/0`).
  """

  import Bitwise

  alias GraphlawEx.Refusal

  @abi_version 1
  @u64 0x1_0000_0000_0000_0000
  @u32_mask 0xFFFF_FFFF
  @max_request_bytes 16 * 1_048_576

  @doc "The ABI version this build of the library speaks."
  @spec version() :: pos_integer()
  def version, do: @abi_version

  @doc "Maximum encoded request size in bytes (16 MiB)."
  @spec max_request_bytes() :: pos_integer()
  def max_request_bytes, do: @max_request_bytes

  @doc """
  Splits a packed `gl_call` result into `{ptr, len}`.

  Negative integers (a signed `i64` view of the same 64 bits) are normalized by
  adding `2^64`.

      iex> GraphlawEx.ABI.unpack_result(Bitwise.bsl(123, 32) + 45)
      {123, 45}

      iex> GraphlawEx.ABI.unpack_result(-1)
      {4_294_967_295, 4_294_967_295}
  """
  @spec unpack_result(integer()) :: {non_neg_integer(), non_neg_integer()}
  def unpack_result(packed) when is_integer(packed) do
    unsigned = if packed < 0, do: packed + @u64, else: packed
    {unsigned >>> 32 &&& @u32_mask, unsigned &&& @u32_mask}
  end

  @doc """
  Frames a payload as a packet-4 frame: `<<byte_size(payload)::32, payload::binary>>`.

  Returns `{:error, refusal}` with `:invalid_encoding` for a payload that is not
  valid UTF-8 (the JSON request is a text document) and `:resource_limit` above
  the 16 MiB cap.
  """
  @spec frame(binary()) :: {:ok, binary()} | {:error, GraphlawEx.Refusal.t()}
  def frame(payload) when is_binary(payload) do
    cond do
      not String.valid?(payload) ->
        {:error, Refusal.new(:invalid_encoding, "request payload is not valid UTF-8")}

      byte_size(payload) > @max_request_bytes ->
        {:error,
         Refusal.new(:resource_limit, "request exceeds the 16 MiB ABI limit", %{
           bytes: byte_size(payload),
           limit: @max_request_bytes
         })}

      true ->
        {:ok, <<byte_size(payload)::unsigned-integer-size(32), payload::binary>>}
    end
  end

  def frame(_other) do
    {:error, Refusal.new(:invalid_json, "request payload must be a binary JSON document")}
  end

  @doc """
  Encodes a request map (string keys) as JSON payload bytes (unframed; framing is
  `frame/1`).

  Returns `{:error, %Refusal{code: :resource_limit}}` above the 16 MiB cap,
  `{:error, %Refusal{code: :invalid_json}}` when the term cannot be JSON-encoded,
  and `{:error, %Refusal{code: :invalid_encoding}}` when a string in it is not
  valid UTF-8.
  """
  @spec encode_request(map()) :: {:ok, binary()} | {:error, Refusal.t()}
  def encode_request(request) when is_map(request) do
    case safe_encode(request) do
      {:ok, payload} when byte_size(payload) > @max_request_bytes ->
        {:error,
         Refusal.new(:resource_limit, "request exceeds the 16 MiB ABI limit", %{
           bytes: byte_size(payload),
           limit: @max_request_bytes
         })}

      {:ok, payload} ->
        {:ok, payload}

      :invalid_encoding ->
        {:error, Refusal.new(:invalid_encoding, "request contains text that is not valid UTF-8")}

      {:error, message} ->
        {:error, Refusal.new(:invalid_json, "request is not JSON-encodable: " <> message)}
    end
  end

  @doc """
  Decodes response bytes into a map.

  Non-JSON bytes yield `:invalid_json`; JSON that is not an object yields
  `:malformed_response`. Both `"ok": true` and `"ok": false` bodies decode to
  `{:ok, map}`; classifying them is the caller's job.
  """
  @spec decode_response(binary()) :: {:ok, map()} | {:error, Refusal.t()}
  def decode_response(body) when is_binary(body) do
    case Jason.decode(body) do
      {:ok, %{} = response} ->
        {:ok, response}

      {:ok, _other} ->
        {:error, Refusal.new(:malformed_response, "GraphLaw response is not a JSON object")}

      {:error, reason} ->
        {:error, Refusal.new(:invalid_json, Exception.message(reason), %{bytes: byte_size(body)})}
    end
  end

  defp safe_encode(request) do
    case Jason.encode(request) do
      {:ok, payload} -> {:ok, payload}
      {:error, reason} -> classify_encode_error(reason)
    end
  rescue
    error -> classify_encode_error(error)
  end

  # Jason rejects a string that is not valid UTF-8 before any byte reaches the
  # engine; that is an encoding refusal, not a JSON-shape one. The message
  # (which would quote the offending text) is deliberately not propagated for it.
  defp classify_encode_error(%Jason.EncodeError{message: "invalid byte " <> _rest}), do: :invalid_encoding
  defp classify_encode_error(error), do: {:error, Exception.message(error)}
end
