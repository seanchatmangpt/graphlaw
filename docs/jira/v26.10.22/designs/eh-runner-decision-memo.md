# EH Runner Decision Memo — operator decision required

Date: 2026-10-22 · Lane: L143 (docs lane, read-only elsewhere) · Repo: `/Users/sac/graphlaw`
Status: **DECISION REQUIRED** — no route has been implemented; this memo recommends, it does not land.

## 1. The witnessed wall

The DuckDB-bearing wasm artifact is unhostable by wasmi. Each claim is receipted:

| # | Claim | Evidence (receipt, real output) |
|---|---|---|
| W1 | Link now succeeds but produces a module carrying a wasm **Tag section** (section id 13, one tag `__cpp_exception`); wasmi's validator rejects it: `exceptions proposal not enabled` | `docs/jira/v26.10.22/designs/gws4-postlink-receipt.md:26-33` |
| W2 | wasmi 2.0.0 `Config::default_features()` omits `EXCEPTIONS` and exposes **no setter** for it (public surface: `mutable_global` .. `enforced_limits` only) | `gws4-postlink-receipt.md:37-40` (wasmi-2.0.0 `src/engine/config.rs:78-95`) |
| W3 | The tag section traces to the **wasi-sdk `libunwind.a`**; every other archive member (libduckdb.a 246 objs, icu, core_functions, libc++/libc++abi, rustc std rlibs) is tag-free | `gws4-postlink-receipt.md:42-50` |
| W4 | Linking `-fwasm-exceptions` code **without** the wasmexcept libunwind fails with typed undefs (`__cpp_exception`, `_Unwind_GetWasmLPadContext`, `_Unwind_RaiseException`) — the wasmexcept libunwind is REQUIRED; a tag-free libunwind cannot close the link | `docs/jira/v26.10.22/designs/wasi-sdk-reproducibility.md:144-148` (real wasm-ld output embedded); member-level TAG isolation `:141-143`; independently witnessed by a live-subject incident where a host libunwind broke the link (`:127-137`) |
| W5 | The engine module is **wasmi-unhostable by design** — it carries legacy-EH `try` instructions | `wasi-sdk-reproducibility.md:149-151` |
| W6 | **wasmtime 48 cannot run the legacy-EH module either** ("legacy_exceptions feature required for try instruction", typed, witnessed); **node 24 (V8) runs it**: EH probe `EH-OK probe, exit 0` | `wasi-sdk-reproducibility.md:58-59` |
| W7 | End-to-end defect witness: `graphlaw-run` (wasmi) exits 95 `exceptions proposal not enabled` on the lane artifact; smalt real-runner court 3/7 (gates 1/1b/1c/3 fail, same exit 95); reproduced 3x | `docs/jira/v26.10.22/designs/smalt-pairing-receipt.md:47-57, 61-72, 121-124` |
| W8 | Rebuild with `RUSTFLAGS="-C target-feature=-exception-handling"` does NOT help — same exit 95 — the EH bytes come from the frozen-duckdb/libunwind blobs, not rustc | `smalt-pairing-receipt.md:56-57` |
| W9 | Positive control: the **pre-duckdb generation** (beam4pm copy, 6,655,651 B `8b4f4bc8…`; the 6.79 MB registry pin `332ea218…` is that generation) loads in wasmi, serves `capabilities` and wraps replies in the stub-mode envelope over real packet-4 frames | `smalt-pairing-receipt.md:74-87, 100-104` |
| W10 | The runner feature exists and is host-only: `cargo build --bin graphlaw-run --features runner` exit 0; wasm host contract 9 passed / 0 failed | `smalt-pairing-receipt.md:26-32` |

W1-W5 force the conclusion: **graphlaw-run and `tests/wasm_abi.rs` (both wasmi-bound) cannot host the engine module** (`smalt-pairing-receipt.md:91-99`, finding 1). W8 rules out a quick Rust-side fix. W6 is the pivot: the only witnessed EH-capable hosts are node 24 (V8) and wasmtime-with-legacy_exceptions (config, not witnessed working — W6 records it failing at default config).

## 2. Routes with honest costs

### Route A — EH-capable host for the engine module

Graphlaw keeps the DuckDB module as-is; the host gains EH support.

