# Vision-2030 Step 1 — Zero-Copy Shared-Memory Protocol Design (hook → GraphLaw WASM runner)

Lane V2030-1, 2026-10-22. DESIGN ONLY — no implementation. The runtime half
is gated on OD-1 (engine-tier host decision, `dual-artifact-policy-design.md`).
One canonical checkout `/Users/sac/graphlaw`; coordinator owns git.

## 0. Honesty header

- **Sub-millisecond is the TARGET, not a witnessed fact.** Every number below
  is cited to its receipt. The only executed end-to-end numbers on disk are
  admission-class, not hook-to-verdict-class.
- The claimed **0.082 µs statement admission** CTQ is **UNVERIFIED** — it
  appears in no court assert and no receipt on disk
  (`docs/jira/v26.10.22/mega/tc-mega-fibo-01/COURT-SPEC.md:46`, note M1:
  "No receipt on disk carries this number; it appears in no court assert").
  It is carried in this design as a ceiling to be witnessed, never as a
  baseline.
- The current CLI path's per-call cost class is **inferred, not measured**:
  no receipt on disk times a full hook→CLI→process-spawn→wasm-instantiate→
  eval→teardown cycle. Its floor is one wasm instantiation of the 6.79 MB
  DuckDB-linked artifact (`graphlaw-run-live-receipt.md` artifact table,
  `332ea218…`, 6,793,617 B) — ms-class by construction (6.8 MB linear-memory
  module through wasmi JIT/instantiation), measured nowhere. The design
  names the instrument that must measure it (§3.4).

## 1. Current path (what must be beaten)

**Witnessed shape**: hook (python) → `graphlaw` CLI → process spawn per call.
The repo's own persistent-runner alternative is **witnessed working**:
`graphlaw-run` (packet-4 stdio runner, `src/bin/graphlaw_run.rs`), lane L115,
`designs/graphlaw-run-live-receipt.md` — runner build exit 0, python3
packet-4 framer (4-byte BE length prefix + JSON body), executed receipts
R1–R6+ (capabilities exit 0 reply len 1470; parse exit 0, 1 quad; malformed
envelope → typed exit 93; EOF mid-frame → typed exit 90; wrong digest → 84;
invalid artifact → 95). It is a persistent wasmi-hosted runner over stdin
frames; the WASM-1 latin1 law applies to any Elixir driver
(`:io.setopts(:standard_io, encoding: :latin1)` before the first frame —
`~/.claude/rules/tools.md`, `receipts/wasm1-otp29-sandbox-port.md` witness
class).

**Cost class, honestly**: the witnessed per-frame runner cost is dominated by
JSON base64 envelope decode + wasmi execution. What the L115 receipt does NOT
time: frame parse µs, wasmi call µs, base64/JSON overhead µs. Those are the
numbers §3.4 must produce before any sub-ms claim.

## 2. Ground truth: the FFI contract the protocol must respect

`wasm/src/ffi.rs` (header comment, normative):

- ABI version 1, request limit **16,777,216 bytes**; `gl_alloc(len) -> ptr`,
  `gl_call(ptr, len) -> packed = (out_ptr << 32) | out_len`,
  `gl_free(out_ptr, out_len)`.
- **Side-table discipline**: the guest never trusts host-supplied layout.
  Every buffer is registered in `LIVE` (ptr → real capacity, initialized
  length, declared length, byte budget); a `(ptr, len)` is rebuilt only on
  exact table hit; unknown pointer / mismatched length / double free yield
  typed error responses, never UB.
- Outstanding-byte cap (`MAX_OUTSTANDING_BYTES`) — a host that never frees
  cannot fill linear memory.

Any zero-copy design MUST preserve this discipline: the shared-memory window
is just another host-owned buffer whose bounds come from the guest's own side
table, never from host-asserted lengths.

## 3. Budget arithmetic (honest magnitudes, every number cited)

### 3.1 In-kernel cost (witnessed)

| item | witnessed | receipt |
|---|---|---|
| receipt fold (`fold_params_anchored`, one 96 B update) | 10M p99 **209 ns** (SPC run: p50 291 ns, p99 375 ns — honest FAIL vs 250 ns USL) | `designs/hot-path-baseline.md:9,35-38`; `designs/ctq-traceability.md:10`; `receipts/spc-2026-10-22.json` |
| typed refusal construction | p50 41 ns / p99 42 ns (10M) | `designs/ctq-traceability.md:8` |
| smoke fold reading | 0.19–0.22 µs at n=1M | `DISSERTATION.md:40`, `designs/hot-path-baseline.md:14` |
| full-rehash fallback path | 1673–1748 ns (~11.6x over its 150 ns row) | `designs/hot-path-baseline.md:7` |
| admission (compile_turtle: SHACL + RDFC-1.0) | 1.56–4.97 ms/compile, once per compile, out of hot path | `designs/hot-path-baseline.md:12` |

