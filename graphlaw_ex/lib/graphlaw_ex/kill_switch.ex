# SPDX-FileCopyrightText: 2026 Sean Chatman <https://github.com/seanchatmangpt/graphlaw/graphs/contributors>
#
# SPDX-License-Identifier: MIT
defmodule GraphlawEx.KillSwitch do
  @moduledoc """
  Art.14 operator kill-switch (GL28-6): a signed Ed25519 revocation, verified
  once at publication (`install/2`), enforced per call by a single
  `:persistent_term` integer compare inside `GraphlawEx.Host.handle_call/3` —
  a pre-envelope check that runs after server-side shedding (`saturated/1`)
  and before `run_request/5`, so a revoked call costs no fuel, no `gl_alloc`,
  no write, no `gl_call` and no recycle.

  The revocation document mirrors `Authority`'s signed-lease container with
  inverted polarity:

      %{
        "revocation" => %{
          "epoch" => 7,
          "scope" => "all",        # or a wasm_sha256 hex digest
          "issued_unix" => 1_769_000_000,
          "reason" => "art14-order-2026-10-28"
        },
        "attestation" => %{
          "key_id" => "ops-2026",
          "alg" => "ed25519",
          "signature" => "<hex Ed25519 over canonical_json(revocation)>"
        }
      }

  Epochs are monotone: `install/2` refuses epoch <= current with a typed
  refusal carrying `reason: :stale_epoch`, so a replayed old revocation cannot
  roll the state back to a permissive epoch.

  Trusted operator keys come only from the `install/2` second argument
  (runtime config) — a caller-supplied document can never introduce its own
  trust anchor.

  ## As an authority input

  The kill-switch is the operator-level, revoking half of the authority model.
  `GraphlawEx.Authority.authorize/2` decides whether a caller *presented* enough
  (a signed lease's ceiling and scope); `check/1` decides whether the operator has
  *withdrawn* authority for an engine (`"all"` or one `wasm_sha256`). A call is
  authorized only when both pass; revocation always wins. `check/1` is pure with
  respect to its arguments (it reads one `:persistent_term`), and its refusal maps
  through `GraphlawEx.Error.wrap/1` like any other, so downstream policy layers can
  treat a revoked engine as a forbidden-class denial via `evidence["refusal_code"]
  == "lease_revoked"`.
  """

  alias GraphlawEx.Refusal
  # Minimal unblock (GL28-11 lane, compile-freeze SLA): the in-file Ed25519
  # module is defined below this line's uses, so an explicit alias is required.
  alias GraphlawEx.KillSwitch.Ed25519

  @pt_key {__MODULE__, :epoch}
  # GL28-11 minimal unblock (compile-freeze SLA): @alg was set but never used;
  # re-add it together with its first use.
  # @alg "ed25519"

  @typedoc "Operator trust anchors: `key_id` -> 32-byte Ed25519 public key (raw binary or hex string)."
  @type trusted_keys :: %{optional(String.t()) => binary() | String.t()}

  @doc """
  Verifies a revocation document against `trusted_keys` and, on success,
  installs its epoch under `{GraphlawEx.KillSwitch, :epoch}`.

  Returns `{:ok, %{epoch, scope}}` (the reason and key id are never handed
  back to callers) or `{:error, %GraphlawEx.Refusal{code: :lease_revoked}}` — a
  malformed, untrusted, badly-signed or stale document is refused at
  publication and the epoch term is left untouched. `scope` must be `"all"` or
  a 64-hex wasm digest (normalized to lower case). Installs are serialized, so
  the epoch never regresses under concurrent publication.
  """
  @spec install(map(), trusted_keys()) :: {:ok, map()} | {:error, Refusal.t()}
  def install(doc, trusted_keys) when is_map(doc) and is_map(trusted_keys) do
    doc |> safe_install(trusted_keys) |> emit_install(doc)
  end

  def install(_doc, _trusted_keys) do
    {:error,
     Refusal.new(:lease_revoked, "install requires a document map and a trusted-keys map", %{
       reason: :malformed
     })}
  end

  defp safe_install(doc, trusted_keys) do
    do_install(doc, trusted_keys)
  rescue
    # Unencodable revocation values (tuples, pids, ...) must refuse, not crash.
    _ in [FunctionClauseError, ArgumentError, Protocol.UndefinedError, Jason.EncodeError] ->
      {:error,
       Refusal.new(:lease_revoked, "revocation document is not canonicalizable", %{
         reason: :malformed
       })}
  end

  # Telemetry: [:graphlaw_ex, :kill_switch, :installed | :refused]. Telemetry
  # must never break an install, so emission failures are swallowed.
  defp emit_install({:ok, %{epoch: epoch, scope: scope}} = result, _doc) do
    safe_emit(:installed, %{epoch: epoch, scope: scope})
    result
  end

  defp emit_install({:error, %Refusal{details: details}} = result, doc) do
    safe_emit(:refused, %{
      reason: Map.get(details || %{}, :reason),
      epoch: get_in_epoch(doc)
    })

    result
  end

  defp get_in_epoch(%{"revocation" => %{"epoch" => e}}) when is_integer(e), do: e
  defp get_in_epoch(_), do: nil

  defp safe_emit(kind, metadata) do
    GraphlawEx.Telemetry.emit([:graphlaw_ex, :kill_switch, kind], %{count: 1}, metadata)
  rescue
    _ -> :ok
  catch
    _, _ -> :ok
  end

  @doc """
  TEST-ONLY. Clears the installed revocation (epoch back to `0`). Refused with a
  typed refusal unless `Application.get_env(:graphlaw_ex, :kill_switch_test_mode)`
  is `true` (set in `config/test.exs`). A tripped switch stays tripped across
  worker recycles because the state lives in `:persistent_term`.
  """
  @spec reset() :: :ok | {:error, Refusal.t()}
  def reset, do: erase()

  defp do_install(doc, trusted_keys) do
    with {:ok, rev} <- revocation_of(doc),
         {:ok, att} <- attestation_of(doc),
         :ok <- alg_of(att),
         {:ok, pk} <- key_of(att, trusted_keys),
         :ok <- signature_of(att, rev, pk) do
      scope = normalize_scope(rev["scope"])
      epoch = rev["epoch"]

      # Compare-and-put under a lock restricted to `[node()]`: the epoch check
      # and the put are one critical section, so a concurrent lower epoch can
      # never overwrite. The scope is deliberately NODE-LOCAL (the epoch lives
      # in this node's :persistent_term); cluster-wide propagation is the job of
      # the `:kill_switch_propagate` hook in `KillSwitch.Server`, and
      # `KillSwitch.Server` is the sole durable writer.
      :global.trans({{__MODULE__, :install}, self()}, fn -> compare_and_put(epoch, scope) end, [
        node()
      ])
      |> case do
        :aborted ->
          {:error,
           Refusal.new(:lease_revoked, "install could not be serialized", %{reason: :lock_failed})}

        result ->
          result
      end
    end
  end

  defp compare_and_put(epoch, scope) do
    with :ok <- monotone_epoch(%{"epoch" => epoch}) do
      :persistent_term.put(@pt_key, %{
        epoch: epoch,
        scope: scope,
        installed_at: System.system_time(:second)
      })

      {:ok, %{epoch: epoch, scope: scope}}
    end
  end

  defp normalize_scope("all"), do: "all"
  defp normalize_scope(scope), do: String.downcase(scope)

  defp valid_scope?("all"), do: true

  defp valid_scope?(scope) when is_binary(scope),
    do: Regex.match?(~r/\A[0-9a-f]{64}\z/, String.downcase(scope))

  @doc """
  The per-call gate. `:ok` when no revocation is installed or the installed
  scope does not cover `wasm_sha256`; otherwise
  `{:error, %GraphlawEx.Refusal{code: :lease_revoked, class: :refused_identity,
  broken_term: :R_missing_standing}}` with details
  `%{epoch, scope, wasm_sha256}`. An exception while checking is treated as
  tripped (fail closed).
  """
  @spec check(String.t() | nil) :: :ok | {:error, Refusal.t()}
  def check(wasm_sha256) do
    # Normalize like the installed scope (lower case) so a differently-cased
    # digest cannot slip past a scoped revocation (fail open).
    wasm_sha256 = if is_binary(wasm_sha256), do: String.downcase(wasm_sha256), else: wasm_sha256

    case :persistent_term.get(@pt_key, nil) do
      nil ->
        :ok

      %{scope: "all"} = installed ->
        {:error, revoked_refusal(installed, wasm_sha256)}

      %{scope: scope} = installed when scope == wasm_sha256 ->
        {:error, revoked_refusal(installed, wasm_sha256)}

      _scope_mismatch ->
        :ok
    end
  rescue
    _ -> {:error, revoked_refusal(%{epoch: nil, scope: nil}, wasm_sha256)}
  catch
    _kind, _reason -> {:error, revoked_refusal(%{epoch: nil, scope: nil}, wasm_sha256)}
  end

  @doc "The installed epoch, or `0` when nothing is installed."
  @spec current_epoch() :: non_neg_integer()
  def current_epoch do
    case :persistent_term.get(@pt_key, nil) do
      nil -> 0
      %{epoch: epoch} -> epoch
    end
  end

  @doc "Canonical JSON (recursively sorted keys, JSON string escaping) the signature covers."
  @spec canonical_json(term()) :: String.t()
  def canonical_json(value), do: value |> canonical_iodata() |> IO.iodata_to_binary()

  @doc """
  TEST-ONLY court seam: erases the installed epoch (back to `0`). Refused unless
  `Application.get_env(:graphlaw_ex, :kill_switch_test_mode)` is `true`. An
  operator re-admits by installing a *higher* epoch whose scope misses the
  engine — production never calls this.
  """
  @spec erase() :: :ok | {:error, Refusal.t()}
  def erase do
    if Application.get_env(:graphlaw_ex, :kill_switch_test_mode, false) == true do
      :persistent_term.erase(@pt_key)
      :ok
    else
      {:error,
       Refusal.new(:lease_revoked, "kill-switch erase/reset is test-only", %{
         reason: :test_only
       })}
    end
  end

  # ------------------------------------------------------------------
  # Install pipeline
  # ------------------------------------------------------------------

  defp revocation_of(doc) do
    case doc do
      %{"revocation" => %{"epoch" => epoch, "scope" => scope} = rev}
      when is_integer(epoch) and epoch > 0 and is_binary(scope) and scope != "" ->
        if valid_scope?(scope) do
          {:ok, rev}
        else
          {:error,
           Refusal.new(:lease_revoked, "revocation scope must be \"all\" or a 64-hex digest", %{
             reason: :bad_scope
           })}
        end

      _ ->
        {:error,
         Refusal.new(
           :lease_revoked,
           "revocation document lacks {revocation: {epoch > 0, scope}}",
           %{
             reason: :malformed
           }
         )}
    end
  end

  defp attestation_of(%{"attestation" => %{"key_id" => key_id, "alg" => alg, "signature" => sig}})
       when is_binary(key_id) and is_binary(sig) do
    {:ok, %{key_id: key_id, alg: alg, signature: sig}}
  end

  defp attestation_of(_) do
    {:error,
     Refusal.new(
       :lease_revoked,
       "revocation document lacks {attestation: {key_id, alg, signature}}",
       %{
         reason: :malformed
       }
     )}
  end

  defp alg_of(%{alg: "ed25519"}), do: :ok

  defp alg_of(%{alg: other}) do
    {:error,
     Refusal.new(:lease_revoked, "unsupported attestation alg #{bounded(other)}", %{
       reason: :bad_alg
     })}
  end

  defp key_of(%{key_id: key_id}, trusted_keys) do
    case Map.fetch(trusted_keys, key_id) do
      {:ok, raw} ->
        case decode_key(raw) do
          {:ok, pk} ->
            {:ok, pk}

          _not_a_key ->
            {:error,
             untrusted("trusted key for #{bounded(key_id)} is not a 32-byte Ed25519 public key")}
        end

      :error ->
        {:error, untrusted("attestation key_id #{bounded(key_id)} is not a trusted operator key")}
    end
  end

  defp signature_of(%{signature: signature}, rev, pk) do
    with {:ok, sig} <- decode_signature(signature) do
      if Ed25519.valid_signature?(sig, canonical_json(rev), pk) do
        :ok
      else
        {:error,
         Refusal.new(:lease_revoked, "revocation signature does not verify", %{
           reason: :invalid_signature
         })}
      end
    end
  end

  defp monotone_epoch(%{"epoch" => epoch}) do
    current = current_epoch()

    if epoch > current do
      :ok
    else
      {:error,
       Refusal.new(
         :lease_revoked,
         "epoch #{epoch} is not above the installed epoch #{current}",
         %{
           reason: :stale_epoch,
           epoch: epoch,
           current_epoch: current
         }
       )}
    end
  end

  defp revoked_refusal(installed, wasm_sha256) do
    Refusal.new(
      :lease_revoked,
      "operator revocation epoch #{installed.epoch} covers this engine",
      %{
        epoch: installed.epoch,
        scope: installed.scope,
        wasm_sha256: wasm_sha256
      }
    )
  end

  defp bounded(value), do: inspect(value, printable_limit: 64, limit: 8)

  defp untrusted(message), do: Refusal.new(:lease_revoked, message, %{reason: :unknown_key_id})

  defp decode_key(pk) when is_binary(pk) and byte_size(pk) == 32, do: {:ok, pk}

  defp decode_key(pk) when is_binary(pk) do
    case hex_decode(pk) do
      {:ok, raw} when byte_size(raw) == 32 -> {:ok, raw}
      _ -> :error
    end
  end

  defp decode_key(_), do: :error

  defp decode_signature(sig) when is_binary(sig) do
    if String.valid?(sig), do: hex_decode(sig), else: {:ok, sig}
  end

  defp hex_decode(binary) when is_binary(binary) and rem(byte_size(binary), 2) == 0 do
    case Base.decode16(binary, case: :mixed) do
      {:ok, raw} ->
        {:ok, raw}

      :error ->
        {:error,
         Refusal.new(:lease_revoked, "attestation field is not hex", %{reason: :malformed})}
    end
  end

  defp hex_decode(_),
    do:
      {:error, Refusal.new(:lease_revoked, "attestation field is not hex", %{reason: :malformed})}

  # ------------------------------------------------------------------
  # Canonical JSON (sorted keys; the sign-over form of the revocation map)
  # ------------------------------------------------------------------

  defp canonical_iodata(%{} = map) do
    entries =
      map
      |> Map.new(fn {k, v} -> {to_json_key(k), v} end)
      |> Enum.sort(:desc)

    inner =
      Enum.map_intersperse(entries, ?,, fn {k, v} ->
        [Jason.encode!(k), ?:, canonical_iodata(v)]
      end)

    [?{, inner, ?}]
  end

  defp canonical_iodata(list) when is_list(list) do
    inner = Enum.map_intersperse(list, ?,, &canonical_iodata/1)
    [?[, inner, ?]]
  end

  defp canonical_iodata(value) when is_binary(value), do: Jason.encode!(value)
  defp canonical_iodata(value) when is_integer(value), do: Integer.to_string(value)
  defp canonical_iodata(true), do: "true"
  defp canonical_iodata(false), do: "false"
  defp canonical_iodata(nil), do: "null"
  defp canonical_iodata(value) when is_float(value), do: Jason.encode!(value)

  defp to_json_key(k) when is_binary(k), do: k
  defp to_json_key(k) when is_atom(k), do: Atom.to_string(k)
  defp to_json_key(k) when is_integer(k), do: Integer.to_string(k)

  # ------------------------------------------------------------------
  # Ed25519 (RFC 8032) in pure Elixir — verify (+ sign for the courts).
  # The host OTP's :crypto EdDSA provider is broken in this environment
  # ("Couldn't get EDDSA public/private key"), and the manifest is
  # coordinator-owned, so the kernel carries its own small implementation
  # over :crypto.hash(:sha512, _). Publication-time and court-only cost;
  # the per-call hot path never touches this module.
  # ------------------------------------------------------------------

  defmodule Ed25519 do
    @moduledoc false

    @p 57_896_044_618_658_097_711_785_492_504_343_953_926_634_992_332_820_282_019_728_792_003_956_564_819_949
    @l 7_237_005_577_332_262_213_973_186_563_042_994_240_857_116_359_379_907_606_001_950_938_285_454_250_989
    @d 37_095_705_934_669_439_343_138_083_508_754_565_189_542_113_879_843_219_016_388_785_533_085_940_283_555
    @sqrt_m1 19_681_161_376_707_505_956_807_079_304_988_542_015_446_066_515_923_890_162_744_021_073_123_829_784_752

    @bx 15_112_221_349_535_400_772_501_151_409_588_531_511_454_012_693_041_857_206_046_113_283_949_847_762_202
    @by 46_316_835_694_926_478_169_428_394_003_475_163_141_307_993_866_256_225_615_783_033_603_165_251_855_960

    @typedoc "64-byte Ed25519 signature."
    @type signature :: binary()

    @doc "True when `signature` is a valid Ed25519 signature of `message` under `public` (32 bytes)."
    @spec valid_signature?(signature(), binary(), binary()) :: boolean()
    def valid_signature?(sig, message, public)
        when byte_size(sig) == 64 and byte_size(public) == 32 do
      <<r_bin::binary-size(32), s_bin::binary-size(32)>> = sig
      s = :binary.decode_unsigned(s_bin, :little)

      with {:ok, r_point} <- decode_point(r_bin),
           {:ok, a_point} <- decode_point(public),
           true <- s < @l do
        k = hash_to_scalar([r_bin, public, message])
        lhs = scalar_mult(s, base_point())
        rhs = point_add(r_point, scalar_mult(k, a_point))
        point_equal?(lhs, rhs)
      else
        _ -> false
      end
    end

    def valid_signature?(_, _, _), do: false

    @doc "Signs `message` with the 32-byte `seed`; returns `{public, signature}`."
    @spec sign(binary(), binary()) :: {binary(), signature()}
    def sign(seed, message) when byte_size(seed) == 32 do
      <<a_raw::binary-size(32), prefix::binary-size(32)>> = sha512(seed)
      a = clamp(a_raw)
      public = encode_point(scalar_mult(a, base_point()))

      r = hash_to_scalar([prefix, message])
      r_bin = encode_point(scalar_mult(r, base_point()))
      k = hash_to_scalar([r_bin, public, message])
      s = Integer.mod(r + k * a, @l)

      {public, r_bin <> pad32(:binary.encode_unsigned(s, :little))}
    end

    # ----------------------------------------------------------------
    # Field / group arithmetic over 2^255 - 19 (extended coordinates,
    # a = -1; the add formulas are strongly unified so doubling is
    # point_add(P, P)).
    # ----------------------------------------------------------------

    defp sha512(data), do: :crypto.hash(:sha512, data)

    defp hash_to_scalar(parts),
      do: Integer.mod(:binary.decode_unsigned(:crypto.hash(:sha512, parts), :little), @l)

    # RFC 8032 clamp of the secret scalar bytes.
    defp clamp(raw) do
      raw
      |> :binary.decode_unsigned(:little)
      |> Bitwise.band(2 ** 254 - 8)
      |> Bitwise.bor(2 ** 254)
    end

    defp pad32(bin) do
      pad = 32 - byte_size(bin)
      if pad > 0, do: <<bin::binary, 0::size(pad * 8)>>, else: binary_part(bin, 0, 32)
    end

    defp base_point, do: {@bx, @by, 1, mod(@bx * @by)}

    defp point_add({x1, y1, z1, t1}, {x2, y2, z2, t2}) do
      a = mod((y1 - x1) * (y2 - x2))
      b = mod((y1 + x1) * (y2 + x2))
      c = mod(2 * t1 * t2 * @d)
      dd = mod(2 * z1 * z2)
      e = b - a
      f = dd - c
      g = dd + c
      h = b + a
      {mod(e * f), mod(g * h), mod(f * g), mod(e * h)}
    end

    defp scalar_mult(scalar, point), do: scalar_mult(scalar, point, {0, 1, 1, 0})

    defp scalar_mult(scalar, point, acc) when scalar > 0 do
      acc = if Bitwise.band(scalar, 1) == 1, do: point_add(acc, point), else: acc
      scalar_mult(Bitwise.bsr(scalar, 1), point_add(point, point), acc)
    end

    defp scalar_mult(0, _point, acc), do: acc

    # 32-byte little-endian encoding of y, with the sign bit of x in the MSB.
    defp encode_point({x, y, z, _t}) do
      zinv = invert(z)
      x_n = mod(x * zinv)
      y_n = mod(y * zinv)
      <<body::binary-size(31), msb>> = pad32(:binary.encode_unsigned(y_n, :little))
      low = Bitwise.bor(Bitwise.band(msb, 127), Bitwise.band(x_n, 1) * 128)
      <<body::binary, low>>
    end

    defp decode_point(raw32) do
      <<body::binary-size(31), msb>> = raw32
      sign = Bitwise.bsr(Bitwise.band(msb, 128), 7)
      y = :binary.decode_unsigned(<<body::binary, Bitwise.band(msb, 127)>>, :little)

      if y < @p do
        y2 = mod(y * y)
        u = mod(y2 - 1)
        v = mod(@d * y2 + 1)

        case recover_x(u, v) do
          {:ok, x} ->
            x = if Bitwise.band(x, 1) != sign, do: mod(-x), else: x
            {:ok, {x, y, 1, mod(x * y)}}

          :error ->
            :error
        end
      else
        :error
      end
    end

    # x = sqrt(u/v) via x^((p+3)/8); if that squares to -u/v, scale by
    # sqrt(-1). A non-residue (u/v) has no root: :error.
    defp recover_x(u, v) do
      t = mod(u * invert(v))
      x = pow(t, div(@p + 3, 8))

      case squares_to(t, x) do
        true -> {:ok, x}
        false -> try_sqrt_m1(t, x)
      end
    end

    defp squares_to(t, x), do: mod(x * x) == t

    defp try_sqrt_m1(t, x) do
      x1 = mod(x * @sqrt_m1)

      if squares_to(t, x1) do
        {:ok, x1}
      else
        :error
      end
    end

    defp pow(base, exponent), do: pow(base, exponent, 1)

    defp pow(_base, 0, acc), do: acc

    defp pow(base, exponent, acc) do
      acc = if Bitwise.band(exponent, 1) == 1, do: mod(acc * base), else: acc
      pow(mod(base * base), Bitwise.bsr(exponent, 1), acc)
    end

    defp invert(x), do: pow(x, @p - 2)

    defp mod(n), do: Integer.mod(n, @p)

    defp point_equal?({x1, y1, z1, _}, {x2, y2, z2, _}),
      do: mod(x1 * z2) == mod(x2 * z1) and mod(y1 * z2) == mod(y2 * z1)
  end
end
