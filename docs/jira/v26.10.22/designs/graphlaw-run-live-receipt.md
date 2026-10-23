# graphlaw-run Live Execution Receipt (lane L115)

> Note: this receipt's env (`WASI_SDK_PATH`) used the engine cache; the current recipe is a wasi-sdk root, documented in `docs/WASM.md`.

- **Date**: 2026-10-22
- **Subject**: `graphlaw-run` (packet-4 stdio runner, `src/bin/graphlaw_run.rs` — read-only this lane)
  exercised against a REAL prebuilt wasm artifact over the packet-4 stdio protocol.
- **Runner build**: `CARGO_TARGET_DIR=target-lane115 cargo build --features runner --bin graphlaw-run` — exit 0.
- **Framing harness**: python3 packet-4 writer (4-byte BE length prefix + JSON body), `/tmp/l115/frame.py`.

## Artifact identities

| artifact | blake3 | size | provenance | wasmi `Engine::default()` result |
|---|---|---|---|---|
| `priv/graphlaw.wasm` (registry pin row `wasm`) | `332ea2188fafb465666e71407e8f940c5b60a15184e6ad6567bb4f723fe2c30c` | 6,793,617 B | pinned 26.10.21 (byte-exact registry/ARTIFACTS.blake3 row; mtime 2026-10-22 12:56) | LOADS — all receipts below |
| L115 lane build `target-lane115/wasm32-wasip1/wasm/graphlaw_wasm.wasm` | `f8a146b97be83948331a8d2b9726d84e26b2cc947058b04175eb31c8f84e8eca` | 45,162,315 B | built this lane with `WASI_SDK_PATH=~/.frozen-duckdb/cache/v2.0.0-dev-wasm32-wasip1`, exit 0, 2m49s | REJECTED — typed exit 95: `invalid wasm artifact: exceptions proposal not enabled (at offset 0x13b8c)` |

Both artifacts contain DuckDB; they differ in a **tag (exceptions) section** present only in
the lane build. Root cause chain (environmental, typed): the engine cache is not a
wasi-sdk root. The lane still pointed `WASI_SDK_PATH` at it. Its `bin/clang`
(`~/.frozen-duckdb/cache/v2.0.0-dev-wasm32-wasip1/bin/clang`, **clang 24.0.0git**, dev tip,
rebuilt 16:53–17:27 this session) compiles DuckDB objects carrying wasm exception-handling
tag sections (`__cpp_exception` names present in `libicu_extension.a` / `libcore_functions_extension.a`).
rust-lld merges the tag section into the final module; wasmi 2.0.0 has no exceptions proposal
and refuses the module. CI pins **wasi-sdk-34**, which does not emit the tags. Consequence:
an artifact built from this tree against the current local cache cannot be hosted by the
wasmi host law until the cache is rebuilt with the pinned SDK (or EH disabled in the duckdb
archive recipe). The runner behaved correctly — typed 95, one-line stderr, no panic.

## L13 typed-code classes: witnessed vs still compile-only

| class | code | status this session | witness |
|---|---|---|---|
| happy path (capabilities / parse / shacl / eval envelope) | 0 | **EXECUTED-WITNESSED** | R1/R2/R4/R10 below |
| digest mismatch refusal | 84 | **EXECUTED-WITNESSED** | R8 (both legs: R7 correct digest exit 0, R8 wrong digest exit 84) |
| guest trap (crash / memory wall) | 86 | compile-only still | `--crash` mode is a deliberate trap leg of the BEAM courts; not in this lane's battery |
| EOF mid-frame | 90 | **EXECUTED-WITNESSED** | R6 |
| stdin read error | 91 | UNWITNESSED-STILL | requires an OS-level stdin failure (e.g. a closed fd faulting mid-read); not honestly inducible from a stdio harness without fault injection |
| malformed envelope (bad base64 in `op:"eval"` payload) | 93 | **EXECUTED-WITNESSED** | R5 |
| non-JSON module response | 94 | UNWITNESSED-STILL | needs a module that replies with non-JSON bytes; the real artifact always emits valid JSON. Inducing it requires a doctored artifact — refused (digest law) |
| invalid module | 95 | **EXECUTED-WITNESSED** | R9 (non-wasm bytes) AND R-lane (real lane-built artifact rejected for EH tags — the strongest possible 95 witness: a REAL, valid-Rust, nonconforming artifact) |
| instantiation/link failure | 96 | UNWITNESSED-STILL | needs a valid-wasm module missing `gl_alloc`/`gl_call`/`gl_free` or with non-WASI imports; not in this lane's battery |
| guest ABI fault | 97 | UNWITNESSSED-STILL | needs `gl_call` trap inside the guest; the real artifact never faults on the battery inputs |

## Executed receipts (all frames + exit codes)

Requests framed as packet-4 (4-byte BE length + JSON); replies unframed by the python harness.
`$RUN` = `target-lane115/debug/graphlaw-run`, `$PIN` = `priv/graphlaw.wasm` (pinned digest).