### 3.2 The one-time vs per-call split

- **Compile/admission of the LAW (SHACL shapes + graph compile):**
  1.56–4.97 ms — done ONCE per law at runner start; cached
  (`designs/C1-cache.md` exists; the query cache court is
  `receipts/query-cache-court-2026-10-22.md`). Not per-call.
- **Per call**: a 3-quad `act:ToolCall` proposal is parse + fold + admit.
  The per-call kernel work is therefore **sub-µs to low-µs class**
  (209 ns fold + 42 ns refusal class + parse of 3 quads — parse cost
  itself UNMEASURED per-quad; a bench must witness it).

### 3.3 IPC overhead classes (magnitude, not benchmark)

| transport | per-message cost class | source of the class |
|---|---|---|
| stdin pipe (packet-4, current runner) | ~1–10 µs/call (copy + 4-byte prefix + JSON decode) | class estimate; UNMEASURED on this repo — §3.4 must measure |
| unix domain socket | ~2–15 µs/call (syscall pair + copy) | class estimate; UNMEASURED here |
| shared memory ring + futex/eventfd | ~0.1–1 µs/call (cache-line writes, one wake) | class estimate; UNMEASURED here |

All three are honest **class estimates, not receipts**. The kernel-side
witnessed costs (209 ns fold) are small enough that **transport, not the
kernel, is the budget** for sub-ms end-to-end.

### 3.4 The measuring instrument (must exist before any sub-ms claim)

New court, sequenced increment (i) adjunct: `bench/hook_roundtrip.rs` (or a
graphlaw-run `--bench` leg) that times, on the REAL pinned artifact
(`332ea218…`):
1. CLI per-call path (spawn → instantiate → eval → teardown), n≥1000 —
   this is the number the design must beat;
2. persistent-runner packet-4 frame round-trip, n≥1M;
3. in-kernel 3-quad admit (parse+fold), n≥1M.

Gate: (1) ≥ 10x (2) and (2) − (3) ≤ 50 µs. Without this court, sub-ms is
UNVERIFIED, same status as M1.

## 4. EH/runner coupling (OD-1 dependency)

The DuckDB-linked artifact must be **new-EH uniform** (try_table/throw_ref,
zero legacy opcodes) — recipe `designs/a1prime-trytable-recipe.md`:
`-mllvm -wasm-use-legacy-eh=0` at compile AND link
(`CMAKE_CXX_FLAGS="-O3 -fwasm-exceptions -mllvm -wasm-use-legacy-eh=0"`);
opcode census is the only reliable gate (target_features is identical in both
modes — recipe §1 probe table).

