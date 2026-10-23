# SQLRDF-24 — Artifact trap: `uninitialized element` on first `gl_call` (indirect-call vtable)

- **Id:** SQLRDF-24
- **Title:** Artifact trap: `uninitialized element` on first `gl_call` (indirect-call vtable)
- **Status:** Open
- **Base:** HEAD `dab6a73` (chore(snapshot): fleet wave safety snapshot —
  2026-10-22 annealing waves W/Z/AA/AB/K)

## Why this ticket exists

The DuckDB-linked wasm artifact traps on its FIRST `gl_call` with
`wasm trap: uninitialized element` — a `call_indirect` through a function
table slot read from linear memory that is still null at call time. The
trap site is inside the `__cxa_thread_atexit` destructor-list runner
(old-artifact function index 68925; new-EH artifact ~68950), reached from
the wasi-libc `gl_*` export epilogue (destructor run + stdio flush).
`Table[0]` is permanently null, so the trap is op-agnostic: whatever the
`gl_*` entry point, the first indirect call in the epilogue fires it. V8
and wasmtime both trap identically on the same artifact, which rules out
the host and localizes the defect to the module itself.

## Conformance target

The dual-artifact policy and the A1' new-EH recipe
([designs/a1prime-trytable-recipe.md](designs/a1prime-trytable-recipe.md),
[designs/dual-artifact-policy-design.md](designs/dual-artifact-policy-design.md))
plus the artifact re-pin discipline of SQLRDF-21 acceptance item (e)
(witnessed digest receipt in [RECEIPTS.md](RECEIPTS.md)). Capability
answers must come from the same artifact that passes the lifecycle EH
courts — no capability surface admitted from an artifact that cannot run.

## Current behavior (verified on disk / witnessed this session, 2026-10-23)

- **Trap witnessed on two artifact flavors.** The DuckDB-linked artifact
  (46,015,267 B original; 46,069,809 B new-EH rebuild, Tag×1) traps on the
  first `gl_call` with `wasm trap: uninitialized element` under both V8
  and wasmtime 48.
- **Trap site.** `call_indirect` inside the `__cxa_thread_atexit`
  destructor-list runner (old-artifact func 68925, new ~68950), reached
  from the wasi-libc export epilogue (dtors + stdio flush); `Table[0]`
  permanently null.
- **EH-flavor mixing exonerated.** A uniform new-EH libunwind candidate B
  links and the EH-OK probe passes, yet the trap persists — the trap is
  not a MIXED legacy+new EH artifact problem.
- **LTO exonerated.** `-C lto=false`, `-C lto=cgu16`-style, and thin LTO
  builds all trap — link-time optimization is not the mechanism.
- **Stack size exonerated as the trap *kind*.** Stack-size experiments
  change nothing about the trap class (`uninitialized element` is a table
  trap, not a stack-exhaustion trap).
- **V8 hosting exonerated.** V8 compiles and instantiates the artifact
  cleanly; the npm artifact-format question is closed — hosting is not
  the defect.

## Required behavior

1. **Root cause with a mechanism, not a workaround.** Identify the exact
   corrupted `Table[0]` entry or the dropped-table-entry mechanism
   (leading hypothesis: `--gc-sections` dropping table entries the
   linker considers unreachable while the data-segment vtable image
   survives, leaving the slot null), naming the entry's producer and why
   it is null at first call.
2. **An artifact that runs.** The DuckDB-linked artifact answers
   capabilities and a full duckdb session lifecycle through
   `graphlaw-run-eh` with zero traps.
3. **Lifecycle EH courts A/B green** on the fixed artifact.
4. **Witnessed re-pin** per the staged runbook (digest + build command +
   exit code in [RECEIPTS.md](RECEIPTS.md)).

## Acceptance

- (a) root cause identified: exact corrupted entry or dropped-table-entry
  mechanism named with evidence (dtor-list memory dump host receipt).
- (b) DuckDB-linked artifact answers capabilities + duckdb session
  lifecycle through `graphlaw-run-eh` with zero traps.
- (c) lifecycle EH courts A/B green.
- (d) witnessed re-pin per the staged runbook.

## Constraints

- **No artifact re-pin until (b)** — a digest may never be pinned for an
  artifact that traps.
- INDEX rows are append-only.

## Already-implemented boundary

The link itself (frozen-duckdb 2.0.2, `static=duckdb` for
wasm32-wasip1), the new-EH rebuild recipe (A1'), the EH host decision
(wasmtime 48 + `wasm_exceptions`), the EH-OK probe, and the exoneration
matrix above (EH mixing, LTO, stack size, V8 hosting) are all landed and
witnessed. This ticket does not re-ask for any of them; it asks for the
root cause and the artifact that runs.

## Progress (2026-10-23)

- **Table-encoding hypothesis REFUTED.** Byte-compatible `flags = 0`
  elem encodings under both toolchains; the final artifact's `Elem[1]`
  is active over 1..=22581 with no gap and slot 0 null by convention.
  The table image itself is intact — the null slot is not a dropped or
  re-encoded entry.
- **Personality-SYMBOL heuristic was a false positive.**
  `ub_duckdb_parser.cpp.obj`: 41 members reference
  `__gxx_wasm_personality_v0` while carrying 701 `try_table` and 0
  legacy `try`. The opcode gate replaced the symbol heuristic.
- **Staging verify now carries the opcode EH gate**: `try_table = 99`,
  legacy `try = 0` → PASS.
- **Link dead-code V4 traps**: `--gc-sections` weakened per the
  dropped-table-entry hypothesis class.
- **FIBO courts witnessed ALIVE on the real module**: TC-FIBO-03
  (SHACL refusal) and TC-FIBO-08 (COPY egress)
  (`tests/wasm_fibo_fixtures_eh.rs:365`, `:402`) — the trap manifests on
  create/insert paths (dtor-registering ops, per the dtor dump).
- **In flight**: dupe-hunt on the atexit writer/reader skew.

## Root cause CONFIRMED (2026-10-23, three-method convergence)

- Symbolized frames: `__funcs_on_exit` ← `__wasm_call_dtors` ←
  `gl_call.command_export` (exit 97, `uninitialized element`).
- Mechanism (coredump memory receipt + static read, independent methods):
  musl atexit global `slot` is not reset when `head` re-attaches to the
  builtin chunk. The module runs ctor/dtor cycles PER COMMAND EXPORT
  (reactor; no `_initialize` export — init rides in func 28 inside every
  export wrapper). Cycle-2 registrations pass the `slot == 32` fullness
  test against the stale global, calloc a fresh chunk ahead of the
  partially-empty builtin, and register there; the walker then advances to
  the builtin chunk with `slot = 32` and drains its zero slots —
  `call_indirect` through table index 0.
- Exoneration matrix complete: EH flavor, LTO, table encoding, gc-sections,
  compat archive, atexit stride misread (musl layout correct; the +4/+132
  "skew" was operand-order in the raw disassembly).
- Minimal fix (one line, atexit.c): `if (!head) { head = &builtin; slot = 0; }`
  — candidate object compiled and store-forward-verified
  (/tmp/foe-static/atexit_fixed.o). Executor lane is relinking against a
  fixed libc.a; qualification courts (FIBO matrix + lifecycle A/B/C) are
  the acceptance gate before the witnessed re-pin.