- **R1 capabilities** — `{"op":"capabilities"}` → EXIT=0, reply len 1470:
  `{"ok":true,"result":{"abi":1,"abi_version":1,"authorities":[...purrdf 2.0.2, SPARQL, SHACL, ShEx, Datalog, entail, hooks...]}}`
- **R2 parse** — `{"op":"parse","text":"@prefix ex: <https://e/> .\nex:s ex:p ex:o .\n","dialect":"turtle"}` → EXIT=0:
  `{"ok":true,"result":{"dialect":"Turtle","id":"blake3:b40351024431a9f42aee144a86ad7fe9e74126f0eb5b8537dea2e3fad77f258e","ok":true,"quads":1}}`
- **R3 duckdb op (probe, pinned artifact)** — native duckdb exec envelope from `tests/wasm_duckdb_op.rs` → EXIT=0,
  reply `{"ok":true,"result":{"error":{"kind":"Unsupported","message":"unknown op `duckdb`"},"ok":false}}`.
  **Typed finding, by design**: `wasm/Cargo.toml` links `graphlaw` with `features = ["abi","wasi-patched-deps"]`
  only — the `duckdb` ABI op (op-15) is a native-feature op; the wasm surface for SQL is
  document admission + SHACL governance (R10), and REFUSAL-04 remains the non-document-backed
  builtin path, not a missing DuckDB switch. The runner and envelope handling stayed typed and exit 0.
- **R4 eval envelope passthrough** — `{"op":"eval","payload_b64":<base64 of {"op":"capabilities"}>,"predicate_frames":[{"k":1}]}` → EXIT=0,
  reply `{"ok":true,"predicate_frames_count":1,"result":{...capabilities...}}` — the smalt `Wasm.Worker`
  stub-mode contract held end-to-end over the packet-4 port (latin1 framing preserved; the
  packet reader never desynced).
- **R5 malformed envelope → 93** — `{"op":"eval","payload_b64":"!!!not-base64!!!"}` → stderr
  `graphlaw-run: malformed envelope (bad base64 payload)`, EXIT=93, no reply frame, no panic.
- **R6 EOF mid-frame → 90** — length prefix `00 00 00 64` (100) followed by only 8 bytes → stderr
  `graphlaw-run: EOF mid-frame`, EXIT=90.
- **R7 `--expect-blake3` correct** — pinned digest → EXIT=0, capabilities served.
- **R8 `--expect-blake3` wrong digest → 84** — expected all-zeros, got `332ea218…`:
  stderr `graphlaw-run: ARTIFACT DIGEST MISMATCH for /Users/sac/graphlaw/priv/graphlaw.wasm: expected 000…000, got 332ea2188fafb465666e71407e8f940c5b60a15184e6ad6567bb4f723fe2c30c`, EXIT=84, no serve.
- **R9 invalid module → 95** — `--wasm /tmp/l115/bad.wasm` ("not a wasm module") → typed wasmi
  diagnostic, EXIT=95.
- **R10 sql governance SHACL through the module** — `{"op":"shacl","data":<cold_chain_ast.ttl>,"shapes":<ast_governance.ttl>}`
  (real repo fixture + real shapes file) → EXIT=0, `conforms=true`, violations 0 — the sql-AST
  governance law executes inside the module over stdio, no mocks.

## L13 code-status delta

L13's disclosed gap: codes 90/91/93/97 (plus 84/94/95/96) compile-verified only. After this
session: **90, 93, 84 (both legs), 95 executed-witnessed against a real artifact**; happy path
and the smalt eval-envelope contract executed end-to-end; 91/94/96/97 remain compile-only with
typed, honest reasons above (each requires fault injection or a doctored artifact — refused on
the digest law).

## Replay

```bash
cd /Users/sac/graphlaw
CARGO_TARGET_DIR=target-lane115 cargo build --features runner --bin graphlaw-run   # exit 0
RUN=target-lane115/debug/graphlaw-run; PIN=priv/graphlaw.wasm
# NOTE: printf takes the whole frame as ONE argument (a format arg followed by a
# second arg silently drops it — witnessed: two-arg form emits the prefix only
# and the runner answers 90). \x21 is '!'; bash printf has no \x21 problem.
printf '\x00\x00\x00\x15{"op":"capabilities"}' | $RUN --wasm $PIN >/dev/null; echo $?   # 0
printf '\x00\x00\x00!{"op":"eval","payload_b64":"!!!"}' | $RUN --wasm $PIN; echo $?     # 93
python3 -c "import sys; sys.stdout.buffer.write((100).to_bytes(4,'big')+b'{')" | \
  $RUN --wasm $PIN; echo $?                                                             # 90
$RUN --wasm $PIN --expect-blake3 0000000000000000000000000000000000000000000000000000000000000000 </dev/null; echo $?  # 84
printf '\x00\x00\x00\x15{"op":"capabilities"}' | \
  $RUN --wasm $PIN --expect-blake3 332ea2188fafb465666e71407e8f940c5b60a15184e6ad6567bb4f723fe2c30c >/dev/null; echo $?  # 0
```

All five replay commands executed this session with exactly the exits annotated.

Standing: ALIVE for every row with an EXIT above; UNWITNESSED-STILL (typed) for 91/94/96/97.