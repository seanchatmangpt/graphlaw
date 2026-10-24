# How to run the WASM module

Run GraphLaw's whole surface through one WASI module from any host (Elixir/Wasmex,
Python wasmtime, browsers via WASI shims): JSON in, JSON out, no JavaScript
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

One host runner ships in this workspace: `graphlaw-run-eh`, which speaks the
packet-4 stdio protocol (4-byte big-endian length-prefixed frames, smalt
eval-envelope auto-detect, `{"ok":true,"result":...}` reply envelope).

| runner | runtime | artifact class | build |
|---|---|---|---|
| `graphlaw-run-eh` | wasmtime 48 (`wasm_exceptions` enabled) | the DuckDB-linked artifact | `cargo build --features runner-eh --bin graphlaw-run-eh` |

`graphlaw-run-eh` is gated by the `runner-eh` feature (`[[bin]]`
`required-features`, root `Cargo.toml:75`/`:96` feature row, `:116-119` bin
block). The wasmi-based `graphlaw-run` binary and its `runner` feature were
retired 2026-10-24 (`docs/jira/v26.10.23/ADR-wasmi-retirement.md`): the EH
runner is a protocol superset with the same exit-code contract, so consumers
see no wire change. Only
`wasi_snapshot_preview1` imports are accepted; anything else is
a typed instantiation refusal.

Exit codes (typed in `RunError`, `src/bin/graphlaw_run_eh.rs`; full table
with diagnostics and recovery text:
[`docs/reference/runner-exit-codes.md`](../reference/runner-exit-codes.md)):
0 success,
2 usage/I/O, 84 artifact digest mismatch, 86 guest-panic / 64 MiB memory-wall
trap, 90 EOF mid-frame, 91 stdin read error, 92 frame length above
`MAX_REQUEST_BYTES`, 93 malformed envelope, 94 non-JSON
module response, 95 (invalid module), 96
(instantiation/link failure), 97 (guest ABI fault), 98 (guest fuel
exhaustion). Malformed input never panics
a runner; every failure is a typed one-line stderr diagnostic plus its stable
exit code.

Artifact identity: `--wasm <path>`, else `GRAPHLAW_WASM`, else the working-tree
build; `--expect-blake3` verifies loaded bytes before serving (mismatch is exit
84, never a silent serve). The DuckDB-linked artifact class is roughly 58 MB;
its current registry pin is `58a44095…` (58,124,612 B, shipped with tag
`v26.10.22`).

## New surfaces (v26.10.22)

The DuckDB-linked artifact served by `graphlaw-run-eh` carries the
session, sequence, and document surfaces that the retired legacy
(pre-DuckDB) artifact class
does not:

- **Persistent DuckDB sessions** — `{"op":"duckdb","mode":"open/exec/
  stream/close","session":"<name>"}` keeps one real connection across
  wire calls. [docs/how-to/persistent-duckdb-sessions.md](persistent-duckdb-sessions.md).
- **Document sequences** — `{"op":"duckdb","mode":"exec","document":
  true}` runs a `sql:StatementSequence` member-by-member with a typed
  receipt per member; `"atomic": true` (or the Turtle root
  `sql:atomic true`) selects BEGIN/COMMIT/ROLLBACK execution over the
  default best-effort pipeline. Unbounded UPDATE/DELETE refuses
  `REFUSAL-05` (`UnboundedMutationRefusal`) at compile, before any
  engine call.
  [docs/how-to/document-sequences.md](document-sequences.md).

End-to-end tutorial: [docs/how-to/first-in-wasm-sql-lifecycle.md](first-in-wasm-sql-lifecycle.md).

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
`policy` through a real wasi runtime (18 tests, verified 2026-10-10:
`18 passed; 0 failed`). That receipt predates the wasmi retirement
(2026-10-24, `docs/jira/v26.10.23/ADR-wasmi-retirement.md`) and belongs to the
legacy
pre-DuckDB artifact court (`tests/wasm_abi.rs`) — the retired wasmi runner's
historical domain, not
the `graphlaw-run-eh` / DuckDB-linked artifact class above.
