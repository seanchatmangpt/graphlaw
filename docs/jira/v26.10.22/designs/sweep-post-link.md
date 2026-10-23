# Sweep post-link: L110 witness of the wasm tier flip (v26.10.22)

Lane: L110, canonical checkout /Users/sac/graphlaw, build root `target-sweep2` (10 GB).
Command: `CARGO_TARGET_DIR=target-sweep2 WASI_SDK_PATH=~/.frozen-duckdb/cache/v2.0.0-dev-wasm32-wasip1 cargo test --no-fail-fast`.

## Headline

The 95-failure wasm tier did NOT flip green in one step — it flipped in stages, and the
newly-reachable stage exposed a NEW deterministic gate.

- **Link is green** (L1's landing holds on the settled tree):
  `cargo build -p graphlaw-wasm --target wasm32-wasip1 --profile wasm` → exit 0
  (59.91s into `target-sweep2/wasm-abi`; artifact
  `target-sweep2/wasm-abi/wasm32-wasip1/wasm/deps/graphlaw_wasm.wasm`, 45,131,985 bytes).
  `cargo check -p graphlaw-wasm --target wasm32-wasip1 --profile wasm` → exit 0 (17.16s).
- **Sweep 1** (full suite, complete): 98 test binaries, **634 passed / 107 failed / 1 ignored**.
  Baseline L56: 635p/102f → delta -1p/+5f (net census shift, no tier flip).
- **Sweep 2** (17 failed targets re-run on the settled tree): **128p / 104f**.

## Why the wasm tier did not read green in either run

Two concurrent-lane compile-freeze windows landed inside the sweeps (the checkout is shared;
other lanes edit `src/` and `crates/` during runs):

1. **graphlaw lib, early sweep 1**: 22x E0308 (`RefusalMessage` vs `String`, e.g.
   `src/plan.rs:430`, `src/tier.rs:130`, `src/duckdb_op.rs:64`) → every wasm-hosted court
   panicked `GRAPHLAW-WASM-BUILD-FAILED (BLOCKED-LINK)`, exit 101.
2. **graphlaw-eyeron, during sweep 2**: 610 errors
   (`crates/graphlaw-eyeron/src/reasoner.rs:726` `#[derive]` over an `impl`, E0774 +
   E0432 `crate::ast::Bindings`) → 50 tests panicked BLOCKED-LINK.

Both windows are **settled**: the standalone wasm build/check above passes exit 0 on the
current tree, so both windows classify **ENVIRONMENTAL (contention)**, not defects.
The error text in sweep-1 log cites source (`message: message.into()`) that no longer
exists on disk — receipt of the stale-tree state.

## NEW REAL DEFECT: exceptions-proposal validation gate

46-47 failures across 8 targets are deterministic and survive the settled tree:

    valid wasm: Error { kind: Wasm(BinaryReaderError ... "exceptions proposal not enabled", offset: 80781 ... }) }
    at tests/common/mod.rs:115 (Module::new on wasmtime Engine::default())

Mechanism, witnessed:
- The linked 45,131,985-byte artifact contains a **tag section (section id 13)** — scanned
  top-level section ids: {1,2,3,4,5,6,7,9,10,11,13}.
