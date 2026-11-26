# SPDX-FileCopyrightText: 2026 Sean Chatman <https://github.com/seanchatmangpt/graphlaw/graphs/contributors>
#
# SPDX-License-Identifier: MIT
defmodule GraphlawEx.EngineLoadTest do
  use ExUnit.Case, async: true

  # Courts for the load boundary ORDER: digest pin BEFORE import allowlist
  # BEFORE required exports BEFORE compile. The real wasmex compiler is the
  # collaborator; the foreign modules are hand-built WebAssembly binaries (a
  # real module the compiler really compiles). No mocks.

  alias GraphlawEx.EngineLoad
  alias GraphlawEx.Refusal

  # Compiling the real 74 MB artifact in the cache court takes minutes.
  @moduletag timeout: 300_000

  # A real, compilable wasm module: type section (()->()) + import section
  # importing "x"."y" — a foreign import module outside the WASI allowlist,
  # and no exports at all (so BOTH surface checks would fail; the court
  # asserts the import refusal wins, proving the order).
  @foreign_import <<
    0,
    97,
    115,
    109,
    1,
    0,
    0,
    0,
    # type section: id 1, size 4, content [1 type, functype ()->()]
    1,
    4,
    1,
    0x60,
    0,
    0,
    # import section: id 2, size 7, count 1, module (len 1) "x", name (len 1) "y", kind 0 (func), type index 0
    2,
    7,
    1,
    1,
    120,
    1,
    121,
    0,
    0
  >>

  # A real compilable module with NO imports and NO exports: it passes the
  # (vacuous) allowlist check and must then fail the required-exports check.
  @no_exports <<0, 97, 115, 109, 1, 0, 0, 0>>

  describe "refusal order" do
    test "the digest pin is judged first: a wrong pin refuses before any compilation" do
      # Garbage bytes that could never compile, with a pin that does not match
      # them: the refusal must be the PIN refusal, not the compile failure.
      assert {:error, %Refusal{code: :wasm_digest_mismatch} = refusal} =
               EngineLoad.admit(<<1, 2, 3>>, expected_sha256: String.duplicate("ab", 32))

      assert refusal.details.actual != refusal.details.expected
      refute Map.has_key?(refusal.details, :reason)
    end

    test "a matching pin admits garbage? no — compile then refuses :invalid_json" do
      # Order proof from the other side: with the EXACT pin of the bytes the
      # pin check passes and compilation is reached, whose failure is a
      # different, typed code.
      bytes = <<1, 2, 3>>
      digest = :crypto.hash(:sha256, bytes) |> Base.encode16(case: :lower)

      assert {:error, %Refusal{code: :invalid_json}} =
               EngineLoad.admit(bytes, expected_sha256: digest)

      # And a wrong pin on the same bytes is still the pin refusal, not compile.
      assert {:error, %Refusal{code: :wasm_digest_mismatch}} =
               EngineLoad.admit(bytes, expected_sha256: String.duplicate("cd", 32))
    end

    test "the import allowlist is judged before exports: a foreign import surfaces the import refusal" do
      assert {:error, %Refusal{code: :wasm_import_surface_mismatch} = refusal} =
               EngineLoad.admit(@foreign_import, expected_sha256: :unpinned)

      # Module "x" is not the closed wasi module, so the MODULE check refuses
      # it first; the per-function list stays empty for a foreign module.
      assert refusal.details.unexpected == ["x"]
      assert refusal.details.unexpected_functions == []
      assert refusal.details.allowed == ["wasi_snapshot_preview1"]
    end

    test "required exports are judged: an import-clean module without gl_alloc/gl_call/gl_free/memory refuses" do
      assert {:error, %Refusal{code: :wasm_import_surface_mismatch} = refusal} =
               EngineLoad.admit(@no_exports, expected_sha256: :unpinned)

      assert refusal.details.missing == ["gl_alloc", "gl_call", "gl_free", "memory"]
      assert refusal.details.required == ["gl_alloc", "gl_call", "gl_free", "memory"]
    end
  end

  describe "boot vs artifact integrity" do
    test "boot ignores :expected_blake3 (BLAKE3 is not run at boot)" do
      wrong = String.duplicate("00", 32)

      # Same bytes, same wrong BLAKE3 pin: verify_artifact refuses, boot does
      # not run BLAKE3 and so reaches surface admission, a typed Refusal.
      assert {:error, {:REFUSED_ARTIFACT_INTEGRITY, _}} =
               EngineLoad.verify_artifact(@no_exports, expected_blake3: wrong)

      assert {:error, %Refusal{code: :wasm_import_surface_mismatch}} =
               EngineLoad.boot(@no_exports, expected_sha256: :unpinned, expected_blake3: wrong)
    end
  end

  # The cache court lives in GraphlawEx.EngineLoadCacheTest below: it purges the
  # global compiled-module cache, so it must run async: false, and this module is
  # async: true.
end

# :wasm - excluded loudly without the artifact; engine_path!/0 raises under
# GRAPHLAW_REQUIRE_ENGINE=1.
defmodule GraphlawEx.EngineLoadCacheTest do
  use ExUnit.Case, async: false

  alias GraphlawEx.EngineLoad

  @moduletag timeout: 300_000

  describe "cache" do
    @tag :wasm
    test "only an admitted module is cached, and the real artifact round-trips the cache" do
      path = GraphlawEx.TestEngine.engine_path!()
      bytes = File.read!(path)
      digest = :crypto.hash(:sha256, bytes) |> Base.encode16(case: :lower)

      EngineLoad.purge_cache()
      refute EngineLoad.cached?(digest, false)

      assert {:ok, admitted} = EngineLoad.admit(bytes, expected_sha256: :unpinned, fuel?: false)
      assert admitted.wasm_sha256 == digest

      assert EngineLoad.cached?(digest, false)
      assert EngineLoad.purge_cache() == :ok
      refute EngineLoad.cached?(digest, false)

      # And a pinned admit of the REAL bytes admits too (the pin matches).
      assert {:ok, _} = EngineLoad.admit(bytes, expected_sha256: digest)
    end
  end
end
