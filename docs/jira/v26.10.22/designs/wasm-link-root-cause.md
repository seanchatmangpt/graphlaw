# WASM-LINK-RC — four-hop wasm32-wasip1 link root cause (consolidated receipt)

- **Id**: WASM-LINK-RC
- **Title**: Root cause chain for the wasm32-wasip1 link failure (`rust-lld: error: unable to find library -lduckdb`) and the EH consequence chain
- **Status**: DOCUMENTED (root cause closed on disk; EH runner decision open — see §EH)
- **Conformance target**: `bash scripts/mutation_court.sh` anti-vacuity law applies to any gate claiming closure; this document files no gate of its own — it consolidates witnessed receipts, every claim cited to an on-disk source.
- **Sources (all on disk, read 2026-10-22)**: `docs/WASM.md`; `docs/jira/v26.10.22/designs/link-green-receipts.md`; `docs/jira/v26.10.22/designs/gws4-postlink-receipt.md`; `docs/jira/v26.10.22/designs/wasi-sdk-reproducibility.md`; `docs/jira/v26.10.22/designs/eh-runner-decision-memo.md` (+ its L126 amendment). No new claims.

## Current behavior (as-witnessed)

- The original red: `rust-lld: error: unable to find library -lduckdb` (`link-green-receipts.md:22`).
- The link is now green: `cargo build -p graphlaw-wasm --target wasm32-wasip1 --profile wasm` exits 0 with a wasi-sdk root on `WASI_SDK_PATH` (`docs/WASM.md:42-49`; `link-green-receipts.md:16-21`). Lane-build witness 45,162,366 B / BLAKE3 `9d299ed8…` — a link witness, not a re-pin; the registry pin stays `332ea218…` 6,793,617 B, the 26.10.21 artifact (`docs/WASM.md:89-106`).
- The linked module is wasmi-unhostable by design (EH tag section) — decision memo required; see §EH below.

## Required behavior

Documentation-only consolidation (lane N4). No code, no build, no pin change. Each hop below must resolve to its cited source line on disk.

---

## Hop 1 — dylib emitted unconditionally on the host-only path → wasi branch

frozen-duckdb-sys emitted `dylib=duckdb` unconditionally, which is host-only, pushing the wasi target onto a branch that could not link (`link-green-receipts.md:25-26`). The current runbook documents the resulting requirement: `WASI_SDK_PATH` must point at a wasi-sdk root (`bin/clang`, `share/cmake/wasi-sdk-p1.cmake`); without it the build fails typed at frozen-duckdb-sys `build.rs` (`WASI_SDK_PATH required`) (`docs/WASM.md:19-32`, `docs/WASM.md:49-51`).

## Hop 2 — crates.io convergence (2.0.2; local patch superseded)

graphlaw consumed frozen-duckdb from crates.io, which froze checkout edits; the fix at the time was `[patch.crates-io]` plus a lock re-point. This was superseded by the upstream 2.0.2 release — the lock now carries 2.0.2 and the patch entries were removed; 2.0.2 works without them (`link-green-receipts.md:27-29`). Current-state corroboration: frozen-duckdb 2.0.2 emits `static=duckdb` and links `libduckdb_wasi_posix` plus wasi-sdk libc++ (`docs/WASM.md:48-49`).

## Hop 3 — `archive_is_wasm32` 256 KiB clamp behind the 10.6 MB symbol table

`archive_is_wasm32`'s 256 KiB probe window sat behind a 10.6 MB ar symbol table and falsely rejected a GENUINE wasm32 archive (first `\0asm` offset 10596838). Fix: widened probe in `frozen-duckdb-builder/src/lib.rs` (`link-green-receipts.md:30-33`).

