# Explanation: the EH closure story

Why the graphlaw wasm module spent a wave unhostable by any runtime, and
how it closed. This is an explanation, not a runbook — the build recipe
lives in `docs/WASM.md`, the ticket in
`docs/jira/v26.10.22/SQLRDF-24.md`, and the wave ledger in
`docs/jira/v26.10.22/RECEIPTS.md`.

## Two generations of wasm exception handling

WebAssembly has two incompatible exception-handling designs. The
**legacy** proposal uses structured `try`/`catch`/`delegate` instructions
and a single `__cpp_exception` tag. The standardized **new** EH (W3C
exception-handling proposal, Section 13) replaces them with
`try_table` — a branch-target table on a block — plus a typed tag
section. The two instruction sets can appear in one module, and that
ability is the trap: most engines either refuse the mix outright or
refuse one flavor entirely.

The toolchain sat on the wrong side of this split. clang 24's
`-fwasm-exceptions` defaulted to emitting *legacy* `try` when lowering
C++ exceptions. Nothing in the build asked for legacy — it was simply
what the compiler produced — so every object in the link chain (DuckDB
C++, libunwind, libc++abi) carried legacy instructions even though the
target feature name (`+exception-handling`) suggested the modern
proposal.

## The hosting matrix that blocked

With a legacy-flavored (later mixed-flavored) artifact, every candidate
host refused, each for its own reason:

| host | verdict | why |
|---|---|---|
| wasmi 2.0.0 | unhostable | validator rejects the Tag section; `Config` lacks any EH setter |
| node 26 (V8) | unhostable | tolerates legacy but *rejects mixed* legacy+new EH modules |
| wasmtime 48 | unhostable | legacy dropped: "legacy_exceptions feature required"; flag gone |

This is the "unhostable by any runtime" period recorded in the EH-runner
decision memo (`designs/eh-runner-decision-memo.md`): wasmi has no EH
knob at all, wasmtime 48 accepts only new EH, and V8 — the only engine
that tolerated legacy — refused the particular mix our link produced.
No mainstream Rust runtime executed legacy EH. The module linked,
validated its imports, and then failed at instantiate in every host,
each with a *different* typed error — which is what made the period
confusing: the defect looked host-shaped from each host's point of view.

The resolution of the matrix was not finding a tolerant host but
removing the tolerance requirement: a **uniform new-EH rebuild**
(`-fwasm-exceptions -mllvm -wasm-use-legacy-eh=0` on every C++
dependency, including the frozen-duckdb archive — wave row W1) produces
a module that V8 and modern wasmtime both accept. The pinned host became
wasmtime 48 with `wasm_exceptions` enabled (`graphlaw-run-eh`, feature
`runner-eh`). wasmi 2.0 stays only for the legacy pre-duckdb artifact
courts; it still cannot host the engine module, and that is now a
documented posture rather than a defect.

## The false leads worth recording

The trap that motivated the closure wave — `uninitialized element` on
the first `gl_call` — survived several plausible hypotheses. The
exoneration matrix is the epistemically valuable residue:

- **Personality-symbol heuristic (false positive).** Detecting legacy EH
  by grepping for `__gxx_wasm_personality_v0` misclassified
  `ub_duckdb_parser.cpp.obj`: 41 members reference the symbol while
  carrying 701 `try_table` and zero legacy `try`. Symbols linger
  after instructions are replaced. The fix was to gate on opcodes, not
  symbols — the staging verify now counts `try_table` (99) and legacy
  `try` (0) directly.
- **The atexit "stride bug" (misread).** Disassembly appeared to show a
  +4/+132 writer/reader skew in musl's atexit table. It did not exist:
  the musl struct layout was correct, and the apparent offset mismatch
  was operand order in the raw disassembly. The *real* defect, once
  found, was elsewhere: the global `slot` cursor was not reset when
  `head` re-attached to the builtin chunk, plus stale function-read
  offsets in the walker.
- **LTO (exonerated).** `lto=false`, a forced-codegen-units variant, and
  thin LTO all trapped identically. Optimization mode changed the trap
  flavor but never the fault class.
- **Table encoding (refuted).** Byte-compatible `flags = 0` elem
  encodings under both toolchains; `Elem[1]` active over 1..=22581 with
  no gap. The table image was intact all along.
