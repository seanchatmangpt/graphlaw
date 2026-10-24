# runner-exit-codes

Typed exit codes for the two graphlaw stdio runners. Source of truth:
`src/bin/graphlaw_run.rs` (`RunError::code`, lines 134-149) and
`src/bin/graphlaw_run_eh.rs` (`RunError::code`, lines 91-106). Diagnostics are
one line on stderr; malformed input never panics a runner — every failure is
typed. Exit codes are stable; courts depend on them.

## graphlaw-run (wasmi host, `src/bin/graphlaw_run.rs`)

Legacy wasmi host for the pre-duckdb artifact class. Before handing bytes to
the engine it walks wasm section headers and refuses an EH-class artifact
(a Tag section, id 13) instead of panicking in the parser.

| Code | Name / condition | stderr shape | Recovery |
|---|---|---|---|
| 0 | Clean EOF at frame boundary | silent | Normal completion. |
| 2 | Usage / artifact I/O fault | `<message>` | Fix argv (flags above) or path. |
| 84 | Digest mismatch (`--expect-blake3`) | `ARTIFACT DIGEST MISMATCH` | Re-pin or rebuild. |
| 86 | Guest trap (`--crash`, or > 64 MiB) | `guest trap` | Fixture. Wall: shrink < 64 MiB. |
| 90 | EOF mid-frame | `EOF mid-frame` | Fix driver: prefix without body. |
| 92 | Frame length prefix exceeds MAX_REQUEST_BYTES | `frame length <len> exceeds cap <cap>` | Driver: chunk or shrink the frame. |
| 91 | Stdin read error | `stdin read error: <io>` | Fix the driver/pipe. |
| 93 | Malformed envelope (bad base64) | `malformed envelope` | Fix the driver's payload. |
| 94 | Non-JSON ABI response | `non-JSON response: <e>` | Rebuild; module ABI is broken. |
| 95 | Not a valid wasm module | `invalid wasm artifact: <e>` | Rebuild or repoint `--wasm`. |
| 96 | Instantiate/link/export fail | `wasm instantiation failed: <e>` | Rebuild. See imports. |
| 96 (EH) | Tag section | `EH-class artifact requires graphlaw-run-eh` | Use `-eh`; pre-parse. |
| 97 | Guest ABI / memory fault | `guest ABI fault: <e>` | Rebuild; inspect the failing op. |

Exact diagnostic strings (each prefixed `graphlaw-run: `):

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
EH-class artifact requires graphlaw-run-eh: <path> carries a
  wasm Tag (exception-handling) section, unsupported by this
  legacy wasmi host
guest ABI fault: <detail>
```

Special mode: `--hang` reads one frame then never replies and never exits
(the court's fail-closed timeout fixture); there is no exit code — the driver
must enforce the timeout.

## graphlaw-run-eh (wasmtime 48 host, `src/bin/graphlaw_run_eh.rs`)

Same packet-4 protocol and stub-compatible exit codes; adds the W3C
exception-handler proposal (`wasm_exceptions(true)`). Compiled under
`--features runner-eh` (`required-features` on the `[[bin]]`). No Tag-section
preflight — EH artifacts are its native input. The eh host hardens the
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

Exact diagnostic strings unique to this host (each prefixed
`graphlaw-run-eh: `; all other strings are the legacy shapes above with the
`graphlaw-run: ` prefix swapped to `graphlaw-run-eh: `):

```text
guest exhausted fuel budget 100000000000
```

Differences from the legacy host on 97: the anyhow chain is rendered with
`{:#}` (root cause, e.g. "uninitialized element 0", reaches stderr), and the
engine prints one `graphlaw-run-eh: guest frame #N in <op>: <fn name>` line
per backtrace frame (`WasmBacktraceDetails::Enable` at config). Exact strings
are the legacy shapes above with the `graphlaw-run: ` prefix swapped to
`graphlaw-run-eh: `.

`--hang` behaves identically: no exit code, driver-enforced timeout.

## Import policy (both runners, exit 96)

Only `wasi_snapshot_preview1` imports are accepted. A module importing
anything else fails instantiation with `unexpected host import <m>::<n>`.
Missing or mistyped `gl_alloc` / `gl_free` / `gl_call` / `memory` exports and
an `_initialize` trap are also 96.

## `--wasm` / `GRAPHLAW_WASM` resolution ladder

Identical order in both runners (graphlaw_run.rs:212-217,
graphlaw_run_eh.rs:178-185):

1. `--wasm <path>` — highest precedence; missing value is exit 2.
2. `GRAPHLAW_WASM` env var, when `--wasm` is absent.
3. Working-tree build default:
   - `graphlaw-run`: `target/wasm-abi/wasm32-wasip1/wasm/graphlaw_wasm.wasm`
   - `graphlaw-run-eh`: `target/wasm32-wasip1/wasm/graphlaw_wasm.wasm`

Note the differing default artifact paths: the legacy host pins the
`wasm-abi` profile build (the artifact `registry/ARTIFACTS.blake3` wasm row
pins); the EH host pins the plain `wasm32-wasip1` profile build. An unreadable
default artifact is exit 2, not a fallback.

## `--expect-blake3` digest gate (84)

Both runners compute BLAKE3 over the loaded bytes (lowercase hex) and compare
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
- `docs/how-to/run-the-wasm-module.md` — operator how-to for driving both
  runners.

## See Also

- `docs/how-to/run-the-wasm-module.md`
- `tests/graphlaw_run_eh_court.rs`
- `docs/refusals.md` (typed refusal register)
