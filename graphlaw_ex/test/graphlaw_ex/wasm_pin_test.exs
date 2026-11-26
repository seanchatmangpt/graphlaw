# SPDX-FileCopyrightText: 2026 Sean Chatman <https://github.com/seanchatmangpt/graphlaw/graphs/contributors>
#
# SPDX-License-Identifier: MIT
defmodule GraphlawEx.WasmPinTest do
  use ExUnit.Case, async: false

  alias GraphlawEx.{Config, Refusal, WasmConfig}

  @hex_a String.duplicate("a", 64)
  @hex_b String.duplicate("b", 64)

  @moduletag :tmp_dir

  defp write_manifest(dir, blake3) do
    path = Path.join(dir, "MANIFEST.json")
    File.write!(path, Jason.encode!(%{"artifact" => %{"blake3" => blake3}}))
    path
  end

  describe "pinned_blake3_result/2" do
    test "manifest alone is the pin", %{tmp_dir: dir} do
      manifest = write_manifest(dir, @hex_a)
      assert {:ok, @hex_a} = WasmConfig.pinned_blake3_result(manifest, Path.join(dir, "e.wasm"))
    end

    test "agreeing sidecar passes", %{tmp_dir: dir} do
      manifest = write_manifest(dir, @hex_a)
      wasm = Path.join(dir, "e.wasm")
      File.write!(wasm <> ".blake3", @hex_a <> "  e.wasm\n")
      assert {:ok, @hex_a} = WasmConfig.pinned_blake3_result(manifest, wasm)
    end

    test "disagreeing sidecar is a refusal, not an override", %{tmp_dir: dir} do
      manifest = write_manifest(dir, @hex_a)
      wasm = Path.join(dir, "e.wasm")
      File.write!(wasm <> ".blake3", @hex_b <> "\n")

      assert {:error, %Refusal{} = r} = WasmConfig.pinned_blake3_result(manifest, wasm)
      assert r.details.detail == :blake3_pin_mismatch
    end

    test "manifest without blake3 has no pin even with a sidecar", %{tmp_dir: dir} do
      manifest = write_manifest(dir, nil)
      wasm = Path.join(dir, "e.wasm")
      File.write!(wasm <> ".blake3", @hex_b)
      assert {:error, :no_pin} = WasmConfig.pinned_blake3_result(manifest, wasm)
    end
  end

  describe "limits/1 opts" do
    test "invalid opts raise naming the key" do
      assert_raise ArgumentError, ~r/:timeout_ms/, fn -> WasmConfig.limits(timeout_ms: "5") end
      assert_raise ArgumentError, ~r/:max_queue/, fn -> WasmConfig.limits(max_queue: 0) end
      assert_raise ArgumentError, ~r/:memories/, fn -> WasmConfig.limits(memories: -1) end
    end

    test "valid opts and zero-ok keys pass" do
      assert %{timeout_ms: 10, memories: 0} = WasmConfig.limits(timeout_ms: 10, memories: 0)
    end
  end

  describe "Config boot validation" do
    test "trusted_keys shape" do
      assert :ok = Config.validate(trusted_keys: ["abc"])
      assert :ok = Config.validate(trusted_keys: [])
      assert {:error, [_]} = Config.validate(trusted_keys: "abc")
      assert {:error, [_]} = Config.validate(trusted_keys: [1, ""])
    end

    test "kill_switch_test_mode refused with auto_start and kill_switch_path" do
      assert {:error, [msg]} =
               Config.validate(
                 kill_switch_test_mode: true,
                 auto_start: true,
                 kill_switch_path: "/tmp/ks"
               )

      assert msg =~ "kill_switch_test_mode"
      assert :ok = Config.validate(kill_switch_test_mode: true, auto_start: true)
      assert :ok = Config.validate(kill_switch_test_mode: true, kill_switch_path: "/tmp/ks")
    end
  end
end