- **gc-sections and the compat archive (exonerated).** Weakening
  `--gc-sections` did not clear the trap; `wasi_posix.c` carries only 10
  ENOSYS stubs. The dropped-table-entry hypothesis class died here.
- **V8 hosting (exonerated as cause).** V8 compiled and instantiated the
  artifact cleanly — hosting was never the defect, which closed the
  npm-artifact-format question and localized the fault to the module.

## The actual fix stack

The closure took four coordinated pieces (witnessed in
`receipts/2026-10-23-eh-closure-wave.md`, sections 7-8):

1. **Uniform new-EH rebuild.** Every C++ archive — frozen-duckdb
   (`libduckdb.a`, 88,161,726 B), libunwind candidate B
   (sha256 `6a5f79f7…`), libc++abi — built with
   `-fwasm-exceptions -mllvm -wasm-use-legacy-eh=0`. The build script's
   expected libunwind digest now pins the new-EH candidate with a typed
   FAIL naming the legacy variant, so a legacy object cannot pass the
   build silently again.
2. **Wasmtime 48 host with `wasm_exceptions`.** The `graphlaw-run-eh`
   binary (feature `runner-eh`) replaced the wasmi-bound runner as the
   engine-tier host; the EH-OK probe (W2, "EH-OK probe", exit 0) closed
   the hosting wall.
3. **`atexit.c.obj` from current wasi-libc.** The trap itself was
   `call_indirect` through table slot 0 inside `__funcs_on_exit`, the
   destructor-list runner, reached from the export epilogue. The stale
   `slot` global made cycle-2 registrations land in a fresh chunk the
   walker then drained past — indirect-calling table index 0. Replacing
   the object with the current wasi-libc member (with `finished_atexit`
   present, sha256 `a1874498…`) fixed the walker side.
4. **Reactor-contract completion in `wasm/src/lib.rs`.** The module is a
   reactor: it runs ctor/dtor cycles per command export rather than
   once at `_initialize` (which it does not export — init rides inside
   every export wrapper). Left half-implemented, that contract re-ran
   destructor registration every call and triggered the atexit defect.
   Completion: static extension registration via `.init_array`
   constructors, plus atexit-family no-op overrides, so destructors
   never run mid-module-life.

The canonical re-pin followed only after qualification:
`priv/graphlaw.wasm` blake3
`fa1558306690493249b176ec312a4f01e97798937e05cd49f2946d61a7e6d107`,
57,781,384 B, Tag(13)×1, wasm_audit PASS, lifecycle courts 3/3 GREEN.

## Why wasmi still cannot host, and the pin court posture

wasmi 2.0.0's validator rejects the Tag section and its `Config` has no
exceptions setter — this is upstream design, not configuration. The
engine module (new-EH, DuckDB-linked) therefore has exactly one Rust
host: wasmtime 48 with `wasm_exceptions`. wasmi remains the host for the
**legacy pre-duckdb artifact courts** — the small pure-Rust generation
(~6.8 MB, no tag section) — under the dual-artifact policy. The two
tiers are permanent, not transitional: the engine tier requires EH
because DuckDB C++ uses exceptions; the court tier requires wasmi
because those courts assert on the pre-duckdb generation.

The corresponding posture on the host side is that the artifact pin and
the host capability are judged together: a re-pin is witnessed (digest,
size, tag count, validation exit) only against a host that actually runs
the lifecycle courts, and a host that cannot validate the artifact
(wasmi) is documented as a tier boundary rather than patched around.
That is why the incident receipt for the mid-wave staging wipe
(`receipts/2026-10-23-staging-wipe-incident.md`) mattered beyond its
operational impact — an unreceipted destructive transition on the
staging the pin is minted from would have left the pin standing on a
substrate nobody could attest to. The law it produced (staging
transitions are receipted and coordinator-owned; lanes never mutate
`~/.frozen-duckdb`) is the same discipline the pin court enforces.

## See Also

- `docs/WASM.md` — the build recipe and current hosting statement
- `docs/jira/v26.10.22/SQLRDF-24.md` — the trap ticket, exoneration
  matrix, and root-cause convergence
- `docs/jira/v26.10.22/designs/eh-runner-decision-memo.md` — the hosting
  matrix and route decision
- `receipts/2026-10-23-eh-closure-wave.md` — the witnessed closure
- `receipts/2026-10-23-staging-wipe-incident.md` — the staging incident
  and its law
