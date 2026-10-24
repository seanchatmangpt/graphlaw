# wasi-sdk reproducibility: scripts/build_wasi_duckdb.sh (L109)

Status: **VERIFY-LANDED + CLEAN-ROOM-LANDED (attempt 7, script-driven end-to-end)** — Date: 2026-10-22. Lane L109.
Repos: /Users/sac/frozen-duckdb (script owner), /Users/sac/rel4r/duckdb-src (read-only pin).
Subject staging: `~/.frozen-duckdb/cache/v2.0.0-dev-wasm32-wasip1`.

## What the script owns

`/Users/sac/frozen-duckdb/scripts/build_wasi_duckdb.sh` — reproduces the ENTIRE
staging:

- `bin/clang` wrapper (`exec <emsdk>/upstream/bin/clang --sysroot=<staging>/share/wasi-sysroot "$@"`),
  `bin/{clang++,llvm-ar,wasm-ld}` symlinks.
- `lib/wasm32-wasip1/`: libc++/libc++abi/libunwind = emsdk `-wasmexcept` variants
  (sha256-verified against emsdk), libclang_rt.builtins.a (emsdk), libc.a +
  crt1-command.o (rust 1.96.0 wasip1 self-contained, sha256-verified),
  **libpthread.a** (see below), the 87,620,792 B libduckdb.a + libcore_functions_extension.a
  + libicu_extension.a (cmake build of DUCKDB_SRC @ b6013f7aa2), libduckdb_wasi_compat.a
  (compiled from frozen-duckdb-sys/wasi_posix.c).
- `share/wasi-sysroot` (include symlink to emsdk sysroot include; lib/wasm32-wasip1
  symlink to the staging lib dir) so the clang driver finds crt1.o and -lc.

## Embedded verify output — real run, exit 0

`bash scripts/build_wasi_duckdb.sh --verify-only` (2026-10-22, exit 0, 19 PASS):
(bash scripts/build_wasi_duckdb.sh --verify-only; echo EXIT=$?) — last lines:

```
== verify staging: /Users/sac/.frozen-duckdb/cache/v2.0.0-dev-wasm32-wasip1
PASS: bin/clang wrapper executable
PASS: wrapper exec line carries --sysroot
PASS: bin/clang++ symlink -> /Users/sac/emsdk/upstream/bin/clang++
PASS: bin/llvm-ar symlink -> /Users/sac/emsdk/upstream/bin/llvm-ar
PASS: bin/wasm-ld symlink -> /Users/sac/emsdk/upstream/bin/wasm-ld
PASS: share/wasi-sysroot merged (include + lib/wasm32-wasip1 resolve)
PASS: libclang_rt.builtins.a == emsdk builtins (sha256 6eb339614a6064fc76c0dfdcc98b363901e32e35c6c89aafe6126973af29b2bc) # historical
PASS: lib/wasm32-wasip1/libc.a == rust wasip1 self-contained (sha256)
PASS: lib/wasm32-wasip1/crt1-command.o == rust wasip1 self-contained (sha256)
PASS: libc++.a == emsdk libc++-wasmexcept.a (sha256 0f632424e349b1e632b3ec05a87ba749cc64f2c9c25e96422177e971d3b76a7a) # historical
PASS: libc++abi.a == emsdk libc++abi-wasmexcept.a (sha256 551b6e6bdf395d61145bcaadf75d89bd443a02dcbe0209359172b0bcc65397c5) # historical
PASS: libunwind.a == emsdk libunwind-wasmexcept.a (sha256 7ab65b4ca03c66172ec7effcd04f9b402a9c1854b3adc400d4fe951040da8a7b) # historical
PASS: libduckdb.a: 87620792 B, wasm32 magic in 64 MiB prefix
PASS: libcore_functions_extension.a: 16165504 B, wasm32 magic in 64 MiB prefix
PASS: libicu_extension.a: 3309950 B, wasm32 magic in 64 MiB prefix
PASS: libduckdb_wasi_compat.a: 1040 B, wasm32 magic in 64 MiB prefix
PASS: duckdb source pin HEAD == b6013f7aa2f36caabb9466ce1791667ea8a280bb
PASS: hello-world C++-EH compiled+linked through wrapper (wasm32 magic)
PASS: EH probe ran under node WASI: EH-OK probe, exit 0
VERIFY-OK: exit 0
```


## Non-vacuity (gate can exit 1) — real runs

- Empty subject: `STAGING=/tmp/wasi-vac ... --verify-only` -> **exit 1**, "FAIL: 16 check(s) failed".
- Tampered libc++.a (`printf x >>`): -> **exit 1**, first typed FAIL
  "libclang_rt.builtins.a missing or sha256 mismatch" (then libc++ mismatch).
- Wasmtime 48 cannot run the legacy-EH module: "legacy_exceptions feature required
  for try instruction" (typed, witnessed), node 24 (V8) runs it: `EH-OK probe`, exit 0.

## Forensic pinning (how the recipe was recovered)

- staged libc++/libc++abi/libunwind are byte-identical (sha256) to emsdk
  `-wasmexcept` variants — confirmed by hash equality, not assumption.
- libduckdb_wasi_compat.a == compiled wasi_posix.c (llvm-nm symbol set match:
  getpid/getrlimit/kill/madvise/mlock/mmap/msync + U __errno_location).
- emsdk: 6.0.13, clang 24.0.0git (83ee07799...), node 24.19.0, wasmtime 48.0.1.
- duckdb pin verified: `git -C /Users/sac/rel4r/duckdb-src rev-parse HEAD` ==
  b6013f7aa2f36caabb9466ce1791667ea8a28 0bb.

