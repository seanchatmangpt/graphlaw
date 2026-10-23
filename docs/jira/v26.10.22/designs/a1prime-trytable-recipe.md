# A1' — try_table (new-style wasm EH) rebuild recipe

Lane L144, 2026-10-22. Read-only design lane; graphlaw code untouched; no
duckdb build run. Goal: the DuckDB-linked `graphlaw-wasm` artifact emits
**new-style wasm EH (try_table / catch_ref / throw_ref, exnref)** uniformly —
zero legacy try/delegate/rethrow opcodes — so both V8 and modern-wasmtime host
it.

## 1. Toolchain ground truth (verified locally via flag probes; no duckdb build)

Toolchain: emsdk at `/Users/sac/emsdk`; clang 24.0.0git (LLVM
83ee07799c341b60965f07892f7bb3d175da05be), LLD 24.0.0 same rev.

| Probe | Result |
|---|---|
| `clang -mllvm -wasm-use-legacy-eh=0` (compile, trivial obj) | accepted, rc=0 |
| `-fwasm-exceptions-semantics=new` | does not exist — `clang: error: unknown argument` |
| `-mllvm -wasm-use-eh-instrs` | does not exist — `Unknown command line argument` |
| `wasm-ld -mllvm -wasm-use-legacy-eh=0` passthrough | accepted, rc=0 |
| EH object compile via staged wrapper `~/.frozen-duckdb/cache/v2.0.0-dev-wasm32-wasip1/bin/clang`, `llvm-objdump` census | default flags → `try(4) catch_all(8) delegate(2) rethrow(4)`; with `-mllvm -wasm-use-legacy-eh=0` → `try_table(12) catch_ref(1) throw_ref(4) exnref(8)` — clean opcode-level flip |
| Full probe link + run (real `main`: throw std::runtime_error, catch, return 0; staged `-wasmexcept` runtime libs) | pure try_table module: **node 24.19.0 (V8 13.6) WASI run exit 0**; **wasmtime 48.0.1 exit 0** |
| Differential (same source, default flags → legacy EH) | legacy module wasmtime 48 **exit 1** (legacy-exceptions refusal); node 24 exit 0 |

Conclusion: clang 24 supports `-mllvm -wasm-use-legacy-eh=0` at BOTH compile
and link. `-fwasm-exceptions-semantics=new` and `-wasm-use-eh-instrs` do NOT
exist in this toolchain; the emscripten-plumbed `-mllvm -wasm-use-legacy-eh=0`
is the supported surface. Note: legacy-vs-new is NOT a target-feature bit —
`target_features` says `+exception-handling` in both modes; the discriminator
is opcode encoding, so the section walk is the only reliable gate.

## 2. The flag set

### 2.1 Compile level (duckdb + extension archives)

Diff to `/Users/sac/frozen-duckdb/scripts/build_wasi_duckdb.sh`,
`build_archives()`:

```diff
@@ cmake configure in build_archives()
-    -DCMAKE_CXX_FLAGS="-O3 -fwasm-exceptions" \
+    -DCMAKE_CXX_FLAGS="-O3 -fwasm-exceptions -mllvm -wasm-use-legacy-eh=0" \
+    -DCMAKE_C_FLAGS="-O2 -mllvm -wasm-use-legacy-eh=0" \
```

`stage_runtime()` is UNCHANGED: emsdk `tools/system_libs.py` (lines ~800–850)
defines the `-wasmexcept` suffix = `Exceptions.WASM` =
`-fwasm-exceptions -sWASM_LEGACY_EXCEPTIONS=0` = **new EH**. The staged
`libc++/libc++abi/libunwind` copies of
`$EMSDK/upstream/emscripten/cache/sysroot/lib/wasm32-emscripten/*-wasmexcept.a`
are already new-EH. No separate "new-EH unwinder" artifact exists in this
emsdk: `libunwind-wasmexcept.a` IS the new-EH unwinder; `*-legacyexcept.a` is
the legacy variant and must not be staged. (Archive-extraction census of the
.a members was permission-denied in this lane; the source-level proof stands,
and the builder lane must census the extracted members per §3.1 — carried as
R2b.)

### 2.2 Rust side (graphlaw `wasm/`, frozen-duckdb-sys)

Rust wasip1 is panic=abort: no unwinding crosses the Rust/C++ boundary, so
Rust objects should census zero EH opcodes either way. If EH-bearing Rust code
is ever needed, the route is
`CARGO_TARGET_WASM32_WASIP1_RUSTFLAGS="-Zllvm-args=-mllvm,-wasm-use-legacy-eh=0"`.
UNVERIFIABLE-LOCALLY: whether rustc 1.96 accepts `-Z llvm-args` on stable
(no nightly toolchain here) — carried as R2c; census the rustc staticlib
objects in the walk regardless.

