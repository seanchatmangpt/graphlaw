# WASM ABI Op-15 Design: `duckdb` (cursor-streamed analytical eval)

Standing: DESIGN (GWS-3 audit, 2026-10-21) · **Landed since (2026-10-22): the op dispatches (`src/abi.rs` → `src/duckdb_op.rs`, exec/stream/close with cursors, `DuckDb`/`ResourceLimit` typed refusals, 16 MiB outstanding cap per GAP-1)**; still pending: registry-table/manifest digest bump and the design's qualification courts · Target: wasm crate, additive tail op · ABI_VERSION: unchanged (op-superset precedent) · Date: 2026-10-21

## Dispatch findings (audit basis)

- The hashed 14-op table omits `duckdb` so `registry_blake3` stays stable until an explicit digest bump. The module still dispatches the op (`src/abi.rs` → `duckdb_op::dispatch`). `gl:evalSql` (`src/rules/builtins.rs`) refuses `REFUSAL-04` only when the reader is not document-backed.
- `wasm/src` is a WASI shell over `graphlaw::abi::call`; frozen-duckdb is a **normal dependency of the `graphlaw` crate compiled into the same wasm module** (no feature gate) — the module dispatches duckdb itself. frozen-duckdb 2.0.2 emits `static=duckdb` for wasm32-wasip1. The link is witnessed (2026-10-22); the recorded pin `332ea218…` was not replaced. `WASI_SDK_PATH` must be a wasi-sdk root (`bin/clang`, `share/cmake/wasi-sdk-p1.cmake`). Without it, frozen-duckdb-sys `build.rs` fails typed (`WASI_SDK_PATH required`).
- 5 gaps (ranked): GAP-1 `MAX_OUTSTANDING_ALLOC_BYTES` (256 MiB) exceeds the smalt sandbox's 64 MiB wall — alloc-success-then-OOM-trap possible; GAP-2 DuckDB `memory_limit`/`threads` pragmas not ABI-enforced (caller-trusted today); GAP-3 `gl_call` non-interruptible (long scans need host timeout + cursor resumption); GAP-4 no `DuckDb`/`sandbox_aborted` error kind in the refusal crosswalk; GAP-5 manifest BLAKE3 re-baseline + wasm-size court risk from frozen-duckdb's footprint.

## Wire format (UTF-8 JSON over existing gl_alloc/gl_call/gl_free)

- `exec`: `{"op":"duckdb","mode":"exec","views":[{"name","glob","hive"}],"query_ast":"<IRI>","ast":{"text","dialect"}?,"subject_prefix":"https://…","vocab_prefix":"https://…","params":[…],"chunk_rows":≤10000}` → `{"ok":true,"receipt","query_hash","rows_emitted",N,"cursor":"opaque","more":bool}`
- `stream`: `{"op":"duckdb","mode":"stream","cursor","max_rows":≤10000}` → `{"ok":true,"nquads","rows","cursor"|null,"more":bool}`
- `close`: `{"op":"duckdb","mode":"close","cursor"}` → `{"ok":true}`

## Bounds (ABI-enforced, not caller-trusted)

- Request ≤ 16 MiB (existing); chunk response ≤ 16 MiB with ≤ 10,000 atoms/chunk.
- DuckDB compile-time constants: `memory_limit=32MiB` (half the smalt 64 MiB wall), `threads=1` (wasi single-thread), spill disabled/bounded.
- `MAX_OUTSTANDING_ALLOC_BYTES` re-derived ≤ 16 MiB for the single `graphlaw-wasm` module (closes GAP-1: alloc-success-then-OOM impossible).
- New error kind `DuckDb{message}` + `ResourceLimit{limit:"duckdb_chunk_bytes"|"duckdb_memory_limit"}` + unknown/expired cursor → typed refusal (closes GAP-4). Never trap.

## Courts required

In-guest exec/stream/close happy path; chunk-limit typed refusal; raw_call fuzz never-traps; native `builtin_eval_sql` ↔ wasm receipt/query_hash byte-identity; `abi_manifest_court` regeneration; wasm-size court re-baseline; smalt Gate-2 analog (65 MiB payload → typed refusal, instance alive).

## Semantics

- Cursor state lives in the wasm instance's DuckDB result — crash-only: worker kill invalidates cursors; host re-`exec`s (documented ephemeral-cursor contract).
- 50 MB+ Parquet stays outside linear memory (WASI fs streams via `read_parquet`); results cross as (a) receipt+count (aggregate pushdown, default) or (b) N-Triples pages.
- smalt bridge mapping: focus-node packing binds as `subject_prefix` (1:1 with `builtin_eval_sql` arg 2) — no new argument semantics.
