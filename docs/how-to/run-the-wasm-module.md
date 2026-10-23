# How to run the WASM module

Run GraphLaw's whole surface through one WASI module from any host (Elixir/Wasmex,
Python wasmtime, wasmi, browsers via WASI shims): JSON in, JSON out, no JavaScript
glue. Reference: [abi-reference](../abi-reference.md) ·
[wasm-integration](../wasm-integration.md) · [refusals](../refusals.md).

## Build the module

```sh
rustup target add wasm32-wasip1        # once per toolchain
WASI_SDK_PATH="/opt/wasi-sdk" \
cargo build --locked -p graphlaw-wasm --target wasm32-wasip1 --profile wasm
# -> target/wasm32-wasip1/wasm/graphlaw_wasm.wasm
```

`wasm32-unknown-unknown` is not a supported module target (it requires a JavaScript
host); the standard clock/RNG paths are supplied by the `vendor/` cfg patches, so
building for WASI outside this workspace is a compile error by design.

## Runners

Two host runners ship in this workspace; both speak the same packet-4 stdio
protocol (4-byte big-endian length-prefixed frames, smalt eval-envelope
auto-detect, `{"ok":true,"result":...}` reply envelope).

| runner | runtime | artifact class | build |
|---|---|---|---|
| `graphlaw-run` | wasmi | legacy (pre-DuckDB) artifact — the class pinned for `tests/wasm_abi.rs` courts | `cargo build --features runner --bin graphlaw-run` |
| `graphlaw-run-eh` | wasmtime 48 (`wasm_exceptions` enabled) | the DuckDB-linked artifact | `cargo build --features runner-eh --bin graphlaw-run-eh` |

`graphlaw-run-eh` is gated by the `runner-eh` feature (`[[bin]]`
`required-features`, `Cargo.toml:89` / `Cargo.toml:118`); wasmi is host-only and
cannot compile for wasm32, which is why the runners are feature-split. Only
`wasi_snapshot_preview1` imports are accepted by either runner; anything else is
a typed instantiation refusal.

Exit codes (stub-compatible across both runners, typed in `RunError`,
`src/bin/graphlaw_run.rs:35` and `src/bin/graphlaw_run_eh.rs:30`): 0 success,
2 usage/I/O, 84 artifact digest mismatch, 86 guest-panic / 64 MiB memory-wall
trap, 90 EOF mid-frame, 91 stdin read error, 93 malformed envelope, 94 non-JSON
module response, and the wasm-artifact classes 95 (invalid module), 96
(instantiation/link failure), 97 (guest ABI fault). Malformed input never panics
a runner; every failure is a typed one-line stderr diagnostic plus its stable
exit code.

Artifact identity: `--wasm <path>`, else `GRAPHLAW_WASM`, else the working-tree
build; `--expect-blake3` verifies loaded bytes before serving (mismatch is exit
84, never a silent serve). The DuckDB-linked artifact class is roughly 45 MB and
its registry re-pin is pending a witnessed digest — no digest is asserted here.

## Host call contract

One self-contained WASI module; imports are `wasi_snapshot_preview1` only
(clock, random, stdio). Call `_initialize` once if the host does not do it for you.

| Export | Contract |
|---|---|
| `gl_alloc(len)` | returns a pointer to `len` bytes, or `0` when `len > 16 MiB` (`MAX_REQUEST_BYTES`) |
| `gl_call(ptr, len)` | reads the request from guest memory, returns `(out_ptr << 32) \| out_len` (UTF-8 JSON) |
| `gl_alloced(ptr)`/`gl_free(ptr, len)` | free the request/response buffers when done |

## Worked requests

Requests are `{"op": ..., ...}` UTF-8 JSON; responses gain `"ok": true` or an
error envelope. Full op reference: [abi-reference](../abi-reference.md); typed
error envelopes: [refusals](../refusals.md).

```jsonc
// request:  {"op":"sniff","text":"<urn:a> <urn:b> <urn:c> ."}
{"ok":true,"dialect":"NTriples","engine":"PurRdf"}

// request: {"op":"entail","data":{"text":"<urn:a:x> <urn:p:subset> <urn:a:y> .","dialect":"ntriples"},"regime":"rdfs"}
{"ok":true,"nquads":"...","added":["<urn:a:x> <urn:p:sub> <urn:graph> ..."]}   // closure quads
```

## Limits (enforced before parse; never panics)

Over-limit input returns a `ResourceLimit` refusal with `details: {limit, observed, max}`;
`gl_alloc(len)` returns `0` past 16 MiB and `gl_call` on a null buffer returns a typed
error, never a trap.

| limit | const | value | rationale |
|---|---|---|---|
| request_bytes | `MAX_REQUEST_BYTES` | 16 MiB | largest one-call document |
| json_depth | `MAX_JSON_DEPTH` | 64 | string-aware bracket scan |
| plan_actions | `MAX_PLAN_ACTIONS` | 1,000 | each action replayed + re-canonicalized |
| atoms_per_field | `MAX_ATOMS_PER_FIELD` | 10,000 | N-Triples lines per pre/add/del/goal field |
| policy_entries | `MAX_POLICY_ENTRIES` | 100,000 | FOND policy entries |
| n3_iterations | `law::N3_MAX_ITERATIONS` | 4,000 | Eyeron max_iterations cap (its default is 1,000,000) |

All consts are `pub` in `graphlaw::abi` / `graphlaw::law`; caps are checked before
parsing or heavy work.

## Drive the hashed op table like a host

The hashed table is 14 ops. `duckdb` is dispatched by the same module and is omitted from that table until an explicit digest bump. Absence from `capabilities` is not `REFUSAL-04`.

```sh
cargo test --test wasm_abi
```

builds the module itself and drives `capabilities`, `sniff`, `parse`, `convert`,
`canonical`, `sparql`, `shacl`, `shex`, `n3`, `entail`, `datalog`, `hooks`, `law`,
`policy` through a real wasmi+WASI runtime (18 tests, verified 2026-10-10:
`18 passed; 0 failed`). This historical wasmi receipt belongs to the legacy
pre-DuckDB artifact court (`tests/wasm_abi.rs`) — the wasmi runner's domain, not
the `graphlaw-run-eh` / DuckDB-linked artifact class above.