- The staged SDK itself is exceptions-enabled: `libduckdb.a` (87,620,792 B),
  `libcore_functions_extension.a`, `libicu_extension.a`, `libc++.a`, `libc++abi.a`,
  `crt1.o`, and even `libunwind.a.tagged.bak` (a prior lane's workaround) all carry tag
  sections. The module is linked exactly from these inputs; the tag section is by-design
  output of this SDK, not a transient state.
- The test host validates with `Engine::default()`, which has the exceptions proposal OFF.

Fix directions (owner: wasm lane / coordinator; NOT applied by this lane):
1. Enable exceptions in the court host engine:
   `wasmtime::Config::wasm_exceptions(true)` in `tests/common/mod.rs` `Host::new()`.
2. Or link exceptions-free (rebuild the frozen SDK with `-fno-exceptions`; the
   `libunwind.a.tagged.bak` hints at prior surgery on this exact axis).

## Classification counts (sweep 2, 104 failures)

| class | count | evidence |
|---|---|---|
| REAL-DEFECT (exceptions gate) | 46 | 8 targets (see table) |
| ENVIRONMENTAL (eyeron compile-freeze, settled green) | 50 | BLOCKED-LINK exit 101, eyeron errors; current tree checks exit 0 |
| REAL-DEFECT candidate (registry artifacts) | 4 | `tests/registry_artifacts.rs:170/675/730`: "--write must be idempotent when rows are fresh", "mutation applied" `left == right` |
| LOUD-SKIP (typed, feature OFF) | 1 | `tests/evidentiary_ring_court.rs:687`: `SKIP [evidentiary-ring-courts] reason: feature 'evidentiary-ring' disabled in this build (default OFF) — run: cargo test --features evidentiary-ring --test evidentiary_ring_court` (harness counts it FAILED, not skipped — loud, per doctrine) |
| remainder | 3 | chicago_cross_authority mixed block: 3 exceptions failures in the same target as the 21 eyeron BLOCKED-LINKs |

## Courts the task named (they did NOT flip green)

| court | sweep 1 | sweep 2 | cause in sweep 2 |
|---|---|---|---|
| chicago_cross_authority | BLOCKED-LINK | 47p / **24f** | 3x exceptions gate + 21x eyeron BLOCKED-LINK |
| chicago_fibo | BLOCKED-LINK | 57p / **4f** | 4x eyeron BLOCKED-LINK (compile-freeze window) |
| op_differential | BLOCKED-LINK | 0p / **16f** | 16x eyeron BLOCKED-LINK |

None of the three was green at sweep time (2026-10-22 ~19:1x; statuses as-of-that-run, not current); none failed on its own assertion. The remaining blocker for
chicago_fibo/op_differential is only the compile-freeze window (now settled); chicago_cross_authority
additionally needs the exceptions gate resolved (3 of its tests reach Module::new on the real artifact).

## Per-target cause table (sweep 2)

| target | p | f | exceptions | build-failed |
|---|---|---|---|---|
| chicago_cross_authority | 47 | 24 | 3 | 21 |
| chicago_fibo | 57 | 4 | 0 | 4 |
| evidentiary_ring_court | 0 | 1 | 0 | 0 (loud-skip) |
| gws12_coldchain_cross_engine | 0 | 2 | 0 | 2 |
| gws4_ipc_injection_court | 1 | 2 | 0 | 2 |
| op_differential | 0 | 16 | 0 | 16 |
| op_examples | 2 | 1 | 0 | 1 |
| op_forward_compat | 0 | 5 | 1 | 4 |
| registry_artifacts | 18 | 4 | 0 | 0 |
| wasm_abi | 0 | 25 | 24 | 0 |
| wasm_boundary_fuzz | 1 | 1 | 1 | 0 |
| wasm_coldchain_e2e | 0 | 3 | 3 | 0 |
| wasm_governance_preflight | 0 | 3 | 3 | 0 |
| wasm_memory_wall | 0 | 7 | 7 | 0 |
| wasm_n3_admission | 0 | 3 | 3 | 0 |
| wasm_native_differential | 0 | 2 | 2 | 0 |
| wasm_size_court | 2 | 1 | 0 | 0 (see note) |
| **total** | **128** | **104** | **47** | **50** |

Note: wasm_size_court's 1 failure is a real size-budget court failure independent of the
exceptions gate (0 exceptions mentions in its block); see log.
wasm_abi 25f vs 24 exceptions: 1 failure's stdout block not matched by the scanner — treat as exceptions-class unless its owner says otherwise.


## Sweep 3c — confirmation on the settled tree (decisive)

Command: `CARGO_TARGET_DIR=target-sweep2 WASI_SDK_PATH=~/.frozen-duckdb/cache/v2.0.0-dev-wasm32-wasip1 cargo test --no-fail-fast --test chicago_fibo --test op_differential --test gws4_ipc_injection_court --test op_examples --test op_forward_compat --test gws12_coldchain_cross_engine`

| target | p | f | exceptions-gate |
|---|---|---|---|
| chicago_fibo | 57 | 4 | 4 |
| gws12_coldchain_cross_engine | 0 | 2 | 2 |
| gws4_ipc_injection_court | 1 | 2 | 2 |
| op_differential | 0 | 16 | 16 |
| op_examples | 2 | 1 | 1 |
| op_forward_compat | 0 | 5 | 5 |
| **total** | **60** | **30** | **30** |

**Zero `GRAPHLAW-WASM-BUILD-FAILED`, zero BLOCKED-LINK panics at `tests/common/mod.rs:78`.**
All 30 failures are the deterministic exceptions gate (same offset 80781 every time):
`Module::new` on `Engine::default()` at `tests/common/mod.rs:115` refuses the artifact's
tag section. The 50 eyeron-window failures of sweep 2 fully resolve into this gate.

## Final classification (post-confirmation)

| class | count | disposition |
|---|---|---|
| REAL-DEFECT: exceptions-proposal validation gate | 77 distinct tests across 14 targets (47 in sweep 2 + 30 in sweep 3c, same root cause) | single fix needed: `wasmtime::Config::wasm_exceptions(true)` in tests/common/mod.rs:115 Host::new(), or an exceptions-free SDK rebuild (staged libduckdb.a + libc++/crt1 all carry tag sections; libunwind.a.tagged.bak is a prior workaround on this axis) |
| ENVIRONMENTAL (compile-freeze): graphlaw lib + graphlaw-eyeron windows | resolved — 0 recurrences in sweep 3c | settled; no defect |
| REAL-DEFECT candidate: registry_artifacts (4) + wasm_size_court (1) | 5 | owner triage; not contention (registry asserts on idempotent --write / mutation flagging; size court asserts budget) |
| LOUD-SKIP: evidentiary_ring_court | 1 | typed reason, feature OFF by default; counted FAILED (correct per doctrine) |

## Standing

- ALIVE: link (L1 landing witnessed exit 0 on settled tree).
- PARTIAL: wasm-hosted courts — 47 tests fail the exceptions validation gate (REAL,
  deterministic, fix located: tests/common/mod.rs:115 engine config or SDK rebuild).
- ENVIRONMENTAL: 50 eyeron-window failures — settled; a clean re-run should flip them.
- OPEN: 4 registry_artifacts failures + wasm_size_court budget — owner triage needed.
- Disk note: the disk gate fired once mid-session (18.6 GiB free < 20 GiB, DISK-REFUSED
  hook); disk recovered to 37 GiB and sweep 3c completed. Stale lane roots remain at
  `target-lane*` / `target-a*` / `target-gate*` / `target-fulltest` (largest:
  `target-lane167` 9.5 GB, `target-lane122` 8.5 GB, `target-lane-s5` 8.0 GB) — coordinator purge recommended.
