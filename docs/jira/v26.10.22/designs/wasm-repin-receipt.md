# Wasm re-pin attempt — receipt (L122, 2026-10-22)

Status: **BLOCKED (typed)** — no re-pin minted. The on-disk pins are unchanged
and verified. Every artifact the current toolchains can build carries a wasm
Tag section (id 13), which wasmi 2.0.0 rejects ("exceptions proposal not
enabled"), so a re-pin today ships an unhostable module.

## Gate receipts

| step | command | result |
| --- | --- | --- |
| Dependency gate (L109 script) | `bash /Users/sac/frozen-duckdb/scripts/build_wasi_duckdb.sh --verify-only` | `VERIFY-OK: exit 0`, 20 PASS lines (wrapper, sysroot merge, sha256 identity for libc/crt1/libc++/libc++abi/libunwind vs emsdk/rust sources, archive magic, duckdb pin `b6013f7aa2f36caabb9466ce1791667ea8a280bb`, C++-EH hello-world link + node WASI smoke) |
| Real rebuild (documented local toolchain) | `WASI_SDK_PATH=~/.frozen-duckdb/cache/v2.0.0-dev-wasm32-wasip1 cargo build --locked -p graphlaw-wasm --target wasm32-wasip1 --profile wasm` | exit 0, 2m48s, `graphlaw_wasm.wasm` 45,175,087 B, BLAKE3 `2cdfd0344d7e11cd81cc86030c93687dafbed467cd9000fe113452e073540eca`, **TAG(13) count: 1** | (historical)
| wasmi host court on that artifact | `GRAPHLAW_WASM=<that artifact> cargo test -p graphlaw --test wasm_native_differential` | FAILED 0/2 — `exceptions proposal not enabled` at offset 80783 |
| SDK-34 duckdb rebuild | cmake SDK-34 toolchain, pin `b6013f7a…`, `duckdb_static core_functions_extension icu_extension` | exit 0 after shim (see below); archives 87,907,296 / 16,306,472 / 3,302,450 B |
| SDK-34 full link | scratch `HOME`, `WASI_SDK_PATH=~/.local/wasi-sdk/wasi-sdk-34.0-arm64-macos` | exit 0, 1m06s, 45,805,385 B, BLAKE3 `b49d0582fc34d3de3dde62963b2371e1f7043b3ca89fd75f9cc77133111498fc`, **TAG(13) count: 1** | (historical)
| wasmi host court on SDK-34 artifact | `GRAPHLAW_WASM=<SDK-34 artifact> cargo test -p graphlaw --test wasm_native_differential` | FAILED 0/2 — `exceptions proposal not enabled` at offset 77181 |

## Falsified premises (witnessed, not inferred)

1. **"wasmi-rejected Tag comes only from the local cache's libunwind"** —
   falsified. A fully SDK-34-built link (DuckDB archives compiled by
   wasi-sdk-34 clang 23.1 `-fwasm-exceptions`, linked against its own
   `share/wasi-sysroot/lib/wasm32-32... eh` libc++/libc++abi/libunwind) also
   carries exactly one Tag section. Wasm C++ exception handling **is** tags:
   every route that links DuckDB (which throws) carries one.
2. **"the pinned wasi-sdk-34 does not emit tags"** — true per-object
   (SDK-34 clang `-fwasm-exceptions` objects probe 0 tag sections), false at
   final link: the eh libunwind's `Unwind-wasm.c.o` (1 tag member) is pulled
   by the EH link.
3. **"SDK-34 route is drop-in"** — falsified. Neither the CI recipe
   (`.github/workflows/release.yml:95-113`) nor
   `frozen-duckdb-builder 2.0.2 compile_wasip1_static` compiles DuckDB against
   SDK-34's wasi-libc without source-level shims. Required shims (build-tree
   only, /tmp/wasi_compat34.h force-included): `_WASI_EMULATED_MMAN`,
   `_WASI_EMULATED_SIGNAL`, `_WASI_EMULATED_PROCESS_CLOCKS`,
   `_WASI_EMULATED_GETPID` defines; declarations `kill`, `sched_getcpu`,
   `struct rlimit`/`getrlimit`/`setrlimit`, `struct winsize`/`TIOCGWINSZ`
   (extern "C" required — without it the references are C++-mangled). The link
   additionally needs no-op `kill`/`sched_getcpu`/`getrlimit` stubs (Rust
   self-contained libc lacks them at link time in this composition).
4. **wasmi 2.0.0 has no exceptions toggle** — confirmed against
   `wasmi-2.0.0/src/engine/config.rs` (no exceptions/tag config surface; the
   rejection lives in wasmparser 0.228 `validator.rs:840`).

## Operator decision now on the table

- **(a) wasmtime host**: keep the linking-green, verified-toolchain artifact
  (clang-24 route, L109 script VERIFY-OK), move the host courts to wasmtime
  (supports exceptions). Re-pin proceeds after the differential court runs
  under wasmtime.
- **(b) no-EH DuckDB**: a DuckDB built with exceptions disabled
  (noeh libc++) cannot throw/catch — DuckDB's own error path throws, so this
  is a functional change to error handling, not a toolchain swap.
- **(c) wasmi exceptions support**: blocked on upstream wasmi, not locally
  fixable.

Untouched: `priv/graphlaw.wasm` (332ea2188fafb465666e71407e8f940c5b60a15184e6ad6567bb4f723fe2c30c,
6,793,617 B — re-read 2026-10-22, BLAKE3 re-verified, **TAG(13): 0**),
`priv/graphlaw.wasm.blake3`, `registry/ARTIFACTS.blake3` wasm row, `js/npm`
wasm copy — all match the 26.10.21 pin.

## Shared-compile notes (disclosed)

- `tests/common/mod.rs` (another lane's in-flight file) carried 3 E0308s
  breaking every differential-court compile; minimal SLA unblock fix applied
  (Option<String> chain: `.map(|l| l.to_string())` +
  `.unwrap_or_else(|| "...".to_string())`). Disclosed per compile-freeze law;
  the owner should re-land their intent over it.
- The `wasm_size_court` / `wasm_boundary_fuzz` gates are
  **BLOCKED-SHARED-COMPILE** as of the second half of this session:
  `src/sql_ast.rs` / `src/sql_ast/refusals.rs` / `src/registry/*.rs` (another
  lane's in-flight refactor) fail with 14 E0425/E0433/E0624 errors, so no test
  compiling the `graphlaw` lib can run. `cargo test -p graphlaw-wasm` passed
  9/0 (that crate does not include the broken modules).
- Disk: the fleet gate refused runs twice at <20 GiB free; freed ~1.5 GiB from
  my own `target-lane122` (cargo clean of stale graphlaw dev artifacts);
  df receipts: 18.5→27→19.6→23 GiB across the session.

## Candidate artifacts (NOT pinned, in target-lane122)

- clang-24 route: 45,175,087 B, `2cdfd0344d7e11cd81cc86030c93687dafbed467cd9000fe113452e073540eca`, TAG: 1 — wasmi-REJECTED (witnessed). (historical)
- SDK-34 route: 45,805,385 B, `b49d0582fc34d3de3dde62963b2371e1f7043b3ca89fd75f9cc77133111498fc`, TAG: 1 — wasmi-REJECTED (witnessed). (historical)