**Caveat on the census (must not be lost)**: `file(1)` cannot distinguish wasm from host archives — it prints "current ar archive" either way. The earlier "host archive" census reading was therefore wrong; the real blocker was the validator clamp, not a wrong-triple archive (`link-green-receipts.md:33-36`). The staging verify scripts avoid this trap by probing for wasm32 magic inside a 64 MiB prefix instead of trusting `file(1)` (`wasi-sdk-reproducibility.md:42-45`, `wasi-sdk-reproducibility.md:110-113`).

## Hop 4 — wasi libc missing → sysroot-aware clang wrapper

`clang --target=wasm32-wasip1` had no wasi libc (`errno.h` not found), so the recipe stages a sysroot-aware wrapper over emsdk clang 24 + emscripten sysroot; libc++/libc++abi/libunwind (wasmexcept variants) staged alongside (`link-green-receipts.md:37-39`). The wrapper is `exec <emsdk>/upstream/bin/clang --sysroot=<staging>/share/wasi-sysroot "$@"` with `clang++`/`llvm-ar`/`wasm-ld` symlinks (`wasi-sdk-reproducibility.md:11-13`), and the sysroot merge is why the clang driver finds `crt1.o` and `-lc` (`wasi-sdk-reproducibility.md:20-21`). Additive staging this session, root cause typed: emsdk's clang resource dir has no wasm32-unknown-wasip1 builtins and the emsdk sysroot has no wasi libc — standalone clang links failed before with `cannot open crt1.o` / `unable to find library -lc`; plus a staged `libpthread.a` because duckdb `CMakeLists:9 find_package(Threads REQUIRED)` cannot resolve `-lpthread` on wasi libc alone (`wasi-sdk-reproducibility.md:71-82`). Script end-to-end: clean-room attempt 7 exit 0 (`wasi-sdk-reproducibility.md:104-118`); honest reproducibility semantics: rebuild-reproducible, NOT bit-reproducible (`wasi-sdk-reproducibility.md:120-125`).

Build-recipe pitfalls carried with hop 3/4 (recorded on the link-green entry): sccache cannot handle the wasm triple (disable launcher); unity builds break under the /tmp symlink (configure under /private/tmp) (`link-green-receipts.md:44-46`; typed sccache failure at `wasi-sdk-reproducibility.md:86-89`, ranlib fix `:90-92`).

Archive identity for the record: DuckDB source pin `/Users/sac/rel4r/duckdb-src` @ b6013f7aa2, compiled `-fwasm-exceptions -DISABLE_THREADS=ON -DUCKDB_CAPABILITIES=none` (staticlib + core_functions + icu), 87.6 MB libduckdb.a (`link-green-receipts.md:41-44`; pin re-verified `wasi-sdk-reproducibility.md:46`, `:68-69`).

---

## The EH chapter — the link's consequence chain (decision open)

The green link did not end the story: the produced module carries exception-handling bytes that no available runtime hosts. Chain, each claim receipted:

