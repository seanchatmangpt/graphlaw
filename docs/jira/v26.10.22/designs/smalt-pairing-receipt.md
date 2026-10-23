# Smalt-Graphlaw Pairing Receipt (Lane L121)

Date: 2026-10-22 · Lane: L121 · Repos: `/Users/sac/graphlaw`, `/Users/sac/smalt` (canonical checkouts)
Status: **PAIRING BLOCKED — confirmed cross-repo defect (wasmi cannot load the frozen-duckdb wasm artifact generation)**

## Question

Does smalt actually drive graphlaw? Attempted end to end, real frames, real
processes, both repos.

## What smalt expects (protocol surface, `lib/smalt/wasm/worker.ex`)

- Packet-4 binary frames over stdio (`Port.open(..., packet: 4, :binary)`).
- Stub-mode frame: `{"op":"eval","payload_b64":...,"predicate_frames":[...]}`;
  reply `{"ok":true,"result":<abi>,"predicate_frames_count":N}`.
- Raw-ABI passthrough frame (any other JSON object, e.g. `{"op":"capabilities"}`);
  reply wrapped in the same `{"ok":true,"result":...}` envelope.
- Config-only swap: `config :smalt, :wasm_runner_path => [graphlaw-run, "--wasm", <artifact>, "--expect-blake3", <digest>]`
  (worker.ex GWS-18 list branch); abi mode (`wasmtime run --max-memory 67108864`)
  exists but is for a stdio-speaking module, not this reactor artifact.

`graphlaw-run` (`src/bin/graphlaw_run.rs`) speaks exactly this protocol — the
framing and envelope shapes match worker.ex line for line. No protocol drift in
the frame grammar itself.

## Builds (all lane-isolated, `CARGO_TARGET_DIR=target-lane121`)

| Command | Result |
|---|---|
| `WASI_SDK_PATH=~/.frozen-duckdb/cache/v2.0.0-dev-wasm32-wasip1 cargo build -p graphlaw-wasm --target wasm32-wasip1 --profile wasm` | exit 0, artifact `/Users/sac/graphlaw/target-lane121/wasm32-wasip1/wasm/graphlaw_wasm.wasm`, 45,161,875 B, BLAKE3 `940af4c7…ed6575` |
| `cargo build --bin graphlaw-run --features runner` | exit 0 |
| `cargo test -p graphlaw-wasm` (host contract) | exit 0, 9 passed / 0 failed |

## Attempt 1 — manual packet-4 loopback (Elixir port driver, mirrors worker.ex)

Driver: `/tmp/lane121_pairing.exs` (`:io.setopts(:standard_io, encoding: :latin1)`
before first frame per the WASM-1 law; `Port.open(packet: 4, :binary)`).

Frames sent (verbatim):
```
SENT  21B {"op":"capabilities"}
SENT  152B {"op":"eval","payload_b64":"eyJkaWFsZWN0IjoibnRyaXBsZXMiLCJ0ZXh0IjoiPGh0dHBzOi8vZS9zPiA8aHR0cHM6Ly9lL3A+IDxodHRwczovL2Uvbz4gLiJ9","predicate_frames":[]}
SENT  108B {"op":"eval","payload_b64":"eyJkaWFsZWN0IjoibnRyaXBsZXMiLCJ0ZXh0IjoiPGE+IDxiPiAuIn0=","predicate_frames":[]}
SENT  39B {"op":"sql","sql":"SELECT 1"}
```

Result against the lane artifact: `EXIT status=95`, runner stderr:
```
graphlaw-run: invalid wasm artifact: exceptions proposal not enabled (at offset 0x13b8b)
```
wasmi 2.0.0 has no exceptions-proposal support (no `wasm_exceptions` toggle in
`wasmi-2.0.0/src/engine/config.rs` — confirmed by reading the vendored source
listing: mutable_global through relaxed_simd, no exceptions). The EH (tag
section / try/catch) bytes come from the precompiled frozen-duckdb + libunwind
blobs in `~/.frozen-duckdb/cache/v2.0.0-dev-wasm32-wasip1`, not from rustc:
rebuilding with `RUSTFLAGS="-C target-feature=-exception-handling"` (exit 0,
BLAKE3 `6c0af07f…8dc54`) still fails identically (exit 95, same diagnostic).

## Attempt 2 — smalt's own court, the real pairing surface