## Deltas this session applied to the live staging (all additive, disclosed)

1. libclang_rt.builtins.a + wasi libc.a/crt1-command.o (+ crt1.o copy) staged;
   plus `share/wasi-sysroot` merge and wrapper retarget. Root cause (typed):
   emsdk's clang resource dir has no wasm32-unknown-wasip1 builtins and the emsdk
   sysroot has no wasi libc — standalone clang links failed before
   ("cannot open crt1.o" / "unable to find library -lc").
2. **libpthread.a** staged: duckdb CMakeLists:9 `find_package(Threads REQUIRED)`
   cannot resolve -lpthread on wasi libc alone (witnessed in clean-room attempt 2
   configure). Stubs extracted from libc.a's pthread_*.c.obj + two explicit stubs
   (pthread_exit/pthread_atfork not defined by wasi libc at this pin).
3. Wrapper retargeted to the merged sysroot.

## Clean-room status

- Attempt 3: configure OK after the libpthread fix, build FAILED sccache launcher
  (typed: "sccache: error: ... unknown target triple 'unknown'"; duckdb
  CMakeLists:83/93 find_program(ccache sccache) — neutralized with
  -DCMAKE_C/CXX_COMPILER_LAUNCHER=).
- Attempt 4: sccache neutralized, FAILED llvm-ar-as-ranlib
  ("llvm-ar: error: expected [relpos] for 'a', 'b', or 'i' modifier";
  CMAKE_RANLIB must be llvm-ranlib, not llvm-ar).
- Attempt 5: BUILD LANDED — `cmake build (duckdb_static) exit 0`,
  `produced libduckdb_static.a: 86,643,010 B, wasm32 magic`, but the
  duckdb_static target does not build the extension archives:
  `FAIL: libcore_functions_extension.a: missing or empty` / icu — the
  duckdb_static target does not depend on the extension targets, so the
  recipe now builds core_functions_extension + icu_extension explicitly.
- Attempt 6: extension targets added; the archive copy loop collapsed find
  output into one arg (cp: ... No such file or directory) — script bug, fixed
  (find | while read). The attempt-6 build tree itself was sound: copying its
  own two extension archives into the attempt-6 staging then verifies
  **exit 0** (witnessed above).
- **Attempt 7: LANDED — full script-driven clean-room, --clean-room
  /tmp/l109/wasi-cleanroom7, exit 0.** End-to-end log: /tmp/l109/cleanroom7.log.
  Embedded tail (real output):

```
== verify staging: /tmp/l109/wasi-cleanroom7
PASS: libduckdb.a: 86643010 B, wasm32 magic in 64 MiB prefix
PASS: libcore_functions_extension.a: 15946718 B, wasm32 magic in 64 MiB prefix
PASS: libicu_extension.a: 3277180 B, wasm32 magic in 64 MiB prefix
PASS: libduckdb_wasi_compat.a: 972 B, wasm32 magic in 64 MiB prefix
PASS: duckdb source pin HEAD == b6013f7aa2f36caabb9466ce1791667ea8a280bb
PASS: hello-world C++-EH compiled+linked through wrapper (wasm32 magic)
PASS: EH probe ran under node WASI: EH-OK probe, exit 0
VERIFY-OK: exit 0
```

- **Reproducibility semantics (honest)**: the clean-room archives are
  wasm32-valid, EH-runnable, and pass every structural probe, but are NOT
  byte-identical to the live staging (libduckdb.a 86,643,010 B clean-room vs
  87,620,792 B live; extension archives differ similarly). CMake/DuckDB here is
  rebuild-reproducible, not bit-reproducible. Byte-identity is a separate,
  unclaimed property.

## Live-subject concurrency incident (disclosed)

At 18:03 mid-session, `lib/wasm32-wasip1/libunwind.a` in the live staging was
replaced by another lane with a HOST libunwind (sha256 cf54f0b6..., members
Unwind-sjlj.c.o / Unwind-seh.cpp.o / UnwindRegisters*.S.o — no wasm symbols,
llvm-nm "no symbols"). That state FAILS the -fwasm-exceptions link
(witnessed: `undefined symbol: _Unwind_SetIP` from cxa_personality.o) and my
gate caught it: `FAIL: libunwind.a sha256 mismatch vs emsdk
libunwind-wasmexcept.a` -> exit 1. The foreign archive is preserved at
/tmp/l109/libunwind.foreign-cf54f0b6.a for forensics; the canonical wasmexcept
variant was restored (fix-forward, disclosed) and verify returned to exit 0.

## EH/wasmi question (coordinator, answered empirically)

- L116's finding confirmed at member granularity: libunwind-wasmexcept's single
  member Unwind-wasm.o carries the TAG section (llvm-objdump -h shows `TAG`);
  libunwind-noexcept's member is tag-free.
- **Option (b) is forced, not chosen**: linking -fwasm-exceptions code WITHOUT
  the wasmexcept libunwind fails with typed undefs
  `__cpp_exception`, `_Unwind_GetWasmLPadContext`, `_Unwind_RaiseException`
  (real wasm-ld output embedded above). The wasmexcept libunwind is REQUIRED by
  -fwasm-exceptions; a tag-free libunwind cannot close the link. Therefore the
  linked artifact carries EH code (legacy-EH `try` instructions) and is
  **wasmi-unhostable by design** — runner must be wasmtime (with -W exceptions=n? no: legacy unsupported) / node V8. This is an operator-level architecture
  decision to record; L122's re-pin recipe inherits it.