1. **Tag section from libunwind**: the linked module contains a wasm Tag section (section id 13, one tag, attribute 0, `__cpp_exception`, typeidx 3); wasmi's validator rejects it — `exceptions proposal not enabled` (wasmparser 0.228.0, validator/core.rs:958) (`gws4-postlink-receipt.md:24-33`).
2. **wasmi no-toggle**: wasmi 2.0.0 `Config::default_features()` omits `WasmFeatures::EXCEPTIONS` and exposes no setter to enable it (`src/engine/config.rs:78-95`; public surface `mutable_global`..`enforced_limits` only) (`gws4-postlink-receipt.md:37-40`).
3. **Member-isolated libunwind**: the tag traces to the wasi-sdk `libunwind.a`; negative controls parsed member-by-member — staged libduckdb.a (246 objs), icu (31), core_functions (25), wasi_compat (1), SDK libc++.a (57), libc++abi.a (18), all rustc wasm32-wasip1 std rlibs — zero tag sections; only libunwind.a has one (`gws4-postlink-receipt.md:42-50`). Member granularity: libunwind-wasmexcept's single member `Unwind-wasm.o` carries the TAG; libunwind-noexcept's member is tag-free (`wasi-sdk-reproducibility.md:139-143`).
4. **Member-isolated libunwind is forced, not chosen**: linking `-fwasm-exceptions` code WITHOUT the wasmexcept libunwind fails with typed undefs `__cpp_exception`, `_Unwind_GetWasmLPadContext`, `_Unwind_RaiseException` — a tag-free libunwind cannot close the link (`wasi-sdk-reproducibility.md:144-148`). Independently witnessed: a live-subject incident where another lane swapped in a HOST libunwind broke the link (`undefined symbol: _Unwind_SetIP`) and the gate caught it (`FAIL: libunwind.a sha256 mismatch`) (`wasi-sdk-reproducibility.md:127-137`).
5. **Mixed-EH V8 rejection**: L126's end-to-end witnesses show the artifact's EH is MIXED legacy+new — real try/catch/rethrow and a `(param i32)` tag from DuckDB C++ `-fwasm-exceptions`, plus the legacy try from staged libunwind. node 26 (V8) REJECTS it: "module uses a mix of legacy and new exception handling instructions" — L109's node EH-OK probe was hello-world only and does not extend (`eh-runner-decision-memo.md:76-83`). Also: wasmtime 48 says "legacy_exceptions feature required" and the `-W` flag no longer exists in 48; wasmi 2.0.0 has no exceptions knob, period (`eh-runner-decision-memo.md:84-85`). Earlier witness: wasmtime 48 failed at default config while node 24 ran the hello-world probe (`wasi-sdk-reproducibility.md:58-59`).
6. **Hostable by no runtime**: consequence, verbatim — "the artifact as built is hostable by NO available runtime" (`eh-runner-decision-memo.md:89`). Corroborating end-to-end defect witnesses: `graphlaw-run` (wasmi) exits 95 `exceptions proposal not enabled`; rebuild with `RUSTFLAGS="-C target-feature=-exception-handling"` does not help (same exit 95 — EH bytes come from the frozen-duckdb/libunwind blobs, not rustc); positive control: the pre-duckdb generation loads in wasmi (`eh-runner-decision-memo.md:18-21`).
7. **Decision memo pointer**: routes and costs live in `docs/jira/v26.10.22/designs/eh-runner-decision-memo.md` — Route A (EH-capable host: A1 node / A2 wasmtime legacy_exceptions), Route B (duckdb `-fno-exceptions` rebuild, risk unknown), Route C (dual-artifact policy) (`eh-runner-decision-memo.md:27-51`); the L126 amendment sharpens to A1' (new-style try_table rebuild), A3 (pinned legacy-EH runtime), B, C — with C the only route keeping the wasmi court tier on a real module short-term (`eh-runner-decision-memo.md:90-100`). Status: **DECISION REQUIRED**; no route has a landed step (`eh-runner-decision-memo.md:64-74`).

---

## Acceptance

Citation-resolution check only (docs lane, no build):

```sh
grep -n "unable to find library -lduckdb" docs/jira/v26.10.22/designs/link-green-receipts.md
grep -n "256 KiB\|10596838\|current ar archive" docs/jira/v26.10.22/designs/link-green-receipts.md
grep -n "exceptions proposal not enabled\|WasmFeatures::EXCEPTIONS\|libunwind.a" docs/jira/v26.10.22/designs/gws4-postlink-receipt.md
grep -n "wasi-sysroot\|libpthread\|Unwind-wasm.o\|_Unwind_RaiseException\|hostable by NO" docs/jira/v26.10.22/designs/wasi-sdk-reproducibility.md docs/jira/v26.10.22/designs/eh-runner-decision-memo.md
grep -n "mix of legacy and new\|DECISION REQUIRED" docs/jira/v26.10.22/designs/eh-runner-decision-memo.md
grep -n "wasi-sdk root\|332ea218" docs/WASM.md
```

All must exit 0. Run below (real output).
