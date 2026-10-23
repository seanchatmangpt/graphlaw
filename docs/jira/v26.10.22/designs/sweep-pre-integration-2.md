# Sweep pre-integration census 2 (lane L56)

> Note: this receipt's env (`WASI_SDK_PATH`) used the engine cache; the current recipe is a wasi-sdk root, documented in `docs/WASM.md`.

Subject: /Users/sac/graphlaw @ 615dbfa (main), 2026-10-22.
Command: `CARGO_TARGET_DIR=target-sweep cargo test --no-fail-fast 2>&1 | tee /tmp/sweep2.log`
Exit: cargo test failed (98 test-result lines; /tmp/sweep2.log sha256 prefix ebb98c96db6d59b5).
Disk receipt: df -h / = 99 GiB free before start, 68 GiB after.

## Census

- **635 passed, 102 failed, 1 ignored, 0 compile-aborts**, 98 test-result lines (97 targets + doc-tests 62/62 ok).
- Prior census: 561/73 + 1 compile-abort (test_parquet_and_builtin_eval). Delta: +74 passed, +29 failed, compile-abort fixed.

## Classification (102 failures)

- **WASM-LINK-BLOCKED: 95.** All trace to one root: frozen-duckdb-sys wasm32 build.rs:97/100 panic —
  `~/.frozen-duckdb/cache/v2.0.0-dev-wasm32-wasip1/libduckdb.a is not a wasm32 archive. A host libduckdb_static.a cannot be linked by wasm-ld. Remove it and rebuild with WASI_SDK_PATH set.`
  Surfaced via tests/common/mod.rs:65 (typed GRAPHLAW-WASM-BUILD-FAILED / BLOCKED-LINK) or per-target wrappers
  (wasm_abi.rs:36, wasm_memory_wall.rs:84, gws4_ipc_injection_court.rs:242 "wasm build failed").
  Targets: chicago_cross_authority(24), wasm_abi(25), op_differential(16), chicago_fibo(4),
  op_forward_compat(5), gws12(2), gws4(2), op_examples(1), wasm_coldchain_e2e(3),
  wasm_governance_preflight(3), wasm_memory_wall(7), wasm_n3_admission(3).
- **LOUD-SKIP (not a pass): 3.** wasm_boundary_fuzz.rs:349 and wasm_size_court.rs:33 ("SKIP is not a pass: no wasm binary/artifact — the court did not run") + evidentiary_ring_court.rs:687 (typed SKIP: feature 'evidentiary-ring' default OFF; 9 courts compiled out).
- **REAL-DEFECT: 4** (3 sites):
  1. tests/authority_manifest.rs:112 — unpublishable path dep `frozen-duckdb = { path = "/Users/sac/frozen-duckdb/..." }` without version (1 test).
  2. tests/capabilities_digest.rs:133,222 — registry digest mismatch: live blake3:7db561aa… vs committed 4cd56fd5… (2 tests).
  3. tests/corpus_conformance.rs:173 — asset manifest stale for ontologies/sql-ast.ttl + ontologies/shapes/ast_governance.ttl (1 test).
- **ENVIRONMENTAL: 0.** No retries needed; zero flake signatures (no timeouts, no contention).