### 2.3 Link level

wasm-ld consumes `-mllvm -wasm-use-legacy-eh=0` (verified rc=0). The final
graphlaw link is rustc-driven; pass via

```text
-C link-arg=-mllvm -C link-arg=-wasm-use-legacy-eh=0
```

wasm-ld merges feature sections but does NOT normalize opcodes: any input
object carrying legacy try/delegate/rethrow propagates to the output. That is
exactly what the §3.1 walk catches.

## 3. Verification plan

### 3.1 Section walk (primary gate; add to verify_staging)

Per produced archive member and the final .wasm:

```bash
# legacy-free: expect 0 matches
llvm-objdump -d <f> | grep -cE '\btry\b|delegate|rethrow'
# new-EH present where EH is used: expect >0
wasm-tools print <f> | grep -cE 'try_table'
wasm-tools print <f> | grep -cE 'catch_ref|throw_ref|exnref'
```

Gate rule: any legacy opcode = FAIL. try_table required wherever
catch_ref/throw_ref/exnref appear.

Non-vacuity is differential, not self-claimed: the same probe source compiled
with and without the flag. Legacy build MUST fail the walk (witnessed:
wasmtime 48 refuses it, exit 1); flagged build passes (witnessed: try_table
census >0, both hosts exit 0). A court that cannot flip is the failure — the
flip is witnessed in §1.

### 3.2 Host probes

- V8: emsdk node 24.19.0 node:wasi run — gate `EH-OK probe, exit 0` (same
  harness as `run_eh_smoke` in build_wasi_duckdb.sh). Verified exit 0 on the
  try_table probe.
- wasmtime 48.0.1: `wasmtime run` exit 0 on try_table artifact (verified);
  exit 1 on the legacy artifact (verified) — the flip is the non-vacuity
  witness.

### 3.3 Acceptance gate

gws4-class court: `/Users/sac/graphlaw/tests/gws4_ipc_injection_court.rs` —
the DuckDB-linked artifact must pass the IPC-injection court end to end under
try_table EH, plus `cargo test -p graphlaw-wasm` (test-ladder rung 6).

## 4. Risk register

| # | Risk | Status / mitigation |
|---|---|---|
| R1 | duckdb C++ EH under new EH needs a recent host (exnref) | Verified floor: node 24.19.0 (V8 13.6) and wasmtime 48.0.1 both run the try_table/exnref probe exit 0. Chrome floor (Chrome 137 / V8 13.7 shipped exnref EH) is UNVERIFIABLE-LOCALLY — no Chrome in this lane. |
| R2 | Runtime-lib variant staging | Closed by source reading: `-wasmexcept` = new EH; staging already uses it, sha-pinned in verify_staging. |
| R2b | Runtime archives may still contain legacy opcodes | UNVERIFIABLE-LOCALLY (archive extraction denied this lane). Carried: builder lane must llvm-ar x each staged .a and census members per §3.1. |
| R2c | rustc `-Z llvm-args` acceptance on stable 1.96 | UNVERIFIABLE-LOCALLY (no nightly). Moot unless Rust EH is introduced; census rustc objects anyway. |
| R3 | sccache/ccache poisoning (warm entries compiled without the flag) | Carried from L1/L109 receipts: launchers stay empty (`unset SCCACHE CCACHE`, `CMAKE_*_COMPILER_LAUNCHER=`), TMPDIR scoped. The §3.1 walk is the poison detector. |
| R4 | Feature-section gates are blind to legacy-vs-new | target_features says `+exception-handling` in both modes; do not use feature checks as the gate — opcode census only. |

## 5. Receipt — files read and probes run

Files read:
- `/Users/sac/frozen-duckdb/scripts/build_wasi_duckdb.sh` (full)
- `/Users/sac/emsdk/upstream/emscripten/tools/building.py` (lines 110–135, ~330–340)
- `/Users/sac/emsdk/upstream/emscripten/tools/system_libs.py` (lines 800–900)
- `/Users/sac/graphlaw/tests/gws4_ipc_injection_court.rs` (existence, -l match)
- graphlaw `CLAUDE.md` test ladder

Probes (all rc/counts real, this session; zero duckdb compilation):
- clang flag-acceptance probes: table §1, rc as listed.
- staged-wrapper EH object compiles rc=0, census counts as listed.
- probe links (hello-world C++ EH against staged runtime libs) rc=0; census:
  try_table 12 / catch_ref 1 / throw_ref 4 / exnref 8 (flagged) vs try 4 /
  delegate 2 / rethrow 4 (default).
- node 24.19.0 WASI: flagged exit 0, legacy exit 0, legacy wasmtime exit 1.
- Not run (typed): duckdb build, gws4 court, cargo any rung — design lane.