**Runner**: wasmtime with `wasm_exceptions` **default-on** (crate feature
`gc` on): witnessed in `designs/wasmi-eh-research.md` verdict table
("wasmtime 48 — YES, new EH (try_table/throw) default-ON, behind crate
feature `gc`"; docs.rs `Config::wasm_exceptions`). wasmi 2.0.0 is a hard NO —
no feature bit exists and module/{export,import}.rs panic on tags
(wasmi-eh-research §Local pin receipts); the L115 receipt witnessed the
converse failure live: a tag-bearing lane artifact rejected by the wasmi host
with typed exit 95 "exceptions proposal not enabled".

**Maturity gate (A1')**: per `dual-artifact-policy-design.md:114` — rebuild
one frozen-duckdb blob set with the new-EH flag, link, witness a REAL module
load + one real frame through the packet-4 loopback driver (latin1 framing
law for Elixir drivers), 3x reproduction, typed-death clause (still-mixed EH
or host rejection → A1' dies loudly, fallback ladder A3 → B). **No zero-copy
work starts before A1' is witnessed.**

## 5. Sequenced increments, each with its falsifier

### (i) Persistent runner + stdin — EXISTS (graphlaw-run)

- L115 receipt: R1–R6 executed over the real pinned artifact, typed exit
  classes 84/90/93/95 witnessed.
- **Falsifier**: a court that sends N=10,000 packet-4 frames over one
  runner process and asserts (a) zero process respawns (single PID observed
  via a probe frame), (b) p99 frame round-trip ≤ 5 ms. If the runner respawns
  per frame or p99 blows, increment (i) is dead.

### (ii) Batch reification

Hook-side reification of a batch of act:ToolCall quads into ONE packet-4
frame (JSON array, not N frames); the kernel admits the batch in one
`gl_call` and returns per-item verdicts.
- **Falsifier**: the batch court sends 100 calls as 1 frame and asserts
  per-call cost strictly less than the 1-frame-per-call p50 of increment (i)
  on the same machine; the verdict vector must be byte-identical to the
  N-frame run (admission is per-item, batching must not change the law).

### (iii) Shared-memory zero-copy

Design (not implementation): a shmem ring between hook and runner; the hook
writes reified quads (a fixed-stride quad record: term-type u8, 32-byte
BLAKE3 or inline literal, length) directly into the ring at
capability-scoped offsets; a doorbell (eventfd/futex) wakes the runner; the
runner maps the ring window INTO the wasm linear memory via `gl_alloc`-shaped
handoff — but instead of copying, the side table `LIVE` registers the ring
window as a guest buffer with real capacity from the guest's own record.
The kernel validates the buffer contents **inside the module** — the
admission law is the same code path as stdin mode (R3's typed finding:
document-backed admission through `gl_call`, REFUSAL-04 is the
non-document-backed builtin path, `docs/refusals.md`).
- Rust wasi does not support mapping external shmem into linear memory
  directly; the honest mechanism is a **runner-side ring consumer that
  `gl_alloc`s a window and memcpy's once** — i.e. one copy, not zero. True
  zero-copy requires a wasm proposer/shared-memory proposal host (wasmtime
  `shared_memory`) — **flagged UNKNOWN, gated on a spike** (increment (iii)a:
  1-day spike proving wasmtime 48 + shared memory + custom buffer protocol;
  if it dies, increment (iii) lands as "single-copy ring", still ~0.5–1 µs
  transport class vs 1–10 µs stdin class).
- **Falsifier**: the ring court runs both transports on the same machine and
  asserts ring p99 < stdin p99 with the verdict bytes identical; plus the
  security court (§6): a hostile buffer (truncated frame, forged length,
  unknown pointer, double free, oversized claim) must produce the same typed
  refusals as stdin mode (ffi.rs side-table classes), never UB, never a
  bypass admit.

## 6. Security boundary: zero-copy must not bypass the law

The hook is untrusted-process-adjacent. Consequences, normative for any
implementation:

1. **The admission law runs INSIDE the module.** Buffer contents are data,
   never authority. A shared-memory write is an untrusted input exactly like
   a stdin frame; it admits nothing by being in memory.
2. **Side-table discipline is load-bearing** (ffi.rs header): bounds from the
   guest's `LIVE` table only; unknown ptr / length mismatch / double free →
   the existing typed error responses (`unknown_buffer_response`,
   `missing_buffer_response`, `limit_response` in `graphlaw::abi`), never
   UB. The ring design adds: offsets are capability-scoped (runner grants
   the hook a window grant id + bounds; out-of-window writes are invisible),
   and the runner re-derives frame length from the ring's own record header,
   never from a host-asserted length.
3. **No new op is added to bypass admission.** REFUSAL-04 remains the
   non-document-backed builtin path (`docs/refusals.md`); a zero-copy fast
   path that skips the SHACL/RDFC admission fold would be a laundering
   channel and is refused by design — the sql-rdf laundering corpus
   (`tests/sql_rdf_laundering_corpus.rs`,
   `receipts/sql-rdf-laundering-corpus-2026-10-22.md`) is the standing
   refusal class.
4. **Digest law unchanged**: the artifact identity pin (`332ea218…`, L115
   receipt) is the trust root; the runner refuses unpinned artifacts (exit
   84 witnessed) regardless of transport.

## 7. Standing summary

| claim | standing |
|---|---|
| persistent runner works end-to-end | WITNESSED (L115 R1–R6) |
| in-kernel fold ≤ 250 ns class | WITNESSED (209 ns p99) but SPC Cpk FAIL |
| 0.082 µs statement admission | UNVERIFIED (COURT-SPEC M1) |
| CLI path cost class | INFERRED ms-class, UNMEASURED — §3.4 court must witness |
| IPC class estimates (§3.3) | UNMEASURED class estimates |
| sub-ms end-to-end | TARGET, UNVERIFIED until §3.4 + increment (iii) courts pass |
| shared-memory in wasm linear memory | UNKNOWN (spike (iii)a required) |

Falsifier for the whole design: §3.4 court leg (1) does not beat leg (2) by
≥10x, or increment (iii)'s ring does not beat stdin on p99 — then the
zero-copy step is refused as not-worth-it and increment (ii) batch mode is
the landing zone.