```
GRAPHLAW_RUN=…/target-lane121/debug/graphlaw-run \
GRAPHLAW_WASM=…/target-lane121/wasm32-wasip1/wasm/graphlaw_wasm.wasm \
GRAPHLAW_ROOT=/Users/sac/graphlaw \
MIX_BUILD_ROOT=_build-lane121 mix test test/integration/wasm_real_runner_court_test.exs
```
Result: **3/7 passed, 4 failed.**
- PASS: gate 2 (memory wall → sandbox_aborted, BEAM survives), gate 2b (wrong
  digest → exit 84), gate 5 (hung runner → gate_timeout fail-closed).
- FAIL (all `{:sandbox_aborted, {:exit_status, 95}}`, same EH cause): gate 1
  (real parse), gate 1b (typed refusal), gate 1c (raw capabilities), gate 3
  (crash-pool healing expected exit 86, got 95).

## Attempt 3 — positive control: pre-duckdb artifact generation

Serving `/Users/sac/beam4pm/native/graphlaw/graphlaw_wasm.wasm` (6,655,651 B,
BLAKE3 `8b4f4bc8…`, the pre-OP15 generation) through the identical loopback:

```
RECV 1333B {"ok":true,"result":{"abi":1,"abi_version":1,"authorities":[{"authority":"purrdf","capability":"RDF 1.2 / codecs / storage IR","revision":"2.0.2"},…
RECV  158B {"ok":true,"predicate_frames_count":0,"result":{"error":{"…kind":"Unsupported","message":"missing string field `op`"},"ok":false}}   ← eval payload without "op"
RECV  131B {"ok":true,"result":{"error":{"…kind":"Unsupported","message":"missing string field `op`"},"ok":false}}   ← {"op":"sql"} — sql op does not exist in that generation
```
The pre-duckdb generation loads in wasmi, serves `capabilities` (abi=1, full
authority list), and wraps raw ABI responses per the stub-mode envelope. The
packet-4 pairing mechanism itself is ALIVE and was working; it broke when the
frozen-duckdb link landed in the wasm unit (OP15 / SQLRDF-08 generation).

## Findings

1. **CONFIRMED CROSS-REPO DEFECT (blocker).** The current artifact generation
   (frozen-duckdb linked, 45 MB) carries wasm exception-handling sections
   inherited from the frozen-duckdb/libunwind precompiled blobs. wasmi 2.0.0 —
   the host engine of BOTH `graphlaw-run` AND graphlaw's own
   `tests/wasm_abi.rs` — has no exceptions proposal, so no wasmi-based host can
   load the artifact. smalt's real-runner pairing is therefore dead against
   the duckdb-bearing unit: court 3/7, gate 1 exit 95. Fix is host-side:
   switch graphlaw-run / wasm_abi host to an EH-capable runtime (wasmtime,
   WAMR, V8) or rebuild the frozen-duckdb wasm blobs without `-fexceptions`.
2. **Registry pin drift (pre-existing, already flagged in THESIS.md §46).**
   Pin `332ea218…` (6,793,617 B) matches neither my rebuild (`940af4c7…`,
   45,161,875 B) nor any on-disk artifact; the 6.79 MB pin is from the
   pre-duckdb generation (beam4pm's copy, 6,655,651 B `8b4f4bc8…`, is that
   generation). Coordinator-owned re-pin (W09).
3. **No protocol drift.** graphlaw-run's frame grammar, eval envelope, digest
   refusal (exit 84), crash (86) / hang (timeout) fault modes, and envelope
   wrap all match `Smalt.Wasm.Worker` expectations; witness: the pre-duckdb
   loopback replies above and court gates 2/2b/5 passing against the real
   runner. The pairing protocol is not the problem — the module load is.
4. smalt's abi mode (`wasmtime run --max-memory 67108864 <binary>`) cannot
   drive this artifact either: the module is a gl_alloc/gl_call reactor, not a
   WASI stdio command, so `wasmtime run` exits without serving frames; the
   only bridge is graphlaw-run, which is wasmi-bound (finding 1).

## Receipt

- Builds: exits 0/0/0 as tabled above; artifact digest `940af4c7…ed6575`
  (45,161,875 B); EH-disabled rebuild `6c0af07f…8dc54`.
- Loopback runs: exit 95 (lane artifact), positive replies against beam4pm
  pre-duckdb artifact (quoted above), exit 2 (my driver bug, corrected path).
- smalt court: 3/7, failed gates 1/1b/1c/3 with `{:sandbox_aborted, {:exit_status, 95}}`.
- Falsifier for the defect claim: serve the lane artifact to graphlaw-run; it
  exits 95 with "exceptions proposal not enabled". Reproduced 3x (2x manual
  loopback, 1x court).
- Out of lane scope (no code changes made): host runtime swap, blob rebuild,
  registry re-pin.