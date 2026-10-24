# runner-exit-codes

Typed exit codes for the graphlaw stdio runner. Source of truth:
`src/bin/graphlaw_run_eh.rs` (`RunError::code`). Diagnostics are one line on
stderr; malformed input never panics the runner — every failure is typed.
Exit codes are stable; courts depend on them.

The legacy `graphlaw-run` wasmi host was retired 2026-10-24
(`docs/jira/v26.10.23/ADR-wasmi-retirement.md`); `graphlaw-run-eh` is the sole
packet-4 stdio runner and every exit code below is stub-compatible with the
retired host's contract.

## graphlaw-run-eh (wasmtime 48 host, `src/bin/graphlaw_run_eh.rs`)

The packet-4 stdio host for the pinned artifact: wasmtime 48 with the W3C
exception-handler proposal (`wasm_exceptions(true)`). Compiled under
`--features runner-eh` (`required-features` on the `[[bin]]`). EH artifacts
are its native input (no Tag-section preflight needed). The host hardens the
boundary: the WASI environment is sterile (explicitly empty — nothing from
the host env reaches the guest), the frame prefix is bounded by
`graphlaw::abi::MAX_REQUEST_BYTES` and refused pre-allocation (92), and
every guest execution runs under a fuel budget of `100_000_000_000` units (reset
per request) with epoch interruption armed as the wall-clock strangle path;
a spent budget is a typed refusal (98), never a raw trap or hang, and a fired
epoch deadline surfaces as an interrupt trap (97) until a host watchdog
increments the epoch.

| Code | Name / condition | stderr shape | Recovery |
|---|---|---|---|
| 0 | Clean EOF at frame boundary | silent | Normal completion. |
| 2 | Usage / artifact I/O fault | `<message>` | Fix argv or the artifact path. |
| 84 | Digest mismatch (`--expect-blake3`) | `ARTIFACT DIGEST MISMATCH` | Re-pin or rebuild. |
| 86 | Guest trap (`--crash`, or > 64 MiB) | `guest trap` | Court fixture; shrink payload. |
| 90 | EOF mid-frame | `EOF mid-frame` | Fix the driver. |
| 92 | Frame length prefix exceeds MAX_REQUEST_BYTES | `frame length <len> exceeds cap <cap>` | Driver: chunk or shrink the frame. |
| 91 | Stdin read error | `stdin read error: <io>` | Fix driver/pipe. |
| 93 | Malformed envelope (bad base64) | `malformed envelope` | Fix the payload. |
| 94 | Non-JSON ABI response | `non-JSON response: <e>` | Rebuild the artifact. |
| 95 | Invalid wasm (or engine config) | `invalid wasm artifact: <e>` | Rebuild or repoint. |
| 96 | Instantiate/link/export failure | `wasm instantiation failed: <e>` | Rebuild vs ABI. |
| 97 | Guest ABI / memory fault | `guest ABI fault` + frames | Read backtrace; rebuild. |
| 98 | Guest fuel budget spent (`FUEL_PER_REQUEST` = 100_000_000_000) | `guest exhausted fuel budget 100000000000` | Driver: shrink the request or raise the budget. |

Exact diagnostic strings (each prefixed `graphlaw-run-eh: `):

```text
ARTIFACT DIGEST MISMATCH for <path>: expected <hex>, got <hex>
guest trap (crash mode or memory wall)
EOF mid-frame
frame length <len> exceeds cap <cap>
stdin read error: <io error>
malformed envelope (bad base64 payload)
module returned non-JSON response: <serde error>
invalid wasm artifact: <engine error>
wasm instantiation failed: <error>
guest ABI fault: <detail>
guest exhausted fuel budget 100000000000
```

Diagnostics on 97: the anyhow chain is rendered with `{:#}` (root cause, e.g.
"uninitialized element 0", reaches stderr), and the engine prints one
`graphlaw-run-eh: guest frame #N in <op>: <fn name>` line per backtrace frame
(`WasmBacktraceDetails::Enable` at config).

`--hang` behaves identically: reads one frame then never replies and never
exits (the court's fail-closed timeout fixture); there is no exit code — the
driver must enforce the timeout.

## Import policy (exit 96)

Only `wasi_snapshot_preview1` imports are accepted. A module importing
anything else fails instantiation with `unexpected host import <m>::<n>`.
Missing or mistyped `gl_alloc` / `gl_free` / `gl_call` / `memory` exports and
an `_initialize` trap are also 96.

## `--wasm` / `GRAPHLAW_WASM` resolution ladder

Resolution order in `graphlaw_run_eh.rs` (178-185):

1. `--wasm <path>` — highest precedence; missing value is exit 2.
2. `GRAPHLAW_WASM` env var, when `--wasm` is absent.
3. Working-tree build default:
   `target/wasm32-wasip1/wasm/graphlaw_wasm.wasm`

An unreadable default artifact is exit 2, not a fallback.

## `--expect-blake3` digest gate (84)

The runner computes BLAKE3 over the loaded bytes (lowercase hex) and compares
against the `--expect-blake3` value. Comparison happens after the file read
but before engine parse and before any packet-4 frame is processed, so a
mismatched artifact is never instantiated or served. Mismatch prints path,
expected, and got digests and exits 84. Missing flag value is exit 2. To
obtain a digest: `b3sum target/.../graphlaw_wasm.wasm` or the pinned row in
`registry/ARTIFACTS.blake3`.

## Courts

- `tests/graphlaw_run_eh_court.rs` — black-box court over the real binary:
  witnessed 7/0 (7 passed, 0 failed) covering 0, 84, 90, 95, 96 (non-WASI
  import and missing-ABI-exports legs), and 97 (pending artifact-pin
  capabilities frame, loud pending-skip until the pin lands).
- `tests/wasm_frame_bound_court.rs` — frame-bound court over the same binary
  (EH legs; the wasmi legs were retired with the host).
- `docs/how-to/run-the-wasm-module.md` — operator how-to for driving the
  runner.

## See Also

- `docs/how-to/run-the-wasm-module.md`
- `tests/graphlaw_run_eh_court.rs`
- `docs/refusals.md` (typed refusal register)
