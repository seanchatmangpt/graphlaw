# SPDX-FileCopyrightText: 2026 Sean Chatman <https://github.com/seanchatmangpt/graphlaw/graphs/contributors>
#
# SPDX-License-Identifier: MIT
defmodule GraphlawEx.IntegrityTest do
  use ExUnit.Case, async: true

  # Gate 1: artifact integrity against EngineLoad.verify_artifact/2. The real
  # wasm is only ever READ; the mutation court flips a byte of an in-memory
  # copy that is also written to a TEMP file. A missing artifact or sidecar is
  # a LOUD skip (flunk under GRAPHLAW_REQUIRE_ENGINE=1), never a pass.

  alias GraphlawEx.EngineLoad

  @wasm_path System.get_env("PRIV_WASM") || GraphlawEx.TestEngine.wasm_path()
  @moduletag timeout: 300_000

  setup do
    dir = Path.join(System.tmp_dir!(), "graphlaw-integrity-#{System.unique_integer([:positive])}")
    File.mkdir_p!(dir)
    on_exit(fn -> File.rm_rf!(dir) end)
    {:ok, dir: dir}
  end

  test "small bytes: matching pin ok, 1-byte mutation refused, no pin refused" do
    bytes = :binary.copy(<<7>>, 3000)
    pin = GraphlawEx.EngineLoad.Blake3.hash_hex(bytes)

    assert :ok = EngineLoad.verify_artifact(bytes, expected_blake3: pin)

    assert {:error, {:REFUSED_ARTIFACT_INTEGRITY, %{expected: ^pin, actual: actual}}} =
             EngineLoad.verify_artifact(flip(bytes, 1500), expected_blake3: pin)

    assert actual != pin

    assert {:error, {:REFUSED_ARTIFACT_INTEGRITY, %{reason: :no_pin}}} =
             EngineLoad.verify_artifact(bytes, expected_blake3: nil)
  end

  test "real artifact: valid bytes ok; 1-byte mutation of a temp copy refused", %{dir: dir} do
    with_artifact(fn bytes, pin ->
      assert :ok = EngineLoad.verify_artifact(bytes, expected_blake3: pin)

      copy = Path.join(dir, "graphlaw.wasm")
      File.write!(copy, flip(bytes, div(byte_size(bytes), 2)))
      mutated = File.read!(copy)

      refute mutated == bytes
      assert byte_size(mutated) == byte_size(bytes)

      assert {:error, {:REFUSED_ARTIFACT_INTEGRITY, %{expected: ^pin, actual: actual}}} =
               EngineLoad.verify_artifact(mutated, expected_blake3: pin)

      assert actual != pin

      # The real artifact is unchanged by the court.
      assert File.read!(@wasm_path) == bytes
    end)
  end

  test "pin equality: MANIFEST == sidecars == ARTIFACTS wasm row (== staged size when present)" do
    priv = Path.expand("../priv/graphlaw", __DIR__)
    artifacts = Path.expand("../../registry/ARTIFACTS.blake3", __DIR__)

    manifest = priv |> Path.join("MANIFEST.json") |> File.read!() |> Jason.decode!()
    m_blake3 = manifest["artifact"]["blake3"]
    m_sha256 = manifest["artifact"]["sha256"]

    assert is_binary(m_blake3) and byte_size(m_blake3) == 64
    assert is_binary(m_sha256) and byte_size(m_sha256) == 64

    assert sidecar_pin(Path.join(priv, "graphlaw.wasm.blake3")) == m_blake3
    assert sidecar_pin(Path.join(priv, "graphlaw.wasm.sha256")) == m_sha256

    rows =
      artifacts
      |> File.read!()
      |> String.split("\n", trim: true)
      |> Enum.reject(&String.starts_with?(String.trim_leading(&1), "#"))
      |> Enum.map(&String.split/1)
      |> Enum.filter(&match?(["wasm", _, _, "graphlaw_wasm.wasm"], &1))

    assert [["wasm", row_blake3, row_size, "graphlaw_wasm.wasm"]] = rows
    assert String.downcase(row_blake3) == m_blake3

    staged = Path.join(priv, "graphlaw.wasm")

    if File.regular?(staged) do
      assert File.stat!(staged).size == String.to_integer(row_size)
    end
  end

  defp sidecar_pin(path) do
    [pin | _] = path |> File.read!() |> String.split()
    String.downcase(pin)
  end

  defp flip(bytes, at) do
    <<head::binary-size(at), b, tail::binary>> = bytes
    <<head::binary, Bitwise.bxor(b, 0xFF), tail::binary>>
  end

  defp with_artifact(fun) do
    sidecar = @wasm_path <> ".blake3"

    cond do
      not File.regular?(@wasm_path) ->
        skip("no wasm artifact at #{@wasm_path} (set PRIV_WASM)")

      not File.regular?(sidecar) ->
        skip("no integrity sidecar at #{sidecar}")

      true ->
        [pin | _] = sidecar |> File.read!() |> String.split()
        fun.(File.read!(@wasm_path), String.downcase(pin))
    end
  end

  defp skip(reason) do
    message = "[SKIP] integrity court: #{reason}. A loud skip is not a pass."
    if System.get_env("GRAPHLAW_REQUIRE_ENGINE") == "1", do: flunk(message)
    IO.puts(message)
  end
end
