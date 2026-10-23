# M4 — wasm boundary receipt (lane L149, filed 2026-10-22 ~21:0x PDT)

Lane L149 files M4's wasm-boundary verdict. Subject tree: `615dbfa` +
working set, canonical checkout `/Users/sac/graphlaw` (coordinator owns
git; this file is this lane's only new write, plus the P5/P6 + M4-row
flips named at the end).

## Verdict: FILED — PASS with two typed BLOCKED rungs

## 1. Size ceiling — PASS (re-measured by this lane at file time)

Ceiling: 64 MiB = 67,108,864 B. Lane artifact re-inspected on disk at
use time (not recalled from the dispatch):

    /tmp/gl-target/wasm32-wasip1/wasm/graphlaw_wasm.wasm
    45,162,113 B (mtime Oct 22 17:21)   <= 67,108,864 B  PASS (3/3 ceiling rungs: profile, deps copy, --whole-archive free — the artifact under ceiling is the only claim under test here; the dispatch's 3/3 stands)
    sha256 = c06b2e8cb0474b8e3dde787b1c6f2b91a8982471721a8531500692e09a3f36f0

Note (no-overclaiming): the dispatch transcribed 45,179,203 B; the
on-disk byte count at file time is 45,162,113 B. Both are far under the
ceiling; the discrepancy is recorded, not reconciled. This lane files
the on-disk number as the receipt of record.

## 2. Fat-fixture non-vacuity — witnessed (lane-run, transcribed)

The build's non-vacuity control: the artifact carries the fat DuckDB
fixture; a thin/empty module would fail the ceiling-court's shape
asserts. Witnessed by the M4 lane run at 17:21; the sha above is this
lane's re-hash of that same artifact at file time.

## 3. Missing-artifact typed panic — witnessed without touching priv/

Lane witness design: rename/no-op nothing in `priv/`; the loader path
exercises the typed missing-artifact panic against a non-existent
path argument only. priv/ state verified clean at file time:

    git status --porcelain priv/  -> (empty, clean vs HEAD 6b1c7d82…)
    git log -1 --format=%H -- priv/ -> 6b1c7d8289d137e3e1e9d6751752639ba4459927
    sha256(priv/graphlaw.wasm)     =
      20c5b3f6489447e41a566e9d5909a4f5c0c08d2b8b0c855c422ce0d1fde73d82
      graphlaw.wasm.blake3 e93e8b99775b0de1da9dd356f5bd316ef1b9e74ccf235c87391ecf3dea218eb1
    mtimes 12:56 — predate every M4-lane run (17:16/17:21): untouched.

Before/after shas match (before per the lane's run log, after re-read
by this lane): no priv/ mutation.

## 4. wasi_patch_guard — 3/3 + 2/2 (re-executed by this lane)

    CARGO_TARGET_DIR=/tmp/gl-target cargo test --test wasi_patch_guard
      -> ok. 3 passed; 0 failed (0.00s):
         wasi_clock_patch_cfg_skip_without_feature
         wasi_patch_guard_skipped_without_the_feature   (loud named skip arm)
         wasi_clock_patch_cfg_is_applied

    CARGO_TARGET_DIR=/tmp/gl-target cargo test --features wasi-patched-deps \
        --test wasi_patch_guard
      -> ok. 2 passed; 0 failed (0.00s):
         wasi_clock_patch_cfg_is_applied
         wasi_patched_deps_feature_implies_vendored_patch_present

## 5. Fuzz + memory-wall — BLOCKED (EH wall, third independent corroboration)

Both rungs cannot run to completion on this toolchain: the
exception-handling personality mismatch blocks the link they consume.
Three independent on-disk corroborations:

- `designs/wasi-sdk-reproducibility.md:58` — Wasmtime 48 rejects the
  legacy-EH module ("legacy_exceptions feature required").
- `designs/wasi-sdk-reproducibility.md:131-133` — `undefined symbol:
  _Unwind_SetIP` from cxa_personality.o on the staged-SDK path.
- This receipt's toolchain finding (§6): wasi-sdk-34 emits
  `__gxx_personality_wasm0` while the crate's legacy-EH objects export
  `__gxx_wasm_personality_v0` — lld refuses the mix.

Reported as BLOCKED, not SKIP-absorbed. Route out: operator decision
memo `designs/eh-runner-decision-memo.md` (Route C + A2 legacy_exceptions
engine tier).

## 6. Toolchain finding — staged emsdk is the only green path

wasi-sdk-34 cannot complete the link: personality symbol mismatch
(`__gxx_personality_wasm0` vs legacy `__gxx_wasm_personality_v0`).
The staged emsdk toolchain remains the only observed exit-0 link path
(witnessed in `designs/link-green-receipts.md` — wasm link exit 0,
artifact `\0asm`).

## Flips performed by this lane (citing this receipt)

- `designs/integration-gonogo.md` P5 row: PENDING -> CLOSED citing
  this file.
- `designs/integration-gonogo.md` P6 row: PENDING -> CLOSED citing
  the N3 on-disk drift-note section
  (`designs/wasm4pm-fixtures-drift.md:64-146`, "Lane N3 re-derivation");
  the board's P6 "OUT" predates N3's landing.
- `../RECEIPTS.md` Wave-M M4 row: OUT -> FILED.
- `../INDEX.md` m-wave-verdicts table row: M4 OUT -> FILED.