- **A1 (node WASI)**: witnessed working (W6). Cost: graphlaw-run gains a second host backend (node/V8 backend beside wasmi), or the runner ships a node-based shim. Node is a heavy runtime dependency for a Rust bin.
- **A2 (wasmtime + legacy_exceptions)**: wasmtime supports `Config::wasm_exceptions(true)` for the legacy EH proposal (cited as R2 in `gws4-postlink-receipt.md:64-65`); default wasmtime 48 is witnessed FAILING (W6), so the config flag must be witnessed before this route is believed. Cost: `runner` feature adds a wasmtime-wasi dependency (host-only, feature already exists — W10); heavier binary, config must be set and courted.
- Risk: either backend must preserve graphlaw-run's typed fault modes (exit 84 digest refusal, crash 86, hang timeout — W7) so smalt's courts keep their fail-closed semantics.
- First implementation step (A2): witness a wasmtime `legacy_exceptions`-enabled host loading the 45 MB artifact (sha256 `500333a4…`) and serving one real eval frame; if the flag does not exist / does not work for legacy EH, the route dies loudly and A1 is the survivor.

### Route B — rebuild DuckDB wasm without C++ exception handling

Rebuild the frozen-duckdb blobs with `-fno-exceptions` (or a wasi-sdk unwinder variant) so the linked module is tag-free and wasmi-valid.

- Cost: large build change to `/Users/sac/frozen-duckdb/scripts/build_wasi_duckdb.sh` and the CMake recipe; **risk unknown, possibly breaks duckdb internals** — duckdb is heavily C++-exception-based, so this may be UNSUPPORTED(BUILD_SEMANTICS) rather than a build flag. W4 already proves the honest link-side knob does not exist (excluding libunwind.a fails the link with typed undefs).
- Upside if it works: wasmi stays the single host; no new runtime dependency.
- First implementation step: compile a duckdb translation unit with `-fno-exceptions` against the staging and link a minimal C++-EH-free binary; the first `error:` from duckdb internals is the typed falsifier.

### Route C — dual-artifact policy

wasmi hosts the **pure-Rust module** (pre-duckdb generation, ~6.8 MB, witnessed serving real frames over packet-4 — W9) for the court tier; node/wasmtime (Route A backend) hosts the engine module for the engine tier.

- Cost: two artifacts with two pins and two host runtimes to keep green; the registry pin must be split (the current single pin `332ea218…` collides generations — W9, `smalt-pairing-receipt.md:100-104`).
- Aligns with repo law: Chicago courts run the real module per tier — the court tier runs a real module (just not the duckdb one); the engine tier runs the real engine module on a real EH host. No fake passes as long as each court names its artifact generation.
- Risk: a court could silently claim more than its tier proves — mitigation is a required `--expect-blake3` pin per tier (mechanism already exists, `smalt-pairing-receipt.md:18`).
- First implementation step: split the registry pin into two named generations and add the generation name to graphlaw-run's load banner.

## 3. Recommendation

**Route C (dual-artifact policy) with Route A2 (wasmtime legacy_exceptions) as the engine-tier backend**, once A2's first step witnesses it. Reasoning per repo laws:

1. **Chicago courts must run the real module** — Route C keeps a real module on every court, with the tier named; Route B risks a duckdb-semantics fake pass ("compiles without exceptions" ≠ "duckdb behaves identically"), the classic laundering shape this repo's courts exist to kill.
2. **Fail-closed** — every falsifier above is typed and loud: A2 dies loudly if the flag is absent (W6), B dies loudly at the first duckdb compile error, C's tier mismatch is detectable via the per-tier `--expect-blake3` pin. No route relies on an unwitnessed assumption to pass.
3. **No fake passes** — W8 already refuted the cheap fix (Rust-side EH-disable); W4 refuted the cheap link-side fix; nothing cheaper than C remains.
4. C is **reversible and additive** (pin split + backend flag) while B is a large irreversible build change with unknown risk; under ERRC asymmetry, the tighten-and-split (C) outranks the rebuild (B).

Route B is not forbidden — it is ranked last: attempt it only if the operator wants wasmi as sole host and accepts the risk that duckdb's C++ internals may be UNSUPPORTED without exceptions.

## 4. DECISION REQUIRED

Operator picks the route. First steps, per route:

- **A1**: add a node/V8 backend to graphlaw-run (witnessed EH-OK, W6); step = `graphlaw-run --host node` loading the 45 MB artifact and serving one real eval frame.
`--host node` is a new flag; the 45 MB artifact is the L121/L116 build (`940af4c7…` / `500333a4…`).
- **A2**: witness wasmtime `legacy_exceptions` loading the artifact + one real frame; if the flag is absent or fails for legacy EH, route dies loudly and A1 survives.
- **B**: minimal `-fno-exceptions` duckdb translation unit + link; first duckdb-internal compile error is the typed falsifier.
- **C**: split the registry pin into two named generations (pre-duckdb court tier / duckdb engine tier); step = registry split + generation name in graphlaw-run's load banner, then witness each tier on its pinned artifact.

Until the operator picks, status stays **DECISION REQUIRED**; no route has a landed step.

## AMENDMENT (L126 stand-down, 2026-10-22 18:4x) — Route A1 refuted for the full artifact

L126's end-to-end witnesses sharpen the decision:
- The linked artifact's EH is MIXED legacy+new: real try/catch/rethrow and
  a `(param i32)` tag from DuckDB C++ `-fwasm-exceptions`, PLUS the legacy
  try from the staged libunwind. node 26 (V8) REJECTS it: "module uses a
  mix of legacy and new exception handling instructions" — L109's node
  EH-OK probe was hello-world only and does not extend.
- wasmtime 48: "legacy_exceptions feature required"; the -W flag no
  longer exists in 48. wasmi 2.0.0: no exceptions knob, period.
- All staged C++/extension archives are member-level tag-free; the Tag
  enters from DuckDB's own EH code + the libunwind try.

Consequence: the artifact as built is hostable by NO available runtime.
Live routes now:
- A1' — emsdk rebuild of duckdb emitting NEW-style try_table uniformly
  (`-mllvm -wasm-use-legacy-eh=0`): V8 and modern wasmtime host it;
  wasmi still cannot. (Build-side change, wasmi court tier needs route C
  regardless.)
- A3 — an old wasmtime/wasmer with legacy-EH support (pin an old runtime;
  freezing on EOL tooling).
- B — DuckDB -fno-exceptions rebuild (unchanged, risk unknown).
- C — dual-artifact (unchanged; now the ONLY route that keeps the wasmi
  court tier hosting a real module short-term).
Recommendation stands: C, with A1' as the engine-tier backend candidate.

## FINAL AMENDMENT (N28, 2026-10-22 ~22:1x) — the host question is closed

N28's empirical work with the real 45 MB artifact settles the options:
- The test host is wasmi 2.0.0 (tests/common/mod.rs:20), NOT wasmtime —
  L110's wasmtime::Config fix named an API absent from the tree.
- wasmi 2.0.0 has 14 public Config setters, none for exceptions; the
  default feature set omits both EH proposals unconditionally.
- The artifact's DuckDB C++ objects emit LEGACY try/catch: section-stripped
  → "legacy exceptions support is not enabled"; no transform fixes it.
- No mainstream Rust runtime executes legacy EH (wasmi: none; wasmtime 48:
  new-EH only). L130's -fno-exceptions archive validates under wasmi but
  fails at first gl_call (noeh-ABI defect in __funcs_on_exit).
- Binaryen exnref translation + V8 --wasm-allow-mixed-eh-for-testing:
  traps in a duckdb vtable call_indirect even on capabilities.

ROUTES NOW:
- (a) -fno-exceptions DuckDB rebuild: archive validates, ABI broken at
  gl_call — needs a noeh-compatible DuckDB build (deep C++ change).
- (b) legacy-EH-capable host: no mainstream Rust runtime; node 24 EH-OK
  probe was hello-world only; full artifact is a legacy/exnref MIX that
  V8 rejects.
- (c) A1' try_table rebuild (L144's witnessed recipe): wasmtime 48 + node
  24 BOTH host try_table builds (exit 0 probes) — the ONLY witnessed
  hostable path for the engine artifact.
- (d) PURE-tier production (dual-artifact): wasmi hosts the 6.79 MB
  pre-duckdb module (frame-witnessed); engine-tier PIN-ONLY typed state.

The witness table: wasmi 2.0.0 rejects Tag sections (N28/L116/M4/S5);
wasmtime 48 accepts try_table, rejects legacy (L144/N22); V8 rejects the
mix (N22/L125); the noeh archive traps (L130). OD-1 decision input is
complete.
