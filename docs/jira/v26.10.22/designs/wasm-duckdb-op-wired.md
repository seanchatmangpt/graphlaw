# wasm duckdb op wired — receipt (L125)

Date: 2026-10-22 · Lane: L125 (wasm boundary) · Repo: /Users/sac/graphlaw · Branch main @ 615dbfa (dirty tree, multi-lane)

## Task

L115 finding: the architecture law says the module dispatches `duckdb` itself, but the
module returned typed `Unsupported / unknown op 'duckdb'`.

## Verdict

**The wiring is present at source; nothing needed wiring.** `src/abi.rs:479`
(`"duckdb" => crate::duckdb_op::dispatch`) is unconditional — no `cfg`, no
feature gate — and `wasm/src/lib.rs`'s generated shell routes every request
through `graphlaw::abi::call` (wasm/src/lib.rs:6-8, 206). The L115
`unknown op \`duckdb\`` came from the **shipped artifact being stale**, not a
missing route: `priv/graphlaw.wasm` (BLAKE3 `332ea2188fafb465666e71407e8f940c5b60a15184e6ad6567bb4f723fe2c30c`,
6,793,617 B, built 26.10.21 per registry/ARTIFACTS.blake3 pin) predates the op.

`registry::NATIVE_ONLY_OPS = ["duckdb"]` untouched (module-owned, unadvertised
on the hashed surface) — per L11 semantics, confirmed
`src/registry.rs:896` unchanged.

## Rungs

### Rung A — the fresh module carries the dispatch (PASS, executed)

    WASI_SDK_PATH=~/.frozen-duckdb/cache/v2.0.0-dev-wasm32-wasip1 \
      cargo build -p graphlaw-wasm --target wasm32-wasip1 --profile wasm
    # exit 0 (rebuilt three times this session, incl. once after the M4
    # purge of target-lane125; final artifact 45,179,094 B,
    # sha256 217336a4817d7e3e…, byte-stable size across rebuilds)

Court: `built_module_carries_the_duckdb_dispatch_statically` asserts >40 MB
(engine linked) and the data section carries the compiled-in op strings
(`missing string field \`mode\``, `unknown mode`, `SELECT batch_id`).

### Rung B — before/after witness (PASS, both executed)

BEFORE (L115's witness, re-executed this session through the same driver):

    GRAPHLAW_WASM=priv/graphlaw.wasm cargo test --test wasm_duckdb_op_wired
    exec refused: {"error":{"kind":"Unsupported","message":"unknown op `duckdb`"},"ok":false}
    → test result: FAILED. 0 passed; 3 failed   (all three: unknown op `duckdb`)

AFTER (native `graphlaw::abi::call` — the exact dispatch function the module
compiles): exec returns `rows_emitted: 2` + receipt IRI
(`https://example.org/data/receipt_…`) + query_hash; stream pages real
N-Triples (`<https://example.org/data/row_1>`,
`<https://example.org/vocab/total_cost>`); close idempotent-ok; no-mode twin
is the typed `DuckDb` refusal naming `mode`
(`duckdb: missing string field \`mode\``) — never a trap.

### Rung C — module-internal execution: BLOCKED (typed), `#[ignore]`-marked

The freshly built module does not execute in any local runtime. Runtime matrix
(all executed this session, typed errors):

| Runtime | Result |
|---|---|
| wasmi 2.0.0 (repo dev-dep, tests/common Host) | `exceptions proposal not enabled` — wasmi 2.0.0 has no EH implementation (grep of its src: no exception support) |
| wasmtime 48.0.1 CLI | `-W legacy-exceptions` — `error: unknown --wasm option` (legacy EH removed); default: `legacy_exceptions feature required for try instruction` |
| Node 26 / V8 | `CompileError: module uses a mix of legacy and new exception handling instructions @+42161761` |
| Node 26 + `--wasm-allow-mixed-eh-for-testing` | compiles, then traps `RuntimeError: function signature mismatch` in a duckdb vtable `call_indirect (type 2)` (fn 68815 ← call_buf chain), reproducible on `{"op":"capabilities"}` and `--no-liftoff`/`--liftoff-only` |
| Binaryen 133 `--translate-to-exnref` | translates successfully (43,276,761 B exnref-only module) — V8 still traps the same `function signature mismatch` ⇒ independent of EH dialect |
| wabt 1.0.42 | `wasm-validate --enable-all` PASSES on the module (valid); its interpreter has no exnref/no mixed mode |
| rustc side-flips | stable 1.96 `-C target-feature=-exception-handling` (no effect: mix persists), nightly-2026-06-22 build (mix persists — the 275 `try_table` are the frozen-duckdb side; 151,036 legacy `try` the rest), LTO=off (mix persists) |

Conclusion: the EH-dialect mix is real but V8's `signature mismatch` on the
exnref-translated module means the duckdb link is not yet executable in ANY
local runtime — a cross-lane blocker above L125's file ownership (frozen-duckdb
archive side / runtime pin).

## Court: tests/wasm_duckdb_op_wired.rs

    ./target-lane125/debug/deps/wasm_duckdb_op_wired-* (via cargo rustc -p graphlaw --test wasm_duckdb_op_wired)
    test built_module_carries_the_duckdb_dispatch_statically ... ok
    test before_state_refusal_and_after_state_native_dispatch ... ok
    test shipped_pre_wiring_artifact_refuses_duckdb_op ... ok
    test wasm_module_executes_duckdb_op ... ignored (BLOCKED-MODULE-EXEC, typed reason)
    test result: ok. 3 passed; 0 failed; 1 ignored

`--ignored` run (loud, not a pass): `wasm_module_executes_duckdb_op ... FAILED`
— `driver closed mid-frame` (the V8 trap above). The before-witness test
self-invalidates loudly (`OBSOLETE-WITNESS`) if priv/graphlaw.wasm is re-pinned
past the pre-wiring build.

## graphlaw-wasm host contract

    cargo nextest run -p graphlaw-wasm
    Summary: 9 tests run: 9 passed, 0 skipped

## Files touched

- `tests/wasm_duckdb_op_wired.rs` (new — wasm-side court; tests/wasm_duckdb_op.rs
  left untouched, it is another lane's in-flight edit)
- `tests/fixtures/l125_wasm_driver.mjs` (new — Node/V8 ABI driver,
  line-delimited JSON; no binary framing, latin1 law not in play)
- `docs/jira/v26.10.22/designs/wasm-duckdb-op-wired.md` (this receipt)
- `wasm/src/lib.rs`: **no edit** — the routing lives in the linked crate's
  `graphlaw::abi::dispatch` (src/abi.rs:479), already compiled into the module.

## Handoffs

1. Re-pin priv/graphlaw.wasm from a fresh build (coordinator; the pin
   `332ea218…` is the entire L115 symptom).
2. Runtime seam (cross-lane): get one runtime executing the duckdb-linked
   module — either a frozen-duckdb rebuild emitting single-dialect EH, or a
   runtime with both EH dialects + working vtable dispatch; then un-ignore
   `wasm_module_executes_duckdb_op` (it already asserts rows + receipt +
   native byte-identity).
3. Shared-tree noise (not L125's): examples/breed_oracle_gen.rs and
   examples/plan_replay_oracle_gen.rs fail to compile under
   `cargo test -p graphlaw` (E0308/E0560/E0609); L116's gws4 E0308 has since
   compiled clean under `cargo rustc --test gws4_ipc_injection_court`.
