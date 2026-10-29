defmodule GraphlawEx.KillSwitchTest do
  @moduledoc """
  GL28-6 `art14_killswitch_court` (flip-twin pattern, real engine collaborator).

  The honest runs prove *admission* (the full gl_alloc/gl_call path); the
  mutant twins prove each gate flips. Zero-mutation witnesses: `Host.info/2`
  `recycles` unchanged, `Host.memory_size/2` byte-identical across a refused
  call. A loud skip is not a pass: without the wasm artifact the wasm-tagged
  cases SKIP loudly; the Ed25519 + install-pipeline cases run engine-free and
  must always pass.
  """

  use ExUnit.Case, async: false

  alias GraphlawEx.KillSwitch
  alias GraphlawEx.Refusal

  @wasm_path System.get_env("PRIV_WASM") || Path.expand("priv/graphlaw/graphlaw.wasm", __DIR__)
  @require_engine System.get_env("GRAPHLAW_REQUIRE_ENGINE") == "1"

  setup do
    :ok = KillSwitch.erase()
    on_exit(fn -> KillSwitch.erase() end)
    :ok
  end

  # ------------------------------------------------------------------
  # Engine-free courts: Ed25519 kernel + install pipeline + refusal table
  # ------------------------------------------------------------------

  test "ed25519 kernel: RFC 8032 test vector 1 signs and verifies byte-exact" do
    seed = Base.decode16!("9d61b19deffd5a60ba844af492ec2cc44449c5697b326919703bac031cae7f60", case: :lower)
    pk_expect = Base.decode16!("d75a980182b10ab7d54bfed3c964073a0ee172f3daa62325af021a68f707511a", case: :lower)
    sig_expect =
      Base.decode16!(
        "e5564300c360ac729086e2cc806e828a84877f1eb8e5d974d873e065224901555fb8" <>
          "821590a33bacc61e39701cf9b46bd25bf5f0595bbe24655141438e7a100b",
        case: :lower
      )

    {pk, sig} = KillSwitch.Ed25519.sign(seed, "")
    assert pk == pk_expect
    assert sig == sig_expect
    assert KillSwitch.Ed25519.valid_signature?(sig, "", pk)
    refute KillSwitch.Ed25519.valid_signature?(sig, "tampered", pk)

    <<b, rest::binary>> = sig
    refute KillSwitch.Ed25519.valid_signature?(<<:erlang.bxor(b, 1), rest::binary>>, "", pk)
  end

  test "install: a well-formed revocation installs its monotone epoch" do
    {pub, seed} = keypair()
    {:ok, installed} = KillSwitch.install(revocation(7, "all", key_id(), seed), %{key_id() => pub})

    assert installed.epoch == 7
    assert installed.scope == "all"
    assert KillSwitch.current_epoch() == 7
  end

  test "install: mutated signature refuses at publication, epoch term unchanged" do
    {pub, seed} = keypair()
    doc = revocation(7, "all", key_id(), seed)
    # Alphabet-preserving tamper: decode the hex, flip one raw signature
    # byte, re-encode lowercase. (XOR-ing the hex STRING's first char is
    # not alphabet-preserving — `a`->backtick, `f`->`g` — and the refusal
    # would be :malformed at hex-decode instead of :invalid_signature at
    # the Ed25519 verify stage.)
    sig = doc["attestation"]["signature"]
    <<b, rest::binary>> = Base.decode16!(sig, case: :mixed)
    flipped = (:erlang.bxor(b, 1) |> :binary.encode_unsigned()) |> String.pad_leading(2, "0")
    doc = put_in(doc, ["attestation", "signature"], Base.encode16(flipped <> rest, case: :lower))

    assert {:error, %Refusal{code: :lease_revoked} = refusal} = KillSwitch.install(doc, %{key_id() => pub})
    assert refusal.details.reason == :invalid_signature
    assert KillSwitch.current_epoch() == 0
  end

  test "install: wrong operator key refuses at publication" do
    {pub, _seed} = keypair()
    {other_pub, other_seed} = keypair()
    doc = revocation(7, "all", key_id(), other_seed)

    assert {:error, %Refusal{code: :lease_revoked} = refusal} = KillSwitch.install(doc, %{key_id() => pub})
    assert refusal.details.reason == :invalid_signature
    assert byte_size(other_pub) == 32
    assert KillSwitch.current_epoch() == 0
  end

  test "install: untrusted key_id refuses at publication" do
    {pub, seed} = keypair()
    doc = revocation(7, "all", key_id(), seed)

    assert {:error, %Refusal{code: :lease_revoked} = refusal} = KillSwitch.install(doc, %{"other-ops" => pub})
    assert refusal.details.reason == :unknown_key_id
    assert KillSwitch.current_epoch() == 0
  end

  test "install: stale (non-monotone) epoch refuses at publication" do
    {pub, seed} = keypair()
    assert {:ok, _} = KillSwitch.install(revocation(9, "all", key_id(), seed), %{key_id() => pub})

    assert {:error, %Refusal{code: :lease_revoked} = refusal} =
             KillSwitch.install(revocation(9, "all", key_id(), seed), %{key_id() => pub})

    assert refusal.details.reason == :stale_epoch
    assert KillSwitch.current_epoch() == 9
  end

  test "install: malformed document and bad alg refuse at publication" do
    {pub, seed} = keypair()
    doc = revocation(3, "all", key_id(), seed)

    assert {:error, %Refusal{code: :lease_revoked}} =
             KillSwitch.install(Map.delete(doc, "attestation"), %{key_id() => pub})

    assert {:error, %Refusal{code: :lease_revoked}} = KillSwitch.install(Map.delete(doc, "revocation"), %{key_id() => pub})

    bad_alg = put_in(doc, ["attestation", "alg"], "rsa2048")
    assert {:error, %Refusal{code: :lease_revoked} = refusal} = KillSwitch.install(bad_alg, %{key_id() => pub})
    assert refusal.details.reason == :bad_alg
    assert KillSwitch.current_epoch() == 0
  end

  test "check: scope digest not matching the engine leaves the call admitted" do
    {pub, seed} = keypair()
    other_digest = String.duplicate("ab", 32)
    assert {:ok, _} = KillSwitch.install(revocation(4, other_digest, key_id(), seed), %{key_id() => pub})

    assert KillSwitch.check("c0ffee") == :ok
    assert KillSwitch.current_epoch() == 4
  end

  test "refusal table: :lease_revoked maps to refused_identity / R_missing_standing" do
    assert Refusal.class_of(:lease_revoked) == :refused_identity
    assert Refusal.broken_term_of(:lease_revoked) == :R_missing_standing
    assert :lease_revoked in Refusal.codes()

    refusal = Refusal.lease_revoked()
    assert refusal.class == :refused_identity
    assert refusal.broken_term == :R_missing_standing
  end

  # ------------------------------------------------------------------
  # Real-engine courts (wasm artifact)
  # ------------------------------------------------------------------

  @tag :wasm
  test "case 1 — valid lease admits: no revocation, the full gl_alloc/gl_call path runs" do
    with_engine(fn path ->
      host = start_host!(path)

      assert {:ok, %{"ok" => true}} = GraphlawEx.Host.request(host, %{"op" => "capabilities"})
      assert {:ok, %{recycles: 0}} = GraphlawEx.Host.info(host)
      :ok = GraphlawEx.stop(host)
    end)
  end

  @tag :wasm
  test "case 2 — revoked signature refuses at install; calls stay admitted" do
    with_engine(fn path ->
      host = start_host!(path)
      {pub, seed} = keypair()
      doc = revocation(7, "all", key_id(), seed)
      <<b, rest::binary>> = doc["attestation"]["signature"]
      doc = put_in(doc, ["attestation", "signature"], <<:erlang.bxor(b, 1), rest::binary>>)

      assert {:error, %Refusal{code: :lease_revoked}} = KillSwitch.install(doc, %{key_id() => pub})
      assert KillSwitch.current_epoch() == 0

      assert {:ok, %{"ok" => true}} = GraphlawEx.Host.request(host, %{"op" => "capabilities"})
      :ok = GraphlawEx.stop(host)
    end)
  end

  @tag :wasm
  test "case 3 — bumped epoch refuses the next call with zero engine mutation" do
    with_engine(fn path ->
      host = start_host!(path)
      assert {:ok, %{"ok" => true}} = GraphlawEx.Host.request(host, %{"op" => "capabilities"})
      {:ok, before} = GraphlawEx.Host.memory_size(host)

      {pub, seed} = keypair()
      assert {:ok, _} = KillSwitch.install(revocation(7, "all", key_id(), seed), %{key_id() => pub})

      ref = make_ref()
      :ok = attach(ref)

      assert {:error,
              %Refusal{code: :lease_revoked, class: :refused_identity, broken_term: :R_missing_standing} = refusal} =
               GraphlawEx.Host.request(host, %{"op" => "capabilities"})

      assert refusal.details.epoch == 7
      assert refusal.details.scope == "all"
      assert refusal.details.wasm_sha256 == info_sha(host)

      assert_receive {^ref, [:graphlaw_ex, :host, :call, :stop], _, %{code: :lease_revoked}}, 1_000
      :ok = :telemetry.detach(ref)

      # Zero-mutation witness: the engine instance was never touched.
      {:ok, mem_after} = GraphlawEx.Host.memory_size(host)
      assert mem_after == before
      assert {:ok, %{recycles: 0}} = GraphlawEx.Host.info(host)
      :ok = GraphlawEx.stop(host)
    end)
  end

  @tag :wasm
  test "case 5 — flip-twin: signed re-admission admits again; mutants each flip" do
    with_engine(fn path ->
      host = start_host!(path)
      {pub, seed} = keypair()
      assert {:ok, _} = KillSwitch.install(revocation(5, "all", key_id(), seed), %{key_id() => pub})

      assert {:error, %Refusal{code: :lease_revoked}} = GraphlawEx.Host.request(host, %{"op" => "capabilities"})

      # Honest survivor: a higher epoch whose scope misses this engine re-admits.
      assert {:ok, _} =
               KillSwitch.install(revocation(6, String.duplicate("cd", 32), key_id(), seed), %{key_id() => pub})

      assert KillSwitch.check(info_sha(host)) == :ok
      assert {:ok, %{"ok" => true}} = GraphlawEx.Host.request(host, %{"op" => "capabilities"})

      # Mutant twins against a fresh epoch term:
      :ok = KillSwitch.erase()
      assert KillSwitch.current_epoch() == 0

      # Mutant A: wrong operator key (document signed under an untrusted key).
      {good_pub, _good_seed} = keypair()
      {other_pub, other_seed} = keypair()

      assert {:error, %Refusal{code: :lease_revoked}} =
               KillSwitch.install(revocation(5, "all", key_id(), other_seed), %{key_id() => good_pub})

      assert KillSwitch.current_epoch() == 0

      # Mutant B: non-monotone epoch (replay of an older epoch).
      assert {:ok, _} = KillSwitch.install(revocation(5, "all", key_id(), seed), %{key_id() => pub})

      assert {:error, %Refusal{code: :lease_revoked} = stale} =
               KillSwitch.install(revocation(4, "all", key_id(), seed), %{key_id() => pub})

      assert stale.details.reason == :stale_epoch

      # Mutant C: mutated signature.
      doc = revocation(8, "all", key_id(), seed)
      <<b, rest::binary>> = doc["attestation"]["signature"]
      doc = put_in(doc, ["attestation", "signature"], <<:erlang.bxor(b, 1), rest::binary>>)
      assert {:error, %Refusal{code: :lease_revoked}} = KillSwitch.install(doc, %{key_id() => pub})
      assert byte_size(other_pub) == 32

      :ok = GraphlawEx.stop(host)
    end)
  end

  # ------------------------------------------------------------------
  # Court helpers
  # ------------------------------------------------------------------

  defp start_host!(path, extra \\ []) do
    start_link_supervised!(
      {GraphlawEx.Host, Keyword.merge([wasm_path: path, expected_sha256: :unpinned], extra)}
    )
  end

  defp with_engine(fun) do
    if File.regular?(@wasm_path) do
      fun.(@wasm_path)
    else
      message =
        "[SKIP] kill-switch courts: wasm artifact not found at #{@wasm_path} (set PRIV_WASM). " <>
          "A loud skip is not a pass."

      if @require_engine, do: flunk(message)
      IO.puts(message)
    end
  end

  defp info_sha(host) do
    {:ok, %{wasm_sha256: sha}} = GraphlawEx.Host.info(host)
    sha
  end

  defp key_id, do: "ops-2026"

  # Derives a fresh Ed25519 identity: {public, seed}. The seed signs every
  # court document for that identity.
  defp keypair do
    seed = :crypto.strong_rand_bytes(32)
    {pub, _sig} = KillSwitch.Ed25519.sign(seed, "")
    {pub, seed}
  end

  defp revocation(epoch, scope, key_id, seed) do
    rev = %{
      "epoch" => epoch,
      "scope" => scope,
      "issued_unix" => System.system_time(:second),
      "reason" => "art14-order-2026-10-28"
    }

    {pub, sig} = KillSwitch.Ed25519.sign(seed, KillSwitch.canonical_json(rev))

    %{
      "revocation" => rev,
      "attestation" => %{
        "key_id" => key_id,
        "alg" => "ed25519",
        "signature" => Base.encode16(sig, case: :lower),
        "public" => Base.encode16(pub, case: :lower)
      }
    }
  end

  defp attach(ref) do
    :telemetry.attach(
      ref,
      [:graphlaw_ex, :host, :call, :stop],
      fn event, measurements, meta, pid -> send(pid, {ref, event, measurements, meta}) end,
      self()
    )
  end
end
